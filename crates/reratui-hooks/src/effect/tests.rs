//! Comprehensive tests for the hooks system including useState and useEffect
//!
//! This module contains extensive tests covering:
//! - useState functionality and thread safety
//! - useEffect functionality and dependency tracking
//! - Cleanup function management
//! - Integration between hooks
//! - Error handling and edge cases

use crate::state::use_state;
use crate::test_utils::{
    with_async_component_id, with_async_test_isolate, with_component_id, with_hook_context,
    with_test_isolate,
};

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Test basic useEffect functionality
#[test]
fn test_use_effect_basic() {
    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_component_id("BasicEffectComponent", |_context| {
            let effect_ran_clone = effect_ran.clone();

            // Effect with no dependencies - should run on every render
            use_effect::<(), _, Box<dyn FnOnce() + Send>>(
                move || {
                    effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>> // No cleanup
                },
                None,
            );

            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });

        // Second render (same component ID)
        with_component_id("BasicEffectComponent", |_context| {
            let effect_ran_clone2 = effect_ran.clone();
            use_effect::<i32, _, _>(
                move || {
                    effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                None,
            );

            assert_eq!(effect_ran.load(Ordering::Relaxed), 2);
        });
    });
}

/// Test useEffect with empty dependencies (run once)
#[test]
fn test_use_effect_empty_deps() {
    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_component_id("EmptyDepsEffectComponent", |_context| {
            let effect_ran_clone = effect_ran.clone();

            // Effect with empty dependencies - should run only once
            use_effect(
                move || {
                    effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                (),
            );

            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });

        // Second render (same component ID)
        with_component_id("EmptyDepsEffectComponent", |_context| {
            let effect_ran_clone2 = effect_ran.clone();
            use_effect(
                move || {
                    effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                (),
            );

            // Should not run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });
    });
}

/// Test useEffect with changing dependencies
#[test]
fn test_use_effect_changing_deps() {
    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render with dependency value 1
        with_component_id("ChangingDepsEffectComponent", |_context| {
            let effect_ran_clone = effect_ran.clone();

            use_effect(
                move || {
                    effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                1,
            );

            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });

        // Second render with same dependency value
        with_component_id("ChangingDepsEffectComponent", |_context| {
            let effect_ran_clone2 = effect_ran.clone();
            use_effect(
                move || {
                    effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                1,
            );

            // Should not run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });

        // Third render with different dependency value
        with_component_id("ChangingDepsEffectComponent", |_context| {
            let effect_ran_clone3 = effect_ran.clone();
            use_effect(
                move || {
                    effect_ran_clone3.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                2,
            );

            // Should run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 2);
        });
    });
}

/// Test useEffect cleanup functions
#[test]
fn test_use_effect_cleanup() {
    with_test_isolate(|| {
        let cleanup_ran = Arc::new(AtomicUsize::new(0));

        // First render with cleanup
        with_component_id("CleanupEffectComponent", |_context| {
            let cleanup_ran_clone = cleanup_ran.clone();

            use_effect(
                move || {
                    Some(move || {
                        cleanup_ran_clone.fetch_add(1, Ordering::Relaxed);
                    })
                },
                1,
            );

            assert_eq!(cleanup_ran.load(Ordering::Relaxed), 0);
        });

        // Second render with different dependency - should run cleanup
        with_component_id("CleanupEffectComponent", |_context| {
            let cleanup_ran_clone2 = cleanup_ran.clone();
            use_effect(
                move || {
                    Some(move || {
                        cleanup_ran_clone2.fetch_add(1, Ordering::Relaxed);
                    })
                },
                2,
            );

            assert_eq!(cleanup_ran.load(Ordering::Relaxed), 1);
        });
    });
}

/// Test useEffect with tuple dependencies
#[test]
fn test_use_effect_tuple_deps() {
    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render with tuple dependency (1, "hello")
        with_component_id("TupleDepsEffectComponent", |_context| {
            let effect_ran_clone = effect_ran.clone();

            use_effect(
                move || {
                    effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                (1, "hello"),
            );

            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });

        // Second render with same tuple dependency
        with_component_id("TupleDepsEffectComponent", |_context| {
            let effect_ran_clone2 = effect_ran.clone();
            use_effect(
                move || {
                    effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                (1, "hello"),
            );

            // Should not run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });

        // Third render with different first element
        with_component_id("TupleDepsEffectComponent", |_context| {
            let effect_ran_clone3 = effect_ran.clone();
            use_effect(
                move || {
                    effect_ran_clone3.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                (2, "hello"),
            );

            // Should run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 2);
        });

        // Fourth render with different second element
        with_component_id("TupleDepsEffectComponent", |_context| {
            let effect_ran_clone4 = effect_ran.clone();
            use_effect(
                move || {
                    effect_ran_clone4.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                (2, "world"),
            );

            // Should run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 3);
        });
    });
}

/// Test useEffect with complex custom dependencies
#[test]
fn test_use_effect_complex_deps() {
    #[derive(Clone, PartialEq, Debug, Hash)]
    struct CustomDeps {
        id: u32,
        name: String,
        active: bool,
    }

    impl EffectDependencies for CustomDeps {
        fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
            if let Some(other_deps) = other.as_any().downcast_ref::<CustomDeps>() {
                self == other_deps
            } else {
                false
            }
        }

        fn clone_deps(&self) -> Box<dyn EffectDependencies> {
            Box::new(self.clone())
        }

        fn debug_deps(&self) -> String {
            format!("{:?}", self)
        }

        fn deps_hash(&self) -> u64 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish()
        }
    }

    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        let deps1 = CustomDeps {
            id: 1,
            name: "test".to_string(),
            active: true,
        };

        // First render
        with_component_id("ComplexDepsEffectComponent", |_context| {
            let effect_ran_clone = effect_ran.clone();

            use_effect(
                move || {
                    effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                deps1.clone(),
            );

            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });

        // Second render with same deps
        with_component_id("ComplexDepsEffectComponent", |_context| {
            let effect_ran_clone2 = effect_ran.clone();
            use_effect(
                move || {
                    effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                deps1.clone(),
            );

            // Should not run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });

        // Third render with different deps
        with_component_id("ComplexDepsEffectComponent", |_context| {
            let effect_ran_clone3 = effect_ran.clone();
            let deps2 = CustomDeps {
                id: 1,
                name: "test".to_string(),
                active: false, // Changed active flag
            };

            use_effect(
                move || {
                    effect_ran_clone3.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                deps2,
            );

            // Should run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 2);
        });
    });
}

/// Test multiple effects execution order
#[test]
fn test_use_effect_execution_order() {
    with_test_isolate(|| {
        with_hook_context(|_context| {
            let execution_order = Arc::new(Mutex::new(Vec::new()));

            // First effect
            let order_clone1 = execution_order.clone();
            use_effect(
                move || {
                    order_clone1.lock().unwrap().push(1);
                    None::<Box<dyn FnOnce() + Send>>
                },
                (),
            );

            // Second effect
            let order_clone2 = execution_order.clone();
            use_effect(
                move || {
                    order_clone2.lock().unwrap().push(2);
                    None::<Box<dyn FnOnce() + Send>>
                },
                (),
            );

            // Third effect
            let order_clone3 = execution_order.clone();
            use_effect(
                move || {
                    order_clone3.lock().unwrap().push(3);
                    None::<Box<dyn FnOnce() + Send>>
                },
                (),
            );

            // Verify execution order
            let order = execution_order.lock().unwrap();
            assert_eq!(*order, vec![1, 2, 3]);
        });
    });
}

/// Test cleanup execution timing
#[test]
fn test_use_effect_cleanup_timing() {
    with_test_isolate(|| {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));

        // First render
        with_component_id("CleanupTimingEffectComponent", |_context| {
            let events_clone1 = events.clone();
            let events_clone1_cleanup = events.clone();
            use_effect(
                move || {
                    events_clone1.lock().unwrap().push("effect1".to_string());
                    Some(move || {
                        events_clone1_cleanup
                            .lock()
                            .unwrap()
                            .push("cleanup1".to_string());
                    })
                },
                1,
            );

            // Verify first effect ran
            {
                let events_vec = events.lock().unwrap();
                assert_eq!(*events_vec, vec!["effect1"]);
            }
        });

        // Second render with different dependency - should trigger cleanup then new effect
        with_component_id("CleanupTimingEffectComponent", |_context| {
            let events_clone2 = events.clone();
            let events_clone2_cleanup = events.clone();
            use_effect(
                move || {
                    events_clone2.lock().unwrap().push("effect2".to_string());
                    Some(move || {
                        events_clone2_cleanup
                            .lock()
                            .unwrap()
                            .push("cleanup2".to_string());
                    })
                },
                2,
            );

            // Verify cleanup ran before new effect
            {
                let events_vec = events.lock().unwrap();
                assert_eq!(*events_vec, vec!["effect1", "cleanup1", "effect2"]);
            }
        });
    });
}

/// Test multiple effects with independent cleanups
#[test]
fn test_use_effect_multiple_cleanups() {
    with_test_isolate(|| {
        let cleanup1_ran = Arc::new(AtomicUsize::new(0));
        let cleanup2_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_component_id("MultipleCleanupEffectComponent", |_context| {
            // First effect with cleanup
            let cleanup1_clone = cleanup1_ran.clone();
            use_effect(
                move || {
                    Some(move || {
                        cleanup1_clone.fetch_add(1, Ordering::Relaxed);
                    })
                },
                1,
            );

            // Second effect with cleanup
            let cleanup2_clone = cleanup2_ran.clone();
            use_effect(
                move || {
                    Some(move || {
                        cleanup2_clone.fetch_add(1, Ordering::Relaxed);
                    })
                },
                "test",
            );

            assert_eq!(cleanup1_ran.load(Ordering::Relaxed), 0);
            assert_eq!(cleanup2_ran.load(Ordering::Relaxed), 0);
        });

        // Second render - change only first effect's dependency
        with_component_id("MultipleCleanupEffectComponent", |_context| {
            let cleanup1_clone2 = cleanup1_ran.clone();
            use_effect(
                move || {
                    Some(move || {
                        cleanup1_clone2.fetch_add(1, Ordering::Relaxed);
                    })
                },
                2, // Changed
            );

            // Second effect with same dependency
            let cleanup2_clone2 = cleanup2_ran.clone();
            use_effect(
                move || {
                    Some(move || {
                        cleanup2_clone2.fetch_add(1, Ordering::Relaxed);
                    })
                },
                "test", // Same
            );

            // Only first cleanup should have run
            assert_eq!(cleanup1_ran.load(Ordering::Relaxed), 1);
            assert_eq!(cleanup2_ran.load(Ordering::Relaxed), 0);
        });
    });
}

/// Test useEffect integration with useState
#[test]
fn test_use_effect_with_use_state() {
    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_component_id("StateIntegrationEffectComponent", |_context| {
            // Simulate useState hook
            let (count_handle, set_count) = use_state(|| 0);
            let count = count_handle.get();

            // Effect that depends on state
            let effect_ran_clone = effect_ran.clone();
            use_effect(
                move || {
                    effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                count,
            );

            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);

            // Update state
            set_count.set(1);
        });

        // Second render - state should persist and effect should run again
        with_component_id("StateIntegrationEffectComponent", |_context| {
            let (count_handle2, _) = use_state(|| 0); // Will get updated value
            let count2 = count_handle2.get();

            let effect_ran_clone2 = effect_ran.clone();
            use_effect(
                move || {
                    effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                    None::<Box<dyn FnOnce() + Send>>
                },
                count2,
            );

            // Effect should run again because count changed
            assert_eq!(effect_ran.load(Ordering::Relaxed), 2);
        });
    });
}

/// Test useEffect called outside component context
#[test]
#[should_panic(expected = "with_hook_context must be called within a hook context")]
fn test_use_effect_outside_context() {
    // This should panic when called outside of component context
    use_effect::<(), _, _>(|| None::<Box<dyn FnOnce() + Send>>, None);
}

/// Test rapid dependency changes
#[test]
fn test_use_effect_rapid_changes() {
    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));
        let cleanup_ran = Arc::new(AtomicUsize::new(0));

        // Simulate rapid dependency changes (multiple renders)
        for i in 0..100 {
            with_component_id("RapidChangesEffectComponent", |_context| {
                let effect_ran_clone = effect_ran.clone();
                let cleanup_ran_clone = cleanup_ran.clone();

                use_effect(
                    move || {
                        effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                        Some(move || {
                            cleanup_ran_clone.fetch_add(1, Ordering::Relaxed);
                        })
                    },
                    i,
                );
            });
        }

        // Effect should have run 100 times
        assert_eq!(effect_ran.load(Ordering::Relaxed), 100);
        // Cleanup should have run 99 times (not on the last iteration)
        assert_eq!(cleanup_ran.load(Ordering::Relaxed), 99);
    });
}

/// Test thread safety of effects (each thread has its own context)
#[test]
fn test_use_effect_thread_safety() {
    use std::sync::Barrier;
    use std::thread;

    let effect_count = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(4));

    let mut handles = vec![];

    // Spawn multiple threads that create effects
    for i in 0..3 {
        let effect_count_clone = effect_count.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Each thread creates its own context using test utilities
            with_hook_context(|_context| {
                barrier_clone.wait();

                use_effect(
                    move || {
                        effect_count_clone.fetch_add(1, Ordering::Relaxed);
                        None::<Box<dyn FnOnce() + Send>>
                    },
                    i,
                );
            });
        });

        handles.push(handle);
    }

    // Wait for all threads to be ready
    barrier.wait();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // All effects should have run
    assert_eq!(effect_count.load(Ordering::Relaxed), 3);
}

/// Test dependency comparison performance
#[test]
fn test_use_effect_dependency_performance() {
    use std::time::Instant;

    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // Create a large tuple dependency to test comparison performance
        let large_deps = (1, 2, 3, 4, 5, 6, 7, 8);

        let start = Instant::now();

        // Run many iterations with same dependencies (simulating multiple renders)
        for _ in 0..1000 {
            with_component_id("DependencyPerformanceEffectComponent", |_context| {
                let effect_ran_clone = effect_ran.clone();

                use_effect(
                    move || {
                        effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                        None::<Box<dyn FnOnce() + Send>>
                    },
                    large_deps,
                );
            });
        }

        let duration = start.elapsed();

        // Effect should only run once (first time)
        assert_eq!(effect_ran.load(Ordering::Relaxed), 1);

        // Performance assertion - dependency comparison should be fast
        assert!(
            duration < std::time::Duration::from_millis(100),
            "Dependency comparison took too long: {:?}",
            duration
        );
    });
}

/// Test memory cleanup when effects are removed
#[test]
fn test_use_effect_memory_cleanup() {
    // This test ensures that effect state is properly cleaned up
    for iteration in 0..100 {
        with_hook_context(|_context| {
            // Create effects with large data
            let large_data = vec![iteration; 1000];
            use_effect(
                move || {
                    let _data = large_data; // Capture large data
                    Some(move || {
                        // Cleanup with large data
                        let _cleanup_data = vec![iteration; 1000];
                    })
                },
                iteration,
            );

            // Create multiple effects
            for i in 0..10 {
                let data = vec![i; 100];
                use_effect(
                    move || {
                        let _data = data;
                        None::<Box<dyn FnOnce() + Send>>
                    },
                    i,
                );
            }
        });
        // Context cleanup happens automatically when with_hook_context ends
    }

    // If we get here without running out of memory, cleanup is working
    // This test passes by not running out of memory
}

/// Test effect scheduling overhead
#[test]
fn test_use_effect_scheduling_overhead() {
    use std::time::Instant;

    with_test_isolate(|| {
        with_hook_context(|_context| {
            let start = Instant::now();

            // Create many effects quickly
            for i in 0..1000 {
                use_effect(
                    move || {
                        // Minimal effect
                        None::<Box<dyn FnOnce() + Send>>
                    },
                    i,
                );
            }

            let duration = start.elapsed();

            // Performance assertion - effect creation should be fast
            assert!(
                duration < std::time::Duration::from_millis(100),
                "Effect creation took too long: {:?}",
                duration
            );
        });
    });
}

/// Test cleanup function management efficiency
#[test]
fn test_use_effect_cleanup_management() {
    with_test_isolate(|| {
        let cleanup_count = Arc::new(AtomicUsize::new(0));

        // First render
        with_component_id("CleanupManagementEffectComponent", |_context| {
            let cleanup_count_clone = cleanup_count.clone();
            use_effect(
                move || {
                    Some(move || {
                        cleanup_count_clone.fetch_add(1, Ordering::Relaxed);
                    })
                },
                1,
            );
        });

        // Change dependency multiple times
        for i in 2..=100 {
            with_component_id("CleanupManagementEffectComponent", |_context| {
                let cleanup_count_clone = cleanup_count.clone();
                use_effect(
                    move || {
                        Some(move || {
                            cleanup_count_clone.fetch_add(1, Ordering::Relaxed);
                        })
                    },
                    i,
                );
            });
        }

        // Cleanup should have run 99 times (once for each dependency change)
        assert_eq!(cleanup_count.load(Ordering::Relaxed), 99);
    });
}

/// Test basic useAsyncEffect functionality
#[tokio::test]
async fn test_use_async_effect_basic() {
    with_async_test_isolate(|| async {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_async_component_id("BasicAsyncEffectComponent", |_context| async {
            let effect_ran_clone = effect_ran.clone();

            // Async effect with no dependencies - should run on every render
            use_async_effect::<(), _, _, _, _>(
                move || {
                    let effect_ran_inner = effect_ran_clone.clone();
                    async move {
                        effect_ran_inner.fetch_add(1, Ordering::Relaxed);
                        None::<
                            fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
                        >
                    }
                },
                None,
            );

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        })
        .await;

        // Second render (same component ID)
        with_async_component_id("BasicAsyncEffectComponent", |_context| async {
            let effect_ran_clone2 = effect_ran.clone();
            use_async_effect::<(), _, _, _, _>(
                move || {
                    let effect_ran_inner = effect_ran_clone2.clone();
                    async move {
                        effect_ran_inner.fetch_add(1, Ordering::Relaxed);
                        None::<
                            fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
                        >
                    }
                },
                None,
            );

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            assert_eq!(effect_ran.load(Ordering::Relaxed), 2);
        })
        .await;
    })
    .await;
}

/// Test useAsyncEffect with empty dependencies (run once)
#[tokio::test]
async fn test_use_async_effect_empty_deps() {
    with_async_test_isolate(|| async {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_async_component_id("EmptyDepsAsyncEffectComponent", |_context| async {
            let effect_ran_clone = effect_ran.clone();

            // Async effect with empty dependencies - should run only once
            use_async_effect(
                move || {
                    let effect_ran_inner = effect_ran_clone.clone();
                    async move {
                        effect_ran_inner.fetch_add(1, Ordering::Relaxed);
                        None::<
                            fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
                        >
                    }
                },
                (),
            );

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        })
        .await;

        // Second render (same component ID)
        with_async_component_id("EmptyDepsAsyncEffectComponent", |_context| async {
            let effect_ran_clone2 = effect_ran.clone();
            use_async_effect(
                move || {
                    let effect_ran_inner = effect_ran_clone2.clone();
                    async move {
                        effect_ran_inner.fetch_add(1, Ordering::Relaxed);
                        None::<
                            fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
                        >
                    }
                },
                (),
            );

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            // Should not run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        })
        .await;
    })
    .await;
}

/// Test useAsyncEffect with changing dependencies
#[tokio::test]
async fn test_use_async_effect_changing_deps() {
    with_async_test_isolate(|| async {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render with dependency value 1
        with_async_component_id("ChangingDepsAsyncEffectComponent", |_context| async {
            let effect_ran_clone = effect_ran.clone();

            use_async_effect(
                move || {
                    let effect_ran_inner = effect_ran_clone.clone();
                    async move {
                        effect_ran_inner.fetch_add(1, Ordering::Relaxed);
                        None::<
                            fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
                        >
                    }
                },
                1,
            );

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        })
        .await;

        // Second render with same dependency value
        with_async_component_id("ChangingDepsAsyncEffectComponent", |_context| async {
            let effect_ran_clone2 = effect_ran.clone();
            use_async_effect(
                move || {
                    let effect_ran_inner = effect_ran_clone2.clone();
                    async move {
                        effect_ran_inner.fetch_add(1, Ordering::Relaxed);
                        None::<
                            fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
                        >
                    }
                },
                1,
            );

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            // Should not run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        })
        .await;

        // Third render with different dependency value
        with_async_component_id("ChangingDepsAsyncEffectComponent", |_context| async {
            let effect_ran_clone3 = effect_ran.clone();
            use_async_effect(
                move || {
                    let effect_ran_inner = effect_ran_clone3.clone();
                    async move {
                        effect_ran_inner.fetch_add(1, Ordering::Relaxed);
                        None::<
                            fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
                        >
                    }
                },
                2,
            );

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            // Should run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 2);
        })
        .await;
    })
    .await;
}

/// Test use_effect_once convenience function
#[test]
fn test_use_effect_once() {
    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));
        let cleanup_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_component_id("OnceEffectComponent", |_context| {
            let effect_ran_clone = effect_ran.clone();
            let cleanup_ran_clone = cleanup_ran.clone();

            use_effect_once(move || {
                effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                move || {
                    cleanup_ran_clone.fetch_add(1, Ordering::Relaxed);
                }
            });

            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
            assert_eq!(cleanup_ran.load(Ordering::Relaxed), 0);
        });

        // Second render - effect should not run again
        with_component_id("OnceEffectComponent", |_context| {
            let effect_ran_clone2 = effect_ran.clone();
            let cleanup_ran_clone2 = cleanup_ran.clone();

            use_effect_once(move || {
                effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                move || {
                    cleanup_ran_clone2.fetch_add(1, Ordering::Relaxed);
                }
            });

            // Should not run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
            assert_eq!(cleanup_ran.load(Ordering::Relaxed), 0);
        });
    });
}

/// Test use_effect_always convenience function
#[test]
fn test_use_effect_always() {
    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));
        let cleanup_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_component_id("AlwaysEffectComponent", |_context| {
            let effect_ran_clone = effect_ran.clone();
            let cleanup_ran_clone = cleanup_ran.clone();

            use_effect_always(move || {
                effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                move || {
                    cleanup_ran_clone.fetch_add(1, Ordering::Relaxed);
                }
            });

            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
            assert_eq!(cleanup_ran.load(Ordering::Relaxed), 0);
        });

        // Second render - effect should run again
        with_component_id("AlwaysEffectComponent", |_context| {
            let effect_ran_clone2 = effect_ran.clone();
            let cleanup_ran_clone2 = cleanup_ran.clone();

            use_effect_always(move || {
                effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                move || {
                    cleanup_ran_clone2.fetch_add(1, Ordering::Relaxed);
                }
            });

            // Should run again and cleanup from previous run
            assert_eq!(effect_ran.load(Ordering::Relaxed), 2);
            assert_eq!(cleanup_ran.load(Ordering::Relaxed), 1);
        });

        // Third render - effect should run again
        with_component_id("AlwaysEffectComponent", |_context| {
            let effect_ran_clone3 = effect_ran.clone();
            let cleanup_ran_clone3 = cleanup_ran.clone();

            use_effect_always(move || {
                effect_ran_clone3.fetch_add(1, Ordering::Relaxed);
                move || {
                    cleanup_ran_clone3.fetch_add(1, Ordering::Relaxed);
                }
            });

            // Should run again and cleanup from previous run
            assert_eq!(effect_ran.load(Ordering::Relaxed), 3);
            assert_eq!(cleanup_ran.load(Ordering::Relaxed), 2);
        });
    });
}

/// Test use_async_effect_once convenience function
#[tokio::test]
async fn test_use_async_effect_once() {
    with_async_test_isolate(|| async {
        let effect_ran = Arc::new(AtomicUsize::new(0));
        let cleanup_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_async_component_id("AsyncOnceEffectComponent", |_context| async {
            let effect_ran_clone = effect_ran.clone();
            let cleanup_ran_clone = cleanup_ran.clone();

            use_async_effect_once(move || async move {
                effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                move || async move {
                    cleanup_ran_clone.fetch_add(1, Ordering::Relaxed);
                }
            });

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
            assert_eq!(cleanup_ran.load(Ordering::Relaxed), 0);
        })
        .await;

        // Second render - effect should not run again
        with_async_component_id("AsyncOnceEffectComponent", |_context| async {
            let effect_ran_clone2 = effect_ran.clone();
            let cleanup_ran_clone2 = cleanup_ran.clone();

            use_async_effect_once(move || async move {
                effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                move || async move {
                    cleanup_ran_clone2.fetch_add(1, Ordering::Relaxed);
                }
            });

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            // Should not run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
            assert_eq!(cleanup_ran.load(Ordering::Relaxed), 0);
        })
        .await;
    })
    .await;
}

/// Test use_async_effect_always convenience function
#[tokio::test]
async fn test_use_async_effect_always() {
    with_async_test_isolate(|| async {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_async_component_id("AsyncAlwaysEffectComponent", |_context| async {
            let effect_ran_clone = effect_ran.clone();

            use_async_effect_always(move || async move {
                effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                || async move {
                    // Simple cleanup without tracking
                }
            });

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        })
        .await;

        // Second render - effect should run again
        with_async_component_id("AsyncAlwaysEffectComponent", |_context| async {
            let effect_ran_clone2 = effect_ran.clone();

            use_async_effect_always(move || async move {
                effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                || async move {
                    // Simple cleanup without tracking
                }
            });

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            // Should run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 2);
        })
        .await;

        // Third render - effect should run again
        with_async_component_id("AsyncAlwaysEffectComponent", |_context| async {
            let effect_ran_clone3 = effect_ran.clone();

            use_async_effect_always(move || async move {
                effect_ran_clone3.fetch_add(1, Ordering::Relaxed);
                || async move {
                    // Simple cleanup without tracking
                }
            });

            // Give async effect time to run
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            // Should run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 3);
        })
        .await;
    })
    .await;
}

/// Test use_effect_once without cleanup (empty closure)
#[test]
fn test_use_effect_once_empty_cleanup() {
    with_test_isolate(|| {
        let effect_ran = Arc::new(AtomicUsize::new(0));

        // First render
        with_component_id("OnceEmptyCleanupEffectComponent", |_context| {
            let effect_ran_clone = effect_ran.clone();

            use_effect_once(move || {
                effect_ran_clone.fetch_add(1, Ordering::Relaxed);
                || {} // Empty cleanup
            });

            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });

        // Second render - effect should not run again
        with_component_id("OnceEmptyCleanupEffectComponent", |_context| {
            let effect_ran_clone2 = effect_ran.clone();

            use_effect_once(move || {
                effect_ran_clone2.fetch_add(1, Ordering::Relaxed);
                || {} // Empty cleanup
            });

            // Should not run again
            assert_eq!(effect_ran.load(Ordering::Relaxed), 1);
        });
    });
}
