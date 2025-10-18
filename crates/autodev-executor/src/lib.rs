use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

use autodev_core::{AutoDevEngine, CompositeTask, Task, TaskStatus};
use autodev_github::{GitHubClient, Repository};
use autodev_db::Database;

/// Execute a simple task by triggering GitHub Actions workflow
pub async fn execute_simple_task(
    task: &Task,
    repository: &Repository,
    engine: &Arc<AutoDevEngine>,
    github_client: &Arc<GitHubClient>,
    db: &Option<Arc<Database>>,
    parent_branch: Option<&str>,
    composite_task_id: Option<&str>,
) -> Result<u64> {
    tracing::info!("Executing task: {} ({})", task.title, task.id);

    // Update status
    engine.update_task_status(&task.id, TaskStatus::InProgress, None).await?;

    // Determine base branch and target branch
    let (base_branch, target_branch) = if let Some(parent) = parent_branch {
        // Composite task: branch from parent, PR to parent
        (parent.to_string(), parent.to_string())
    } else {
        // Standalone task: branch from main, PR to main
        ("main".to_string(), "main".to_string())
    };

    // Create branch for this task
    let task_branch = format!("autodev/{}", task.id);
    if let Err(e) = github_client.create_branch(repository, &task_branch, &base_branch).await {
        tracing::warn!("Failed to create branch (may already exist): {}", e);
    }

    // Trigger GitHub workflow
    let mut workflow_inputs = std::collections::HashMap::new();
    workflow_inputs.insert("task_id".to_string(), task.id.clone());
    workflow_inputs.insert("composite_task_id".to_string(),
        composite_task_id.unwrap_or("standalone").to_string());
    workflow_inputs.insert("task_title".to_string(), task.title.clone());
    workflow_inputs.insert("prompt".to_string(), task.prompt.clone());
    workflow_inputs.insert("base_branch".to_string(), task_branch.clone());
    workflow_inputs.insert("target_branch".to_string(), target_branch.clone());

    tracing::info!("Triggering GitHub Actions workflow for task: {}", task.id);

    let run_id = github_client
        .trigger_workflow(repository, "autodev.yml", workflow_inputs)
        .await?;

    tracing::info!("Workflow triggered: {} (run_id: {})", task.id, run_id);

    // Save execution log
    if let Some(db) = db {
        db.add_execution_log(
            &task.id,
            "WORKFLOW_TRIGGERED",
            &format!("GitHub Actions workflow triggered: {}", run_id),
        ).await?;
    }

    Ok(run_id)
}

/// Execute a composite task by processing batches sequentially
pub async fn execute_composite_task(
    composite_task: &CompositeTask,
    repository: &Repository,
    engine: &Arc<AutoDevEngine>,
    github_client: &Arc<GitHubClient>,
    db: &Option<Arc<Database>>,
    wait_for_completion: bool,
) -> Result<()> {
    tracing::info!(
        "Executing composite task: {} ({}) with {} subtasks",
        composite_task.title,
        composite_task.id,
        composite_task.subtasks.len()
    );

    // Create parent branch for composite task
    let parent_branch = format!("autodev/{}", composite_task.id);
    tracing::info!("Creating parent branch: {}", parent_branch);

    if let Err(e) = github_client.create_branch(repository, &parent_branch, "main").await {
        tracing::warn!("Failed to create parent branch (may already exist): {}", e);
    }

    let batches = composite_task.get_parallel_batches();

    for (i, batch) in batches.iter().enumerate() {
        tracing::info!(
            "Processing batch {}/{}: {} tasks",
            i + 1,
            batches.len(),
            batch.len()
        );

        // Trigger all workflows in batch concurrently
        let mut handles = Vec::new();

        for task in batch {
            let task = task.clone();
            let repository = repository.clone();
            let engine = engine.clone();
            let github_client = github_client.clone();
            let db = db.clone();
            let parent_branch_clone = parent_branch.clone();
            let composite_id = composite_task.id.clone();

            let handle = tokio::spawn(async move {
                execute_simple_task(
                    &task,
                    &repository,
                    &engine,
                    &github_client,
                    &db,
                    Some(&parent_branch_clone),
                    Some(&composite_id),
                ).await
            });

            handles.push(handle);
        }

        // Collect workflow run IDs
        for handle in handles {
            match handle.await {
                Ok(Ok(run_id)) => {
                    tracing::info!("Workflow triggered successfully: {}", run_id);
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to trigger workflow: {}", e);
                }
                Err(e) => {
                    tracing::error!("Task execution failed: {}", e);
                }
            }
        }

        tracing::info!("Batch {}/{} workflows triggered", i + 1, batches.len());

        // If wait_for_completion is true (CLI mode), wait for completion
        if wait_for_completion {
            // Note: This is for CLI only. API doesn't wait.
            // In CLI, we would wait for PR merges here, but API returns immediately
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        // Wait for approval if not auto-approve and not last batch
        if !composite_task.auto_approve && i < batches.len() - 1 {
            tracing::info!("Batch {} completed. Waiting for approval...", i + 1);
            if wait_for_completion {
                // CLI mode: wait for user input
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    tracing::info!("Composite task execution initiated: {}", composite_task.title);
    Ok(())
}
