//! Basic example of using the panic handler without logging.
//!
//! This example shows the minimal setup for the panic handler.
//! No logging is configured - panics will only be displayed via
//! better_panic (debug) or human_panic (release).
//!
//! Run with: `cargo run --example basic`

fn main() {
    // Set up the panic handler
    reratui_panic::setup_panic_handler();

    println!("Application started");
    println!("Panic handler is configured");

    // Example: Normal operation
    println!("Performing some work...");

    // Uncomment to test panic handling:
    // panic!("This is a test panic!");

    println!("Application completed successfully");
}
