use anyhow::Result;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use autodev_core::{AutoDevEngine, Task};

// TaskScheduler는 향후 사용 예정
#[allow(dead_code)]
pub struct TaskScheduler {
    engine: Arc<AutoDevEngine>,
}

#[allow(dead_code)]
impl TaskScheduler {
    pub fn new(engine: Arc<AutoDevEngine>) -> Self {
        Self { engine }
    }

    /// Schedule tasks for execution based on dependencies
    pub async fn schedule_tasks(&self) -> Result<Vec<Task>> {
        let all_tasks = self.engine.list_active_tasks().await;
        let completed = self.engine.completed_tasks.read().await.clone();

        // Find tasks that are ready to run
        let ready_tasks: Vec<Task> = all_tasks
            .into_iter()
            .filter(|task| {
                task.status == autodev_core::TaskStatus::Pending
                    || task.status == autodev_core::TaskStatus::WaitingDependencies
            })
            .filter(|task| task.can_start(&completed))
            .collect();

        Ok(ready_tasks)
    }

    /// Schedule composite task batches
    pub async fn schedule_composite_task(&self, composite_id: &str) -> Result<Vec<Vec<Task>>> {
        let composite_task = self.engine
            .get_composite_task(composite_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Composite task not found"))?;

        Ok(composite_task.get_parallel_batches())
    }

    /// Get next batch of tasks to execute for a composite task
    pub async fn get_next_batch(&self, composite_id: &str) -> Result<Option<Vec<Task>>> {
        let composite_task = self.engine
            .get_composite_task(composite_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Composite task not found"))?;

        let completed = self.engine.completed_tasks.read().await.clone();
        let batches = composite_task.get_parallel_batches();

        // Find the first batch where not all tasks are completed
        for batch in batches {
            let all_completed = batch.iter().all(|task| completed.contains(&task.id));

            if !all_completed {
                // Check if all tasks in this batch can start
                let can_start = batch.iter().all(|task| task.can_start(&completed));

                if can_start {
                    return Ok(Some(batch));
                } else {
                    // Dependencies not met yet
                    return Ok(None);
                }
            }
        }

        // All batches completed
        Ok(None)
    }

    /// Detect circular dependencies
    pub fn detect_circular_dependencies(&self, tasks: &[Task]) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for task in tasks {
            if !visited.contains(&task.id) {
                if self.has_cycle(
                    &task.id,
                    tasks,
                    &mut visited,
                    &mut rec_stack,
                )? {
                    anyhow::bail!("Circular dependency detected involving task: {}", task.id);
                }
            }
        }

        Ok(())
    }

    fn has_cycle(
        &self,
        task_id: &str,
        tasks: &[Task],
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<bool> {
        visited.insert(task_id.to_string());
        rec_stack.insert(task_id.to_string());

        // Find the task
        let task = tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

        // Check all dependencies
        for dep_id in &task.dependencies {
            if !visited.contains(dep_id) {
                if self.has_cycle(dep_id, tasks, visited, rec_stack)? {
                    return Ok(true);
                }
            } else if rec_stack.contains(dep_id) {
                // Found a cycle
                return Ok(true);
            }
        }

        rec_stack.remove(task_id);
        Ok(false)
    }

    /// Calculate critical path (longest dependency chain)
    pub fn calculate_critical_path(&self, tasks: &[Task]) -> Vec<String> {
        let mut path_lengths: HashMap<String, usize> = HashMap::new();
        let mut paths: HashMap<String, Vec<String>> = HashMap::new();

        // Topological sort
        let sorted_tasks = self.topological_sort(tasks);

        for task in &sorted_tasks {
            if task.dependencies.is_empty() {
                path_lengths.insert(task.id.clone(), 1);
                paths.insert(task.id.clone(), vec![task.id.clone()]);
            } else {
                let max_dep_length = task.dependencies
                    .iter()
                    .map(|dep| path_lengths.get(dep).unwrap_or(&0))
                    .max()
                    .unwrap_or(&0);

                path_lengths.insert(task.id.clone(), max_dep_length + 1);

                // Find the dependency with the longest path
                if let Some(longest_dep) = task.dependencies
                    .iter()
                    .max_by_key(|dep| path_lengths.get(*dep).unwrap_or(&0))
                {
                    let mut path = paths.get(longest_dep).cloned().unwrap_or_default();
                    path.push(task.id.clone());
                    paths.insert(task.id.clone(), path);
                }
            }
        }

        // Find the longest path overall
        paths.values()
            .max_by_key(|path| path.len())
            .cloned()
            .unwrap_or_default()
    }

    fn topological_sort(&self, tasks: &[Task]) -> Vec<Task> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for task in tasks {
            if !visited.contains(&task.id) {
                self.dfs_visit(
                    task,
                    tasks,
                    &mut visited,
                    &mut temp_visited,
                    &mut sorted,
                );
            }
        }

        sorted.reverse();
        sorted
    }

    fn dfs_visit(
        &self,
        task: &Task,
        tasks: &[Task],
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        sorted: &mut Vec<Task>,
    ) {
        if temp_visited.contains(&task.id) {
            // Cycle detected, but we'll ignore for sorting
            return;
        }

        if visited.contains(&task.id) {
            return;
        }

        temp_visited.insert(task.id.clone());

        // Visit dependencies first
        for dep_id in &task.dependencies {
            if let Some(dep_task) = tasks.iter().find(|t| t.id == *dep_id) {
                self.dfs_visit(dep_task, tasks, visited, temp_visited, sorted);
            }
        }

        temp_visited.remove(&task.id);
        visited.insert(task.id.clone());
        sorted.push(task.clone());
    }
}