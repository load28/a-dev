use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Repository not found: {0}")]
    RepoNotFound(String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Pull request not found: {0}")]
    PullRequestNotFound(String),

    #[error("Unsupported webhook event: {0}")]
    UnsupportedEvent(String),

    #[error("Octocrab error: {0}")]
    Octocrab(#[from] octocrab::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;