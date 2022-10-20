use axum::{
    extract::Extension,
    Router
};
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tower_http::{
    cors::CorsLayer,trace::TraceLayer,
};
    
mod models;
mod db;
mod handlers;
mod logger;

type DynStorer = Arc<dyn db::Storer + Send + Sync>;

#[tokio::main]
async fn main() {
    let port = 3000;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let paste_store = Arc::new(db::InMemory::default()) as DynStorer;

    let app = create_app(paste_store.clone());

    tracing::debug!("listening on {}", addr);
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen to ctrl-c");
        });

    tokio::select! {
        res = server => {
            if let Err(error) = res {
                tracing::error!("error: {}", error)
            }
        },
        res = paste_store.delete_periodically(60) => {
            if let Err(error) = res {
                tracing::error!("error: {}", error)
            }
        }
    }

}

fn create_app(storer: DynStorer) -> Router {
    logger::setup();

    let app = Router::new()
    .merge(handlers::pastes::create_router())
    .merge(handlers::status::create_router())
    .layer(Extension(storer))
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());
    return app
}


#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    use axum::{
        http::{Request, StatusCode},
        body::Body,
    };
    // use hyper::body;

    #[tokio::test]
    async fn root() {
        let mock_store = Arc::new(db::InMemory::default()) as DynStorer;
        let app = create_app(mock_store);

        let resp = app
            .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // let body = body::to_bytes(resp.into_body()).await.unwrap();
        // assert_eq!(&body[..], b"hello world");
    }
}

