use async_trait::async_trait;
use autodev_core::Task;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Claude,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Claude => write!(f, "claude"),
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" | "claude-code" | "claude-3" | "claude-opus" | "claude-sonnet" => {
                Ok(AgentType::Claude)
            }
            _ => Err(format!("Unsupported agent type: {}. Only Claude is supported.", s)),
        }
    }
}

impl Default for AgentType {
    fn default() -> Self {
        AgentType::Claude
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

    /// Chat with JSON mode (structured output)
    /// System prompt and user prompt are combined to request structured JSON response
    async fn chat_json(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> crate::Result<String>;
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
        let system_prompt = include_str!("../prompts/task_execution_system.txt");

        format!(
            "{}\n\n## 작업 정보\n\n작업명: {}\n설명: {}\n저장소 경로: {}\n\n상세 지침:\n{}",
            system_prompt, task.title, task.description, repo_path, task.prompt
        )
    }

    /// Build prompt for code review
    pub fn build_review_prompt(&self, pr_diff: &str, comments: &[String]) -> String {
        let system_prompt = include_str!("../prompts/code_review_system.txt");

        format!(
            "{}\n\n## 코드 변경사항\n\n```diff\n{}\n```\n\n## 리뷰 코멘트\n\n{}",
            system_prompt,
            pr_diff,
            comments.join("\n")
        )
    }

    /// Build prompt for CI fix
    pub fn build_ci_fix_prompt(&self, ci_logs: &str) -> String {
        let system_prompt = include_str!("../prompts/ci_fix_system.txt");

        format!(
            "{}\n\n## CI 실패 로그\n\n```\n{}\n```",
            system_prompt,
            ci_logs
        )
    }
}