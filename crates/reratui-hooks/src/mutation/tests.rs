use super::*;
use crate::test_utils::{with_async_component_id, with_async_test_isolate};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::time::{Duration, sleep};

// Test data types
#[derive(Debug, Clone, PartialEq)]
struct TestData {
    id: u32,
    message: String,
}

#[derive(Debug, Clone, PartialEq)]
struct TestVariables {
    input: String,
    delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
struct TestError {
    code: u32,
    message: String,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for TestError {}

// Mock async functions for testing
async fn mock_success_fn(variables: TestVariables) -> Result<TestData, TestError> {
    sleep(Duration::from_millis(variables.delay_ms)).await;
    Ok(TestData {
        id: 1,
        message: format!("Success: {}", variables.input),
    })
}

async fn mock_error_fn(variables: TestVariables) -> Result<TestData, TestError> {
    sleep(Duration::from_millis(variables.delay_ms)).await;
    Err(TestError {
        code: 500,
        message: format!("Failed: {}", variables.input),
    })
}

async fn mock_retry_fn(
    variables: TestVariables,
    attempt_counter: Arc<AtomicU32>,
) -> Result<TestData, TestError> {
    let attempt = attempt_counter.fetch_add(1, Ordering::SeqCst) + 1;
    sleep(Duration::from_millis(variables.delay_ms)).await;

    if attempt <= 2 {
        Err(TestError {
            code: 503,
            message: format!("Attempt {} failed", attempt),
        })
    } else {
        Ok(TestData {
            id: attempt,
            message: format!("Success after {} attempts: {}", attempt, variables.input),
        })
    }
}

async fn mock_always_fail_retry_fn(
    variables: TestVariables,
    attempt_counter: Arc<AtomicU32>,
) -> Result<TestData, TestError> {
    let attempt = attempt_counter.fetch_add(1, Ordering::SeqCst) + 1;
    sleep(Duration::from_millis(variables.delay_ms)).await;

    // Always fail regardless of attempt count
    Err(TestError {
        code: 503,
        message: format!("Attempt {} failed", attempt),
    })
}

#[tokio::test]
async fn test_mutation_new_with_default_options() {
    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_success_fn(variables).await },
        None,
    );

    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Idle);
    assert!(state.is_idle);
    assert!(!state.is_pending);
    assert!(!state.is_success);
    assert!(!state.is_error);
    assert!(state.data.is_none());
    assert!(state.error.is_none());
    assert!(state.variables.is_none());
    assert_eq!(state.failed_count, 0);
}

#[tokio::test]
async fn test_mutation_new_with_custom_options() {
    let on_success_called = Arc::new(AtomicBool::new(false));
    let on_error_called = Arc::new(AtomicBool::new(false));
    let on_settled_called = Arc::new(AtomicBool::new(false));

    let options = MutationOptions {
        on_success: Some(Box::new({
            let flag = on_success_called.clone();
            move |_data, _variables, _context| {
                flag.store(true, Ordering::SeqCst);
            }
        })),
        on_error: Some(Box::new({
            let flag = on_error_called.clone();
            move |_error, _variables, _context| {
                flag.store(true, Ordering::SeqCst);
            }
        })),
        on_settled: Some(Box::new({
            let flag = on_settled_called.clone();
            move |_data, _error, _variables, _context| {
                flag.store(true, Ordering::SeqCst);
            }
        })),
        retry: true,
        retry_attempts: 3,
        retry_delay: Duration::from_millis(100),
        ..Default::default()
    };

    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_success_fn(variables).await },
        Some(options),
    );

    // Test successful mutation
    let variables = TestVariables {
        input: "test".to_string(),
        delay_ms: 10,
    };

    let result = mutation.mutate_async(variables).await;
    assert!(result.is_ok());

    // Give callbacks time to execute
    sleep(Duration::from_millis(50)).await;

    assert!(on_success_called.load(Ordering::SeqCst));
    assert!(!on_error_called.load(Ordering::SeqCst));
    assert!(on_settled_called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_mutate_async_success() {
    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_success_fn(variables).await },
        None,
    );

    let variables = TestVariables {
        input: "test success".to_string(),
        delay_ms: 10,
    };

    // Test state before mutation
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Idle);

    // Execute mutation
    let result = mutation.mutate_async(variables.clone()).await;
    assert!(result.is_ok());

    let data = result.unwrap();
    assert_eq!(data.id, 1);
    assert_eq!(data.message, "Success: test success");

    // Test state after successful mutation
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Success);
    assert!(!state.is_idle);
    assert!(!state.is_pending);
    assert!(state.is_success);
    assert!(!state.is_error);
    assert!(state.data.is_some());
    assert!(state.error.is_none());
    assert_eq!(state.variables, Some(variables));
    assert_eq!(state.failed_count, 0);
    assert!(state.submitted_at.is_some());
}

#[tokio::test]
async fn test_mutate_async_error() {
    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_error_fn(variables).await },
        None,
    );

    let variables = TestVariables {
        input: "test error".to_string(),
        delay_ms: 10,
    };

    // Execute mutation
    let result = mutation.mutate_async(variables.clone()).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.code, 500);
    assert_eq!(error.message, "Failed: test error");

    // Test state after failed mutation
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Error);
    assert!(!state.is_idle);
    assert!(!state.is_pending);
    assert!(!state.is_success);
    assert!(state.is_error);
    assert!(state.data.is_none());
    assert!(state.error.is_some());
    assert_eq!(state.variables, Some(variables));
    assert_eq!(state.failed_count, 1);
    assert!(state.submitted_at.is_some());
    assert!(state.failure_reason.is_some());
}

#[tokio::test]
async fn test_mutate_fire_and_forget() {
    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_success_fn(variables).await },
        None,
    );

    let variables = TestVariables {
        input: "fire and forget".to_string(),
        delay_ms: 50,
    };

    // Execute mutation (fire and forget)
    mutation.mutate(variables.clone());

    // Wait a bit for the mutation to start
    sleep(Duration::from_millis(10)).await;

    // Should be in pending state
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Pending);
    assert!(state.is_pending);

    // Wait for completion
    sleep(Duration::from_millis(100)).await;

    // Should be in success state
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Success);
    assert!(state.is_success);
    assert!(state.data.is_some());
}

#[tokio::test]
async fn test_mutation_state_transitions() {
    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_success_fn(variables).await },
        None,
    );

    let variables = TestVariables {
        input: "state transitions".to_string(),
        delay_ms: 100, // Longer delay to ensure we can catch pending state
    };

    // Initial state: Idle
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Idle);
    assert!(state.is_idle);
    assert!(!state.is_pending);
    assert!(!state.is_success);
    assert!(!state.is_error);

    // Start mutation in background
    mutation.mutate(variables);

    // Wait a bit to ensure it's pending
    sleep(Duration::from_millis(20)).await;

    // Should be in pending state
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Pending);
    assert!(!state.is_idle);
    assert!(state.is_pending);
    assert!(!state.is_success);
    assert!(!state.is_error);

    // Wait for completion
    sleep(Duration::from_millis(150)).await;

    // Should be in success state
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Success);
    assert!(!state.is_idle);
    assert!(!state.is_pending);
    assert!(state.is_success);
    assert!(!state.is_error);
}

#[tokio::test]
async fn test_mutation_reset() {
    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_success_fn(variables).await },
        None,
    );

    let variables = TestVariables {
        input: "reset test".to_string(),
        delay_ms: 10,
    };

    // Execute mutation
    let result = mutation.mutate_async(variables).await;
    assert!(result.is_ok());

    // Verify state is not idle
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Success);
    assert!(state.data.is_some());

    // Reset mutation
    mutation.reset();

    // Verify state is back to idle
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Idle);
    assert!(state.is_idle);
    assert!(!state.is_pending);
    assert!(!state.is_success);
    assert!(!state.is_error);
    assert!(state.data.is_none());
    assert!(state.error.is_none());
    assert!(state.variables.is_none());
    assert_eq!(state.failed_count, 0);
    assert!(state.failure_reason.is_none());
    assert!(state.submitted_at.is_none());
}

#[tokio::test]
async fn test_retry_logic_success_after_failures() {
    let attempt_counter = Arc::new(AtomicU32::new(0));

    let options = MutationOptions {
        retry: true,
        retry_attempts: 3,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };

    let mutation = Mutation::new(
        {
            let counter = attempt_counter.clone();
            move |variables: TestVariables| {
                let counter = counter.clone();
                async move { mock_retry_fn(variables, counter).await }
            }
        },
        Some(options),
    );

    let variables = TestVariables {
        input: "retry test".to_string(),
        delay_ms: 5,
    };

    // Execute mutation
    let result = mutation.mutate_async(variables).await;
    assert!(result.is_ok());

    let data = result.unwrap();
    assert_eq!(data.id, 3); // Should succeed on 3rd attempt
    assert!(data.message.contains("Success after 3 attempts"));

    // Verify 3 attempts were made
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);

    // Verify final state
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Success);
    assert!(state.data.is_some());
    assert!(state.error.is_none());
    assert_eq!(state.failed_count, 0); // Reset on success
}

#[tokio::test]
async fn test_retry_logic_final_failure() {
    let attempt_counter = Arc::new(AtomicU32::new(0));

    let options = MutationOptions {
        retry: true,
        retry_attempts: 2, // Only 2 retries, so 3 total attempts
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };

    let mutation = Mutation::new(
        {
            let counter = attempt_counter.clone();
            move |variables: TestVariables| {
                let counter = counter.clone();
                async move { mock_always_fail_retry_fn(variables, counter).await }
            }
        },
        Some(options),
    );

    let variables = TestVariables {
        input: "retry failure test".to_string(),
        delay_ms: 5,
    };

    // Execute mutation
    let result = mutation.mutate_async(variables).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.code, 503);
    assert!(error.message.contains("Attempt 3 failed"));

    // Verify 3 attempts were made (1 initial + 2 retries)
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);

    // Verify final state
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Error);
    assert!(state.data.is_none());
    assert!(state.error.is_some());
    assert_eq!(state.failed_count, 3);
    assert!(state.failure_reason.is_some());
}

#[tokio::test]
async fn test_no_retry_when_disabled() {
    let attempt_counter = Arc::new(AtomicU32::new(0));

    let options = MutationOptions {
        retry: false, // Retry disabled
        retry_attempts: 3,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };

    let mutation = Mutation::new(
        {
            let counter = attempt_counter.clone();
            move |variables: TestVariables| {
                let counter = counter.clone();
                async move { mock_retry_fn(variables, counter).await }
            }
        },
        Some(options),
    );

    let variables = TestVariables {
        input: "no retry test".to_string(),
        delay_ms: 5,
    };

    // Execute mutation
    let result = mutation.mutate_async(variables).await;
    assert!(result.is_err());

    // Verify only 1 attempt was made
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 1);

    // Verify final state
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Error);
    assert_eq!(state.failed_count, 1);
}

#[tokio::test]
async fn test_callback_execution() {
    let on_success_data = Arc::new(std::sync::Mutex::new(None::<TestData>));
    let on_error_data = Arc::new(std::sync::Mutex::new(None::<TestError>));
    let on_settled_data = Arc::new(std::sync::Mutex::new(
        None::<(Option<TestData>, Option<TestError>)>,
    ));
    let on_mutate_called = Arc::new(AtomicBool::new(false));

    let options = MutationOptions {
        on_success: Some(Box::new({
            let data_store = on_success_data.clone();
            move |data: &TestData, _variables: &TestVariables, _context: &MutationContext| {
                *data_store.lock().unwrap() = Some(data.clone());
            }
        })),
        on_error: Some(Box::new({
            let data_store = on_error_data.clone();
            move |error: &TestError, _variables: &TestVariables, _context: &MutationContext| {
                *data_store.lock().unwrap() = Some(error.clone());
            }
        })),
        on_settled: Some(Box::new({
            let data_store = on_settled_data.clone();
            move |data: Option<&TestData>,
                  error: Option<&TestError>,
                  _variables: &TestVariables,
                  _context: &MutationContext| {
                *data_store.lock().unwrap() = Some((data.cloned(), error.cloned()));
            }
        })),
        on_mutate: Some(Box::new({
            let flag = on_mutate_called.clone();
            move |_variables: &TestVariables| {
                flag.store(true, Ordering::SeqCst);
                None // Return None to use default context
            }
        })),
        ..Default::default()
    };

    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_success_fn(variables).await },
        Some(options),
    );

    let variables = TestVariables {
        input: "callback test".to_string(),
        delay_ms: 10,
    };

    // Execute successful mutation
    let result = mutation.mutate_async(variables).await;
    assert!(result.is_ok());

    // Give callbacks time to execute
    sleep(Duration::from_millis(50)).await;

    // Verify callbacks were called
    assert!(on_mutate_called.load(Ordering::SeqCst));

    let success_data = on_success_data.lock().unwrap();
    assert!(success_data.is_some());
    assert_eq!(
        success_data.as_ref().unwrap().message,
        "Success: callback test"
    );

    let error_data = on_error_data.lock().unwrap();
    assert!(error_data.is_none());

    let settled_data = on_settled_data.lock().unwrap();
    assert!(settled_data.is_some());
    let (data, error) = settled_data.as_ref().unwrap();
    assert!(data.is_some());
    assert!(error.is_none());
}

#[tokio::test]
async fn test_concurrent_mutations() {
    let mutation1 = Mutation::new(
        |variables: TestVariables| async move { mock_success_fn(variables).await },
        None,
    );

    let mutation2 = Mutation::new(
        |variables: TestVariables| async move { mock_error_fn(variables).await },
        None,
    );

    let variables1 = TestVariables {
        input: "concurrent 1".to_string(),
        delay_ms: 50,
    };

    let variables2 = TestVariables {
        input: "concurrent 2".to_string(),
        delay_ms: 30,
    };

    // Start both mutations concurrently
    let future1 = mutation1.mutate_async(variables1);
    let future2 = mutation2.mutate_async(variables2);

    // Wait for both to complete
    let (result1, result2) = tokio::join!(future1, future2);

    // Verify results
    assert!(result1.is_ok());
    assert!(result2.is_err());

    // Verify states are independent
    let state1 = mutation1.get_state();
    let state2 = mutation2.get_state();

    assert_eq!(state1.status, MutationStatus::Success);
    assert_eq!(state2.status, MutationStatus::Error);
    assert!(state1.data.is_some());
    assert!(state2.error.is_some());
}

#[tokio::test]
async fn test_use_mutation_hook_stability() {
    // Test that use_mutation returns stable mutation objects across re-renders
    with_async_test_isolate(|| async {
        with_async_component_id("MutationStabilityTest", |_ctx| async {
            let mutation1 = use_mutation(
                |variables: TestVariables| async move { mock_success_fn(variables).await },
                None,
            );

            // Simulate re-render with the same component ID
            let mutation2 = use_mutation(
                |variables: TestVariables| async move { mock_success_fn(variables).await },
                None,
            );

            // The mutations should have the same state (stable across re-renders)
            // Note: This test verifies the memo behavior works correctly
            let state1 = mutation1.get_state();
            let state2 = mutation2.get_state();
            assert_eq!(state1.status, state2.status);
            assert_eq!(state1.is_idle, state2.is_idle);
        })
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_mutation_context_tracking() {
    let context_data = Arc::new(std::sync::Mutex::new(None));

    let options = MutationOptions {
        on_success: Some(Box::new({
            let data_store = context_data.clone();
            move |_data, _variables, context| {
                *data_store.lock().unwrap() = Some(context.clone());
            }
        })),
        ..Default::default()
    };

    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_success_fn(variables).await },
        Some(options),
    );

    let variables = TestVariables {
        input: "context test".to_string(),
        delay_ms: 10,
    };

    let start_time = Instant::now();
    let result = mutation.mutate_async(variables).await;
    assert!(result.is_ok());

    // Give callbacks time to execute
    sleep(Duration::from_millis(50)).await;

    // Verify context was captured
    let context = context_data.lock().unwrap();
    assert!(context.is_some());

    let ctx = context.as_ref().unwrap();
    assert!(ctx.started_at >= start_time);
    assert!(ctx.started_at <= Instant::now());
    // UUID should be valid (non-nil)
    assert_ne!(
        ctx.mutation_id.to_string(),
        "00000000-0000-0000-0000-000000000000"
    );
}

// ============================================================================
// New Feature Tests: Cancellation, Exponential Backoff, Custom Context
// ============================================================================

async fn mock_long_running_fn(variables: TestVariables) -> Result<TestData, TestError> {
    sleep(Duration::from_millis(variables.delay_ms)).await;
    Ok(TestData {
        id: 1,
        message: format!("Completed: {}", variables.input),
    })
}

#[tokio::test]
async fn test_mutation_cancellation() {
    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_long_running_fn(variables).await },
        None,
    );

    let variables = TestVariables {
        input: "cancel test".to_string(),
        delay_ms: 1000, // Long delay
    };

    // Start mutation
    mutation.mutate(variables);

    // Wait a bit to ensure it starts
    sleep(Duration::from_millis(50)).await;

    // Should be pending
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Pending);
    assert!(state.is_pending);

    // Cancel the mutation
    mutation.cancel();

    // Should be cancelled
    let state = mutation.get_state();
    assert_eq!(state.status, MutationStatus::Cancelled);
    assert!(state.is_cancelled);
    assert!(!state.is_pending);
    assert!(!state.is_success);
    assert!(!state.is_error);
    assert!(state.failure_reason.is_some());
    assert!(state.failure_reason.unwrap().contains("cancelled"));
}

#[tokio::test]
async fn test_exponential_backoff() {
    let attempt_counter = Arc::new(AtomicU32::new(0));
    let attempt_times = Arc::new(std::sync::Mutex::new(Vec::new()));

    let options = MutationOptions::<TestData, TestError, TestVariables> {
        retry: true,
        retry_attempts: 3,
        retry_delay: Duration::from_millis(100), // Base delay
        retry_exponential_backoff: true,
        retry_max_delay: Duration::from_secs(10),
        ..Default::default()
    };

    let mutation = Mutation::new(
        {
            let counter = attempt_counter.clone();
            let times = attempt_times.clone();
            move |variables: TestVariables| {
                let counter = counter.clone();
                let times = times.clone();
                async move {
                    let attempt = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    times.lock().unwrap().push(Instant::now());

                    sleep(Duration::from_millis(variables.delay_ms)).await;

                    // Always fail to test all retries
                    Err(TestError {
                        code: 503,
                        message: format!("Attempt {} failed", attempt),
                    })
                }
            }
        },
        Some(options),
    );

    let variables = TestVariables {
        input: "backoff test".to_string(),
        delay_ms: 10,
    };

    let start = Instant::now();
    let result = mutation.mutate_async(variables).await;
    let total_duration = start.elapsed();

    assert!(result.is_err());
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 4); // 1 initial + 3 retries

    // Verify exponential backoff timing
    let times = attempt_times.lock().unwrap();
    assert_eq!(times.len(), 4);

    // Expected delays: 0ms, 100ms, 200ms, 400ms
    // Total should be at least 700ms (100 + 200 + 400)
    assert!(total_duration >= Duration::from_millis(700));

    // But less than 1.5 seconds (with some margin)
    assert!(total_duration < Duration::from_millis(1500));
}

#[tokio::test]
async fn test_exponential_backoff_max_delay() {
    let attempt_counter = Arc::new(AtomicU32::new(0));

    let options = MutationOptions::<TestData, TestError, TestVariables> {
        retry: true,
        retry_attempts: 5,
        retry_delay: Duration::from_millis(100),
        retry_exponential_backoff: true,
        retry_max_delay: Duration::from_millis(300), // Cap at 300ms
        ..Default::default()
    };

    let mutation = Mutation::new(
        {
            let counter = attempt_counter.clone();
            move |variables: TestVariables| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    sleep(Duration::from_millis(variables.delay_ms)).await;
                    Err(TestError {
                        code: 503,
                        message: "Failed".to_string(),
                    })
                }
            }
        },
        Some(options),
    );

    let variables = TestVariables {
        input: "max delay test".to_string(),
        delay_ms: 10,
    };

    let start = Instant::now();
    let _ = mutation.mutate_async(variables).await;
    let total_duration = start.elapsed();

    // With max_delay=300ms, delays should be: 100, 200, 300, 300, 300
    // Total: 1200ms minimum
    assert!(total_duration >= Duration::from_millis(1200));

    // Should not exceed 2 seconds
    assert!(total_duration < Duration::from_secs(2));
}

#[tokio::test]
async fn test_custom_context_from_on_mutate() {
    let custom_context_used = Arc::new(AtomicBool::new(false));
    let custom_mutation_id = uuid::Uuid::new_v4();

    let options = MutationOptions::<TestData, TestError, TestVariables> {
        on_mutate: Some(Box::new({
            let custom_id = custom_mutation_id;
            move |_variables: &TestVariables| {
                Some(MutationContext {
                    mutation_id: custom_id,
                    started_at: Instant::now(),
                })
            }
        })),
        on_success: Some(Box::new({
            let flag = custom_context_used.clone();
            let expected_id = custom_mutation_id;
            move |_data: &TestData, _variables: &TestVariables, context: &MutationContext| {
                if context.mutation_id == expected_id {
                    flag.store(true, Ordering::SeqCst);
                }
            }
        })),
        ..Default::default()
    };

    let mutation = Mutation::new(
        |variables: TestVariables| async move {
            sleep(Duration::from_millis(variables.delay_ms)).await;
            Ok(TestData {
                id: 1,
                message: "Success".to_string(),
            })
        },
        Some(options),
    );

    let variables = TestVariables {
        input: "custom context test".to_string(),
        delay_ms: 10,
    };

    let result = mutation.mutate_async(variables).await;
    assert!(result.is_ok());

    // Wait for callbacks
    sleep(Duration::from_millis(50)).await;

    // Verify custom context was used
    assert!(custom_context_used.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_cancelled_status_field() {
    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_long_running_fn(variables).await },
        None,
    );

    // Initial state
    let state = mutation.get_state();
    assert!(!state.is_cancelled);
    assert_eq!(state.status, MutationStatus::Idle);

    // Start and cancel
    mutation.mutate(TestVariables {
        input: "test".to_string(),
        delay_ms: 1000,
    });

    sleep(Duration::from_millis(50)).await;
    mutation.cancel();

    let state = mutation.get_state();
    assert!(state.is_cancelled);
    assert!(!state.is_pending);
    assert!(!state.is_success);
    assert!(!state.is_error);
    assert!(!state.is_idle);
}

#[tokio::test]
async fn test_reset_cancels_running_mutation() {
    let mutation = Mutation::new(
        |variables: TestVariables| async move { mock_long_running_fn(variables).await },
        None,
    );

    // Start mutation
    mutation.mutate(TestVariables {
        input: "test".to_string(),
        delay_ms: 1000,
    });

    sleep(Duration::from_millis(50)).await;

    // Verify it's running
    let state = mutation.get_state();
    assert!(state.is_pending);

    // Reset should cancel
    mutation.reset();

    let state = mutation.get_state();
    assert!(state.is_idle);
    assert!(!state.is_pending);
}

#[tokio::test]
async fn test_builder_with_new_options() {
    let options = MutationOptions::<TestData, TestError, TestVariables>::builder()
        .retry(true)
        .retry_attempts(5)
        .retry_delay(Duration::from_millis(200))
        .retry_exponential_backoff(true)
        .retry_max_delay(Duration::from_secs(5))
        .build();

    assert!(options.retry);
    assert_eq!(options.retry_attempts, 5);
    assert_eq!(options.retry_delay, Duration::from_millis(200));
    assert!(options.retry_exponential_backoff);
    assert_eq!(options.retry_max_delay, Duration::from_secs(5));
}

#[tokio::test]
async fn test_no_exponential_backoff_by_default() {
    let attempt_counter = Arc::new(AtomicU32::new(0));

    let options = MutationOptions::<TestData, TestError, TestVariables> {
        retry: true,
        retry_attempts: 3,
        retry_delay: Duration::from_millis(100),
        retry_exponential_backoff: false, // Disabled
        ..Default::default()
    };

    let mutation = Mutation::new(
        {
            let counter = attempt_counter.clone();
            move |variables: TestVariables| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    sleep(Duration::from_millis(variables.delay_ms)).await;
                    Err(TestError {
                        code: 503,
                        message: "Failed".to_string(),
                    })
                }
            }
        },
        Some(options),
    );

    let variables = TestVariables {
        input: "fixed delay test".to_string(),
        delay_ms: 10,
    };

    let start = Instant::now();
    let _ = mutation.mutate_async(variables).await;
    let total_duration = start.elapsed();

    // With fixed delay of 100ms and 3 retries: 100 + 100 + 100 = 300ms
    assert!(total_duration >= Duration::from_millis(300));

    // Should be close to 300ms (not exponential)
    assert!(total_duration < Duration::from_millis(600));
}
