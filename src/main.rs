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
mod handler;
mod logger;

#[tokio::main]
async fn main() {
    logger::setup();

    let paste_store = Arc::new(db::InMemory::default()) as DynStorer;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    let service = create_app(paste_store.clone()).into_make_service();

    let server = axum::Server::bind(&addr)
        .serve(service)
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

    let app = handler::routes()
        .layer(Extension(storer))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    return app
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
        let mock_store = Arc::new(db::InMemory::default()) as DynStorer;
        let app = create_app(mock_store);

        let resp = app
            .oneshot(Request::builder().uri("/hello").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(&body[..], b"hello world");
    }
}

