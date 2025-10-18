use anyhow::Result;
use std::sync::Arc;

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

    // Use shared executor module
    autodev_executor::execute_composite_task(
        composite_task,
        repository,
        engine,
        github_client,
        db,
    ).await?;

    println!("\n✓ Composite task completed: {}", composite_task.title);

    Ok(())
}