use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub difficulty: Difficulty,
    pub category: String,
    pub tags: Vec<String>,
    pub answer: Option<String>,
    pub explanation: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemSet {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub problem_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgress {
    pub problem_id: i64,
    pub status: ProgressStatus,
    pub attempts: i32,
    pub last_attempt: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgressStatus {
    NotStarted,
    InProgress,
    Completed,
    Reviewing,
}
