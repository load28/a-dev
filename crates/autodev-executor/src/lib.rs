use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

use autodev_core::{AutoDevEngine, CompositeTask, Task, TaskStatus};
use autodev_github::{GitHubClient, Repository};
use autodev_db::Database;

/// Wait for a batch of tasks to complete (workflow + PR merge)
async fn wait_for_batch_completion(
    workflow_runs: Vec<(Task, u64)>,
    repository: &Repository,
    github_client: &Arc<GitHubClient>,
) -> Result<()> {
    for (task, run_id) in workflow_runs {
        let task_branch = format!("autodev/{}", task.id);

        tracing::info!("Waiting for task {} to complete...", task.title);

        // Step 1: Wait for workflow to complete
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            match github_client.get_workflow_run_status(repository, run_id).await {
                Ok(status) => {
                    if let Some(conclusion) = &status.conclusion {
                        match conclusion.as_str() {
                            "success" => {
                                tracing::info!("Workflow completed for task: {}", task.title);
                                break;
                            }
                            "failure" | "cancelled" | "timed_out" => {
                                tracing::error!("Workflow failed for task {}: {}", task.title, conclusion);
                                return Err(anyhow::anyhow!(
                                    "Workflow failed with conclusion: {}",
                                    conclusion
                                ));
                            }
                            _ => {
                                // Still running or other state
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Error checking workflow status: {}", e);
                }
            }
        }

        // Step 2: Wait for PR to be created and merged
        tracing::info!("Waiting for PR merge for task: {}", task.title);
        let mut pr_number: Option<u64> = None;

        for _ in 0..20 {  // Max 10 minutes (20 * 30s)
            tokio::time::sleep(Duration::from_secs(30)).await;

            // Find PR by branch
            if pr_number.is_none() {
                if let Ok(Some(num)) = github_client.find_pr_by_branch(repository, &task_branch).await {
                    pr_number = Some(num);
                    tracing::info!("Found PR #{} for task: {}", num, task.title);
                }
            }

            // Check if PR is merged
            if let Some(num) = pr_number {
                match github_client.is_pr_merged(repository, num).await {
                    Ok(true) => {
                        tracing::info!("PR #{} merged for task: {}", num, task.title);
                        break;
                    }
                    Ok(false) => {
                        // Still waiting for merge
                    }
                    Err(e) => {
                        tracing::warn!("Error checking PR merge status: {}", e);
                    }
                }
            }
        }

        if pr_number.is_none() {
            return Err(anyhow::anyhow!("PR not found for task: {}", task.title));
        }
    }

    Ok(())
}

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
                let run_id = execute_simple_task(
                    &task,
                    &repository,
                    &engine,
                    &github_client,
                    &db,
                    Some(&parent_branch_clone),
                    Some(&composite_id),
                ).await?;
                Ok::<(Task, u64), anyhow::Error>((task, run_id))
            });

            handles.push(handle);
        }

        // Collect workflow run IDs
        let mut workflow_runs = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok((task, run_id))) => {
                    tracing::info!("Workflow triggered successfully for {}: {}", task.title, run_id);
                    workflow_runs.push((task, run_id));
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to trigger workflow: {}", e);
                    return Err(e);
                }
                Err(e) => {
                    tracing::error!("Task execution failed: {}", e);
                    return Err(anyhow::anyhow!("Task execution failed: {}", e));
                }
            }
        }

        tracing::info!("Batch {}/{} workflows triggered", i + 1, batches.len());

        // Wait for all workflows and PRs in this batch to complete
        wait_for_batch_completion(workflow_runs, repository, github_client).await?;

        tracing::info!("Batch {}/{} completed and merged", i + 1, batches.len());
    }

    tracing::info!("Composite task execution initiated: {}", composite_task.title);
    Ok(())
}
