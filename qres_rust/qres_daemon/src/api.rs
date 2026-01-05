use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use crate::daemon::DaemonManager;

#[derive(Clone, Default)]
pub struct ApiState {
    // Shared state for the API
}

// Response types
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    running: bool,
    pid: Option<u32>,
    metrics: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
pub struct StartRequest {
    wan: Option<bool>,
    gossip_interval: Option<u64>,
}

// Handlers
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_status() -> Json<StatusResponse> {
    let pid_file = DaemonManager::get_pid_file();
    let state_file = DaemonManager::get_state_file();

    let (running, pid) = if let Ok(content) = std::fs::read_to_string(&pid_file) {
        if let Ok(pid_val) = content.trim().parse::<u32>() {
            let s = sysinfo::System::new_all();
            let pid_obj = sysinfo::Pid::from(pid_val as usize);
            (s.process(pid_obj).is_some(), Some(pid_val))
        } else {
            (false, None)
        }
    } else {
        (false, None)
    };

    let metrics = if running {
        std::fs::read_to_string(&state_file)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    } else {
        None
    };

    Json(StatusResponse {
        running,
        pid,
        metrics,
    })
}

async fn start_swarm(
    Json(payload): Json<StartRequest>,
) -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let wan = payload.wan.unwrap_or(false);
    let interval = payload.gossip_interval.unwrap_or(600);

    match DaemonManager::start(wan, interval) {
        Ok(_) => Ok(Json(StatusResponse {
            running: true,
            pid: None, // Will be set after process starts
            metrics: None,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

async fn stop_swarm() -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    match DaemonManager::stop() {
        Ok(_) => Ok(Json(StatusResponse {
            running: false,
            pid: None,
            metrics: None,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

async fn get_brain_wisdom() -> Json<serde_json::Value> {
    let brain_path = "qres_brain.json";

    if let Ok(json) = std::fs::read_to_string(brain_path) {
        if let Ok(value) = serde_json::from_str(&json) {
            return Json(value);
        }
    }

    Json(serde_json::json!({
        "error": "Brain not found or invalid"
    }))
}

async fn get_stats() -> Json<crate::stats::CompressionStats> {
    let stats = crate::stats::GLOBAL_STATS.lock().unwrap();
    Json(stats.clone())
}

async fn get_analytics() -> Json<crate::analytics::BrainHistory> {
    Json(crate::analytics::BrainHistory::load())
}

async fn get_config() -> Json<crate::config::Config> {
    match crate::config::Config::load() {
        Ok(config) => Json(config),
        Err(_) => Json(crate::config::Config::default()),
    }
}

pub async fn run_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let _state = Arc::new(RwLock::new(ApiState::default()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/status", get(get_status))
        .route("/api/swarm/start", post(start_swarm))
        .route("/api/swarm/stop", post(stop_swarm))
        .route("/api/brain/wisdom", get(get_brain_wisdom))
        .route("/api/stats", get(get_stats))
        .route("/api/analytics", get(get_analytics))
        .route("/api/config", get(get_config))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr = if std::env::var("QRES_PUBLIC").is_ok() {
        format!("0.0.0.0:{}", port)
    } else {
        format!("127.0.0.1:{}", port)
    };
    println!("🌐 API Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
