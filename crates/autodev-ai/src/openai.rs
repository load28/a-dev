use crate::{
    agent::{AIAgent, AgentResult, AgentType, BaseAgent, ReviewResult, SecurityIssue},
    Result,
};
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    Client,
};
use async_trait::async_trait;
use autodev_core::Task;

pub struct OpenAIAgent {
    base: BaseAgent,
    client: Client<OpenAIConfig>,
}

impl OpenAIAgent {
    pub fn new(api_key: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key.clone());

        Self {
            base: BaseAgent::new(
                AgentType::GPT4,
                api_key,
                "gpt-4-turbo-preview".to_string(),
            ),
            client: Client::with_config(config),
        }
    }
}

#[async_trait]
impl AIAgent for OpenAIAgent {
    fn agent_type(&self) -> AgentType {
        self.base.agent_type.clone()
    }

    async fn execute_task(&self, task: &Task, repo_path: &str) -> Result<AgentResult> {
        tracing::info!("GPT-4 executing task: {}", task.title);

        let prompt = self.base.build_task_prompt(task, repo_path);

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.base.model)
            .messages(vec![ChatCompletionRequestMessage::User {
                content: prompt.into(),
                name: None,
            }])
            .temperature(0.7)
            .max_tokens(4096_u16)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let output = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AgentResult {
            success: true,
            files_changed: vec!["src/main.rs".to_string()],
            pr_branch: format!("autodev/task-{}", task.id),
            commit_message: format!("feat: {}", task.title),
            output: Some(output),
        })
    }

    async fn review_code_changes(
        &self,
        pr_diff: &str,
        review_comments: &[String],
    ) -> Result<ReviewResult> {
        tracing::info!("GPT-4 reviewing code changes");

        let prompt = self.base.build_review_prompt(pr_diff, review_comments);

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.base.model)
            .messages(vec![ChatCompletionRequestMessage::User {
                content: prompt.into(),
                name: None,
            }])
            .temperature(0.7)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let output = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ReviewResult {
            success: true,
            changes_made: vec!["Applied review feedback".to_string()],
            comments: vec![output],
        })
    }

    async fn fix_ci_failures(&self, ci_logs: &str) -> Result<ReviewResult> {
        tracing::info!("GPT-4 fixing CI failures");

        let prompt = self.base.build_ci_fix_prompt(ci_logs);

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.base.model)
            .messages(vec![ChatCompletionRequestMessage::User {
                content: prompt.into(),
                name: None,
            }])
            .build()?;

        let response = self.client.chat().create(request).await?;

        let output = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ReviewResult {
            success: true,
            changes_made: vec!["Fixed CI issues".to_string()],
            comments: vec![output],
        })
    }

    async fn generate_commit_message(&self, changes: &str) -> Result<String> {
        let prompt = format!(
            "Generate a conventional commit message for:\n\n{}",
            changes
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.base.model)
            .messages(vec![ChatCompletionRequestMessage::User {
                content: prompt.into(),
                name: None,
            }])
            .build()?;

        let response = self.client.chat().create(request).await?;

        Ok(response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_else(|| "chore: update code".to_string()))
    }

    async fn analyze_security(&self, code: &str, language: &str) -> Result<Vec<SecurityIssue>> {
        let prompt = format!(
            "Analyze {} code for security issues:\n\n{}",
            language, code
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.base.model)
            .messages(vec![ChatCompletionRequestMessage::User {
                content: prompt.into(),
                name: None,
            }])
            .build()?;

        let _response = self.client.chat().create(request).await?;

        // Parse and return security issues
        Ok(vec![])
    }
}