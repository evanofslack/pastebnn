use axum::{routing::{get, post, delete},
    http::StatusCode,
    response::IntoResponse,
    extract::{Extension, Path},
    Json, Router
};

use super::models;
use super::DynStorer;


async fn root() -> &'static str {
    "hello world"
}

async fn create_paste(
    Json(payload): Json<models::CreatePaste>,
    Extension(state): Extension<DynStorer>,
) -> Result<impl IntoResponse, StatusCode > {

    let paste = models::Paste::new(payload.key, payload.text, payload.seconds_until_expire);
    if let Ok(paste) = state.create(paste.clone()).await {
        return Ok((StatusCode::CREATED, Json(paste)))
    } else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
}

async fn find_paste(
    Path(key): Path<String>,
    Extension(state): Extension<DynStorer>,
) -> Result<impl IntoResponse, StatusCode>  {
    
    if let Ok(paste) = state.get(key).await {
        return Ok(Json(paste.clone()));

    } else {
        return Err(StatusCode::NOT_FOUND);
    }
}

async fn delete_paste(
    Path(key): Path<String>,
    Extension(state): Extension<DynStorer>,
) -> Result<impl IntoResponse, StatusCode>  {

    if let Ok(paste) = state.delete(&key).await {
        return Ok(Json(paste.clone()));

    } else {
        return Err(StatusCode::NOT_FOUND);
    }
}

pub fn routes() -> Router {
    let router: Router = Router::new()
        .route("/hello", get(root))
        .route("/api/paste", post(create_paste))
        .route("/api/paste/:key", get(find_paste))
        .route("/api/paste/:key", delete(delete_paste));
    
    return router
}