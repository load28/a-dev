use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskRecord {
    pub id: String,
    pub title: String,
    pub description: String,
    pub prompt: String,
    pub task_type: String,
    pub status: String,
    pub dependencies: Vec<String>,
    pub repository_owner: String,
    pub repository_name: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub pr_url: Option<String>,
    pub workflow_run_id: Option<String>,
    pub error: Option<String>,
    pub auto_approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CompositeTaskRecord {
    pub id: String,
    pub title: String,
    pub description: String,
    pub auto_approve: bool,
    pub repository_owner: String,
    pub repository_name: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionLog {
    pub id: i32,
    pub task_id: String,
    pub event_type: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Metrics {
    pub id: i32,
    pub task_id: String,
    pub execution_time_ms: i64,
    pub files_changed: i32,
    pub lines_added: i32,
    pub lines_removed: i32,
    pub ai_tokens_used: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateStats {
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub avg_execution_time_ms: Option<f64>,
    pub total_files_changed: Option<i64>,
    pub total_tokens_used: Option<i64>,
}