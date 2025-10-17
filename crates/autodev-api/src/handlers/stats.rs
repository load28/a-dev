use axum::{extract::State, Json};
use serde::Serialize;

use crate::state::ApiState;

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub engine_stats: EngineStats,
    pub db_stats: Option<DbStats>,
}

#[derive(Debug, Serialize)]
pub struct EngineStats {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub in_progress_tasks: usize,
    pub composite_tasks: usize,
}

#[derive(Debug, Serialize)]
pub struct DbStats {
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub avg_execution_time_ms: Option<f64>,
    pub total_files_changed: Option<i64>,
    pub total_tokens_used: Option<i64>,
}

pub async fn get_statistics(State(state): State<ApiState>) -> Json<StatsResponse> {
    // Get engine statistics
    let engine_stats_raw = state.engine.get_statistics().await;
    let engine_stats = EngineStats {
        total_tasks: engine_stats_raw.total_tasks,
        completed_tasks: engine_stats_raw.completed_tasks,
        failed_tasks: engine_stats_raw.failed_tasks,
        in_progress_tasks: engine_stats_raw.in_progress_tasks,
        composite_tasks: engine_stats_raw.composite_tasks,
    };

    // Get database statistics if available
    let db_stats = if let Some(ref db) = state.db {
        match db.get_aggregate_stats().await {
            Ok(stats) => Some(DbStats {
                total_tasks: stats.total_tasks,
                completed_tasks: stats.completed_tasks,
                failed_tasks: stats.failed_tasks,
                avg_execution_time_ms: stats.avg_execution_time_ms,
                total_files_changed: stats.total_files_changed,
                total_tokens_used: stats.total_tokens_used,
            }),
            Err(e) => {
                tracing::error!("Failed to get database stats: {}", e);
                None
            }
        }
    } else {
        None
    };

    Json(StatsResponse {
        engine_stats,
        db_stats,
    })
}