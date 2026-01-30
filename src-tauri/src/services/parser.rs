//! Markdown/LaTeX 解析引擎
//! 使用 pulldown-cmark 的 Pull 模式解析 Markdown 文档，提取题目信息

use pulldown_cmark::{Event, Parser, Tag, TagEnd, HeadingLevel};
use regex::Regex;
use std::collections::HashMap;

/// 题目类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum QuestionType {
    MultipleChoice,
    FillInBlank,
    TrueOrFalse,
    ShortAnswer,
    Essay,
    Unknown,
}

/// 题目结构
#[derive(Debug, Clone)]
pub struct Question {
    pub id: String,
    pub number: Option<u32>,
    pub question_type: QuestionType,
    pub content: String,
    pub options: Vec<String>,
    pub answer: Option<String>,
    pub explanation: Option<String>,
    pub difficulty: Option<u32>,
    pub knowledge_tags: Vec<String>,
}

/// 解析的文档结构
#[derive(Debug)]
pub struct ParsedDocument {
    pub title: String,
    pub questions: Vec<Question>,
    pub metadata: DocumentMetadata,
}

/// 文档元数据
#[derive(Debug, Default)]
pub struct DocumentMetadata {
    pub source_path: Option<String>,
    pub total_questions: usize,
    pub parsed_at: chrono::DateTime<chrono::Utc>,
}

/// LaTeX 公式提取结果
#[derive(Debug, Clone)]
pub struct LatexFormula {
    pub formula: String,
    pub is_block: bool,
    pub position: usize,
}

/// Markdown 解析器配置
#[derive(Debug, Default)]
pub struct MarkdownParserConfig {
    pub recognize_question_numbers: bool,
    pub extract_options: bool,
    pub preserve_latex: bool,
    pub question_patterns: Vec<Regex>,
}

impl MarkdownParserConfig {
    pub fn new() -> Self {
        Self {
            recognize_question_numbers: true,
            extract_options: true,
            preserve_latex: true,
            question_patterns: vec![
                Regex::new(r"^(\d+)\.").unwrap(),
                Regex::new(r"^（(\d+)）").unwrap(),
                Regex::new(r"^(\d+)\.(\d+)").unwrap(),
            ],
        }
    }
}

/// Markdown 解析器
#[derive(Debug)]
pub struct MarkdownParser {
    config: MarkdownParserConfig,
    question_counter: u32,
    current_question: Option<Question>,
    questions: Vec<Question>,
    title: String,
    current_content: String,
    current_options: Vec<String>,
    in_options_list: bool,
    attributes: HashMap<String, String>,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self::with_config(MarkdownParserConfig::new())
    }

    pub fn with_config(config: MarkdownParserConfig) -> Self {
        Self {
            config,
            question_counter: 0,
            current_question: None,
            questions: Vec::new(),
            title: String::new(),
            current_content: String::new(),
            current_options: Vec::new(),
            in_options_list: false,
            attributes: HashMap::new(),
        }
    }

    pub fn parse(&mut self, content: &str) -> ParsedDocument {
        let parser = Parser::new(content);
        self.reset();
        
        for event in parser {
            self.process_event(event);
        }
        
        self.save_current_question();
        self.build_parsed_document()
    }
    
    fn reset(&mut self) {
        self.questions.clear();
        self.question_counter = 0;
        self.current_question = None;
        self.title.clear();
        self.current_content.clear();
        self.current_options.clear();
        self.in_options_list = false;
        self.attributes.clear();
    }

    fn process_event(&mut self, event: Event) {
        match event {
            Event::Start(Tag::Heading { level, id: _, classes: _, attrs: _ }) => {
                self.save_current_question();
                
                if level == HeadingLevel::H1 {
                    self.current_content.clear();
                } else if level == HeadingLevel::H2 {
                    self.start_new_question();
                }
            }
            Event::End(TagEnd::Heading(level)) => {
                if level == HeadingLevel::H1 && !self.current_content.is_empty() {
                    self.title = self.current_content.trim().to_string();
                    self.current_content.clear();
                } else if level == HeadingLevel::H2 {
                    self.save_current_question();
                }
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {}
            Event::Start(Tag::List(_)) => {
                if self.current_question.is_some() {
                    self.in_options_list = true;
                    self.current_options.clear();
                }
            }
            Event::End(TagEnd::List(_)) => {
                if self.in_options_list {
                    self.in_options_list = false;
                    if let Some(ref mut q) = self.current_question {
                        q.options = self.current_options.clone();
                    }
                }
            }
            Event::Start(Tag::Item) => {}
            Event::End(TagEnd::Item) => {}
            Event::Text(text) | Event::Code(text) => {
                self.current_content.push_str(&text);
            }
            Event::SoftBreak => {
                self.current_content.push(' ');
            }
            Event::HardBreak => {
                self.current_content.push('\n');
            }
            _ => {}
        }
    }

    fn start_new_question(&mut self) {
        self.save_current_question();
        self.question_counter += 1;
        self.current_question = Some(Question {
            id: format!("q{}", self.question_counter),
            number: Some(self.question_counter),
            question_type: QuestionType::Unknown,
            content: String::new(),
            options: Vec::new(),
            answer: None,
            explanation: None,
            difficulty: None,
            knowledge_tags: Vec::new(),
        });
        self.current_content.clear();
        self.current_options.clear();
        self.attributes.clear();
    }

    fn save_current_question(&mut self) {
        if let Some(question) = self.current_question.take() {
            if !question.content.trim().is_empty() {
                let content = question.content.clone();
                let question_type = self.detect_question_type(&content, &question.options);
                
                let mut question = question;
                question.question_type = question_type;
                
                for (key, value) in &self.attributes {
                    match key.as_str() {
                        "answer" => question.answer = Some(value.clone()),
                        "explanation" | "解析" => question.explanation = Some(value.clone()),
                        "difficulty" | "难度" => {
                            if let Ok(d) = value.parse::<u32>() {
                                question.difficulty = Some(d.clamp(1, 5));
                            }
                        }
                        _ => {}
                    }
                }
                
                self.questions.push(question);
            }
        }
    }

    fn build_parsed_document(&mut self) -> ParsedDocument {
        ParsedDocument {
            title: self.title.clone(),
            questions: self.questions.clone(),
            metadata: DocumentMetadata {
                source_path: None,
                total_questions: self.questions.len(),
                parsed_at: chrono::Utc::now(),
            },
        }
    }

    fn detect_question_type(&self, content: &str, options: &[String]) -> QuestionType {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("对") || content_lower.contains("错") ||
           content_lower.contains("正确") || content_lower.contains("错误") ||
           content_lower.contains("true") || content_lower.contains("false") {
            return QuestionType::TrueOrFalse;
        }

        if !options.is_empty() && options.len() >= 2 {
            let has_option_markers = options.iter().all(|opt| {
                let opt_trimmed = opt.trim();
                opt_trimmed.starts_with('A') || opt_trimmed.starts_with('B') ||
                opt_trimmed.starts_with('C') || opt_trimmed.starts_with('D') ||
                opt_trimmed.starts_with('a') || opt_trimmed.starts_with('b') ||
                opt_trimmed.starts_with('c') || opt_trimmed.starts_with('d')
            });
            if has_option_markers {
                return QuestionType::MultipleChoice;
            }
        }

        if content.contains("___") || content.contains("____") ||
           content_lower.contains("填空") || Regex::new(r"\(\s*\)").unwrap().is_match(content) {
            return QuestionType::FillInBlank;
        }

        if content_lower.contains("简述") || content_lower.contains("说明") ||
           content_lower.contains("回答") || content_lower.contains("解释") {
            return QuestionType::ShortAnswer;
        }

        if content_lower.contains("论述") || content_lower.contains("分析") ||
           content_lower.contains("探讨") || content_lower.contains("谈谈") {
            return QuestionType::Essay;
        }

        QuestionType::Unknown
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

/// 提取 Markdown 内容中的 LaTeX 公式
pub fn extract_latex(content: &str) -> Vec<LatexFormula> {
    let mut formulas = Vec::new();
    
    let block_pattern = Regex::new(r"\$\$([^$]+)\$\$").unwrap();
    for cap in block_pattern.captures_iter(content) {
        if let Some(mat) = cap.get(1) {
            formulas.push(LatexFormula {
                formula: mat.as_str().to_string(),
                is_block: true,
                position: mat.start(),
            });
        }
    }
    
    let inline_pattern = Regex::new(r"\$([^$\n]+)\$").unwrap();
    for cap in inline_pattern.captures_iter(content) {
        if let Some(mat) = cap.get(1) {
            let is_already_matched = formulas.iter().any(|f| {
                f.formula == mat.as_str() && f.is_block
            });
            if !is_already_matched {
                formulas.push(LatexFormula {
                    formula: mat.as_str().to_string(),
                    is_block: false,
                    position: mat.start(),
                });
            }
        }
    }
    
    formulas.sort_by_key(|f| f.position);
    formulas
}

/// 清理 LaTeX 内容，保留源码但规范化格式
pub fn clean_latex_content(content: &str) -> String {
    let mut result = content.to_string();
    
    let inline_pattern = Regex::new(r"\$(\s*)([^\$\n]+)(\s*)\$").unwrap();
    result = inline_pattern.replace_all(&result, |caps: &regex::Captures| {
        format!("${}$", caps.get(2).unwrap().as_str().trim())
    }).to_string();
    
    let block_pattern = Regex::new(r"\$\$\s*([^\$]+)\s*\$\$").unwrap();
    result = block_pattern.replace_all(&result, |caps: &regex::Captures| {
        format!("$$\n{}\n$$", caps.get(1).unwrap().as_str().trim())
    }).to_string();
    
    result
}

/// 解析选择题选项
pub fn parse_multiple_choice_options(content: &str) -> Vec<String> {
    let mut options = Vec::new();
    
    let pattern = Regex::new(r"(?:^|\n)\s*(?:[A-Da-d]|[（(]?[A-Da-d][）)]?)\s*[.。:：)\s]\s*(.+)").unwrap();
    
    for cap in pattern.captures_iter(content) {
        if let Some(mat) = cap.get(1) {
            options.push(mat.as_str().trim().to_string());
        }
    }
    
    options
}

/// 从文本中提取题目编号
pub fn extract_question_number(text: &str) -> Option<u32> {
    let patterns = vec![
        (r"^(\d+)\.", 1),
        (r"^（(\d+)）", 1),
        (r"^(\d+)\.(\d+)", 1),
    ];
    
    for (pattern, group) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(text) {
                if let Some(mat) = cap.get(group) {
                    return mat.as_str().parse().ok();
                }
            }
        }
    }
    
    None
}

/// 简单解析 Markdown 内容
pub fn simple_parse(content: &str) -> ParsedDocument {
    let mut parser = MarkdownParser::new();
    parser.parse(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_latex() {
        let content = r#"公式 $E=mc^2$，另一个公式 $$\int_0^\infty e^{-x^2} dx$$"#;
        let formulas = extract_latex(content);
        
        assert_eq!(formulas.len(), 2);
        assert_eq!(formulas[0].formula, "E=mc^2");
        assert!(!formulas[0].is_block);
        assert!(formulas[1].is_block);
    }

    #[test]
    fn test_clean_latex_content() {
        let content = r#"$ x + y $ 和 $$ a^2 + b^2 $$"#;
        let cleaned = clean_latex_content(content);
        
        assert!(cleaned.contains("$x + y$"));
        assert!(cleaned.contains("$$\na^2 + b^2\n$$"));
    }

    #[test]
    fn test_parse_multiple_choice() {
        let content = "A. 选项一\nB. 选项二";
        let options = parse_multiple_choice_options(content);
        
        assert_eq!(options.len(), 2);
        assert_eq!(options[0], "选项一");
    }

    #[test]
    fn test_extract_question_number() {
        assert_eq!(extract_question_number("1. 这是一道题"), Some(1));
        assert_eq!(extract_question_number("（2）这是另一道题"), Some(2));
    }

    #[test]
    fn test_simple_parse() {
        let content = r#"# 数学试卷

## 1. 选择题
下列关于函数的说法正确的是

A. $f(x) = x^2$ 是偶函数
B. $g(x) = x^3$ 是奇函数

## 2. 填空题
$\int_0^1 x^2 dx = \frac{1}{3}$"#;
        
        let result = simple_parse(content);
        
        assert_eq!(result.title, "数学试卷");
        assert_eq!(result.questions.len(), 2);
        assert_eq!(result.questions[0].number, Some(1));
        assert_eq!(result.questions[1].number, Some(2));
    }
}
