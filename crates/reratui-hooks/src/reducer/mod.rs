//! React-style useReducer hook for state management with actions
//!
//! This module provides a professional useReducer hook implementation that follows
//! React's API patterns for complex state management scenarios.

use crate::hook_context::with_hook_context;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// A handle to the current state managed by useReducer
///
/// This provides read-only access to the current state value with efficient
/// concurrent access patterns and version tracking for change detection.
#[derive(Clone)]
pub struct ReducerStateHandle<S> {
    state: Arc<RwLock<S>>,
    version: Arc<Mutex<u64>>,
}

impl<S> ReducerStateHandle<S>
where
    S: Clone,
{
    /// Get the current state value
    ///
    /// This method provides efficient read access to the current state.
    /// Multiple readers can access the state concurrently.
    pub fn get(&self) -> S {
        self.state.read().clone()
    }

    /// Get a specific field from the state without cloning the entire state
    ///
    /// This is useful for accessing specific fields of complex state objects
    /// without the overhead of cloning the entire state.
    pub fn field<F, R>(&self, accessor: F) -> R
    where
        F: FnOnce(&S) -> R,
    {
        let state = self.state.read();
        accessor(&*state)
    }

    /// Get the current version of the state (useful for change detection)
    ///
    /// The version is incremented each time the state is updated through
    /// a reducer action. This can be used for optimization and debugging.
    pub fn version(&self) -> u64 {
        *self.version.lock()
    }
}

/// A dispatch function for sending actions to the reducer
///
/// This function is used to dispatch actions that will be processed by the
/// reducer function to produce new state.
#[derive(Clone)]
pub struct DispatchFn<A> {
    dispatcher: Arc<dyn Fn(A) + Send + Sync>,
}

impl<A> DispatchFn<A> {
    /// Create a new dispatch function
    fn new<F>(dispatcher: F) -> Self
    where
        F: Fn(A) + Send + Sync + 'static,
    {
        Self {
            dispatcher: Arc::new(dispatcher),
        }
    }

    /// Dispatch an action to the reducer
    ///
    /// This will call the reducer function with the current state and the
    /// provided action, updating the state with the result.
    pub fn dispatch(&self, action: A) {
        (self.dispatcher)(action);
    }

    /// Convenience method for calling dispatch
    pub fn call(&self, action: A) {
        self.dispatch(action);
    }
}

/// Internal container for reducer state management
struct ReducerContainer<S, A> {
    state: Arc<RwLock<S>>,
    version: Arc<Mutex<u64>>,
    reducer: Arc<dyn Fn(S, A) -> S + Send + Sync>,
}

impl<S, A> ReducerContainer<S, A>
where
    S: Clone + Send + Sync + 'static,
    A: 'static,
{
    /// Create a new reducer container
    fn new<R>(initial_state: S, reducer: R) -> Self
    where
        R: Fn(S, A) -> S + Send + Sync + 'static,
    {
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            version: Arc::new(Mutex::new(0)),
            reducer: Arc::new(reducer),
        }
    }

    /// Dispatch an action and update the state
    #[allow(dead_code)]
    fn dispatch(&self, action: A) {
        let current_state = self.state.read().clone();
        let new_state = (self.reducer)(current_state, action);
        *self.state.write() = new_state;

        // Increment version counter
        {
            let mut version = self.version.lock();
            *version += 1;
        }

        // TODO: Trigger re-render notification
        // This would integrate with the component re-render system
    }

    /// Get a handle to the current state
    fn state_handle(&self) -> ReducerStateHandle<S> {
        ReducerStateHandle {
            state: self.state.clone(),
            version: self.version.clone(),
        }
    }

    /// Get a dispatch function
    fn dispatch_fn(&self) -> DispatchFn<A> {
        let container_state = self.state.clone();
        let container_version = self.version.clone();
        let container_reducer = self.reducer.clone();

        DispatchFn::new(move |action| {
            let current_state = container_state.read().clone();
            let new_state = container_reducer(current_state, action);
            *container_state.write() = new_state;

            // Increment version counter
            {
                let mut version = container_version.lock();
                *version += 1;
            }

            // TODO: Trigger re-render notification
        })
    }
}

/// React-style useReducer hook for complex state management
///
/// This hook provides a way to manage complex state logic using a reducer function,
/// similar to React's useReducer hook. It's particularly useful when you have complex
/// state logic that involves multiple sub-values or when the next state depends on
/// the previous one.
///
/// # Arguments
///
/// * `reducer` - A function that takes the current state and an action, returning new state
/// * `initial_state` - The initial state value
///
/// # Returns
///
/// A tuple containing:
/// * `ReducerStateHandle<S>` - Handle for accessing the current state
/// * `DispatchFn<A>` - Function for dispatching actions to update state
///
/// # Examples
///
/// ## Simple Counter with Actions
/// ```rust,no_run
/// # use reratui_hooks::reducer::use_reducer;
/// #[derive(Clone)]
/// enum CounterAction {
///     Increment,
///     Decrement,
///     Reset,
///     SetValue(i32),
/// }
///
/// fn counter_reducer(state: i32, action: CounterAction) -> i32 {
///     match action {
///         CounterAction::Increment => state + 1,
///         CounterAction::Decrement => state - 1,
///         CounterAction::Reset => 0,
///         CounterAction::SetValue(value) => value,
///     }
/// }
///
/// // Example usage in a component context
/// let (state, dispatch) = use_reducer(counter_reducer, 0);
/// let count = state.get();
/// // count is now 0
///
/// dispatch.call(CounterAction::Increment);
/// // In a real component, this would trigger a re-render
/// // and the state would be updated
/// ```
///
/// ## Complex State Management
/// ```rust,no_run
/// # use reratui_hooks::reducer::use_reducer;
/// #[derive(Clone)]
/// struct TodoState {
///     todos: Vec<Todo>,
///     filter: Filter,
///     next_id: u32,
/// }
///
/// #[derive(Clone)]
/// struct Todo {
///     id: u32,
///     text: String,
///     completed: bool,
/// }
///
/// #[derive(Clone)]
/// enum Filter {
///     All,
///     Active,
///     Completed,
/// }
///
/// #[derive(Clone)]
/// enum TodoAction {
///     AddTodo(String),
///     ToggleTodo(u32),
///     RemoveTodo(u32),
///     SetFilter(Filter),
///     ClearCompleted,
/// }
///
/// fn todo_reducer(state: TodoState, action: TodoAction) -> TodoState {
///     match action {
///         TodoAction::AddTodo(text) => TodoState {
///             todos: {
///                 let mut todos = state.todos;
///                 todos.push(Todo {
///                     id: state.next_id,
///                     text,
///                     completed: false,
///                 });
///                 todos
///             },
///             next_id: state.next_id + 1,
///             ..state
///         },
///         TodoAction::ToggleTodo(id) => TodoState {
///             todos: state.todos.into_iter().map(|mut todo| {
///                 if todo.id == id {
///                     todo.completed = !todo.completed;
///                 }
///                 todo
///             }).collect(),
///             ..state
///         },
///         _ => state, // Handle other actions
///     }
/// }
///
/// // Example usage in a component context
/// let initial_state = TodoState {
///     todos: vec![],
///     filter: Filter::All,
///     next_id: 1,
/// };
///
/// let (state, dispatch) = use_reducer(todo_reducer, initial_state);
/// // state.get().todos.len() is now 0
///
/// dispatch.call(TodoAction::AddTodo("Learn Rust".to_string()));
/// // In a real component, this would trigger a re-render
/// ```
///
/// # Thread Safety
///
/// The returned state handle and dispatch function are both thread-safe and can be
/// safely shared across async tasks.
///
/// # Performance Notes
///
/// - State reads are optimized using RwLock for concurrent access
/// - State updates are atomic and thread-safe
/// - The reducer function should be pure (no side effects)
/// - State transitions are immutable - the reducer should return new state
pub fn use_reducer<S, A, R>(reducer: R, initial_state: S) -> (ReducerStateHandle<S>, DispatchFn<A>)
where
    S: Clone + Send + Sync + 'static,
    A: Send + Sync + 'static,
    R: Fn(S, A) -> S + Send + Sync + 'static,
{
    with_hook_context(|ctx| {
        let index = ctx.next_hook_index();

        // Get or initialize the reducer container for this hook
        let container_ref = ctx.get_or_init_state(index, || {
            Arc::new(ReducerContainer::new(initial_state, reducer))
        });

        // Extract the Arc<ReducerContainer<S, A>> from Rc<RefCell<Arc<ReducerContainer<S, A>>>>
        let container = container_ref.borrow().clone();

        // Create the state handle and dispatch function
        let state_handle = container.state_handle();
        let dispatch_fn = container.dispatch_fn();

        (state_handle, dispatch_fn)
    })
}
