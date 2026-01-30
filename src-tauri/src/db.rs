use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub id: Option<i64>,
    pub question_type: String,
    pub stem: String,
    pub options: Option<String>,
    pub reference_answer: String,
    pub detailed_analysis: String,
    pub media_refs: Option<String>,
    pub knowledge_tags: Option<String>,
    pub difficulty: Option<i32>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionAttempt {
    pub id: Option<i64>,
    pub question_id: i64,
    pub user_answer: String,
    pub is_correct: bool,
    pub confidence_score: Option<f64>,
    pub time_spent_seconds: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchImportResult {
    pub success: bool,
    pub imported_count: usize,
    pub errors: Vec<String>,
}

fn get_db_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    Ok(app_data_dir.join("shuati.db"))
}

#[tauri::command]
pub fn init_database(app_handle: AppHandle) -> Result<(), String> {
    let db_path = get_db_path(&app_handle)?;

    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )
    .map_err(|e| format!("Failed to open database: {}", e))?;

    // Enable WAL mode for better concurrency
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA cache_size = 10000;
        PRAGMA foreign_keys = ON;
    ",
    )
    .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

    // Create tables
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS questions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_type TEXT NOT NULL CHECK(question_type IN ('multiple_choice', 'fill_in_the_blank', 'essay')),
            stem TEXT NOT NULL,
            options TEXT,
            reference_answer TEXT NOT NULL,
            detailed_analysis TEXT NOT NULL,
            media_refs TEXT,
            knowledge_tags TEXT,
            difficulty INTEGER CHECK(difficulty BETWEEN 1 AND 5),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE INDEX IF NOT EXISTS idx_questions_type ON questions(question_type);
        CREATE INDEX IF NOT EXISTS idx_questions_difficulty ON questions(difficulty);
        
        CREATE TABLE IF NOT EXISTS question_attempts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_id INTEGER NOT NULL,
            user_answer TEXT NOT NULL,
            is_correct BOOLEAN NOT NULL DEFAULT 0,
            confidence_score REAL,
            time_spent_seconds INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (question_id) REFERENCES questions(id) ON DELETE CASCADE
        );
        
        CREATE INDEX IF NOT EXISTS idx_attempts_question ON question_attempts(question_id);
        CREATE INDEX IF NOT EXISTS idx_attempts_correct ON question_attempts(is_correct);
        
        CREATE TABLE IF NOT EXISTS mistake_collections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question_id INTEGER NOT NULL,
            mistake_count INTEGER NOT NULL DEFAULT 1,
            last_mistake_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            review_count INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (question_id) REFERENCES questions(id) ON DELETE CASCADE
        );
        
        CREATE INDEX IF NOT EXISTS idx_mistakes_question ON mistake_collections(question_id);
    ").map_err(|e| format!("Failed to create tables: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn batch_import_questions(
    app_handle: AppHandle,
    questions: Vec<Question>,
) -> Result<BatchImportResult, String> {
    let db_path = get_db_path(&app_handle)?;
    let mut conn =
        Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    let mut imported_count = 0;
    let mut errors = Vec::new();

    for (idx, q) in questions.iter().enumerate() {
        let result = tx.execute(
            "INSERT INTO questions (question_type, stem, options, reference_answer, detailed_analysis, media_refs, knowledge_tags, difficulty) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                &q.question_type,
                &q.stem,
                q.options.as_deref(),
                &q.reference_answer,
                &q.detailed_analysis,
                q.media_refs.as_deref(),
                q.knowledge_tags.as_deref(),
                q.difficulty,
            ],
        );

        match result {
            Ok(_) => imported_count += 1,
            Err(e) => errors.push(format!("Question {}: {}", idx + 1, e)),
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(BatchImportResult {
        success: errors.is_empty(),
        imported_count,
        errors,
    })
}

#[tauri::command]
pub fn get_all_questions(app_handle: AppHandle) -> Result<Vec<Question>, String> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, question_type, stem, options, reference_answer, detailed_analysis, media_refs, knowledge_tags, difficulty, created_at 
         FROM questions ORDER BY created_at DESC"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let questions = stmt
        .query_map([], |row| {
            Ok(Question {
                id: row.get(0)?,
                question_type: row.get(1)?,
                stem: row.get(2)?,
                options: row.get(3)?,
                reference_answer: row.get(4)?,
                detailed_analysis: row.get(5)?,
                media_refs: row.get(6)?,
                knowledge_tags: row.get(7)?,
                difficulty: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Failed to query questions: {}", e))?;

    questions
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect questions: {}", e))
}

#[tauri::command]
pub fn save_attempt(app_handle: AppHandle, attempt: QuestionAttempt) -> Result<i64, String> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute(
        "INSERT INTO question_attempts (question_id, user_answer, is_correct, confidence_score, time_spent_seconds) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            attempt.question_id,
            attempt.user_answer,
            attempt.is_correct as i32,
            attempt.confidence_score.unwrap_or(0.0),
            attempt.time_spent_seconds,
        ],
    ).map_err(|e| format!("Failed to save attempt: {}", e))?;

    let id = conn.last_insert_rowid();

    // If incorrect, add to mistake collection
    if !attempt.is_correct {
        conn.execute(
            "INSERT INTO mistake_collections (question_id, mistake_count) 
             VALUES (?1, 1)
             ON CONFLICT(question_id) DO UPDATE SET
             mistake_count = mistake_count + 1,
             last_mistake_at = CURRENT_TIMESTAMP",
            [attempt.question_id],
        )
        .map_err(|e| format!("Failed to update mistake collection: {}", e))?;
    }

    Ok(id)
}

#[tauri::command]
pub fn get_mistakes_by_tag(
    app_handle: AppHandle,
    tag: Option<String>,
) -> Result<Vec<(Question, i32)>, String> {
    let db_path = get_db_path(&app_handle)?;
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    let sql = if let Some(t) = tag {
        format!(
            "SELECT q.*, m.mistake_count 
             FROM questions q 
             JOIN mistake_collections m ON q.id = m.question_id 
             WHERE q.knowledge_tags LIKE '%{}%'
             ORDER BY m.mistake_count DESC, m.last_mistake_at DESC",
            t
        )
    } else {
        "SELECT q.*, m.mistake_count 
         FROM questions q 
         JOIN mistake_collections m ON q.id = m.question_id 
         ORDER BY m.mistake_count DESC, m.last_mistake_at DESC"
            .to_string()
    };

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let mistakes = stmt
        .query_map([], |row| {
            let mistake_count: i32 = row.get(10)?;
            let question = Question {
                id: row.get(0)?,
                question_type: row.get(1)?,
                stem: row.get(2)?,
                options: row.get(3)?,
                reference_answer: row.get(4)?,
                detailed_analysis: row.get(5)?,
                media_refs: row.get(6)?,
                knowledge_tags: row.get(7)?,
                difficulty: row.get(8)?,
                created_at: row.get(9)?,
            };
            Ok((question, mistake_count))
        })
        .map_err(|e| format!("Failed to query mistakes: {}", e))?;

    mistakes
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect mistakes: {}", e))
}
