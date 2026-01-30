// 数据库服务模块
// 提供 SQLite 数据库操作，支持题库管理和用户进度追踪

use rusqlite::{Connection, Result, Row};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fs;

/// 数据库配置
const DEFAULT_DB_PATH: &str = "data/shuati.db";

/// 题目数据结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub set_id: String,
    pub number: Option<u32>,
    pub question_type: String,
    pub content: String,
    pub options: String,           // JSON 序列化
    pub answer: Option<String>,
    pub explanation: Option<String>,
    pub difficulty: Option<i32>,
    pub knowledge_tags: String,    // JSON 序列化
    pub source_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 题目集数据结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemSet {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub total_problems: i32,
    pub created_at: DateTime<Utc>,
}

/// 用户进度数据结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProgress {
    pub id: String,
    pub problem_id: String,
    pub status: String,            // "correct", "incorrect", "skipped"
    pub user_answer: Option<String>,
    pub time_spent: i32,           // 秒
    pub attempted_at: DateTime<Utc>,
}

/// 错题本数据结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WrongNote {
    pub id: String,
    pub problem_id: String,
    pub user_answer: String,
    pub correct_answer: String,
    pub note: Option<String>,
    pub review_count: i32,
    pub last_reviewed_at: DateTime<Utc>,
}

/// 进度统计
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressStats {
    pub total_attempts: i32,
    pub correct_count: i32,
    pub incorrect_count: i32,
    pub skipped_count: i32,
    pub average_time_spent: f64,
    pub accuracy_rate: f64,
}

/// 数据库服务
pub struct DatabaseService {
    pool: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl DatabaseService {
    /// 创建新的数据库服务
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = Self::get_default_db_path()?;
        
        // 确保数据目录存在
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let pool = Arc::new(Mutex::new(Connection::open(&db_path)?));
        
        let service = Self {
            pool,
            db_path,
        };
        
        service.initialize()?;
        Ok(service)
    }

    /// 获取默认数据库路径
    fn get_default_db_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
        let data_dir = exe_dir.join("data");
        
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }
        
        Ok(data_dir.join("shuati.db"))
    }

    /// 初始化数据库表结构
    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        // 题目集合表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS problem_sets (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                file_path TEXT,
                total_problems INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // 题目表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS problems (
                id TEXT PRIMARY KEY,
                set_id TEXT NOT NULL,
                number INTEGER,
                question_type TEXT NOT NULL,
                content TEXT NOT NULL,
                options TEXT,
                answer TEXT,
                explanation TEXT,
                difficulty INTEGER,
                knowledge_tags TEXT,
                source_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (set_id) REFERENCES problem_sets(id)
            )",
            [],
        )?;

        // 用户进度表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_progress (
                id TEXT PRIMARY KEY,
                problem_id TEXT NOT NULL,
                status TEXT NOT NULL,
                user_answer TEXT,
                time_spent INTEGER DEFAULT 0,
                attempted_at TEXT NOT NULL,
                FOREIGN KEY (problem_id) REFERENCES problems(id)
            )",
            [],
        )?;

        // 错题本表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wrong_notes (
                id TEXT PRIMARY KEY,
                problem_id TEXT NOT NULL,
                user_answer TEXT NOT NULL,
                correct_answer TEXT NOT NULL,
                note TEXT,
                review_count INTEGER DEFAULT 0,
                last_reviewed_at TEXT NOT NULL,
                FOREIGN KEY (problem_id) REFERENCES problems(id)
            )",
            [],
        )?;

        // 创建索引优化查询性能
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_problems_set_id ON problems(set_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_problems_difficulty ON problems(difficulty)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_problems_knowledge_tags ON problems(knowledge_tags)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_progress_problem_id ON user_progress(problem_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wrong_notes_problem_id ON wrong_notes(problem_id)",
            [],
        )?;

        Ok(())
    }

    // ==================== 题目集管理 ====================

    /// 创建题目集
    pub fn create_problem_set(
        &self,
        title: &str,
        description: Option<&str>,
        file_path: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO problem_sets (id, title, description, file_path, total_problems, created_at)
             VALUES (?, ?, ?, ?, 0, ?)",
            rusqlite::params![id, title, description, file_path, now],
        )?;

        Ok(id)
    }

    /// 获取题目集
    pub fn get_problem_set(&self, id: &str) -> Result<Option<ProblemSet>, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, title, description, file_path, total_problems, created_at 
             FROM problem_sets WHERE id = ?",
        )?;
        
        let mut rows = stmt.query(rusqlite::params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_problem_set(row)?))
        } else {
            Ok(None)
        }
    }

    /// 列出所有题目集
    pub fn list_problem_sets(&self) -> Result<Vec<ProblemSet>, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, title, description, file_path, total_problems, created_at 
             FROM problem_sets ORDER BY created_at DESC",
        )?;
        
        let rows = stmt.query_map([], |row| Self::row_to_problem_set(row))?;
        
        let mut problem_sets = Vec::new();
        for row in rows {
            problem_sets.push(row?);
        }
        
        Ok(problem_sets)
    }

    // ==================== 题目 CRUD ====================

    /// 添加单道题目
    pub fn add_problem(&self, problem: &Problem) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO problems 
             (id, set_id, number, question_type, content, options, answer, 
              explanation, difficulty, knowledge_tags, source_path, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                problem.id,
                problem.set_id,
                problem.number,
                problem.question_type,
                problem.content,
                problem.options,
                problem.answer,
                problem.explanation,
                problem.difficulty,
                problem.knowledge_tags,
                problem.source_path,
                problem.created_at.to_rfc3339(),
                problem.updated_at.to_rfc3339(),
            ],
        )?;

        // 更新题目集总数
        conn.execute(
            "UPDATE problem_sets SET total_problems = total_problems + 1 WHERE id = ?",
            rusqlite::params![problem.set_id],
        )?;

        Ok(())
    }

    /// 批量添加题目
    pub fn add_problems_batch(&self, problems: &[Problem]) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        let tx = conn.transaction()?;
        
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO problems 
             (id, set_id, number, question_type, content, options, answer, 
              explanation, difficulty, knowledge_tags, source_path, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?;

        for problem in problems {
            stmt.execute(rusqlite::params![
                problem.id,
                problem.set_id,
                problem.number,
                problem.question_type,
                problem.content,
                problem.options,
                problem.answer,
                problem.explanation,
                problem.difficulty,
                problem.knowledge_tags,
                problem.source_path,
                problem.created_at.to_rfc3339(),
                problem.updated_at.to_rfc3339(),
            ])?;
        }

        stmt.finish()?;
        tx.commit()?;

        // 更新题目集总数
        if let Some(first_problem) = problems.first() {
            conn.execute(
                "UPDATE problem_sets SET total_problems = total_problems + ? WHERE id = ?",
                rusqlite::params![problems.len() as i32, first_problem.set_id],
            )?;
        }

        Ok(())
    }

    /// 获取单道题目
    pub fn get_problem(&self, id: &str) -> Result<Option<Problem>, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, set_id, number, question_type, content, options, answer, 
                    explanation, difficulty, knowledge_tags, source_path, created_at, updated_at
             FROM problems WHERE id = ?",
        )?;
        
        let mut rows = stmt.query(rusqlite::params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_problem(row)?))
        } else {
            Ok(None)
        }
    }

    /// 获取题目集下所有题目
    pub fn get_problems_by_set(&self, set_id: &str) -> Result<Vec<Problem>, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, set_id, number, question_type, content, options, answer, 
                    explanation, difficulty, knowledge_tags, source_path, created_at, updated_at
             FROM problems WHERE set_id = ? ORDER BY number",
        )?;
        
        let rows = stmt.query_map(rusqlite::params![set_id], |row| Self::row_to_problem(row))?;
        
        let mut problems = Vec::new();
        for row in rows {
            problems.push(row?);
        }
        
        Ok(problems)
    }

    /// 搜索题目
    pub fn search_problems(&self, keyword: &str, limit: i32) -> Result<Vec<Problem>, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, set_id, number, question_type, content, options, answer, 
                    explanation, difficulty, knowledge_tags, source_path, created_at, updated_at
             FROM problems WHERE content LIKE ? OR knowledge_tags LIKE ?
             ORDER BY created_at DESC LIMIT ?",
        )?;
        
        let search_pattern = format!("%{}%", keyword);
        let tag_pattern = format!("%{}%", keyword);
        
        let rows = stmt.query_map(
            rusqlite::params![search_pattern, tag_pattern, limit],
            |row| Self::row_to_problem(row)
        )?;
        
        let mut problems = Vec::new();
        for row in rows {
            problems.push(row?);
        }
        
        Ok(problems)
    }

    /// 按知识标签获取题目
    pub fn get_problems_by_tag(&self, tag: &str) -> Result<Vec<Problem>, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, set_id, number, question_type, content, options, answer, 
                    explanation, difficulty, knowledge_tags, source_path, created_at, updated_at
             FROM problems WHERE knowledge_tags LIKE ?",
        )?;
        
        let tag_pattern = format!("%{}%", tag);
        let rows = stmt.query_map(
            rusqlite::params![tag_pattern],
            |row| Self::row_to_problem(row)
        )?;
        
        let mut problems = Vec::new();
        for row in rows {
            problems.push(row?);
        }
        
        Ok(problems)
    }

    /// 按难度获取题目
    pub fn get_problems_by_difficulty(&self, difficulty: i32) -> Result<Vec<Problem>, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, set_id, number, question_type, content, options, answer, 
                    explanation, difficulty, knowledge_tags, source_path, created_at, updated_at
             FROM problems WHERE difficulty = ?",
        )?;
        
        let rows = stmt.query_map(
            rusqlite::params![difficulty],
            |row| Self::row_to_problem(row)
        )?;
        
        let mut problems = Vec::new();
        for row in rows {
            problems.push(row?);
        }
        
        Ok(problems)
    }

    // ==================== 用户进度 ====================

    /// 记录答题进度
    pub fn record_progress(&self, progress: &UserProgress) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        conn.execute(
            "INSERT INTO user_progress 
             (id, problem_id, status, user_answer, time_spent, attempted_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                progress.id,
                progress.problem_id,
                progress.status,
                progress.user_answer,
                progress.time_spent,
                progress.attempted_at.to_rfc3339(),
            ],
        )?;

        // 如果答错，添加到错题本
        if progress.status == "incorrect" {
            // 获取题目正确答案
            let problem = self.get_problem(&progress.problem_id)?;
            if let Some(p) = problem {
                if let Some(answer) = &p.answer {
                    self.add_wrong_note_from_incorrect(
                        &progress.problem_id,
                        progress.user_answer.as_deref().unwrap_or(""),
                        answer,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// 获取题目的所有进度记录
    pub fn get_progress_for_problem(&self, problem_id: &str) -> Result<Vec<UserProgress>, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, problem_id, status, user_answer, time_spent, attempted_at
             FROM user_progress WHERE problem_id = ? ORDER BY attempted_at DESC",
        )?;
        
        let rows = stmt.query_map(rusqlite::params![problem_id], |row| {
            Ok(UserProgress {
                id: row.get(0)?,
                problem_id: row.get(1)?,
                status: row.get(2)?,
                user_answer: row.get(3)?,
                time_spent: row.get(4)?,
                attempted_at: row.get::<_, String>(5)?.parse::<DateTime<Utc>>()?.into(),
            })
        })?;
        
        let mut progress = Vec::new();
        for row in rows {
            progress.push(row?);
        }
        
        Ok(progress)
    }

    /// 获取题目集进度统计
    pub fn get_problem_set_progress(&self, set_id: &str) -> Result<ProgressStats, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN status = 'correct' THEN 1 ELSE 0 END) as correct,
                SUM(CASE WHEN status = 'incorrect' THEN 1 ELSE 0 END) as incorrect,
                SUM(CASE WHEN status = 'skipped' THEN 1 ELSE 0 END) as skipped,
                AVG(CASE WHEN time_spent > 0 THEN time_spent ELSE NULL END) as avg_time
             FROM user_progress
             WHERE problem_id IN (SELECT id FROM problems WHERE set_id = ?)",
        )?;
        
        let mut rows = stmt.query(rusqlite::params![set_id])?;
        
        if let Some(row) = rows.next()? {
            let total: i32 = row.get(0)?;
            let correct: i32 = row.get(1)?;
            let incorrect: i32 = row.get(2)?;
            let skipped: i32 = row.get(3)?;
            let avg_time: Option<f64> = row.get(4);
            
            let accuracy_rate = if total > 0 {
                correct as f64 / total as f64 * 100.0
            } else {
                0.0
            };

            Ok(ProgressStats {
                total_attempts: total,
                correct_count: correct,
                incorrect_count: incorrect,
                skipped_count: skipped,
                average_time_spent: avg_time.unwrap_or(0.0),
                accuracy_rate,
            })
        } else {
            Ok(ProgressStats {
                total_attempts: 0,
                correct_count: 0,
                incorrect_count: 0,
                skipped_count: 0,
                average_time_spent: 0.0,
                accuracy_rate: 0.0,
            })
        }
    }

    // ==================== 错题本 ====================

    /// 从答错记录添加错题
    fn add_wrong_note_from_incorrect(
        &self,
        problem_id: &str,
        user_answer: &str,
        correct_answer: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // 检查是否已存在
        let mut stmt = conn.prepare(
            "SELECT id FROM wrong_notes WHERE problem_id = ?",
        )?;
        
        let existing: Result<Option<String>, _> = stmt.query_row(
            rusqlite::params![problem_id],
            |row| row.get(0)
        );

        match existing {
            Ok(Some(_)) => {
                // 已存在，更新用户答案
                conn.execute(
                    "UPDATE wrong_notes SET user_answer = ?, last_reviewed_at = ? WHERE problem_id = ?",
                    rusqlite::params![user_answer, now, problem_id],
                )?;
            }
            _ => {
                // 不存在，添加新记录
                conn.execute(
                    "INSERT INTO wrong_notes 
                     (id, problem_id, user_answer, correct_answer, note, review_count, last_reviewed_at)
                     VALUES (?, ?, ?, ?, NULL, 0, ?)",
                    rusqlite::params![id, problem_id, user_answer, correct_answer, now],
                )?;
            }
        }

        Ok(())
    }

    /// 添加错题记录
    pub fn add_wrong_note(&self, note: &WrongNote) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO wrong_notes 
             (id, problem_id, user_answer, correct_answer, note, review_count, last_reviewed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                note.id,
                note.problem_id,
                note.user_answer,
                note.correct_answer,
                note.note,
                note.review_count,
                note.last_reviewed_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// 获取错题本
    pub fn get_wrong_notes(&self, limit: i32) -> Result<Vec<WrongNote>, Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, problem_id, user_answer, correct_answer, note, review_count, last_reviewed_at
             FROM wrong_notes ORDER BY last_reviewed_at DESC LIMIT ?",
        )?;
        
        let rows = stmt.query_map(rusqlite::params![limit], |row| {
            Ok(WrongNote {
                id: row.get(0)?,
                problem_id: row.get(1)?,
                user_answer: row.get(2)?,
                correct_answer: row.get(3)?,
                note: row.get(4)?,
                review_count: row.get(5)?,
                last_reviewed_at: row.get::<_, String>(6)?.parse::<DateTime<Utc>>()?.into(),
            })
        })?;
        
        let mut notes = Vec::new();
        for row in rows {
            notes.push(row?);
        }
        
        Ok(notes)
    }

    /// 增加复习次数
    pub fn increment_review_count(&self, note_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE wrong_notes SET review_count = review_count + 1, last_reviewed_at = ? WHERE id = ?",
            rusqlite::params![now, note_id],
        )?;

        Ok(())
    }

    /// 删除错题记录
    pub fn delete_wrong_note(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.lock().unwrap();

        conn.execute(
            "DELETE FROM wrong_notes WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    // ==================== 辅助方法 ====================

    /// 从数据库行转换为 ProblemSet
    fn row_to_problem_set(row: &Row) -> Result<ProblemSet, rusqlite::Error> {
        Ok(ProblemSet {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            file_path: row.get(3)?,
            total_problems: row.get(4)?,
            created_at: row.get::<_, String>(5)?.parse::<DateTime<Utc>>()?.into(),
        })
    }

    /// 从数据库行转换为 Problem
    fn row_to_problem(row: &Row) -> Result<Problem, rusqlite::Error> {
        Ok(Problem {
            id: row.get(0)?,
            set_id: row.get(1)?,
            number: row.get(2)?,
            question_type: row.get(3)?,
            content: row.get(4)?,
            options: row.get(5)?,
            answer: row.get(6)?,
            explanation: row.get(7)?,
            difficulty: row.get(8)?,
            knowledge_tags: row.get(9)?,
            source_path: row.get(10)?,
            created_at: row.get::<_, String>(11)?.parse::<DateTime<Utc>>()?.into(),
            updated_at: row.get::<_, String>(12)?.parse::<DateTime<Utc>>()?.into(),
        })
    }
}

/// 获取全局数据库实例
pub fn get_database() -> Result<Arc<Mutex<Connection>>, Box<dyn std::error::Error>> {
    let db_path = std::env::current_exe()?
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("data")
        .join("shuati.db");

    // 确保数据目录存在
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let conn = Connection::open(db_path)?;
    Ok(Arc::new(Mutex::new(conn)))
}
