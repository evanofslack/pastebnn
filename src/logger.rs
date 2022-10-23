use tracing_subscriber;

pub fn setup(log_level: String) {
    std::env::set_var("RUST_LOG", std::env::var("RUST_LOG").unwrap_or(log_level));
    tracing_subscriber::fmt::init();
}
