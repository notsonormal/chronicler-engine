use std::{fs, path::Path};

use chrono::Local;

/// [DOC: docs/architecture/system.md]
pub fn init_logging() {
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("Warning: Could not create logs directory: {e}");
        }
    }

    let timestamp = Local::now().format("%Y%m%d");
    let log_file_path = log_dir.join(format!("chronicler_{timestamp}.log"));

    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        Ok(log_file) => {
            // Configure env_logger to write to the file
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Debug)
                .target(env_logger::Target::Pipe(Box::new(log_file)))
                .init();
        }
        Err(e) => {
            eprintln!("Warning: Could not open log file {log_file_path:?}: {e}");
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Debug)
                .init();
        }
    }

    // Also print to console so user sees output when running cargo run
    println!("Logging to file: {log_file_path:?}");

    log::info!("Logging initialized. Log file: {log_file_path:?}");
}
