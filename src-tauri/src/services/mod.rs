// 服务模块
// 提供核心业务逻辑服务

pub mod llama;
pub mod parser;
pub mod database;

pub use llama::{
    LlamaConfig,
    LlamaSidecar,
    ModelSize,
    InferenceRequest,
    InferenceResponse,
    InferenceQueue,
    QuestionTransformationPrompt,
    TransformedQuestion,
    FillInBlankInput,
    ProgressUpdate,
    ServerStatus,
    ChatMessage,
    get_sidecar_path,
    get_platform_binary_name,
    build_server_args,
    get_model_path_by_size,
    get_config_by_model_size,
    check_and_switch_model,
};

pub use parser::{
    MarkdownParser,
    MarkdownParserConfig,
    Question,
    QuestionType,
    ParsedDocument,
    DocumentMetadata,
    LatexFormula,
    extract_latex,
    clean_latex_content,
    parse_multiple_choice_options,
    extract_question_number,
    simple_parse,
};

pub use database::{
    DatabaseService,
    Problem,
    ProblemSet,
    UserProgress,
    WrongNote,
    ProgressStats,
};
