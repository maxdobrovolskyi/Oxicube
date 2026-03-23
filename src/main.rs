use tracing_subscriber::EnvFilter;
use tracing::{info, debug, trace, warn, error};

fn main() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .pretty()
        .init();

    info!("App started");
    println!("Hello World");
}