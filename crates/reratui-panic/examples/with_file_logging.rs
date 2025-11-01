//! Example showing how to integrate file logging with the panic handler.
//!
//! This example demonstrates how to set up tracing to log both to console
//! and to files, capturing panic information in persistent logs.
//!
//! Run with: `cargo run --example with_file_logging`

use std::io;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt,
    prelude::*,
};

fn main() {
    // Step 1: Set up logging with file output
    let _guard = setup_file_logging();

    // Step 2: Set up the panic handler
    reratui_panic::setup_panic_handler();

    info!("Application started with file logging enabled");
    info!("Logs are being written to the 'logs' directory");

    // Example: Normal operation
    info!("Performing some work...");

    // Uncomment to test panic handling with file logging:
    // panic!("This is a test panic that will be logged to file!");

    info!("Application completed successfully");

    // Note: The _guard must be kept alive for the duration of the program
    // to ensure all logs are flushed to disk
}

/// Sets up tracing subscriber for logging to both console and files.
/// Returns a guard that must be kept alive to ensure logs are flushed.
fn setup_file_logging() -> WorkerGuard {
    let env_filter = EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into());

    // Console logging to stderr
    let console_layer = fmt::Layer::new().with_writer(io::stderr).with_target(true);

    // File logging with daily rotation
    let log_file_path = "logs";
    let file_appender = tracing_appender::rolling::daily(log_file_path, "application.log");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::Layer::new()
        .with_writer(non_blocking_appender)
        .with_ansi(false) // Don't use ANSI colors in log files
        .json(); // Use JSON format for structured logging

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    guard
}
