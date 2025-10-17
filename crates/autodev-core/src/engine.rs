use crate::{CompositeTask, Result, Task, TaskStatus};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AutoDevEngine {
    pub active_tasks: Arc<RwLock<HashMap<String, Task>>>,
    pub completed_tasks: Arc<RwLock<HashSet<String>>>,
    pub composite_tasks: Arc<RwLock<HashMap<String, CompositeTask>>>,
}

impl AutoDevEngine {
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashSet::new())),
            composite_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a simple task
    pub async fn create_simple_task(
        &self,
        title: String,
        description: String,
        prompt: String,
    ) -> Result<Task> {
        let task = Task::new(title, description, prompt);

        let mut tasks = self.active_tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());

        tracing::info!("Created simple task: {} ({})", task.title, task.id);

        Ok(task)
    }

    /// Create a composite task
    pub async fn create_composite_task(
        &self,
        title: String,
        description: String,
        subtasks: Vec<Task>,
        auto_approve: bool,
    ) -> Result<CompositeTask> {
        let composite_task = CompositeTask::new(title, description, subtasks.clone())
            .with_auto_approve(auto_approve);

        // Add subtasks to active tasks
        let mut tasks = self.active_tasks.write().await;
        for task in &subtasks {
            tasks.insert(task.id.clone(), task.clone());
        }

        // Store composite task
        let mut composites = self.composite_tasks.write().await;
        composites.insert(composite_task.id.clone(), composite_task.clone());

        tracing::info!(
            "Created composite task: {} ({}) with {} subtasks",
            composite_task.title,
            composite_task.id,
            composite_task.subtasks.len()
        );

        // Log parallel execution plan
        let batches = composite_task.get_parallel_batches();
        for (i, batch) in batches.iter().enumerate() {
            let titles: Vec<&str> = batch.iter().map(|t| t.title.as_str()).collect();
            tracing::debug!("Batch {}: {:?}", i + 1, titles);
        }

        Ok(composite_task)
    }

    /// Update task status
    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        error: Option<String>,
    ) -> Result<()> {
        let mut tasks = self.active_tasks.write().await;

        if let Some(task) = tasks.get_mut(task_id) {
            task.status = status;
            if let Some(err) = error {
                task.error = Some(err);
            }

            if status == TaskStatus::Completed {
                let mut completed = self.completed_tasks.write().await;
                completed.insert(task_id.to_string());
                task.completed_at = Some(chrono::Utc::now());

                tracing::info!("Task completed: {} ({})", task.title, task_id);
            } else if status == TaskStatus::Failed {
                task.completed_at = Some(chrono::Utc::now());
                tracing::error!("Task failed: {} ({})", task.title, task_id);
            }
        }

        Ok(())
    }

    /// Get task by ID
    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        let tasks = self.active_tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// List all active tasks
    pub async fn list_active_tasks(&self) -> Vec<Task> {
        let tasks = self.active_tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// Get composite task by ID
    pub async fn get_composite_task(&self, composite_id: &str) -> Option<CompositeTask> {
        let composites = self.composite_tasks.read().await;
        composites.get(composite_id).cloned()
    }

    /// Get ready tasks (dependencies met)
    pub async fn get_ready_tasks(&self) -> Vec<Task> {
        let tasks = self.active_tasks.read().await;
        let completed = self.completed_tasks.read().await;

        tasks
            .values()
            .filter(|task| {
                task.status == TaskStatus::Pending || task.status == TaskStatus::WaitingDependencies
            })
            .filter(|task| task.can_start(&completed))
            .cloned()
            .collect()
    }

    /// Get task statistics
    pub async fn get_statistics(&self) -> EngineStatistics {
        let tasks = self.active_tasks.read().await;
        let completed = self.completed_tasks.read().await;
        let composites = self.composite_tasks.read().await;

        let total_tasks = tasks.len();
        let completed_tasks = completed.len();
        let failed_tasks = tasks
            .values()
            .filter(|t| t.status == TaskStatus::Failed)
            .count();
        let in_progress_tasks = tasks
            .values()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();

        EngineStatistics {
            total_tasks,
            completed_tasks,
            failed_tasks,
            in_progress_tasks,
            composite_tasks: composites.len(),
        }
    }
}

impl Default for AutoDevEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EngineStatistics {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub in_progress_tasks: usize,
    pub composite_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = AutoDevEngine::new();
        let stats = engine.get_statistics().await;
        assert_eq!(stats.total_tasks, 0);
    }

    #[tokio::test]
    async fn test_create_simple_task() {
        let engine = AutoDevEngine::new();

        let task = engine
            .create_simple_task(
                "Test Task".to_string(),
                "Description".to_string(),
                "Prompt".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(task.title, "Test Task");

        let retrieved = engine.get_task(&task.id).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let engine = AutoDevEngine::new();

        let task = engine
            .create_simple_task(
                "Test".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .await
            .unwrap();

        engine
            .update_task_status(&task.id, TaskStatus::InProgress, None)
            .await
            .unwrap();

        let updated = engine.get_task(&task.id).await.unwrap();
        assert_eq!(updated.status, TaskStatus::InProgress);
    }
}