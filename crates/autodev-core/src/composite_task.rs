use crate::task::Task;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub subtasks: Vec<Task>,
    pub auto_approve: bool,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl CompositeTask {
    pub fn new(title: String, description: String, subtasks: Vec<Task>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            subtasks,
            auto_approve: false,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn with_auto_approve(mut self, auto_approve: bool) -> Self {
        self.auto_approve = auto_approve;
        self
    }

    /// Generate dependency graph
    pub fn get_dependency_graph(&self) -> HashMap<String, Vec<String>> {
        self.subtasks
            .iter()
            .map(|task| (task.id.clone(), task.dependencies.clone()))
            .collect()
    }

    /// Group tasks into parallel execution batches
    pub fn get_parallel_batches(&self) -> Vec<Vec<Task>> {
        let mut completed = HashSet::new();
        let mut batches = Vec::new();
        let mut remaining: Vec<Task> = self.subtasks.clone();

        while !remaining.is_empty() {
            // Find tasks that can be executed now
            let ready_tasks: Vec<Task> = remaining
                .iter()
                .filter(|task| task.can_start(&completed))
                .cloned()
                .collect();

            if ready_tasks.is_empty() {
                // Circular dependency or unmet dependencies
                tracing::warn!("Unable to schedule remaining tasks due to dependencies");
                break;
            }

            batches.push(ready_tasks.clone());

            // Mark as completed
            for task in &ready_tasks {
                completed.insert(task.id.clone());
                remaining.retain(|t| t.id != task.id);
            }
        }

        batches
    }

    /// Calculate total estimated time (assuming parallel execution)
    pub fn estimate_total_time(&self, avg_task_time_secs: u64) -> u64 {
        let batches = self.get_parallel_batches();
        batches.len() as u64 * avg_task_time_secs
    }

    /// Check if all subtasks are completed
    pub fn is_completed(&self) -> bool {
        self.subtasks
            .iter()
            .all(|task| matches!(task.status, crate::task::TaskStatus::Completed))
    }

    /// Get progress percentage
    pub fn get_progress(&self) -> f32 {
        if self.subtasks.is_empty() {
            return 100.0;
        }

        let completed = self
            .subtasks
            .iter()
            .filter(|task| matches!(task.status, crate::task::TaskStatus::Completed))
            .count();

        (completed as f32 / self.subtasks.len() as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskStatus;

    #[test]
    fn test_composite_task_creation() {
        let tasks = vec![
            Task::new("Task 1".to_string(), "".to_string(), "".to_string()),
            Task::new("Task 2".to_string(), "".to_string(), "".to_string()),
        ];

        let composite = CompositeTask::new(
            "Composite".to_string(),
            "Description".to_string(),
            tasks.clone(),
        );

        assert_eq!(composite.title, "Composite");
        assert_eq!(composite.subtasks.len(), 2);
        assert!(!composite.auto_approve);
    }

    #[test]
    fn test_parallel_batches() {
        let task_a = Task::new("A".to_string(), "".to_string(), "".to_string());
        let mut task_b = Task::new("B".to_string(), "".to_string(), "".to_string());
        task_b.dependencies = vec![task_a.id.clone()];
        let mut task_c = Task::new("C".to_string(), "".to_string(), "".to_string());
        task_c.dependencies = vec![task_a.id.clone()];

        let composite = CompositeTask::new(
            "Test".to_string(),
            "".to_string(),
            vec![task_a, task_b, task_c],
        );

        let batches = composite.get_parallel_batches();
        assert_eq!(batches.len(), 2); // First A, then B and C in parallel
        assert_eq!(batches[0].len(), 1); // Just A
        assert_eq!(batches[1].len(), 2); // B and C
    }

    #[test]
    fn test_progress_calculation() {
        let mut tasks = vec![
            Task::new("Task 1".to_string(), "".to_string(), "".to_string()),
            Task::new("Task 2".to_string(), "".to_string(), "".to_string()),
            Task::new("Task 3".to_string(), "".to_string(), "".to_string()),
        ];

        tasks[0].status = TaskStatus::Completed;

        let composite = CompositeTask::new("Test".to_string(), "".to_string(), tasks);

        let progress = composite.get_progress();
        assert_eq!(progress, 33.333336); // 1/3 completed
    }
}