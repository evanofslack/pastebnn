use axum::{http::StatusCode, response::IntoResponse, routing::get_service, Router};
use std::io;
use tower_http::services::{ServeDir, ServeFile};

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

pub fn create_router() -> Router {
    let serve_dir =
        ServeDir::new("web/build").not_found_service(ServeFile::new("web/build/200.html"));
    let serve_dir = get_service(serve_dir).handle_error(handle_error);

    Router::new().fallback(serve_dir)
}
