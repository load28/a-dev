use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{handlers, state::ApiState};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))

        // Task endpoints
        .route("/tasks", post(handlers::task::create_task))
        .route("/tasks", get(handlers::task::list_tasks))
        .route("/tasks/:task_id", get(handlers::task::get_task_status))
        .route("/tasks/:task_id/execute", post(handlers::task::execute_task))
        .route("/tasks/decompose", post(handlers::task::decompose_task))
        .route("/tasks/:composite_task_id/orchestrate", post(handlers::task::orchestrate_task))

        // Composite task endpoints
        .route("/composite-tasks", post(handlers::composite::create_composite_task))
        .route("/composite-tasks/:task_id", get(handlers::composite::get_composite_task))
        .route("/composite-tasks/:task_id/execute", post(handlers::composite::execute_composite_task))

        // Statistics
        .route("/stats", get(handlers::stats::get_statistics))

        // GitHub webhook
        .route("/webhook/github", post(handlers::webhook::handle_github_webhook))

        // Callbacks
        .route("/callbacks/workflow-complete", post(handlers::callback::workflow_complete))

        // Add state
        .with_state(state)

        // Add CORS
        .layer(CorsLayer::permissive())
}