use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod cli;

use cli::Cli;

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
    let ai_agent: Arc<dyn autodev_ai::AIAgent> = match cli.agent_type.as_str() {
        "claude" | "claude-code" => {
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY must be set for Claude");
            Arc::new(autodev_ai::ClaudeAgent::new(api_key))
        }
        "gpt-4" | "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .expect("OPENAI_API_KEY must be set for OpenAI");
            Arc::new(autodev_ai::OpenAIAgent::new(api_key))
        }
        _ => {
            tracing::warn!("Unknown AI agent type: {}, using Claude", cli.agent_type);
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY must be set");
            Arc::new(autodev_ai::ClaudeAgent::new(api_key))
        }
    };

    // Execute command
    commands::execute(cli.command, engine, db, github_client, ai_agent).await
}