use async_trait::async_trait;
use autodev_core::Task;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    ClaudeCode,
    GPT4,
    Gemini,
    Codex,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::ClaudeCode => write!(f, "claude-code"),
            AgentType::GPT4 => write!(f, "gpt-4"),
            AgentType::Gemini => write!(f, "gemini"),
            AgentType::Codex => write!(f, "codex"),
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude-code" | "claude" => Ok(AgentType::ClaudeCode),
            "gpt-4" | "gpt4" => Ok(AgentType::GPT4),
            "gemini" => Ok(AgentType::Gemini),
            "codex" => Ok(AgentType::Codex),
            _ => Err(format!("Unknown agent type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub success: bool,
    pub files_changed: Vec<String>,
    pub pr_branch: String,
    pub commit_message: String,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub success: bool,
    pub changes_made: Vec<String>,
    pub comments: Vec<String>,
}

#[async_trait]
pub trait AIAgent: Send + Sync {
    /// Get agent type
    fn agent_type(&self) -> AgentType;

    /// Execute a task
    async fn execute_task(
        &self,
        task: &Task,
        repo_path: &str,
    ) -> crate::Result<AgentResult>;

    /// Review code changes
    async fn review_code_changes(
        &self,
        pr_diff: &str,
        review_comments: &[String],
    ) -> crate::Result<ReviewResult>;

    /// Fix CI failures
    async fn fix_ci_failures(
        &self,
        ci_logs: &str,
    ) -> crate::Result<ReviewResult>;

    /// Generate commit message
    async fn generate_commit_message(
        &self,
        changes: &str,
    ) -> crate::Result<String>;

    /// Analyze code for security issues
    async fn analyze_security(
        &self,
        code: &str,
        language: &str,
    ) -> crate::Result<Vec<SecurityIssue>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub severity: SecuritySeverity,
    pub title: String,
    pub description: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Base implementation for common agent functionality
pub struct BaseAgent {
    pub agent_type: AgentType,
    pub api_key: String,
    pub model: String,
}

impl BaseAgent {
    pub fn new(agent_type: AgentType, api_key: String, model: String) -> Self {
        Self {
            agent_type,
            api_key,
            model,
        }
    }

    /// Build prompt for task execution
    pub fn build_task_prompt(&self, task: &Task, repo_path: &str) -> String {
        format!(
            r#"
Task: {}
Description: {}

Repository: {}

Instructions:
{}

Please complete this task following best practices:
1. Write clean, maintainable code
2. Include appropriate error handling
3. Add tests if applicable
4. Follow the existing code style
5. Create meaningful commit messages

Return the list of files changed and a summary of changes.
"#,
            task.title, task.description, repo_path, task.prompt
        )
    }

    /// Build prompt for code review
    pub fn build_review_prompt(&self, pr_diff: &str, comments: &[String]) -> String {
        format!(
            r#"
Review the following code changes and address the review comments:

Code Changes:
```diff
{}
```

Review Comments:
{}

Please provide:
1. Specific fixes for each review comment
2. Any additional improvements you identify
3. Updated code that addresses all concerns
"#,
            pr_diff,
            comments.join("\n")
        )
    }

    /// Build prompt for CI fix
    pub fn build_ci_fix_prompt(&self, ci_logs: &str) -> String {
        format!(
            r#"
The CI pipeline has failed with the following errors:

```
{}
```

Please analyze the errors and provide:
1. Root cause of each failure
2. Specific fixes needed
3. Code changes to resolve the issues
4. Any additional improvements to prevent future failures
"#,
            ci_logs
        )
    }
}