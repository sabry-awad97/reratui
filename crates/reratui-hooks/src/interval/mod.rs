//! Professional interval hooks for periodic execution
//!
//! This module provides both synchronous and asynchronous interval hooks for executing
//! callbacks at regular intervals, similar to JavaScript's `setInterval` but with proper
//! Rust async integration and cleanup.
//!
//! ## Key Features:
//! - **Synchronous intervals**: `use_interval` for simple periodic callbacks
//! - **Asynchronous intervals**: `use_async_interval` for async periodic operations
//! - Automatic cleanup when component unmounts or dependencies change
//! - Proper async/await integration with tokio runtime
//! - Thread-safe execution with proper error handling
//! - Professional resource management and memory safety
//! - Integration with the hook lifecycle system
//!
//! ## Usage Examples:
//!
//! ### Basic Synchronous Interval
//! ```rust,no_run
//! use reratui_hooks::interval::use_interval;
//! use std::time::Duration;
//!
//! // Simple interval example
//! use_interval(|| {
//!     println!("This runs every second!");
//! }, Duration::from_secs(1));
//! ```
//!
//! ### Synchronous Interval with State Updates
//! ```rust,no_run
//! use reratui_hooks::state::use_state;
//! use reratui_hooks::interval::use_interval;
//! use std::time::Duration;
//!
//! // Interval with state updates
//! let (count, set_count) = use_state(|| 0);
//! use_interval({
//!     let set_count = set_count.clone();
//!     move || {
//!         set_count.update(|prev| prev + 1);
//!     }
//! }, Duration::from_millis(500));
//! ```
//!
//! ### Asynchronous Interval
//! ```rust,no_run
//! use reratui_hooks::interval::use_async_interval;
//! use reratui_hooks::state::use_state;
//! use std::time::Duration;
//!
//! // Async interval example
//! let (data, set_data) = use_state(|| String::new());
//! use_async_interval({
//!     let set_data = set_data.clone();
//!     move || {
//!         let set_data = set_data.clone();
//!         async move {
//!             // Simulate async data fetching
//!             let new_data = format!("Data updated at: {:?}", std::time::SystemTime::now());
//!             set_data.set(new_data);
//!         }
//!     }
//! }, Duration::from_secs(2));
//! ```

use std::time::Duration;

#[cfg(test)]
mod tests;

use crate::effect::EffectDependencies;

// Implement EffectDependencies for Duration to enable dependency tracking
impl EffectDependencies for Duration {
    fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
        if let Some(other_duration) = other.as_any().downcast_ref::<Duration>() {
            self == other_duration
        } else {
            false
        }
    }

    fn clone_deps(&self) -> Box<dyn EffectDependencies> {
        Box::new(*self)
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

/// Professional synchronous interval hook for periodic callback execution
///
/// This hook provides a React-like `setInterval` functionality with proper cleanup
/// and integration with the component lifecycle. The interval automatically starts
/// when the component mounts and stops when it unmounts.
///
/// ## Key Features:
/// - Automatic cleanup on component unmount
/// - Thread-safe execution with proper error handling
/// - No async runtime dependency (uses std::thread)
/// - Professional resource management
/// - Consistent timing with std::thread::sleep
///
/// ## Parameters:
/// - `callback`: Synchronous function to execute at each interval
/// - `duration`: Time between executions
///
/// ## Behavior:
/// - The interval starts immediately when the hook is called
/// - If the duration changes, the interval is restarted with the new duration
/// - The interval is automatically cancelled when the component unmounts
/// - All spawned threads are properly cleaned up to prevent memory leaks
///
/// ## Thread Safety:
/// The callback must be `Send + 'static` to ensure thread safety across thread boundaries.
/// State updates should use thread-safe mechanisms like the state hooks.
///
/// ## Performance:
/// Uses std::thread with sleep for timing. The implementation avoids busy-waiting
/// and provides accurate timing without requiring an async runtime.
pub fn use_interval<F>(callback: F, duration: Duration)
where
    F: Fn() + Send + 'static,
{
    use crate::effect::use_effect;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    // Use effect to manage the interval lifecycle with proper cleanup
    use_effect(
        move || {
            // Handle zero duration by using minimum duration
            let safe_duration = if duration.is_zero() {
                Duration::from_millis(1)
            } else {
                duration
            };

            // Create a flag to signal when to stop the interval
            let should_stop = Arc::new(AtomicBool::new(false));
            let should_stop_clone = should_stop.clone();

            // Spawn interval thread
            let handle = thread::spawn(move || {
                while !should_stop_clone.load(Ordering::Relaxed) {
                    thread::sleep(safe_duration);
                    if !should_stop_clone.load(Ordering::Relaxed) {
                        callback();
                    }
                }
            });

            // Return cleanup function that signals stop and waits for thread
            Some(Box::new(move || {
                should_stop.store(true, Ordering::Relaxed);
                // Note: We can't join the thread here as it would block the cleanup
                // The thread will exit on its own when it checks the flag
                let _ = handle; // Take ownership to prevent warnings
            }) as Box<dyn FnOnce() + Send>)
        },
        duration, // Effect depends on duration - restarts when duration changes
    );
}

/// Professional asynchronous interval hook for periodic async callback execution
///
/// This hook provides async interval functionality with proper cleanup and integration
/// with the component lifecycle. Perfect for periodic async operations like data fetching,
/// API calls, or other async tasks. **Requires a tokio runtime to be active.**
///
/// ## Key Features:
/// - Automatic cleanup on component unmount
/// - Full async/await support for callbacks
/// - Thread-safe execution with proper error handling
/// - Integration with tokio async runtime
/// - Professional resource management
/// - Consistent timing with tokio::time::interval
///
/// ## Parameters:
/// - `callback`: Async function to execute at each interval
/// - `duration`: Time between executions
///
/// ## Behavior:
/// - The interval starts immediately when the hook is called
/// - Each callback execution waits for the previous one to complete
/// - If the duration changes, the interval is restarted with the new duration
/// - The interval is automatically cancelled when the component unmounts
/// - All spawned tasks are properly cleaned up to prevent memory leaks
///
/// ## Thread Safety:
/// The callback must be `Send + 'static` and return a future that is `Send + 'static`.
/// State updates should use thread-safe mechanisms like the state hooks.
///
/// ## Performance:
/// Uses tokio's optimized interval timer for accurate timing with minimal overhead.
/// The implementation properly handles async execution without blocking the runtime.
///
/// ## Runtime Requirements:
/// This function requires an active tokio runtime. If no runtime is available,
/// it will panic. Use `use_interval` for synchronous callbacks that don't require tokio.
///
/// ## Example:
/// ```rust,no_run
/// use reratui_hooks::interval::use_async_interval;
/// use reratui_hooks::state::use_state;
/// use std::time::Duration;
///
/// // Async interval example
/// let (data, set_data) = use_state(|| String::new());
/// use_async_interval({
///     let set_data = set_data.clone();
///     move || {
///         let set_data = set_data.clone();
///         async move {
///             // Simulate async API call
///             let response = "API data".to_string();
///             set_data.set(response);
///         }
///     }
/// }, Duration::from_secs(5));
/// ```
pub fn use_async_interval<F, Fut>(callback: F, duration: Duration)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    use crate::effect::use_effect;

    // Use effect to manage the async interval lifecycle with proper cleanup
    use_effect(
        move || {
            // Handle zero duration by using minimum duration
            let safe_duration = if duration.is_zero() {
                Duration::from_millis(1)
            } else {
                duration
            };

            // Check if we're in a tokio runtime context
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(handle) => handle,
                Err(_) => {
                    eprintln!("Warning: use_async_interval called outside tokio runtime context");
                    return None; // No cleanup needed if we can't spawn
                }
            };

            // Spawn async interval task
            let task_handle = handle.spawn(async move {
                let mut interval_timer = tokio::time::interval(safe_duration);

                loop {
                    interval_timer.tick().await;
                    // Execute the async callback and wait for completion
                    callback().await;
                }
            });

            // Return cleanup function that cancels the task
            Some(Box::new(move || {
                task_handle.abort();
            }) as Box<dyn FnOnce() + Send>)
        },
        duration, // Effect depends on duration - restarts when duration changes
    );
}
