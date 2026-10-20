use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
struct AppConfig {
    app_url: String,
}

async fn config(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let cfg = AppConfig {
        app_url: state.app_url,
    };
    Ok((StatusCode::OK, Json(cfg)))
}

pub fn create_router() -> Router<AppState> {
    Router::new().route("/api/config", get(config))
}
