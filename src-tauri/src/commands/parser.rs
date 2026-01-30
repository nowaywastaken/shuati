//! Markdown 解析相关 Tauri 命令
//! 提供给前端调用的解析接口

use crate::services::parser::{
    extract_latex, clean_latex_content, parse_multiple_choice_options,
    extract_question_number, simple_parse, MarkdownParser, LatexFormula,
    ParsedDocument, Question, QuestionType, DocumentMetadata,
};
use serde::{Deserialize, Serialize};
use tauri::State;

/// 解析 Markdown 文档命令
#[tauri::command]
pub async fn parse_markdown_document(
    content: String,
    file_path: Option<String>,
) -> Result<ParsedDocument, String> {
    let mut parser = MarkdownParser::new();
    let mut result = parser.parse(&content);
    
    // 设置文件路径
    result.metadata.source_path = file_path;
    
    Ok(result)
}

/// 从文件路径读取并解析 Markdown 文档
#[tauri::command]
pub async fn extract_questions_from_file(
    file_path: String,
) -> Result<ParsedDocument, String> {
    use std::fs;
    
    // 检查文件是否存在
    if !std::path::Path::new(&file_path).exists() {
        return Err(format!("文件不存在: {}", file_path));
    }
    
    // 读取文件内容
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 解析文档
    let mut parser = MarkdownParser::new();
    let mut result = parser.parse(&content);
    
    // 设置文件路径
    result.metadata.source_path = Some(file_path);
    
    Ok(result)
}

/// 提取 LaTeX 公式命令
#[tauri::command]
pub async fn extract_latex_content(
    content: String,
) -> Result<Vec<LatexFormulaDto>, String> {
    let formulas = extract_latex(&content);
    let dtos: Vec<LatexFormulaDto> = formulas.into_iter().map(Into::into).collect();
    Ok(dtos)
}

/// 清理 LaTeX 内容命令
#[tauri::command]
pub async fn clean_latex(
    content: String,
) -> Result<String, String> {
    Ok(clean_latex_content(&content))
}

/// 解析选择题选项命令
#[tauri::command]
pub async fn parse_options(
    content: String,
) -> Result<Vec<String>, String> {
    Ok(parse_multiple_choice_options(&content))
}

/// 提取题目编号命令
#[tauri::command]
pub async fn get_question_number(
    text: String,
) -> Result<Option<u32>, String> {
    Ok(extract_question_number(&text))
}

/// 批量解析多个 Markdown 文档
#[tauri::command]
pub async fn parse_multiple_documents(
    documents: Vec<DocumentInput>,
) -> Result<Vec<ParsedDocument>, String> {
    let mut results = Vec::new();
    
    for doc in documents {
        let mut parser = MarkdownParser::new();
        let mut result = parser.parse(&doc.content);
        result.metadata.source_path = doc.file_path;
        results.push(result);
    }
    
    Ok(results)
}

/// 解析单个题目
#[tauri::command]
pub async fn parse_question(
    content: String,
) -> Result<QuestionDto, String> {
    let full_content = format!("# 题目\n{}", content);
    let result = simple_parse(&full_content);
    
    if result.questions.is_empty() {
        return Err("未能在内容中识别到题目".to_string());
    }
    
    Ok(result.questions[0].clone().into())
}

/// 验证 Markdown 格式
#[tauri::command]
pub async fn validate_markdown(
    content: String,
) -> Result<MarkdownValidationResult, String> {
    let parser = MarkdownParser::new();
    let mut has_heading = false;
    let mut has_latex = false;
    let mut question_count = 0;
    
    // 基本检查
    for event in pulldown_cmark::Parser::new(&content) {
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading { level: _, id: _, classes: _, attrs: _ }) => {
                has_heading = true;
            }
            pulldown_cmark::Event::Text(_) | pulldown_cmark::Event::Code(_) => {
                let dollar_sign = '$';
                if content.contains(dollar_sign) {
                    has_latex = true;
                }
            }
            _ => {}
        }
    }
    
    // 简单估算题目数量
    let mut question_parser = MarkdownParser::new();
    let parsed = question_parser.parse(&content);
    question_count = parsed.questions.len();
    
    Ok(MarkdownValidationResult {
        is_valid: !content.trim().is_empty(),
        has_heading,
        has_latex,
        estimated_question_count: question_count,
        issues: Vec::new(),
    })
}

// ==================== DTO 类型定义 ====================

/// LaTeX 公式 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct LatexFormulaDto {
    pub formula: String,
    pub is_block: bool,
    pub position: usize,
}

impl From<LatexFormula> for LatexFormulaDto {
    fn from(f: LatexFormula) -> Self {
        Self {
            formula: f.formula,
            is_block: f.is_block,
            position: f.position,
        }
    }
}

/// 题目 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionDto {
    pub id: String,
    pub number: Option<u32>,
    pub question_type: String,
    pub content: String,
    pub options: Vec<String>,
    pub answer: Option<String>,
    pub explanation: Option<String>,
    pub difficulty: Option<u32>,
    pub knowledge_tags: Vec<String>,
}

impl From<Question> for QuestionDto {
    fn from(q: Question) -> Self {
        Self {
            id: q.id,
            number: q.number,
            question_type: match q.question_type {
                QuestionType::MultipleChoice => "multiple_choice".to_string(),
                QuestionType::FillInBlank => "fill_in_blank".to_string(),
                QuestionType::TrueOrFalse => "true_or_false".to_string(),
                QuestionType::ShortAnswer => "short_answer".to_string(),
                QuestionType::Essay => "essay".to_string(),
                QuestionType::Unknown => "unknown".to_string(),
            },
            content: q.content,
            options: q.options,
            answer: q.answer,
            explanation: q.explanation,
            difficulty: q.difficulty,
            knowledge_tags: q.knowledge_tags,
        }
    }
}

/// 解析文档 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedDocumentDto {
    pub title: String,
    pub questions: Vec<QuestionDto>,
    pub total_questions: usize,
    pub source_path: Option<String>,
    pub parsed_at: String,
}

/// 文档输入 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentInput {
    pub content: String,
    pub file_path: Option<String>,
}

/// Markdown 验证结果
#[derive(Debug, Serialize, Deserialize)]
pub struct MarkdownValidationResult {
    pub is_valid: bool,
    pub has_heading: bool,
    pub has_latex: bool,
    pub estimated_question_count: usize,
    pub issues: Vec<String>,
}

/// 解析器状态（用于存储持久化配置）
#[derive(Debug, Clone, Default)]
pub struct ParserState {
    pub default_options_extraction: bool,
    pub preserve_latex: bool,
    pub auto_detect_type: bool,
}

/// 获取解析器状态
#[tauri::command]
pub async fn get_parser_state(
    state: State<'_, ParserState>,
) -> Result<ParserState, String> {
    Ok(state.inner().clone())
}

/// 更新解析器配置
#[tauri::command]
pub async fn update_parser_config(
    state: State<'_, ParserState>,
    options_extraction: Option<bool>,
    preserve_latex: Option<bool>,
    auto_detect_type: Option<bool>,
) -> Result<(), String> {
    let mut s = state.inner().clone();
    if let Some(v) = options_extraction {
        s.default_options_extraction = v;
    }
    if let Some(v) = preserve_latex {
        s.preserve_latex = v;
    }
    if let Some(v) = auto_detect_type {
        s.auto_detect_type = v;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::parser::MarkdownParser;

    #[tokio::test]
    async fn test_parse_markdown_document() {
        let content = r#"# 测试试卷

## 1. 题目内容
这是一道选择题

A. 选项 A
B. 选项 B

## 2. 另一题
这是填空题 _____"#.to_string();
        
        let result = parse_markdown_document(content, None).await;
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.title, "测试试卷");
        assert_eq!(doc.questions.len(), 2);
    }

    #[tokio::test]
    async fn test_extract_latex_content() {
        let content = r#"公式 $E=mc^2$ 和 $$\sum_{i=1}^n i = \frac{n(n+1)}{2}$$"#.to_string();
        
        let result = extract_latex_content(content).await;
        assert!(result.is_ok());
        let formulas = result.unwrap();
        assert_eq!(formulas.len(), 2);
    }

    #[tokio::test]
    async fn test_validate_markdown() {
        let content = r#"# 标题

这是一段内容"#.to_string();
        
        let result = validate_markdown(content).await;
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid);
        assert!(validation.has_heading);
    }
}
