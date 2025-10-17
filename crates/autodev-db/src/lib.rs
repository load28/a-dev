pub mod models;
pub mod repository;
pub mod error;

// Re-exports
pub use models::{TaskRecord, CompositeTaskRecord, ExecutionLog, Metrics, AggregateStats};
pub use repository::Database;
pub use error::{Error, Result};