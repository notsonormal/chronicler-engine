use std::{fs, path::Path};

use chrono::Local;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

/// [DOC: docs/architecture/system.md]
pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("Warning: Could not create logs directory: {e}");
        }
    }

    let timestamp = Local::now().format("%Y%m%d");
    let log_file_path = log_dir.join(format!("chronicler_{timestamp}.log"));

    let file_appender = match RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(format!("chronicler_{timestamp}"))
        .filename_suffix("log")
        .build(log_dir)
    {
        Ok(appender) => appender,
        Err(e) => {
            eprintln!("Failed to create file appender: {e}");
            eprintln!("Falling back to console-only logging");
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::new("debug"))
                .init();
            tracing::info!("Logging initialized (console only, file appender failed)");
            return tracing_appender::non_blocking(std::io::stdout()).1;
        }
    };

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true),
        )
        .init();

    tracing::info!("Logging initialized. Log file: {log_file_path:?}");

    guard
}
