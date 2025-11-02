//! Tests for the timeout hook

use super::*;
use crate::state::use_state;
use crate::test_utils::{with_component_id, with_test_isolate};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_use_timeout_basic() {
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        with_component_id("TimeoutBasicTest", |_ctx| {
            use_timeout(
                move || {
                    *called_clone.lock() = true;
                },
                Duration::from_millis(50),
            );
        });

        // Should not be called immediately
        assert!(!*called.lock(), "Should not be called immediately");
    });

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Note: In a real component lifecycle, the callback would be called
    // This test verifies the hook setup completes without errors
}

#[tokio::test]
async fn test_use_timeout_with_state() {
    with_test_isolate(|| {
        let value = Arc::new(Mutex::new(0));
        let value_clone = value.clone();

        with_component_id("TimeoutStateTest", |_ctx| {
            let (count, set_count) = use_state(|| 0);

            use_timeout(
                {
                    let set_count = set_count.clone();
                    move || {
                        set_count.set(42);
                    }
                },
                Duration::from_millis(50),
            );

            *value_clone.lock() = count.get();
        });

        // Initial value should be 0
        assert_eq!(*value.lock(), 0);
    });
}

#[tokio::test]
async fn test_use_timeout_with_reset() {
    with_test_isolate(|| {
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        with_component_id("TimeoutResetTest", |_ctx| {
            let reset = use_timeout_with_reset(
                move || {
                    *call_count_clone.lock() += 1;
                },
                Duration::from_millis(50),
            );

            // Reset immediately (should restart the timer)
            reset();
        });

        assert_eq!(*call_count.lock(), 0, "Should not be called yet");
    });
}

#[tokio::test]
async fn test_use_timeout_controlled_start() {
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        with_component_id("TimeoutControlledTest", |_ctx| {
            let (start, _cancel, is_active) = use_timeout_controlled(
                move || {
                    *called_clone.lock() = true;
                },
                Duration::from_millis(50),
            );

            // Should not be active initially
            assert!(!is_active(), "Should not be active initially");

            // Start the timeout
            start();

            // Should be active after start
            assert!(is_active(), "Should be active after start");
        });
    });
}

#[tokio::test]
async fn test_use_timeout_controlled_cancel() {
    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    with_test_isolate(|| {
        with_component_id("TimeoutCancelTest", |_ctx| {
            let (start, cancel, is_active) = use_timeout_controlled(
                {
                    let called = called_clone.clone();
                    move || {
                        *called.lock() = true;
                    }
                },
                Duration::from_millis(50),
            );

            // Start the timeout
            start();
            assert!(is_active(), "Should be active after start");

            // Cancel immediately
            cancel();
            assert!(!is_active(), "Should not be active after cancel");
        });
    });

    // Wait to ensure callback is not called
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!*called.lock(), "Should not be called after cancel");
}

#[test]
fn test_use_timeout_zero_duration() {
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        with_component_id("TimeoutZeroTest", |_ctx| {
            use_timeout(
                move || {
                    *called_clone.lock() = true;
                },
                Duration::from_millis(0),
            );
        });

        // Should complete without errors
    });
}

#[test]
fn test_use_timeout_long_duration() {
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        with_component_id("TimeoutLongTest", |_ctx| {
            use_timeout(
                move || {
                    *called_clone.lock() = true;
                },
                Duration::from_secs(3600), // 1 hour
            );
        });

        // Should complete without errors
        assert!(!*called.lock(), "Should not be called immediately");
    });
}

#[test]
fn test_use_timeout_multiple_in_component() {
    with_test_isolate(|| {
        let count1 = Arc::new(Mutex::new(0));
        let count2 = Arc::new(Mutex::new(0));
        let count1_clone = count1.clone();
        let count2_clone = count2.clone();

        with_component_id("TimeoutMultipleTest", |_ctx| {
            // First timeout
            use_timeout(
                move || {
                    *count1_clone.lock() += 1;
                },
                Duration::from_millis(50),
            );

            // Second timeout
            use_timeout(
                move || {
                    *count2_clone.lock() += 1;
                },
                Duration::from_millis(100),
            );
        });

        // Both should be set up without errors
        assert_eq!(*count1.lock(), 0);
        assert_eq!(*count2.lock(), 0);
    });
}

#[tokio::test]
async fn test_use_timeout_with_reset_multiple_resets() {
    with_test_isolate(|| {
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        with_component_id("TimeoutMultipleResetsTest", |_ctx| {
            let reset = use_timeout_with_reset(
                move || {
                    *call_count_clone.lock() += 1;
                },
                Duration::from_millis(50),
            );

            // Reset multiple times
            reset();
            reset();
            reset();
        });

        assert_eq!(*call_count.lock(), 0);
    });
}

#[test]
fn test_use_timeout_controlled_restart() {
    with_test_isolate(|| {
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        with_component_id("TimeoutRestartTest", |_ctx| {
            let (start, cancel, is_active) = use_timeout_controlled(
                move || {
                    *call_count_clone.lock() += 1;
                },
                Duration::from_millis(50),
            );

            // Start, cancel, start again
            start();
            assert!(is_active());

            cancel();
            assert!(!is_active());

            start();
            assert!(is_active());
        });
    });
}

#[test]
fn test_use_timeout_idempotent_setup() {
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Multiple renders should not cause issues
        for _ in 0..3 {
            with_component_id("TimeoutIdempotentTest", |_ctx| {
                use_timeout(
                    {
                        let called = called_clone.clone();
                        move || {
                            *called.lock() = true;
                        }
                    },
                    Duration::from_millis(50),
                );
            });
        }
    });
}
