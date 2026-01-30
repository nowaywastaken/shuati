#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod services;
mod commands;

use services::database::DatabaseService;
use std::sync::Mutex;

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

    tauri::Builder::default()
        .manage(AppState { version, platform })
        .manage(db_state)
        .manage(ParserState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_version,
            get_platform,
            parse_markdown,
        ])
        .invoke_handler(tauri::generate_handler![
            // 数据库命令
            commands::init_database,
            commands::create_problem_set,
            commands::get_problem_sets,
            commands::get_problem_set,
            commands::add_problems,
            commands::get_problems,
            commands::search_problems,
            commands::get_problem,
            commands::get_problems_by_tag,
            commands::get_problems_by_difficulty,
            commands::record_answer,
            commands::record_skip,
            commands::get_problem_set_progress,
            commands::get_wrong_notes,
            commands::increment_review_count,
            commands::delete_wrong_note,
            commands::update_wrong_note,
            // 解析命令
            commands::parse_markdown_document,
            commands::extract_questions_from_file,
            commands::extract_latex_content,
            commands::clean_latex,
            commands::parse_options,
            commands::get_question_number,
            commands::parse_multiple_documents,
            commands::parse_question,
            commands::validate_markdown,
            commands::get_parser_state,
            commands::update_parser_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
