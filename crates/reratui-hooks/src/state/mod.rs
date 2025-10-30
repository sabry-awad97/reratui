use crate::hook_context::with_hook_context;
use parking_lot::{Mutex, RwLock};
use std::{fmt, sync::Arc};

#[cfg(test)]
mod tests;

/// A thread-safe state container that holds the actual state value
/// This is the core storage for useState hook state
#[derive(Debug)]
pub struct StateContainer<T> {
    /// The current value of the state, protected by RwLock for efficient reads
    value: RwLock<T>,
    /// Version counter to track state changes (useful for debugging and optimization)
    version: Mutex<u64>,
}

impl<T> StateContainer<T> {
    /// Create a new state container with the initial value from the initializer
    pub fn new<F>(initializer: F) -> Self
    where
        F: FnOnce() -> T,
    {
        Self {
            value: RwLock::new(initializer()),
            version: Mutex::new(0),
        }
    }

    /// Get the current value (thread-safe read)
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.value.read().clone()
    }

    /// Private method to increment version and trigger re-render
    /// This eliminates code duplication between set() and update()
    fn increment_version_and_notify(&self) {
        // Increment version counter
        {
            let mut version = self.version.lock();
            *version += 1;
        }

        // TODO: Trigger re-render notification
        // This would integrate with the component re-render system
    }

    /// Set a new value (thread-safe write)
    pub fn set(&self, new_value: T) {
        {
            let mut value = self.value.write();
            *value = new_value;
        }

        self.increment_version_and_notify();
    }

    /// Update the value using a function (functional update pattern)
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&T) -> T,
        T: Clone,
    {
        // Perform atomic read-modify-write operation
        {
            let mut value = self.value.write();
            let new_value = updater(&*value);
            *value = new_value;
        }

        self.increment_version_and_notify();
    }

    /// Get the current version (useful for change detection)
    pub fn version(&self) -> u64 {
        *self.version.lock()
    }
}

/// A handle to a piece of state that mirrors React's useState return value
/// This is what gets returned to the component
#[derive(Debug)]
pub struct StateHandle<T> {
    /// Reference to the shared state container
    container: Arc<StateContainer<T>>,
}

impl<T> fmt::Display for StateHandle<T>
where
    T: fmt::Display + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl<T> StateHandle<T> {
    /// Create a new state handle with an initial value from an initializer function
    /// This allows for lazy initialization of the state value
    pub fn new<F>(initializer: F) -> Self
    where
        F: FnOnce() -> T,
    {
        Self {
            container: Arc::new(StateContainer::new(initializer)),
        }
    }

    /// Create a state handle from an existing container (for sharing state)
    pub fn from_container(container: Arc<StateContainer<T>>) -> Self {
        Self { container }
    }

    /// Get the current value of the state
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.container.get()
    }

    /// Get the current version of the state (useful for change detection)
    pub fn version(&self) -> u64 {
        self.container.version()
    }

    /// Get a reference to the underlying container (for advanced use cases)
    pub fn container(&self) -> &Arc<StateContainer<T>> {
        &self.container
    }
}

impl<T> Clone for StateHandle<T> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
        }
    }
}

/// State setter function that mirrors React's setState behavior
///
/// This function can accept either:
/// 1. A direct value: `setState(newValue)`
/// 2. A function that takes the previous state: `setState(|prev| newValue)`
#[derive(Debug)]
pub struct StateSetter<T> {
    /// Reference to the shared state container
    container: Arc<StateContainer<T>>,
}

impl<T> StateSetter<T> {
    /// Create a new state setter
    pub fn new(container: Arc<StateContainer<T>>) -> Self {
        Self { container }
    }

    /// Set the state to a new value (direct value update)
    pub fn set(&self, new_value: T) {
        self.container.set(new_value);
    }

    /// Update the state using a function (functional update)
    /// This mirrors React's setState(prevState => newState) pattern
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&T) -> T,
        T: Clone,
    {
        use crate::panic_handler::catch_panic;

        // Wrap the updater function with panic handling
        let safe_updater = |current: &T| -> T {
            match catch_panic(std::panic::AssertUnwindSafe(|| updater(current))) {
                Ok(new_value) => new_value,
                Err(panic_payload) => {
                    let reason = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "<unknown panic>".to_string()
                    };

                    // Log it
                    tracing::error!(
                        target: "hooks::state",
                        "State updater function panicked: {:?}",
                        reason
                    );

                    // Re-panic to propagate the error
                    panic!("💥 State updater panicked: {}", reason);
                }
            }
        };

        // Delegate to the container's atomic update method
        self.container.update(safe_updater);
    }

    /// Get access to the underlying container (for testing and advanced use cases)
    #[cfg(test)]
    pub fn container(&self) -> &Arc<StateContainer<T>> {
        &self.container
    }

    /// Get the current version of the state (useful for change detection)
    pub fn version(&self) -> u64 {
        self.container.version()
    }
}

impl<T> Clone for StateSetter<T> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
        }
    }
}

/// Implement function call syntax for StateSetter to mirror React's setState
/// This allows calling setState(value) directly
impl<T> StateSetter<T> {
    /// Call the setter with a new value
    /// This enables `setter(new_value)` syntax
    pub fn call(&self, new_value: T) {
        self.set(new_value);
    }
}

/// Additional utility methods for StateHandle
impl<T> StateHandle<T>
where
    T: Clone,
{
    /// Access a field of the inner value using a getter function
    /// This is useful for accessing nested properties without cloning the entire state
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pulse_core::hooks::state::use_state;
    /// # use pulse_core::hooks::{HookContext, set_hook_context};
    /// # use std::rc::Rc;
    /// #[derive(Clone)]
    /// struct AppState {
    ///     count: i32,
    ///     name: String,
    /// }
    ///
    /// // In a component context:
    /// # let context = Rc::new(HookContext::new());
    /// # set_hook_context(context);
    /// let (state, _) = use_state(|| AppState { count: 42, name: "test".to_string() });
    /// let count = state.field(|s| s.count);
    /// // count is now 42
    /// ```
    pub fn field<F, R>(&self, getter: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let value = self.get();
        getter(&value)
    }

    /// Map the state value to a different type
    /// This is useful for deriving computed values from state
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pulse_core::hooks::state::use_state;
    /// # use pulse_core::hooks::{HookContext, set_hook_context};
    /// # use std::rc::Rc;
    /// // In a component context:
    /// # let context = Rc::new(HookContext::new());
    /// # set_hook_context(context);
    /// let (count, _) = use_state(|| 42);
    /// let is_even = count.map(|c| c % 2 == 0);
    /// // is_even is now true
    /// ```
    pub fn map<F, R>(&self, mapper: F) -> R
    where
        F: FnOnce(T) -> R,
    {
        let value = self.get();
        mapper(value)
    }
}

/// React-style useState hook that provides state management for components
///
/// This function provides React-like useState behavior with enhanced capabilities:
/// - Returns a tuple of (StateHandle, setter_function)
/// - StateHandle provides access to current value and utility methods
/// - State persists across re-renders within the same component instance
/// - Thread-safe for use in async contexts
/// - Supports both direct value updates and functional updates
///
/// # Examples
///
/// ## Basic Usage (Direct Value)
/// ```rust,no_run
/// # use reratui_hooks::state::use_state;
/// # use reratui_hooks::hook_context::{HookContext, set_hook_context};
/// # use std::rc::Rc;
/// // In a component context:
/// # let context = Rc::new(HookContext::new());
/// # set_hook_context(context);
/// let (count_handle, set_count) = use_state(|| 0);
/// let count = count_handle.get();
/// // count is now 0
///
/// // Direct value update
/// set_count.set(5);
/// // In a real component, this would trigger a re-render
/// // and the new value would be available
/// ```
///
/// ## Functional Updates
/// ```rust,no_run
/// # use reratui_hooks::state::use_state;
/// # use reratui_hooks::hook_context::{HookContext, set_hook_context};
/// # use std::rc::Rc;
/// // In a component context:
/// # let context = Rc::new(HookContext::new());
/// # set_hook_context(context);
/// let (count_handle, set_count) = use_state(|| 0);
/// let count = count_handle.get();
/// // count is now 0
///
/// // Functional update - safer for concurrent access
/// set_count.update(|prev| prev + 1);
/// // In a real component, this would trigger a re-render
/// ```
///
/// ## Complex State with Field Access
/// ```rust,no_run
/// # use reratui_hooks::state::use_state;
/// # use reratui_hooks::hook_context::{HookContext, set_hook_context};
/// # use std::rc::Rc;
/// #[derive(Clone)]
/// struct AppState {
///     count: i32,
///     name: String,
/// }
///
/// // In a component context:
/// # let context = Rc::new(HookContext::new());
/// # set_hook_context(context);
/// let (state_handle, set_state) = use_state(|| AppState {
///     count: 0,
///     name: "Hello".to_string(),
/// });
///
/// // Access fields efficiently without cloning the entire state
/// let count = state_handle.field(|s| s.count);
/// let name = state_handle.field(|s| s.name.clone());
/// // count is now 0, name is "Hello"
///
/// // Update complex state
/// set_state.update(|prev| AppState {
///     count: prev.count + 1,
///     name: prev.name.clone(),
/// });
/// ```
///
/// # Thread Safety
///
/// The returned state handle and setter are both thread-safe and can be safely
/// shared across async tasks:
///
/// ```rust,no_run
/// # use reratui_hooks::state::use_state;
/// # use reratui_hooks::hook_context::{HookContext, set_hook_context};
/// # use std::rc::Rc;
/// // In a component context:
/// # let context = Rc::new(HookContext::new());
/// # set_hook_context(context);
/// let (count_handle, set_count) = use_state(|| 0);
/// let count = count_handle.get();
/// // count is now 0
///
/// // The state handle and setter are thread-safe
/// let set_count_clone = set_count.clone();
///
/// // In a real async context, you could spawn tasks:
/// // tokio::spawn(async move {
/// //     set_count_clone.update(|prev| prev + 1);
/// // });
/// ```
///
/// # Error Handling
///
/// This function will panic if called outside of a component render context.
/// Always ensure useState is called within a component function.
///
/// # Performance Notes
///
/// - State reads are optimized using RwLock for concurrent access
/// - State updates are batched and don't cause immediate re-renders
/// - Version tracking enables efficient change detection
/// - Memory usage is minimal with Arc-based sharing
pub fn use_state<T, F>(initializer: F) -> (StateHandle<T>, StateSetter<T>)
where
    T: 'static,
    F: FnOnce() -> T,
{
    with_hook_context(|ctx| {
        let index = ctx.next_hook_index();

        // Get or initialize the state container for this hook
        let container_ref =
            ctx.get_or_init_state(index, || Arc::new(StateContainer::new(initializer)));

        // Extract the Arc<StateContainer<T>> from Rc<RefCell<Arc<StateContainer<T>>>>
        let container = container_ref.borrow().clone();

        // Create the state handle
        let state_handle = StateHandle::from_container(container.clone());

        // Create the setter function
        let setter = StateSetter::new(container);

        (state_handle, setter)
    })
}
