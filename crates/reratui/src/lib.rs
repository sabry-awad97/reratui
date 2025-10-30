//! # Reratui - A Modern, Reactive TUI Framework for Rust
//!
//! Reratui brings React-inspired component architecture and hooks to terminal user interfaces,
//! enabling developers to build complex, interactive TUI applications with clean, maintainable code.
//!
//! ## Features
//!
//! - **Component-Based Architecture** - Build modular UIs with reusable components
//! - **Hooks System** - Manage state and side effects with React-like hooks
//! - **RSX Macro** - Declarative JSX-like syntax for intuitive UI construction
//! - **Type-Safe Props** - Automatic prop validation with `#[derive(Props)]`
//! - **Hook Rules Validation** - Compile-time enforcement of the Rules of Hooks
//! - **Async-First** - Built on Tokio with first-class async/await support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use reratui::prelude::*;
//!
//! #[component]
//! fn Counter() -> Element {
//!     let (count, set_count) = use_state(|| 0);
//!
//!     if let Some(Event::Key(key)) = use_event()
//!         && key.is_press()
//!     {
//!         match key.code {
//!             KeyCode::Char('j') => set_count.update(|n| n + 1),
//!             KeyCode::Char('k') => set_count.update(|n| n - 1),
//!             _ => {}
//!         }
//!     }
//!
//!     rsx! {
//!         <Block title="Counter" borders={Borders::ALL}>
//!             <Paragraph alignment={Alignment::Center}>
//!                 {format!("Count: {}", count.get())}
//!             </Paragraph>
//!         </Block>
//!     }
//! }
//!
//! #[reratui::main]
//! async fn main() -> Result<()> {
//!     render(|| rsx! { <Counter /> }).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Available Hooks
//!
//! - [`use_state`](hooks::state::use_state) - Local component state
//! - [`use_reducer`](hooks::reducer::use_reducer) - Complex state with actions
//! - [`use_effect`](hooks::effect::use_effect) - Side effects with dependencies
//! - [`use_context`](hooks::context::use_context) - Share data across components
//! - [`use_ref`](hooks::ref_hook::use_ref) - Mutable references
//! - [`use_callback`](hooks::callback) - Memoized callbacks
//! - [`use_event`](hooks::event::use_event) - Terminal event handling
//! - [`use_frame`](hooks::frame::use_frame) - Frame timing and context
//! - [`use_area`](hooks::area::use_area) - Component rendering area
//!
//! ## Component Patterns
//!
//! ### Simple Function Component
//!
//! ```rust,no_run
//! use reratui::prelude::*;
//!
//! #[component]
//! fn Greeting() -> Element {
//!     rsx! {
//!         <Paragraph>{"Hello, World!"}</Paragraph>
//!     }
//! }
//! ```
//!
//! ### Component with Props
//!
//! ```rust,no_run
//! use reratui::prelude::*;
//!
//! #[derive(Props)]
//! struct GreetingProps {
//!     name: String,
//! }
//!
//! #[component]
//! fn Greeting(props: &GreetingProps) -> Element {
//!     rsx! {
//!         <Paragraph>{format!("Hello, {}!", props.name)}</Paragraph>
//!     }
//! }
//! ```
//!
//! ### Complex Component with Manual Render
//!
//! For components that need full control over rendering:
//!
//! ```rust,no_run
//! use reratui::prelude::*;
//!
//! struct MyComponent {
//!     title: String,
//! }
//!
//! impl Component for MyComponent {
//!     fn render(&self, area: Rect, buffer: &mut Buffer) {
//!         let (state, set_state) = use_state(|| 0);
//!         
//!         // Custom layout logic
//!         let chunks = Layout::default()
//!             .direction(Direction::Vertical)
//!             .constraints([Constraint::Length(3), Constraint::Min(0)])
//!             .split(area);
//!         
//!         // Render sub-components
//!         let vnode = rsx! { <Paragraph>{self.title.clone()}</Paragraph> };
//!         vnode.render(chunks[0], buffer);
//!     }
//! }
//! ```
//!
//! ## Architecture
//!
//! Reratui follows SOLID principles and Domain-Driven Design:
//!
//! - **reratui-core** - Core types (Element, Component, VNode)
//! - **reratui-macro** - Procedural macros (component, rsx, Props)
//! - **reratui-hooks** - Hook implementations
//! - **reratui-runtime** - Event loop and rendering runtime
//!
//! ## Rules of Hooks
//!
//! Hooks must follow these rules (enforced at compile-time):
//!
//! 1. Only call hooks at the top level of your component
//! 2. Don't call hooks inside loops, conditions, or nested functions
//! 3. Hooks must be called in the same order every render
//!
//! The `#[component]` macro validates these rules and provides helpful error messages.
//!
//! ## Examples
//!
//! See the [`examples/`](https://github.com/sabry-awad97/reratui/tree/main/examples) directory for:
//!
//! - **counter** - Basic state management and event handling
//! - **rsx_demo** - Comprehensive RSX macro features
//! - **router** - Navigation and routing (coming soon)

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
