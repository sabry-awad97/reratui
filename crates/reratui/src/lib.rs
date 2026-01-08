//! # Reratui - A Modern, Reactive TUI Framework for Rust
//!
//! Reratui brings React-inspired component architecture and hooks to terminal user interfaces,
//! enabling developers to build complex, interactive TUI applications with clean, maintainable code.
//!
//! ## Features
//!
//! - **Fiber Architecture** - React-like fiber system with proper effect timing and state batching
//! - **Component-Based Architecture** - Build modular UIs with reusable components
//! - **Hooks System** - Manage state and side effects with React-like hooks
//! - **Async-First** - Built on Tokio with first-class async/await support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use reratui::prelude::*;
//!
//! struct Counter;
//!
//! impl ComponentV2 for Counter {
//!     fn render(&self, area: Rect, buffer: &mut Buffer) {
//!         let (count, set_count) = use_state_v2(|| 0);
//!
//!         if let Some(Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. })) = use_event() {
//!             match code {
//!                 KeyCode::Char('j') => set_count.update(|n| n + 1),
//!                 KeyCode::Char('k') => set_count.update(|n| n - 1),
//!                 KeyCode::Char('q') => request_exit_v2(),
//!                 _ => {}
//!             }
//!         }
//!
//!         let block = Block::default()
//!             .title("Counter")
//!             .borders(Borders::ALL);
//!         let paragraph = Paragraph::new(format!("Count: {}", count))
//!             .alignment(Alignment::Center)
//!             .block(block);
//!         paragraph.render(area, buffer);
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     render_v2(|| Counter).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Available Hooks
//!
//! - [`use_state_v2`](fiber::hooks::use_state_v2) - Local component state with batching
//! - [`use_reducer_v2`](fiber::hooks::use_reducer_v2) - Complex state with actions
//! - [`use_effect_v2`](fiber::hooks::use_effect_v2) - Side effects with proper post-commit timing
//! - [`use_context_v2`](fiber::hooks::use_context_v2) - Share data across components
//! - [`use_ref_v2`](fiber::hooks::use_ref_v2) - Mutable references
//! - [`use_callback_v2`](fiber::hooks::use_callback_v2) - Memoized callbacks
//! - [`use_memo_v2`](fiber::hooks::use_memo_v2) - Memoized values
//! - [`use_event`](fiber::hooks::use_event) - Terminal event handling
//!
//! ## Component Pattern
//!
//! Implement the `ComponentV2` trait for your components:
//!
//! ```rust,no_run
//! use reratui::prelude::*;
//!
//! struct MyComponent {
//!     title: String,
//! }
//!
//! impl ComponentV2 for MyComponent {
//!     fn render(&self, area: Rect, buffer: &mut Buffer) {
//!         let (state, set_state) = use_state_v2(|| 0);
//!         
//!         // Custom layout logic
//!         let chunks = Layout::default()
//!             .direction(Direction::Vertical)
//!             .constraints([Constraint::Length(3), Constraint::Min(0)])
//!             .split(area);
//!         
//!         // Render widgets directly
//!         let paragraph = Paragraph::new(self.title.clone());
//!         paragraph.render(chunks[0], buffer);
//!     }
//! }
//! ```
//!
//! ## Architecture
//!
//! - **reratui-fiber** - Fiber architecture with hooks, runtime, and component system
//!
//! ## Examples
//!
//! See the [`examples/`](https://github.com/sabry-awad97/reratui/tree/main/examples) directory for:
//!
//! - **counter_v2** - Basic state management and event handling with fiber architecture

// Re-export fiber crate as the primary API
pub use reratui_fiber as fiber;

// Re-export ratatui for widget usage
pub use ratatui;

// Re-export crossterm for event handling
pub use crossterm;

/// Prelude module for convenient imports - uses fiber-based v2 APIs
pub mod prelude {
    // Re-export all fiber prelude items
    pub use crate::fiber::prelude::*;
}

/// Render function using fiber architecture
///
/// This function:
/// - Initializes the terminal
/// - Sets up fiber-based hook context for state management
/// - Runs the 4-phase render pipeline (event → render → commit → effect)
/// - Handles user input and component lifecycle
///
/// # Example
/// ```ignore
/// use reratui::prelude::*;
///
/// struct Counter;
///
/// impl ComponentV2 for Counter {
///     fn render(&self, area: Rect, buffer: &mut Buffer) {
///         let (count, set_count) = use_state_v2(|| 0);
///         let paragraph = Paragraph::new(format!("Count: {}", count));
///         paragraph.render(area, buffer);
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     reratui::render_v2(|| Counter).await?;
///     Ok(())
/// }
/// ```
pub use fiber::render_v2;

/// Request application exit
pub use fiber::request_exit_v2;

/// Check if exit was requested
pub use fiber::should_exit_v2;

/// Render options for customizing the render loop
pub use fiber::RenderOptions;

/// Render with custom options
pub use fiber::render_v2_with_options;
