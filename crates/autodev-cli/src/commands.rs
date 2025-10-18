use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

use crate::cli::Commands;
use autodev_core::{AutoDevEngine, CompositeTask, Task, TaskStatus};
use autodev_github::{GitHubClient, Repository};
use autodev_ai::AIAgent;
use autodev_db::Database;
use autodev_executor;

pub async fn execute(
    command: Commands,
    engine: Arc<AutoDevEngine>,
    db: Option<Arc<Database>>,
    github_client: Arc<GitHubClient>,
    ai_agent: Arc<dyn AIAgent>,
) -> Result<()> {
    match command {
        Commands::Task {
            owner,
            repo,
            title,
            description,
            prompt,
            execute,
        } => {
            println!("Creating simple task...");
            let repository = Repository::new(owner.clone(), repo.clone());

            let task = engine
                .create_simple_task(title, description, prompt)
                .await?;

            println!("✓ Task created: {}", task.id);
            println!("  Title: {}", task.title);
            println!("  Status: {:?}", task.status);

            // Save to database
            if let Some(db) = &db {
                db.save_task(&task, &owner, &repo).await?;
                println!("  Saved to database");
            }

            if execute {
                println!("\nExecuting task...");
                let _run_id = execute_task(&task, &repository, &engine, &github_client, &ai_agent, &db, None, None).await?;
                println!();
                println!("⏳ Note: The task will complete asynchronously in GitHub Actions.");
                println!("   You can close this terminal - the workflow will continue running.");
            }
        }

        Commands::Composite {
            owner,
            repo,
            title,
            description,
            prompt,
            auto_approve,
            execute,
        } => {
            println!("Creating composite task...");
            let repository = Repository::new(owner.clone(), repo.clone());

            // Decompose task using AI
            let decomposer = autodev_ai::TaskDecomposer::new(ai_agent.clone());
            let subtasks = decomposer.decompose(&prompt).await?;

            let composite_task = engine
                .create_composite_task(title, description, subtasks, auto_approve)
                .await?;

            println!("✓ Composite task created: {}", composite_task.id);
            println!("  Title: {}", composite_task.title);
            println!("  Subtasks: {}", composite_task.subtasks.len());
            println!("  Auto-approve: {}", composite_task.auto_approve);

            // Display parallel batches
            let batches = composite_task.get_parallel_batches();
            println!("  Parallel execution plan: {} batches", batches.len());
            for (i, batch) in batches.iter().enumerate() {
                let titles: Vec<&str> = batch.iter().map(|t| t.title.as_str()).collect();
                println!("    Batch {}: {:?}", i + 1, titles);
            }

            // Save to database
            if let Some(db) = &db {
                db.save_composite_task(&composite_task, &owner, &repo).await?;
                println!("  Saved to database");
            }

            if execute {
                println!("\nExecuting composite task...");
                execute_composite_task(&composite_task, &repository, &engine, &github_client, &ai_agent, &db).await?;
            }
        }

        Commands::Execute {
            task_id,
            owner,
            repo,
        } => {
            println!("Executing task: {}", task_id);

            let task = engine.get_task(&task_id).await
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

            let repository = Repository::new(owner, repo);
            let _run_id = execute_task(&task, &repository, &engine, &github_client, &ai_agent, &db, None, None).await?;
            println!();
            println!("⏳ Note: The task will complete asynchronously in GitHub Actions.");
            println!("   You can close this terminal - the workflow will continue running.");
        }

        Commands::Status { task_id } => {
            match engine.get_task(&task_id).await {
                Some(task) => {
                    println!("Task: {}", task.id);
                    println!("  Title: {}", task.title);
                    println!("  Status: {:?}", task.status);
                    println!("  Created: {}", task.created_at);

                    if let Some(started) = task.started_at {
                        println!("  Started: {}", started);
                    }

                    if let Some(completed) = task.completed_at {
                        println!("  Completed: {}", completed);

                        if let Some(started) = task.started_at {
                            let duration = completed.signed_duration_since(started);
                            println!("  Duration: {} seconds", duration.num_seconds());
                        }
                    }

                    if let Some(pr_url) = &task.pr_url {
                        println!("  PR: {}", pr_url);
                    }

                    if let Some(error) = &task.error {
                        println!("  Error: {}", error);
                    }

                    // Get logs from database
                    if let Some(db) = &db {
                        let logs = db.get_execution_logs(&task_id).await?;
                        if !logs.is_empty() {
                            println!("\n  Execution Logs:");
                            for log in logs.iter().take(5) {
                                println!("    [{:?}] {}: {}", log.timestamp, log.event_type, log.message);
                            }
                        }

                        // Get metrics
                        if let Some(metrics) = db.get_task_metrics(&task_id).await? {
                            println!("\n  Metrics:");
                            println!("    Execution time: {}ms", metrics.execution_time_ms);
                            println!("    Files changed: {}", metrics.files_changed);
                            println!("    Lines added: {}", metrics.lines_added);
                            println!("    Lines removed: {}", metrics.lines_removed);
                            println!("    AI tokens used: {}", metrics.ai_tokens_used);
                        }
                    }
                }
                None => {
                    println!("Task not found: {}", task_id);

                    // Try database
                    if let Some(db) = &db {
                        if let Some(record) = db.get_task(&task_id).await? {
                            println!("\nFound in database:");
                            println!("  Title: {}", record.title);
                            println!("  Status: {}", record.status);
                            println!("  Repository: {}/{}", record.repository_owner, record.repository_name);
                        }
                    }
                }
            }
        }

        Commands::List { status, limit } => {
            let tasks = engine.list_active_tasks().await;

            let filtered_tasks: Vec<_> = if let Some(status_filter) = status {
                tasks.into_iter()
                    .filter(|t| format!("{:?}", t.status).to_lowercase() == status_filter.to_lowercase())
                    .take(limit)
                    .collect()
            } else {
                tasks.into_iter().take(limit).collect()
            };

            println!("Active Tasks: {}", filtered_tasks.len());
            println!();

            for task in filtered_tasks {
                println!("ID: {}", task.id);
                println!("  Title: {}", task.title);
                println!("  Status: {:?}", task.status);
                println!("  Created: {}", task.created_at);
                if let Some(pr_url) = &task.pr_url {
                    println!("  PR: {}", pr_url);
                }
                println!();
            }

            // Database tasks
            if let Some(db) = &db {
                let recent = db.get_recent_tasks(limit as i64).await?;
                if !recent.is_empty() {
                    println!("\nRecent tasks from database: {}", recent.len());
                    for record in recent.iter().take(5) {
                        println!("  {} - {} ({})", record.id, record.title, record.status);
                    }
                }
            }
        }

        Commands::Serve { port } => {
            println!("Starting API server on port {}...", port);

            if db.is_none() {
                println!("Warning: No database configured. Tasks won't be persisted.");
            }

            // Create API state
            let api_state = autodev_api::state::ApiState {
                engine,
                db,
                github_client,
                ai_agent,
            };

            // Create and run server
            let app = autodev_api::routes::create_router(api_state);

            let addr = format!("0.0.0.0:{}", port);
            println!("🚀 AutoDev API Server running on http://{}", addr);

            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await?;
        }

        Commands::Stats => {
            println!("AutoDev Statistics\n");

            let tasks = engine.list_active_tasks().await;
            let total = tasks.len();
            let completed = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Completed)).count();
            let failed = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Failed)).count();
            let in_progress = tasks.iter().filter(|t| matches!(t.status, TaskStatus::InProgress)).count();

            println!("In-Memory Stats:");
            println!("  Total tasks: {}", total);
            println!("  Completed: {}", completed);
            println!("  Failed: {}", failed);
            println!("  In Progress: {}", in_progress);

            if let Some(db) = &db {
                println!("\nDatabase Stats:");
                let stats = db.get_aggregate_stats().await?;
                println!("  Total tasks: {}", stats.total_tasks);
                println!("  Completed: {}", stats.completed_tasks);
                println!("  Failed: {}", stats.failed_tasks);

                if let Some(avg_time) = stats.avg_execution_time_ms {
                    println!("  Avg execution time: {:.2}s", avg_time / 1000.0);
                }

                if let Some(files) = stats.total_files_changed {
                    println!("  Total files changed: {}", files);
                }

                if let Some(tokens) = stats.total_tokens_used {
                    println!("  Total AI tokens used: {}", tokens);
                }
            }
        }

        Commands::InitDb => {
            match &db {
                Some(database) => {
                    println!("Initializing database schema...");
                    database.init_schema().await?;
                    println!("✓ Database initialized successfully");
                }
                None => {
                    anyhow::bail!("No database URL provided. Set DATABASE_URL environment variable.");
                }
            }
        }
    }

    Ok(())
}

async fn execute_task(
    task: &Task,
    repository: &Repository,
    engine: &Arc<AutoDevEngine>,
    github_client: &Arc<GitHubClient>,
    _ai_agent: &Arc<dyn AIAgent>,
    db: &Option<Arc<Database>>,
    parent_branch: Option<&str>,
    composite_task_id: Option<&str>,
) -> Result<u64> {
    println!("\n{}", "=".repeat(60));
    println!("Executing: {}", task.title);
    println!("{}", "=".repeat(60));

    // Use shared executor module
    let run_id = autodev_executor::execute_simple_task(
        task,
        repository,
        engine,
        github_client,
        db,
        parent_branch,
        composite_task_id,
    ).await?;

    println!("✓ Workflow triggered: {}", run_id);
    println!();
    println!("🤖 Claude 4.5 Sonnet is now running in GitHub Actions (Docker + API).");
    println!("   Check progress at: https://github.com/{}/actions", repository.full_name());
    println!();
    println!("💡 The workflow will:");
    println!("   1. Checkout the repository");
    println!("   2. Run Claude API in Docker container");
    println!("   3. Automatically commit changes");
    println!("   4. Create a pull request");
    println!("   5. Notify AutoDev server on completion");
    println!();
    println!("✓ Task dispatched to GitHub Actions");
    println!("  Task ID: {}", task.id);
    println!("  Workflow Run: {}", run_id);

    Ok(run_id)
}

/// Wait for a task's workflow to complete and PR to merge
async fn wait_for_task_completion(
    task: &Task,
    run_id: u64,
    repository: &Repository,
    github_client: &Arc<GitHubClient>,
) -> Result<()> {
    let task_branch = format!("autodev/{}", task.id);

    print!("  {} ", task.title);
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    // Step 1: Wait for workflow to complete
    let mut last_status = String::new();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        match github_client.get_workflow_run_status(repository, run_id).await {
            Ok(status) => {
                // Only print status changes
                if status.status != last_status {
                    print!(".");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    last_status = status.status.clone();
                }

                if let Some(conclusion) = &status.conclusion {
                    match conclusion.as_str() {
                        "success" => {
                            print!(" ✓ workflow completed");
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            break;
                        }
                        "failure" | "cancelled" | "timed_out" => {
                            println!(" ✗ failed");
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
                // Continue polling
            }
        }
    }

    // Step 2: Wait for PR to be created and merged
    print!(" → waiting for PR merge");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let mut pr_number: Option<u64> = None;
    for _ in 0..20 {  // Max 10 minutes (20 * 30s)
        tokio::time::sleep(Duration::from_secs(30)).await;

        // Find PR by branch
        if pr_number.is_none() {
            if let Ok(Some(num)) = github_client.find_pr_by_branch(repository, &task_branch).await {
                pr_number = Some(num);
                print!(" (PR #{})", num);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }

        // Check if PR is merged
        if let Some(num) = pr_number {
            match github_client.is_pr_merged(repository, num).await {
                Ok(true) => {
                    println!(" → merged ✓");
                    return Ok(());
                }
                Ok(false) => {
                    print!(".");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
                Err(e) => {
                    tracing::warn!("Error checking PR merge status: {}", e);
                }
            }
        }
    }

    // If we get here, PR didn't merge in time
    println!(" ⚠ timeout");
    Err(anyhow::anyhow!("PR did not merge within timeout period"))
}

async fn execute_composite_task(
    composite_task: &CompositeTask,
    repository: &Repository,
    engine: &Arc<AutoDevEngine>,
    github_client: &Arc<GitHubClient>,
    _ai_agent: &Arc<dyn AIAgent>,
    db: &Option<Arc<Database>>,
) -> Result<()> {
    println!("\n{}", "=".repeat(60));
    println!("Executing Composite Task: {}", composite_task.title);
    println!("Subtasks: {}", composite_task.subtasks.len());
    println!("Auto-approve: {}", composite_task.auto_approve);
    println!("{}", "=".repeat(60));

    // Create parent branch for composite task
    let parent_branch = format!("autodev/{}", composite_task.id);
    println!("\nCreating parent branch: {}", parent_branch);

    if let Err(e) = github_client.create_branch(repository, &parent_branch, "main").await {
        tracing::warn!("Failed to create parent branch (may already exist): {}", e);
    } else {
        println!("✓ Parent branch created: {}", parent_branch);
    }

    let batches = composite_task.get_parallel_batches();

    for (i, batch) in batches.iter().enumerate() {
        println!("\n{}", "=".repeat(60));
        println!("Batch {}/{}: {} tasks", i + 1, batches.len(), batch.len());
        println!("{}", "=".repeat(60));
        let titles: Vec<&str> = batch.iter().map(|t| t.title.as_str()).collect();
        for (j, title) in titles.iter().enumerate() {
            println!("  {}. {}", j + 1, title);
        }
        println!();

        // Step 1: Trigger all workflows in batch concurrently using executor module
        println!("🚀 Triggering workflows...");
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
                let run_id = autodev_executor::execute_simple_task(
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

        // Collect all workflow run IDs
        let mut workflow_runs = Vec::new();
        for handle in handles {
            let (task, run_id) = handle.await??;
            workflow_runs.push((task, run_id));
        }

        println!("✓ All workflows triggered");
        println!();

        // Step 2: Wait for all workflows to complete and PRs to merge
        println!("⏳ Waiting for workflows to complete and PRs to merge...");
        for (task, run_id) in workflow_runs {
            wait_for_task_completion(
                &task,
                run_id,
                repository,
                github_client,
            ).await?;
        }

        println!();
        println!("✓ Batch {}/{} fully completed and merged to parent branch", i + 1, batches.len());

        // Wait for approval if not auto-approve and not last batch
        if !composite_task.auto_approve && i < batches.len() - 1 {
            println!();
            println!("{}", "=".repeat(60));
            println!("⚠️  Batch {} completed. Review changes before proceeding.", i + 1);
            println!("   Parent branch: autodev/{}", composite_task.id);
            println!("   Review at: https://github.com/{}/tree/autodev/{}",
                repository.full_name(), composite_task.id);
            println!("{}", "=".repeat(60));
            println!();
            println!("Press Enter to continue to Batch {}...", i + 2);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
        } else if i < batches.len() - 1 {
            println!();
            println!("🚀 Auto-approve enabled, proceeding to Batch {}...", i + 2);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    println!("\n✓ Composite task completed: {}", composite_task.title);

    Ok(())
}