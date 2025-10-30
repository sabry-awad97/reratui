//! Effect Event Hook - Stable callbacks that always call the latest handler
//!
//! This module provides React's experimental `useEffectEvent` functionality for Rust TUI applications.
//! It creates a stable callback reference that doesn't change between renders, but always calls
//! the latest version of the provided handler. This is crucial for:
//! - Event handlers in effects with empty dependency arrays
//! - Callbacks passed to memoized child components
//! - Avoiding unnecessary re-renders while maintaining fresh state access
//! - Breaking the dependency cycle in effects
//!

use crate::callback::{Callback, use_callback};
use crate::effect::use_effect_always;
use crate::ref_hook::use_ref;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// A stable callback handle that always invokes the latest handler
///
/// This is the return type of `use_effect_event`. The callback identity
/// remains stable across renders, but it always calls the most recent
/// version of the handler function.
pub type EffectEvent<IN, OUT> = Callback<IN, OUT>;

/// React-style `useEffectEvent` hook that creates a stable callback with fresh handler
///
/// This hook follows the exact React pattern using existing hooks:
/// 1. Store the handler in a ref (using `use_ref`)
/// 2. Update the ref on every render (using `use_effect_always`)
/// 3. Return a stable callback that calls the current ref value (using `use_callback`)
///
/// # Examples
///
/// ## Basic Usage
/// ```rust,no_run
/// # use reratui_hooks::effect_event::use_effect_event;
/// # use reratui_hooks::state::use_state;
/// # use reratui_hooks::hook_context::{HookContext, set_hook_context};
/// # use std::rc::Rc;
/// # let context = Rc::new(HookContext::new());
/// # set_hook_context(context);
/// let (count, set_count) = use_state(|| 0);
///
/// // This callback has a stable identity but always sees the latest count
/// let log_count = use_effect_event(move |_: ()| {
///     println!("Current count: {}", count.get());
/// });
///
/// log_count.emit(());
/// ```
pub fn use_effect_event<IN, OUT, F>(handler: F) -> EffectEvent<IN, OUT>
where
    F: Fn(IN) -> OUT + Send + Sync + Clone + 'static,
    IN: 'static,
    OUT: 'static,
{
    // Step 1: Store handler in a ref (like useRef(handler))
    let handler_arc: Arc<dyn Fn(IN) -> OUT + Send + Sync> = Arc::new(handler.clone());
    let handler_ref = use_ref(|| handler_arc);

    // Step 2: Update ref on every render (like useLayoutEffect)
    let handler_ref_clone = handler_ref.clone();
    let handler_clone = handler.clone();
    use_effect_always(move || {
        let handler_arc: Arc<dyn Fn(IN) -> OUT + Send + Sync> = Arc::new(handler_clone.clone());
        handler_ref_clone.set(handler_arc);
        || {} // No cleanup
    });

    // Step 3: Return stable callback (like useCallback with empty deps)
    use_callback(
        move |input: IN| {
            let handler = handler_ref.get();
            handler(input)
        },
        (),
    )
}
