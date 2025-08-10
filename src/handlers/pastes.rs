use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

use crate::{models, AppState};

async fn hello() -> &'static str {
    "hello world"
}

async fn create_paste(
    Json(payload): Json<models::CreatePaste>,
    Extension(state): Extension<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let paste = models::Paste::new(
        payload.key.clone(),
        payload.text.clone(),
        payload.seconds_until_expire,
        payload.burn_on_read,
    );
    tracing::debug!(
        key = %payload.key,
        text_length = payload.text.len(),
        seconds_until_expire = payload.seconds_until_expire,
        burn_on_read = payload.burn_on_read,
        "creating paste"
    );

    match state.store.create(paste.clone()).await {
        Ok(_) => {
            tracing::debug!(
                key = %payload.key,
                text_length = payload.text.len(),
                seconds_until_expire = payload.seconds_until_expire,
                burn_on_read = payload.burn_on_read,
                "created paste successfully"
            );
            Ok((StatusCode::CREATED, Json(paste)))
        }
        Err(e) => {
            tracing::warn!(
                key = %payload.key,
                text_length = payload.text.len(),
                seconds_until_expire = payload.seconds_until_expire,
                burn_on_read = payload.burn_on_read,
                err = e,
                "failed to create paste"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_paste(
    Path(key): Path<String>,
    Extension(state): Extension<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::debug!("get paste {}", key);
    if let Ok(paste) = state.store.get(key).await {
        tracing::debug!("got paste");
        Ok(Json(paste.clone()))
    } else {
        tracing::debug!("could not get paste");
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_paste(
    Path(key): Path<String>,
    Extension(state): Extension<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::debug!("delete paste {}", key);
    if state.store.delete(&key).await.is_ok() {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub fn create_router() -> Router {
    Router::new()
        .route("/hello", get(hello))
        .route("/api/paste", post(create_paste))
        .route("/api/paste/:key", get(get_paste))
        .route("/api/paste/:key", delete(delete_paste))
}
