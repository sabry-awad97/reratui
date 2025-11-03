use crate::reducer::{DispatchFn, use_reducer};
use std::collections::VecDeque;

#[cfg(test)]
pub mod tests;

/// Represents the state of the history manager
#[derive(Clone, PartialEq)]
pub struct HistoryState<T> {
    /// The current state
    current: T,
    /// Past states for undo
    past: VecDeque<T>,
    /// Future states for redo
    future: VecDeque<T>,
    /// Maximum number of history entries
    max_history: usize,
}

/// Actions that can be performed on the history
#[derive(Clone)]
pub enum HistoryAction<T: Clone> {
    /// Push a new state
    Push(T),
    /// Undo the last change
    Undo,
    /// Redo the last undone change
    Redo,
}

/// A manager for state history with undo/redo functionality
#[derive(Clone)]
pub struct HistoryManager<T: Clone> {
    /// Function to get the current state
    get_state: std::sync::Arc<dyn Fn() -> HistoryState<T> + Send + Sync>,
    /// Dispatch function for actions
    dispatch: DispatchFn<HistoryAction<T>>,
}

impl<T: Clone + 'static> HistoryManager<T> {
    /// Gets the current state
    pub fn current(&self) -> T {
        (self.get_state)().current.clone()
    }

    /// Checks if undo is available
    pub fn can_undo(&self) -> bool {
        !(self.get_state)().past.is_empty()
    }

    /// Checks if redo is available
    pub fn can_redo(&self) -> bool {
        !(self.get_state)().future.is_empty()
    }

    /// Pushes a new state to the history
    pub fn push(&self, new_state: T) {
        self.dispatch.dispatch(HistoryAction::Push(new_state));
    }

    /// Undoes the last change
    pub fn undo(&self) {
        if self.can_undo() {
            self.dispatch.dispatch(HistoryAction::Undo);
        }
    }

    /// Redoes the last undone change
    pub fn redo(&self) {
        if self.can_redo() {
            self.dispatch.dispatch(HistoryAction::Redo);
        }
    }
}

fn history_reducer<T: Clone>(state: HistoryState<T>, action: HistoryAction<T>) -> HistoryState<T> {
    match action {
        HistoryAction::Push(new_state) => {
            let mut past = state.past;
            past.push_back(state.current);

            // Trim history from the front if it exceeds max size
            while past.len() > state.max_history {
                past.pop_front();
            }

            HistoryState {
                current: new_state,
                past,
                future: VecDeque::new(), // Clear redo stack
                max_history: state.max_history,
            }
        }
        HistoryAction::Undo => {
            if state.past.is_empty() {
                return state;
            }

            let mut past = state.past;
            let mut future = state.future;

            let previous = past.pop_back().unwrap();
            future.push_front(state.current);

            HistoryState {
                current: previous,
                past,
                future,
                max_history: state.max_history,
            }
        }
        HistoryAction::Redo => {
            if state.future.is_empty() {
                return state;
            }

            let mut past = state.past;
            let mut future = state.future;

            let next = future.pop_front().unwrap();
            past.push_back(state.current);

            HistoryState {
                current: next,
                past,
                future,
                max_history: state.max_history,
            }
        }
    }
}

/// A hook for managing state history for undo/redo functionality
pub fn use_history<T>(initial_state: T, max_history: usize) -> HistoryManager<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    // Create the initial history state
    let initial_history = HistoryState {
        current: initial_state,
        past: VecDeque::new(),
        future: VecDeque::new(),
        max_history,
    };

    // Use the reducer to manage state
    let (state, dispatch) = use_reducer(history_reducer, initial_history);

    // Create a function to get the current state
    let state_for_getter = state.clone();
    let get_state = std::sync::Arc::new(move || {
        // This will always get the latest state from the reducer
        state_for_getter.get()
    });

    // Create and return the HistoryManager
    HistoryManager {
        get_state,
        dispatch,
    }
}
