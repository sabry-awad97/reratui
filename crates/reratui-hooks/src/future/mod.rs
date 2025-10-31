use parking_lot::{Mutex, RwLock};
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::task::JoinHandle;

#[cfg(test)]
mod tests;

use crate::effect::EffectDependencies;
use crate::hook_context::with_hook_context;
use crate::panic_handler::spawn_catch_panic;

/// Default error type for futures - provides good ergonomics for most use cases
pub type DefaultError = Box<dyn std::error::Error + Send + Sync>;

/// Type alias for FutureHandle with default error type
pub type DefaultFutureHandle<T> = FutureHandle<T, DefaultError>;

/// Progress callback type for reporting future progress
///
/// This callback receives a progress value between 0.0 and 1.0
pub type ProgressCallback = Arc<dyn Fn(f32) + Send + Sync>;

/// Maximum number of concurrent futures allowed per component
/// This prevents resource exhaustion attacks
const MAX_CONCURRENT_FUTURES_PER_COMPONENT: usize = 50;

/// Global counter for tracking total active futures across all components
/// This provides system-wide protection against resource exhaustion
static GLOBAL_ACTIVE_FUTURES: AtomicUsize = AtomicUsize::new(0);

/// Maximum total concurrent futures across the entire application
const MAX_GLOBAL_CONCURRENT_FUTURES: usize = 1000;

/// Represents the current state of a future operation
///
/// This enum provides a comprehensive view of the future's lifecycle,
/// similar to Promise states in JavaScript or Result types in Rust.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum FutureState<T, E = Box<dyn std::error::Error + Send + Sync>> {
    /// The future has not been started yet (lazy/manual trigger)
    ///
    /// This state is useful for futures that need explicit triggering
    /// or lazy initialization. Different from Pending which means "in progress".
    #[default]
    Idle,
    /// The future is currently pending (actively running)
    Pending,
    /// The future has resolved successfully with a value
    Resolved(T),
    /// The future has failed with an error
    Error(E),
}

impl<T, E> FutureState<T, E> {
    /// Returns true if the future has not been started yet
    pub fn is_idle(&self) -> bool {
        matches!(self, FutureState::Idle)
    }

    /// Returns true if the future is currently pending (actively running)
    pub fn is_pending(&self) -> bool {
        matches!(self, FutureState::Pending)
    }

    /// Returns true if the future has resolved successfully
    pub fn is_resolved(&self) -> bool {
        matches!(self, FutureState::Resolved(_))
    }

    /// Returns true if the future has failed with an error
    pub fn is_error(&self) -> bool {
        matches!(self, FutureState::Error(_))
    }

    /// Returns the resolved value if available, otherwise None
    pub fn value(&self) -> Option<&T> {
        match self {
            FutureState::Resolved(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the error if available, otherwise None
    pub fn error(&self) -> Option<&E> {
        match self {
            FutureState::Error(error) => Some(error),
            _ => None,
        }
    }

    /// Maps the resolved value to a new type using the provided function
    pub fn map<U, F>(self, f: F) -> FutureState<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            FutureState::Idle => FutureState::Idle,
            FutureState::Pending => FutureState::Pending,
            FutureState::Resolved(value) => FutureState::Resolved(f(value)),
            FutureState::Error(error) => FutureState::Error(error),
        }
    }

    /// Maps the error to a new type using the provided function
    pub fn map_err<F, G>(self, f: F) -> FutureState<T, G>
    where
        F: FnOnce(E) -> G,
    {
        match self {
            FutureState::Idle => FutureState::Idle,
            FutureState::Pending => FutureState::Pending,
            FutureState::Resolved(value) => FutureState::Resolved(value),
            FutureState::Error(error) => FutureState::Error(f(error)),
        }
    }
}

/// Enhanced API: Convert Result directly to FutureState
///
/// This provides better ergonomics when working with Results,
/// allowing direct conversion without manual pattern matching.
impl<T, E> From<Result<T, E>> for FutureState<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => FutureState::Resolved(value),
            Err(error) => FutureState::Error(error),
        }
    }
}

/// A handle to a future operation that provides access to its current state
///
/// This handle allows components to read the current state of an async operation
/// and provides utility methods for working with the future's result.
#[derive(Debug, Default)]
pub struct FutureHandle<T, E = Box<dyn std::error::Error + Send + Sync>> {
    /// Reference to the shared future state container
    /// Using RwLock for better read performance since state is read frequently
    state: Arc<RwLock<FutureState<T, E>>>,
    /// Handle to the running task (for cancellation)
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl<T, E> FutureHandle<T, E>
where
    T: Clone,
    E: Clone,
{
    /// Create a new future handle with initial idle state
    fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(FutureState::Idle)),
            task_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the current state of the future
    pub fn state(&self) -> FutureState<T, E> {
        self.state.read().clone()
    }

    /// Returns true if the future has not been started yet
    ///
    /// This method is optimized to avoid cloning the entire state.
    /// Uses read lock for better performance in concurrent scenarios.
    pub fn is_idle(&self) -> bool {
        matches!(&*self.state.read(), FutureState::Idle)
    }

    /// Returns true if the future is currently pending (actively running)
    ///
    /// This method is optimized to avoid cloning the entire state.
    /// Uses read lock for better performance in concurrent scenarios.
    pub fn is_pending(&self) -> bool {
        matches!(&*self.state.read(), FutureState::Pending)
    }

    /// Returns true if the future has resolved successfully
    ///
    /// This method is optimized to avoid cloning the entire state.
    /// Uses read lock for better performance in concurrent scenarios.
    pub fn is_resolved(&self) -> bool {
        matches!(&*self.state.read(), FutureState::Resolved(_))
    }

    /// Returns true if the future has failed with an error
    ///
    /// This method is optimized to avoid cloning the entire state.
    /// Uses read lock for better performance in concurrent scenarios.
    pub fn is_error(&self) -> bool {
        matches!(&*self.state.read(), FutureState::Error(_))
    }

    /// Returns the resolved value if available, otherwise None
    ///
    /// This method is optimized to avoid unnecessary cloning of the entire state.
    /// It directly accesses the state and clones only the value if present.
    /// Uses read lock for better performance in concurrent scenarios.
    pub fn value(&self) -> Option<T> {
        match &*self.state.read() {
            FutureState::Resolved(value) => Some(value.clone()),
            _ => None,
        }
    }

    /// Returns the error if available, otherwise None
    ///
    /// This method is optimized to avoid unnecessary cloning of the entire state.
    /// It directly accesses the state and clones only the error if present.
    /// Uses read lock for better performance in concurrent scenarios.
    pub fn error(&self) -> Option<E> {
        match &*self.state.read() {
            FutureState::Error(error) => Some(error.clone()),
            _ => None,
        }
    }

    /// Cancel the running future task if it exists
    ///
    /// This method can be used to manually cancel a long-running future
    /// to free up resources and prevent unnecessary work.
    pub fn cancel(&self) {
        if let Some(task_handle) = self.task_handle.lock().take() {
            task_handle.abort();
        }
    }

    /// Internal method to update the state
    /// Uses write lock for state mutations
    fn set_state(&self, new_state: FutureState<T, E>) {
        *self.state.write() = new_state;
    }

    /// Internal method to set the task handle
    fn set_task_handle(&self, handle: JoinHandle<()>) {
        *self.task_handle.lock() = Some(handle);
    }
}

impl<T, E> Clone for FutureHandle<T, E> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            task_handle: self.task_handle.clone(),
        }
    }
}

/// Enhanced Resource Management: Automatic cleanup on drop
///
/// This ensures that futures are properly cancelled when the handle
/// is dropped, preventing resource leaks and zombie tasks.
impl<T, E> Drop for FutureHandle<T, E> {
    fn drop(&mut self) {
        // Ensure cleanup on drop - cancel any running task
        // We directly access the task_handle to avoid trait bound issues
        if let Some(task_handle) = self.task_handle.lock().take() {
            task_handle.abort();
        }
    }
}

/// Internal state for tracking future operations
struct FutureHookState<T, E> {
    /// Previous dependencies for comparison
    prev_deps: Option<Box<dyn EffectDependencies>>,
    /// Handle to the current future operation
    handle: FutureHandle<T, E>,
    /// Whether this hook has been initialized
    initialized: bool,
    /// Counter for active futures in this component
    active_futures: Arc<AtomicUsize>,
}

impl<T, E> FutureHookState<T, E>
where
    T: Clone,
    E: Clone,
{
    fn new() -> Self {
        Self {
            prev_deps: None,
            handle: FutureHandle::new(),
            initialized: false,
            active_futures: Arc::new(AtomicUsize::new(0)),
        }
    }
}

/// Implement Drop to ensure proper cleanup of dependencies
impl<T, E> Drop for FutureHookState<T, E> {
    fn drop(&mut self) {
        // Clear dependencies to prevent memory leaks
        self.prev_deps = None;

        // Cancel any running task and decrement counters
        if let Some(task_handle) = self.handle.task_handle.lock().take() {
            task_handle.abort();

            // Security: Decrement counters when task is cancelled during drop
            let active_count = self.active_futures.load(Ordering::Relaxed);
            if active_count > 0 {
                self.active_futures.fetch_sub(1, Ordering::Relaxed);
                GLOBAL_ACTIVE_FUTURES.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }
}

/// Error type for future operations that don't return a Result
#[derive(Debug, Clone)]
pub struct FutureError {
    message: String,
}

impl fmt::Display for FutureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Future error: {}", self.message)
    }
}

impl std::error::Error for FutureError {}

/// React-style useFuture hook that provides async future management for components
///
/// This function provides future-based async operations with React-like behavior:
/// - Futures are executed when dependencies change or on first render
/// - Supports dependency arrays for conditional re-execution
/// - Provides comprehensive state management (pending, resolved, error)
/// - Handles future cancellation on dependency changes or component unmount
/// - Thread-safe and optimized for concurrent access
/// - Supports both Result-returning and direct value futures
///
/// # Examples
///
/// ## Basic Future (runs once on mount)
/// ```rust,no_run
/// use reratui_hooks::future::{use_future, FutureState};
/// use std::time::Duration;
///
/// // Simple example showing basic future usage
/// let future_handle = use_future::<(), _, _, _, _>(|| async {
///     // Simulate async work
///     tokio::time::sleep(Duration::from_millis(100)).await;
///     Ok::<String, String>("Hello, World!".to_string())
/// }, None);
///
/// match future_handle.state() {
///     FutureState::Idle => println!("Not started"),
///     FutureState::Pending => println!("Loading..."),
///     FutureState::Progress(_) => println!("Processing..."),
///     FutureState::Resolved(data) => println!("Data: {}", data),
///     FutureState::Error(err) => println!("Error: {}", err),
/// }
/// ```
///
/// ## Future with Dependencies
/// ```rust,no_run
/// use reratui_hooks::future::{use_future, FutureState};
/// use std::time::Duration;
///
/// // Example with dependency tracking
/// let user_id = 123;
/// let user_future = use_future::<i32, _, _, _, _>(move || async move {
///     // Simulate API call
///     tokio::time::sleep(Duration::from_millis(200)).await;
///     Ok::<String, String>(format!("User data for ID: {}", user_id))
/// }, Some(user_id));
///
/// match user_future.state() {
///     FutureState::Idle => println!("Not started"),
///     FutureState::Pending => println!("Loading user..."),
///     FutureState::Progress(_) => println!("Processing user data..."),
///     FutureState::Resolved(user) => println!("User: {}", user),
///     FutureState::Error(err) => println!("Failed to load user: {}", err),
/// }
/// ```
///
/// ## Future with Manual Triggering
/// ```rust,no_run
/// use reratui_hooks::future::{use_future, FutureState};
/// use std::time::Duration;
///
/// // Example with manual triggering
/// let trigger = 1;
/// let data_future = use_future::<i32, _, _, _, _>(move || async move {
///     // Simulate work
///     tokio::time::sleep(Duration::from_millis(100)).await;
///     Ok::<String, String>(format!("Data fetched at trigger: {}", trigger))
/// }, Some(trigger));
///
/// match data_future.state() {
///     FutureState::Idle => println!("Not started"),
///     FutureState::Pending => println!("Fetching..."),
///     FutureState::Progress(_) => println!("Processing..."),
///     FutureState::Resolved(data) => println!("{}", data),
///     FutureState::Error(err) => println!("Error: {}", err),
/// }
/// ```
///
/// # Error Handling
///
/// This function will panic if called outside of a component render context.
/// Always ensure useFuture is called within a component function.
///
/// # Performance Notes
///
/// - Futures are automatically cancelled when dependencies change
/// - State updates are thread-safe and optimized for concurrent access
/// - Dependency comparison uses PartialEq for efficient change detection
/// - Multiple futures in the same component are managed independently
/// - Memory usage is minimal with Arc-based sharing
pub fn use_future<Deps, F, Fut, T, E>(
    future_factory: F,
    deps: impl Into<Option<Deps>>,
) -> FutureHandle<T, E>
where
    Deps: EffectDependencies + Clone + PartialEq + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T, E>> + Send + 'static,
    T: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + From<String> + 'static,
{
    let deps = deps.into();
    with_hook_context(|ctx| {
        let hook_index = ctx.next_hook_index();
        let mut states = ctx.states.borrow_mut();

        // Get or create future state for this hook
        let future_state = states
            .entry(hook_index)
            .or_insert_with(|| Box::new(FutureHookState::<T, E>::new()))
            .downcast_mut::<FutureHookState<T, E>>()
            .expect("Future state type mismatch");

        // Determine if future should run
        let should_run = match &deps {
            None => {
                // No dependencies - run on every render (usually not recommended for futures)
                true
            }
            Some(current_deps) => {
                // Check if dependencies have changed
                match &future_state.prev_deps {
                    None => {
                        // First run - always execute
                        true
                    }
                    Some(prev_deps) => {
                        // Optimized dependency comparison with hash-based fast path
                        if prev_deps.deps_hash() != current_deps.deps_hash() {
                            // Hash mismatch - dependencies definitely changed
                            true
                        } else {
                            // Hash match - might be collision, do detailed comparison
                            !current_deps.deps_eq(prev_deps.as_ref())
                        }
                    }
                }
            }
        };

        if should_run {
            // Security Check: Prevent resource exhaustion attacks
            let component_futures = future_state.active_futures.load(Ordering::Relaxed);
            let global_futures = GLOBAL_ACTIVE_FUTURES.load(Ordering::Relaxed);

            if component_futures >= MAX_CONCURRENT_FUTURES_PER_COMPONENT {
                future_state.handle.set_state(FutureState::Error(
                    format!(
                        "Component future limit exceeded: {}/{}",
                        component_futures, MAX_CONCURRENT_FUTURES_PER_COMPONENT
                    )
                    .into(),
                ));
                return future_state.handle.clone();
            }

            if global_futures >= MAX_GLOBAL_CONCURRENT_FUTURES {
                future_state.handle.set_state(FutureState::Error(
                    format!(
                        "Global future limit exceeded: {}/{}",
                        global_futures, MAX_GLOBAL_CONCURRENT_FUTURES
                    )
                    .into(),
                ));
                return future_state.handle.clone();
            }

            // Cancel any existing future
            future_state.handle.cancel();

            // Reset state to pending
            future_state.handle.set_state(FutureState::Pending);

            // Store new dependencies
            if let Some(current_deps) = &deps {
                future_state.prev_deps = Some(current_deps.clone_deps());
            } else {
                future_state.prev_deps = None;
            }

            // Increment counters before spawning
            future_state.active_futures.fetch_add(1, Ordering::Relaxed);
            GLOBAL_ACTIVE_FUTURES.fetch_add(1, Ordering::Relaxed);

            // Create clones for the async task
            let handle_clone_for_success = future_state.handle.clone();
            let handle_clone_for_error = future_state.handle.clone();
            let handle_clone_for_panic = future_state.handle.clone();
            let active_futures_clone = future_state.active_futures.clone();

            // Spawn the future
            let task_handle = tokio::spawn(async move {
                let result = spawn_catch_panic(async move {
                    match future_factory().await {
                        Ok(value) => {
                            handle_clone_for_success.set_state(FutureState::Resolved(value));
                        }
                        Err(error) => {
                            handle_clone_for_error.set_state(FutureState::Error(error));
                        }
                    }
                })
                .await;

                // Security Fix: Better panic handling with detailed error information
                if let Err(join_error) = result {
                    let panic_message = if join_error.is_panic() {
                        format!("Future panicked: {}", join_error)
                    } else if join_error.is_cancelled() {
                        "Future was cancelled".to_string()
                    } else {
                        format!("Future failed: {}", join_error)
                    };

                    handle_clone_for_panic.set_state(FutureState::Error(panic_message.into()));
                }

                // Security: Always decrement counters when future completes (success, error, or panic)
                active_futures_clone.fetch_sub(1, Ordering::Relaxed);
                GLOBAL_ACTIVE_FUTURES.fetch_sub(1, Ordering::Relaxed);
            });

            // Store the task handle for potential cancellation
            future_state.handle.set_task_handle(task_handle);
            future_state.initialized = true;
        }

        // Return a clone of the handle
        future_state.handle.clone()
    })
}
