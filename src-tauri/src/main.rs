#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod services;
mod commands;

use services::database::DatabaseService;
use services::llama::{LlamaSidecar, LlamaConfig};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;

struct AppState {
    version: String,
    platform: String,
}

struct DbState(Mutex<DatabaseService>);

struct ParserState {
    default_options_extraction: bool,
    preserve_latex: bool,
    auto_detect_type: bool,
}

impl Default for ParserState {
    fn default() -> Self {
        Self {
            default_options_extraction: true,
            preserve_latex: false,
            auto_detect_type: true,
        }
    }
}

/// Llama 状态包装器
struct LlamaState(TokioMutex<commands::llama::LlamaState>);

impl LlamaState {
    fn new() -> Self {
        let config = LlamaConfig::default();
        let sidecar = Arc::new(TokioMutex::new(LlamaSidecar::new(config)));
        let state = commands::llama::LlamaState::new(sidecar);
        Self(TokioMutex::new(state))
    }
}

#[tauri::command]
fn greet(name: String, state: tauri::State<'_, AppState>) -> String {
    format!("欢迎使用刷题神器, {}! 版本: {}, 平台: {}", name, state.version, state.platform)
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

#[tauri::command]
fn parse_markdown(content: String) -> String {
    use pulldown_cmark::{Parser, Options, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

fn main() {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let platform = std::env::consts::OS.to_string();

    // 初始化数据库
    let db = DatabaseService::new().expect("Failed to initialize database");
    let db_state = DbState(Mutex::new(db));

    // 初始化 Llama 状态
    let llama_state = LlamaState::new();

    tauri::Builder::default()
        .manage(AppState { version, platform })
        .manage(db_state)
        .manage(ParserState::default())
        .manage(llama_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_version,
            get_platform,
            parse_markdown,
        ])
        .invoke_handler(tauri::generate_handler![
            // 数据库命令
            commands::database::init_database,
            commands::database::create_problem_set,
            commands::database::get_problem_sets,
            commands::database::get_problem_set,
            commands::database::add_problems,
            commands::database::get_problems,
            commands::database::search_problems,
            commands::database::get_problem,
            commands::database::get_problems_by_tag,
            commands::database::get_problems_by_difficulty,
            commands::database::record_answer,
            commands::database::record_skip,
            commands::database::get_problem_set_progress,
            commands::database::get_wrong_notes,
            commands::database::increment_review_count,
            commands::database::delete_wrong_note,
            commands::database::update_wrong_note,
            // Llama 命令
            commands::llama::start_llama_server,
            commands::llama::stop_llama_server,
            commands::llama::is_llama_server_healthy,
            commands::llama::get_llama_server_status,
            commands::llama::transform_fill_in_blank,
            commands::llama::generate_distractors,
            commands::llama::generate_explanation,
            commands::llama::extract_knowledge_tags,
            commands::llama::chat_complete,
            commands::llama::get_text_embedding,
            commands::llama::batch_transform_questions,
            commands::llama::switch_model,
            // 解析命令
            commands::parser::parse_markdown_document,
            commands::parser::extract_questions_from_file,
            commands::parser::extract_latex_content,
            commands::parser::clean_latex,
            commands::parser::parse_options,
            commands::parser::get_question_number,
            commands::parser::parse_multiple_documents,
            commands::parser::parse_question,
            commands::parser::validate_markdown,
            commands::parser::get_parser_state,
            commands::parser::update_parser_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
