use axum::Router;
use clap::{ArgAction, Parser, ValueEnum};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;

mod db;
mod handlers;
mod logger;
mod models;
mod ratelimit;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Settings {
    /// Listening port of http server
    #[arg(long, env = "PASTEBNN_API_PORT", default_value_t = 8080)]
    pub port: u16,
    /// Listening host of http server
    #[arg(long, env = "PASTEBNN_API_HOST", default_value = "0.0.0.0")]
    pub host: String,
    /// App base url
    #[arg(long, env = "PASTEBNN_APP_URL", default_value = "localhost")]
    pub app_url: String,
    /// Log level (same syntax as RUST_LOG)
    #[arg(long, env = "PASTEBNN_LOG_LEVEL", default_value = "info")]
    pub log_level: String,
    /// Time in seconds between clearing expired pastes
    #[arg(long, env = "PASTEBNN_PURGE_PERIOD", default_value_t = 60)]
    pub purge_period: u64,
    /// Max size of paste in bytes
    #[arg(long, env = "PASTEBNN_MAX_SIZE_BYTES", default_value_t = 1024)]
    pub max_size: usize,
    /// Max combined size of all pastes in bytes (in memory storage only)
    #[arg(long, env = "PASTEBNN_MAX_TOTAL_BYTES", default_value_t = 256 * 1024 * 1024)]
    pub max_total_bytes: usize,
    /// Max lifetime of an expiring paste in seconds
    #[arg(long, env = "PASTEBNN_MAX_EXPIRE_SECONDS", default_value_t = 30 * 24 * 3600)]
    pub max_expire_seconds: u64,
    /// Max paste API requests per minute per client IP (0 disables)
    #[arg(long, env = "PASTEBNN_RATE_LIMIT_PER_MINUTE", default_value_t = 60)]
    pub rate_limit_per_minute: u32,
    /// Use X-Forwarded-For for client IP (only enable behind a reverse proxy)
    #[arg(long, env = "PASTEBNN_TRUST_PROXY", default_value_t = false, action = ArgAction::Set)]
    pub trust_proxy: bool,

    /// Storage backend
    #[arg(long, value_enum, env = "PASTEBNN_STORAGE_BACKEND", default_value_t = StorageBackend::InMemory)]
    pub storage_backend: StorageBackend,

    /// Redis username
    #[arg(long, env = "PASTEBNN_REDIS_USERNAME")]
    pub redis_username: Option<String>,
    /// Redis password
    #[arg(long, env = "PASTEBNN_REDIS_PASSWORD", hide_env_values = true)]
    pub redis_password: Option<String>,
    /// Redis host (required for redis storage)
    #[arg(
        long,
        env = "PASTEBNN_REDIS_HOST",
        required_if_eq("storage_backend", "redis")
    )]
    pub redis_host: Option<String>,
    /// Redis port
    #[arg(long, env = "PASTEBNN_REDIS_PORT", default_value_t = 6379)]
    pub redis_port: u16,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum StorageBackend {
    InMemory,
    Redis,
}

type DynStorer = Arc<dyn db::Storer + Send + Sync>;

#[derive(Clone)]
struct AppState {
    store: DynStorer,
    app_url: String,
    max_size: usize,
    max_expire_seconds: u64,
}

#[tokio::main]
async fn main() {
    let settings = Settings::parse();
    logger::setup(&settings.log_level);

    let addr = format!("{}:{}", settings.host, settings.port)
        .parse::<SocketAddr>()
        .expect("failed to parse socket address");

    let store: DynStorer = match settings.storage_backend {
        StorageBackend::InMemory => {
            tracing::info!("using in memory store");
            Arc::new(
                db::inmemory::InMemory::new(settings.max_size, settings.max_total_bytes)
                    .await
                    .expect("failed to create in memory store"),
            )
        }
        StorageBackend::Redis => {
            tracing::info!("using redis store");
            let conn_info = db::redis::ConnInfo::new(
                settings.redis_host.expect("redis host is required"),
                settings.redis_port,
                settings.redis_username,
                settings.redis_password,
            );
            Arc::new(
                db::redis::Redis::new(conn_info, settings.max_size)
                    .await
                    .expect("failed to connect to redis"),
            )
        }
    };

    let app_state = AppState {
        store: store.clone(),
        app_url: settings.app_url,
        max_size: settings.max_size,
        max_expire_seconds: settings.max_expire_seconds,
    };
    let limiter = Arc::new(ratelimit::RateLimiter::new(
        settings.rate_limit_per_minute,
        settings.trust_proxy,
    ));

    let app = create_app(app_state, limiter.clone());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind listener");
    tracing::info!("pastebnn server listening on {}", addr);
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal());

    tokio::select! {
        res = server => {
            if let Err(error) = res {
                tracing::error!("error: {}", error)
            }
        },
        _ = store.delete_periodically(settings.purge_period) => {},
        _ = limiter.sweep_periodically() => {},
    }
}

/// Resolve on ctrl-c or SIGTERM (sent by `docker stop`)
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl-c");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutting down");
}

fn create_app(state: AppState, limiter: Arc<ratelimit::RateLimiter>) -> Router {
    handlers::pastes::create_router(state.max_size, limiter)
        .merge(handlers::config::create_router())
        .with_state(state)
        .merge(handlers::status::create_router())
        .merge(handlers::frontend::create_router())
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::ConnectInfo,
        http::{Request, Response, StatusCode},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    async fn test_app(max_size: usize, rate_limit: u32) -> Router {
        let mock_store = Arc::new(
            db::inmemory::InMemory::new(max_size, 1024 * 1024)
                .await
                .unwrap(),
        ) as DynStorer;
        let state = AppState {
            store: mock_store,
            app_url: "test".to_string(),
            max_size,
            max_expire_seconds: 3600,
        };
        create_app(
            state,
            Arc::new(ratelimit::RateLimiter::new(rate_limit, false)),
        )
    }

    fn post_paste(body: Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/paste")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn delete_paste(key: &str, token: Option<&str>) -> Request<Body> {
        let mut req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/paste/{}", key));
        if let Some(token) = token {
            req = req.header("x-delete-token", token);
        }
        req.body(Body::empty()).unwrap()
    }

    fn get_paste(key: &str) -> Request<Body> {
        Request::builder()
            .uri(format!("/api/paste/{}", key))
            .body(Body::empty())
            .unwrap()
    }

    async fn json_body(resp: Response<Body>) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn root() {
        let app = test_app(1024, 0).await;

        let resp = app
            .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn create_generates_key_and_token() {
        let app = test_app(1024, 0).await;

        let resp = app
            .clone()
            .oneshot(post_paste(json!({"text": "hello", "burn_on_read": false})))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let created = json_body(resp).await;
        let key = created["key"].as_str().unwrap();
        assert_eq!(key.len(), models::GENERATED_KEY_LEN);
        assert_eq!(created["delete_token"].as_str().unwrap().len(), 32);

        let resp = app.oneshot(get_paste(key)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let fetched = json_body(resp).await;
        assert_eq!(fetched["text"], "hello");
        assert!(fetched.get("delete_token").is_none());
    }

    #[tokio::test]
    async fn delete_requires_token() {
        let app = test_app(1024, 0).await;
        let resp = app
            .clone()
            .oneshot(post_paste(json!({"key": "mine", "text": "hello"})))
            .await
            .unwrap();
        let token = json_body(resp).await["delete_token"]
            .as_str()
            .unwrap()
            .to_string();

        let resp = app
            .clone()
            .oneshot(delete_paste("mine", None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        let resp = app
            .clone()
            .oneshot(delete_paste("mine", Some("wrong")))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        let resp = app
            .clone()
            .oneshot(delete_paste("missing", Some(&token)))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let resp = app
            .clone()
            .oneshot(delete_paste("mine", Some(&token)))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp = app.oneshot(get_paste("mine")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn create_rejects_bad_input() {
        let app = test_app(8, 0).await;
        let cases = [
            (json!({"key": "dup", "text": "a"}), StatusCode::CREATED),
            (json!({"key": "dup", "text": "b"}), StatusCode::CONFLICT),
            (
                json!({"key": "bad key", "text": "a"}),
                StatusCode::BAD_REQUEST,
            ),
            (json!({"text": "123456789"}), StatusCode::PAYLOAD_TOO_LARGE),
            (
                json!({"text": "a", "seconds_until_expire": 3601}),
                StatusCode::BAD_REQUEST,
            ),
            (
                json!({"text": "a", "seconds_until_expire": 0}),
                StatusCode::BAD_REQUEST,
            ),
            (json!({"nope": true}), StatusCode::BAD_REQUEST),
        ];
        for (body, status) in cases {
            let resp = app.clone().oneshot(post_paste(body.clone())).await.unwrap();
            assert_eq!(resp.status(), status, "body: {}", body);
        }
    }

    #[tokio::test]
    async fn oversized_body_rejected() {
        let app = test_app(8, 0).await;
        let resp = app
            .oneshot(post_paste(json!({"text": "a".repeat(1024 * 1024)})))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn rate_limited() {
        let app = test_app(1024, 2).await;
        let addr = ConnectInfo(SocketAddr::from(([1, 2, 3, 4], 5000)));
        let mut statuses = Vec::new();
        for _ in 0..3 {
            let mut req = get_paste("missing");
            req.extensions_mut().insert(addr);
            statuses.push(app.clone().oneshot(req).await.unwrap().status());
        }
        assert_eq!(
            statuses,
            [
                StatusCode::NOT_FOUND,
                StatusCode::NOT_FOUND,
                StatusCode::TOO_MANY_REQUESTS
            ]
        );

        // non paste routes are not limited
        let mut req = Request::builder().uri("/ping").body(Body::empty()).unwrap();
        req.extensions_mut().insert(addr);
        assert_eq!(
            app.oneshot(req).await.unwrap().status(),
            StatusCode::NO_CONTENT
        );
    }
}
