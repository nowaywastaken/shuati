// 数据库命令模块
// 提供供前端调用的数据库操作命令

use crate::services::database::{
    DatabaseService,
    Problem,
    ProblemSet,
    UserProgress,
    WrongNote,
    ProgressStats,
};
use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;

/// 数据库应用状态
pub struct DbState(pub Mutex<DatabaseService>);

/// 问题传输对象（前端传入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionDto {
    pub id: Option<String>,
    pub number: Option<u32>,
    pub question_type: String,
    pub content: String,
    pub options: String,
    pub answer: Option<String>,
    pub explanation: Option<String>,
    pub difficulty: Option<i32>,
    pub knowledge_tags: String,
    pub source_path: Option<String>,
}

/// 问题传输对象（返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDto {
    pub id: String,
    pub set_id: String,
    pub number: Option<u32>,
    pub question_type: String,
    pub content: String,
    pub options: String,
    pub answer: Option<String>,
    pub explanation: Option<String>,
    pub difficulty: Option<i32>,
    pub knowledge_tags: String,
    pub source_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 题目集传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemSetDto {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub total_problems: i32,
    pub created_at: String,
}

/// 用户进度传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgressDto {
    pub id: String,
    pub problem_id: String,
    pub status: String,
    pub user_answer: Option<String>,
    pub time_spent: i32,
    pub attempted_at: String,
}

/// 错题本传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrongNoteDto {
    pub id: String,
    pub problem_id: String,
    pub user_answer: String,
    pub correct_answer: String,
    pub note: Option<String>,
    pub review_count: i32,
    pub last_reviewed_at: String,
}

/// 进度统计传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStatsDto {
    pub total_attempts: i32,
    pub correct_count: i32,
    pub incorrect_count: i32,
    pub skipped_count: i32,
    pub average_time_spent: f64,
    pub accuracy_rate: f64,
}

/// 初始化数据库
#[tauri::command]
pub async fn init_database(state: State<'_, DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    // 确保表结构已创建
    db.initialize().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 创建题目集
#[tauri::command]
pub async fn create_problem_set(
    title: String,
    description: Option<String>,
    file_path: Option<String>,
    state: State<'_, DbState>,
) -> Result<String, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let id = db.create_problem_set(
        &title,
        description.as_deref(),
        file_path.as_deref(),
    ).map_err(|e| e.to_string())?;
    
    Ok(id)
}

/// 获取题目集列表
#[tauri::command]
pub async fn get_problem_sets(
    state: State<'_, DbState>,
) -> Result<Vec<ProblemSetDto>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let problem_sets = db.list_problem_sets().map_err(|e| e.to_string())?;
    
    let dtos: Vec<ProblemSetDto> = problem_sets
        .into_iter()
        .map(|ps| ProblemSetDto {
            id: ps.id,
            title: ps.title,
            description: ps.description,
            file_path: ps.file_path,
            total_problems: ps.total_problems,
            created_at: ps.created_at.to_rfc3339(),
        })
        .collect();
    
    Ok(dtos)
}

/// 获取题目集
#[tauri::command]
pub async fn get_problem_set(
    id: String,
    state: State<'_, DbState>,
) -> Result<Option<ProblemSetDto>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let problem_set = db.get_problem_set(&id).map_err(|e| e.to_string())?;
    
    Ok(problem_set.map(|ps| ProblemSetDto {
        id: ps.id,
        title: ps.title,
        description: ps.description,
        file_path: ps.file_path,
        total_problems: ps.total_problems,
        created_at: ps.created_at.to_rfc3339(),
    }))
}

/// 添加题目
#[tauri::command]
pub async fn add_problems(
    set_id: String,
    questions: Vec<QuestionDto>,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let now = Utc::now();
    
    let problems: Vec<Problem> = questions
        .into_iter()
        .map(|q| Problem {
            id: q.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            set_id: set_id.clone(),
            number: q.number,
            question_type: q.question_type,
            content: q.content,
            options: q.options,
            answer: q.answer,
            explanation: q.explanation,
            difficulty: q.difficulty,
            knowledge_tags: q.knowledge_tags,
            source_path: q.source_path,
            created_at: now,
            updated_at: now,
        })
        .collect();
    
    db.add_problems_batch(&problems).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 获取题目集下所有题目
#[tauri::command]
pub async fn get_problems(
    set_id: String,
    state: State<'_, DbState>,
) -> Result<Vec<ProblemDto>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let problems = db.get_problems_by_set(&set_id).map_err(|e| e.to_string())?;
    
    let dtos: Vec<ProblemDto> = problems
        .into_iter()
        .map(|p| ProblemDto {
            id: p.id,
            set_id: p.set_id,
            number: p.number,
            question_type: p.question_type,
            content: p.content,
            options: p.options,
            answer: p.answer,
            explanation: p.explanation,
            difficulty: p.difficulty,
            knowledge_tags: p.knowledge_tags,
            source_path: p.source_path,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        })
        .collect();
    
    Ok(dtos)
}

/// 搜索题目
#[tauri::command]
pub async fn search_problems(
    keyword: String,
    limit: i32,
    state: State<'_, DbState>,
) -> Result<Vec<ProblemDto>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let problems = db.search_problems(&keyword, limit).map_err(|e| e.to_string())?;
    
    let dtos: Vec<ProblemDto> = problems
        .into_iter()
        .map(|p| ProblemDto {
            id: p.id,
            set_id: p.set_id,
            number: p.number,
            question_type: p.question_type,
            content: p.content,
            options: p.options,
            answer: p.answer,
            explanation: p.explanation,
            difficulty: p.difficulty,
            knowledge_tags: p.knowledge_tags,
            source_path: p.source_path,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        })
        .collect();
    
    Ok(dtos)
}

/// 获取单道题目
#[tauri::command]
pub async fn get_problem(
    id: String,
    state: State<'_, DbState>,
) -> Result<Option<ProblemDto>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let problem = db.get_problem(&id).map_err(|e| e.to_string())?;
    
    Ok(problem.map(|p| ProblemDto {
        id: p.id,
        set_id: p.set_id,
        number: p.number,
        question_type: p.question_type,
        content: p.content,
        options: p.options,
        answer: p.answer,
        explanation: p.explanation,
        difficulty: p.difficulty,
        knowledge_tags: p.knowledge_tags,
        source_path: p.source_path,
        created_at: p.created_at.to_rfc3339(),
        updated_at: p.updated_at.to_rfc3339(),
    }))
}

/// 按知识标签获取题目
#[tauri::command]
pub async fn get_problems_by_tag(
    tag: String,
    state: State<'_, DbState>,
) -> Result<Vec<ProblemDto>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let problems = db.get_problems_by_tag(&tag).map_err(|e| e.to_string())?;
    
    let dtos: Vec<ProblemDto> = problems
        .into_iter()
        .map(|p| ProblemDto {
            id: p.id,
            set_id: p.set_id,
            number: p.number,
            question_type: p.question_type,
            content: p.content,
            options: p.options,
            answer: p.answer,
            explanation: p.explanation,
            difficulty: p.difficulty,
            knowledge_tags: p.knowledge_tags,
            source_path: p.source_path,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        })
        .collect();
    
    Ok(dtos)
}

/// 按难度获取题目
#[tauri::command]
pub async fn get_problems_by_difficulty(
    difficulty: i32,
    state: State<'_, DbState>,
) -> Result<Vec<ProblemDto>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let problems = db.get_problems_by_difficulty(difficulty).map_err(|e| e.to_string())?;
    
    let dtos: Vec<ProblemDto> = problems
        .into_iter()
        .map(|p| ProblemDto {
            id: p.id,
            set_id: p.set_id,
            number: p.number,
            question_type: p.question_type,
            content: p.content,
            options: p.options,
            answer: p.answer,
            explanation: p.explanation,
            difficulty: p.difficulty,
            knowledge_tags: p.knowledge_tags,
            source_path: p.source_path,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        })
        .collect();
    
    Ok(dtos)
}

/// 记录答题结果
#[tauri::command]
pub async fn record_answer(
    problem_id: String,
    user_answer: String,
    is_correct: bool,
    time_spent: i32,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let status = if is_correct { "correct" } else { "incorrect" };
    
    let progress = UserProgress {
        id: Uuid::new_v4().to_string(),
        problem_id,
        status: status.to_string(),
        user_answer: Some(user_answer),
        time_spent,
        attempted_at: Utc::now(),
    };
    
    db.record_progress(&progress).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 记录跳过
#[tauri::command]
pub async fn record_skip(
    problem_id: String,
    time_spent: i32,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let progress = UserProgress {
        id: Uuid::new_v4().to_string(),
        problem_id,
        status: "skipped".to_string(),
        user_answer: None,
        time_spent,
        attempted_at: Utc::now(),
    };
    
    db.record_progress(&progress).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 获取题目集进度统计
#[tauri::command]
pub async fn get_problem_set_progress(
    set_id: String,
    state: State<'_, DbState>,
) -> Result<ProgressStatsDto, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let stats = db.get_problem_set_progress(&set_id).map_err(|e| e.to_string())?;
    
    Ok(ProgressStatsDto {
        total_attempts: stats.total_attempts,
        correct_count: stats.correct_count,
        incorrect_count: stats.incorrect_count,
        skipped_count: stats.skipped_count,
        average_time_spent: stats.average_time_spent,
        accuracy_rate: stats.accuracy_rate,
    })
}

/// 获取错题本
#[tauri::command]
pub async fn get_wrong_notes(
    limit: i32,
    state: State<'_, DbState>,
) -> Result<Vec<WrongNoteDto>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let notes = db.get_wrong_notes(limit).map_err(|e| e.to_string())?;
    
    let dtos: Vec<WrongNoteDto> = notes
        .into_iter()
        .map(|n| WrongNoteDto {
            id: n.id,
            problem_id: n.problem_id,
            user_answer: n.user_answer,
            correct_answer: n.correct_answer,
            note: n.note,
            review_count: n.review_count,
            last_reviewed_at: n.last_reviewed_at.to_rfc3339(),
        })
        .collect();
    
    Ok(dtos)
}

/// 增加错题复习次数
#[tauri::command]
pub async fn increment_review_count(
    note_id: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    db.increment_review_count(&note_id).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 删除错题记录
#[tauri::command]
pub async fn delete_wrong_note(
    id: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    db.delete_wrong_note(&id).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 更新错题笔记
#[tauri::command]
pub async fn update_wrong_note(
    id: String,
    note: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    
    let conn = db.pool.lock().unwrap();
    
    conn.execute(
        "UPDATE wrong_notes SET note = ? WHERE id = ?",
        rusqlite::params![note, id],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}
