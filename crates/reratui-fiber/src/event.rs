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
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tracing::debug;

/// Structure to track an event and whether it has been processed by each hook.
///
/// This allows multiple components to independently check for and process events,
/// while ensuring each hook instance only sees an event once per render cycle.
#[derive(Default)]
pub struct EventState {
    /// The current event being processed.
    pub(crate) event: Option<Arc<Event>>,
    /// Map of hook indices to whether they've processed the event.
    /// This allows each hook to independently process the event.
    pub(crate) processed_by: HashMap<usize, bool>,
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

    /// Returns the number of hooks that have processed the current event.
    pub fn processed_count(&self) -> usize {
        self.processed_by.values().filter(|&&v| v).count()
    }
}

/// Global storage for the current event.
///
/// This is thread-local to ensure proper isolation in multi-threaded scenarios.
pub(crate) static CURRENT_EVENT: Lazy<RwLock<EventState>> = Lazy::new(Default::default);

/// Sets the current event in the global storage.
///
/// This function should be called by the runtime when an event is received.
/// It clears the processed state for all hooks, allowing them to process the new event.
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
    current_event.processed_by.clear();

    debug!("Set current event: {:?}", event_debug);
}

/// Gets the current event for a specific hook index.
///
/// This function checks if the hook has already processed the event and returns
/// None if it has. Otherwise, it marks the event as processed by this hook and
/// returns the event.
///
/// # Arguments
///
/// * `hook_index` - The index of the hook requesting the event.
///
/// # Returns
///
/// * `Some(Arc<Event>)` - The current event if available and not yet processed by this hook.
/// * `None` - If no event is available or the hook has already processed it.
pub fn get_current_event(hook_index: usize) -> Option<Arc<Event>> {
    let event_state = CURRENT_EVENT.read().unwrap();

    // Get the current event, return None if no event is available
    let event = match event_state.event.as_ref() {
        Some(e) => e.clone(),
        None => {
            debug!("No event available for hook {}", hook_index);
            return None;
        }
    };

    // Check if this hook has already processed the event
    let already_processed = event_state
        .processed_by
        .get(&hook_index)
        .copied()
        .unwrap_or(false);

    if already_processed {
        debug!("Hook {} already processed the event", hook_index);
        return None;
    }

    // Release the read lock before acquiring the write lock
    drop(event_state);

    // Mark the event as processed by this hook
    mark_event_processed(hook_index);
    debug!("Hook {} processing event", hook_index);

    Some(event)
}

/// Marks the current event as processed by the specified hook.
///
/// # Arguments
///
/// * `hook_index` - The index of the hook that processed the event.
pub fn mark_event_processed(hook_index: usize) {
    let mut event_state = CURRENT_EVENT.write().unwrap();
    event_state.processed_by.insert(hook_index, true);
    debug!("Marked event as processed by hook {}", hook_index);
}

/// Clears the current event from the global storage.
///
/// This function should be called at the end of each render cycle to ensure
/// events don't persist across renders.
pub fn clear_current_event() {
    set_current_event(None);
}

/// Checks if a specific hook has already processed the current event.
///
/// # Arguments
///
/// * `hook_index` - The index of the hook to check.
///
/// # Returns
///
/// * `true` if the hook has processed the event, `false` otherwise.
pub fn has_hook_processed(hook_index: usize) -> bool {
    let event_state = CURRENT_EVENT.read().unwrap();
    event_state
        .processed_by
        .get(&hook_index)
        .copied()
        .unwrap_or(false)
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

    #[test]
    fn test_set_and_get_event() {
        let _lock = TEST_MUTEX.lock();
        // Clear any existing state
        clear_current_event();

        let event = create_test_key_event('a');
        set_current_event(Some(Arc::new(event.clone())));

        let retrieved = get_current_event(0);
        assert!(retrieved.is_some());

        if let Some(e) = retrieved {
            if let Event::Key(key) = &*e {
                assert_eq!(key.code, KeyCode::Char('a'));
            } else {
                panic!("Expected Key event");
            }
        }
    }

    #[test]
    fn test_event_processed_once_per_hook() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();

        let event = create_test_key_event('b');
        set_current_event(Some(Arc::new(event)));

        // First call should return the event
        let first = get_current_event(0);
        assert!(first.is_some());

        // Second call with same hook index should return None
        let second = get_current_event(0);
        assert!(second.is_none());
    }

    #[test]
    fn test_different_hooks_can_process_same_event() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();

        let event = create_test_key_event('c');
        set_current_event(Some(Arc::new(event)));

        // Hook 0 processes the event
        let hook0 = get_current_event(0);
        assert!(hook0.is_some());

        // Hook 1 can also process the same event
        let hook1 = get_current_event(1);
        assert!(hook1.is_some());

        // Hook 0 cannot process again
        let hook0_again = get_current_event(0);
        assert!(hook0_again.is_none());
    }

    #[test]
    fn test_clear_event() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();

        let event = create_test_key_event('d');
        set_current_event(Some(Arc::new(event)));

        // Event should be available
        assert!(peek_current_event().is_some());

        // Clear the event
        clear_current_event();

        // Event should no longer be available
        assert!(peek_current_event().is_none());
    }

    #[test]
    fn test_new_event_resets_processed_state() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();

        let event1 = create_test_key_event('e');
        set_current_event(Some(Arc::new(event1)));

        // Hook 0 processes the first event
        let _ = get_current_event(0);
        assert!(has_hook_processed(0));

        // Set a new event
        let event2 = create_test_key_event('f');
        set_current_event(Some(Arc::new(event2)));

        // Hook 0 should be able to process the new event
        assert!(!has_hook_processed(0));
        let result = get_current_event(0);
        assert!(result.is_some());
    }

    #[test]
    fn test_peek_does_not_mark_processed() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();

        let event = create_test_key_event('g');
        set_current_event(Some(Arc::new(event)));

        // Peek at the event
        let peeked = peek_current_event();
        assert!(peeked.is_some());

        // Hook should still be able to get the event
        let retrieved = get_current_event(0);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_event_state_helpers() {
        let _lock = TEST_MUTEX.lock();
        clear_current_event();

        let state = CURRENT_EVENT.read().unwrap();
        assert!(!state.has_event());
        assert_eq!(state.processed_count(), 0);
        drop(state);

        let event = create_test_key_event('h');
        set_current_event(Some(Arc::new(event)));

        let state = CURRENT_EVENT.read().unwrap();
        assert!(state.has_event());
        drop(state);

        // Process with two hooks
        let _ = get_current_event(0);
        let _ = get_current_event(1);

        let state = CURRENT_EVENT.read().unwrap();
        assert_eq!(state.processed_count(), 2);
    }
}

#[cfg(test)]
mod property_tests {
    use super::tests::TEST_MUTEX;
    use super::*;
    use crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    use proptest::prelude::*;

    // **Property 1: Event Round-Trip**
    // **Validates: Requirements 1.3, 1.4, 1.6, 5.2, 5.5**
    //
    // For any terminal event that is set via `set_current_event`, when `get_current_event`
    // is called with a hook index that hasn't processed the event, the same event SHALL be returned.

    // Strategy to generate random key codes
    fn key_code_strategy() -> impl Strategy<Value = KeyCode> {
        prop_oneof![
            any::<char>().prop_map(KeyCode::Char),
            Just(KeyCode::Enter),
            Just(KeyCode::Backspace),
            Just(KeyCode::Tab),
            Just(KeyCode::Esc),
            Just(KeyCode::Up),
            Just(KeyCode::Down),
            Just(KeyCode::Left),
            Just(KeyCode::Right),
            Just(KeyCode::Home),
            Just(KeyCode::End),
            Just(KeyCode::PageUp),
            Just(KeyCode::PageDown),
            Just(KeyCode::Delete),
            Just(KeyCode::Insert),
            (1u8..=12).prop_map(KeyCode::F),
        ]
    }

    // Strategy to generate random key modifiers
    fn key_modifiers_strategy() -> impl Strategy<Value = KeyModifiers> {
        prop_oneof![
            Just(KeyModifiers::NONE),
            Just(KeyModifiers::SHIFT),
            Just(KeyModifiers::CONTROL),
            Just(KeyModifiers::ALT),
            Just(KeyModifiers::SHIFT | KeyModifiers::CONTROL),
            Just(KeyModifiers::SHIFT | KeyModifiers::ALT),
            Just(KeyModifiers::CONTROL | KeyModifiers::ALT),
        ]
    }

    // Strategy to generate random key events
    fn key_event_strategy() -> impl Strategy<Value = KeyEvent> {
        (key_code_strategy(), key_modifiers_strategy()).prop_map(|(code, modifiers)| {
            KeyEvent::new_with_kind(code, modifiers, KeyEventKind::Press)
        })
    }

    // Strategy to generate random mouse events
    fn mouse_event_strategy() -> impl Strategy<Value = MouseEvent> {
        (
            prop_oneof![
                Just(MouseEventKind::Down(MouseButton::Left)),
                Just(MouseEventKind::Down(MouseButton::Right)),
                Just(MouseEventKind::Down(MouseButton::Middle)),
                Just(MouseEventKind::Up(MouseButton::Left)),
                Just(MouseEventKind::Up(MouseButton::Right)),
                Just(MouseEventKind::Moved),
                Just(MouseEventKind::ScrollUp),
                Just(MouseEventKind::ScrollDown),
            ],
            0u16..1000,
            0u16..1000,
            key_modifiers_strategy(),
        )
            .prop_map(|(kind, column, row, modifiers)| MouseEvent {
                kind,
                column,
                row,
                modifiers,
            })
    }

    // Strategy to generate random events
    fn event_strategy() -> impl Strategy<Value = Event> {
        prop_oneof![
            key_event_strategy().prop_map(Event::Key),
            mouse_event_strategy().prop_map(Event::Mouse),
            (1u16..1000, 1u16..1000).prop_map(|(w, h)| Event::Resize(w, h)),
            Just(Event::FocusGained),
            Just(Event::FocusLost),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 1: Event round-trip - set event, get same event back
        #[test]
        fn prop_event_round_trip(event in event_strategy(), hook_index in 0usize..1000) {
            let _lock = TEST_MUTEX.lock();
            // Clear any existing state
            clear_current_event();

            // Set the event
            set_current_event(Some(Arc::new(event.clone())));

            // Get the event back
            let retrieved = get_current_event(hook_index);

            // Property: The retrieved event should match the original
            prop_assert!(retrieved.is_some(), "Event should be retrievable");
            let retrieved_event = retrieved.unwrap();
            prop_assert_eq!(retrieved_event.as_ref(), &event, "Retrieved event should match original");
        }

        /// Property: No event returns None
        #[test]
        fn prop_no_event_returns_none(hook_index in 0usize..1000) {
            let _lock = TEST_MUTEX.lock();
            clear_current_event();

            let retrieved = get_current_event(hook_index);

            prop_assert!(retrieved.is_none(), "No event should return None");
        }

        /// Property: Setting None clears the event
        #[test]
        fn prop_set_none_clears_event(event in event_strategy(), hook_index in 0usize..1000) {
            let _lock = TEST_MUTEX.lock();
            clear_current_event();

            // Set an event
            set_current_event(Some(Arc::new(event)));

            // Clear it
            set_current_event(None);

            // Should return None
            let retrieved = get_current_event(hook_index);
            prop_assert!(retrieved.is_none(), "Cleared event should return None");
        }

        /// Property: Peek returns event without consuming it
        #[test]
        fn prop_peek_does_not_consume(event in event_strategy(), hook_index in 0usize..1000) {
            let _lock = TEST_MUTEX.lock();
            clear_current_event();

            set_current_event(Some(Arc::new(event.clone())));

            // Peek at the event
            let peeked = peek_current_event();
            prop_assert!(peeked.is_some(), "Peek should return event");
            let peeked_event = peeked.unwrap();
            prop_assert_eq!(peeked_event.as_ref(), &event, "Peeked event should match");

            // Should still be able to get the event
            let retrieved = get_current_event(hook_index);
            prop_assert!(retrieved.is_some(), "Event should still be available after peek");
            let retrieved_event = retrieved.unwrap();
            prop_assert_eq!(retrieved_event.as_ref(), &event, "Retrieved event should match");
        }

        /// Property: New event resets processed state for all hooks
        #[test]
        fn prop_new_event_resets_processed(
            event1 in event_strategy(),
            event2 in event_strategy(),
            hook_index in 0usize..1000
        ) {
            let _lock = TEST_MUTEX.lock();
            clear_current_event();

            // Set first event and process it
            set_current_event(Some(Arc::new(event1)));
            let _ = get_current_event(hook_index);

            // Hook should have processed the event
            prop_assert!(has_hook_processed(hook_index), "Hook should have processed first event");

            // Set new event
            set_current_event(Some(Arc::new(event2.clone())));

            // Hook should be able to process the new event
            prop_assert!(!has_hook_processed(hook_index), "Hook should not have processed new event yet");

            let retrieved = get_current_event(hook_index);
            prop_assert!(retrieved.is_some(), "Hook should be able to get new event");
            let retrieved_event = retrieved.unwrap();
            prop_assert_eq!(retrieved_event.as_ref(), &event2, "Retrieved event should be the new event");
        }

        // ============================================================
        // Property 2: Event Processing Isolation
        // ============================================================

        /// **Property 2: Event Processing Isolation**
        /// **Validates: Requirements 5.3, 5.4**
        ///
        /// For any hook index, once `get_current_event` has been called and returned an event,
        /// subsequent calls with the same hook index SHALL return `None` until a new event is set.

        /// Property: Same hook cannot process event twice
        #[test]
        fn prop_same_hook_cannot_process_twice(
            event in event_strategy(),
            hook_index in 0usize..1000
        ) {
            let _lock = TEST_MUTEX.lock();
            clear_current_event();

            set_current_event(Some(Arc::new(event)));

            // First call should return the event
            let first = get_current_event(hook_index);
            prop_assert!(first.is_some(), "First call should return event");

            // Second call with same hook index should return None
            let second = get_current_event(hook_index);
            prop_assert!(second.is_none(), "Second call with same hook should return None");

            // Third call should also return None
            let third = get_current_event(hook_index);
            prop_assert!(third.is_none(), "Third call with same hook should return None");
        }

        /// Property: Different hooks can process the same event independently
        #[test]
        fn prop_different_hooks_independent(
            event in event_strategy(),
            hook1 in 0usize..500,
            hook2 in 500usize..1000
        ) {
            let _lock = TEST_MUTEX.lock();
            clear_current_event();

            set_current_event(Some(Arc::new(event.clone())));

            // Hook 1 processes the event
            let result1 = get_current_event(hook1);
            prop_assert!(result1.is_some(), "Hook 1 should get event");
            let event1 = result1.unwrap();
            prop_assert_eq!(event1.as_ref(), &event, "Hook 1 should get correct event");

            // Hook 2 can also process the same event
            let result2 = get_current_event(hook2);
            prop_assert!(result2.is_some(), "Hook 2 should also get event");
            let event2 = result2.unwrap();
            prop_assert_eq!(event2.as_ref(), &event, "Hook 2 should get correct event");

            // Hook 1 cannot process again
            let result1_again = get_current_event(hook1);
            prop_assert!(result1_again.is_none(), "Hook 1 should not get event again");

            // Hook 2 cannot process again
            let result2_again = get_current_event(hook2);
            prop_assert!(result2_again.is_none(), "Hook 2 should not get event again");
        }

        /// Property: Processing state is tracked per hook
        #[test]
        fn prop_processing_state_per_hook(
            event in event_strategy(),
            hooks in prop::collection::vec(0usize..1000, 2..10)
        ) {
            let _lock = TEST_MUTEX.lock();
            clear_current_event();

            set_current_event(Some(Arc::new(event)));

            // Each hook should be able to process the event exactly once
            for &hook_index in &hooks {
                // First call should succeed (if not already processed by same index)
                let _first = get_current_event(hook_index);
                // Note: might be None if hooks contains duplicates

                // Second call should always fail
                let second = get_current_event(hook_index);
                prop_assert!(second.is_none(), "Hook {} should not process twice", hook_index);
            }
        }

        /// Property: has_hook_processed correctly tracks state
        #[test]
        fn prop_has_hook_processed_tracks_state(
            event in event_strategy(),
            hook_index in 0usize..1000
        ) {
            let _lock = TEST_MUTEX.lock();
            clear_current_event();

            // Before setting event, hook should not be marked as processed
            prop_assert!(!has_hook_processed(hook_index), "Hook should not be processed before event");

            set_current_event(Some(Arc::new(event)));

            // After setting event but before processing, hook should not be marked
            prop_assert!(!has_hook_processed(hook_index), "Hook should not be processed before get_current_event");

            // Process the event
            let _ = get_current_event(hook_index);

            // After processing, hook should be marked
            prop_assert!(has_hook_processed(hook_index), "Hook should be marked as processed after get_current_event");
        }

        /// Property: mark_event_processed directly marks hook
        #[test]
        fn prop_mark_event_processed_works(
            event in event_strategy(),
            hook_index in 0usize..1000
        ) {
            let _lock = TEST_MUTEX.lock();
            clear_current_event();

            set_current_event(Some(Arc::new(event)));

            // Manually mark as processed
            mark_event_processed(hook_index);

            // Hook should be marked
            prop_assert!(has_hook_processed(hook_index), "Hook should be marked after mark_event_processed");

            // get_current_event should return None
            let result = get_current_event(hook_index);
            prop_assert!(result.is_none(), "get_current_event should return None after mark_event_processed");
        }
    }
}
