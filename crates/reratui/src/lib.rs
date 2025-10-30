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

// Re-export tokio for use by the main macro
#[doc(hidden)]
pub use tokio;

// Re-export ratatui for use by the rsx macro
#[doc(hidden)]
pub use ratatui;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::{AnyWidget, Component, ComponentProps, Element, PropValue, Result};
    pub use crate::{component, render, rsx};

    // Re-export hooks
    pub use crate::hooks::area::{ComponentArea, use_area};
    pub use crate::hooks::callback::{Callback, IntoCallbackProp};
    pub use crate::hooks::context::{use_context, use_context_provider};
    pub use crate::hooks::effect::{use_effect, use_effect_always, use_effect_once};
    pub use crate::hooks::event::use_event;
    pub use crate::hooks::frame::{FrameContext, FrameExt, FrameInfo, use_frame};
    pub use crate::hooks::reducer::{DispatchFn, ReducerStateHandle, use_reducer};
    pub use crate::hooks::ref_hook::use_ref;
    pub use crate::hooks::state::use_state;

    // Re-export Props derive macro
    pub use reratui_macro::Props;

    // Re-export runtime utilities
    pub use crate::runtime::{request_exit, should_exit};

    // Re-export ratatui types for convenience
    pub use ratatui::{
        buffer::Buffer,
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span, Text},
        widgets::{Block, Borders, Paragraph, Tabs, Widget},
    };

    pub use crossterm::event::*;
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
