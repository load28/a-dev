pub mod agent;
pub mod claude;
pub mod openai;
pub mod decomposer;
pub mod error;

// Re-exports
pub use agent::{AIAgent, AgentResult, AgentType, ReviewResult};
pub use claude::ClaudeAgent;
pub use openai::OpenAIAgent;
pub use decomposer::TaskDecomposer;
pub use error::{Error, Result};