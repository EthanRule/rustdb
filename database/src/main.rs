use database::init_tracing;
use tracing::info;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // TODO: Consider adding return type to main fn and error handling.
    init_tracing();

    info!("Application starting up!");

    Ok(())
}
