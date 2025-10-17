use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("AI API error: {0}")]
    ApiError(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    #[error("Task decomposition failed: {0}")]
    DecompositionFailed(String),

    #[error("Prompt too long: {0} tokens")]
    PromptTooLong(usize),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;