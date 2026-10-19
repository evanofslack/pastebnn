use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

pub fn create_router() -> Router {
    // unknown paths get the SPA shell so client side routing can handle them
    let serve_dir = ServeDir::new("web/build").fallback(ServeFile::new("web/build/200.html"));

    Router::new().fallback_service(serve_dir)
}
