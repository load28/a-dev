mod error;
mod docker_executor;

pub use error::{LocalExecutorError, Result};
pub use docker_executor::{DockerExecutor, TaskResult};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub pr_number: Option<u64>,
    pub pr_url: Option<String>,
    pub error: Option<String>,
    pub output: String,
}

// Convert TaskResult to ExecutionResult for backward compatibility
impl From<TaskResult> for ExecutionResult {
    fn from(result: TaskResult) -> Self {
        ExecutionResult {
            success: result.success,
            pr_number: result.pr_number,
            pr_url: result.pr_url.clone(),
            error: result.error.clone(),
            output: if result.success {
                format!(
                    "Task completed successfully. PR: {}",
                    result.pr_url.unwrap_or_else(|| "N/A".to_string())
                )
            } else {
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            },
        }
    }
}
