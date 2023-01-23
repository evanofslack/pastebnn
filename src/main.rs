use axum::{extract::Extension, Router};
use clap::{Parser, ValueEnum};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

mod db;
mod handlers;
mod logger;
mod models;

#[derive(Parser, Debug)]
#[clap(
    version, author = env!("CARGO_PKG_HOMEPAGE"), about,
)]
pub struct Settings {
    /// Listening port of http server
    #[clap(long, env("PASTEBNN_API_PORT"), default_value("8080"))]
    pub port: u16,
    /// Listening host of http server
    #[clap(long, env("PASTEBNN_API_HOST"), default_value("0.0.0.0"))]
    pub host: String,
    /// Log level (same syntax as RUST_LOG)
    #[clap(long, env("PASTEBNN_LOG_LEVEL"), default_value("info"))]
    pub log_level: String,
    /// Time in seconds between clearing expired pastes
    #[clap(long, env("PASTEBNN_PURGE_PERIOD"), default_value("60"))]
    pub purge_period: u64,
    /// Max size of paste in bytes
    #[clap(long, env("PASTEBNN_MAX_SIZE_BYTES"), default_value("1024"))]
    pub max_size: usize,

    /// Storage backend
    #[clap(value_enum, default_value_t=StorageBackend::InMemory, env("PASTEBNN_STORAGE_BACKEND"))]
    pub storage_backend: StorageBackend,

    /// Redis username
    #[clap(long, env("PASTEBNN_REDIS_USERNAME"))]
    pub redis_username: Option<String>,
    /// Redis password
    #[clap(long, env("PASTEBNN_REDIS_PASSWORD"))]
    pub redis_password: Option<String>,
    /// Redis host
    #[clap(long, env("PASTEBNN_REDIS_HOST"))]
    pub redis_host: Option<String>,
    /// Redis port
    #[clap(long, env("PASTEBNN_REDIS_PORT"))]
    pub redis_port: Option<i32>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum StorageBackend {
    InMemory,
    Redis,
}

type DynStorer = Arc<dyn db::Storer + Send + Sync>;

#[tokio::main]
async fn main() {
    let settings = Settings::parse();
    let addr = format!("{}:{}", settings.host, settings.port)
        .parse::<SocketAddr>()
        .expect("failed to parse socket address");

    let store: DynStorer = match settings.storage_backend {
        StorageBackend::InMemory => {
            tracing::info!("using in memory store");
            Arc::new(db::inmemory::InMemory::new(settings.max_size).await.unwrap())
        }
        StorageBackend::Redis => {
            tracing::info!("using redis store");
            let conn_info = db::redis::ConnInfo::new(
                settings.redis_host.unwrap(),
                settings.redis_port.unwrap(),
                settings.redis_username,
                settings.redis_password,
            );
            Arc::new(db::redis::Redis::new(conn_info, settings.max_size).await.unwrap())
        }
    };

    let app = create_app(store.clone());

    logger::setup(settings.log_level);

    tracing::info!("pastebnn server listening on {}", addr);
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
        res = store.delete_periodically(settings.purge_period) => {
            if let Err(error) = res {
                tracing::error!("error: {}", error)
            }
        }
    }
}

fn create_app(storer: DynStorer) -> Router {
    let app = handlers::pastes::create_router()
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(Extension(storer))
        .merge(handlers::status::create_router());
    return app;
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    // use hyper::body;

    #[tokio::test]
    async fn root() {
        let mock_store = Arc::new(db::inmemory::InMemory::new(1024).await.unwrap()) as DynStorer;
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
