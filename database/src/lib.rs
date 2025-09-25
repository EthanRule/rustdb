#[allow(unused_imports)] //TODO: Remove unused imports
use tracing::{debug, error, info, warn};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub mod document;
pub mod error;
pub mod result;
pub mod storage;
pub mod ui;
pub use crate::document::types::Value;
pub use crate::document::Document;
pub use crate::document::bson;
pub use crate::storage::page_layout;
pub use crate::storage::storage_engine;

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
