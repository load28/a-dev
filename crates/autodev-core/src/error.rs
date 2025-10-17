use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Invalid task state: {0}")]
    InvalidTaskState(String),

    #[error("Dependency cycle detected")]
    DependencyCycle,

    #[error("Engine error: {0}")]
    EngineError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;