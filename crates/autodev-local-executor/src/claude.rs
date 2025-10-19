use crate::docker::DockerManager;
use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, debug, error};

pub struct ClaudeExecutor {
    docker: DockerManager,
    image_name: String,
}

impl ClaudeExecutor {
    pub fn new(docker: DockerManager) -> Self {
        Self {
            docker,
            image_name: "autodev-claude-executor:latest".to_string(),
        }
    }

    pub fn with_image(mut self, image_name: String) -> Self {
        self.image_name = image_name;
        self
    }

    /// Execute a task using Claude Code CLI in Docker
    pub async fn execute_task(
        &self,
        workspace_path: &Path,
        prompt: &str,
        anthropic_api_key: &str,
        github_token: &str,
    ) -> Result<ClaudeExecutionResult> {
        info!("Executing Claude Code task in Docker");
        debug!("Workspace: {:?}", workspace_path);
        debug!("Prompt: {}", prompt);

        // Prepare environment variables
        let mut env_vars = HashMap::new();
        env_vars.insert("ANTHROPIC_API_KEY".to_string(), anthropic_api_key.to_string());
        env_vars.insert("GITHUB_TOKEN".to_string(), github_token.to_string());
        env_vars.insert("GH_TOKEN".to_string(), github_token.to_string()); // GitHub CLI uses GH_TOKEN

        // Prepare command
        // Using Claude Code CLI with non-interactive mode
        let command = vec![
            "claude".to_string(),
            "--dangerously-skip-permissions".to_string(),
            "--allowedTools".to_string(),
            "Bash,Read,Write,Edit,Glob,Grep".to_string(),
            "--model".to_string(),
            "sonnet".to_string(),
            "--append-system-prompt".to_string(),
            "Make autonomous decisions and modify files directly without asking questions.".to_string(),
            prompt.to_string(),
        ];

        // Execute in Docker
        let (stdout, stderr, exit_code) = self
            .docker
            .run_command(&self.image_name, command, workspace_path, env_vars)
            .await?;

        if exit_code != 0 {
            error!("Claude Code execution failed with exit code: {}", exit_code);
            error!("STDERR: {}", stderr);

            return Ok(ClaudeExecutionResult {
                success: false,
                output: stdout,
                error: Some(format!(
                    "Claude Code failed with exit code {}: {}",
                    exit_code, stderr
                )),
            });
        }

        info!("Claude Code execution completed successfully");

        Ok(ClaudeExecutionResult {
            success: true,
            output: stdout,
            error: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ClaudeExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_executor_creation() {
        let docker = DockerManager::new().unwrap();
        let executor = ClaudeExecutor::new(docker);
        assert_eq!(executor.image_name, "autodev-claude-executor:latest");
    }

    #[test]
    fn test_claude_executor_with_custom_image() {
        let docker = DockerManager::new().unwrap();
        let executor = ClaudeExecutor::new(docker)
            .with_image("custom-image:v1".to_string());
        assert_eq!(executor.image_name, "custom-image:v1");
    }
}
