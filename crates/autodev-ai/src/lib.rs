pub mod agent;
pub mod claude;
pub mod decomposer;
pub mod error;
pub mod schema;
pub mod examples;

// Re-exports
pub use agent::{AIAgent, AgentResult, AgentType, ReviewResult};
pub use claude::ClaudeAgent;
pub use decomposer::TaskDecomposer;
pub use error::{Error, Result};
pub use schema::{TaskDecompositionResponse, TaskSchema, TaskDomain, ComplexityEstimate};
pub use examples::{ExampleDatabase, FewShotExample};