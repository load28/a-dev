use std::sync::Arc;

#[derive(Clone)]
pub struct ApiState {
    pub engine: Arc<autodev_core::AutoDevEngine>,
    pub db: Option<Arc<autodev_db::Database>>,
    pub github_client: Arc<autodev_github::GitHubClient>,
    pub ai_agent: Arc<dyn autodev_ai::AIAgent>,
}