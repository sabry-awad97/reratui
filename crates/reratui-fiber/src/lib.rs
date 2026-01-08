//! # reratui-fiber
//!
//! React-like fiber architecture for Reratui with proper effect timing,
//! state batching, and component lifecycle management.
//!
//! This crate provides React-like semantics including:
//! - Fiber-based component instances with isolated hook state
//! - Post-commit effect execution
//! - State update batching
//! - Proper context provider lifecycle
//! - Strict mode for development
//!
//! ## Key Types
//!
//! - [`FiberId`] - Unique identifier for a component instance
//! - [`Fiber`] - A mounted component instance with its own hook state
//! - [`FiberTree`] - Global fiber tree tracking all mounted components
//!
//! ## Usage
//!
//! ```rust,ignore
//! use reratui_fiber::prelude::*;
//!
//! #[component]
//! fn Counter() -> Element {
//!     let (count, set_count) = use_state_v2(|| 0);
//!     
//!     use_effect_v2(|| {
//!         println!("Count changed to: {}", count);
//!         None
//!     }, (count,));
//!     
//!     rsx! { <Text text={count.to_string()} /> }
//! }
//!
//! render_v2(|| rsx! { <Counter /> }).await?;
//! ```

// Core fiber types
mod fiber;
pub mod fiber_tree;

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
pub use event::{clear_current_event, get_current_event, mark_event_processed, set_current_event};
pub use fiber::{
    AsyncCleanupFn, AsyncEffectFn, AsyncEffectFuture, AsyncPendingEffect, CleanupFn, Fiber,
    FiberId, PendingEffect,
};
pub use fiber_tree::FiberTree;
pub use global_events::{clear_global_handlers, on_global_event, process_global_event};
pub use panic_handler::setup_panic_handler;
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
