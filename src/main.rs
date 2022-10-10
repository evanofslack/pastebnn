use axum::{routing::{get, post, delete},
    http::StatusCode,
    response::IntoResponse,
    extract::{Extension, Path},
    Json, Router};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tower_http::{cors::CorsLayer,trace::TraceLayer};
use tracing_subscriber;
use tracing::debug;

mod models;
mod db;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    debug!("listening on {}", addr);
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(create_app().into_make_service())
        .await
        .unwrap();
}

fn create_app() -> Router {
    let shared_state = SharedState::default();

    let app = Router::new()
        .route("/", get(root))
        .route("/api/paste", post(create_paste))
        .route("/api/paste/:key", get(find_paste))
        .route("/api/paste/:key", delete(delete_paste))
        .layer(Extension(shared_state))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    return app
}

async fn root() -> &'static str {
    "hello world"
}

async fn create_paste(
    Json(payload): Json<models::CreatePaste>,
    Extension(state): Extension<SharedState>,
) -> impl IntoResponse  {

    let paste = models::Paste::new(payload.key, payload.text, payload.expires);
    state.write().unwrap().create(paste.clone());

    return (StatusCode::CREATED, Json(paste))
}

async fn find_paste(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, StatusCode>  {
    

    if let Ok(paste) = state.write().unwrap().get(key) {
        return Ok(Json(paste.clone()));

    } else {
        return Err(StatusCode::NOT_FOUND);
    }
    // let db = &state.read().unwrap().db;

    // if let Some(value) = db.get(&key) {
    //     return Ok(Json(value.clone()));
    // } else {
    //     return Err(StatusCode::NOT_FOUND);
    // }
}

async fn delete_paste(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, StatusCode>  {

    // let db = &state.write().unwrap().db;
    // if let Some(value) = db.remove(&key){

    // if let Some(value) = &state.write().unwrap().db.remove(&key) {
    //     return Ok(Json(value.clone()));
    // } else {
    //     return Err(StatusCode::NOT_FOUND);
    // }

    if let Ok(paste) = state.write().unwrap().delete(key) {
        return Ok(Json(paste.clone()));

    } else {
        return Err(StatusCode::NOT_FOUND);
    }

}


// #[derive(Default)]
// struct AppState {
//     db: HashMap<String, models::Paste>
// }


type SharedState = Arc<RwLock<dyn db::PasteService>>;
// type SharedState = Arc<RwLock<AppState>>;

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    use axum::{
        http::{Request, StatusCode},
        body::Body,
    };
    use hyper::body;

    #[tokio::test]
    async fn root() {
        let app = create_app();

        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(&body[..], b"hello world");
    }
}

