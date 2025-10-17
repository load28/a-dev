use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod executor;
mod scheduler;

use autodev_core::{AutoDevEngine, TaskStatus};
use autodev_github::GitHubClient;
use autodev_ai::AIAgent;
use autodev_db::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "autodev_worker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    tracing::info!("Starting AutoDev Worker");

    // Initialize components
    let engine = Arc::new(AutoDevEngine::new());

    let github_token = std::env::var("GITHUB_TOKEN")
        .expect("GITHUB_TOKEN must be set");
    let github_client = Arc::new(GitHubClient::new(github_token)?);

    let ai_agent_type = std::env::var("AI_AGENT_TYPE")
        .unwrap_or_else(|_| "claude-code".to_string());

    let ai_agent: Arc<dyn AIAgent> = match ai_agent_type.as_str() {
        "claude" | "claude-code" => {
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY must be set");
            Arc::new(autodev_ai::ClaudeAgent::new(api_key))
        }
        "gpt-4" | "openai" => {
            tracing::warn!("OpenAI agent not implemented, using Claude instead");
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY must be set");
            Arc::new(autodev_ai::ClaudeAgent::new(api_key))
        }
        _ => {
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY must be set");
            Arc::new(autodev_ai::ClaudeAgent::new(api_key))
        }
    };

    let db = if let Ok(db_url) = std::env::var("DATABASE_URL") {
        let database = Database::new(&db_url).await?;
        Some(Arc::new(database))
    } else {
        None
    };

    // Start worker loop
    let mut ticker = interval(Duration::from_secs(10));

    loop {
        ticker.tick().await;

        // Get ready tasks
        let ready_tasks = engine.get_ready_tasks().await;

        if !ready_tasks.is_empty() {
            tracing::info!("Found {} ready tasks", ready_tasks.len());

            for task in ready_tasks {
                tracing::info!("Processing task: {} - {}", task.id, task.title);

                // Execute task
                let executor = executor::TaskExecutor::new(
                    engine.clone(),
                    github_client.clone(),
                    ai_agent.clone(),
                    db.clone(),
                );

                match executor.execute_task(&task).await {
                    Ok(_) => {
                        tracing::info!("Task {} completed successfully", task.id);
                    }
                    Err(e) => {
                        tracing::error!("Task {} failed: {}", task.id, e);

                        // Update task status
                        let _ = engine
                            .update_task_status(&task.id, TaskStatus::Failed, Some(e.to_string()))
                            .await;

                        // Log to database
                        if let Some(ref db) = db {
                            let _ = db
                                .add_execution_log(&task.id, "FAILED", &e.to_string())
                                .await;
                        }
                    }
                }
            }
        }

        // Check for stalled tasks
        check_stalled_tasks(&engine, &db).await?;

        // Clean up completed tasks periodically
        cleanup_completed_tasks(&engine, &db).await?;
    }
}

async fn check_stalled_tasks(
    engine: &Arc<AutoDevEngine>,
    db: &Option<Arc<Database>>,
) -> Result<()> {
    let tasks = engine.list_active_tasks().await;
    let now = chrono::Utc::now();

    for task in tasks {
        if task.status == TaskStatus::InProgress {
            if let Some(started_at) = task.started_at {
                let duration = now.signed_duration_since(started_at);

                // If task has been running for more than 1 hour, mark as failed
                if duration.num_hours() > 1 {
                    tracing::warn!("Task {} appears to be stalled, marking as failed", task.id);

                    let _ = engine
                        .update_task_status(
                            &task.id,
                            TaskStatus::Failed,
                            Some("Task timed out after 1 hour".to_string()),
                        )
                        .await;

                    if let Some(ref db) = db {
                        let _ = db
                            .add_execution_log(&task.id, "TIMEOUT", "Task timed out after 1 hour")
                            .await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn cleanup_completed_tasks(
    _engine: &Arc<AutoDevEngine>,
    db: &Option<Arc<Database>>,
) -> Result<()> {
    if let Some(ref db) = db {
        // Get tasks completed more than 7 days ago
        let old_tasks = db.get_tasks_by_status(TaskStatus::Completed).await?;
        let now = chrono::Utc::now();

        for task in old_tasks {
            if let Some(completed_at) = task.completed_at {
                let duration = now.signed_duration_since(completed_at);

                if duration.num_days() > 7 {
                    tracing::debug!("Archiving old task: {}", task.id);
                    // In a real implementation, move to archive table
                }
            }
        }
    }

    Ok(())
}