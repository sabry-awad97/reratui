//! Context hooks with proper lifecycle management.
//!
//! This module provides React-like context hooks that allow components to share
//! values without explicitly passing props through every level of the tree.
//!
//! # Example
//!
//! ```rust,ignore
//! use reratui_fiber::hooks::{use_context_provider_v2, use_context_v2};
//!
//! // Define a theme context
//! #[derive(Clone)]
//! struct Theme {
//!     primary: Color,
//!     background: Color,
//! }
//!
//! // Provider component
//! #[component]
//! fn ThemeProvider(children: Element) -> Element {
//!     let theme = use_context_provider_v2(|| Theme {
//!         primary: Color::Cyan,
//!         background: Color::Black,
//!     });
//!     
//!     rsx! { {children} }
//! }
//!
//! // Consumer component
//! #[component]
//! fn ThemedButton(label: &str) -> Element {
//!     let theme = use_context_v2::<Theme>();
//!     
//!     rsx! {
//!         <Block style={Style::default().fg(theme.primary)}>
//!             {label}
//!         </Block>
//!     }
//! }
//! ```

use std::any::TypeId;

use crate::context_stack::push_context;
use crate::fiber_tree::with_current_fiber;

/// Provide a context value to all descendants.
///
/// The value is created using the provided initializer function and made available
/// to all descendant components via `use_context_v2`. The value is automatically
/// cleaned up when the provider fiber unmounts.
///
/// # Type Parameters
///
/// * `T` - The context value type. Must be `Clone + Send + Sync + 'static`.
/// * `F` - The initializer function type.
///
/// # Arguments
///
/// * `create_value` - A function that creates the initial context value.
///   Only called on the first render.
///
/// # Returns
///
/// The context value (cloned).
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Clone)]
/// struct AppConfig {
///     api_url: String,
///     debug_mode: bool,
/// }
///
/// #[component]
/// fn App() -> Element {
///     // Provide config to all descendants
///     let config = use_context_provider_v2(|| AppConfig {
///         api_url: "https://api.example.com".to_string(),
///         debug_mode: cfg!(debug_assertions),
///     });
///     
///     rsx! { <MainContent /> }
/// }
/// ```
///
/// # Panics
///
/// Panics if called outside of a component render context (no current fiber).
pub fn use_context_provider_v2<T, F>(create_value: F) -> T
where
    T: Clone + Send + Sync + 'static,
    F: FnOnce() -> T,
{
    with_current_fiber(|fiber| {
        let hook_index = fiber.next_hook_index();
        let type_id = TypeId::of::<T>();
        let fiber_id = fiber.id;

        // Check if we already have a value for this hook
        if let Some(existing) = fiber.get_hook::<T>(hook_index) {
            return existing;
        }

        // Create the new value
        let value = create_value();

        // Store in fiber's hook state
        fiber.set_hook(hook_index, value.clone());

        // Push to context stack with fiber ownership
        push_context(fiber_id, value.clone());

        // Track that this fiber provides this context type
        if !fiber.provided_contexts.contains(&type_id) {
            fiber.provided_contexts.push(type_id);
        }

        value
    })
    .expect("use_context_provider_v2 must be called within a component render context")
}

/// Consume a context value from the nearest ancestor provider.
///
/// Returns the value from the nearest ancestor that called `use_context_provider_v2`
/// with the same type `T`.
///
/// # Type Parameters
///
/// * `T` - The context value type. Must be `Clone + Send + Sync + 'static`.
///
/// # Returns
///
/// The context value from the nearest provider.
///
/// # Example
///
/// ```rust,ignore
/// #[component]
/// fn UserProfile() -> Element {
///     // Get the user context from an ancestor provider
///     let user = use_context_v2::<User>();
///     
///     rsx! {
///         <Block>
///             <Text text={format!("Hello, {}!", user.name)} />
///         </Block>
///     }
/// }
/// ```
///
/// # Panics
///
/// Panics if no provider exists for the context type `T`.
/// Use `try_use_context_v2` if you want to handle missing providers gracefully.
pub fn use_context_v2<T>() -> T
where
    T: Clone + Send + Sync + 'static,
{
    try_use_context_v2::<T>().unwrap_or_else(|| {
        panic!(
            "use_context_v2: No provider found for context type `{}`. \
             Make sure a parent component calls use_context_provider_v2 with this type.",
            std::any::type_name::<T>()
        )
    })
}

/// Try to consume a context value, returning None if no provider exists.
///
/// This is a non-panicking version of `use_context_v2` that returns `None`
/// if no provider exists for the context type.
///
/// # Type Parameters
///
/// * `T` - The context value type. Must be `Clone + Send + Sync + 'static`.
///
/// # Returns
///
/// `Some(value)` if a provider exists, `None` otherwise.
///
/// # Example
///
/// ```rust,ignore
/// #[component]
/// fn OptionalThemeConsumer() -> Element {
///     // Works with or without a theme provider
///     let style = match try_use_context_v2::<Theme>() {
///         Some(theme) => Style::default().fg(theme.primary),
///         None => Style::default().fg(Color::White),
///     };
///     
///     rsx! {
///         <Block style={style}>
///             {"Content"}
///         </Block>
///     }
/// }
/// ```
pub fn try_use_context_v2<T>() -> Option<T>
where
    T: Clone + Send + Sync + 'static,
{
    crate::context_stack::get_context::<T>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_stack::clear_context_stack;
    use crate::fiber::FiberId;
    use crate::fiber_tree::{FiberTree, clear_fiber_tree, set_fiber_tree};

    fn setup_test_fiber() -> FiberId {
        let mut tree = FiberTree::new();
        let fiber_id = tree.mount(None, None);
        tree.begin_render(fiber_id);
        set_fiber_tree(tree);
        fiber_id
    }

    fn cleanup_test() {
        clear_fiber_tree();
        clear_context_stack();
    }

    #[test]
    fn test_use_context_provider_v2_creates_value() {
        let _fiber_id = setup_test_fiber();

        let value = use_context_provider_v2(|| 42i32);
        assert_eq!(value, 42);

        cleanup_test();
    }

    #[test]
    fn test_use_context_provider_v2_returns_same_value_on_rerender() {
        let fiber_id = setup_test_fiber();

        // First render
        let value1 = use_context_provider_v2(|| 100i32);
        assert_eq!(value1, 100);

        // Simulate re-render
        crate::fiber_tree::with_fiber_tree_mut(|tree| {
            tree.end_render();
            tree.begin_render(fiber_id);
        });

        // Second render - should return same value, not call initializer again
        let value2 = use_context_provider_v2(|| 999i32);
        assert_eq!(value2, 100); // Still 100, not 999

        cleanup_test();
    }

    #[test]
    fn test_use_context_provider_v2_pushes_to_context_stack() {
        let _fiber_id = setup_test_fiber();

        use_context_provider_v2(|| "test-value".to_string());

        // Value should be available in context stack
        let value = crate::context_stack::get_context::<String>();
        assert_eq!(value, Some("test-value".to_string()));

        cleanup_test();
    }

    #[test]
    fn test_use_context_provider_v2_tracks_provided_contexts() {
        let fiber_id = setup_test_fiber();

        use_context_provider_v2(|| 42i32);
        use_context_provider_v2(|| "hello".to_string());

        // Check that fiber tracks provided context types
        crate::fiber_tree::with_fiber_tree_mut(|tree| {
            let fiber = tree.get(fiber_id).unwrap();
            assert!(fiber.provided_contexts.contains(&TypeId::of::<i32>()));
            assert!(fiber.provided_contexts.contains(&TypeId::of::<String>()));
        });

        cleanup_test();
    }

    #[test]
    fn test_try_use_context_v2_returns_value() {
        let _fiber_id = setup_test_fiber();

        use_context_provider_v2(|| 42i32);

        let value = try_use_context_v2::<i32>();
        assert_eq!(value, Some(42));

        cleanup_test();
    }

    #[test]
    fn test_try_use_context_v2_returns_none_without_provider() {
        cleanup_test(); // Ensure clean state

        let value = try_use_context_v2::<i32>();
        assert_eq!(value, None);
    }

    #[test]
    fn test_use_context_v2_returns_value() {
        let _fiber_id = setup_test_fiber();

        use_context_provider_v2(|| "context-value".to_string());

        let value = use_context_v2::<String>();
        assert_eq!(value, "context-value");

        cleanup_test();
    }

    #[test]
    #[should_panic(expected = "No provider found for context type")]
    fn test_use_context_v2_panics_without_provider() {
        cleanup_test(); // Ensure clean state

        // This should panic because there's no provider
        let _ = use_context_v2::<i32>();
    }

    #[test]
    fn test_nested_providers_shadow() {
        // Setup outer fiber
        let mut tree = FiberTree::new();
        let outer_fiber = tree.mount(None, None);
        let inner_fiber = tree.mount(Some(outer_fiber), None);
        set_fiber_tree(tree);

        // Outer provider
        crate::fiber_tree::with_fiber_tree_mut(|tree| {
            tree.begin_render(outer_fiber);
        });
        use_context_provider_v2(|| "outer".to_string());
        crate::fiber_tree::with_fiber_tree_mut(|tree| {
            tree.end_render();
        });

        // Inner provider shadows outer
        crate::fiber_tree::with_fiber_tree_mut(|tree| {
            tree.begin_render(inner_fiber);
        });
        use_context_provider_v2(|| "inner".to_string());

        // Should get inner value
        let value = use_context_v2::<String>();
        assert_eq!(value, "inner");

        cleanup_test();
    }

    #[test]
    fn test_multiple_context_types() {
        let _fiber_id = setup_test_fiber();

        use_context_provider_v2(|| 42i32);
        use_context_provider_v2(|| "hello".to_string());
        use_context_provider_v2(|| true);

        assert_eq!(use_context_v2::<i32>(), 42);
        assert_eq!(use_context_v2::<String>(), "hello");
        assert!(use_context_v2::<bool>());

        cleanup_test();
    }

    #[test]
    fn test_context_with_custom_type() {
        #[derive(Clone, Debug, PartialEq)]
        struct Theme {
            name: String,
            dark_mode: bool,
        }

        let _fiber_id = setup_test_fiber();

        let provided_theme = use_context_provider_v2(|| Theme {
            name: "default".to_string(),
            dark_mode: true,
        });

        assert_eq!(provided_theme.name, "default");
        assert!(provided_theme.dark_mode);

        let consumed_theme = use_context_v2::<Theme>();
        assert_eq!(consumed_theme, provided_theme);

        cleanup_test();
    }

    #[test]
    #[should_panic(
        expected = "use_context_provider_v2 must be called within a component render context"
    )]
    fn test_use_context_provider_v2_panics_outside_render() {
        cleanup_test();

        // This should panic because there's no current fiber
        let _ = use_context_provider_v2(|| 42i32);
    }
}
