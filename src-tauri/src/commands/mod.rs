// Tauri 命令模块
// 提供供前端调用的命令接口

pub mod llama;
pub mod parser;
pub mod database;

pub use llama::{
    init_llama_service,
    start_llama_server,
    stop_llama_server,
    is_llama_server_healthy,
    get_llama_server_status,
    transform_fill_in_blank,
    generate_distractors,
    generate_explanation,
    extract_knowledge_tags,
    chat_complete,
    get_text_embedding,
    batch_transform_questions,
    switch_model,
    LlamaState,
    ServerStatusDto,
    TransformedQuestionDto,
    ChatMessageDto,
    ProgressUpdateDto,
};

pub use parser::{
    parse_markdown_document,
    extract_questions_from_file,
    extract_latex_content,
    clean_latex,
    parse_options,
    get_question_number,
    parse_multiple_documents,
    parse_question,
    validate_markdown,
    get_parser_state,
    update_parser_config,
    ParserState,
    DocumentInput,
    QuestionDto,
    ParsedDocumentDto,
    LatexFormulaDto,
    MarkdownValidationResult,
};

pub use database::{
    init_database,
    create_problem_set,
    get_problem_sets,
    get_problem_set,
    add_problems,
    get_problems,
    search_problems,
    get_problem,
    get_problems_by_tag,
    get_problems_by_difficulty,
    record_answer,
    record_skip,
    get_problem_set_progress,
    get_wrong_notes,
    increment_review_count,
    delete_wrong_note,
    update_wrong_note,
    DbState,
    QuestionDto as DbQuestionDto,
    ProblemDto,
    ProblemSetDto,
    UserProgressDto,
    WrongNoteDto,
    ProgressStatsDto,
};
