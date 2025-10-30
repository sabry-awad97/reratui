//! Reratui Core - Core runtime and virtual DOM types
//!
//! This crate provides the foundational types and traits for the Reratui framework.

/// Result type alias for Reratui operations
pub type Result<T> = anyhow::Result<T>;

pub mod component;
pub mod layout;
pub mod props;
pub mod vnode;

// Re-export commonly used types
pub use component::Component;
pub use layout::{AnyWidget, BlockWrapper, LayoutWrapper};
pub use props::ComponentProps;
pub use vnode::{Element, PropValue};
