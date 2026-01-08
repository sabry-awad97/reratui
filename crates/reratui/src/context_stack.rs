//! Context stack with proper lifecycle management.
//!
//! Provides scoped context values that are automatically cleaned up
//! when their provider fiber unmounts. This implements React-like context
//! semantics where nested providers shadow parent values.
//!
//! # Example
//!
//! ```rust,ignore
//! use reratui_fiber::context_stack::{push_context, get_context, pop_context_for_fiber};
//! use reratui_fiber::FiberId;
//!
//! // Provider pushes a value
//! push_context(FiberId(1), "theme-dark".to_string());
//!
//! // Consumer gets the value
//! let theme = get_context::<String>().unwrap();
//! assert_eq!(theme, "theme-dark");
//!
//! // Nested provider shadows the parent
//! push_context(FiberId(2), "theme-light".to_string());
//! let theme = get_context::<String>().unwrap();
//! assert_eq!(theme, "theme-light");
//!
//! // When inner provider unmounts, outer value is restored
//! pop_context_for_fiber(FiberId(2));
//! let theme = get_context::<String>().unwrap();
//! assert_eq!(theme, "theme-dark");
//! ```

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::fiber::FiberId;

thread_local! {
    /// Thread-local context stack
    static CONTEXT_STACK: RefCell<ContextStack> = RefCell::new(ContextStack::new());
}

/// Type alias for the provider stack to reduce complexity
type ProviderStack = Vec<(FiberId, Box<dyn Any + Send + Sync>)>;

/// Context stack with proper lifecycle management.
///
/// Each context type has its own stack of values, where each value is
/// associated with the fiber that provided it. When a fiber unmounts,
/// all its context values are automatically removed.
pub struct ContextStack {
    /// Stack of values per type, with fiber ownership
    providers: HashMap<TypeId, ProviderStack>,
}

impl ContextStack {
    /// Create a new empty context stack
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Push a context value (called during render).
    ///
    /// The value is associated with the given fiber and will be automatically
    /// removed when `pop_for_fiber` is called for that fiber.
    pub fn push<T: Send + Sync + 'static>(&mut self, fiber_id: FiberId, value: T) {
        let type_id = TypeId::of::<T>();
        self.providers
            .entry(type_id)
            .or_default()
            .push((fiber_id, Box::new(value)));
    }

    /// Pop all context values for a fiber (called on unmount).
    ///
    /// This removes all context values that were pushed by the given fiber,
    /// restoring any shadowed values from parent providers.
    pub fn pop_for_fiber(&mut self, fiber_id: FiberId) {
        for stack in self.providers.values_mut() {
            stack.retain(|(id, _)| *id != fiber_id);
        }
    }

    /// Get the nearest context value of type T.
    ///
    /// Returns the most recently pushed value of type T, which corresponds
    /// to the nearest ancestor provider in the component tree.
    pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.providers
            .get(&type_id)?
            .last()
            .and_then(|(_, value)| value.downcast_ref::<T>())
            .cloned()
    }

    /// Check if a context of type T exists.
    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.providers
            .get(&type_id)
            .map(|stack| !stack.is_empty())
            .unwrap_or(false)
    }

    /// Clear all context values.
    pub fn clear(&mut self) {
        self.providers.clear();
    }

    /// Get the number of providers for a given type.
    #[cfg(test)]
    pub fn provider_count<T: 'static>(&self) -> usize {
        let type_id = TypeId::of::<T>();
        self.providers
            .get(&type_id)
            .map(|stack| stack.len())
            .unwrap_or(0)
    }
}

impl Default for ContextStack {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Thread-local API functions
// ============================================================================

/// Push a context value to the thread-local stack.
///
/// The value is associated with the given fiber and will be automatically
/// removed when `pop_context_for_fiber` is called for that fiber.
pub fn push_context<T: Send + Sync + 'static>(fiber_id: FiberId, value: T) {
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().push(fiber_id, value);
    });
}

/// Get the nearest context value from the thread-local stack.
///
/// Returns the most recently pushed value of type T, which corresponds
/// to the nearest ancestor provider in the component tree.
pub fn get_context<T: Clone + Send + Sync + 'static>() -> Option<T> {
    CONTEXT_STACK.with(|stack| stack.borrow().get::<T>())
}

/// Pop all context values for a fiber from the thread-local stack.
///
/// This removes all context values that were pushed by the given fiber,
/// restoring any shadowed values from parent providers.
pub fn pop_context_for_fiber(fiber_id: FiberId) {
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().pop_for_fiber(fiber_id);
    });
}

/// Check if a context of type T exists in the thread-local stack.
pub fn has_context<T: 'static>() -> bool {
    CONTEXT_STACK.with(|stack| stack.borrow().has::<T>())
}

/// Clear all context values from the thread-local stack.
pub fn clear_context_stack() {
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().clear();
    });
}

/// Execute a closure with the thread-local context stack.
pub fn with_context_stack<R, F: FnOnce(&ContextStack) -> R>(f: F) -> R {
    CONTEXT_STACK.with(|stack| f(&stack.borrow()))
}

/// Execute a closure with mutable access to the thread-local context stack.
pub fn with_context_stack_mut<R, F: FnOnce(&mut ContextStack) -> R>(f: F) -> R {
    CONTEXT_STACK.with(|stack| f(&mut stack.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_stack_creation() {
        let stack = ContextStack::new();
        assert!(stack.providers.is_empty());
    }

    #[test]
    fn test_push_and_get() {
        let mut stack = ContextStack::new();
        let fiber_id = FiberId(1);

        stack.push(fiber_id, 42i32);
        assert_eq!(stack.get::<i32>(), Some(42));
    }

    #[test]
    fn test_nested_providers_shadow() {
        let mut stack = ContextStack::new();
        let fiber1 = FiberId(1);
        let fiber2 = FiberId(2);

        stack.push(fiber1, "outer".to_string());
        assert_eq!(stack.get::<String>(), Some("outer".to_string()));

        stack.push(fiber2, "inner".to_string());
        assert_eq!(stack.get::<String>(), Some("inner".to_string()));
    }

    #[test]
    fn test_pop_for_fiber() {
        let mut stack = ContextStack::new();
        let fiber1 = FiberId(1);
        let fiber2 = FiberId(2);

        stack.push(fiber1, "outer".to_string());
        stack.push(fiber2, "inner".to_string());

        stack.pop_for_fiber(fiber2);
        assert_eq!(stack.get::<String>(), Some("outer".to_string()));

        stack.pop_for_fiber(fiber1);
        assert_eq!(stack.get::<String>(), None);
    }

    #[test]
    fn test_multiple_types() {
        let mut stack = ContextStack::new();
        let fiber_id = FiberId(1);

        stack.push(fiber_id, 42i32);
        stack.push(fiber_id, "hello".to_string());

        assert_eq!(stack.get::<i32>(), Some(42));
        assert_eq!(stack.get::<String>(), Some("hello".to_string()));
    }

    #[test]
    fn test_has_context() {
        let mut stack = ContextStack::new();
        let fiber_id = FiberId(1);

        assert!(!stack.has::<i32>());

        stack.push(fiber_id, 42i32);
        assert!(stack.has::<i32>());
        assert!(!stack.has::<String>());
    }

    #[test]
    fn test_get_nonexistent_returns_none() {
        let stack = ContextStack::new();
        assert_eq!(stack.get::<i32>(), None);
    }

    #[test]
    fn test_clear() {
        let mut stack = ContextStack::new();
        let fiber_id = FiberId(1);

        stack.push(fiber_id, 42i32);
        stack.push(fiber_id, "hello".to_string());

        assert!(stack.has::<i32>());
        assert!(stack.has::<String>());

        stack.clear();

        assert!(!stack.has::<i32>());
        assert!(!stack.has::<String>());
    }

    #[test]
    fn test_provider_count() {
        let mut stack = ContextStack::new();
        let fiber1 = FiberId(1);
        let fiber2 = FiberId(2);

        assert_eq!(stack.provider_count::<i32>(), 0);

        stack.push(fiber1, 1i32);
        assert_eq!(stack.provider_count::<i32>(), 1);

        stack.push(fiber2, 2i32);
        assert_eq!(stack.provider_count::<i32>(), 2);

        stack.pop_for_fiber(fiber2);
        assert_eq!(stack.provider_count::<i32>(), 1);
    }

    #[test]
    fn test_thread_local_push_and_get() {
        clear_context_stack();

        let fiber_id = FiberId(1);
        push_context(fiber_id, 42i32);

        assert_eq!(get_context::<i32>(), Some(42));
        assert!(has_context::<i32>());

        clear_context_stack();
    }

    #[test]
    fn test_thread_local_pop_for_fiber() {
        clear_context_stack();

        let fiber1 = FiberId(1);
        let fiber2 = FiberId(2);

        push_context(fiber1, "outer".to_string());
        push_context(fiber2, "inner".to_string());

        assert_eq!(get_context::<String>(), Some("inner".to_string()));

        pop_context_for_fiber(fiber2);
        assert_eq!(get_context::<String>(), Some("outer".to_string()));

        clear_context_stack();
    }

    #[test]
    fn test_with_context_stack() {
        clear_context_stack();

        let fiber_id = FiberId(1);
        push_context(fiber_id, 42i32);

        let has_int = with_context_stack(|stack| stack.has::<i32>());
        assert!(has_int);

        clear_context_stack();
    }

    #[test]
    fn test_with_context_stack_mut() {
        clear_context_stack();

        with_context_stack_mut(|stack| {
            stack.push(FiberId(1), 42i32);
        });

        assert!(has_context::<i32>());

        clear_context_stack();
    }

    #[test]
    fn test_default_impl() {
        let stack: ContextStack = Default::default();
        assert!(!stack.has::<i32>());
    }

    #[test]
    fn test_deeply_nested_providers() {
        let mut stack = ContextStack::new();

        // Simulate a deep component tree
        for i in 1..=5 {
            stack.push(FiberId(i), format!("level-{}", i));
        }

        // Should get the innermost value
        assert_eq!(stack.get::<String>(), Some("level-5".to_string()));

        // Pop from innermost to outermost
        for i in (1..=5).rev() {
            assert_eq!(stack.get::<String>(), Some(format!("level-{}", i)));
            stack.pop_for_fiber(FiberId(i));
        }

        assert_eq!(stack.get::<String>(), None);
    }

    #[test]
    fn test_multiple_contexts_same_fiber() {
        let mut stack = ContextStack::new();
        let fiber_id = FiberId(1);

        // A single fiber can provide multiple context types
        stack.push(fiber_id, 42i32);
        stack.push(fiber_id, "theme".to_string());
        stack.push(fiber_id, true);

        assert_eq!(stack.get::<i32>(), Some(42));
        assert_eq!(stack.get::<String>(), Some("theme".to_string()));
        assert_eq!(stack.get::<bool>(), Some(true));

        // Popping the fiber removes all its contexts
        stack.pop_for_fiber(fiber_id);

        assert_eq!(stack.get::<i32>(), None);
        assert_eq!(stack.get::<String>(), None);
        assert_eq!(stack.get::<bool>(), None);
    }
}
