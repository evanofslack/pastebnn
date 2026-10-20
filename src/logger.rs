use tracing_subscriber::EnvFilter;

/// Log using RUST_LOG if set, falling back to the configured level
pub fn setup(log_level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
