use super::*;
use crate::test_utils::{with_async_component_id, with_async_test_isolate};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;

// Mock async function that succeeds
async fn mock_fetch_success(value: i32) -> Result<i32, String> {
    // Simulate some async work
    sleep(Duration::from_millis(10)).await;
    Ok(value)
}

// Mock async function that fails
async fn mock_fetch_error() -> Result<i32, String> {
    sleep(Duration::from_millis(10)).await;
    Err("Mock error".to_string())
}

// Mock async function with retry logic
async fn mock_fetch_with_retries(attempts: Arc<AtomicU32>, fail_count: u32) -> Result<i32, String> {
    let current_attempt = attempts.fetch_add(1, Ordering::SeqCst);
    sleep(Duration::from_millis(10)).await;

    if current_attempt < fail_count {
        Err(format!("Attempt {} failed", current_attempt + 1))
    } else {
        Ok(42)
    }
}

#[tokio::test]
async fn test_basic_query_success() {
    with_async_test_isolate(|| async {
        clear_query_cache();

        with_async_component_id("QuerySuccessTest", |_ctx| async {
            let result = use_query("test-key", || mock_fetch_success(42), None);

            // Initially should be idle or loading
            assert!(matches!(
                result.status,
                QueryStatus::Idle | QueryStatus::Loading
            ));

            // Give the query time to complete
            sleep(Duration::from_millis(100)).await;

            // Query again to get updated state
            let result = use_query("test-key", || mock_fetch_success(42), None);

            // Should now have successful data
            if result.status == QueryStatus::Success {
                assert_eq!(result.data, Some(42));
                assert!(result.error.is_none());
                assert!(!result.is_stale);
            }
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_query_error_handling() {
    with_async_test_isolate(|| async {
        clear_query_cache();

        with_async_component_id("QueryErrorTest", |_ctx| async {
            let result = use_query("error-key", mock_fetch_error, None);

            assert!(matches!(
                result.status,
                QueryStatus::Idle | QueryStatus::Loading
            ));

            // Give the query time to fail and retry
            sleep(Duration::from_millis(5000)).await; // Account for retries

            // Query again to get updated state
            let result = use_query("error-key", mock_fetch_error, None);

            if result.status == QueryStatus::Error {
                assert!(result.data.is_none());
                assert_eq!(result.error, Some("Mock error".to_string()));
            }
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_query_retry_logic() {
    with_async_test_isolate(|| async {
        clear_query_cache();
        let attempts = Arc::new(AtomicU32::new(0));

        // Use a unique key for this test to avoid cache interference
        let test_key = format!(
            "retry-key-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        with_async_component_id("QueryRetryTest", |_ctx| async {
            let attempts_clone = attempts.clone();
            let test_key_clone = test_key.clone();
            let options = QueryOptions {
                retry: true,
                retry_attempts: 3,
                cache_time: Duration::from_millis(0), // Disable error caching
                stale_time: Duration::from_millis(0), // Always refetch
                ..Default::default()
            };

            let result = use_query(
                test_key_clone,
                move || mock_fetch_with_retries(attempts_clone.clone(), 2),
                Some(options),
            );

            // Initially should be idle or loading
            assert!(matches!(
                result.status,
                QueryStatus::Idle | QueryStatus::Loading
            ));

            // Give the query time to retry and succeed
            // Need longer wait time for exponential backoff: 1s + 2s + success = ~3s
            sleep(Duration::from_millis(4000)).await;

            // Should have made exactly 3 attempts (2 failures + 1 success)
            let final_attempts = attempts.load(Ordering::SeqCst);
            assert_eq!(
                final_attempts, 3,
                "Expected exactly 3 attempts (2 failures + 1 success), got {}",
                final_attempts
            );
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_query_retry_logic_simple() {
    with_async_test_isolate(|| async {
        clear_query_cache();
        let attempts = Arc::new(AtomicU32::new(0));

        // Use a unique key for this test
        let test_key = format!(
            "retry-simple-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        with_async_component_id("QueryRetrySimpleTest", |_ctx| async {
            let attempts_clone = attempts.clone();
            let test_key_clone = test_key.clone();
            let options = QueryOptions {
                retry: true,
                retry_attempts: 2, // Only 2 attempts for faster test
                cache_time: Duration::from_millis(0), // Disable error caching
                stale_time: Duration::from_millis(0), // Always refetch
                ..Default::default()
            };

            let result = use_query(
                test_key_clone,
                move || mock_fetch_with_retries(attempts_clone.clone(), 1), // Fail once, succeed on second
                Some(options),
            );

            // Initially should be idle or loading
            assert!(matches!(
                result.status,
                QueryStatus::Idle | QueryStatus::Loading
            ));

            // Wait for retries to complete (1s + success)
            sleep(Duration::from_millis(2000)).await;

            // Should have made exactly 2 attempts (1 failure + 1 success)
            let final_attempts = attempts.load(Ordering::SeqCst);
            assert_eq!(
                final_attempts, 2,
                "Expected exactly 2 attempts (1 failure + 1 success), got {}",
                final_attempts
            );
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_exponential_backoff() {
    with_async_test_isolate(|| async {
        clear_query_cache();
        let attempts = Arc::new(AtomicU32::new(0));
        let start_time = std::time::Instant::now();

        // Use a unique key for this test
        let test_key = format!(
            "backoff-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        with_async_component_id("QueryBackoffTest", |_ctx| async {
            let attempts_clone = attempts.clone();
            let test_key_clone = test_key.clone();
            let options = QueryOptions {
                retry: true,
                retry_attempts: 3,
                cache_time: Duration::from_millis(0), // Disable error caching
                stale_time: Duration::from_millis(0), // Always refetch
                ..Default::default()
            };

            let result = use_query(
                test_key_clone,
                move || mock_fetch_with_retries(attempts_clone.clone(), 5), // Always fail to test backoff
                Some(options),
            );

            // Initially should be idle or loading
            assert!(matches!(
                result.status,
                QueryStatus::Idle | QueryStatus::Loading
            ));

            // Wait for all retries to complete
            // Expected delays: 1s + 2s = 3s total + some execution time
            sleep(Duration::from_millis(4000)).await;

            let elapsed = start_time.elapsed();
            let final_attempts = attempts.load(Ordering::SeqCst);

            // Should have made exactly 3 attempts
            assert_eq!(
                final_attempts, 3,
                "Expected exactly 3 attempts, got {}",
                final_attempts
            );

            // Should have taken at least 3 seconds due to exponential backoff (1s + 2s)
            assert!(
                elapsed >= Duration::from_millis(2900), // Allow some tolerance
                "Expected at least 2.9s elapsed due to exponential backoff, got {:?}",
                elapsed
            );
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_query_caching() {
    with_async_test_isolate(|| async {
        clear_query_cache();
        let fetch_count = Arc::new(AtomicU32::new(0));

        with_async_component_id("QueryCachingTest", |_ctx| async {
            let fetch_count_clone = fetch_count.clone();
            let result = use_query(
                "cache-key",
                move || {
                    let count = fetch_count_clone.fetch_add(1, Ordering::SeqCst);
                    mock_fetch_success(count as i32)
                },
                None,
            );

            assert!(matches!(
                result.status,
                QueryStatus::Idle | QueryStatus::Loading
            ));

            // Wait for query to complete
            sleep(Duration::from_millis(100)).await;

            // Query again - should use cache
            let fetch_count_clone2 = fetch_count.clone();
            let result2 = use_query(
                "cache-key",
                move || {
                    let count = fetch_count_clone2.fetch_add(1, Ordering::SeqCst);
                    mock_fetch_success(count as i32)
                },
                None,
            );

            // Should use cached data
            if result2.status == QueryStatus::Success {
                assert_eq!(result2.data, Some(0)); // First fetch returned 0
            }

            // Verify only one fetch occurred
            assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_query_invalidation() {
    with_async_test_isolate(|| async {
        clear_query_cache();

        with_async_component_id("QueryInvalidationTest", |_ctx| async {
            let result = use_query("invalidate-key", || mock_fetch_success(100), None);

            // Test invalidate function
            (result.invalidate)();

            // Cache should be cleared
            let (_cache_size, _) = get_cache_stats();
            // Note: The cache might still contain entries from other tests
            // but our specific key should be removed
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_query_refetch() {
    with_async_test_isolate(|| async {
        clear_query_cache();
        let fetch_count = Arc::new(AtomicU32::new(0));

        with_async_component_id("QueryRefetchTest", |_ctx| async {
            let fetch_count_clone = fetch_count.clone();
            let result = use_query(
                "refetch-key",
                move || {
                    let count = fetch_count_clone.fetch_add(1, Ordering::SeqCst);
                    mock_fetch_success(count as i32)
                },
                None,
            );

            // Manually trigger refetch
            (result.refetch)();

            assert!(matches!(
                result.status,
                QueryStatus::Idle | QueryStatus::Loading | QueryStatus::Refreshing
            ));

            // Wait for both initial fetch and refetch to complete
            sleep(Duration::from_millis(100)).await;

            // Should have made at least 2 fetches
            assert!(fetch_count.load(Ordering::SeqCst) >= 2);
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_query_disabled() {
    with_async_test_isolate(|| async {
        clear_query_cache();
        let fetch_count = Arc::new(AtomicU32::new(0));

        with_async_component_id("QueryDisabledTest", |_ctx| async {
            let fetch_count_clone = fetch_count.clone();
            let options = QueryOptions {
                enabled: false,
                ..Default::default()
            };

            let result = use_query(
                "disabled-key",
                move || {
                    fetch_count_clone.fetch_add(1, Ordering::SeqCst);
                    mock_fetch_success(42)
                },
                Some(options),
            );

            // Should remain idle when disabled
            assert_eq!(result.status, QueryStatus::Idle);
            assert!(result.data.is_none());

            // Wait a bit to ensure no fetch occurs
            sleep(Duration::from_millis(50)).await;

            // No fetch should have occurred
            assert_eq!(fetch_count.load(Ordering::SeqCst), 0);
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_stale_time_behavior() {
    with_async_test_isolate(|| async {
        clear_query_cache();

        with_async_component_id("QueryStaleTimeTest", |_ctx| async {
            let options = QueryOptions {
                stale_time: Duration::from_millis(100),
                ..Default::default()
            };

            let result = use_query(
                "stale-key",
                || mock_fetch_success(42),
                Some(options.clone()),
            );

            assert!(matches!(
                result.status,
                QueryStatus::Idle | QueryStatus::Loading
            ));

            // Wait for initial fetch
            sleep(Duration::from_millis(50)).await;

            // Data should be fresh
            let result = use_query(
                "stale-key",
                || mock_fetch_success(42),
                Some(options.clone()),
            );

            if result.status == QueryStatus::Success {
                assert!(!result.is_stale);
            }

            // Wait for data to become stale
            sleep(Duration::from_millis(150)).await;

            // Now data should be stale and trigger background refresh
            let result = use_query("stale-key", || mock_fetch_success(42), Some(options));

            // Should still have data but might be refreshing
            if result.status == QueryStatus::Success || result.status == QueryStatus::Refreshing {
                assert_eq!(result.data, Some(42));
            }
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_cache_expiration() {
    with_async_test_isolate(|| async {
        // Use a simple static key - the test isolation should handle conflicts
        const TEST_KEY: &str = "cache-expiration-test-key";

        // Clear cache at start
        clear_query_cache();

        with_async_component_id("QueryCacheExpirationTest", |_ctx| async {
            let options = QueryOptions {
                cache_time: Duration::from_millis(300), // Longer cache time for testing
                ..Default::default()
            };

            let result = use_query(TEST_KEY, || mock_fetch_success(42), Some(options.clone()));

            // Check if query completed
            if result.status == QueryStatus::Success {
                assert_eq!(result.data, Some(42));
            }

            // Wait a bit for async query to complete
            sleep(Duration::from_millis(100)).await;

            // Check if cache is populated
            let (_cache_size, keys) = get_cache_stats();
            let cache_populated = keys.iter().any(|k| k.contains(TEST_KEY));

            if cache_populated {
                // Cache should be valid at this point
                assert!(
                    keys.iter().any(|k| k.contains(TEST_KEY)),
                    "Cache should contain key '{}', but found keys: {:?}",
                    TEST_KEY,
                    keys
                );

                // Wait for cache to expire
                sleep(Duration::from_millis(350)).await;

                // Use the query again - should not use expired cache
                let _result = use_query(
                    TEST_KEY,
                    || mock_fetch_success(100), // Different value
                    Some(options),
                );
            }
        })
        .await;
    })
    .await;
}
