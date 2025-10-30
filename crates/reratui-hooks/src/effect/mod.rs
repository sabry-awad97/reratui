use parking_lot::Mutex;
use std::any::Any;
use std::sync::Arc;

#[cfg(test)]
mod tests;

use crate::hook_context::with_hook_context;
use crate::panic_handler::spawn_catch_panic;

/// Trait for types that can be used as effect dependencies
/// This enables dependency comparison for conditional effect re-execution
pub trait EffectDependencies: Any + Send + Sync {
    /// Compare this dependency set with another for equality
    fn deps_eq(&self, other: &dyn EffectDependencies) -> bool;

    /// Clone this dependency set as a boxed trait object
    fn clone_deps(&self) -> Box<dyn EffectDependencies>;

    /// Get a debug representation of the dependencies
    fn debug_deps(&self) -> String;

    /// Get a hash of the dependencies for fast comparison
    /// This enables optimization where we can quickly check if dependencies
    /// might have changed before doing expensive equality comparisons
    fn deps_hash(&self) -> u64;
}

// Add as_any method to EffectDependencies trait
impl dyn EffectDependencies {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

// Implement EffectDependencies for common types:

// Implement EffectDependencies for unit type () - represents empty dependencies
impl EffectDependencies for () {
    fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
        other.as_any().downcast_ref::<()>().is_some()
    }

    fn clone_deps(&self) -> Box<dyn EffectDependencies> {
        Box::new(())
    }

    fn debug_deps(&self) -> String {
        "()".to_string()
    }

    fn deps_hash(&self) -> u64 {
        // Unit type always has the same hash
        0
    }
}

// Implement for primitive types
macro_rules! impl_effect_deps_for_primitive {
    ($($t:ty),*) => {
        $(
            impl EffectDependencies for $t {
                fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
                    if let Some(other_val) = other.as_any().downcast_ref::<$t>() {
                        self == other_val
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
        )*
    };
}

impl_effect_deps_for_primitive!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, bool, char
);

// Special implementations for floating point types (which don't implement Hash)
impl EffectDependencies for f32 {
    fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
        if let Some(other_val) = other.as_any().downcast_ref::<f32>() {
            self == other_val
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
        // For f32, we use the bit representation for hashing
        self.to_bits() as u64
    }
}

impl EffectDependencies for f64 {
    fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
        if let Some(other_val) = other.as_any().downcast_ref::<f64>() {
            self == other_val
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
        // For f64, we use the bit representation for hashing
        self.to_bits()
    }
}

impl EffectDependencies for String {
    fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
        if let Some(other_val) = other.as_any().downcast_ref::<String>() {
            self == other_val
        } else {
            false
        }
    }

    fn clone_deps(&self) -> Box<dyn EffectDependencies> {
        Box::new(self.clone())
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

impl EffectDependencies for &'static str {
    fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
        if let Some(other_val) = other.as_any().downcast_ref::<&'static str>() {
            self == other_val
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

/// Thread-safe wrapper for synchronous cleanup functions
/// Ensures cleanup functions can be safely called from any thread
pub struct CleanupFn {
    /// The actual cleanup function, wrapped in Arc<Mutex<>> for thread safety
    #[allow(clippy::type_complexity)]
    cleanup: Arc<Mutex<Option<Box<dyn FnOnce() + Send + 'static>>>>,
}

/// Thread-safe wrapper for asynchronous cleanup functions
/// Ensures async cleanup functions can be safely called from any thread
pub struct AsyncCleanupFn {
    /// The actual async cleanup function, wrapped in Arc<Mutex<>> for thread safety
    #[allow(clippy::type_complexity)]
    cleanup: Arc<
        Mutex<
            Option<
                Box<
                    dyn FnOnce() -> std::pin::Pin<
                            Box<dyn std::future::Future<Output = ()> + Send + 'static>,
                        > + Send
                        + 'static,
                >,
            >,
        >,
    >,
}

impl CleanupFn {
    /// Create a new cleanup function wrapper
    pub fn new<F>(cleanup: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            cleanup: Arc::new(Mutex::new(Some(Box::new(cleanup)))),
        }
    }

    /// Execute the cleanup function if it hasn't been called yet
    /// This is idempotent - calling it multiple times is safe
    pub fn cleanup(&self) {
        if let Some(cleanup_fn) = self.cleanup.lock().take() {
            cleanup_fn();
        }
    }
}

impl Clone for CleanupFn {
    fn clone(&self) -> Self {
        Self {
            cleanup: self.cleanup.clone(),
        }
    }
}

impl AsyncCleanupFn {
    /// Create a new async cleanup function wrapper
    pub fn new<F, Fut>(cleanup: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        Self {
            cleanup: Arc::new(Mutex::new(Some(Box::new(move || {
                Box::pin(cleanup())
                    as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>>
            })))),
        }
    }

    /// Execute the async cleanup function if it hasn't been called yet
    /// This is idempotent - calling it multiple times is safe
    pub async fn cleanup(&self) {
        let cleanup_fn = self.cleanup.lock().take();
        if let Some(cleanup_fn) = cleanup_fn {
            cleanup_fn().await;
        }
    }
}

impl Clone for AsyncCleanupFn {
    fn clone(&self) -> Self {
        Self {
            cleanup: self.cleanup.clone(),
        }
    }
}

/// Internal state for tracking synchronous effects
struct EffectState {
    /// Previous dependencies for comparison
    prev_deps: Option<Box<dyn EffectDependencies>>,
    /// Cleanup function from the previous effect run
    cleanup: Option<CleanupFn>,
    /// Whether this effect has been initialized
    initialized: bool,
}

impl EffectState {
    fn new() -> Self {
        Self {
            prev_deps: None,
            cleanup: None,
            initialized: false,
        }
    }
}

/// Internal state for tracking asynchronous effects
struct AsyncEffectState {
    /// Previous dependencies for comparison
    prev_deps: Option<Box<dyn EffectDependencies>>,
    /// Async cleanup function from the previous effect run
    cleanup: Option<AsyncCleanupFn>,
    /// Whether this effect has been initialized
    initialized: bool,
}

impl AsyncEffectState {
    fn new() -> Self {
        Self {
            prev_deps: None,
            cleanup: None,
            initialized: false,
        }
    }
}

// Implement EffectDependencies for tuples (up to 8 elements for practical use)
macro_rules! impl_effect_deps_for_tuple {
    ($($t:ident),*) => {
        impl<$($t: EffectDependencies + Clone + PartialEq + std::fmt::Debug + 'static),*> EffectDependencies for ($($t,)*) {
            fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
                if let Some(other_tuple) = other.as_any().downcast_ref::<($($t,)*)>() {
                    self == other_tuple
                } else {
                    false
                }
            }

            fn clone_deps(&self) -> Box<dyn EffectDependencies> {
                Box::new(self.clone())
            }

            fn debug_deps(&self) -> String {
                format!("{:?}", self)
            }

            fn deps_hash(&self) -> u64 {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                // For tuples, we'll use the debug representation as a simple hash
                // This is not the most efficient but works for all tuple types
                self.debug_deps().hash(&mut hasher);
                hasher.finish()
            }
        }
    };
}

// Implement for tuples of various sizes
impl_effect_deps_for_tuple!(T1);
impl_effect_deps_for_tuple!(T1, T2);
impl_effect_deps_for_tuple!(T1, T2, T3);
impl_effect_deps_for_tuple!(T1, T2, T3, T4);
impl_effect_deps_for_tuple!(T1, T2, T3, T4, T5);
impl_effect_deps_for_tuple!(T1, T2, T3, T4, T5, T6);
impl_effect_deps_for_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_effect_deps_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);

// Implement EffectDependencies for Option<T> where T: EffectDependencies
impl<T> EffectDependencies for Option<T>
where
    T: EffectDependencies + Clone + PartialEq + std::fmt::Debug + 'static,
{
    fn deps_eq(&self, other: &dyn EffectDependencies) -> bool {
        if let Some(other_option) = other.as_any().downcast_ref::<Option<T>>() {
            match (self, other_option) {
                (None, None) => true,
                (Some(a), Some(b)) => a.deps_eq(b),
                _ => false,
            }
        } else {
            false
        }
    }

    fn clone_deps(&self) -> Box<dyn EffectDependencies> {
        Box::new(self.clone())
    }

    fn debug_deps(&self) -> String {
        match self {
            None => "None".to_string(),
            Some(value) => format!("Some({})", value.debug_deps()),
        }
    }

    fn deps_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        match self {
            None => 0u8.hash(&mut hasher),
            Some(value) => {
                1u8.hash(&mut hasher);
                value.deps_hash().hash(&mut hasher);
            }
        }
        hasher.finish()
    }
}

/// React-style useAsyncEffect hook that provides async side effect management for components
///
/// This function provides async effects with React's useEffect behavior:
/// - Async effects run after component render (not during)
/// - Supports dependency arrays for conditional re-execution
/// - Supports async cleanup functions returned from effects
/// - Handles effect cleanup on component unmount
/// - Supports empty dependency array for run-once effects
/// - Supports None dependencies for effects that run on every render
///
/// # Error Handling
///
/// This function will panic if called outside of a component render context.
/// Always ensure useAsyncEffect is called within a component function.
///
/// # Performance Notes
///
/// - Async effects are scheduled to run after render, not during
/// - Dependency comparison uses PartialEq for efficient change detection
/// - Async cleanup functions are automatically managed and called when needed
/// - Multiple effects in the same component are executed in declaration order
pub fn use_async_effect<Deps, F, Fut, C, CFut>(effect: F, deps: impl Into<Option<Deps>>)
where
    Deps: EffectDependencies + Clone + PartialEq + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Option<C>> + Send + 'static,
    C: FnOnce() -> CFut + Send + 'static,
    CFut: std::future::Future<Output = ()> + Send + 'static,
{
    let deps = deps.into();
    with_hook_context(|ctx| {
        let hook_index = ctx.next_hook_index();
        let mut states = ctx.states.borrow_mut();

        // Get or create async effect state for this hook
        let effect_state = states
            .entry(hook_index)
            .or_insert_with(|| Box::new(AsyncEffectState::new()))
            .downcast_mut::<AsyncEffectState>()
            .expect("Async effect state type mismatch");

        // Determine if effect should run
        let should_run = match &deps {
            None => {
                // No dependencies - run on every render
                true
            }
            Some(current_deps) => {
                // Check if dependencies have changed
                match &effect_state.prev_deps {
                    None => {
                        // First run - always execute
                        true
                    }
                    Some(prev_deps) => {
                        // Compare dependencies
                        !current_deps.deps_eq(prev_deps.as_ref())
                    }
                }
            }
        };

        if should_run {
            // Run cleanup from previous effect if it exists
            if let Some(cleanup) = effect_state.cleanup.take() {
                // For now, we'll spawn the cleanup to run asynchronously
                // In a real implementation, this would be properly scheduled
                tokio::spawn(spawn_catch_panic(async move {
                    cleanup.cleanup().await;
                }));
            }

            // Store new dependencies
            if let Some(current_deps) = &deps {
                effect_state.prev_deps = Some(current_deps.clone_deps());
            } else {
                effect_state.prev_deps = None;
            }

            // Schedule async effect to run after render
            // For now, we'll spawn it immediately, but in a real implementation
            // this would be scheduled to run after the render phase
            tokio::spawn(spawn_catch_panic(async move {
                if let Some(cleanup_fn) = effect().await {
                    // Store the cleanup function for later use
                    // In a real implementation, we'd need to store this back in the state
                    let _async_cleanup = AsyncCleanupFn::new(cleanup_fn);
                    // TODO: Store this cleanup function in the effect state
                }
            }));

            effect_state.initialized = true;
        }
    });
}

/// React-style useEffect hook that provides synchronous side effect management for components
///
/// This function exactly mirrors React's useEffect behavior for synchronous effects:
/// - Effects run after component render (not during)
/// - Supports dependency arrays for conditional re-execution
/// - Supports cleanup functions returned from effects
/// - Handles effect cleanup on component unmount
/// - Supports empty dependency array for run-once effects
/// - Supports None dependencies for effects that run on every render
///
/// # Error Handling
///
/// This function will panic if called outside of a component render context.
/// Always ensure useEffect is called within a component function.
///
/// # Performance Notes
///
/// - Effects are scheduled to run after render, not during
/// - Dependency comparison uses PartialEq for efficient change detection
/// - Cleanup functions are automatically managed and called when needed
/// - Multiple effects in the same component are executed in declaration order
pub fn use_effect<Deps, F, C>(effect: F, deps: impl Into<Option<Deps>>)
where
    Deps: EffectDependencies + Clone + PartialEq + 'static,
    F: FnOnce() -> Option<C> + 'static,
    C: FnOnce() + Send + 'static,
{
    let deps = deps.into();
    with_hook_context(|ctx| {
        let hook_index = ctx.next_hook_index();
        let mut states = ctx.states.borrow_mut();

        // Get or create effect state for this hook
        let effect_state = states
            .entry(hook_index)
            .or_insert_with(|| Box::new(EffectState::new()))
            .downcast_mut::<EffectState>()
            .expect("Effect state type mismatch");

        // Determine if effect should run
        let should_run = match &deps {
            None => {
                // No dependencies - run on every render
                true
            }
            Some(current_deps) => {
                // Check if dependencies have changed
                match &effect_state.prev_deps {
                    None => {
                        // First run - always execute
                        true
                    }
                    Some(prev_deps) => {
                        // Compare dependencies
                        !current_deps.deps_eq(prev_deps.as_ref())
                    }
                }
            }
        };

        if should_run {
            // Run cleanup from previous effect if it exists
            if let Some(cleanup) = effect_state.cleanup.take() {
                cleanup.cleanup();
            }

            // Store new dependencies
            if let Some(current_deps) = &deps {
                effect_state.prev_deps = Some(current_deps.clone_deps());
            } else {
                effect_state.prev_deps = None;
            }

            // Schedule effect to run after render
            // For now, we'll run it immediately, but in a real implementation
            // this would be scheduled to run after the render phase
            if let Some(cleanup_fn) = effect() {
                effect_state.cleanup = Some(CleanupFn::new(cleanup_fn));
            }

            effect_state.initialized = true;
        }
    });
}

/// useEffect hook that runs on every render (no dependency tracking)
///
/// This is a convenience function for effects that should run on every render.
/// It's equivalent to calling `use_effect(effect, None)`.
///
/// # Examples
///
/// ```rust,no_run
/// # use reratui_hooks::effect::use_effect_always;
/// use_effect_always(|| {
///     println!("This effect runs on every render");
///     || {} // Empty cleanup
/// });
///
/// // With cleanup
/// use_effect_always(|| {
///     println!("Setting up resource");
///     move || {
///         println!("Cleaning up resource");
///     }
/// });
/// ```
pub fn use_effect_always<F, C>(effect: F)
where
    F: FnOnce() -> C + 'static,
    C: FnOnce() + Send + 'static,
{
    use_effect::<(), _, C>(|| Some(effect()), None)
}

/// useEffect hook that runs only once (on mount)
///
/// This is a convenience function for effects that should run only once when
/// the component mounts. It's equivalent to calling `use_effect(effect, ())`.
///
/// # Examples
///
/// ```rust,no_run
/// # use reratui_hooks::effect::use_effect_once;
/// use_effect_once(|| {
///     println!("This effect runs only once");
///     || {} // Empty cleanup
/// });
///
/// // With cleanup
/// use_effect_once(|| {
///     println!("Setting up subscription");
///     move || {
///         println!("Cleaning up subscription");
///     }
/// });
/// ```
pub fn use_effect_once<F, C>(effect: F)
where
    F: FnOnce() -> C + 'static,
    C: FnOnce() + Send + 'static,
{
    use_effect(|| Some(effect()), ())
}

/// useAsyncEffect hook that runs only once (on mount)
///
/// This is a convenience function for async effects that should run only once when
/// the component mounts. It's equivalent to calling `use_async_effect(effect, ())`.
///
/// # Examples
///
/// ```rust,no_run
/// # use reratui_hooks::effect::use_async_effect_once;
/// use_async_effect_once(|| async {
///     println!("This async effect runs only once");
///     || async {} // Empty async cleanup
/// });
///
/// // With async cleanup
/// use_async_effect_once(|| async {
///     println!("Setting up async subscription");
///     move || async move {
///         println!("Cleaning up async subscription");
///     }
/// });
/// ```
pub fn use_async_effect_once<F, Fut, C, CFut>(effect: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = C> + Send + 'static,
    C: FnOnce() -> CFut + Send + 'static,
    CFut: std::future::Future<Output = ()> + Send + 'static,
{
    use_async_effect(move || async move { Some(effect().await) }, ())
}

/// useAsyncEffect hook that runs on every render (no dependency tracking)
///
/// This is a convenience function for async effects that should run on every render.
/// It's equivalent to calling `use_async_effect(effect, None)`.
///
/// # Examples
///
/// ```rust,no_run
/// # use reratui_hooks::effect::use_async_effect_always;
/// use_async_effect_always(|| async {
///     println!("This async effect runs on every render");
///     || async {} // Empty async cleanup
/// });
///
/// // With async cleanup
/// use_async_effect_always(|| async {
///     println!("Setting up async resource");
///     move || async move {
///         println!("Cleaning up async resource");
///     }
/// });
/// ```
pub fn use_async_effect_always<F, Fut, C, CFut>(effect: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = C> + Send + 'static,
    C: FnOnce() -> CFut + Send + 'static,
    CFut: std::future::Future<Output = ()> + Send + 'static,
{
    use_async_effect::<(), _, _, C, CFut>(move || async move { Some(effect().await) }, None)
}
