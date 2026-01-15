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
//! - **Cross-Thread State Updates** - Background tasks can safely update UI state
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
//! - [`use_state_v2`](hooks::use_state_v2) - Local component state with batching
//! - [`use_reducer_v2`](hooks::use_reducer_v2) - Complex state with actions
//! - [`use_effect_v2`](hooks::use_effect_v2) - Side effects with proper post-commit timing
//! - [`use_context_v2`](hooks::use_context_v2) - Share data across components
//! - [`use_ref_v2`](hooks::use_ref_v2) - Mutable references
//! - [`use_callback_v2`](hooks::use_callback_v2) - Memoized callbacks
//! - [`use_memo_v2`](hooks::use_memo_v2) - Memoized values
//! - [`use_event`](hooks::use_event) - Terminal event handling
//! - [`use_interval_v2`](hooks::use_interval_v2) - Periodic callbacks
//! - [`use_timeout_v2`](hooks::use_timeout_v2) - Delayed callbacks
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
//! ## Key Types
//!
//! - [`FiberId`] - Unique identifier for a component instance
//! - [`Fiber`] - A mounted component instance with its own hook state
//! - [`FiberTree`] - Global fiber tree tracking all mounted components
//! - [`ComponentV2`] - Trait for implementing components
//!
//! ## Examples
//!
//! See the [`examples/`](https://github.com/sabry-awad97/reratui/tree/main/examples) directory for:
//!
//! - **counter_v2** - Basic state management and event handling
//! - **command_palette** - Complex UI with animations and keyboard navigation

// Core fiber types
mod fiber;
pub mod fiber_tree;

// Consolidated render context
pub mod render_context;

// Element types (virtual DOM)
pub mod element;

// Event system
pub mod event;

// Global event handlers
pub mod global_events;

// Panic handler
pub mod panic_handler;

// ComponentV2 trait and related types
mod component;

// Context management
pub mod context_stack;

// Hooks (React-like APIs)
pub mod hooks;

// Scheduler (batching, effects, reconciliation)
pub mod scheduler;

// Runtime (render loop)
mod runtime;

// Strict mode for development
mod strict_mode;

// Re-exports for public API
pub use component::{ComponentArea, ComponentV2, reset_component_position_counter};
pub use context_stack::ContextStack;
pub use element::{Element, RenderableComponentV2};
pub use event::{
    clear_current_event, get_current_event, reset_all_fiber_event_flags, set_current_event,
};
pub use fiber::{
    AsyncCleanupFn, AsyncEffectFn, AsyncEffectFuture, AsyncPendingEffect, CleanupFn, Fiber,
    FiberId, PendingEffect,
};
pub use fiber_tree::FiberTree;
pub use global_events::{clear_global_handlers, on_global_event, process_global_event};
pub use panic_handler::setup_panic_handler;
pub use render_context::{
    RenderContext, clear_render_context, init_render_context, is_render_context_initialized,
    with_render_context, with_render_context_mut,
};
pub use runtime::{
    RenderOptions, is_in_render_phase, render_v2, render_v2_with_options, request_exit_v2,
    reset_exit_v2, should_exit_v2, warn_if_effect_during_render,
};
pub use strict_mode::{StrictMode, is_strict_mode_enabled, set_strict_mode_enabled};

// Re-export crossterm event types for convenience
pub use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

// Re-export ratatui for standalone usage (no need for separate ratatui dependency)
pub use ratatui;

// Re-export commonly used ratatui types at crate root for convenience
pub use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::component::{ComponentArea, ComponentV2};
    pub use crate::context_stack::ContextStack;
    pub use crate::element::{Element, RenderableComponentV2};
    pub use crate::fiber::{Fiber, FiberId};
    pub use crate::fiber_tree::FiberTree;
    pub use crate::render_context::{
        RenderContext, clear_render_context, init_render_context, is_render_context_initialized,
        with_render_context, with_render_context_mut,
    };
    pub use crate::runtime::{
        RenderOptions, is_in_render_phase, render_v2, render_v2_with_options, request_exit_v2,
        should_exit_v2,
    };
    pub use crate::strict_mode::{StrictMode, is_strict_mode_enabled, set_strict_mode_enabled};

    // Re-export hooks
    pub use crate::hooks::{
        DispatchV2, EffectEventV2, HistoryHandle, IntervalHandle, RefV2, StateSetterV2,
        TimeoutHandle, try_use_context_v2, use_async_effect_once, use_async_effect_v2,
        use_callback_v2, use_context_provider_v2, use_context_v2, use_effect_event_v2,
        use_effect_once, use_effect_v2, use_event, use_history_v2, use_id_v2, use_interval_v2,
        use_memo_v2, use_reducer_v2, use_ref_v2, use_state_v2, use_timeout_v2,
    };

    // Re-export crossterm event types
    pub use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    // Re-export ratatui types for standalone usage
    pub use ratatui::{
        buffer::Buffer,
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style, Stylize},
        text::{Line, Span, Text},
        widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
    };
}
