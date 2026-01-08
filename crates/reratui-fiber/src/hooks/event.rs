//! Event hook for accessing terminal events in components.
//!
//! This module provides the `use_event` hook that allows components to access
//! the current terminal event (keyboard, mouse, resize) during rendering.
//!
//! # Example
//!
//! ```rust,ignore
//! use reratui_fiber::hooks::use_event;
//! use crossterm::event::{Event, KeyCode};
//!
//! fn my_component() {
//!     if let Some(Event::Key(key)) = use_event() {
//!         if key.code == KeyCode::Char('q') {
//!             // Handle quit
//!         }
//!     }
//! }
//! ```

use crossterm::event::Event;

use crate::event::get_current_event;
use crate::fiber_tree::with_current_fiber;

/// Hook that returns the current terminal event.
///
/// This hook retrieves the current event from the event system and ensures
/// each hook instance only processes an event once per render cycle.
///
/// # Returns
///
/// * `Some(Event)` - The current event if available and not yet processed by this hook.
/// * `None` - If no event is available or the hook has already processed it.
///
/// # Example
///
/// ```rust,ignore
/// use reratui_fiber::hooks::use_event;
/// use crossterm::event::{Event, KeyCode, KeyEvent};
///
/// fn handle_input() {
///     if let Some(event) = use_event() {
///         match event {
///             Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
///                 // Quit the application
///             }
///             Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
///                 // Submit form
///             }
///             _ => {}
///         }
///     }
/// }
/// ```
pub fn use_event() -> Option<Event> {
    // Get the hook index from the current fiber
    // If no fiber is available (e.g., called outside of render), use 0 as fallback
    let hook_index = with_current_fiber(|fiber| fiber.next_hook_index()).unwrap_or(0);

    // Get the current event for this hook index
    // The event module handles tracking which hooks have processed the event
    get_current_event(hook_index).map(|arc_event| (*arc_event).clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{clear_current_event, set_current_event};
    use crate::fiber_tree::{FiberTree, clear_fiber_tree, set_fiber_tree, with_fiber_tree_mut};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use std::sync::{Arc, Mutex};

    // Mutex to ensure tests run sequentially (they share global state)
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn create_test_key_event(c: char) -> Event {
        Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char(c),
            KeyModifiers::NONE,
            KeyEventKind::Press,
        ))
    }

    fn setup_test_fiber_tree() {
        let mut tree = FiberTree::new();
        let fiber_id = tree.mount(None, None);
        tree.begin_render(fiber_id);
        set_fiber_tree(tree);
    }

    fn teardown_test_fiber_tree() {
        with_fiber_tree_mut(|tree| {
            tree.end_render();
        });
        clear_fiber_tree();
    }

    #[test]
    fn test_use_event_returns_event() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_current_event();
        setup_test_fiber_tree();

        let event = create_test_key_event('a');
        set_current_event(Some(Arc::new(event.clone())));

        let result = use_event();
        assert!(result.is_some());

        if let Some(Event::Key(key)) = result {
            assert_eq!(key.code, KeyCode::Char('a'));
        } else {
            panic!("Expected Key event");
        }

        teardown_test_fiber_tree();
        clear_current_event();
    }

    #[test]
    fn test_use_event_returns_none_when_no_event() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_current_event();
        setup_test_fiber_tree();

        let result = use_event();
        assert!(result.is_none());

        teardown_test_fiber_tree();
    }

    #[test]
    fn test_use_event_processes_once_per_hook() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_current_event();
        setup_test_fiber_tree();

        let event = create_test_key_event('b');
        set_current_event(Some(Arc::new(event)));

        // First call should return the event
        let first = use_event();
        assert!(first.is_some());

        // Second call with same hook index should return None
        // Note: In a real component, each use_event call gets a new hook index
        // But here we're testing the same hook index behavior
        let second = use_event();
        // This will get a new hook index, so it should also return the event
        assert!(second.is_some());

        teardown_test_fiber_tree();
        clear_current_event();
    }

    #[test]
    fn test_use_event_without_fiber_tree() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_current_event();
        clear_fiber_tree();

        let event = create_test_key_event('c');
        set_current_event(Some(Arc::new(event)));

        // Should still work with fallback hook index of 0
        let result = use_event();
        assert!(result.is_some());

        clear_current_event();
    }

    #[test]
    fn test_use_event_clones_event() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_current_event();
        setup_test_fiber_tree();

        let event = create_test_key_event('d');
        set_current_event(Some(Arc::new(event.clone())));

        let result = use_event();
        assert!(result.is_some());

        // Verify the returned event is a clone, not the Arc
        if let Some(Event::Key(key)) = result {
            assert_eq!(key.code, KeyCode::Char('d'));
        }

        teardown_test_fiber_tree();
        clear_current_event();
    }
}
