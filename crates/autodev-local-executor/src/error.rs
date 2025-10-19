use thiserror::Error;

#[derive(Error, Debug)]
pub enum LocalExecutorError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Docker error: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    #[error("GitHub error: {0}")]
    GitHub(#[from] autodev_github::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, LocalExecutorError>;
