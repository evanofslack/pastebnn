use axum::{routing::{get, post, delete},
    http::StatusCode,
    response::IntoResponse,
    extract::{Extension, Path},
    Json, Router};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing_subscriber;
use tracing::debug;




#[tokio::main]
async fn main() {
    
    tracing_subscriber::fmt::init();

    let shared_state = SharedState::default();

    let app = Router::new()
        .route("/paste", post(create_paste))
        .route("/paste/:key", get(find_paste))
        .route("/paste/:key", delete(delete_paste))
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    debug!("listening on {}", addr);
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_paste(
    Json(payload): Json<CreatePaste>,
    Extension(state): Extension<SharedState>,
) -> impl IntoResponse  {

    let paste = Paste {
        id: Uuid::new_v4(),
        key: payload.key,
        text: payload.text,
    };

    state.write().unwrap().db.insert(paste.key.clone(), paste.clone());

    return (StatusCode::CREATED, Json(paste))
}

async fn find_paste(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, StatusCode>  {

    let db = &state.read().unwrap().db;

    if let Some(value) = db.get(&key) {
        return Ok(Json(value.clone()));
    } else {
        return Err(StatusCode::NOT_FOUND);
    }
}

async fn delete_paste(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, StatusCode>  {

    // let db = &state.write().unwrap().db;
    // if let Some(value) = db.remove(&key){

    if let Some(value) = &state.write().unwrap().db.remove(&key) {
        return Ok(Json(value.clone()));
    } else {
        return Err(StatusCode::NOT_FOUND);
    }
}

#[derive(Deserialize)]
struct CreatePaste {
    key: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Paste {
    id: Uuid,
    key: String,
    text: String,
}

#[derive(Default)]
struct AppState {
    db: HashMap<String, Paste>
}

type SharedState = Arc<RwLock<AppState>>;
