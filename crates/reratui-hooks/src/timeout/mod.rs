//! Timeout hook for executing callbacks after a delay
//!
//! This module provides the `use_timeout` hook for scheduling one-time callbacks
//! that execute after a specified duration.

use crate::effect::use_effect;
use std::time::Duration;

#[cfg(test)]
mod tests;

/// A hook that executes a callback after a specified timeout duration.
///
/// This hook schedules a one-time callback to run after the given duration.
/// The timeout starts when the component mounts or when dependencies change.
///
/// # Arguments
///
/// * `callback` - The function to execute after the timeout
/// * `duration` - How long to wait before executing the callback
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,no_run
/// use reratui_hooks::timeout::use_timeout;
/// use std::time::Duration;
///
/// // Show a notification after 3 seconds
/// use_timeout(
///     || {
///         println!("Timeout triggered!");
///     },
///     Duration::from_secs(3),
/// );
/// ```
///
/// ## With State Updates
///
/// ```rust,no_run
/// use reratui_hooks::{timeout::use_timeout, state::use_state};
/// use std::time::Duration;
///
/// let (show_message, set_show_message) = use_state(|| false);
///
/// use_timeout(
///     {
///         let set_show_message = set_show_message.clone();
///         move || {
///             set_show_message.set(true);
///         }
///     },
///     Duration::from_secs(2),
/// );
/// ```
///
/// ## Auto-hide Notification
///
/// ```rust,no_run
/// use reratui_hooks::{timeout::use_timeout, state::use_state};
/// use std::time::Duration;
///
/// let (notification, set_notification) = use_state(|| Some("Welcome!".to_string()));
///
/// // Auto-hide after 5 seconds
/// use_timeout(
///     {
///         let set_notification = set_notification.clone();
///         move || {
///             set_notification.set(None);
///         }
///     },
///     Duration::from_secs(5),
/// );
/// ```
///
/// ## Delayed Action
///
/// ```rust,no_run
/// use reratui_hooks::timeout::use_timeout;
/// use std::time::Duration;
///
/// // Perform action after delay
/// use_timeout(
///     || {
///         // Fetch data, show tooltip, etc.
///         perform_delayed_action();
///     },
///     Duration::from_millis(500),
/// );
///
/// fn perform_delayed_action() {
///     // Implementation
/// }
/// ```
///
/// # Implementation Details
///
/// - The timeout is started when the component mounts
/// - If the component unmounts before the timeout, the callback is cancelled
/// - The callback is only executed once
/// - Uses Tokio's async runtime for scheduling
///
/// # Notes
///
/// - The callback captures values at the time the hook is called
/// - For recurring timeouts, use `use_interval` instead
/// - The timeout is automatically cleaned up on unmount
/// - Minimum duration is 1 millisecond
///
/// # Performance
///
/// This hook spawns a lightweight async task that sleeps for the specified duration.
/// The task is automatically cancelled if the component unmounts.
pub fn use_timeout<F>(callback: F, duration: Duration)
where
    F: Fn() + Send + 'static,
{
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
                    eprintln!("Warning: use_timeout called outside tokio runtime context");
                    return None;
                }
            };

            // Spawn async timeout task
            let task_handle = handle.spawn(async move {
                tokio::time::sleep(safe_duration).await;
                callback();
            });

            // Return cleanup function that cancels the task
            Some(Box::new(move || {
                task_handle.abort();
            }) as Box<dyn FnOnce() + Send>)
        },
        (), // Run once on mount
    );
}

/// A hook that executes a callback after a timeout, with the ability to reset the timer.
///
/// This variant returns a reset function that allows you to restart the timeout.
///
/// # Returns
///
/// A function that resets the timeout when called.
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::timeout::use_timeout_with_reset;
/// use std::time::Duration;
///
/// let reset = use_timeout_with_reset(
///     || {
///         println!("Timeout!");
///     },
///     Duration::from_secs(3),
/// );
///
/// // Reset the timer (e.g., on user activity)
/// reset();
/// ```
pub fn use_timeout_with_reset<F>(callback: F, duration: Duration) -> impl Fn() + Clone
where
    F: Fn() + Send + 'static,
{
    use crate::state::use_state;

    let (reset_count, set_reset_count) = use_state(|| 0u32);

    use_effect(
        move || {
            let safe_duration = if duration.is_zero() {
                Duration::from_millis(1)
            } else {
                duration
            };

            let handle = match tokio::runtime::Handle::try_current() {
                Ok(handle) => handle,
                Err(_) => {
                    eprintln!(
                        "Warning: use_timeout_with_reset called outside tokio runtime context"
                    );
                    return None;
                }
            };

            let task_handle = handle.spawn(async move {
                tokio::time::sleep(safe_duration).await;
                callback();
            });

            Some(Box::new(move || {
                task_handle.abort();
            }) as Box<dyn FnOnce() + Send>)
        },
        reset_count.get(),
    );

    // Return reset function
    move || {
        set_reset_count.update(|count| count + 1);
    }
}

/// A hook that executes a callback after a timeout, with manual control.
///
/// This variant returns functions to start, cancel, and check the timeout status.
///
/// # Returns
///
/// A tuple of `(start, cancel, is_active)` functions.
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::timeout::use_timeout_controlled;
/// use std::time::Duration;
///
/// let (start, cancel, is_active) = use_timeout_controlled(
///     || {
///         println!("Timeout!");
///     },
///     Duration::from_secs(3),
/// );
///
/// // Start the timeout manually
/// start();
///
/// // Check if active
/// if is_active() {
///     // Cancel if needed
///     cancel();
/// }
/// ```
pub fn use_timeout_controlled<F>(
    callback: F,
    duration: Duration,
) -> (
    impl Fn() + Clone,
    impl Fn() + Clone,
    impl Fn() -> bool + Clone,
)
where
    F: Fn() + Send + 'static,
{
    use crate::state::use_state;

    let (is_active, set_is_active) = use_state(|| false);
    let (trigger, set_trigger) = use_state(|| 0u32);

    use_effect(
        {
            let is_active = is_active.clone();
            let set_is_active = set_is_active.clone();
            move || {
                if !is_active.get() {
                    return None;
                }

                let safe_duration = if duration.is_zero() {
                    Duration::from_millis(1)
                } else {
                    duration
                };

                let handle = match tokio::runtime::Handle::try_current() {
                    Ok(handle) => handle,
                    Err(_) => {
                        eprintln!(
                            "Warning: use_timeout_controlled called outside tokio runtime context"
                        );
                        return None;
                    }
                };

                let task_handle = handle.spawn({
                    let set_is_active = set_is_active.clone();
                    async move {
                        tokio::time::sleep(safe_duration).await;
                        callback();
                        set_is_active.set(false);
                    }
                });

                Some(Box::new(move || {
                    task_handle.abort();
                }) as Box<dyn FnOnce() + Send>)
            }
        },
        (is_active.get(), trigger.get()),
    );

    let start = {
        let set_is_active = set_is_active.clone();
        let set_trigger = set_trigger.clone();
        move || {
            set_is_active.set(true);
            set_trigger.update(|t| t + 1);
        }
    };

    let cancel = {
        let set_is_active = set_is_active.clone();
        move || {
            set_is_active.set(false);
        }
    };

    let check_active = move || is_active.get();

    (start, cancel, check_active)
}
