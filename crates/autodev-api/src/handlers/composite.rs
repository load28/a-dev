use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::ApiState;
use autodev_github::Repository;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCompositeTaskRequest {
    pub repository_owner: String,
    pub repository_name: String,
    pub title: String,
    pub description: String,
    pub composite_prompt: String,
    pub auto_approve: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompositeTaskResponse {
    pub id: String,
    pub title: String,
    pub subtasks: Vec<crate::handlers::task::TaskResponse>,
    pub batches: Vec<Vec<String>>, // Task IDs in each batch
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Create a composite task and execute it immediately
pub async fn create_composite_task(
    State(state): State<ApiState>,
    Json(payload): Json<CreateCompositeTaskRequest>,
) -> Result<Json<CompositeTaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = Repository::new(
        payload.repository_owner.clone(),
        payload.repository_name.clone(),
    );

    // Use AI to decompose the task
    let decomposer = autodev_ai::TaskDecomposer::new(state.ai_agent.clone());

    match decomposer.decompose(&payload.composite_prompt).await {
        Ok(subtasks) => {
            match state
                .engine
                .create_composite_task(
                    payload.title,
                    payload.description,
                    subtasks,
                    payload.auto_approve,
                )
                .await
            {
                Ok(composite_task) => {
                    // Save to database if available
                    if let Some(ref db) = state.db {
                        if let Err(e) = db
                            .save_composite_task(&composite_task, &repo.owner, &repo.name)
                            .await
                        {
                            tracing::error!("Failed to save composite task to database: {}", e);
                        }
                    }

                    // Execute composite task immediately in background using Docker executor
                    let composite_clone = composite_task.clone();
                    let repo_clone = repo.clone();
                    let engine_clone = state.engine.clone();
                    let github_clone = state.github_client.clone();
                    let db_clone = state.db.clone();
                    let docker_clone = state.docker_executor.clone();

                    tokio::spawn(async move {
                        if let Err(e) = autodev_executor::execute_composite_task_docker(
                            &composite_clone,
                            &repo_clone,
                            &engine_clone,
                            &github_clone,
                            &db_clone,
                            &docker_clone,
                        ).await {
                            tracing::error!("Failed to execute composite task {}: {}", composite_clone.id, e);
                        }
                    });

                    Ok(Json(composite_task_to_response(&composite_task)))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: e.to_string(),
                    }),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to decompose task: {}", e),
            }),
        )),
    }
}

/// Get composite task
pub async fn get_composite_task(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<Json<CompositeTaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.engine.get_composite_task(&task_id).await {
        Some(composite_task) => Ok(Json(composite_task_to_response(&composite_task))),
        None => {
            // Try database
            if let Some(ref db) = state.db {
                if let Ok(Some(record)) = db.get_composite_task(&task_id).await {
                    // Get subtasks
                    if let Ok(subtasks) = db.get_composite_subtasks(&task_id).await {
                        let subtask_responses: Vec<crate::handlers::task::TaskResponse> =
                            subtasks.iter().map(|t| crate::handlers::task::TaskResponse {
                                id: t.id.clone(),
                                title: t.title.clone(),
                                status: t.status.clone(),
                                pr_url: t.pr_url.clone(),
                                created_at: t.created_at.to_rfc3339(),
                                completed_at: t.completed_at.map(|dt| dt.to_rfc3339()),
                            }).collect();

                        return Ok(Json(CompositeTaskResponse {
                            id: record.id,
                            title: record.title,
                            subtasks: subtask_responses,
                            batches: vec![],
                        }));
                    }
                }
            }

            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Composite task not found".to_string(),
                }),
            ))
        }
    }
}

/// Execute composite task
pub async fn execute_composite_task(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<Json<CompositeTaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get composite task
    let composite_task = match state.engine.get_composite_task(&task_id).await {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Composite task not found".to_string(),
                }),
            ))
        }
    };

    // Get repository info
    let (repo_owner, repo_name) = if let Some(ref db) = state.db {
        match db.get_composite_task(&task_id).await {
            Ok(Some(record)) => (record.repository_owner, record.repository_name),
            _ => ("myorg".to_string(), "myproject".to_string()),
        }
    } else {
        ("myorg".to_string(), "myproject".to_string())
    };

    let repo = Repository::new(repo_owner, repo_name);

    // Execute composite task asynchronously using Docker executor
    let composite_clone = composite_task.clone();
    let repo_clone = repo.clone();
    let engine_clone = state.engine.clone();
    let github_clone = state.github_client.clone();
    let db_clone = state.db.clone();
    let docker_clone = state.docker_executor.clone();

    tokio::spawn(async move {
        if let Err(e) = autodev_executor::execute_composite_task_docker(
            &composite_clone,
            &repo_clone,
            &engine_clone,
            &github_clone,
            &db_clone,
            &docker_clone,
        ).await {
            tracing::error!("Failed to execute composite task {}: {}", composite_clone.id, e);
        }
    });

    Ok(Json(composite_task_to_response(&composite_task)))
}

fn composite_task_to_response(composite_task: &autodev_core::CompositeTask) -> CompositeTaskResponse {
    let subtasks: Vec<crate::handlers::task::TaskResponse> = composite_task
        .subtasks
        .iter()
        .map(crate::handlers::task::task_to_response)
        .collect();

    let batches: Vec<Vec<String>> = composite_task
        .get_parallel_batches()
        .iter()
        .map(|batch| batch.iter().map(|t| t.id.clone()).collect())
        .collect();

    CompositeTaskResponse {
        id: composite_task.id.clone(),
        title: composite_task.title.clone(),
        subtasks,
        batches,
    }
}