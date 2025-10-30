//! Reratui - A reactive TUI framework for Rust
//!
//! This is the main crate that re-exports all functionality from the Reratui framework.

// Re-export core types
pub use reratui_core as core;

// Re-export hooks
pub use reratui_hooks as hooks;

// Re-export runtime
pub use reratui_runtime as runtime;

// Re-export macros
pub use reratui_macro::{component, main, rsx};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::{AnyWidget, Component, ComponentProps, Element, PropValue, Result};
    pub use crate::{component, render, rsx};

    // Re-export hooks
    pub use reratui_hooks::event::use_event;
    pub use reratui_hooks::state::use_state;

    // Re-export runtime utilities
    pub use crate::runtime::{request_exit, should_exit};

    // Re-export ratatui types for convenience
    pub use ratatui::{
        buffer::Buffer,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span, Text},
        widgets::{Block, Borders, Paragraph, Widget},
    };
}

/// Render function that runs the application with hooks support
///
/// This function:
/// - Initializes the terminal
/// - Sets up hook context for state management
/// - Sets up the event loop with global event handling
/// - Renders the root element
/// - Handles user input and component lifecycle
///
/// # Example
/// ```ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn Counter() -> Element {
///     let (count, set_count) = use_state(|| 0);
///     rsx! { <Text text={format!("Count: {}", count)} /> }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // Direct component
///     reratui::render(|| Counter()).await?;
///     
///     // Or with RSX
///     reratui::render(|| {
///         rsx! { <Counter /> }
///     }).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn render<F>(app_fn: F) -> anyhow::Result<()>
where
    F: Fn() -> core::Element + 'static,
{
    runtime::render(app_fn).await
}
