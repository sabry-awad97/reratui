//! Reratui Core - Core runtime and virtual DOM types
//!
//! This crate provides the foundational types and traits for the Reratui framework.

use std::fmt;

/// Result type alias for Reratui operations
pub type Result<T> = anyhow::Result<T>;

/// Represents a renderable element in the virtual DOM
///
/// This is a placeholder type that will eventually contain the virtual DOM node structure.
#[derive(Clone)]
pub struct Element {
    /// Placeholder for VNode data
    _inner: (),
}

impl Element {
    /// Creates a new empty element (placeholder)
    pub fn new() -> Self {
        Self { _inner: () }
    }
}

impl Default for Element {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Element").finish()
    }
}

/// Trait for components that can be rendered
pub trait Component {
    /// Renders the component to an Element
    fn render(&self) -> Element;
}
