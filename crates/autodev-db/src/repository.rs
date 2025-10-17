use crate::{
    models::{AggregateStats, CompositeTaskRecord, ExecutionLog, Metrics, TaskRecord},
    Result,
};
use autodev_core::{CompositeTask, Task, TaskStatus};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[derive(Clone)]
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    /// Create new database connection
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Initialize database schema
    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id VARCHAR(255) PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                prompt TEXT NOT NULL,
                task_type VARCHAR(50) NOT NULL,
                status VARCHAR(50) NOT NULL,
                dependencies TEXT[] NOT NULL DEFAULT '{}',
                repository_owner VARCHAR(255) NOT NULL,
                repository_name VARCHAR(255) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                started_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                pr_url TEXT,
                workflow_run_id VARCHAR(255),
                error TEXT,
                auto_approve BOOLEAN NOT NULL DEFAULT FALSE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS composite_tasks (
                id VARCHAR(255) PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                auto_approve BOOLEAN NOT NULL DEFAULT FALSE,
                repository_owner VARCHAR(255) NOT NULL,
                repository_name VARCHAR(255) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                completed_at TIMESTAMPTZ
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS composite_task_subtasks (
                composite_task_id VARCHAR(255) NOT NULL,
                subtask_id VARCHAR(255) NOT NULL,
                subtask_order INTEGER NOT NULL,
                PRIMARY KEY (composite_task_id, subtask_id),
                FOREIGN KEY (composite_task_id) REFERENCES composite_tasks(id),
                FOREIGN KEY (subtask_id) REFERENCES tasks(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS execution_logs (
                id SERIAL PRIMARY KEY,
                task_id VARCHAR(255) NOT NULL,
                event_type VARCHAR(100) NOT NULL,
                message TEXT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics (
                id SERIAL PRIMARY KEY,
                task_id VARCHAR(255) NOT NULL,
                execution_time_ms BIGINT NOT NULL,
                files_changed INTEGER NOT NULL DEFAULT 0,
                lines_added INTEGER NOT NULL DEFAULT 0,
                lines_removed INTEGER NOT NULL DEFAULT 0,
                ai_tokens_used INTEGER NOT NULL DEFAULT 0,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at DESC)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_execution_logs_task_id ON execution_logs(task_id)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Task Operations
    // ========================================================================

    /// Save task
    pub async fn save_task(&self, task: &Task, repo_owner: &str, repo_name: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tasks (
                id, title, description, prompt, task_type, status,
                dependencies, repository_owner, repository_name,
                created_at, started_at, completed_at, pr_url,
                workflow_run_id, error, auto_approve
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (id) DO UPDATE SET
                status = $6,
                started_at = $11,
                completed_at = $12,
                pr_url = $13,
                workflow_run_id = $14,
                error = $15
            "#,
        )
        .bind(&task.id)
        .bind(&task.title)
        .bind(&task.description)
        .bind(&task.prompt)
        .bind(format!("{:?}", task.task_type))
        .bind(format!("{:?}", task.status))
        .bind(&task.dependencies)
        .bind(repo_owner)
        .bind(repo_name)
        .bind(task.created_at)
        .bind(task.started_at)
        .bind(task.completed_at)
        .bind(&task.pr_url)
        .bind(&task.workflow_run_id)
        .bind(&task.error)
        .bind(task.auto_approve)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get task by ID
    pub async fn get_task(&self, task_id: &str) -> Result<Option<TaskRecord>> {
        let record = sqlx::query_as::<_, TaskRecord>("SELECT * FROM tasks WHERE id = $1")
            .bind(task_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(record)
    }

    /// Get tasks by status
    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<TaskRecord>> {
        let records = sqlx::query_as::<_, TaskRecord>(
            "SELECT * FROM tasks WHERE status = $1 ORDER BY created_at DESC",
        )
        .bind(format!("{:?}", status))
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Get recent tasks
    pub async fn get_recent_tasks(&self, limit: i64) -> Result<Vec<TaskRecord>> {
        let records = sqlx::query_as::<_, TaskRecord>(
            "SELECT * FROM tasks ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Update task status
    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        error: Option<String>,
    ) -> Result<()> {
        sqlx::query("UPDATE tasks SET status = $1, error = $2 WHERE id = $3")
            .bind(format!("{:?}", status))
            .bind(error)
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ========================================================================
    // Composite Task Operations
    // ========================================================================

    /// Save composite task
    pub async fn save_composite_task(
        &self,
        composite_task: &CompositeTask,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<()> {
        // Save composite task
        sqlx::query(
            r#"
            INSERT INTO composite_tasks (
                id, title, description, auto_approve,
                repository_owner, repository_name, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&composite_task.id)
        .bind(&composite_task.title)
        .bind(&composite_task.description)
        .bind(composite_task.auto_approve)
        .bind(repo_owner)
        .bind(repo_name)
        .bind(composite_task.created_at)
        .execute(&self.pool)
        .await?;

        // Save subtasks
        for (order, subtask) in composite_task.subtasks.iter().enumerate() {
            self.save_task(subtask, repo_owner, repo_name).await?;

            sqlx::query(
                r#"
                INSERT INTO composite_task_subtasks (
                    composite_task_id, subtask_id, subtask_order
                ) VALUES ($1, $2, $3)
                "#,
            )
            .bind(&composite_task.id)
            .bind(&subtask.id)
            .bind(order as i32)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get composite task by ID
    pub async fn get_composite_task(&self, task_id: &str) -> Result<Option<CompositeTaskRecord>> {
        let record = sqlx::query_as::<_, CompositeTaskRecord>(
            "SELECT * FROM composite_tasks WHERE id = $1",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get composite task's subtasks
    pub async fn get_composite_subtasks(&self, composite_task_id: &str) -> Result<Vec<TaskRecord>> {
        let records = sqlx::query_as::<_, TaskRecord>(
            r#"
            SELECT t.* FROM tasks t
            JOIN composite_task_subtasks cts ON t.id = cts.subtask_id
            WHERE cts.composite_task_id = $1
            ORDER BY cts.subtask_order
            "#,
        )
        .bind(composite_task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    // ========================================================================
    // Logging Operations
    // ========================================================================

    /// Add execution log
    pub async fn add_execution_log(
        &self,
        task_id: &str,
        event_type: &str,
        message: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO execution_logs (task_id, event_type, message, timestamp)
            VALUES ($1, $2, $3, NOW())
            "#,
        )
        .bind(task_id)
        .bind(event_type)
        .bind(message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get task execution logs
    pub async fn get_execution_logs(&self, task_id: &str) -> Result<Vec<ExecutionLog>> {
        let logs = sqlx::query_as::<_, ExecutionLog>(
            "SELECT * FROM execution_logs WHERE task_id = $1 ORDER BY timestamp DESC",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    // ========================================================================
    // Metrics Operations
    // ========================================================================

    /// Save metrics
    pub async fn save_metrics(
        &self,
        task_id: &str,
        execution_time_ms: i64,
        files_changed: i32,
        lines_added: i32,
        lines_removed: i32,
        ai_tokens_used: i32,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO metrics (
                task_id, execution_time_ms, files_changed,
                lines_added, lines_removed, ai_tokens_used, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#,
        )
        .bind(task_id)
        .bind(execution_time_ms)
        .bind(files_changed)
        .bind(lines_added)
        .bind(lines_removed)
        .bind(ai_tokens_used)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get task metrics
    pub async fn get_task_metrics(&self, task_id: &str) -> Result<Option<Metrics>> {
        let metrics = sqlx::query_as::<_, Metrics>(
            "SELECT * FROM metrics WHERE task_id = $1 ORDER BY timestamp DESC LIMIT 1",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(metrics)
    }

    /// Get aggregate statistics
    pub async fn get_aggregate_stats(&self) -> Result<AggregateStats> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_tasks,
                COUNT(CASE WHEN status = 'Completed' THEN 1 END) as completed_tasks,
                COUNT(CASE WHEN status = 'Failed' THEN 1 END) as failed_tasks,
                AVG(CASE
                    WHEN completed_at IS NOT NULL AND started_at IS NOT NULL
                    THEN EXTRACT(EPOCH FROM (completed_at - started_at)) * 1000
                END) as avg_execution_time_ms,
                SUM(m.files_changed) as total_files_changed,
                SUM(m.ai_tokens_used) as total_tokens_used
            FROM tasks t
            LEFT JOIN metrics m ON t.id = m.task_id
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AggregateStats {
            total_tasks: row.get("total_tasks"),
            completed_tasks: row.get("completed_tasks"),
            failed_tasks: row.get("failed_tasks"),
            avg_execution_time_ms: row.get("avg_execution_time_ms"),
            total_files_changed: row.get("total_files_changed"),
            total_tokens_used: row.get("total_tokens_used"),
        })
    }
}