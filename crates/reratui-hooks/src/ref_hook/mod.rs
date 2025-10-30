//! Ref Hook - Mutable references that persist across renders without triggering re-renders
//!
//! This module provides React's `useRef`-like functionality for Rust TUI applications.
//! Unlike `use_state`, mutations to refs do NOT trigger re-renders, making them perfect for:
//! - Storing mutable values that don't affect rendering
//! - Accessing DOM-like elements (terminal widgets)
//! - Keeping track of previous values
//! - Managing timers, intervals, or other side-effect handles
//! - Caching expensive computations

use parking_lot::RwLock;
use std::sync::Arc;

use crate::hook_context::with_hook_context;

#[cfg(test)]
mod tests;

/// A thread-safe reference container that holds a mutable value
///
/// Unlike `StateContainer`, this does NOT trigger re-renders on mutation.
/// This is the core storage for the `use_ref` hook.
#[derive(Debug)]
pub struct RefContainer<T> {
    /// The current value, protected by RwLock for thread-safe access
    value: RwLock<T>,
}

impl<T> RefContainer<T> {
    /// Create a new ref container with an initial value
    pub fn new<F>(initializer: F) -> Self
    where
        F: FnOnce() -> T,
    {
        Self {
            value: RwLock::new(initializer()),
        }
    }

    /// Get a clone of the current value (thread-safe read)
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.value.read().clone()
    }

    /// Set a new value (thread-safe write)
    ///
    /// Note: This does NOT trigger a re-render
    pub fn set(&self, new_value: T) {
        let mut value = self.value.write();
        *value = new_value;
    }

    /// Update the value using a function (functional update pattern)
    ///
    /// Note: This does NOT trigger a re-render
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        let mut value = self.value.write();
        updater(&mut *value);
    }

    /// Access the value immutably with a closure
    ///
    /// This is useful for reading nested fields without cloning
    pub fn with<F, R>(&self, accessor: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let value = self.value.read();
        accessor(&*value)
    }

    /// Access the value mutably with a closure
    ///
    /// This is useful for complex mutations without cloning
    /// Note: This does NOT trigger a re-render
    pub fn with_mut<F, R>(&self, mutator: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut value = self.value.write();
        mutator(&mut *value)
    }

    /// Replace the value and return the old value
    pub fn replace(&self, new_value: T) -> T {
        let mut value = self.value.write();
        std::mem::replace(&mut *value, new_value)
    }

    /// Take the value, leaving `Default::default()` in its place
    pub fn take(&self) -> T
    where
        T: Default,
    {
        let mut value = self.value.write();
        std::mem::take(&mut *value)
    }
}

/// A handle to a mutable reference that persists across renders
///
/// This mirrors React's `useRef` return value. Unlike `StateHandle`,
/// mutations do NOT trigger re-renders.
#[derive(Debug)]
pub struct RefHandle<T> {
    /// Reference to the shared ref container
    container: Arc<RefContainer<T>>,
}

impl<T> RefHandle<T> {
    /// Create a new ref handle with an initial value
    pub fn new<F>(initializer: F) -> Self
    where
        F: FnOnce() -> T,
    {
        Self {
            container: Arc::new(RefContainer::new(initializer)),
        }
    }

    /// Create a ref handle from an existing container
    pub fn from_container(container: Arc<RefContainer<T>>) -> Self {
        Self { container }
    }

    /// Get a clone of the current value
    ///
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.container.get()
    }

    /// Set a new value (does NOT trigger re-render)
    pub fn set(&self, new_value: T) {
        self.container.set(new_value);
    }

    /// Update the value using a mutable closure (does NOT trigger re-render)
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        self.container.update(updater);
    }

    /// Access the value immutably with a closure
    pub fn with<F, R>(&self, accessor: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.container.with(accessor)
    }

    /// Access the value mutably with a closure (does NOT trigger re-render)
    pub fn with_mut<F, R>(&self, mutator: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.container.with_mut(mutator)
    }

    /// Replace the value and return the old value
    pub fn replace(&self, new_value: T) -> T {
        self.container.replace(new_value)
    }

    /// Take the value, leaving `Default::default()` in its place
    pub fn take(&self) -> T
    where
        T: Default,
    {
        self.container.take()
    }

    /// Get access to the underlying container (for advanced use cases)
    pub fn container(&self) -> &Arc<RefContainer<T>> {
        &self.container
    }
}

impl<T> Clone for RefHandle<T> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
        }
    }
}

/// React-style `useRef` hook that provides mutable references without triggering re-renders
///
/// This hook creates a mutable reference that persists across component re-renders.
/// Unlike `use_state`, mutations to refs do NOT trigger re-renders, making them ideal for:
///
/// - **Storing mutable values**: Track values that change but don't affect rendering
/// - **Previous values**: Keep track of previous state or props
/// - **DOM-like references**: Store references to terminal widgets or areas
/// - **Timers and intervals**: Manage async handles without re-rendering
/// - **Caching**: Store expensive computation results
/// - **Instance variables**: Component-scoped mutable storage
///
/// # Thread Safety
///
/// The returned `RefHandle` is thread-safe and can be safely shared across async tasks.
///
/// ## Comparison with `use_state`
///
/// | Feature | `use_ref` | `use_state` |
/// |---------|-----------|-------------|
/// | Triggers re-render | ❌ No | ✅ Yes |
/// | Mutable access | ✅ Direct | ❌ Via setter |
/// | Use case | Side effects, caching | UI state |
/// | Performance | Faster (no re-render) | Slower (re-renders) |
///
/// # Performance Notes
///
/// - Mutations are O(1) with minimal overhead
/// - No re-render triggered, making it very efficient
/// - Thread-safe with RwLock for concurrent access
/// - Memory usage is minimal with Arc-based sharing
///
/// # Error Handling
///
/// This function will panic if called outside of a component render context.
/// Always ensure `use_ref` is called within a component function.
pub fn use_ref<T, F>(initializer: F) -> RefHandle<T>
where
    T: 'static,
    F: FnOnce() -> T,
{
    with_hook_context(|ctx| {
        let index = ctx.next_hook_index();

        // Get or initialize the ref container for this hook
        let container_ref =
            ctx.get_or_init_state(index, || Arc::new(RefContainer::new(initializer)));

        // Extract the Arc<RefContainer<T>> from Rc<RefCell<Arc<RefContainer<T>>>>
        let container = container_ref.borrow().clone();

        // Create and return the ref handle
        RefHandle::from_container(container)
    })
}
