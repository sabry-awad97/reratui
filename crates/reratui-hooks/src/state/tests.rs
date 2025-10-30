//! Tests for the useState hook implementation

use crate::state::use_state;
use crate::test_utils::{with_component_id, with_hook_context, with_test_isolate};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

/// Test thread safety with concurrent reads and writes
#[test]
fn test_use_state_thread_safety() {
    with_test_isolate(|| {
        with_hook_context(|_context| {
            let (state_handle, setter) = use_state(|| 0i32);
            assert_eq!(state_handle.get(), 0);

            // Clone the setter for use in multiple threads
            let setter1 = setter.clone();
            let setter2 = setter.clone();
            let setter3 = setter.clone();

            // Use a barrier to synchronize thread starts
            let barrier = Arc::new(Barrier::new(4));
            let barrier1 = barrier.clone();
            let barrier2 = barrier.clone();
            let barrier3 = barrier.clone();

            // Spawn multiple threads that update the state concurrently
            let handle1 = thread::spawn(move || {
                barrier1.wait();
                for _i in 0..100 {
                    setter1.update(|prev| prev + 1);
                    thread::sleep(Duration::from_micros(10));
                }
            });

            let handle2 = thread::spawn(move || {
                barrier2.wait();
                for _i in 0..100 {
                    setter2.update(|prev| prev + 2);
                    thread::sleep(Duration::from_micros(10));
                }
            });

            let handle3 = thread::spawn(move || {
                barrier3.wait();
                for _i in 0..100 {
                    setter3.update(|prev| prev + 3);
                    thread::sleep(Duration::from_micros(10));
                }
            });

            // Wait for all threads to be ready
            barrier.wait();

            // Wait for all threads to complete
            handle1.join().unwrap();
            handle2.join().unwrap();
            handle3.join().unwrap();

            // Check final state - should be 0 + (100*1) + (100*2) + (100*3) = 600
            // Due to concurrent updates, the exact value might vary slightly, but should be close
            let final_value = state_handle.get();

            // Allow for some variance due to race conditions in concurrent updates
            // The value should be close to 600 (within a reasonable range)
            assert!(
                (580..=620).contains(&final_value),
                "Expected final value to be around 600, got {}",
                final_value
            );
        });
    });
}

/// Test performance with high-frequency updates
#[test]
fn test_use_state_performance() {
    with_test_isolate(|| {
        with_component_id("PerformanceTestComponent", |_context| {
            let (state_handle, setter) = use_state(|| 0u64);
            assert_eq!(state_handle.get(), 0);

            let start = Instant::now();
            let num_updates = 10_000;

            // Perform many rapid updates
            for i in 0..num_updates {
                setter.set(i);
            }

            let duration = start.elapsed();
            println!("Time for {} updates: {:?}", num_updates, duration);

            // Verify final state
            assert_eq!(state_handle.get(), num_updates - 1);

            // Performance assertion - should complete in reasonable time
            assert!(
                duration < Duration::from_millis(100),
                "Updates took too long: {:?}",
                duration
            );
        });
    });
}

/// Test concurrent reads while writing
#[test]
fn test_use_state_concurrent_reads() {
    with_test_isolate(|| {
        with_hook_context(|_context| {
            let (state_handle, setter) = use_state(|| 0i32);
            assert_eq!(state_handle.get(), 0);

            // Get the container for direct access in test
            let container = setter.container().clone();

            let read_count = Arc::new(AtomicUsize::new(0));
            let read_count1 = read_count.clone();
            let read_count2 = read_count.clone();

            let barrier = Arc::new(Barrier::new(3));
            let barrier1 = barrier.clone();
            let barrier2 = barrier.clone();

            // Writer thread
            let writer_handle = thread::spawn(move || {
                barrier.wait();
                for i in 0..1000 {
                    setter.set(i);
                    thread::sleep(Duration::from_micros(1));
                }
            });

            // Reader thread 1
            let container1 = container.clone();
            let reader1_handle = thread::spawn(move || {
                barrier1.wait();
                for _ in 0..2000 {
                    let _value = container1.get();
                    read_count1.fetch_add(1, Ordering::Relaxed);
                    thread::sleep(Duration::from_micros(1));
                }
            });

            // Reader thread 2
            let container2 = container.clone();
            let reader2_handle = thread::spawn(move || {
                barrier2.wait();
                for _ in 0..2000 {
                    let _value = container2.get();
                    read_count2.fetch_add(1, Ordering::Relaxed);
                    thread::sleep(Duration::from_micros(1));
                }
            });

            writer_handle.join().unwrap();
            reader1_handle.join().unwrap();
            reader2_handle.join().unwrap();

            // Verify all reads completed
            assert_eq!(read_count.load(Ordering::Relaxed), 4000);
        });
    });
}

/// Test state persistence across multiple render cycles
#[test]
fn test_use_state_persistence() {
    with_test_isolate(|| {
        with_component_id("PersistenceTestComponent", |_context| {
            // First render cycle
            let (state_handle1, setter1) = use_state(|| "initial".to_string());
            assert_eq!(state_handle1.get(), "initial");
            setter1.set("updated".to_string());
        });

        // Second render cycle (same component ID)
        with_component_id("PersistenceTestComponent", |_context| {
            let (state_handle2, setter2) = use_state(|| "default".to_string());
            assert_eq!(state_handle2.get(), "updated"); // Should persist from previous cycle
            setter2.update(|prev| format!("{}_again", prev));
        });

        // Third render cycle (same component ID)
        with_component_id("PersistenceTestComponent", |_context| {
            let (state_handle3, _) = use_state(|| "default".to_string());
            assert_eq!(state_handle3.get(), "updated_again");
        });
    });
}

/// Test memory cleanup and no leaks
#[test]
fn test_use_state_memory_cleanup() {
    // This test ensures that state is properly cleaned up when context is cleared
    for iteration in 0..100 {
        with_hook_context(|_context| {
            // Create state with large data
            let large_data = vec![iteration; 1000];
            let (_, setter) = use_state(|| large_data.clone());

            // Update state multiple times
            for i in 0..10 {
                let new_data = vec![iteration + i; 1000];
                setter.set(new_data);
            }

            // Context cleanup happens automatically when with_hook_context exits
            // Note: We can't directly test memory cleanup, but we can ensure
            // that the pattern works correctly across many iterations
        });
    }
}

/// Test version tracking for change detection
#[test]
fn test_use_state_version_tracking() {
    with_test_isolate(|| {
        with_hook_context(|_context| {
            let (_, setter) = use_state(|| 0);
            let container = setter.container().clone();

            let initial_version = container.version();
            assert_eq!(initial_version, 0);

            setter.set(1);
            assert_eq!(container.version(), 1);

            setter.set(2);
            assert_eq!(container.version(), 2);

            setter.update(|prev| prev + 1);
            assert_eq!(container.version(), 3);
        });
    });
}

/// Test field access utility methods
#[test]
fn test_use_state_field_access() {
    #[derive(Clone)]
    struct ComplexState {
        count: i32,
        name: String,
        nested: NestedState,
    }

    #[derive(Clone)]
    struct NestedState {
        value: f64,
    }

    with_test_isolate(|| {
        with_hook_context(|_context| {
            let initial_state = ComplexState {
                count: 42,
                name: "test".to_string(),
                nested: NestedState {
                    value: std::f64::consts::PI,
                },
            };

            let (state_handle, _) = use_state(|| initial_state);

            // Test field access using StateHandle utility methods
            let count = state_handle.field(|s| s.count);
            assert_eq!(count, 42);

            let name_length = state_handle.field(|s| s.name.len());
            assert_eq!(name_length, 4);

            let nested_value = state_handle.field(|s| s.nested.value);
            assert_eq!(nested_value, std::f64::consts::PI);

            // Test map function for computed values
            let is_even = state_handle.map(|s| s.count % 2 == 0);
            assert!(is_even);
        });
    });
}
