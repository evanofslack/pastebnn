use axum::{routing::{get, post, delete},
    http::StatusCode,
    response::IntoResponse,
    extract::{Extension, Path},
    Json, Router
};
// use axum_extra::routing::SpaRouter;
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tower_http::{
    cors::CorsLayer,trace::TraceLayer,
};
    
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
    let paste_store = Arc::new(db::InMemory::default()) as DynStorer;

    let app: Router = Router::new()
        .route("/hello", get(root))
        .route("/api/paste", post(create_paste))
        .route("/api/paste/:key", get(find_paste))
        .route("/api/paste/:key", delete(delete_paste))
        .layer(Extension(paste_store))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    return app
}

async fn root() -> &'static str {
    "hello world"
}

async fn create_paste(
    Json(payload): Json<models::CreatePaste>,
    Extension(state): Extension<DynStorer>,
) -> Result<impl IntoResponse, StatusCode > {

    let paste = models::Paste::new(payload.key, payload.text, payload.expires);
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

    if let Ok(paste) = state.delete(key).await {
        return Ok(Json(paste.clone()));

    } else {
        return Err(StatusCode::NOT_FOUND);
    }
}

type DynStorer = Arc<dyn db::Storer + Send + Sync>;

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
            .oneshot(Request::builder().uri("/hello").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(&body[..], b"hello world");
    }
}

