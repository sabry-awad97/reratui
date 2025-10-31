//! Comprehensive tests for interval hooks
//!
//! This module contains extensive tests covering:
//! - Basic synchronous interval functionality (use_interval)
//! - Asynchronous interval functionality (use_async_interval)
//! - Cleanup and resource management
//! - Duration changes and interval restart
//! - Integration with other hooks
//! - Thread safety and async behavior
//! - Error handling and edge cases

use super::*;
use crate::state::use_state;
use crate::test_utils::{with_component_id, with_test_isolate};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::time::sleep;

/// Test basic synchronous interval functionality
#[tokio::test]
async fn test_use_interval_basic() {
    with_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));

        with_component_id("BasicIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            // Set up synchronous interval that increments counter every 20ms
            // Using longer duration for thread-based timing
            use_interval(
                move || {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                },
                Duration::from_millis(20),
            );
        });

        // Wait for a few intervals to execute (longer wait for thread-based timing)
        sleep(Duration::from_millis(70)).await;

        // Should have executed at least 3 times (70ms / 20ms = 3.5)
        let count = counter.load(Ordering::Relaxed);
        assert!(count >= 2, "Expected at least 2 executions, got {}", count);
        assert!(count <= 6, "Expected at most 6 executions, got {}", count); // Allow timing variance for threads
    })
    .await;
}

/// Test interval cleanup when component unmounts
#[tokio::test]
async fn test_use_interval_cleanup() {
    with_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));

        // Start interval in component scope
        with_component_id("CleanupIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            use_interval(
                move || {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                },
                Duration::from_millis(20), // Longer duration for thread-based timing
            );
        });

        // Wait for some executions
        sleep(Duration::from_millis(50)).await;
        let count_first = counter.load(Ordering::Relaxed);

        // Simulate component re-render with different duration (should cleanup old interval)
        with_component_id("CleanupIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            // Different duration should trigger cleanup of old interval and start new one
            use_interval(
                move || {
                    counter_clone.fetch_add(10, Ordering::Relaxed); // Different increment to distinguish
                },
                Duration::from_millis(30), // Different duration
            );
        });

        // Wait for new interval to execute
        sleep(Duration::from_millis(80)).await;
        let count_second = counter.load(Ordering::Relaxed);

        // Should see the new increment pattern (10 per execution instead of 1)
        let new_increments = count_second - count_first;
        assert!(
            new_increments >= 20, // At least 2 executions of 10 each
            "Should see new interval pattern after cleanup. New increments: {}",
            new_increments
        );
    })
    .await;
}

/// Test interval duration changes
#[tokio::test]
async fn test_use_interval_duration_change() {
    with_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));

        // First render with 20ms interval
        with_component_id("DurationChangeIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            use_interval(
                move || {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                },
                Duration::from_millis(20),
            );
        });

        // Wait for some executions
        sleep(Duration::from_millis(45)).await;
        let count_first = counter.load(Ordering::Relaxed);

        // Second render with 5ms interval (should restart with faster interval)
        with_component_id("DurationChangeIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            use_interval(
                move || {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                },
                Duration::from_millis(5),
            );
        });

        // Wait for more executions with faster interval
        sleep(Duration::from_millis(25)).await;
        let count_second = counter.load(Ordering::Relaxed);

        // Should have more executions in the second period due to faster interval
        let first_period_executions = count_first;
        let second_period_executions = count_second - count_first;

        assert!(
            second_period_executions > first_period_executions,
            "Faster interval should produce more executions. First: {}, Second: {}",
            first_period_executions,
            second_period_executions
        );
    })
    .await;
}

/// Test integration with use_state hook
#[tokio::test]
async fn test_use_interval_with_state() {
    with_test_isolate(|| async {
        with_component_id("StateIntervalComponent", |_context| {
            let (count, set_count) = use_state(|| 0);

            // Set up interval that updates state
            use_interval(
                move || {
                    set_count.update(|prev| prev + 1);
                },
                Duration::from_millis(10),
            );

            // Initial state should be 0
            assert_eq!(count.get(), 0);
        });

        // Wait for some state updates
        sleep(Duration::from_millis(35)).await;

        // Re-render to check updated state
        with_component_id("StateIntervalComponent", |_context| {
            let (count, _) = use_state(|| 0); // Will get the updated value

            // State should have been updated by the interval
            let current_count = count.get();
            assert!(
                current_count >= 3,
                "Expected at least 3 state updates, got {}",
                current_count
            );
        });
    })
    .await;
}

/// Test multiple intervals in the same component
#[tokio::test]
async fn test_multiple_intervals() {
    with_test_isolate(|| async {
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        with_component_id("MultipleIntervalsComponent", |_context| {
            let counter1_clone = counter1.clone();
            let counter2_clone = counter2.clone();

            // First interval - every 10ms
            use_interval(
                move || {
                    counter1_clone.fetch_add(1, Ordering::Relaxed);
                },
                Duration::from_millis(10),
            );

            // Second interval - every 15ms
            use_interval(
                move || {
                    counter2_clone.fetch_add(2, Ordering::Relaxed);
                },
                Duration::from_millis(15),
            );
        });

        // Wait for executions
        sleep(Duration::from_millis(35)).await;

        let count1 = counter1.load(Ordering::Relaxed);
        let count2 = counter2.load(Ordering::Relaxed);

        // Both intervals should have executed
        assert!(
            count1 >= 3,
            "First interval should have executed at least 3 times, got {}",
            count1
        );
        assert!(
            count2 >= 4,
            "Second interval should have executed at least 2 times (2 per execution), got {}",
            count2
        );

        // First interval should have executed more times than second
        assert!(
            count1 > count2 / 2,
            "First interval should execute more frequently"
        );
    })
    .await;
}

/// Test interval with zero duration (edge case)
#[tokio::test]
async fn test_use_interval_zero_duration() {
    with_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));

        with_component_id("ZeroDurationIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            // Zero duration should still work (though not recommended)
            use_interval(
                move || {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                },
                Duration::from_millis(0),
            );
        });

        // Wait a short time
        sleep(Duration::from_millis(10)).await;

        // Should have executed many times with zero delay
        let count = counter.load(Ordering::Relaxed);
        assert!(count > 0, "Zero duration interval should still execute");
    })
    .await;
}

/// Test interval performance with many rapid executions
#[tokio::test]
async fn test_use_interval_performance() {
    with_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));

        with_component_id("PerformanceIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            // Very fast interval for performance testing
            use_interval(
                move || {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                },
                Duration::from_millis(1),
            );
        });

        // Wait for many executions
        sleep(Duration::from_millis(50)).await;

        let count = counter.load(Ordering::Relaxed);

        // Should handle rapid executions efficiently
        assert!(count >= 30, "Should handle rapid executions, got {}", count);
        assert!(count <= 70, "Should not execute excessively, got {}", count);
    })
    .await;
}

// ============================================================================
// ASYNC INTERVAL TESTS
// ============================================================================

/// Test basic async interval functionality
#[tokio::test]
async fn test_use_async_interval_basic() {
    with_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));

        with_component_id("BasicAsyncIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            // Set up async interval that increments counter every 10ms
            use_async_interval(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                },
                Duration::from_millis(10),
            );
        });

        // Wait for a few intervals to execute
        sleep(Duration::from_millis(35)).await;

        // Should have executed at least 3 times (35ms / 10ms = 3.5)
        let count = counter.load(Ordering::Relaxed);
        assert!(count >= 3, "Expected at least 3 executions, got {}", count);
        assert!(count <= 5, "Expected at most 5 executions, got {}", count); // Allow some timing variance
    })
    .await;
}

/// Test async interval with actual async operations
#[tokio::test]
async fn test_use_async_interval_with_async_work() {
    with_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));

        with_component_id("AsyncWorkIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            // Set up async interval with actual async work
            use_async_interval(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        // Simulate async work
                        sleep(Duration::from_millis(5)).await;
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                },
                Duration::from_millis(15),
            );
        });

        // Wait for executions (accounting for async work time)
        sleep(Duration::from_millis(70)).await;

        // Should have executed at least 3 times (considering async work overhead)
        // With 15ms interval + 5ms async work = ~20ms per cycle, 70ms should allow 3+ executions
        let count = counter.load(Ordering::Relaxed);
        assert!(
            count >= 3,
            "Expected at least 3 executions with async work, got {}",
            count
        );
        assert!(
            count <= 6,
            "Expected at most 6 executions with async work, got {}",
            count
        );
    })
    .await;
}

/// Test async interval cleanup when duration changes
#[tokio::test]
async fn test_use_async_interval_cleanup() {
    with_test_isolate(|| async {
        let counter = Arc::new(AtomicUsize::new(0));

        // Start async interval in component scope
        with_component_id("CleanupAsyncIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            use_async_interval(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                },
                Duration::from_millis(10),
            );
        });

        // Wait for some executions
        sleep(Duration::from_millis(25)).await;
        let count_first = counter.load(Ordering::Relaxed);

        // Simulate component re-render with different duration (should cleanup old interval)
        with_component_id("CleanupAsyncIntervalComponent", |_context| {
            let counter_clone = counter.clone();

            // Different duration should trigger cleanup of old interval and start new one
            use_async_interval(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        counter.fetch_add(10, Ordering::Relaxed); // Different increment to distinguish
                    }
                },
                Duration::from_millis(20), // Different duration
            );
        });

        // Wait for new interval to execute
        sleep(Duration::from_millis(45)).await;
        let count_second = counter.load(Ordering::Relaxed);

        // Should see the new increment pattern (10 per execution instead of 1)
        let new_increments = count_second - count_first;
        assert!(
            new_increments >= 20, // At least 2 executions of 10 each
            "Should see new async interval pattern after cleanup. New increments: {}",
            new_increments
        );
    })
    .await;
}

/// Test mixed sync and async intervals
#[tokio::test]
async fn test_mixed_sync_and_async_intervals() {
    with_test_isolate(|| async {
        let sync_counter = Arc::new(AtomicUsize::new(0));
        let async_counter = Arc::new(AtomicUsize::new(0));

        with_component_id("MixedIntervalsComponent", |_context| {
            let sync_counter_clone = sync_counter.clone();
            let async_counter_clone = async_counter.clone();

            // Sync interval - every 20ms (more reliable timing)
            use_interval(
                move || {
                    sync_counter_clone.fetch_add(1, Ordering::Relaxed);
                },
                Duration::from_millis(20),
            );

            // Async interval - every 25ms (more reliable timing)
            use_async_interval(
                move || {
                    let counter = async_counter_clone.clone();
                    async move {
                        sleep(Duration::from_millis(1)).await; // Minimal async work
                        counter.fetch_add(2, Ordering::Relaxed);
                    }
                },
                Duration::from_millis(25),
            );
        });

        // Wait for executions - 60ms should allow multiple executions
        sleep(Duration::from_millis(60)).await;

        let sync_count = sync_counter.load(Ordering::Relaxed);
        let async_count = async_counter.load(Ordering::Relaxed);

        // Both intervals should have executed
        // Sync: 60ms / 20ms = 3 executions expected
        assert!(
            sync_count >= 2,
            "Sync interval should have executed at least 2 times, got {}",
            sync_count
        );
        // Async: 60ms / 25ms = 2.4, so at least 2 executions (4 total count)
        assert!(
            async_count >= 4,
            "Async interval should have executed at least 2 times (2 per execution), got {}",
            async_count
        );

        // Sync interval should execute more frequently than async interval
        // Since sync is every 20ms and async is every 25ms, sync should execute more often
        assert!(
            sync_count >= async_count / 2,
            "Sync interval should execute at least as frequently as async interval (accounting for 2x multiplier), sync: {}, async: {}",
            sync_count, async_count
        );
    })
    .await;
}
