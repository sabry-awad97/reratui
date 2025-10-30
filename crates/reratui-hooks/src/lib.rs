//! Reratui Hooks - React-like hooks for state management
//!
//! This crate provides hooks for managing state and side effects in Reratui components.

use std::cell::RefCell;
use std::rc::Rc;

/// State hook that returns a value and a setter function
///
/// This is a placeholder implementation that provides basic state management.
/// In a full implementation, this would integrate with the component lifecycle
/// and trigger re-renders on state changes.
///
/// # Example
/// ```ignore
/// let (count, set_count) = use_state(|| 0);
/// set_count(count + 1);
/// ```
pub fn use_state<T, F>(init: F) -> (T, impl Fn(T))
where
    T: Clone + 'static,
    F: FnOnce() -> T,
{
    // Initialize state with the provided function
    let state = Rc::new(RefCell::new(init()));
    let state_clone = state.clone();

    // Return current value and setter
    let current_value = state.borrow().clone();
    let setter = move |new_value: T| {
        *state_clone.borrow_mut() = new_value;
        // TODO: Trigger component re-render
    };

    (current_value, setter)
}

/// Effect hook for side effects (placeholder)
///
/// This will eventually handle side effects and cleanup.
pub fn use_effect<F, D>(_effect: F, _deps: D)
where
    F: FnOnce() + 'static,
    D: 'static,
{
    // TODO: Implement effect scheduling
}

/// Ref hook for mutable references (placeholder)
pub fn use_ref<T>(initial: T) -> Rc<RefCell<T>>
where
    T: 'static,
{
    Rc::new(RefCell::new(initial))
}

/// Memo hook for memoization (placeholder)
pub fn use_memo<T, F, D>(compute: F, _deps: D) -> T
where
    T: Clone + 'static,
    F: FnOnce() -> T,
    D: 'static,
{
    // TODO: Implement proper memoization with dependency tracking
    compute()
}

/// Callback hook for memoized callbacks (placeholder)
pub fn use_callback<F, D>(callback: F, _deps: D) -> F
where
    F: Clone + 'static,
    D: 'static,
{
    // TODO: Implement proper callback memoization
    callback
}
