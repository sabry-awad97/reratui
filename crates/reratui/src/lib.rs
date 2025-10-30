//! Reratui - A reactive TUI framework for Rust
//!
//! This is the main crate that re-exports all functionality from the Reratui framework.

// Re-export core types
pub use reratui_core as core;

// Re-export hooks
pub use reratui_hooks as hooks;

// Re-export macros
pub use reratui_macro::{component, main, rsx};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::{Component, Element, Result};
    pub use crate::hooks::{use_callback, use_effect, use_memo, use_ref, use_state};
    pub use crate::{component, render, rsx};

    // Placeholder types for UI components (will be implemented later)
    pub struct Block;
    pub struct Paragraph;
    pub struct Button;
    pub struct Borders;

    impl Borders {
        pub const ALL: Self = Borders;
    }
}

/// Render function that runs the application
///
/// This is a placeholder implementation that will eventually:
/// - Initialize the terminal
/// - Set up the event loop
/// - Render components
/// - Handle user input
///
/// # Example
/// ```ignore
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     reratui::render(|| {
///         rsx! { <MyComponent /> }
///     }).await;
///     Ok(())
/// }
/// ```
pub async fn render<F>(_root: F)
where
    F: Fn() -> core::Element + 'static,
{
    // Placeholder: In a real implementation, this would:
    // 1. Initialize the terminal backend
    // 2. Create the runtime
    // 3. Start the event loop
    // 4. Render the root component
    // 5. Handle events and re-renders

    println!("Reratui application started (placeholder)");
    println!("Press Ctrl+C to exit");

    // Keep the application running
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c");

    println!("\nApplication terminated");
}
