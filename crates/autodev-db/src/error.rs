use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Composite task not found: {0}")]
    CompositeTaskNotFound(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;