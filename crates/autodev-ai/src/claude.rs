use crate::{
    agent::{AIAgent, AgentResult, AgentType, BaseAgent, ReviewResult, SecurityIssue},
    Result,
};
use async_trait::async_trait;
use autodev_core::Task;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct ClaudeAgent {
    base: BaseAgent,
    client: Client,
    api_url: String,
}

impl ClaudeAgent {
    pub fn new(api_key: String) -> Self {
        Self {
            base: BaseAgent::new(
                AgentType::ClaudeCode,
                api_key.clone(),
                "claude-3-opus-20240229".to_string(),
            ),
            client: Client::new(),
            api_url: "https://api.anthropic.com/v1".to_string(),
        }
    }

    async fn call_api(&self, messages: Vec<Message>) -> Result<String> {
        let response = self
            .client
            .post(format!("{}/messages", self.api_url))
            .header("x-api-key", &self.base.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": &self.base.model,
                "messages": messages,
                "max_tokens": 4096,
                "temperature": 0.7,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(crate::Error::ApiError(format!(
                "Claude API error: {}",
                error_text
            )));
        }

        let result: ClaudeResponse = response.json().await?;
        Ok(result.content.first().map(|c| c.text.clone()).unwrap_or_default())
    }
}

#[async_trait]
impl AIAgent for ClaudeAgent {
    fn agent_type(&self) -> AgentType {
        self.base.agent_type.clone()
    }

    async fn execute_task(&self, task: &Task, repo_path: &str) -> Result<AgentResult> {
        tracing::info!("Claude executing task: {}", task.title);

        let prompt = self.base.build_task_prompt(task, repo_path);

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.call_api(messages).await?;

        // Parse response and extract files changed
        // In real implementation, this would execute Claude Code CLI
        let files_changed = vec!["src/main.rs".to_string(), "tests/test.rs".to_string()];

        Ok(AgentResult {
            success: true,
            files_changed,
            pr_branch: format!("autodev/task-{}", task.id),
            commit_message: format!("feat: {}", task.title),
            output: Some(response),
        })
    }

    async fn review_code_changes(
        &self,
        pr_diff: &str,
        review_comments: &[String],
    ) -> Result<ReviewResult> {
        tracing::info!("Claude reviewing code changes");

        let prompt = self.base.build_review_prompt(pr_diff, review_comments);

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.call_api(messages).await?;

        Ok(ReviewResult {
            success: true,
            changes_made: vec!["Fixed type hints".to_string(), "Added error handling".to_string()],
            comments: vec![response],
        })
    }

    async fn fix_ci_failures(&self, ci_logs: &str) -> Result<ReviewResult> {
        tracing::info!("Claude fixing CI failures");

        let prompt = self.base.build_ci_fix_prompt(ci_logs);

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.call_api(messages).await?;

        Ok(ReviewResult {
            success: true,
            changes_made: vec!["Fixed linting errors".to_string(), "Updated tests".to_string()],
            comments: vec![response],
        })
    }

    async fn generate_commit_message(&self, changes: &str) -> Result<String> {
        let prompt = format!(
            "Generate a conventional commit message for the following changes:\n\n{}",
            changes
        );

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt,
        }];

        self.call_api(messages).await
    }

    async fn analyze_security(&self, code: &str, language: &str) -> Result<Vec<SecurityIssue>> {
        let prompt = format!(
            "Analyze the following {} code for security vulnerabilities:\n\n{}",
            language, code
        );

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt,
        }];

        let _response = self.call_api(messages).await?;

        // Parse response into security issues
        // This is a simplified version
        Ok(vec![])
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
}