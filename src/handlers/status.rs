use axum::{Router, http::StatusCode, routing::get};

async fn ping() -> StatusCode {
    StatusCode::NO_CONTENT
}

pub fn create_router() -> Router {
    Router::new().route("/ping", get(ping))
}
