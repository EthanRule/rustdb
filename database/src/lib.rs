#[allow(unused_imports)] //TODO: Remove unused imports.
use tracing::{debug, error, info, warn};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub mod collection;
pub mod document;
pub mod error;
pub mod index;
pub mod query;
pub mod result;
pub mod server;
pub mod storage;

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .with_file(true)
                .with_line_number(true),
        )
        .init();

    info!("Tracing initialized");
}
