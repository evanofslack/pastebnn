use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;

use crate::{
    AppState,
    db::StoreError,
    models::{self, CreatePaste, CreatedPaste, PasteView},
    ratelimit::{self, RateLimiter},
};

/// Attempts at generating an unused random key before giving up
const KEY_GEN_ATTEMPTS: usize = 5;
/// Allowance on top of the paste text for JSON fields and escaping
const JSON_OVERHEAD_BYTES: usize = 4096;
const DELETE_TOKEN_HEADER: &str = "x-delete-token";

fn error_status(err: &StoreError) -> StatusCode {
    match err {
        StoreError::NotFound => StatusCode::NOT_FOUND,
        StoreError::AlreadyExists => StatusCode::CONFLICT,
        StoreError::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        StoreError::StorageFull => StatusCode::INSUFFICIENT_STORAGE,
        StoreError::Internal(msg) => {
            tracing::warn!(err = %msg, "store error");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn hello() -> &'static str {
    "hello world"
}

async fn create_paste(
    State(state): State<AppState>,
    body: Bytes,
) -> Result<impl IntoResponse, StatusCode> {
    // body size is capped by DefaultBodyLimit, parse manually to return 400 on bad json
    let payload: CreatePaste =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    if let Some(secs) = payload.seconds_until_expire
        && (secs == 0 || secs > state.max_expire_seconds)
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let custom_key = payload.key.filter(|key| !key.is_empty());
    if let Some(key) = &custom_key
        && !models::valid_key(key)
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    tracing::debug!(
        custom_key = custom_key.is_some(),
        text_length = payload.text.len(),
        seconds_until_expire = payload.seconds_until_expire,
        burn_on_read = payload.burn_on_read,
        "creating paste"
    );

    let attempts = if custom_key.is_some() {
        1
    } else {
        KEY_GEN_ATTEMPTS
    };
    for _ in 0..attempts {
        let key = custom_key.clone().unwrap_or_else(models::generate_key);
        let paste = models::Paste::new(
            key,
            payload.text.clone(),
            payload.seconds_until_expire,
            payload.burn_on_read,
        );
        match state.store.create(paste.clone()).await {
            Ok(()) => {
                tracing::debug!(key = %paste.key, "created paste successfully");
                return Ok((StatusCode::CREATED, Json(CreatedPaste::from(&paste))));
            }
            Err(StoreError::AlreadyExists) if custom_key.is_none() => continue,
            Err(e) => {
                tracing::debug!(err = %e, "failed to create paste");
                return Err(error_status(&e));
            }
        }
    }
    tracing::warn!("failed to generate unused paste key");
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_paste(
    Path(key): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::debug!("get paste {}", key);
    match state.store.get(&key).await {
        Ok(paste) => Ok(Json(PasteView::from(&paste))),
        Err(e) => Err(error_status(&e)),
    }
}

async fn delete_paste(
    Path(key): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::debug!("delete paste {}", key);
    let token = headers
        .get(DELETE_TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;
    let paste = state.store.peek(&key).await.map_err(|e| error_status(&e))?;
    if !models::token_matches(&paste.delete_token, token) {
        return Err(StatusCode::FORBIDDEN);
    }
    state
        .store
        .delete(&key)
        .await
        .map_err(|e| error_status(&e))?;
    Ok(StatusCode::OK)
}

/// Max request body: escaping can grow text in JSON, so leave headroom beyond max_size
pub fn body_limit(max_size: usize) -> usize {
    max_size
        .saturating_mul(2)
        .saturating_add(JSON_OVERHEAD_BYTES)
}

pub fn create_router(max_size: usize, limiter: Arc<RateLimiter>) -> Router<AppState> {
    Router::new()
        .route("/api/paste", post(create_paste))
        .route("/api/paste/{key}", get(get_paste).delete(delete_paste))
        .route_layer(middleware::from_fn_with_state(limiter, ratelimit::limit))
        .layer(DefaultBodyLimit::max(body_limit(max_size)))
        .route("/hello", get(hello))
}
