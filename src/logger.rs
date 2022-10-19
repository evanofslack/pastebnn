use std::env;
use tracing_subscriber;

pub fn setup() {
    if env::var_os("RUST_LOG").is_none() {
        let level = "debug";
        let env = format!("pastebnn={},tower_http={}", level, level);

        env::set_var("RUST_LOG", env);
    }
    tracing_subscriber::fmt::init();
}