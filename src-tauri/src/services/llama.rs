//! Llama.cpp 推理服务模块
//! 提供 AI 模型推理能力，包括 sidecar 进程管理、任务队列和提示词工程

use anyhow::{Error, Result};
use async_stream::stream;
use futures::stream::Stream;
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::{timeout, Duration};

/// 模型尺寸枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModelSize {
    Small,   // 1.5B (Qwen 2.5) - 快速响应，省电模式
    Medium,  // 3B (Llama 3.2) - 均衡性能
    Large,   // 7B/8B (Llama 3.1) - 高质量生成
}

/// Llama 配置
#[derive(Debug, Clone)]
pub struct LlamaConfig {
    pub model_path: PathBuf,
    pub context_size: u32,        // 默认 4096
    pub batch_size: u32,          // 默认 512
    pub gpu_layers: i32,          // -1 表示全 GPU 卸载
    pub use_flash_attn: bool,
    pub threads: u32,
    pub port: u16,                // 推理服务端口
    pub temp: f32,                // 温度参数
    pub repeat_penalty: f32,      // 重复惩罚
}

impl Default for LlamaConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            context_size: 4096,
            batch_size: 512,
            gpu_layers: -1,
            use_flash_attn: true,
            threads: 8,
            port: 8080,
            temp: 0.7,
            repeat_penalty: 1.1,
        }
    }
}

/// 推理请求
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stop_tokens: Vec<String>,
    pub stream: bool,
}

/// 推理响应
#[derive(Debug, Clone)]
pub struct InferenceResponse {
    pub text: String,
    pub tokens_generated: u32,
    pub inference_time_ms: u64,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,  // "system", "user", "assistant"
    pub content: String,
}

/// 服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub is_running: bool,
    pub server_url: String,
    pub model_path: String,
    pub port: u16,
    pub model_size: ModelSize,
    pub gpu_layers: i32,
    pub memory_usage: Option<u64>,
}

/// 健康检查响应
#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
}

/// Tokenize 响应
#[derive(Debug, Deserialize)]
struct TokenizeResponse {
    tokens: Vec<i32>,
}

/// Completion 请求
#[derive(Debug, Serialize)]
struct CompletionRequest {
    prompt: String,
    n_predict: u32,
    temperature: f32,
    stop: Vec<String>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_prompt: Option<bool>,
}

/// Completion 响应
#[derive(Debug, Deserialize)]
struct CompletionResponse {
    content: String,
    generation_settings: Option<GenerationSettings>,
    timings: Option<Timings>,
}

#[derive(Debug, Deserialize)]
struct GenerationSettings {
    n_predict: u32,
    temperature: f32,
    #[serde(rename = "repeatPenalty")]
    repeat_penalty: f32,
}

#[derive(Debug, Deserialize)]
struct Timings {
    #[serde(rename = "predicted_per_token_ms")]
    predicted_per_token_ms: f32,
    #[serde(rename = "totalpredicted_per_token_ms")]
    total_predicted_per_token_ms: f32,
}

/// 流式响应
#[derive(Debug, Deserialize)]
struct StreamResponse {
    content: String,
    stop: bool,
    #[serde(rename = "timings")]
    timings: Option<Timings>,
}

/// 获取 sidecar 路径
pub fn get_sidecar_path(app: &AppHandle, binary_name: &str) -> Result<PathBuf, Error> {
    let target_os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let sidecar_dir = app
        .path_resolver()
        .resource_dir()?
        .join("sidecars");

    let extension = if target_os == "windows" { ".exe" } else { "" };
    let file_name = format!("{}-{}-{}", binary_name, target_os, arch);

    Ok(sidecar_dir.join(format!("{}{}", file_name, extension)))
}

/// 获取平台特定的二进制名称
pub fn get_platform_binary_name() -> &'static str {
    let target_os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (target_os, arch) {
        ("macos", "aarch64") => "llama-server-aarch64-apple-darwin",
        ("macos", "x86_64") => "llama-server-x86_64-apple-darwin",
        ("windows", "x86_64") => "llama-server-x86_64-pc-windows-msvc.exe",
        ("linux", "x86_64") => "llama-server-x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "llama-server-aarch64-unknown-linux-gnu",
        _ => "llama-server",
    }
}

/// 构建 llama-server 启动参数
pub fn build_server_args(config: &LlamaConfig) -> Vec<String> {
    let mut args = vec![
        "--model".to_string(),
        config.model_path.to_string_lossy().to_string(),
        "--host".to_string(),
        "127.0.0.1".to_string(),
        "--port".to_string(),
        config.port.to_string(),
        "--ctx-size".to_string(),
        config.context_size.to_string(),
        "--batch-size".to_string(),
        config.batch_size.to_string(),
        "--n-gpu-layers".to_string(),
        config.gpu_layers.to_string(),
        "--threads".to_string(),
        config.threads.to_string(),
        "--temp".to_string(),
        config.temp.to_string(),
        "--repeat-penalty".to_string(),
        config.repeat_penalty.to_string(),
        "--no-mmap".to_string(),  // 禁用内存映射，避免 macOS 上的问题
    ];

    if config.use_flash_attn {
        args.push("--flash-attn".to_string());
    }

    args
}

/// Llama Sidecar 进程管理器
#[derive(Clone)]
pub struct LlamaSidecar {
    config: Arc<Mutex<LlamaConfig>>,
    child_process: Arc<Mutex<Option<Child>>>,
    server_url: Arc<Mutex<String>>,
    http_client: Arc<reqwest::Client>,
}

impl LlamaSidecar {
    /// 创建新的 LlamaSidecar 实例
    pub fn new(config: LlamaConfig) -> Self {
        let port = config.port;
        Self {
            config: Arc::new(Mutex::new(config)),
            child_process: Arc::new(Mutex::new(None)),
            server_url: Arc::new(Mutex::new(format!("http://127.0.0.1:{}", port))),
            http_client: Arc::new(reqwest::Client::new()),
        }
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> LlamaConfig {
        self.config.lock().await.clone()
    }

    /// 获取服务器 URL
    pub async fn get_server_url(&self) -> String {
        self.server_url.lock().await.clone()
    }

    /// 启动 llama-server 进程
    pub async fn start(&mut self, binary_path: &Path) -> Result<(), Error> {
        // 检查进程是否已在运行
        if self.is_running().await? {
            return Ok(());
        }

        // 检查二进制文件是否存在
        if !binary_path.exists() {
            return Err(Error::msg(format!(
                "Llama binary not found at: {}",
                binary_path.display()
            )));
        }

        let config = self.config.lock().await.clone();
        let args = build_server_args(&config);

        // 设置环境变量以优化性能
        let mut env_vars = std::env::vars().collect::<Vec<_>>();
        
        // macOS Metal 优化
        if std::env::consts::OS == "macos" {
            // 设置 Metal 性能模式
            env_vars.push(("METAL_DEVICE_WRAPPER_TYPE".to_string(), "1".to_string()));
        }

        // 启动进程
        let mut child = Command::new(binary_path)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(env_vars)
            .spawn()?;

        // 等待进程启动并健康检查
        *self.child_process.lock().await = Some(child);
        
        // 等待服务器就绪
        self.wait_for_healthy(60).await?;

        Ok(())
    }

    /// 停止 llama-server 进程
    pub async fn stop(&mut self) -> Result<(), Error> {
        let mut child_guard = self.child_process.lock().await;
        
        if let Some(child) = child_guard.take() {
            // 发送 SIGTERM 信号
            child.kill()?;
            
            // 等待进程退出
            let _ = child.wait()?;
        }

        Ok(())
    }

    /// 检查进程是否在运行
    pub async fn is_running(&self) -> Result<bool, Error> {
        let child_guard = self.child_process.lock().await;
        
        if let Some(child) = child_guard.as_ref() {
            match child.try_wait() {
                Ok(Some(_)) => return Ok(false),  // 进程已退出
                Ok(None) => {
                    // 检查健康状态
                    return self.is_healthy().await;
                }
                Err(_) => return Ok(false),
            }
        }
        
        Ok(false)
    }

    /// 健康检查
    pub async fn is_healthy(&self) -> Result<bool, Error> {
        let url = format!("{}/health", self.get_server_url().await);
        
        match self.http_client.get(&url).send().await {
            Ok(resp) => {
                if resp.status() == 200 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// 等待服务器健康
    pub async fn wait_for_healthy(&self, max_wait_secs: u64) -> Result<(), Error> {
        let start = std::time::Instant::now();
        let check_interval = Duration::from_millis(500);
        let max_wait = Duration::from_secs(max_wait_secs);

        while start.elapsed() < max_wait {
            if self.is_healthy().await? {
                return Ok(());
            }
            tokio::time::sleep(check_interval).await;
        }

        Err(Error::msg("Server health check timeout"))
    }

    /// 推理补全
    pub async fn complete(&self, request: InferenceRequest) -> Result<InferenceResponse, Error> {
        let start_time = std::time::Instant::now();
        let url = format!("{}/completion", self.get_server_url().await);

        let completion_request = CompletionRequest {
            prompt: request.prompt,
            n_predict: request.max_tokens,
            temperature: request.temperature,
            stop: request.stop_tokens,
            stream: false,
            cache_prompt: Some(true),
        };

        let response = self
            .http_client
            .post(&url)
            .json(&completion_request)
            .send()
            .await?
            .json::<CompletionResponse>()
            .await?;

        let inference_time_ms = start_time.elapsed().as_millis() as u64;
        let tokens = response.timings.map(|t| t.total_predicted_per_token_ms).unwrap_or(0.0);
        let tokens_generated = (tokens * 1000.0) as u32; // 估算

        Ok(InferenceResponse {
            text: response.content,
            tokens_generated,
            inference_time_ms,
        })
    }

    /// 流式推理补全
    pub async fn complete_stream(
        &self,
        request: InferenceRequest,
    ) -> impl Stream<Item = Result<InferenceResponse, Error>> + Unpin {
        let start_time = std::time::Instant::now();
        let url = format!("{}/completion", self.get_server_url().await);

        let completion_request = CompletionRequest {
            prompt: request.prompt,
            n_predict: request.max_tokens,
            temperature: request.temperature,
            stop: request.stop_tokens,
            stream: true,
            cache_prompt: Some(true),
        };

        let client = self.http_client.clone();
        let base_url = self.get_server_url().await;

        stream! {
            let response = match client
                .post(&url)
                .json(&completion_request)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    yield Err(Error::new(e, "Failed to send request"));
                    return;
                }
            };

            let mut buf_reader = BufReader::new(response);
            let mut line = String::new();

            while let Ok(n) = buf_reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }

                let content = line.trim().to_string();
                line.clear();

                if content.is_empty() || !content.starts_with("data: ") {
                    continue;
                }

                let json_str = &content[6..]; // Remove "data: " prefix
                
                if json_str == "[DONE]" {
                    break;
                }

                match serde_json::from_str::<StreamResponse>(json_str) {
                    Ok(stream_resp) => {
                        let inference_time_ms = start_time.elapsed().as_millis() as u64;
                        yield Ok(InferenceResponse {
                            text: stream_resp.content,
                            tokens_generated: 1,
                            inference_time_ms,
                        });

                        if stream_resp.stop {
                            break;
                        }
                    }
                    Err(e) => {
                        // 忽略解析错误，继续读取下一行
                        continue;
                    }
                }
            }
        }
    }

    /// 聊天补全
    pub async fn chat_complete(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<InferenceResponse, Error> {
        // 构建 chat template
        let prompt = Self::build_chat_prompt(&messages);
        
        self.complete(InferenceRequest {
            prompt,
            max_tokens,
            temperature,
            stop_tokens: vec!["</s>".to_string(), "[/INST]".to_string()],
            stream: false,
        }).await
    }

    /// 构建聊天提示词
    fn build_chat_prompt(messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();
        
        for message in messages {
            match message.role.as_str() {
                "system" => {
                    prompt.push_str(&format!("<|im_start|>system\n{}\n<|im_end|>\n", message.content));
                }
                "user" => {
                    prompt.push_str(&format!("<|im_start|>user\n{}\n<|im_end|>\n", message.content));
                }
                "assistant" => {
                    prompt.push_str(&format!("<|im_start|>assistant\n{}\n<|im_end|>\n", message.content));
                }
                _ => {}
            }
        }
        
        prompt.push_str("<|im_start|>assistant\n");
        prompt
    }

    /// 获取文本嵌入向量
    pub async fn embed(&self, text: String) -> Result<Vec<f32>, Error> {
        // 注意: llama-server 的 /embedding 端点需要特殊处理
        // 这里使用简化的实现
        let url = format!("{}/embedding", self.get_server_url().await);
        
        #[derive(Serialize)]
        struct EmbedRequest {
            content: String,
        }

        let response = self
            .http_client
            .post(&url)
            .json(&EmbedRequest { content: text })
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // 解析嵌入向量
        if let Some(embedding) = response["embedding"].as_array() {
            let mut result = Vec::with_capacity(embedding.len());
            for v in embedding {
                if let Some(f) = v.as_f64() {
                    result.push(f as f32);
                }
            }
            Ok(result)
        } else {
            Err(Error::msg("Failed to parse embedding response"))
        }
    }
}

/// 推理任务
struct InferenceTask {
    request: InferenceRequest,
    response_sender: oneshot::Sender<Result<InferenceResponse, Error>>,
}

/// 推理队列
#[derive(Clone)]
pub struct InferenceQueue {
    sender: mpsc::Sender<InferenceTask>,
    _receiver: Arc<Mutex<mpsc::Receiver<InferenceTask>>>,
    active_requests: Arc<AtomicUsize>,
    max_concurrent: usize,
    llama: Arc<LlamaSidecar>,
}

impl InferenceQueue {
    /// 创建新的推理队列
    pub fn new(llama: Arc<LlamaSidecar>, max_concurrent: usize) -> Self {
        let (sender, receiver) = mpsc::channel(100);
        
        Self {
            sender,
            _receiver: Arc::new(Mutex::new(receiver)),
            active_requests: Arc::new(AtomicUsize::new(0)),
            max_concurrent,
            llama,
        }
    }

    /// 入队推理任务
    pub async fn enqueue(&self, request: InferenceRequest) -> Result<InferenceResponse, Error> {
        let (response_sender, response_receiver) = oneshot::channel();
        
        let task = InferenceTask {
            request,
            response_sender,
        };

        self.sender.send(task).await?;
        
        // 等待结果
        response_receiver.await??
    }

    /// 获取活动请求数
    pub fn active_count(&self) -> usize {
        self.active_requests.load(Ordering::Relaxed)
    }

    /// 检查是否可以处理新请求
    pub fn can_accept(&self) -> bool {
        self.active_count() < self.max_concurrent
    }
}

/// 问题转换提示词工程
pub struct QuestionTransformationPrompt;

impl QuestionTransformationPrompt {
    /// 构建填空题转选择题的提示词
    pub fn fill_in_blank_to_multiple_choice(
        question: &str,
        answer: &str,
        difficulty: u32,
    ) -> String {
        let difficulty_instruction = match difficulty {
            1 => "生成简单的干扰项，只改变1-2个字符",
            2 => "生成中等难度的干扰项，需要理解题目含义才能区分",
            3 => "生成高难度干扰项，语义接近正确答案，容易混淆",
            _ => "生成合理的干扰项",
        };

        format!(
            r#"You are an expert exam question designer. Your task is to convert a fill-in-the-blank question into a high-quality multiple-choice question.

## Original Question:
{}

## Correct Answer:
{}

## Difficulty Level: {}

## Requirements:
1. {}
2. Generate exactly 3 distractors (wrong options)
3. Each distractor must be:
   - Plausible (could be a reasonable answer if not carefully considered)
   - Grammatically consistent with the correct answer
   - In the same category/concept as the correct answer
   - Not obviously wrong

4. Distractor design principles:
   - **Grammar level**: Same part of speech, tense, number agreement
   - **Semantic level**: Same concept category, related terms
   - **Cognitive level**: Common misconceptions, frequently confused concepts

5. Output format (JSON):
{{
  "question": "The multiple-choice question with [BLANK] replaced by _____",
  "correct_answer": "{}",
  "distractors": ["distractor1", "distractor2", "distractor3"],
  "explanation": "Brief explanation of why the correct answer is correct"
}}

Output only the JSON, no other text."#,
            question,
            answer,
            difficulty,
            difficulty_instruction,
            answer
        )
    }

    /// 生成解题步骤
    pub fn generate_explanation(question: &str, answer: &str) -> String {
        format!(
            r#"You are an expert tutor. Provide a detailed step-by-step explanation for the following question.

## Question:
{}

## Answer:
{}

## Please provide:
1. **Analysis**: Break down the question and identify key concepts
2. **Step-by-step Solution**: Show the reasoning process
3. **Key Points**: Highlight the most important takeaways
4. **Common Mistakes**: What errors do students typically make?

Be thorough and educational. Use clear formatting with headings and bullet points."#,
            question, answer
        )
    }

    /// 提取知识点标签
    pub fn extract_knowledge_tags(question: &str) -> String {
        format!(
            r#"Analyze the following question and extract relevant knowledge tags.

## Question:
{}

## Output format:
Return a JSON array of tags, including:
- Subject category (e.g., "math", "physics", "programming", "english")
- Topic (e.g., "calculus", "mechanics", "algorithms", "grammar")
- Specific concepts (e.g., "derivatives", "newton's laws", "binary search", "verb tenses")
- Difficulty indicators (e.g., "fundamental", "intermediate", "advanced")

Example output:
["math", "calculus", "derivatives", "chain rule", "intermediate"]

Output only the JSON array, no other text."#,
            question
        )
    }

    /// 生成选择题干扰项
    pub fn generate_distractors(
        question: &str,
        correct_answer: &str,
        count: u32,
        difficulty: u32,
    ) -> String {
        format!(
            r#"You are an expert question designer. Generate {} distractor options for a multiple-choice question.

## Question:
{}

## Correct Answer:
{}

## Difficulty: {}

## Distractor Generation Guidelines:
1. **Plausibility**: Each distractor must seem reasonable
2. **Consistency**: Match the format and style of the correct answer
3. **Common Errors**: Target typical student mistakes
4. **Variety**: Each distractor should be wrong for a different reason

Generate exactly {} unique distractors. Output as a JSON array of strings.

Example:
["wrong_option_1", "wrong_option_2", "wrong_option_3"]"#,
            count, question, correct_answer, difficulty, count
        )
    }
}

/// 问题转换输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedQuestion {
    pub question: String,
    pub correct_answer: String,
    pub distractors: Vec<String>,
    pub explanation: String,
    pub knowledge_tags: Vec<String>,
}

/// 批量转换输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillInBlankInput {
    pub question: String,
    pub answer: String,
    pub difficulty: u32,
}

/// 进度更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub current: u32,
    pub total: u32,
    pub status: String,
    pub current_item: Option<String>,
}

/// 检查电池状态并切换模型
pub async fn check_and_switch_model(
    llama: &LlamaSidecar,
    battery_level: u8,
    is_on_battery: bool,
) -> Result<ModelSize> {
    let current_config = llama.get_config().await;
    
    let target_size = if is_on_battery && battery_level < 20 {
        // 低电量模式，切换到小模型
        ModelSize::Small
    } else {
        ModelSize::Large
    };
    
    // 这里可以实现模型热切换逻辑
    // 实际切换需要停止当前服务，加载新模型，然后重启服务
    
    Ok(target_size)
}

/// 获取模型路径（根据模型尺寸）
pub fn get_model_path_by_size(base_dir: &Path, size: ModelSize) -> PathBuf {
    match size {
        ModelSize::Small => base_dir.join("models/qwen2.5-1.5b-instruct.Q4_K_M.gguf"),
        ModelSize::Medium => base_dir.join("models/llama3.2-3b-instruct.Q4_K_M.gguf"),
        ModelSize::Large => base_dir.join("models/llama3.1-8b-instruct.Q4_K_M.gguf"),
    }
}

/// 根据模型尺寸获取推荐配置
pub fn get_config_by_model_size(size: ModelSize, port: u16) -> LlamaConfig {
    match size {
        ModelSize::Small => LlamaConfig {
            model_path: PathBuf::new(), // 运行时设置
            context_size: 2048,
            batch_size: 256,
            gpu_layers: -1,
            use_flash_attn: true,
            threads: 4,
            port,
            temp: 0.6,
            repeat_penalty: 1.1,
        },
        ModelSize::Medium => LlamaConfig {
            model_path: PathBuf::new(),
            context_size: 4096,
            batch_size: 512,
            gpu_layers: -1,
            use_flash_attn: true,
            threads: 6,
            port,
            temp: 0.7,
            repeat_penalty: 1.1,
        },
        ModelSize::Large => LlamaConfig {
            model_path: PathBuf::new(),
            context_size: 4096,
            batch_size: 512,
            gpu_layers: -1,
            use_flash_attn: true,
            threads: 8,
            port,
            temp: 0.7,
            repeat_penalty: 1.1,
        },
    }
}
