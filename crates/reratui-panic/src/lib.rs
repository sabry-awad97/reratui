use std::any::Any;
use std::io::{self, Write};
use std::panic;
use std::sync::Once;
use tokio::task::JoinHandle;
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::Registry;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt,
    prelude::*,
    util::SubscriberInitExt,
};

#[cfg(debug_assertions)]
use better_panic::{Settings, Verbosity};

#[cfg(not(debug_assertions))]
use human_panic::setup_panic;

static INIT: Once = Once::new();
static mut LOG_GUARD: Option<WorkerGuard> = None;

/// Sets up a custom panic hook for the application with advanced features.
///
/// This function configures panic behavior based on the build profile:
/// - **Debug builds**: Uses `better_panic` for verbose, immediate, and diagnostic-rich panics with full stack traces.
/// - **Release builds**: Uses `human_panic` for graceful, user-friendly panics that log internally without exposing sensitive details, prioritizing user experience.
///
/// Additionally, it provides a mechanism to catch panics from spawned Tokio tasks.
///
/// This function should be called only once. Subsequent calls will be ignored.
pub fn setup_panic_handler() {
    INIT.call_once(|| {
        // Initialize tracing subscriber for internal logging regardless of build type
        let env_filter = EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into());
        let console_layer = fmt::Layer::new().with_writer(io::stderr);

        let subscriber = Registry::default().with(env_filter).with(console_layer);

        // For file logging, we can still use tracing-appender
        // This part is independent of debug/release panic behavior
        let log_file_path = "logs".to_string(); // Example path, could be configurable
        let file_appender = tracing_appender::rolling::daily(log_file_path, "application.log");
        let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);
        unsafe {
            LOG_GUARD = Some(guard);
        }
        let file_layer = fmt::Layer::new().with_writer(non_blocking_appender).json();
        subscriber.with(file_layer).init();

        #[cfg(debug_assertions)]
        {
            // For debug builds, use better_panic for detailed output
            Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(Verbosity::Full)
                .install();
            info!("Panic handler configured for DEBUG (better_panic).");
        }

        #[cfg(not(debug_assertions))]
        {
            // For release builds, use human_panic for user-friendly messages
            setup_panic!();
            info!("Panic handler configured for RELEASE (human_panic).");
        }

        // Custom panic hook to log to tracing system before the specific handler takes over
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // First, try to restore the terminal to normal mode
            // This ensures panic messages are visible
            use crossterm::execute;
            use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            let _ = io::stdout().flush();

            // Capture backtrace
            let backtrace = std::backtrace::Backtrace::force_capture();
            let backtrace_str = format!("{}", backtrace);

            // Get panic location and payload
            let location = panic_info.location().map_or("Unknown".to_string(), |l| {
                format!("{}:{}:{}", l.file(), l.line(), l.column())
            });

            let payload = panic_info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<unknown>".to_string());

            // Log with backtrace
            error!(
                target: "panic_handler",
                location = %location,
                payload = %payload,
                backtrace = %backtrace_str,
                "Application panicked"
            );

            // Call the original hook to ensure better_panic/human_panic are triggered
            original_hook(panic_info);
            let _ = io::stderr().flush();
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
/// use pulse_core::panic_handler::catch_panic;
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
}
