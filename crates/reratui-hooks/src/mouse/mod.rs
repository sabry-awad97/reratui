//! Mouse event hook
//!
//! Provides a convenient hook for handling mouse events with stable callbacks.

use crate::{effect_event::use_effect_event, event::use_event, ref_hook::use_ref};
use crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

/// A hook that handles mouse events with a stable callback.
///
/// This hook uses `use_effect_event` internally to ensure the callback always
/// sees the latest captured values while maintaining a stable identity.
///
/// # Type Parameters
///
/// * `F` - A function that takes a `MouseEvent` and returns nothing
///
/// # Arguments
///
/// * `handler` - A callback function that will be invoked when a mouse event occurs
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::mouse::use_mouse;
/// use reratui_hooks::state::use_state;
/// use crossterm::event::{MouseEventKind, MouseButton};
///
/// // Track mouse clicks
/// let (click_count, set_click_count) = use_state(|| 0);
///
/// use_mouse(move |mouse_event| {
///     if matches!(mouse_event.kind, MouseEventKind::Down(MouseButton::Left)) {
///         println!("Mouse clicked at: ({}, {})", mouse_event.column, mouse_event.row);
///         set_click_count.update(|c| *c + 1);
///     }
/// });
/// ```
///
/// # Note
///
/// - The callback always sees the latest state values (via effect event pattern)
/// - Each mouse event is only processed once per component
/// - The callback has a stable identity across renders
/// - Only mouse events trigger the callback (keyboard, resize, etc. are ignored)
/// - Mouse capture must be enabled in the terminal (enabled by default in tui-pulse)
pub fn use_mouse<F>(handler: F)
where
    F: Fn(MouseEvent) + Clone + Send + Sync + 'static,
{
    // Create a stable callback using effect event pattern
    let stable_handler = use_effect_event(move |mouse_event: MouseEvent| {
        handler(mouse_event);
    });

    // Check for mouse events
    if let Some(Event::Mouse(mouse_event)) = use_event() {
        // Emit the event to the stable handler
        stable_handler.emit(mouse_event);
    }
}

/// A hook that handles mouse click events only (filters out movement and drag).
///
/// This is a convenience wrapper around `use_mouse` that only triggers the callback
/// when a mouse button is clicked (pressed down), ignoring movement, drag, and scroll events.
///
/// # Type Parameters
///
/// * `F` - A function that takes `(MouseButton, u16, u16)` (button, column, row) and returns nothing
///
/// # Arguments
///
/// * `handler` - A callback function that will be invoked when a mouse button is clicked
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::mouse::use_mouse_click;
/// use crossterm::event::MouseButton;
///
/// // Track left clicks only
/// use_mouse_click(move |button, x, y| {
///     if button == MouseButton::Left {
///         println!("Left click at ({}, {})", x, y);
///     }
/// });
/// ```
///
/// # Note
///
/// - Only triggers on `MouseEventKind::Down` events
/// - Filters out movement, drag, scroll, and button release events
/// - The callback always sees the latest state values (via effect event pattern)
/// - The callback has a stable identity across renders
pub fn use_mouse_click<F>(handler: F)
where
    F: Fn(MouseButton, u16, u16) + Clone + Send + Sync + 'static,
{
    use_mouse(move |mouse_event| {
        // Only handle click (down) events
        if let MouseEventKind::Down(button) = mouse_event.kind {
            handler(button, mouse_event.column, mouse_event.row);
        }
    });
}

/// Information about a drag operation
#[derive(Clone, Debug)]
pub struct DragInfo {
    /// The mouse button being used for dragging
    pub button: MouseButton,
    /// Starting position (column, row)
    pub start: (u16, u16),
    /// Current position (column, row)
    pub current: (u16, u16),
    /// Whether the drag just started
    pub is_start: bool,
    /// Whether the drag just ended
    pub is_end: bool,
}

/// A hook that detects mouse drag operations with start and end positions.
///
/// This hook tracks drag operations, providing information about the drag button,
/// start position, current position, and whether the drag is starting or ending.
///
/// # Type Parameters
///
/// * `F` - A function that takes `DragInfo` and returns nothing
///
/// # Arguments
///
/// * `handler` - A callback function that will be invoked during drag operations
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::mouse::use_mouse_drag;
/// use crossterm::event::MouseButton;
///
/// // Track drag operations
/// use_mouse_drag(move |drag_info| {
///     if drag_info.is_start {
///         println!("Drag started at {:?}", drag_info.start);
///     } else if drag_info.is_end {
///         println!("Drag ended at {:?}", drag_info.current);
///     } else {
///         println!("Dragging from {:?} to {:?}", drag_info.start, drag_info.current);
///     }
/// });
/// ```
///
/// # Note
///
/// - Tracks drag start (button down), drag movement, and drag end (button up)
/// - Uses `use_ref` internally to track drag state without re-renders
/// - The callback always sees the latest state values (via effect event pattern)
/// - The callback has a stable identity across renders
pub fn use_mouse_drag<F>(handler: F)
where
    F: Fn(DragInfo) + Clone + Send + Sync + 'static,
{
    // Track drag state: Option<(button, start_x, start_y)>
    let drag_state = use_ref(|| None::<(MouseButton, u16, u16)>);

    use_mouse(move |mouse_event| {
        match mouse_event.kind {
            MouseEventKind::Down(button) => {
                // Start drag
                drag_state.set(Some((button, mouse_event.column, mouse_event.row)));
                handler(DragInfo {
                    button,
                    start: (mouse_event.column, mouse_event.row),
                    current: (mouse_event.column, mouse_event.row),
                    is_start: true,
                    is_end: false,
                });
            }
            MouseEventKind::Drag(button) => {
                // Continue drag
                if let Some((drag_button, start_x, start_y)) = drag_state.get()
                    && button == drag_button
                {
                    handler(DragInfo {
                        button,
                        start: (start_x, start_y),
                        current: (mouse_event.column, mouse_event.row),
                        is_start: false,
                        is_end: false,
                    });
                }
            }
            MouseEventKind::Up(button) => {
                // End drag
                if let Some((drag_button, start_x, start_y)) = drag_state.get()
                    && button == drag_button
                {
                    handler(DragInfo {
                        button,
                        start: (start_x, start_y),
                        current: (mouse_event.column, mouse_event.row),
                        is_start: false,
                        is_end: true,
                    });
                    drag_state.set(None);
                }
            }
            _ => {}
        }
    });
}

/// A hook that detects double-click events with configurable timing.
///
/// This hook detects when a mouse button is clicked twice within a specified
/// time window (default 500ms).
///
/// # Type Parameters
///
/// * `F` - A function that takes `(MouseButton, u16, u16)` (button, column, row) and returns nothing
///
/// # Arguments
///
/// * `max_delay` - Maximum time between clicks to be considered a double-click
/// * `handler` - A callback function that will be invoked when a double-click is detected
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::mouse::use_double_click;
/// use std::time::Duration;
///
/// // Detect double-clicks with 500ms window
/// use_double_click(Duration::from_millis(500), move |button, x, y| {
///     println!("Double-click at ({}, {})", x, y);
/// });
/// ```
///
/// # Note
///
/// - Default timing window is 500ms (typical for most UIs)
/// - Only triggers on the second click of a double-click
/// - Uses `use_ref` internally to track click timing without re-renders
/// - The callback always sees the latest state values (via effect event pattern)
/// - The callback has a stable identity across renders
pub fn use_double_click<F>(max_delay: Duration, handler: F)
where
    F: Fn(MouseButton, u16, u16) + Clone + Send + Sync + 'static,
{
    // Track last click: Option<(button, x, y, time)>
    let last_click = use_ref(|| None::<(MouseButton, u16, u16, Instant)>);

    use_mouse(move |mouse_event| {
        if let MouseEventKind::Down(button) = mouse_event.kind {
            let now = Instant::now();
            let current_pos = (mouse_event.column, mouse_event.row);

            if let Some((last_button, last_x, last_y, last_time)) = last_click.get() {
                // Check if this is a double-click
                let time_diff = now.duration_since(last_time);
                let same_button = button == last_button;
                let same_position = current_pos == (last_x, last_y);

                if same_button && same_position && time_diff <= max_delay {
                    // Double-click detected!
                    handler(button, mouse_event.column, mouse_event.row);
                    // Reset to prevent triple-click from triggering another double-click
                    last_click.set(None);
                    return;
                }
            }

            // Store this click for potential double-click
            last_click.set(Some((button, mouse_event.column, mouse_event.row, now)));
        }
    });
}
