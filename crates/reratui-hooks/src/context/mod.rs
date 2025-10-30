//! Beautiful Context Provider API for sharing state between components
//!
//! This module provides a more elegant context API that allows components to share state
//! without having to pass props down through many levels, similar to React's Context API.
//! This implementation is designed to be more ergonomic and beautiful to use.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

use crate::hook_context::with_hook_context;

thread_local! {
    static CONTEXT_PROVIDERS: RefCell<HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>> =
        RefCell::new(HashMap::new());
}

/// Clear all context providers (called when hook context is reset)
pub fn clear_context_providers() {
    CONTEXT_PROVIDERS.with(|providers| {
        providers.borrow_mut().clear();
    });
}

/// Provides a context value for a type
///
/// This function creates a context value that will be available to all components
/// rendered within the current component's render function. It's similar to React's
/// Context.Provider component but as a hook.
///
/// # Type Parameters
///
/// * `T` - The type of the context value
///
/// # Arguments
///
/// * `create_value` - A function that creates the context value
///
/// # Returns
///
/// * The context value
pub fn use_context_provider<T, F>(create_value: F) -> T
where
    T: Clone + Send + Sync + 'static,
    F: FnOnce() -> T,
{
    with_hook_context(|_ctx| {
        let type_id = TypeId::of::<T>();
        let value = create_value();
        let value_clone = value.clone();

        // Store the value in the thread-local provider stack
        CONTEXT_PROVIDERS.with(|providers| {
            let mut providers = providers.borrow_mut();
            let provider_stack = providers.entry(type_id).or_default();
            provider_stack.push(Box::new(value_clone));
        });

        value
    })
}

/// Consumes a context value for a type
///
/// This function retrieves a context value that was provided by a parent component
/// using `use_context_provider`. If no context value is found, it will panic.
///
/// # Type Parameters
///
/// * `T` - The type of the context value
///
/// # Returns
///
/// * The context value
///
pub fn use_context<T>() -> T
where
    T: Clone + Send + Sync + 'static,
{
    with_hook_context(|_ctx| {
        let type_id = TypeId::of::<T>();

        // Try to get the value from the thread-local provider stack
        let value = CONTEXT_PROVIDERS.with(|providers| {
            let providers = providers.borrow();
            if let Some(provider_stack) = providers.get(&type_id)
                && let Some(last_provider) = provider_stack.last()
                && let Some(value) = last_provider.downcast_ref::<T>()
            {
                return Some(value.clone());
            }
            None
        });

        // If found in the thread-local stack, return it
        if let Some(value) = value {
            return value;
        }

        // If not found, panic with a helpful error message
        panic!(
            "Context value for type {} not found. Make sure to call use_context_provider in a parent component.",
            std::any::type_name::<T>()
        );
    })
}
