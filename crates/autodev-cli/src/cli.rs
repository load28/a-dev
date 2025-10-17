use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "autodev")]
#[command(about = "AutoDev - Automated AI Development Platform", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// GitHub token
    #[arg(long, env = "GITHUB_TOKEN")]
    pub github_token: String,

    /// AI agent type (claude-code, codex, etc.)
    #[arg(long, default_value = "claude-code")]
    pub agent_type: String,

    /// Database URL
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a simple task
    Task {
        /// Repository owner
        #[arg(long)]
        owner: String,

        /// Repository name
        #[arg(long)]
        repo: String,

        /// Task title
        #[arg(long)]
        title: String,

        /// Task description
        #[arg(long)]
        description: String,

        /// AI agent prompt
        #[arg(long)]
        prompt: String,

        /// Execute immediately
        #[arg(long)]
        execute: bool,
    },

    /// Create a composite task
    Composite {
        /// Repository owner
        #[arg(long)]
        owner: String,

        /// Repository name
        #[arg(long)]
        repo: String,

        /// Task title
        #[arg(long)]
        title: String,

        /// Task description
        #[arg(long)]
        description: String,

        /// Composite prompt
        #[arg(long)]
        prompt: String,

        /// Auto-approve subtasks
        #[arg(long)]
        auto_approve: bool,

        /// Execute immediately
        #[arg(long)]
        execute: bool,
    },

    /// Execute a task by ID
    Execute {
        /// Task ID
        task_id: String,

        /// Repository owner
        #[arg(long)]
        owner: String,

        /// Repository name
        #[arg(long)]
        repo: String,
    },

    /// Show task status
    Status {
        /// Task ID
        task_id: String,
    },

    /// List all active tasks
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,

        /// Limit number of results
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Start API server
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "3000")]
        port: u16,
    },

    /// Show statistics
    Stats,

    /// Initialize database
    InitDb,
}