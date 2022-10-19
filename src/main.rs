use axum::{
    extract::Extension,
    Router
};
use std::{
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tower_http::{
    cors::CorsLayer,trace::TraceLayer,
};
    
use tracing_subscriber;
use tokio::time; 

mod models;
mod db;
mod handler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let paste_store = Arc::new(db::InMemory::default()) as DynStorer;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

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
                println!("error: {}", error)
            }
        },
        res = remove_periodically(paste_store, 5) => {
            if let Err(error) = res {
                println!("error: {}", error)
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

async fn remove_periodically(storer: DynStorer, period_seconds: u64) -> Result<(), &'static str> {
    let mut interval = time::interval(Duration::from_secs(period_seconds));

    loop {
        interval.tick().await;
        storer.delete_expired().await?
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

