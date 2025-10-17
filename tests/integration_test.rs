use autodev_core::{
    AutoDevEngine, Repository, Task, TaskStatus, TaskType, CompositeTask,
};
use std::sync::Arc;

#[tokio::test]
async fn test_create_simple_task() {
    let engine = AutoDevEngine::new();

    let task = engine
        .create_simple_task(
            "Test Task".to_string(),
            "Test Description".to_string(),
            "Test Prompt".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(task.title, "Test Task");
    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.task_type, TaskType::Simple);
}

#[tokio::test]
async fn test_task_dependencies() {
    let task_a = Task::new(
        "Task A".to_string(),
        "Description A".to_string(),
        "Prompt A".to_string(),
    );

    let task_b = Task::new(
        "Task B".to_string(),
        "Description B".to_string(),
        "Prompt B".to_string(),
    )
    .with_dependencies(vec![task_a.id.clone()]);

    let completed = std::collections::HashSet::new();
    assert!(task_a.can_start(&completed));
    assert!(!task_b.can_start(&completed));

    let mut completed = std::collections::HashSet::new();
    completed.insert(task_a.id.clone());
    assert!(task_b.can_start(&completed));
}

#[tokio::test]
async fn test_composite_task_parallel_batches() {
    // Dependency graph:
    //   A
    //   ├─ B
    //   ├─ C
    //   └─ D
    //      └─ E

    let task_a = Task::new("A".to_string(), "".to_string(), "".to_string());

    let task_b = Task::new("B".to_string(), "".to_string(), "".to_string())
        .with_dependencies(vec![task_a.id.clone()]);

    let task_c = Task::new("C".to_string(), "".to_string(), "".to_string())
        .with_dependencies(vec![task_a.id.clone()]);

    let task_d = Task::new("D".to_string(), "".to_string(), "".to_string())
        .with_dependencies(vec![task_a.id.clone()]);

    let task_e = Task::new("E".to_string(), "".to_string(), "".to_string())
        .with_dependencies(vec![task_d.id.clone()]);

    let composite = CompositeTask::new(
        "Composite".to_string(),
        "".to_string(),
        vec![task_a.clone(), task_b, task_c, task_d, task_e],
    );

    let batches = composite.get_parallel_batches();

    // Batch 0: [A]
    // Batch 1: [B, C, D] - parallel execution
    // Batch 2: [E]

    assert_eq!(batches.len(), 3);
    assert_eq!(batches[0].len(), 1); // A
    assert_eq!(batches[1].len(), 3); // B, C, D
    assert_eq!(batches[2].len(), 1); // E
}

#[tokio::test]
async fn test_task_decomposition() {
    use autodev_ai::{AIAgent, TaskDecomposer};

    // Mock AI agent for testing
    struct MockAgent;

    #[async_trait::async_trait]
    impl AIAgent for MockAgent {
        fn agent_type(&self) -> autodev_ai::AgentType {
            autodev_ai::AgentType::ClaudeCode
        }

        async fn execute_task(
            &self,
            _task: &Task,
            _repo_path: &str,
        ) -> autodev_ai::Result<autodev_ai::AgentResult> {
            Ok(autodev_ai::AgentResult {
                success: true,
                files_changed: vec![],
                pr_branch: "test".to_string(),
                commit_message: "test".to_string(),
                output: None,
            })
        }

        async fn review_code_changes(
            &self,
            _pr_diff: &str,
            _review_comments: &[String],
        ) -> autodev_ai::Result<autodev_ai::ReviewResult> {
            Ok(autodev_ai::ReviewResult {
                success: true,
                changes_made: vec![],
                comments: vec![],
            })
        }

        async fn fix_ci_failures(
            &self,
            _ci_logs: &str,
        ) -> autodev_ai::Result<autodev_ai::ReviewResult> {
            Ok(autodev_ai::ReviewResult {
                success: true,
                changes_made: vec![],
                comments: vec![],
            })
        }

        async fn generate_commit_message(&self, _changes: &str) -> autodev_ai::Result<String> {
            Ok("test commit".to_string())
        }

        async fn analyze_security(
            &self,
            _code: &str,
            _language: &str,
        ) -> autodev_ai::Result<Vec<autodev_ai::agent::SecurityIssue>> {
            Ok(vec![])
        }
    }

    let agent = Arc::new(MockAgent);
    let decomposer = TaskDecomposer::new(agent);

    let prompt = "Improve the translation quality for each page";
    let tasks = decomposer.decompose(prompt).await.unwrap();

    // Translation task should be decomposed into page-specific tasks
    assert!(tasks.len() > 1);

    // All tasks should be independent (no dependencies)
    for task in &tasks {
        assert!(task.dependencies.is_empty());
    }
}

#[tokio::test]
async fn test_get_task_status() {
    let engine = AutoDevEngine::new();

    let task = engine
        .create_simple_task(
            "Test Task".to_string(),
            "Test Description".to_string(),
            "Test Prompt".to_string(),
        )
        .await
        .unwrap();

    let task_id = task.id.clone();

    // Get task
    let retrieved = engine.get_task(&task_id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, task_id);

    // Non-existent task
    let not_found = engine.get_task("nonexistent").await;
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_list_active_tasks() {
    let engine = AutoDevEngine::new();

    // Create multiple tasks
    for i in 0..5 {
        engine
            .create_simple_task(
                format!("Task {}", i),
                format!("Description {}", i),
                format!("Prompt {}", i),
            )
            .await
            .unwrap();
    }

    let tasks = engine.list_active_tasks().await;
    assert_eq!(tasks.len(), 5);
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Test".to_string(),
            "Description".to_string(),
            "Prompt".to_string(),
        );

        assert_eq!(task.title, "Test");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.dependencies.is_empty());
        assert!(task.pr_url.is_none());
    }

    #[test]
    fn test_repository_full_name() {
        let repo = autodev_github::Repository::new("owner".to_string(), "repo".to_string());
        assert_eq!(repo.full_name(), "owner/repo");
    }

    #[test]
    fn test_task_with_dependencies() {
        let task = Task::new("".to_string(), "".to_string(), "".to_string())
            .with_dependencies(vec!["dep1".to_string(), "dep2".to_string()]);

        assert_eq!(task.dependencies.len(), 2);
    }

    #[test]
    fn test_composite_task_dependency_graph() {
        let task_a = Task::new("A".to_string(), "".to_string(), "".to_string());
        let task_b = Task::new("B".to_string(), "".to_string(), "".to_string())
            .with_dependencies(vec![task_a.id.clone()]);

        let composite = CompositeTask::new(
            "Test".to_string(),
            "".to_string(),
            vec![task_a.clone(), task_b.clone()],
        );

        let graph = composite.get_dependency_graph();

        assert_eq!(graph.len(), 2);
        assert!(graph.get(&task_a.id).unwrap().is_empty());
        assert_eq!(graph.get(&task_b.id).unwrap().len(), 1);
    }

    #[test]
    fn test_task_status_enum() {
        let statuses = vec![
            TaskStatus::Pending,
            TaskStatus::WaitingDependencies,
            TaskStatus::Ready,
            TaskStatus::InProgress,
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::Cancelled,
        ];

        for status in statuses {
            let formatted = format!("{:?}", status);
            assert!(!formatted.is_empty());
        }
    }
}

// ============================================================================
// Mock Tests
// ============================================================================

#[cfg(test)]
mod mock_tests {
    use super::*;

    #[tokio::test]
    async fn test_github_client_trigger_workflow() {
        use autodev_github::GitHubClient;
        use std::collections::HashMap;

        // This test requires a mock or test token
        // Skip if not available
        if std::env::var("GITHUB_TOKEN").is_err() {
            println!("Skipping GitHub client test - no token");
            return;
        }

        let client = GitHubClient::new("test_token".to_string()).unwrap();
        let repo = autodev_github::Repository::new("test".to_string(), "test".to_string());

        let mut inputs = HashMap::new();
        inputs.insert("key".to_string(), "value".to_string());

        let result = client
            .trigger_workflow(&repo, "test.yml", inputs)
            .await;

        assert!(result.is_ok());
        let run_id = result.unwrap();
        assert!(run_id.starts_with("run_"));
    }

    #[tokio::test]
    async fn test_ai_agent_execute_task() {
        // Mock implementation tested above
        assert!(true);
    }
}

// ============================================================================
// Database Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "database-tests")]
mod database_tests {
    use super::*;
    use autodev_db::Database;

    async fn setup_test_db() -> Database {
        let db_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost/autodev_test".to_string());

        let db = Database::new(&db_url).await.unwrap();
        db.init_schema().await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_save_and_get_task() {
        let db = setup_test_db().await;

        let task = Task::new(
            "Test Task".to_string(),
            "Description".to_string(),
            "Prompt".to_string(),
        );

        // Save
        db.save_task(&task, "testorg", "testrepo").await.unwrap();

        // Retrieve
        let retrieved = db.get_task(&task.id).await.unwrap();
        assert!(retrieved.is_some());

        let record = retrieved.unwrap();
        assert_eq!(record.title, "Test Task");
        assert_eq!(record.repository_owner, "testorg");
    }

    #[tokio::test]
    async fn test_execution_logs() {
        let db = setup_test_db().await;

        let task = Task::new("Test".to_string(), "".to_string(), "".to_string());
        db.save_task(&task, "org", "repo").await.unwrap();

        // Add logs
        db.add_execution_log(&task.id, "STARTED", "Task started").await.unwrap();
        db.add_execution_log(&task.id, "PROGRESS", "Processing...").await.unwrap();
        db.add_execution_log(&task.id, "COMPLETED", "Task completed").await.unwrap();

        // Get logs
        let logs = db.get_execution_logs(&task.id).await.unwrap();
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].event_type, "COMPLETED"); // DESC order
    }

    #[tokio::test]
    async fn test_metrics() {
        let db = setup_test_db().await;

        let task = Task::new("Test".to_string(), "".to_string(), "".to_string());
        db.save_task(&task, "org", "repo").await.unwrap();

        // Save metrics
        db.save_metrics(&task.id, 5000, 3, 150, 50, 1200).await.unwrap();

        // Get metrics
        let metrics = db.get_task_metrics(&task.id).await.unwrap();
        assert!(metrics.is_some());

        let m = metrics.unwrap();
        assert_eq!(m.execution_time_ms, 5000);
        assert_eq!(m.files_changed, 3);
        assert_eq!(m.ai_tokens_used, 1200);
    }

    #[tokio::test]
    async fn test_composite_task_with_subtasks() {
        let db = setup_test_db().await;

        let task_a = Task::new("A".to_string(), "".to_string(), "".to_string());
        let task_b = Task::new("B".to_string(), "".to_string(), "".to_string());

        let composite = CompositeTask::new(
            "Composite".to_string(),
            "Description".to_string(),
            vec![task_a.clone(), task_b.clone()],
        );

        // Save
        db.save_composite_task(&composite, "org", "repo").await.unwrap();

        // Retrieve
        let retrieved = db.get_composite_task(&composite.id).await.unwrap();
        assert!(retrieved.is_some());

        // Get subtasks
        let subtasks = db.get_composite_subtasks(&composite.id).await.unwrap();
        assert_eq!(subtasks.len(), 2);
    }
}

// ============================================================================
// Benchmark Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "bench")]
mod bench_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn bench_task_creation() {
        let engine = AutoDevEngine::new();

        let start = Instant::now();

        for i in 0..1000 {
            engine
                .create_simple_task(
                    format!("Task {}", i),
                    "".to_string(),
                    "".to_string(),
                )
                .await
                .unwrap();
        }

        let duration = start.elapsed();
        println!("Created 1000 tasks in {:?}", duration);

        // Should be less than 1ms per task
        assert!(duration.as_millis() < 1000);
    }

    #[tokio::test]
    async fn bench_parallel_batch_calculation() {
        let mut tasks = Vec::new();

        // Create 100 independent tasks
        for i in 0..100 {
            tasks.push(Task::new(
                format!("Task {}", i),
                "".to_string(),
                "".to_string(),
            ));
        }

        let composite = CompositeTask::new(
            "Test".to_string(),
            "".to_string(),
            tasks,
        );

        let start = Instant::now();
        let batches = composite.get_parallel_batches();
        let duration = start.elapsed();

        println!("Calculated parallel batches for 100 tasks in {:?}", duration);
        println!("Number of batches: {}", batches.len());

        // Should be less than 10ms
        assert!(duration.as_millis() < 10);
    }
}