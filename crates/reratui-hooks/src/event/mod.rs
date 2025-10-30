//! Event context for sharing events between components
//!
//! This module provides a context for sharing events between components,
//! allowing child components to access the current event without having to
//! pass it through props.

use crossterm::event::Event;

pub mod global_events;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use once_cell::sync::Lazy;
use tracing::debug;

use crate::hook_context::with_hook_context;

/// Structure to track an event and whether it has been processed
#[derive(Default)]
pub(crate) struct EventState {
    /// The current event
    pub(crate) event: Option<Arc<Event>>,
    /// Map of component IDs to whether they've processed the event
    /// This allows each component to independently process the event
    pub(crate) processed_by: HashMap<usize, bool>,
}

/// Global storage for the current event
pub(crate) static CURRENT_EVENT: Lazy<RwLock<EventState>> = Lazy::new(Default::default);

/// Sets the current event in the global storage
///
/// This function should be called by the App when an event is received.
///
/// # Arguments
///
/// * `event` - The event to set in the context
pub fn set_current_event(event: Option<Arc<Event>>) {
    // Clone the event for debugging
    let event_debug = event.clone();

    // Store the event in the global storage
    let mut current_event = CURRENT_EVENT.write().unwrap();
    current_event.event = event;
    current_event.processed_by.clear(); // Reset the processed map for the new event

    debug!("Set current event in context: {:?}", event_debug);
    debug!("Reset processed state for all components");

    // For debugging
    if event_debug.is_none() {
        debug!("Event is None");
    }
}

/// Gets the current event from the context
///
/// This function should be called by components to access the current event.
/// Each component can only access the event once per event cycle.
///
/// # Returns
///
/// * `Option<Arc<Event>>` - The current event, or None if no event is available or already processed
pub(crate) fn get_current_event() -> Option<Arc<Event>> {
    // Use hook context to get component's hook index
    let hook_index = with_hook_context(|ctx| ctx.next_hook_index());

    // Check the global storage
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

    // If already processed, return None
    if already_processed {
        debug!("Hook {} already processed the event", hook_index);
        return None;
    }

    drop(event_state); // Release the read lock before acquiring the write lock

    // Mark the event as processed by this hook
    mark_event_processed(hook_index);
    debug!("Hook {} processing event", hook_index);

    Some(event)
}

/// Marks the current event as processed by the specified component
///
/// # Arguments
///
/// * `component_id` - The ID of the component that processed the event
pub fn mark_event_processed(component_id: usize) {
    let mut event_state = CURRENT_EVENT.write().unwrap();
    event_state.processed_by.insert(component_id, true);
    debug!("Marked event as processed by component {}", component_id);
}

/// A React-style hook that returns the current terminal event being processed
///
/// This hook allows components to handle terminal events like keyboard, mouse, and resize events
/// in their render methods. Each hook instance can access an event exactly once per render cycle.
///
/// # Event Types
///
/// Handles all Crossterm event types:
/// - `Event::Key` - Keyboard events with modifiers (Ctrl, Alt, Shift)
/// - `Event::Mouse` - Mouse clicks, drags, scrolls, and movement
/// - `Event::Resize` - Terminal window resize events
/// - `Event::FocusGained`/`Event::FocusLost` - Terminal focus events
/// - `Event::Paste` - Paste events
///
/// # Usage Patterns
///
/// 1. Keyboard Event Handling:
/// ```rust,no_run
/// # use reratui_hooks::event::use_event;
/// # use crossterm::event::{Event, KeyCode, KeyModifiers};
/// // In a component context:
/// if let Some(Event::Key(key)) = use_event() {
///     match key.code {
///         KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
///             // Handle Ctrl+Q
///         }
///         KeyCode::Enter => {
///             // Handle Enter key
///         }
///         _ => {}
///     }
/// }
/// ```
///
/// 2. Mouse Event Handling:
/// ```rust,no_run
/// # use reratui_hooks::event::use_event;
/// # use crossterm::event::{Event, MouseEventKind, MouseButton};
/// // In a component context:
/// if let Some(Event::Mouse(mouse)) = use_event() {
///     match mouse.kind {
///         MouseEventKind::Down(MouseButton::Left) => {
///             // Handle left click at (mouse.column, mouse.row)
///         }
///         MouseEventKind::Drag(MouseButton::Left) => {
///             // Handle drag
///         }
///         _ => {}
///     }
/// }
/// ```
///
/// 3. Resize Event Handling:
/// ```rust,no_run
/// # use reratui_hooks::event::use_event;
/// # use crossterm::event::Event;
/// // In a component context:
/// if let Some(Event::Resize(width, height)) = use_event() {
///     // Handle terminal resize
/// }
/// ```
///
/// # Integration with Other Hooks
///
/// Works seamlessly with other hooks like `use_state`:
/// ```rust,no_run
/// # use reratui_hooks::event::use_event;
/// # use reratui_hooks::state::use_state;
/// # use crossterm::event::Event;
/// // In a component context:
/// let (key_count, set_key_count) = use_state(|| 0);
/// if let Some(Event::Key(_)) = use_event() {
///     set_key_count.update(|prev| prev + 1);
/// }
/// ```
///
/// # Note
///
/// - Events are consumed when accessed - each event can only be handled once per hook
/// - Multiple components can use this hook independently
/// - Events are cleared at the start of each render cycle
/// - Use with `use_state` for tracking event-related state
///
/// # Returns
///
/// * `Option<Event>` - The current event if available and not yet processed by this hook,
///   or None if no event is available or already processed
pub fn use_event() -> Option<Event> {
    use crate::panic_handler::catch_panic;

    match catch_panic(|| get_current_event().map(|e| (*e).clone())) {
        Ok(event) => event,
        Err(panic_payload) => {
            tracing::error!(
                target: "hooks::event",
                "Event processing panicked: {:?}",
                panic_payload.downcast_ref::<&str>().unwrap_or(&"<unknown panic>")
            );
            // Return None if event processing panics
            None
        }
    }
}
