//! Memo Hook - Memoize expensive computations
//!
//! This module provides React's `useMemo` functionality for Rust TUI applications.
//! It memoizes the result of an expensive computation and only recalculates when
//! dependencies change. This is crucial for:
//! - Avoiding expensive recalculations on every render
//! - Optimizing performance in complex components
//! - Maintaining referential equality for derived data
//!
//! # Implementation
//!
//! This follows the exact React pattern:
//! ```javascript
//! const memoizedValue = useMemo(() => computeExpensiveValue(a, b), [a, b])
//! ```

use crate::effect::EffectDependencies;
use crate::hook_context::with_hook_context;

#[cfg(test)]
mod tests;

/// Internal state for a memoized value
struct MemoState<T> {
    /// The cached value
    value: Option<T>,
    /// Previous dependencies (as trait object)
    prev_deps: Option<Box<dyn std::any::Any>>,
    /// Version counter to track memo recomputations (useful for debugging and optimization)
    version: usize,
}

impl<T> MemoState<T> {
    fn new() -> Self {
        Self {
            value: None,
            prev_deps: None,
            version: 0,
        }
    }
}

/// Memoize an expensive computation
///
/// This hook caches the result of a computation and only recalculates when
/// dependencies change. Similar to React's `useMemo`.
///
/// # Arguments
///
/// * `factory` - A function that computes the value
/// * `deps` - Dependencies that trigger recomputation when changed
///
/// # Examples
///
/// ## Basic Usage
/// ```rust,ignore
/// use reratui_hooks::memo::use_memo;
/// use reratui_hooks::state::use_state;
///
/// let (count, _) = use_state(|| 0);
///
/// // Expensive computation only runs when count changes
/// let doubled = use_memo(|| {
///     println!("Computing...");
///     count.get() * 2
/// }, count.get());
///
/// assert_eq!(doubled, 0);
/// ```
///
/// ## With Multiple Dependencies
/// ```rust,ignore
/// use reratui_hooks::memo::use_memo;
/// use reratui_hooks::state::use_state;
///
/// let (width, _) = use_state(|| 10);
/// let (height, _) = use_state(|| 20);
///
/// let area = use_memo(|| {
///     width.get() * height.get()
/// }, (width.get(), height.get()));
///
/// assert_eq!(area, 200);
/// ```
pub fn use_memo<T, F, Deps>(factory: F, deps: impl Into<Option<Deps>>) -> T
where
    T: Clone + 'static,
    F: FnOnce() -> T,
    Deps: EffectDependencies + Clone + PartialEq + 'static,
{
    let deps = deps.into();

    with_hook_context(|ctx| {
        let index = ctx.next_hook_index();

        // Get or initialize the memo state
        let state_ref = ctx.get_or_init_state(index, MemoState::<T>::new);

        let mut state = state_ref.borrow_mut();

        // Determine if we need to recompute
        let should_compute = match &deps {
            None => {
                // No dependencies - compute only once
                state.value.is_none()
            }
            Some(current_deps) => {
                // Check if dependencies have changed
                match &state.prev_deps {
                    None => {
                        // First run - always compute
                        true
                    }
                    Some(prev_deps) => {
                        // Compare dependencies
                        if let Some(prev) = prev_deps.downcast_ref::<Deps>() {
                            !current_deps.deps_eq(prev)
                        } else {
                            // Type mismatch - recompute
                            true
                        }
                    }
                }
            }
        };

        if should_compute {
            // Compute new value
            let new_value = factory();
            state.value = Some(new_value);

            // Increment version counter
            state.version += 1;

            // Store new dependencies
            if let Some(current_deps) = deps {
                state.prev_deps = Some(Box::new(current_deps));
            }
        }

        // Return the cached value
        state
            .value
            .clone()
            .expect("Memoized value should be initialized")
    })
}

/// Convenience function for use_memo without dependencies
///
/// This computes the value once and caches it forever (until component unmounts).
///
/// # Examples
///
/// ```rust,ignore
/// use reratui_hooks::memo::use_memo_once;
///
/// // Computed only once
/// let expensive_value = use_memo_once(|| {
///     // Expensive computation
///     42
/// });
/// ```
pub fn use_memo_once<T, F>(factory: F) -> T
where
    T: Clone + 'static,
    F: FnOnce() -> T,
{
    use_memo::<T, F, ()>(factory, None)
}
