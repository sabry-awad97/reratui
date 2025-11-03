//! Query hook for managing cached data fetching operations
//!
//! This module provides a hook similar to TanStack React Query for managing
//! server state, caching, and data fetching operations using Tokio for async execution.

use crate::effect::use_effect;
use crate::reducer::use_reducer;

#[cfg(test)]
pub mod tests;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace, warn};

/// Status of a query operation
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum QueryStatus {
    /// The query has not started yet
    #[default]
    Idle,
    /// The query is in progress
    Loading,
    /// The query is fetching new data while stale data is available
    Refreshing,
    /// The query completed successfully
    Success,
    /// The query failed
    Error,
}

/// Configuration options for a query
#[derive(Clone)]
pub struct QueryOptions {
    /// Whether to enable automatic background refreshing
    pub enabled: bool,
    /// How long the data should be considered fresh
    pub stale_time: Duration,
    /// How long to keep inactive data in cache
    pub cache_time: Duration,
    /// Whether to retry on failure
    pub retry: bool,
    /// Number of retry attempts
    pub retry_attempts: u32,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            stale_time: Duration::from_secs(0),
            cache_time: Duration::from_secs(300), // 5 minutes
            retry: true,
            retry_attempts: 3,
        }
    }
}

/// Cached query data with metadata
#[derive(Clone, Debug)]
struct CachedQuery<T> {
    data: Option<T>,
    last_updated: Instant,
    is_stale: bool,
}

impl<T> CachedQuery<T> {
    fn is_fresh(&self, stale_time: Duration) -> bool {
        !self.is_stale && self.last_updated.elapsed() < stale_time
    }

    fn should_cache_expire(&self, cache_time: Duration) -> bool {
        self.last_updated.elapsed() > cache_time
    }
}

/// Global query cache
type QueryCache = HashMap<String, Box<dyn std::any::Any + Send + Sync>>;

static QUERY_CACHE: once_cell::sync::Lazy<Arc<Mutex<QueryCache>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Result of a query operation
#[derive(Clone)]
pub struct QueryResult<T, E> {
    /// Current status of the query
    pub status: QueryStatus,
    /// The fetched data, if available
    pub data: Option<T>,
    /// The error, if any occurred
    pub error: Option<E>,
    /// Whether the data is stale and being refreshed
    pub is_stale: bool,
    /// Function to manually refetch the data
    pub refetch: Arc<dyn Fn() + Send + Sync>,
    /// Function to invalidate the query cache
    pub invalidate: Arc<dyn Fn() + Send + Sync>,
}

/// State for a query operation
#[derive(Clone, PartialEq, Debug)]
pub struct QueryState<T, E> {
    status: QueryStatus,
    data: Option<T>,
    error: Option<E>,
    is_stale: bool,
}

/// Actions that can be performed on a query
#[derive(Clone)]
enum QueryAction<T: Clone, E: Clone> {
    Loading,
    Refreshing,
    Success(T),
    Error(E),
}

/// A hook for managing cached query operations with Tokio
///
/// This hook provides caching, background updates, retry logic with exponential backoff,
/// and query invalidation using Tokio for async execution.
///
/// # Retry Logic
///
/// When a query fails and `options.retry` is true, the hook will automatically retry
/// the query up to `options.retry_attempts` times with exponential backoff:
/// - 1st retry: 1 second delay
/// - 2nd retry: 2 second delay
/// - 3rd retry: 4 second delay
/// - And so on, capped at 2^10 seconds to prevent overflow
///
/// # Caching Behavior
///
/// - Successful results are cached according to `cache_time` and `stale_time`
/// - Failed results are only cached if `cache_time > 0` to allow retries
/// - Cache can be manually invalidated using the `invalidate` function
///
/// # Type Parameters
///
/// * `K` - The type of the query key
/// * `F` - The type of the query function
/// * `Fut` - The type of the future returned by the query function
/// * `T` - The type of the data returned by the query
/// * `E` - The type of the error that can be returned
///
/// # Arguments
///
/// * `key` - A unique key for the query
/// * `query_fn` - The async function to fetch the data
/// * `options` - Optional configuration for the query
///
/// # Examples
///
/// ```ignore
/// // Basic usage with automatic retries
/// let result = use_query("user-1", || async {
///     fetch_user(1).await
/// }, None);
///
/// // With custom retry configuration
/// let options = QueryOptions {
///     retry: true,
///     retry_attempts: 5,
///     stale_time: Duration::from_secs(30),
///     cache_time: Duration::from_secs(300),
///     ..Default::default()
/// };
/// let result = use_query("posts", || async {
///     fetch_posts().await
/// }, Some(options));
///
/// // Manual operations
/// result.refetch(); // Force refetch
/// result.invalidate(); // Clear cache
/// ```
pub fn use_query<K, F, Fut, T, E>(
    key: K,
    query_fn: F,
    options: Option<QueryOptions>,
) -> QueryResult<T, E>
where
    K: Hash + Eq + Clone + Send + Sync + Debug + 'static,
    F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<T, E>> + Send + 'static,
    T: Clone + PartialEq + Send + Sync + Debug + 'static,
    E: Clone + PartialEq + Send + Sync + Debug + 'static,
{
    let options = options.unwrap_or_default();

    // Create a unique cache key using the query key
    let cache_key = format!("{:?}", key);
    debug!(
        query_key = ?key,
        cache_key = %cache_key,
        "Initializing query hook with Tokio"
    );

    // Define the reducer function
    let reducer = |state: QueryState<T, E>, action: QueryAction<T, E>| match action {
        QueryAction::Loading => QueryState {
            status: QueryStatus::Loading,
            data: state.data,
            error: None,
            is_stale: false,
        },
        QueryAction::Refreshing => QueryState {
            status: QueryStatus::Refreshing,
            data: state.data,
            error: None,
            is_stale: true,
        },
        QueryAction::Success(data) => QueryState {
            status: QueryStatus::Success,
            data: Some(data),
            error: None,
            is_stale: false,
        },
        QueryAction::Error(error) => QueryState {
            status: QueryStatus::Error,
            data: state.data,
            error: Some(error),
            is_stale: false,
        },
    };

    // Initialize state
    let initial_state = QueryState {
        status: QueryStatus::Idle,
        data: None,
        error: None,
        is_stale: false,
    };

    // Use the reducer
    let (state, dispatch) = use_reducer(reducer, initial_state);
    trace!(
        query_key = ?key,
        initial_state = ?state.get(),
        "Query state initialized"
    );

    // Create a shared state for the query execution
    let query_state = Arc::new(Mutex::new(()));

    // Create the refetch function using Tokio
    let refetch_arc = {
        let query_state = Arc::clone(&query_state);
        let query_fn = query_fn.clone();
        let dispatch = dispatch.clone();
        let state = state.clone();
        let options = options.clone();
        let key = key.clone();
        let cache_key = cache_key.clone();

        Arc::new(move || {
            let _lock = query_state.lock();
            let key = key.clone();
            info!(
                query_key = ?key,
                "Refetching query data with Tokio"
            );

            let dispatch = dispatch.clone();
            let current_state = state.get();
            let query_fn = query_fn.clone();
            let options = options.clone();
            let cache_key = cache_key.clone();

            // Spawn the query execution task using Tokio
            let _handle = tokio::spawn(async move {
                // Update status based on current data
                if current_state.data.is_some() {
                    debug!(
                        query_key = ?key,
                        "Refreshing stale data"
                    );
                    dispatch.dispatch(QueryAction::Refreshing);
                } else {
                    debug!(
                        query_key = ?key,
                        "Starting initial load"
                    );
                    dispatch.dispatch(QueryAction::Loading);
                }

                let mut attempts = 0;
                loop {
                    let current_query = query_fn.clone();
                    let key = key.clone();
                    trace!(
                        query_key = ?key,
                        attempt = attempts + 1,
                        "Executing query"
                    );

                    match current_query().await {
                        Ok(result) => {
                            info!(
                                query_key = ?key,
                                "Query completed successfully"
                            );

                            // Update cache
                            {
                                let mut cache = QUERY_CACHE.lock();
                                let cached_query: CachedQuery<T> = CachedQuery {
                                    data: Some(result.clone()),
                                    last_updated: Instant::now(),
                                    is_stale: false,
                                };
                                cache.insert(cache_key.clone(), Box::new(cached_query));
                            }

                            dispatch.dispatch(QueryAction::Success(result));
                            break;
                        }
                        Err(err) => {
                            attempts += 1;
                            if !options.retry || attempts >= options.retry_attempts {
                                error!(
                                    query_key = ?key,
                                    attempts,
                                    error = ?err,
                                    "Query failed permanently"
                                );

                                // Update cache with error only if cache_time > 0
                                if options.cache_time > Duration::from_secs(0) {
                                    let mut cache = QUERY_CACHE.lock();
                                    let cached_query: CachedQuery<T> = CachedQuery {
                                        data: None,
                                        last_updated: Instant::now(),
                                        is_stale: false,
                                    };
                                    cache.insert(cache_key.clone(), Box::new(cached_query));
                                }

                                dispatch.dispatch(QueryAction::Error(err));
                                break;
                            }

                            // Calculate delay for exponential backoff (attempts is 1-based here)
                            let delay_ms = 2u64.pow((attempts - 1).min(10)) * 1000; // Cap at 2^10 to prevent overflow
                            warn!(
                                query_key = ?key,
                                attempt = attempts,
                                retry_delay_ms = delay_ms,
                                "Query failed, retrying"
                            );

                            // Use Tokio sleep with exponential backoff
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        }
                    }
                }
            });
        })
    };

    // Create the invalidate function
    let invalidate_arc = {
        let cache_key = cache_key.clone();
        Arc::new(move || {
            debug!(
                cache_key = %cache_key,
                "Invalidating query cache"
            );
            let mut cache = QUERY_CACHE.lock();
            cache.remove(&cache_key);
        })
    };

    // Check cache for existing data
    let cached_data = {
        let cache = QUERY_CACHE.lock();
        if let Some(cached) = cache.get(&cache_key) {
            if let Some(cached_query) = cached.downcast_ref::<CachedQuery<T>>() {
                if !cached_query.should_cache_expire(options.cache_time) {
                    Some(cached_query.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    // Initialize state with cached data if available and successful
    // Don't use cached errors to allow retries
    if let Some(ref cached) = cached_data
        && cached.is_fresh(options.stale_time)
        && cached.data.is_some()
    {
        debug!(
            query_key = ?key,
            "Using fresh cached successful data"
        );
        // Only update state with successful cached data
        if let Some(data) = &cached.data {
            dispatch.dispatch(QueryAction::Success(data.clone()));
        }
    }

    // Set up effect for initial query and background refresh
    {
        let refetch = Arc::clone(&refetch_arc);
        let key = key.clone();
        let unique_key = format!("{:?}", key);

        use_effect(
            move || {
                if options.enabled {
                    debug!(
                        query_key = ?key,
                        stale_time_secs = ?options.stale_time.as_secs(),
                        "Setting up query effect with Tokio"
                    );

                    // Only refetch if we don't have fresh cached successful data
                    // Always refetch if we have cached errors to allow retries
                    let should_fetch = if let Some(cached) = &cached_data {
                        !cached.is_fresh(options.stale_time) || cached.data.is_none()
                    } else {
                        true
                    };

                    if should_fetch {
                        refetch();
                    }

                    // Set up background refresh if stale_time is configured
                    let bg_task_handle = if options.stale_time > Duration::from_secs(0) {
                        let refetch_for_bg = Arc::clone(&refetch);
                        let key_for_bg = key.clone();

                        Some(tokio::spawn(async move {
                            loop {
                                tokio::time::sleep(options.stale_time).await;
                                trace!(
                                    query_key = ?key_for_bg,
                                    "Executing background refresh"
                                );
                                refetch_for_bg();
                            }
                        }))
                    } else {
                        None
                    };

                    // Return cleanup function that cancels the background task
                    let key = key.clone();
                    Some(move || {
                        if let Some(handle) = bg_task_handle {
                            handle.abort();
                            debug!(
                                query_key = ?key,
                                "Cancelled background refresh task"
                            );
                        }
                        debug!(
                            query_key = ?key,
                            "Cleaning up query effect"
                        );
                    })
                } else {
                    debug!(
                        query_key = ?key,
                        "Query disabled, skipping effect"
                    );
                    None
                }
            },
            unique_key,
        );
    }

    let current_state = state.get();
    trace!(
        query_key = ?key,
        cache_key = %cache_key,
        status = ?current_state.status,
        has_data = current_state.data.is_some(),
        has_error = current_state.error.is_some(),
        is_stale = current_state.is_stale,
        "Returning query result"
    );

    QueryResult {
        status: current_state.status,
        data: current_state.data,
        error: current_state.error,
        is_stale: current_state.is_stale,
        refetch: refetch_arc,
        invalidate: invalidate_arc,
    }
}

/// Clear all cached queries - useful for testing and cleanup
pub fn clear_query_cache() {
    let mut cache = QUERY_CACHE.lock();
    cache.clear();
    debug!("Query cache cleared");
}

/// Get cache statistics for debugging
pub fn get_cache_stats() -> (usize, Vec<String>) {
    let cache = QUERY_CACHE.lock();
    let size = cache.len();
    let keys: Vec<String> = cache.keys().cloned().collect();
    (size, keys)
}
