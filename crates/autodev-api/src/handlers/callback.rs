use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::ApiState;
use autodev_github::Repository;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowCompleteRequest {
    pub task_id: String,
    pub composite_task_id: String,
    pub repository_owner: String,
    pub repository_name: String,
    pub pr_number: Option<u64>,
    pub pr_url: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowCompleteResponse {
    pub message: String,
    pub next_tasks_started: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Handle workflow completion callback
pub async fn workflow_complete(
    State(state): State<ApiState>,
    Json(payload): Json<WorkflowCompleteRequest>,
) -> Result<Json<WorkflowCompleteResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!(
        "Workflow complete callback for task {} (composite: {})",
        payload.task_id,
        payload.composite_task_id
    );

    // Update task status
    let status = if payload.success {
        autodev_core::TaskStatus::Completed
    } else {
        autodev_core::TaskStatus::Failed
    };

    if let Err(e) = state
        .engine
        .update_task_status(&payload.task_id, status, payload.error.clone())
        .await
    {
        tracing::error!("Failed to update task status: {}", e);
    }

    // Update PR URL if available
    if let Some(_task) = state.engine.get_task(&payload.task_id).await {
        if let Some(ref pr_url) = payload.pr_url {
            // Store PR URL in task
            // Note: We need to add pr_url field update capability to the engine
            tracing::info!("Task {} PR created: {}", payload.task_id, pr_url);
        }
    }

    // Update database if available
    if let Some(ref db) = state.db {
        let _ = db
            .update_task_status(&payload.task_id, status, payload.error.clone())
            .await;
    }

    // If the task succeeded and has PR, auto-merge if it's a subtask
    if payload.success && payload.pr_number.is_some() && payload.composite_task_id != "standalone" {
        let repo = Repository::new(
            payload.repository_owner.clone(),
            payload.repository_name.clone(),
        );

        // Auto-merge subtask PR to parent branch
        if let Some(pr_number) = payload.pr_number {
            tracing::info!(
                "Auto-merging subtask PR #{} for task {} to parent branch",
                pr_number,
                payload.task_id
            );

            match state.github_client.merge_pull_request(&repo, pr_number).await {
                Ok(_) => {
                    tracing::info!(
                        "✓ Subtask PR #{} auto-merged to parent branch",
                        pr_number
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to auto-merge subtask PR #{}: {}",
                        pr_number,
                        e
                    );
                }
            }
        }
    }

    // If the task succeeded, check if we can start dependent tasks
    let mut next_tasks = Vec::new();

    if payload.success {
        // Get composite task
        if let Some(composite_task) = state.engine.get_composite_task(&payload.composite_task_id).await {
            let repo = Repository::new(
                payload.repository_owner.clone(),
                payload.repository_name.clone(),
            );

            // Get all ready tasks (dependencies satisfied)
            let ready_tasks = state.engine.get_ready_tasks().await;

            // Filter tasks that belong to this composite task
            let composite_task_ids: Vec<String> = composite_task
                .subtasks
                .iter()
                .map(|t| t.id.clone())
                .collect();

            let ready_in_composite: Vec<_> = ready_tasks
                .into_iter()
                .filter(|t| composite_task_ids.contains(&t.id))
                .collect();

            // Start workflows for ready tasks
            let parent_branch = format!("autodev/{}", composite_task.id);

            for task in ready_in_composite {
                let task_branch = format!("autodev/{}/subtask-{}", composite_task.id, task.id);

                // Create branch for this subtask
                if let Err(e) = state
                    .github_client
                    .create_branch(&repo, &task_branch, &parent_branch)
                    .await
                {
                    tracing::error!("Failed to create branch for subtask {}: {}", task.id, e);
                    continue;
                }

                // Dispatch workflow
                let mut inputs = std::collections::HashMap::new();
                inputs.insert("task_id".to_string(), task.id.clone());
                inputs.insert("composite_task_id".to_string(), composite_task.id.clone());
                inputs.insert("task_title".to_string(), task.title.clone());
                inputs.insert("prompt".to_string(), task.prompt.clone());
                inputs.insert("base_branch".to_string(), task_branch.clone());
                inputs.insert("target_branch".to_string(), parent_branch.clone());

                match state
                    .github_client
                    .trigger_workflow(&repo, "autodev-subtask.yml", inputs)
                    .await
                {
                    Ok(workflow_run_id) => {
                        tracing::info!(
                            "Started workflow {} for dependent subtask {}",
                            workflow_run_id,
                            task.id
                        );
                        next_tasks.push(task.id.clone());

                        // Update task status
                        let _ = state
                            .engine
                            .update_task_status(&task.id, autodev_core::TaskStatus::InProgress, None)
                            .await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to trigger workflow for subtask {}: {}", task.id, e);
                    }
                }
            }

            // Check if all tasks in composite task are complete
            if composite_task.is_completed() {
                tracing::info!(
                    "Composite task {} is fully completed! Creating DRAFT PR to main for user review.",
                    composite_task.id
                );

                // Create DRAFT PR from parent branch to main (requires user approval)
                let parent_branch = format!("autodev/{}", composite_task.id);
                let pr_body = format!(
                    "# {}\n\n## ⚠️ Review Required\n\
                    This is an automatically generated composite task PR. Please review all changes before merging.\n\n\
                    ## Description\n{}\n\n\
                    ## Subtasks Completed\n{}\n\n\
                    ## Review Checklist\n\
                    - [ ] All subtask PRs reviewed and verified\n\
                    - [ ] Code quality meets standards\n\
                    - [ ] Tests passing\n\
                    - [ ] No security issues\n\n\
                    ## Next Steps\n\
                    1. Review all changes in this PR\n\
                    2. If satisfied, mark as \"Ready for Review\"\n\
                    3. Merge when approved\n\n\
                    ---\n\
                    🤖 Generated by AutoDev\n\
                    Co-Authored-By: Claude <noreply@anthropic.com>",
                    composite_task.title,
                    composite_task.description,
                    composite_task
                        .subtasks
                        .iter()
                        .map(|t| format!("- [x] {} ({})", t.title, t.id))
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                match state
                    .github_client
                    .create_pull_request(
                        &repo,
                        format!("[AutoDev] {}", composite_task.title),
                        pr_body,
                        parent_branch,
                        "main".to_string(),
                        true,  // draft = true (requires user approval)
                    )
                    .await
                {
                    Ok(pr) => {
                        tracing::info!(
                            "Created DRAFT PR #{} for composite task {} - awaiting user review",
                            pr.number,
                            composite_task.id
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to create DRAFT PR for composite task {}: {}",
                            composite_task.id,
                            e
                        );
                    }
                }
            }
        }
    }

    Ok(Json(WorkflowCompleteResponse {
        message: format!("Task {} processed successfully", payload.task_id),
        next_tasks_started: next_tasks,
    }))
}
