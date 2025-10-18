use anyhow::Result;
use std::env;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod routes;
mod state;

use autodev_core::AutoDevEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "autodev_api=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Get configuration
    let port = env::var("API_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;

    let github_token = env::var("GITHUB_TOKEN")
        .expect("GITHUB_TOKEN must be set");

    let ai_agent_type = env::var("AI_AGENT_TYPE")
        .unwrap_or_else(|_| "claude-code".to_string());

    // Initialize engine
    let engine = Arc::new(AutoDevEngine::new());

    // Initialize database (optional)
    let db = if let Ok(db_url) = env::var("DATABASE_URL") {
        let database = autodev_db::Database::new(&db_url).await?;
        database.init_schema().await?;
        Some(Arc::new(database))
    } else {
        tracing::warn!("No DATABASE_URL provided, running without persistence");
        None
    };

    // Initialize GitHub client
    let github_client = Arc::new(
        autodev_github::GitHubClient::new(github_token.clone())?
    );

    // Initialize AI agent
    let ai_agent: Arc<dyn autodev_ai::AIAgent> = match ai_agent_type.as_str() {
        "claude" | "claude-code" => {
            let api_key = env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY must be set for Claude");
            Arc::new(autodev_ai::ClaudeAgent::new(api_key))
        }
        "gpt-4" | "openai" => {
            tracing::warn!("OpenAI agent not implemented, using Claude instead");
            let api_key = env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY must be set");
            Arc::new(autodev_ai::ClaudeAgent::new(api_key))
        }
        _ => {
            tracing::warn!("Unknown AI agent type: {}, using Claude", ai_agent_type);
            let api_key = env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY must be set");
            Arc::new(autodev_ai::ClaudeAgent::new(api_key))
        }
    };

    // Initialize Docker executor for local task execution
    let anthropic_api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set");
    let github_token_clone = github_token.clone();

    let server_url = format!("http://localhost:{}", port);
    let docker_executor = Arc::new(
        autodev_worker::DockerExecutor::new(
            anthropic_api_key,
            github_token_clone,
            Some(server_url),
        ).await?
    );

    tracing::info!("✓ Docker executor initialized");

    // Create app state
    let state = state::ApiState {
        engine,
        db,
        github_client,
        ai_agent,
        docker_executor,
    };

    // Build router
    let app = routes::create_router(state);

    // Start server
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("🚀 AutoDev API Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}