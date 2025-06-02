use env_logger;
use log;

// Run this example with the following command to see the logging output:
// On Unix-like systems: RUST_LOG=trace; cargo run --example logging_demo
// On Windows: $env:RUST_LOG="trace"; cargo run --example logging_demo

fn main() {
    env_logger::init();

    log::error!("This is an error message");
    log::warn!("This is a warning message");
    log::info!("This is an info message");
    log::debug!("This is a debug message");
    log::trace!("This is a trace message");

    println!("Logging demo completed successfully!");
}
