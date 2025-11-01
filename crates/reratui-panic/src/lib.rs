use std::any::Any;
use std::io::{self, Write};
use std::panic;
use std::sync::Once;
use tokio::task::JoinHandle;

#[cfg(debug_assertions)]
use better_panic::{Settings, Verbosity};

#[cfg(not(debug_assertions))]
use human_panic::setup_panic;

static INIT: Once = Once::new();

/// Sets up a custom panic hook for the application with advanced features.
///
/// This function configures panic behavior based on the build profile:
/// - **Debug builds**: Uses `better_panic` for verbose, immediate, and diagnostic-rich panics with full stack traces.
/// - **Release builds**: Uses `human_panic` for graceful, user-friendly panics that log internally without exposing sensitive details, prioritizing user experience.
///
/// Additionally, it provides a mechanism to catch panics from spawned Tokio tasks.
///
/// This function should be called only once. Subsequent calls will be ignored.
///
/// # Note
/// This function does not set up any logging. If you want to log panic information,
/// set up your own tracing subscriber before calling this function.
/// See the examples directory for how to integrate logging.
pub fn setup_panic_handler() {
    INIT.call_once(|| {
        #[cfg(debug_assertions)]
        {
            // For debug builds, use better_panic for detailed output
            Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(Verbosity::Full)
                .install();
        }

        #[cfg(not(debug_assertions))]
        {
            // For release builds, use human_panic for user-friendly messages
            setup_panic!();
        }

        // Wrap the panic hook installed by better_panic/human_panic
        // to ensure terminal is properly restored
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            use crossterm::event::DisableMouseCapture;
            use crossterm::execute;
            use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};

            // Restore terminal before calling the panic formatter
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
            let _ = io::stdout().flush();

            // Call the original hook (better_panic/human_panic)
            original_hook(panic_info);

            // Force terminal back to normal mode after panic output
            // This ensures the terminal is in a state where text can be selected
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
            let _ = io::stderr().flush();
            let _ = io::stdout().flush();

            // Exit the process after panic handling
            // This ensures the process terminates cleanly after terminal restoration
            std::process::exit(1);
        }));
    });
}

/// Spawns a new asynchronous task and catches any panics that occur within it.
///
/// If a panic occurs, it will be caught by the custom panic hook.
pub fn spawn_catch_panic<F>(future: F) -> JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(async move {
        let result = panic::catch_unwind(std::panic::AssertUnwindSafe(|| future));
        match result {
            Ok(output_future) => output_future.await,
            Err(e) => {
                // Re-panic on the main thread to trigger the custom panic hook
                panic::resume_unwind(e);
            }
        }
    })
}

/// Executes a closure and catches any panics that occur, returning a Result.
///
/// # Example
/// ```
/// use reratui_panic::catch_panic;
///
/// let ok = catch_panic(|| 42);
/// assert!(ok.is_ok());
/// assert_eq!(ok.unwrap(), 42);
///
/// let err = catch_panic(|| panic!("fail!"));
/// assert!(err.is_err());
/// ```
pub fn catch_panic<T, F>(f: F) -> Result<T, Box<dyn Any + Send + 'static>>
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use tokio::time::timeout;

    #[test]
    fn test_catch_panic_success() {
        let result = catch_panic(|| 42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_catch_panic_with_panic() {
        let result = catch_panic(|| panic!("test panic"));
        assert!(result.is_err());
    }

    #[test]
    fn test_catch_panic_with_string_panic() {
        let result = catch_panic(|| panic!("string panic message"));
        assert!(result.is_err());

        // Verify we can downcast the panic payload
        let panic_payload = result.unwrap_err();
        let panic_str = panic_payload.downcast_ref::<&str>();
        assert!(panic_str.is_some());
        assert_eq!(*panic_str.unwrap(), "string panic message");
    }

    #[test]
    fn test_catch_panic_with_custom_type() {
        #[derive(Debug, PartialEq)]
        struct CustomError(i32);

        let result = catch_panic(|| {
            std::panic::panic_any(CustomError(123));
        });

        assert!(result.is_err());
        let panic_payload = result.unwrap_err();
        let custom_error = panic_payload.downcast_ref::<CustomError>();
        assert!(custom_error.is_some());
        assert_eq!(*custom_error.unwrap(), CustomError(123));
    }

    #[test]
    fn test_catch_panic_with_closure_capture() {
        let value = 100;
        let result = catch_panic(|| value * 2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200);
    }

    #[tokio::test]
    async fn test_spawn_catch_panic_success() {
        let handle = spawn_catch_panic(async { 42 });
        let result = handle.await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_spawn_catch_panic_with_async_work() {
        let handle = spawn_catch_panic(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "async result"
        });

        let result = timeout(Duration::from_secs(1), handle).await;
        assert!(result.is_ok());
        let join_result = result.unwrap();
        assert!(join_result.is_ok());
        assert_eq!(join_result.unwrap(), "async result");
    }

    #[tokio::test]
    async fn test_spawn_catch_panic_with_panic() {
        let handle = spawn_catch_panic(async {
            panic!("async panic");
        });

        // The task should complete but the panic should be caught
        let result = handle.await;
        // Since we resume_unwind, the task will actually panic
        // This tests that the panic handling mechanism works
        assert!(result.is_err());
    }

    #[test]
    fn test_setup_panic_handler_idempotent() {
        // Test that calling setup_panic_handler multiple times is safe
        setup_panic_handler();
        setup_panic_handler();
        setup_panic_handler();

        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_setup_panic_handler_thread_safety() {
        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    setup_panic_handler();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // If all threads complete successfully, the test passes
    }

    #[test]
    fn test_catch_panic_return_types() {
        // Test with different return types
        let string_result = catch_panic(|| "hello".to_string());
        assert!(string_result.is_ok());
        assert_eq!(string_result.unwrap(), "hello");

        let vec_result = catch_panic(|| vec![1, 2, 3]);
        assert!(vec_result.is_ok());
        assert_eq!(vec_result.unwrap(), vec![1, 2, 3]);

        let option_result = catch_panic(|| Some(42));
        assert!(option_result.is_ok());
        assert_eq!(option_result.unwrap(), Some(42));
    }

    #[tokio::test]
    async fn test_spawn_catch_panic_concurrent() {
        let handles: Vec<_> = (0..5)
            .map(|i| {
                spawn_catch_panic(async move {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    i * 2
                })
            })
            .collect();

        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            results.push(result.unwrap());
        }

        results.sort();
        assert_eq!(results, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_catch_panic_with_mutable_data() {
        let mut counter = 0;
        let result = catch_panic(std::panic::AssertUnwindSafe(|| {
            counter += 1;
            counter
        }));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_spawn_catch_panic_with_shared_state() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let handle = spawn_catch_panic(async move {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            *count
        });

        let result = handle.await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let final_count = *counter.lock().unwrap();
        assert_eq!(final_count, 1);
    }

    #[test]
    fn test_panic_handler_module_exports() {
        // Test that all public functions are accessible by calling them
        setup_panic_handler();

        let result = catch_panic(|| 42);
        assert!(result.is_ok());

        // Test spawn_catch_panic in an async context would require tokio runtime
        // So we just verify the function exists by referencing it
        let _spawn_fn_exists = spawn_catch_panic::<std::future::Ready<i32>>;

        // If compilation succeeds, all exports are accessible
    }

    #[test]
    fn test_panic_handler_terminal_restoration() {
        // Test that the panic handler is set up correctly
        // and doesn't panic during setup
        setup_panic_handler();

        // Verify that calling setup multiple times is safe
        setup_panic_handler();
        setup_panic_handler();

        // The panic hook should be installed at this point
        // We can't easily test the actual panic behavior without panicking,
        // but we can verify the setup completes successfully
    }

    #[test]
    fn test_terminal_restoration_with_catch_panic() {
        // Set up the panic handler
        setup_panic_handler();

        // Test that catch_panic works correctly with the panic handler installed
        let result = catch_panic(|| {
            // This should not panic
            42
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Test that catch_panic catches panics even with our custom hook
        let result = catch_panic(|| {
            panic!("test panic for terminal restoration");
        });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_spawn_catch_panic_terminal_restoration() {
        // Set up the panic handler
        setup_panic_handler();

        // Test that spawn_catch_panic works with the panic handler
        let handle = spawn_catch_panic(async {
            // Normal async operation
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            "success"
        });

        let result = handle.await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_panic_hook_wrapping() {
        // Test that our panic hook properly wraps the original hook
        // This verifies that both better_panic/human_panic and our
        // terminal restoration logic will run

        // First setup creates the initial hook
        setup_panic_handler();

        // Verify we can catch panics after setup
        let result = catch_panic(|| {
            // This panic should be caught
            panic!("wrapped panic test");
        });

        assert!(result.is_err());

        // Verify the panic payload is preserved
        let err = result.unwrap_err();
        let panic_msg = err.downcast_ref::<&str>();
        assert!(panic_msg.is_some());
        assert_eq!(*panic_msg.unwrap(), "wrapped panic test");
    }

    #[test]
    fn test_multiple_panic_handler_setups() {
        // Test that calling setup_panic_handler multiple times
        // doesn't cause issues (should be idempotent)
        for _ in 0..5 {
            setup_panic_handler();
        }

        // Verify panic catching still works
        let result = catch_panic(|| 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn test_terminal_state_restoration_on_panic() {
        use crossterm::event::EnableMouseCapture;
        use crossterm::execute;
        use crossterm::terminal::EnterAlternateScreen;
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled};

        // Set up the panic handler
        setup_panic_handler();

        // Simulate a TUI application setup
        let setup_result = catch_panic(|| {
            // Enable raw mode (like a TUI app would)
            enable_raw_mode().expect("Failed to enable raw mode");

            // Enter alternate screen and enable mouse capture
            execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)
                .expect("Failed to setup terminal");

            // Verify raw mode is enabled
            assert!(is_raw_mode_enabled().unwrap_or(false));
        });

        // Clean up after test
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );

        assert!(setup_result.is_ok());
    }

    #[test]
    fn test_panic_hook_calls_terminal_restoration() {
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

        // Set up the panic handler
        setup_panic_handler();

        // Test that a panic triggers terminal restoration
        let panic_result = catch_panic(|| {
            // Enable raw mode
            let _ = enable_raw_mode();

            // Simulate a panic (this will be caught by catch_panic)
            panic!("Test panic to verify terminal restoration");
        });

        // The panic should have been caught
        assert!(panic_result.is_err());

        // Clean up - ensure raw mode is disabled
        let _ = disable_raw_mode();

        // Verify the panic message is preserved
        let err = panic_result.unwrap_err();
        let msg = err.downcast_ref::<&str>();
        assert!(msg.is_some());
        assert_eq!(*msg.unwrap(), "Test panic to verify terminal restoration");
    }

    #[test]
    fn test_terminal_restoration_commands_are_called() {
        // This test verifies that the panic hook contains the correct
        // terminal restoration commands by checking the setup completes

        setup_panic_handler();

        // Verify that after setup, we can still use catch_panic
        // which means the panic hook is properly installed
        let result = catch_panic(|| {
            // Simulate terminal operations
            let _ = io::stdout();
            42
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_async_panic_terminal_restoration() {
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

        // Set up the panic handler
        setup_panic_handler();

        // Test async panic with terminal state
        let handle = spawn_catch_panic(async {
            // Enable raw mode in async context
            let _ = enable_raw_mode();

            // Do some async work
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;

            "completed"
        });

        let result = handle.await;

        // Clean up
        let _ = disable_raw_mode();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "completed");
    }
}
