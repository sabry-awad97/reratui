use crate::hook_context::{HookContext, clear_hook_context, set_hook_context};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Global test mutex to ensure test isolation
///
/// This mutex serializes all tests that use global state (like event handlers)
/// to prevent race conditions and ensure deterministic test results.
///
/// Use `TEST_MUTEX.lock()` at the start of any test that modifies global state.
pub static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

// Thread-local registry to track component contexts by ID for testing
thread_local! {
    static COMPONENT_CONTEXTS: RefCell<HashMap<&'static str, Rc<HookContext>>> =
        RefCell::new(HashMap::new());
}

/// Professional component context manager for testing with component ID lifecycle
///
/// This function provides a realistic testing environment that:
/// - Maintains separate hook contexts per component ID
/// - When called with the same ID, it represents the next render of that component
/// - Automatically resets hook counter for re-renders (same ID)
/// - Creates new context only for new component IDs
/// - Properly cleans up contexts when test completes
/// - Follows real component lifecycle patterns
///
/// # Usage
/// ```rust,no_run
/// # use reratui_hooks::test_utils::with_component_id;
/// # use reratui_hooks::state::use_state;
/// with_component_id("MyComponent", |_context| {
///     // First render - context is fresh
///     let (count, set_count) = use_state(0);
///     assert_eq!(count.get(), 0);
///     set_count.set(42);
/// });
/// ```
pub fn with_component_id<F, R>(component_id: &'static str, test_fn: F) -> R
where
    F: FnOnce(&Rc<HookContext>) -> R,
{
    let context = COMPONENT_CONTEXTS.with(|contexts| {
        let mut contexts = contexts.borrow_mut();

        if let Some(existing_context) = contexts.get(component_id) {
            // Same component ID - this is a re-render, reset hook counter
            existing_context.reset_hook_index();
            existing_context.clone()
        } else {
            // New component ID - create fresh context
            let new_context = Rc::new(HookContext::new());
            contexts.insert(component_id, new_context.clone());
            new_context
        }
    });

    set_hook_context(context.clone());
    let result = test_fn(&context);
    clear_hook_context();

    result
}

/// Async version of component context manager for async hook testing
///
/// This function provides the same component lifecycle simulation as `with_component_id`
/// but supports async test functions for testing async hooks like `use_future`.
///
/// # Usage
/// ```rust,no_run
/// # use reratui_hooks::test_utils::with_async_component_id;
/// # use reratui_hooks::state::use_state;
/// # async fn example() {
/// with_async_component_id("MyAsyncComponent", async |_context| {
///     // Async hook testing
///     let (count, set_count) = use_state(0);
///     assert_eq!(count.get(), 0);
///     set_count.set(42);
/// }).await;
/// # }
/// ```
pub async fn with_async_component_id<F, Fut, R>(component_id: &'static str, test_fn: F) -> R
where
    F: FnOnce(&Rc<HookContext>) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let context = COMPONENT_CONTEXTS.with(|contexts| {
        let mut contexts = contexts.borrow_mut();

        if let Some(existing_context) = contexts.get(component_id) {
            // Same component ID - this is a re-render, reset hook counter
            existing_context.reset_hook_index();
            existing_context.clone()
        } else {
            // New component ID - create fresh context
            let new_context = Rc::new(HookContext::new());
            contexts.insert(component_id, new_context.clone());
            new_context
        }
    });

    set_hook_context(context.clone());
    let result = test_fn(&context).await;
    clear_hook_context();

    result
}

/// Cleanup function to clear all component contexts (call at end of test suite)
pub fn cleanup_component_contexts() {
    COMPONENT_CONTEXTS.with(|contexts| {
        contexts.borrow_mut().clear();
    });
}

/// Professional test isolation wrapper that automatically handles cleanup
///
/// This function provides complete test isolation by:
/// - Ensuring clean state at the start of each test
/// - Automatically cleaning up component contexts after test completion
/// - Providing proper error handling and cleanup even if test panics
/// - Following professional testing patterns for resource management
///
/// # Usage
/// ```rust,no_run
/// # use pulse_core::hooks::test_utils::{with_test_isolate, with_component_id};
/// # use
/// fn my_test() {
///     with_test_isolate(|| {
///         // Test code here - no need to manually call cleanup
///         with_component_id("MyComponent", |_| {
///             let (count, set_count) = use_state(0);
///             assert_eq!(count.get(), 0);
///             set_count.set(42);
///         });
///
///         with_component_id("MyComponent", |_| {
///             // Next render - state persists automatically
///             let (count, _) = use_state(0);
///             assert_eq!(count.get(), 42);
///         });
///     });
///     // cleanup_component_contexts() called automatically
/// }
/// ```
pub fn with_test_isolate<F, R>(test_fn: F) -> R
where
    F: FnOnce() -> R,
{
    // Ensure clean state at start of test
    cleanup_component_contexts();

    // Use a guard to ensure cleanup happens even if test panics
    struct CleanupGuard;
    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            cleanup_component_contexts();
        }
    }

    let _guard = CleanupGuard;

    // Run the test
    test_fn()

    // Cleanup happens automatically when _guard is dropped
}

/// Async version of test isolation wrapper for tokio tests
///
/// This function provides complete test isolation for async tests by:
/// - Ensuring clean state at the start of each test
/// - Automatically cleaning up component contexts after test completion
/// - Providing proper error handling and cleanup even if test panics
/// - Supporting async test functions with tokio::test
///
/// # Usage
/// ```rust,no_run
///
/// #[tokio::test]
/// async fn my_async_test() {
///     with_async_test_isolate(async {
///         // Async test code here - no need to manually call cleanup
///         with_component_id("MyComponent", |_| {
///             let future = use_future(|| async { computation().await });
///             // Test assertions...
///         });
///     }).await;
///     // cleanup_component_contexts() called automatically
/// }
/// ```
pub async fn with_async_test_isolate<F, Fut, R>(test_fn: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    // Ensure clean state at start of test
    cleanup_component_contexts();

    // Use a guard to ensure cleanup happens even if test panics
    struct AsyncCleanupGuard;
    impl Drop for AsyncCleanupGuard {
        fn drop(&mut self) {
            cleanup_component_contexts();
        }
    }

    let _guard = AsyncCleanupGuard;

    // Run the async test
    test_fn().await

    // Cleanup happens automatically when _guard is dropped
}

/// Simple test context helper for basic hook testing
///
/// This is a simpler alternative to `with_component_id` for tests that don't need
/// component lifecycle simulation. Creates a fresh context for each call.
///
/// # Usage
/// ```rust,no_run
/// # use reratui_hooks::test_utils::with_hook_context;
/// # use reratui_hooks::state::use_state;
/// fn simple_test() {
///     with_hook_context(|_context| {
///         let (state, set_state) = use_state(42);
///         assert_eq!(state.get(), 42);
///     });
/// }
/// ```
pub fn with_hook_context<F, R>(test_fn: F) -> R
where
    F: FnOnce(&Rc<HookContext>) -> R,
{
    let context = Rc::new(HookContext::new());
    set_hook_context(context.clone());

    let result = test_fn(&context);

    clear_hook_context();
    result
}

/// Async version of simple hook context helper for async hook testing
///
/// This is an async alternative to `with_hook_context` for tests that don't need
/// component lifecycle simulation but do need to test async hooks.
///
/// # Usage
/// ```rust,no_run
/// # use reratui_hooks::test_utils::with_async_hook_context;
/// # use reratui_hooks::state::use_state;
/// # async fn example() {
/// with_async_hook_context(async |_context| {
///     let (count, set_count) = use_state(0);
///     assert_eq!(count.get(), 0);
///     set_count.set(42);
/// }).await;
/// # }
/// ```
pub async fn with_async_hook_context<F, Fut, R>(test_fn: F) -> R
where
    F: FnOnce(&Rc<HookContext>) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let context = Rc::new(HookContext::new());
    set_hook_context(context.clone());

    let result = test_fn(&context).await;

    clear_hook_context();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hook_context::get_hook_context, state::use_state};

    #[test]
    fn test_with_hook_context_basic() {
        // Test that with_hook_context provides a working hook context
        let result = with_hook_context(|context| {
            // Verify context is available
            assert_eq!(context.current_hook_index(), 0);

            // Test that hooks work within the context
            let (state, setter) = use_state(|| 42);
            assert_eq!(state.get(), 42);

            setter.set(100);
            assert_eq!(state.get(), 100);

            "test_result"
        });

        assert_eq!(result, "test_result");

        // Verify context is cleaned up after
        assert!(get_hook_context().is_none());
    }

    #[test]
    fn test_with_hook_context_isolation() {
        // Test that each call to with_hook_context is isolated
        with_hook_context(|_| {
            let (state, setter) = use_state(|| 10);
            setter.set(20);
            assert_eq!(state.get(), 20);
        });

        with_hook_context(|_| {
            // Should be a fresh context, not affected by previous call
            let (state, _) = use_state(|| 10);
            assert_eq!(state.get(), 10); // Should be initial value, not 20
        });
    }

    #[test]
    fn test_with_component_id_basic() {
        // Test basic component ID functionality
        let result = with_component_id("TestComponent", |context| {
            assert_eq!(context.current_hook_index(), 0);

            let (state, setter) = use_state(|| 5);
            assert_eq!(state.get(), 5);
            setter.set(15);

            "component_result"
        });

        assert_eq!(result, "component_result");
    }

    #[test]
    fn test_with_component_id_persistence() {
        // Test that state persists across renders of the same component
        with_component_id("PersistentComponent", |_| {
            let (state, setter) = use_state(|| 0);
            assert_eq!(state.get(), 0);
            setter.set(42);
        });

        // Second "render" of the same component - state should persist
        with_component_id("PersistentComponent", |_| {
            let (state, setter) = use_state(|| 0);
            assert_eq!(state.get(), 42); // Should have persisted value
            setter.set(84);
        });

        // Third "render" - should have the updated value
        with_component_id("PersistentComponent", |_| {
            let (state, _) = use_state(|| 0);
            assert_eq!(state.get(), 84);
        });
    }

    #[test]
    fn test_with_component_id_hook_counter_reset() {
        // Test that hook counter resets between renders
        with_component_id("CounterResetComponent", |context| {
            let (state1, _) = use_state(|| 1);
            let (state2, _) = use_state(|| 2);
            assert_eq!(state1.get(), 1);
            assert_eq!(state2.get(), 2);
            // Hook counter should be at 2 now
            assert_eq!(context.current_hook_index(), 2);
        });

        // Second render - hook counter should reset to 0
        with_component_id("CounterResetComponent", |context| {
            // Counter should be reset
            assert_eq!(context.current_hook_index(), 0);

            let (state1, _) = use_state(|| 1);
            let (state2, _) = use_state(|| 2);
            // Values should persist from previous render
            assert_eq!(state1.get(), 1);
            assert_eq!(state2.get(), 2);
        });
    }

    #[test]
    fn test_with_component_id_different_components() {
        // Test that different component IDs have separate contexts
        with_component_id("ComponentA", |_| {
            let (state, setter) = use_state(|| 100);
            setter.set(200);
            assert_eq!(state.get(), 200);
        });

        with_component_id("ComponentB", |_| {
            let (state, setter) = use_state(|| 300);
            setter.set(400);
            assert_eq!(state.get(), 400);
        });

        // Verify they maintain separate state
        with_component_id("ComponentA", |_| {
            let (state, _) = use_state(|| 100);
            assert_eq!(state.get(), 200); // Should have ComponentA's value
        });

        with_component_id("ComponentB", |_| {
            let (state, _) = use_state(|| 300);
            assert_eq!(state.get(), 400); // Should have ComponentB's value
        });
    }

    #[test]
    fn test_cleanup_component_contexts() {
        // Set up some component contexts
        with_component_id("CleanupTest1", |_| {
            let (_, setter) = use_state(|| 1);
            setter.set(10);
        });

        with_component_id("CleanupTest2", |_| {
            let (_, setter) = use_state(|| 2);
            setter.set(20);
        });

        // Verify contexts exist by checking state persistence
        with_component_id("CleanupTest1", |_| {
            let (state, _) = use_state(|| 1);
            assert_eq!(state.get(), 10);
        });

        // Clean up all contexts
        cleanup_component_contexts();

        // After cleanup, contexts should be fresh
        with_component_id("CleanupTest1", |_| {
            let (state, _) = use_state(|| 1);
            assert_eq!(state.get(), 1); // Should be initial value again
        });
    }

    #[test]
    fn test_with_test_isolate_cleanup() {
        // Set up some state outside of test isolate
        with_component_id("IsolateTest", |_| {
            let (_, setter) = use_state(|| 50);
            setter.set(60);
        });

        // Verify state exists
        with_component_id("IsolateTest", |_| {
            let (state, _) = use_state(|| 50);
            assert_eq!(state.get(), 60);
        });

        // Run test in isolation
        let result = with_test_isolate(|| {
            // Inside isolation, should have clean state
            with_component_id("IsolateTest", |_| {
                let (state, setter) = use_state(|| 50);
                assert_eq!(state.get(), 50); // Should be initial value
                setter.set(70);
            });

            "isolated_result"
        });

        assert_eq!(result, "isolated_result");

        // After isolation, state should be cleaned up
        with_component_id("IsolateTest", |_| {
            let (state, _) = use_state(|| 50);
            assert_eq!(state.get(), 50); // Should be initial value again
        });
    }

    #[test]
    fn test_with_test_isolate_panic_cleanup() {
        // Set up some state
        with_component_id("PanicTest", |_| {
            let (_, setter) = use_state(|| 80);
            setter.set(90);
        });

        // Test that cleanup happens even if test panics
        let panic_result = std::panic::catch_unwind(|| {
            with_test_isolate(|| {
                with_component_id("PanicTest", |_| {
                    let (_, setter) = use_state(|| 80);
                    setter.set(95);
                });

                panic!("Test panic");
            })
        });

        assert!(panic_result.is_err());

        // Cleanup should still have happened
        with_component_id("PanicTest", |_| {
            let (state, _) = use_state(|| 80);
            assert_eq!(state.get(), 80); // Should be initial value
        });
    }

    #[test]
    fn test_multiple_hooks_in_component() {
        // Test multiple hooks in the same component
        with_component_id("MultiHookComponent", |_| {
            let (count, set_count) = use_state(|| 0);
            let (name, set_name) = use_state(|| "initial".to_string());
            let (flag, set_flag) = use_state(|| false);

            assert_eq!(count.get(), 0);
            assert_eq!(name.get(), "initial");
            assert!(!flag.get());

            set_count.set(42);
            set_name.set("updated".to_string());
            set_flag.set(true);
        });

        // Second render - all hooks should maintain their state
        with_component_id("MultiHookComponent", |_| {
            let (count, _) = use_state(|| 0);
            let (name, _) = use_state(|| "initial".to_string());
            let (flag, _) = use_state(|| false);

            assert_eq!(count.get(), 42);
            assert_eq!(name.get(), "updated");
            assert!(flag.get());
        });
    }

    #[tokio::test]
    async fn test_with_async_hook_context() {
        let result = with_async_hook_context(|context| {
            let context = context.clone();
            async move {
                // Verify context is available
                assert_eq!(context.current_hook_index(), 0);

                // Test that hooks work within async context
                let (state, setter) = use_state(|| 123);
                assert_eq!(state.get(), 123);

                setter.set(456);
                assert_eq!(state.get(), 456);

                // Simulate some async work
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

                "async_result"
            }
        })
        .await;

        assert_eq!(result, "async_result");

        // Verify context is cleaned up after
        assert!(get_hook_context().is_none());
    }

    #[tokio::test]
    async fn test_with_async_component_id() {
        // Test async component functionality
        let result = with_async_component_id("AsyncComponent", |context| {
            let context = context.clone();
            async move {
                assert_eq!(context.current_hook_index(), 0);

                let (state, setter) = use_state(|| 777);
                assert_eq!(state.get(), 777);
                setter.set(888);

                // Simulate async work
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

                "async_component_result"
            }
        })
        .await;

        assert_eq!(result, "async_component_result");

        // Test state persistence in async context
        with_async_component_id("AsyncComponent", |_| {
            async move {
                let (state, _) = use_state(|| 777);
                assert_eq!(state.get(), 888); // Should have persisted value
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_with_async_test_isolate() {
        // Set up some state
        with_component_id("AsyncIsolateTest", |_| {
            let (_, setter) = use_state(|| 999);
            setter.set(1000);
        });

        let result = with_async_test_isolate(|| async {
            // Should have clean state in isolation
            with_component_id("AsyncIsolateTest", |_| {
                let (state, setter) = use_state(|| 999);
                assert_eq!(state.get(), 999);
                setter.set(1001);
            });

            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

            "async_isolate_result"
        })
        .await;

        assert_eq!(result, "async_isolate_result");

        // State should be cleaned up after isolation
        with_component_id("AsyncIsolateTest", |_| {
            let (state, _) = use_state(|| 999);
            assert_eq!(state.get(), 999);
        });
    }
}
