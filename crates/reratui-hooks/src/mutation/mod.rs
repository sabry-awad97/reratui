pub use parking_lot::Mutex;
pub use parking_lot::RwLock;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::memo::use_memo_once;

#[cfg(test)]
pub mod tests;

/// Status of a mutation operation
#[derive(Debug, Clone, PartialEq)]
pub enum MutationStatus {
    /// Mutation has not been triggered yet
    Idle,
    /// Mutation is currently executing
    Pending,
    /// Mutation completed successfully
    Success,
    /// Mutation failed with an error
    Error,
    /// Mutation was cancelled
    Cancelled,
}

/// Context passed to mutation callbacks
#[derive(Debug, Clone)]
pub struct MutationContext {
    /// Unique identifier for this mutation execution
    pub mutation_id: Uuid,
    /// Timestamp when the mutation was started
    pub started_at: Instant,
}

/// Type alias for success callback function
pub type OnSuccessCallback<TData, TVariables> =
    Box<dyn Fn(&TData, &TVariables, &MutationContext) + Send + Sync>;

/// Type alias for error callback function
pub type OnErrorCallback<TError, TVariables> =
    Box<dyn Fn(&TError, &TVariables, &MutationContext) + Send + Sync>;

/// Type alias for settled callback function
pub type OnSettledCallback<TData, TError, TVariables> =
    Box<dyn Fn(Option<&TData>, Option<&TError>, &TVariables, &MutationContext) + Send + Sync>;

/// Type alias for mutate callback function
pub type OnMutateCallback<TVariables> =
    Box<dyn Fn(&TVariables) -> Option<MutationContext> + Send + Sync>;

/// Type alias for mutation function
pub type MutationFn<TData, TError, TVariables> = Arc<
    dyn Fn(TVariables) -> Pin<Box<dyn Future<Output = Result<TData, TError>> + Send>> + Send + Sync,
>;

/// Options for configuring mutation behavior
pub struct MutationOptions<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    /// Callback called when mutation succeeds
    pub on_success: Option<OnSuccessCallback<TData, TVariables>>,
    /// Callback called when mutation fails
    pub on_error: Option<OnErrorCallback<TError, TVariables>>,
    /// Callback called when mutation settles (success or error)
    pub on_settled: Option<OnSettledCallback<TData, TError, TVariables>>,
    /// Callback called before mutation starts
    pub on_mutate: Option<OnMutateCallback<TVariables>>,
    /// Whether to retry failed mutations
    pub retry: bool,
    /// Number of retry attempts
    pub retry_attempts: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Whether to use exponential backoff for retries
    pub retry_exponential_backoff: bool,
    /// Maximum delay for exponential backoff (default: 30 seconds)
    pub retry_max_delay: Duration,
}

impl<TData, TError, TVariables> Default for MutationOptions<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            on_success: None,
            on_error: None,
            on_settled: None,
            on_mutate: None,
            retry: false,
            retry_attempts: 0,
            retry_delay: Duration::from_millis(1000),
            retry_exponential_backoff: false,
            retry_max_delay: Duration::from_secs(30),
        }
    }
}

impl<TData, TError, TVariables> MutationOptions<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    /// Create a new builder for MutationOptions
    pub fn builder() -> MutationOptionsBuilder<TData, TError, TVariables> {
        MutationOptionsBuilder::new()
    }
}

/// Builder for constructing MutationOptions with a fluent API
pub struct MutationOptionsBuilder<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    on_success: Option<OnSuccessCallback<TData, TVariables>>,
    on_error: Option<OnErrorCallback<TError, TVariables>>,
    on_settled: Option<OnSettledCallback<TData, TError, TVariables>>,
    on_mutate: Option<OnMutateCallback<TVariables>>,
    retry: bool,
    retry_attempts: u32,
    retry_delay: Duration,
    retry_exponential_backoff: bool,
    retry_max_delay: Duration,
}

impl<TData, TError, TVariables> MutationOptionsBuilder<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            on_success: None,
            on_error: None,
            on_settled: None,
            on_mutate: None,
            retry: false,
            retry_attempts: 0,
            retry_delay: Duration::from_millis(1000),
            retry_exponential_backoff: false,
            retry_max_delay: Duration::from_secs(30),
        }
    }

    /// Set the success callback
    ///
    /// # Example
    /// ```rust,ignore
    /// MutationOptions::builder()
    ///     .on_success(|data, variables, context| {
    ///         println!("Success: {:?}", data);
    ///     })
    /// ```
    pub fn on_success<F>(mut self, callback: F) -> Self
    where
        F: Fn(&TData, &TVariables, &MutationContext) + Send + Sync + 'static,
    {
        self.on_success = Some(Box::new(callback));
        self
    }

    /// Set the error callback
    ///
    /// # Example
    /// ```rust,ignore
    /// MutationOptions::builder()
    ///     .on_error(|error, variables, context| {
    ///         eprintln!("Error: {:?}", error);
    ///     })
    /// ```
    pub fn on_error<F>(mut self, callback: F) -> Self
    where
        F: Fn(&TError, &TVariables, &MutationContext) + Send + Sync + 'static,
    {
        self.on_error = Some(Box::new(callback));
        self
    }

    /// Set the settled callback (called on both success and error)
    ///
    /// # Example
    /// ```rust,ignore
    /// MutationOptions::builder()
    ///     .on_settled(|data, error, variables, context| {
    ///         println!("Mutation completed");
    ///     })
    /// ```
    pub fn on_settled<F>(mut self, callback: F) -> Self
    where
        F: Fn(Option<&TData>, Option<&TError>, &TVariables, &MutationContext)
            + Send
            + Sync
            + 'static,
    {
        self.on_settled = Some(Box::new(callback));
        self
    }

    /// Set the mutate callback (called before mutation starts)
    ///
    /// # Example
    /// ```rust,ignore
    /// MutationOptions::builder()
    ///     .on_mutate(|variables| {
    ///         println!("Starting mutation with: {:?}", variables);
    ///         None // Return None to use default context
    ///     })
    /// ```
    pub fn on_mutate<F>(mut self, callback: F) -> Self
    where
        F: Fn(&TVariables) -> Option<MutationContext> + Send + Sync + 'static,
    {
        self.on_mutate = Some(Box::new(callback));
        self
    }

    /// Enable or disable retry on failure
    ///
    /// # Example
    /// ```rust,ignore
    /// MutationOptions::builder()
    ///     .retry(true)
    /// ```
    pub fn retry(mut self, retry: bool) -> Self {
        self.retry = retry;
        self
    }

    /// Set the number of retry attempts
    ///
    /// # Example
    /// ```rust,ignore
    /// MutationOptions::builder()
    ///     .retry_attempts(3)
    /// ```
    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Set the delay between retry attempts
    ///
    /// # Example
    /// ```rust,ignore
    /// use std::time::Duration;
    ///
    /// MutationOptions::builder()
    ///     .retry_delay(Duration::from_millis(500))
    /// ```
    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Enable exponential backoff for retries
    ///
    /// When enabled, retry delays increase exponentially: delay * 2^attempt
    ///
    /// # Example
    /// ```rust,ignore
    /// MutationOptions::builder()
    ///     .retry_exponential_backoff(true)
    /// ```
    pub fn retry_exponential_backoff(mut self, enabled: bool) -> Self {
        self.retry_exponential_backoff = enabled;
        self
    }

    /// Set the maximum delay for exponential backoff
    ///
    /// # Example
    /// ```rust,ignore
    /// use std::time::Duration;
    ///
    /// MutationOptions::builder()
    ///     .retry_max_delay(Duration::from_secs(60))
    /// ```
    pub fn retry_max_delay(mut self, max_delay: Duration) -> Self {
        self.retry_max_delay = max_delay;
        self
    }

    /// Build the final MutationOptions
    pub fn build(self) -> MutationOptions<TData, TError, TVariables> {
        MutationOptions {
            on_success: self.on_success,
            on_error: self.on_error,
            on_settled: self.on_settled,
            on_mutate: self.on_mutate,
            retry: self.retry,
            retry_attempts: self.retry_attempts,
            retry_delay: self.retry_delay,
            retry_exponential_backoff: self.retry_exponential_backoff,
            retry_max_delay: self.retry_max_delay,
        }
    }
}

impl<TData, TError, TVariables> Default for MutationOptionsBuilder<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a mutation operation
#[derive(Debug, Clone)]
pub struct MutationResult<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    /// Current status of the mutation
    pub status: MutationStatus,
    /// Data returned by successful mutation
    pub data: Option<TData>,
    /// Error returned by failed mutation
    pub error: Option<TError>,
    /// Variables passed to the mutation
    pub variables: Option<TVariables>,
    /// Whether the mutation is currently pending
    pub is_pending: bool,
    /// Whether the mutation completed successfully
    pub is_success: bool,
    /// Whether the mutation failed with an error
    pub is_error: bool,
    /// Whether the mutation is in idle state
    pub is_idle: bool,
    /// Whether the mutation was cancelled
    pub is_cancelled: bool,
    /// Timestamp when the mutation was submitted
    pub submitted_at: Option<Instant>,
    /// Number of failed attempts
    pub failed_count: u32,
    /// Reason for the final failure
    pub failure_reason: Option<String>,
    /// Context information for this mutation
    pub context: Option<MutationContext>,
}

impl<TData, TError, TVariables> Default for MutationResult<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            status: MutationStatus::Idle,
            data: None,
            error: None,
            variables: None,
            is_pending: false,
            is_success: false,
            is_error: false,
            is_idle: true,
            is_cancelled: false,
            submitted_at: None,
            failed_count: 0,
            failure_reason: None,
            context: None,
        }
    }
}

impl<TData, TError, TVariables> MutationResult<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    fn update_status(&mut self, status: MutationStatus) {
        self.status = status.clone();
        self.is_idle = matches!(status, MutationStatus::Idle);
        self.is_pending = matches!(status, MutationStatus::Pending);
        self.is_success = matches!(status, MutationStatus::Success);
        self.is_error = matches!(status, MutationStatus::Error);
        self.is_cancelled = matches!(status, MutationStatus::Cancelled);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Mutation object that provides methods to trigger mutations
pub struct Mutation<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    state: Arc<Mutex<MutationResult<TData, TError, TVariables>>>,
    mutation_fn: MutationFn<TData, TError, TVariables>,
    options: Arc<MutationOptions<TData, TError, TVariables>>,
    /// Handle to the currently running mutation task (if any)
    task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl<TData, TError, TVariables> Clone for Mutation<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            mutation_fn: Arc::clone(&self.mutation_fn),
            options: Arc::clone(&self.options),
            task_handle: Arc::clone(&self.task_handle),
        }
    }
}

impl<TData, TError, TVariables> Mutation<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
{
    /// Create a new mutation with the given function and options
    pub fn new<F, Fut>(
        mutation_fn: F,
        options: Option<MutationOptions<TData, TError, TVariables>>,
    ) -> Self
    where
        F: Fn(TVariables) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<TData, TError>> + Send + 'static,
    {
        let boxed_fn = Arc::new(move |variables: TVariables| -> Pin<Box<dyn Future<Output = Result<TData, TError>> + Send>> {
            Box::pin(mutation_fn(variables))
        });

        Self {
            state: Arc::new(Mutex::new(MutationResult::default())),
            mutation_fn: boxed_fn,
            options: Arc::new(options.unwrap_or_default()),
            task_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the current mutation state
    pub fn get_state(&self) -> MutationResult<TData, TError, TVariables> {
        self.state.lock().clone()
    }

    /// Reset the mutation state to idle
    pub fn reset(&self) {
        // Cancel any running task first
        self.cancel();

        let mut state = self.state.lock();
        state.reset();
    }

    /// Cancel the currently running mutation
    ///
    /// If a mutation is currently executing, it will be aborted and the state
    /// will be updated to Cancelled.
    pub fn cancel(&self) {
        if let Some(handle) = self.task_handle.lock().take() {
            handle.abort();

            let mut state = self.state.lock();
            state.update_status(MutationStatus::Cancelled);
            state.failure_reason = Some("Mutation cancelled by user".to_string());
        }
    }

    /// Trigger a mutation (fire and forget)
    pub fn mutate(&self, variables: TVariables) {
        let state = Arc::clone(&self.state);
        let mutation_fn = Arc::clone(&self.mutation_fn);
        let options = Arc::clone(&self.options);
        let task_handle = Arc::clone(&self.task_handle);

        // Spawn the mutation task and store the handle
        let handle = tokio::spawn(async move {
            Self::execute_mutation(state, mutation_fn, options, variables).await;
        });

        *task_handle.lock() = Some(handle);
    }

    /// Trigger a mutation and return a future that resolves when complete
    pub async fn mutate_async(&self, variables: TVariables) -> Result<TData, TError> {
        let state = Arc::clone(&self.state);
        let mutation_fn = Arc::clone(&self.mutation_fn);
        let options = Arc::clone(&self.options);

        Self::execute_mutation_async(state, mutation_fn, options, variables).await
    }

    async fn execute_mutation(
        state: Arc<Mutex<MutationResult<TData, TError, TVariables>>>,
        mutation_fn: MutationFn<TData, TError, TVariables>,
        options: Arc<MutationOptions<TData, TError, TVariables>>,
        variables: TVariables,
    ) {
        let _ = Self::execute_mutation_async(state, mutation_fn, options, variables).await;
    }

    async fn execute_mutation_async(
        state: Arc<Mutex<MutationResult<TData, TError, TVariables>>>,
        mutation_fn: MutationFn<TData, TError, TVariables>,
        options: Arc<MutationOptions<TData, TError, TVariables>>,
        variables: TVariables,
    ) -> Result<TData, TError> {
        let mutation_id = Uuid::new_v4();
        let started_at = Instant::now();

        // Call on_mutate callback and use custom context if provided
        let context = if let Some(on_mutate) = &options.on_mutate {
            on_mutate(&variables).unwrap_or(MutationContext {
                mutation_id,
                started_at,
            })
        } else {
            MutationContext {
                mutation_id,
                started_at,
            }
        };

        // Update state to pending
        {
            let mut state_guard = state.lock();
            state_guard.update_status(MutationStatus::Pending);
            state_guard.variables = Some(variables.clone());
            state_guard.submitted_at = Some(started_at);
            state_guard.context = Some(context.clone());
            state_guard.data = None;
            state_guard.error = None;
        }

        let mut attempts = 0;
        let max_attempts = if options.retry {
            options.retry_attempts + 1
        } else {
            1
        };

        loop {
            attempts += 1;

            // Execute the mutation
            let result = mutation_fn(variables.clone()).await;

            match result {
                Ok(data) => {
                    // Success - update state
                    {
                        let mut state_guard = state.lock();
                        state_guard.update_status(MutationStatus::Success);
                        state_guard.data = Some(data.clone());
                        state_guard.error = None;
                        state_guard.failed_count = 0;
                        state_guard.failure_reason = None;
                    }

                    // Call success callback
                    if let Some(on_success) = &options.on_success {
                        on_success(&data, &variables, &context);
                    }

                    // Call settled callback
                    if let Some(on_settled) = &options.on_settled {
                        on_settled(Some(&data), None, &variables, &context);
                    }

                    return Ok(data);
                }
                Err(error) => {
                    if attempts < max_attempts {
                        // Calculate retry delay with optional exponential backoff
                        let delay = if options.retry_exponential_backoff {
                            // Exponential backoff: base_delay * 2^(attempt - 1)
                            let exponential_delay = options
                                .retry_delay
                                .checked_mul(2_u32.pow(attempts - 1))
                                .unwrap_or(options.retry_max_delay);

                            // Cap at max_delay
                            exponential_delay.min(options.retry_max_delay)
                        } else {
                            options.retry_delay
                        };

                        // Retry after calculated delay
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    // Final failure - update state
                    {
                        let mut state_guard = state.lock();
                        state_guard.update_status(MutationStatus::Error);
                        state_guard.data = None;
                        state_guard.error = Some(error.clone());
                        state_guard.failed_count = attempts;
                        state_guard.failure_reason =
                            Some(format!("Failed after {} attempts", attempts));
                    }

                    // Call error callback
                    if let Some(on_error) = &options.on_error {
                        on_error(&error, &variables, &context);
                    }

                    // Call settled callback
                    if let Some(on_settled) = &options.on_settled {
                        on_settled(None, Some(&error), &variables, &context);
                    }

                    return Err(error);
                }
            }
        }
    }
}

/// Hook for creating and managing mutations
///
/// # Example
/// ```rust,ignore
/// use reratui_hooks::mutation::{use_mutation, MutationOptions};
///
/// // Define your request/response types
/// struct CreateUserRequest {
///     name: String,
/// }
///
/// let mutation = use_mutation(
///     |variables: CreateUserRequest| async move {
///         // Your async mutation logic here
///         Ok::<String, String>(format!("Created user: {}", variables.name))
///     },
///     Some(MutationOptions::builder()
///         .on_success(|data, variables, context| {
///             println!("Success: {}", data);
///         })
///         .on_error(|error, variables, context| {
///             eprintln!("Error: {}", error);
///         })
///         .build())
/// );
///
/// // Usage in component
/// mutation.mutate(CreateUserRequest { name: "John".to_string() });
/// ```
pub fn use_mutation<TData, TError, TVariables, F, Fut>(
    mutation_fn: F,
    options: Option<MutationOptions<TData, TError, TVariables>>,
) -> Mutation<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
    F: Fn(TVariables) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<TData, TError>> + Send + 'static,
{
    // Use memo to ensure the mutation object is stable across re-renders
    // Empty dependencies - mutation function should be stable
    use_memo_once(|| Mutation::new(mutation_fn, options))
}
