//! Example showing how to integrate logging with the panic handler.
//!
//! This example demonstrates how to set up tracing/logging before
//! initializing the panic handler, allowing you to capture panic
//! information in your logs.
//!
//! Run with: `cargo run --example with_logging`

use std::io;
use tracing::info;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt,
    prelude::*,
};

fn main() {
    // Step 1: Set up your logging/tracing subscriber BEFORE the panic handler
    setup_logging();

    // Step 2: Set up the panic handler
    reratui_panic::setup_panic_handler();

    info!("Application started with logging enabled");

    // Example: Normal operation
    info!("Performing some work...");

    // Uncomment to test panic handling with logging:
    // panic!("This is a test panic with logging!");

    info!("Application completed successfully");
}

/// Sets up tracing subscriber for logging to console and/or files.
fn setup_logging() {
    let env_filter = EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into());

    // Console logging to stderr
    let console_layer = fmt::Layer::new()
        .with_writer(io::stderr)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .init();
}
