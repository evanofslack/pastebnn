use axum::{
    extract::Extension, http::StatusCode, response::IntoResponse, routing::get, Json, Router,
};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
struct AppConfig {
    app_url: String,
}

async fn config(Extension(state): Extension<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let cfg = AppConfig {
        app_url: state.app_url,
    };
    Ok((StatusCode::OK, Json(cfg)))
}

pub fn create_router() -> Router {
    Router::new().route("/api/config", get(config))
}
