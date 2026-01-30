//! Llama.cpp Tauri 命令模块
//! 提供供前端调用的 AI 推理命令接口

use crate::services::llama::{
    self, FillInBlankInput, InferenceQueue, LlamaSidecar, ModelSize,
    QuestionTransformationPrompt, TransformedQuestion,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

/// Llama 服务状态
pub struct LlamaState {
    pub sidecar: Arc<Mutex<LlamaSidecar>>,
    pub queue: Arc<InferenceQueue>,
    pub is_initialized: bool,
    pub current_model_size: Option<ModelSize>,
}

impl LlamaState {
    /// 创建新的 Llama 状态
    pub fn new(sidecar: Arc<Mutex<LlamaSidecar>>) -> Self {
        let sidecar_clone = sidecar.clone();
        let queue = Arc::new(InferenceQueue::new(sidecar_clone, 4));
        
        Self {
            sidecar,
            queue,
            is_initialized: false,
            current_model_size: None,
        }
    }
}

/// 初始化 Llama 服务
#[tauri::command]
pub async fn init_llama_service(app: AppHandle, state: State<'_, Mutex<LlamaState>>) -> Result<(), String> {
    let state_guard = state.lock().await;
    let sidecar = state_guard.sidecar.clone();

    // 获取 sidecar 路径
    let binary_name = llama::get_platform_binary_name();
    let binary_path = llama::get_sidecar_path(&app, binary_name)
        .map_err(|e| e.to_string())?;

    // 初始化配置
    let config = llama::LlamaConfig {
        model_path: std::path::PathBuf::new(),
        context_size: 4096,
        batch_size: 512,
        gpu_layers: -1,
        use_flash_attn: true,
        threads: 8,
        port: 8080,
        temp: 0.7,
        repeat_penalty: 1.1,
    };

    // 更新 sidecar 配置
    {
        let mut sidecar_guard = sidecar.lock().await;
        *sidecar_guard.config.lock().await = config;
    }

    // 标记为已初始化
    drop(state_guard);
    let mut state_mut = state.lock().await;
    state_mut.is_initialized = true;

    Ok(())
}

/// 启动 Llama 服务器
#[tauri::command]
pub async fn start_llama_server(
    app: AppHandle,
    state: State<'_, Mutex<LlamaState>>,
    model_path: String,
    model_size: String,
) -> Result<ServerStatusDto, String> {
    let state_guard = state.lock().await;
    let mut sidecar = state_guard.sidecar.lock().await;
    
    if sidecar.is_running().await.map_err(|e| e.to_string())? {
        let config = sidecar.get_config().await;
        return Ok(ServerStatusDto {
            is_running: true,
            server_url: sidecar.get_server_url().await,
            model_path: model_path.clone(),
            port: config.port,
            model_size: model_size.clone(),
            gpu_layers: config.gpu_layers,
        });
    }

    // 解析模型尺寸
    let size = match model_size.as_str() {
        "small" => ModelSize::Small,
        "medium" => ModelSize::Medium,
        "large" => ModelSize::Large,
        _ => ModelSize::Medium,
    };

    // 获取 sidecar 路径
    let binary_name = llama::get_platform_binary_name();
    let binary_path = llama::get_sidecar_path(&app, binary_name)
        .map_err(|e| e.to_string())?;

    // 配置
    let config = llama::LlamaConfig {
        model_path: model_path.clone().into(),
        context_size: 4096,
        batch_size: 512,
        gpu_layers: -1,
        use_flash_attn: true,
        threads: 8,
        port: 8080,
        temp: 0.7,
        repeat_penalty: 1.1,
    };

    // 更新状态
    *sidecar.config.lock().await = config;

    // 启动服务器
    sidecar.start(&binary_path).await.map_err(|e| e.to_string())?;

    // 释放锁后更新状态
    drop(sidecar);
    
    let mut state_mut = state.lock().await;
    state_mut.is_initialized = true;
    state_mut.current_model_size = Some(size);

    Ok(ServerStatusDto {
        is_running: true,
        server_url: format!("http://127.0.0.1:{}", 8080),
        model_path,
        port: 8080,
        model_size,
        gpu_layers: -1,
    })
}

/// 停止 Llama 服务器
#[tauri::command]
pub async fn stop_llama_server(state: State<'_, Mutex<LlamaState>>) -> Result<(), String> {
    let state_guard = state.lock().await;
    let mut sidecar = state_guard.sidecar.lock().await;
    
    sidecar.stop().await.map_err(|e| e.to_string())?;
    
    drop(sidecar);
    
    let mut state_mut = state.lock().await;
    state_mut.is_initialized = false;
    
    Ok(())
}

/// 检查 Llama 服务器健康状态
#[tauri::command]
pub async fn is_llama_server_healthy(
    state: State<'_, Mutex<LlamaState>>,
) -> Result<bool, String> {
    let state_guard = state.lock().await;
    
    if !state_guard.is_initialized {
        return Ok(false);
    }
    
    let sidecar = state_guard.sidecar.lock().await;
    sidecar.is_healthy().await.map_err(|e| e.to_string())
}

/// 获取服务器状态
#[tauri::command]
pub async fn get_llama_server_status(
    state: State<'_, Mutex<LlamaState>>,
) -> Result<ServerStatusDto, String> {
    let state_guard = state.lock().await;
    let sidecar = state_guard.sidecar.lock().await;
    
    let is_running = sidecar.is_running().await.map_err(|e| e.to_string())?;
    let config = sidecar.get_config().await;
    
    Ok(ServerStatusDto {
        is_running,
        server_url: sidecar.get_server_url().await,
        model_path: config.model_path.to_string_lossy().to_string(),
        port: config.port,
        model_size: match state_guard.current_model_size {
            Some(ModelSize::Small) => "small".to_string(),
            Some(ModelSize::Medium) => "medium".to_string(),
            Some(ModelSize::Large) => "large".to_string(),
            None => "unknown".to_string(),
        },
        gpu_layers: config.gpu_layers,
    })
}

/// 转换填空题为选择题
#[tauri::command]
pub async fn transform_fill_in_blank(
    state: State<'_, Mutex<LlamaState>>,
    question: String,
    answer: String,
    difficulty: u32,
) -> Result<TransformedQuestionDto, String> {
    let state_guard = state.lock().await;
    let sidecar = state_guard.sidecar.lock().await;
    
    // 生成提示词
    let prompt = QuestionTransformationPrompt::fill_in_blank_to_multiple_choice(
        &question,
        &answer,
        difficulty,
    );

    // 推理
    let response = sidecar.complete(llama::InferenceRequest {
        prompt,
        max_tokens: 1024,
        temperature: 0.7,
        stop_tokens: vec![],
        stream: false,
    }).await.map_err(|e| e.to_string())?;

    // 解析 JSON 响应
    let transformed: TransformedQuestion = serde_json::from_str(&response.text)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(TransformedQuestionDto {
        question: transformed.question,
        correct_answer: transformed.correct_answer,
        distractors: transformed.distractors,
        explanation: transformed.explanation,
        knowledge_tags: transformed.knowledge_tags,
    })
}

/// 生成干扰项
#[tauri::command]
pub async fn generate_distractors(
    state: State<'_, Mutex<LlamaState>>,
    question: String,
    correct_answer: String,
    count: u32,
) -> Result<Vec<String>, String> {
    let state_guard = state.lock().await;
    let sidecar = state_guard.sidecar.lock().await;
    
    let prompt = QuestionTransformationPrompt::generate_distractors(
        &question,
        &correct_answer,
        count,
        2, // 默认中等难度
    );

    let response = sidecar.complete(llama::InferenceRequest {
        prompt,
        max_tokens: 512,
        temperature: 0.8,
        stop_tokens: vec![],
        stream: false,
    }).await.map_err(|e| e.to_string())?;

    // 解析 JSON 数组
    let distractors: Vec<String> = serde_json::from_str(&response.text)
        .map_err(|e| format!("Failed to parse distractors: {}", e))?;

    Ok(distractors)
}

/// 生成解题步骤
#[tauri::command]
pub async fn generate_explanation(
    state: State<'_, Mutex<LlamaState>>,
    question: String,
    answer: String,
) -> Result<String, String> {
    let state_guard = state.lock().await;
    let sidecar = state_guard.sidecar.lock().await;
    
    let prompt = QuestionTransformationPrompt::generate_explanation(&question, &answer);

    let response = sidecar.complete(llama::InferenceRequest {
        prompt,
        max_tokens: 2048,
        temperature: 0.7,
        stop_tokens: vec![],
        stream: false,
    }).await.map_err(|e| e.to_string())?;

    Ok(response.text)
}

/// 提取知识点标签
#[tauri::command]
pub async fn extract_knowledge_tags(
    state: State<'_, Mutex<LlamaState>>,
    question: String,
) -> Result<Vec<String>, String> {
    let state_guard = state.lock().await;
    let sidecar = state_guard.sidecar.lock().await;
    
    let prompt = QuestionTransformationPrompt::extract_knowledge_tags(&question);

    let response = sidecar.complete(llama::InferenceRequest {
        prompt,
        max_tokens: 256,
        temperature: 0.3,
        stop_tokens: vec![],
        stream: false,
    }).await.map_err(|e| e.to_string())?;

    // 解析 JSON 数组
    let tags: Vec<String> = serde_json::from_str(&response.text)
        .map_err(|e| format!("Failed to parse tags: {}", e))?;

    Ok(tags)
}

/// 聊天补全
#[tauri::command]
pub async fn chat_complete(
    state: State<'_, Mutex<LlamaState>>,
    messages: Vec<ChatMessageDto>,
    max_tokens: u32,
    temperature: f32,
) -> Result<String, String> {
    let state_guard = state.lock().await;
    let sidecar = state_guard.sidecar.lock().await;
    
    let chat_messages: Vec<llama::ChatMessage> = messages
        .into_iter()
        .map(|m| llama::ChatMessage {
            role: m.role,
            content: m.content,
        })
        .collect();

    let response = sidecar
        .chat_complete(chat_messages, max_tokens, temperature)
        .await
        .map_err(|e| e.to_string())?;

    Ok(response.text)
}

/// 文本嵌入
#[tauri::command]
pub async fn get_text_embedding(
    state: State<'_, Mutex<LlamaState>>,
    text: String,
) -> Result<Vec<f32>, String> {
    let state_guard = state.lock().await;
    let sidecar = state_guard.sidecar.lock().await;
    
    let embedding = sidecar.embed(text).await.map_err(|e| e.to_string())?;
    
    Ok(embedding)
}

/// 批量转换问题
#[tauri::command]
pub async fn batch_transform_questions(
    app: AppHandle,
    state: State<'_, Mutex<LlamaState>>,
    questions: Vec<FillInBlankInput>,
) -> Result<Vec<TransformedQuestionDto>, String> {
    let state_guard = state.lock().await;
    let sidecar = state_guard.sidecar.lock().await;
    
    let total = questions.len() as u32;
    
    let mut results = Vec::with_capacity(questions.len());
    
    for (index, input) in questions.iter().enumerate() {
        // 发送进度更新
        let _ = app.emit(
            "llama_progress",
            ProgressUpdateDto {
                current: (index + 1) as u32,
                total,
                status: "processing".to_string(),
                current_item: Some(input.question.clone()),
            },
        );

        let prompt = QuestionTransformationPrompt::fill_in_blank_to_multiple_choice(
            &input.question,
            &input.answer,
            input.difficulty,
        );

        let response = sidecar.complete(llama::InferenceRequest {
            prompt,
            max_tokens: 1024,
            temperature: 0.7,
            stop_tokens: vec![],
            stream: false,
        }).await.map_err(|e| e.to_string())?;

        let transformed: TransformedQuestion = serde_json::from_str(&response.text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        results.push(TransformedQuestionDto {
            question: transformed.question,
            correct_answer: transformed.correct_answer,
            distractors: transformed.distractors,
            explanation: transformed.explanation,
            knowledge_tags: transformed.knowledge_tags,
        });
    }

    // 发送完成通知
    let _ = app.emit(
        "llama_progress",
        ProgressUpdateDto {
            current: total,
            total,
            status: "completed".to_string(),
            current_item: None,
        },
    );

    Ok(results)
}

/// 切换模型
#[tauri::command]
pub async fn switch_model(
    app: AppHandle,
    state: State<'_, Mutex<LlamaState>>,
    model_size: String,
    model_path: String,
) -> Result<ServerStatusDto, String> {
    let state_guard = state.lock().await;
    let mut sidecar = state_guard.sidecar.lock().await;
    
    // 停止当前服务
    sidecar.stop().await.map_err(|e| e.to_string())?;
    
    // 解析新模型尺寸
    let size = match model_size.as_str() {
        "small" => ModelSize::Small,
        "medium" => ModelSize::Medium,
        "large" => ModelSize::Large,
        _ => ModelSize::Medium,
    };

    // 获取 sidecar 路径
    let binary_name = llama::get_platform_binary_name();
    let binary_path = llama::get_sidecar_path(&app, binary_name)
        .map_err(|e| e.to_string())?;

    // 更新配置
    let config = llama::LlamaConfig {
        model_path: model_path.clone().into(),
        context_size: 4096,
        batch_size: 512,
        gpu_layers: -1,
        use_flash_attn: true,
        threads: 8,
        port: 8080,
        temp: 0.7,
        repeat_penalty: 1.1,
    };

    *sidecar.config.lock().await = config;

    // 启动新服务
    sidecar.start(&binary_path).await.map_err(|e| e.to_string())?;
    
    drop(sidecar);
    
    let mut state_mut = state.lock().await;
    state_mut.current_model_size = Some(size);

    Ok(ServerStatusDto {
        is_running: true,
        server_url: format!("http://127.0.0.1:{}", 8080),
        model_path,
        port: 8080,
        model_size,
        gpu_layers: -1,
    })
}

// DTO 类型定义

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusDto {
    pub is_running: bool,
    pub server_url: String,
    pub model_path: String,
    pub port: u16,
    pub model_size: String,
    pub gpu_layers: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedQuestionDto {
    pub question: String,
    pub correct_answer: String,
    pub distractors: Vec<String>,
    pub explanation: String,
    pub knowledge_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageDto {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdateDto {
    pub current: u32,
    pub total: u32,
    pub status: String,
    pub current_item: Option<String>,
}
