use axum::{
    body::Body,
    extract::{Extension, Path, RawBody},
    http::{header, HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use hyper::body::HttpBody;
use std::sync::Arc;

use crate::{
    db::StoreError,
    models::{self, CreatePaste, CreatedPaste, PasteView},
    ratelimit::{self, RateLimiter},
    AppState,
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

/// Read the request body, rejecting it as soon as it exceeds limit bytes
async fn read_body(
    headers: &HeaderMap,
    mut body: Body,
    limit: usize,
) -> Result<Vec<u8>, StatusCode> {
    let content_length = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok());
    if matches!(content_length, Some(len) if len > limit) {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let mut buf = Vec::with_capacity(content_length.unwrap_or(0));
    while let Some(chunk) = body.data().await {
        let chunk = chunk.map_err(|_| StatusCode::BAD_REQUEST)?;
        if buf.len() + chunk.len() > limit {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

async fn hello() -> &'static str {
    "hello world"
}

async fn create_paste(
    Extension(state): Extension<AppState>,
    headers: HeaderMap,
    RawBody(body): RawBody,
) -> Result<impl IntoResponse, StatusCode> {
    // escaping can grow text in JSON, so leave headroom beyond max_size
    let limit = state
        .max_size
        .saturating_mul(2)
        .saturating_add(JSON_OVERHEAD_BYTES);
    let bytes = read_body(&headers, body, limit).await?;
    let payload: CreatePaste =
        serde_json::from_slice(&bytes).map_err(|_| StatusCode::BAD_REQUEST)?;

    if let Some(secs) = payload.seconds_until_expire {
        if secs == 0 || secs > state.max_expire_seconds {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let custom_key = payload.key.filter(|key| !key.is_empty());
    if let Some(key) = &custom_key {
        if !models::valid_key(key) {
            return Err(StatusCode::BAD_REQUEST);
        }
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
    Extension(state): Extension<AppState>,
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
    Extension(state): Extension<AppState>,
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

pub fn create_router(limiter: Arc<RateLimiter>) -> Router {
    Router::new()
        .route("/api/paste", post(create_paste))
        .route("/api/paste/:key", get(get_paste))
        .route("/api/paste/:key", delete(delete_paste))
        .layer(middleware::from_fn(
            move |req: Request<Body>, next: Next<Body>| {
                ratelimit::limit(limiter.clone(), req, next)
            },
        ))
        .route("/hello", get(hello))
}
