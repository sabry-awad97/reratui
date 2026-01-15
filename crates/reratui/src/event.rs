//! Event system for sharing terminal events between components.
//!
//! This module provides the event state management for the fiber-based component system.
//! It allows components to access terminal events (keyboard, mouse, resize) through
//! the `use_event` hook.
//!
//! # Architecture
//!
//! Events are stored in thread-local storage and tracked per-hook-index to ensure
//! each hook instance can only process an event once per render cycle.
//!
//! # Example
//!
//! ```rust,ignore
//! use reratui_fiber::event::{set_current_event, clear_current_event};
//! use crossterm::event::Event;
//! use std::sync::Arc;
//!
//! // Set an event (typically done by the runtime)
//! set_current_event(Some(Arc::new(Event::Key(...))));
//!
//! // Components can then use use_event() to access it
//!
//! // Clear the event at the end of the render cycle
//! clear_current_event();
//! ```

use crossterm::event::Event;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use tracing::debug;

/// Structure to track the current event.
///
/// Event consumption is now tracked at the fiber level rather than per hook index.
/// This simplifies the event system and aligns with the fiber-based architecture.
#[derive(Default)]
pub struct EventState {
    /// The current event being processed.
    pub(crate) event: Option<Arc<Event>>,
}

impl EventState {
    /// Creates a new empty EventState.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if an event is currently available.
    pub fn has_event(&self) -> bool {
        self.event.is_some()
    }
}

/// Global storage for the current event.
///
/// This is thread-local to ensure proper isolation in multi-threaded scenarios.
pub(crate) static CURRENT_EVENT: Lazy<RwLock<EventState>> = Lazy::new(Default::default);

/// Sets the current event in the global storage.
///
/// This function should be called by the runtime when an event is received.
/// Event consumption tracking is now handled at the fiber level.
///
/// # Arguments
///
/// * `event` - The event to set, or None to clear the current event.
///
/// # Example
///
/// ```rust,ignore
/// use reratui_fiber::event::set_current_event;
/// use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
/// use std::sync::Arc;
///
/// let key_event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
/// set_current_event(Some(Arc::new(key_event)));
/// ```
pub fn set_current_event(event: Option<Arc<Event>>) {
    let event_debug = event.clone();

    let mut current_event = CURRENT_EVENT.write().unwrap();
    current_event.event = event;

    debug!("Set current event: {:?}", event_debug);
}

/// Gets the current event for the current fiber.
///
/// This function checks if the current fiber has already consumed the event and returns
/// None if it has. Otherwise, it marks the event as consumed by this fiber and
/// returns the event.
///
/// # Returns
///
/// * `Some(Arc<Event>)` - The current event if available and not yet consumed by this fiber.
/// * `None` - If no event is available or the fiber has already consumed it.
pub fn get_current_event() -> Option<Arc<Event>> {
    use crate::fiber_tree::with_current_fiber;

    let event_state = CURRENT_EVENT.read().unwrap();

    // Get the current event, return None if no event is available
    let event = match event_state.event.as_ref() {
        Some(e) => e.clone(),
        None => {
            debug!("No event available");
            return None;
        }
    };

    // Release the read lock before accessing fiber
    drop(event_state);

    // Check if the current fiber has already consumed the event
    let already_consumed = match with_current_fiber(|fiber| fiber.event_consumed) {
        Some(consumed) => consumed,
        None => {
            debug!("No current fiber");
            return None;
        }
    };

    if already_consumed {
        debug!("Fiber already consumed the event");
        return None;
    }

    // Mark the event as consumed by this fiber
    with_current_fiber(|fiber| {
        fiber.event_consumed = true;
    });
    debug!("Fiber consuming event");

    Some(event)
}

/// Clears the current event from the global storage.
///
/// This function should be called at the end of each render cycle to ensure
/// events don't persist across renders.
pub fn clear_current_event() {
    set_current_event(None);
}

/// Resets event consumed flags for all fibers.
///
/// This function should be called when a new event is set to allow all fibers
/// to consume the new event.
pub fn reset_all_fiber_event_flags() {
    use crate::fiber_tree::with_fiber_tree_mut;

    with_fiber_tree_mut(|tree| {
        for fiber in tree.fibers.values_mut() {
            fiber.reset_event_consumed();
        }
    });
}

/// Returns the current event without marking it as processed.
///
/// This is useful for peeking at the event without consuming it.
///
/// # Returns
///
/// * `Some(Arc<Event>)` - The current event if available.
/// * `None` - If no event is available.
pub fn peek_current_event() -> Option<Arc<Event>> {
    let event_state = CURRENT_EVENT.read().unwrap();
    event_state.event.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fiber_tree::{FiberTree, clear_fiber_tree, set_fiber_tree, with_fiber_tree_mut};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use once_cell::sync::Lazy;
    use parking_lot::Mutex;

    /// Test mutex to ensure tests run sequentially since they share global state
    /// This is shared between unit tests and property tests
    pub(super) static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn create_test_key_event(c: char) -> Event {
        Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char(c),
            KeyModifiers::NONE,
            KeyEventKind::Press,
        ))
    }

    fn setup_test_fiber() {
        let mut tree = FiberTree::new();
        let fiber_id = tree.mount(None, None);
        tree.begin_render(fiber_id);
        set_fiber_tree(tree);
    }

    fn teardown_test_fiber() {
        with_fiber_tree_mut(|tree| {
            tree.end_render();
        });
        clear_fiber_tree();
    }

    #[test]
    fn test_set_and_get_event() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();
        setup_test_fiber();

        let event = create_test_key_event('a');
        set_current_event(Some(Arc::new(event.clone())));
        reset_all_fiber_event_flags();

        let retrieved = get_current_event();
        assert!(retrieved.is_some());

        if let Some(e) = retrieved {
            if let Event::Key(key) = &*e {
                assert_eq!(key.code, KeyCode::Char('a'));
            } else {
                panic!("Expected Key event");
            }
        }

        teardown_test_fiber();
    }

    #[test]
    fn test_event_consumed_once_per_fiber() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();
        setup_test_fiber();

        let event = create_test_key_event('b');
        set_current_event(Some(Arc::new(event)));
        reset_all_fiber_event_flags();

        // First call should return the event
        let first = get_current_event();
        assert!(first.is_some());

        // Second call with same fiber should return None
        let second = get_current_event();
        assert!(second.is_none());

        teardown_test_fiber();
    }

    #[test]
    fn test_different_fibers_can_consume_same_event() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();

        let mut tree = FiberTree::new();
        let fiber1 = tree.mount(None, None);
        let fiber2 = tree.mount(None, None);
        set_fiber_tree(tree);

        let event = create_test_key_event('c');
        set_current_event(Some(Arc::new(event)));
        reset_all_fiber_event_flags();

        // Fiber 1 consumes the event
        with_fiber_tree_mut(|tree| {
            tree.begin_render(fiber1);
        });
        let result1 = get_current_event();
        assert!(result1.is_some());
        with_fiber_tree_mut(|tree| {
            tree.end_render();
        });

        // Fiber 2 can also consume the same event
        with_fiber_tree_mut(|tree| {
            tree.begin_render(fiber2);
        });
        let result2 = get_current_event();
        assert!(result2.is_some());
        with_fiber_tree_mut(|tree| {
            tree.end_render();
        });

        // Fiber 1 cannot consume again
        with_fiber_tree_mut(|tree| {
            tree.begin_render(fiber1);
        });
        let result1_again = get_current_event();
        assert!(result1_again.is_none());
        with_fiber_tree_mut(|tree| {
            tree.end_render();
        });

        clear_fiber_tree();
    }

    #[test]
    fn test_clear_event() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();
        setup_test_fiber();

        let event = create_test_key_event('d');
        set_current_event(Some(Arc::new(event)));
        reset_all_fiber_event_flags();

        // Event should be available
        assert!(peek_current_event().is_some());

        // Clear the event
        clear_current_event();

        // Event should no longer be available
        assert!(peek_current_event().is_none());

        teardown_test_fiber();
    }

    #[test]
    fn test_new_event_resets_consumed_state() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();
        setup_test_fiber();

        let event1 = create_test_key_event('e');
        set_current_event(Some(Arc::new(event1)));
        reset_all_fiber_event_flags();

        // Fiber consumes the first event
        let _ = get_current_event();

        // Set a new event and reset flags
        let event2 = create_test_key_event('f');
        set_current_event(Some(Arc::new(event2)));
        reset_all_fiber_event_flags();

        // Fiber should be able to consume the new event
        let result = get_current_event();
        assert!(result.is_some());

        teardown_test_fiber();
    }

    #[test]
    fn test_peek_does_not_mark_consumed() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();
        setup_test_fiber();

        let event = create_test_key_event('g');
        set_current_event(Some(Arc::new(event)));
        reset_all_fiber_event_flags();

        // Peek at the event
        let peeked = peek_current_event();
        assert!(peeked.is_some());

        // Fiber should still be able to get the event
        let retrieved = get_current_event();
        assert!(retrieved.is_some());

        teardown_test_fiber();
    }

    #[test]
    fn test_event_state_helpers() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();

        let state = CURRENT_EVENT.read().unwrap();
        assert!(!state.has_event());
        drop(state);

        let event = create_test_key_event('h');
        set_current_event(Some(Arc::new(event)));

        let state = CURRENT_EVENT.read().unwrap();
        assert!(state.has_event());
    }
}
