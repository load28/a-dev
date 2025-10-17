use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    WaitingDependencies,
    Ready,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Simple,
    Composite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub prompt: String,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub pr_url: Option<String>,
    pub workflow_run_id: Option<String>,
    pub error: Option<String>,
    pub auto_approve: bool,
}

impl Task {
    pub fn new(title: String, description: String, prompt: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            prompt,
            task_type: TaskType::Simple,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            pr_url: None,
            workflow_run_id: None,
            error: None,
            auto_approve: false,
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self.status = if deps.is_empty() {
            TaskStatus::Ready
        } else {
            TaskStatus::WaitingDependencies
        };
        self
    }

    pub fn can_start(&self, completed_tasks: &HashSet<String>) -> bool {
        self.dependencies.iter().all(|dep| completed_tasks.contains(dep))
    }

    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
        self.started_at = Some(Utc::now());
    }

    pub fn complete(&mut self, pr_url: Option<String>) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.pr_url = pr_url;
    }

    pub fn fail(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Test Task".to_string(),
            "Test Description".to_string(),
            "Test Prompt".to_string(),
        );

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.dependencies.is_empty());
    }

    #[test]
    fn test_task_dependencies() {
        let task = Task::new("".to_string(), "".to_string(), "".to_string())
            .with_dependencies(vec!["dep1".to_string()]);

        assert_eq!(task.status, TaskStatus::WaitingDependencies);
        assert_eq!(task.dependencies.len(), 1);
    }

    #[test]
    fn test_can_start() {
        let task = Task::new("".to_string(), "".to_string(), "".to_string())
            .with_dependencies(vec!["dep1".to_string()]);

        let mut completed = HashSet::new();
        assert!(!task.can_start(&completed));

        completed.insert("dep1".to_string());
        assert!(task.can_start(&completed));
    }
}