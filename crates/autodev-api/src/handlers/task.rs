use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::ApiState;
use autodev_github::Repository;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub repository_owner: String,
    pub repository_name: String,
    pub title: String,
    pub description: String,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub status: String,
    pub pr_url: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Create a simple task
pub async fn create_task(
    State(state): State<ApiState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = Repository::new(payload.repository_owner.clone(), payload.repository_name.clone());

    match state
        .engine
        .create_simple_task(payload.title, payload.description, payload.prompt)
        .await
    {
        Ok(task) => {
            // Save to database if available
            if let Some(ref db) = state.db {
                if let Err(e) = db
                    .save_task(&task, &repo.owner, &repo.name)
                    .await
                {
                    tracing::error!("Failed to save task to database: {}", e);
                }
            }

            Ok(Json(task_to_response(&task)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

/// Execute a task
pub async fn execute_task(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get task
    let task = match state.engine.get_task(&task_id).await {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Task not found".to_string(),
                }),
            ))
        }
    };

    // Get repository info from database or use default
    let (repo_owner, repo_name) = if let Some(ref db) = state.db {
        match db.get_task(&task_id).await {
            Ok(Some(record)) => (record.repository_owner, record.repository_name),
            _ => ("myorg".to_string(), "myproject".to_string()),
        }
    } else {
        ("myorg".to_string(), "myproject".to_string())
    };

    let repo = Repository::new(repo_owner, repo_name);

    // Execute task asynchronously
    let engine = state.engine.clone();
    let task_clone = task.clone();
    let repo_clone = repo.clone();
    let github = state.github_client.clone();
    let ai = state.ai_agent.clone();
    let db = state.db.clone();

    tokio::spawn(async move {
        // Execute with AI agent
        match ai.execute_task(&task_clone, &repo_clone.full_name()).await {
            Ok(result) => {
                // Trigger GitHub workflow
                let mut inputs = std::collections::HashMap::new();
                inputs.insert("task_id".to_string(), task_clone.id.clone());
                inputs.insert("branch".to_string(), result.pr_branch);
                inputs.insert("commit_message".to_string(), result.commit_message);

                if let Ok(run_id) = github
                    .trigger_workflow(&repo_clone, "autodev.yml", inputs)
                    .await
                {
                    // Update task status
                    if let Err(e) = engine
                        .update_task_status(
                            &task_clone.id,
                            autodev_core::TaskStatus::Completed,
                            None,
                        )
                        .await
                    {
                        tracing::error!("Failed to update task status: {}", e);
                    }

                    // Update database
                    if let Some(db) = db {
                        let _ = db.update_task_status(
                            &task_clone.id,
                            autodev_core::TaskStatus::Completed,
                            None,
                        ).await;
                    }

                    tracing::info!("Task {} completed with workflow {}", task_clone.id, run_id);
                }
            }
            Err(e) => {
                tracing::error!("Task execution failed: {}", e);
                let _ = engine
                    .update_task_status(
                        &task_clone.id,
                        autodev_core::TaskStatus::Failed,
                        Some(e.to_string()),
                    )
                    .await;
            }
        }
    });

    Ok(Json(task_to_response(&task)))
}

/// Get task status
pub async fn get_task_status(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.engine.get_task(&task_id).await {
        Some(task) => Ok(Json(task_to_response(&task))),
        None => {
            // Try database
            if let Some(ref db) = state.db {
                if let Ok(Some(record)) = db.get_task(&task_id).await {
                    return Ok(Json(TaskResponse {
                        id: record.id,
                        title: record.title,
                        status: record.status,
                        pr_url: record.pr_url,
                        created_at: record.created_at.to_rfc3339(),
                        completed_at: record.completed_at.map(|dt| dt.to_rfc3339()),
                    }));
                }
            }

            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Task not found".to_string(),
                }),
            ))
        }
    }
}

/// List all active tasks
pub async fn list_tasks(
    State(state): State<ApiState>,
) -> Result<Json<Vec<TaskResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let tasks = state.engine.list_active_tasks().await;
    let responses: Vec<TaskResponse> = tasks.iter().map(task_to_response).collect();
    Ok(Json(responses))
}

pub fn task_to_response(task: &autodev_core::Task) -> TaskResponse {
    TaskResponse {
        id: task.id.clone(),
        title: task.title.clone(),
        status: format!("{:?}", task.status),
        pr_url: task.pr_url.clone(),
        created_at: task.created_at.to_rfc3339(),
        completed_at: task.completed_at.map(|dt| dt.to_rfc3339()),
    }
}