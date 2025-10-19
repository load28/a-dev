use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod cli;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "autodev=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Run CLI
    run(cli).await
}

async fn run(cli: Cli) -> Result<()> {
    // Initialize engine
    let engine = Arc::new(autodev_core::AutoDevEngine::new());

    // Initialize database (optional)
    let db = if let Some(ref db_url) = cli.database_url {
        let database = autodev_db::Database::new(db_url).await?;
        Some(Arc::new(database))
    } else if let Ok(db_url) = std::env::var("DATABASE_URL") {
        let database = autodev_db::Database::new(&db_url).await?;
        Some(Arc::new(database))
    } else {
        None
    };

    // Initialize GitHub client
    let github_client = Arc::new(
        autodev_github::GitHubClient::new(cli.github_token.clone())?
    );

    // Initialize AI agent
    // Note: For CLI commands that directly use AI (decompose, etc), API key is required
    // For 'serve' command with local Docker executor, AI agent is not used (Docker worker handles it)
    let needs_ai_agent = !matches!(cli.command, Commands::Serve { .. });

    let ai_agent: Arc<dyn autodev_ai::AIAgent> = if needs_ai_agent {
        // CLI commands need API key
        match cli.agent_type.as_str() {
            "claude" | "claude-code" => {
                let api_key = std::env::var("ANTHROPIC_API_KEY")
                    .expect("ANTHROPIC_API_KEY must be set for Claude agent in CLI mode");
                Arc::new(autodev_ai::ClaudeAgent::new(api_key))
            }
            "gpt-4" | "openai" => {
                tracing::warn!("OpenAI agent not implemented, using Claude instead");
                let api_key = std::env::var("ANTHROPIC_API_KEY")
                    .expect("ANTHROPIC_API_KEY must be set");
                Arc::new(autodev_ai::ClaudeAgent::new(api_key))
            }
            _ => {
                tracing::warn!("Unknown AI agent type: {}, using Claude", cli.agent_type);
                let api_key = std::env::var("ANTHROPIC_API_KEY")
                    .expect("ANTHROPIC_API_KEY must be set");
                Arc::new(autodev_ai::ClaudeAgent::new(api_key))
            }
        }
    } else {
        // For 'serve' command, try OAuth token first, fallback to API key
        if let Ok(oauth_token) = std::env::var("CLAUDE_CODE_OAUTH_TOKEN") {
            tracing::info!("Using Docker-based AI executor with Claude subscription OAuth token");
            Arc::new(
                autodev_ai::DockerAIExecutor::new(oauth_token)
                    .expect("Failed to initialize Docker AI executor")
            )
        } else if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            tracing::info!("Using HTTP API-based AI agent with API key");
            Arc::new(autodev_ai::ClaudeAgent::new(api_key))
        } else {
            panic!("Either CLAUDE_CODE_OAUTH_TOKEN or ANTHROPIC_API_KEY must be set for serve command");
        }
    };

    // Execute command
    commands::execute(cli.command, engine, db, github_client, ai_agent).await
}