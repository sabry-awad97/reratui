//! Mouse event hook
//!
//! Provides a convenient hook for handling mouse events with stable callbacks.

use crate::{
    effect_event::use_effect_event, event::use_event, ref_hook::use_ref, state::use_state,
};
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DragInfo {
    /// The mouse button being used for dragging
    pub button: Option<MouseButton>,
    /// Starting position (column, row)
    pub start: (u16, u16),
    /// Current position (column, row)
    pub current: (u16, u16),
    /// Whether the drag is currently active
    pub is_dragging: bool,
    /// Whether the drag just started
    pub is_start: bool,
    /// Whether the drag just ended
    pub is_end: bool,
}

/// Hook for tracking mouse drag operations.
///
/// Returns a tuple containing the current drag state and a reset function.
/// The reset function can be used to clear the drag state and reset tracking.
///
/// This hook automatically updates the drag state based on mouse events from the current event context.
///
/// # Returns
///
/// A tuple `(DragInfo, impl Fn())` where:
/// - First element is the current drag information
/// - Second element is a reset function to clear the drag state
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::mouse::use_mouse_drag;
///
/// let (drag_info, reset_drag) = use_mouse_drag();
///
/// if drag_info.is_start {
///     println!("Drag started at {:?}", drag_info.start);
/// } else if drag_info.is_dragging {
///     println!("Dragging from {:?} to {:?}", drag_info.start, drag_info.current);
/// } else if drag_info.is_end {
///     println!("Drag ended at {:?}", drag_info.current);
/// }
///
/// // Reset drag state if needed
/// if some_condition {
///     reset_drag();
/// }
/// ```
///
/// # Note
///
/// - Tracks drag start (button down), drag movement, and drag end (button up)
/// - The drag state persists across renders until the drag ends or is reset
/// - `is_dragging` is `true` during the entire drag operation
/// - `is_start` is only `true` on the first frame of the drag
/// - `is_end` is only `true` on the last frame of the drag
pub fn use_mouse_drag() -> (DragInfo, impl Fn()) {
    let (drag_info, set_drag_info) = use_state(DragInfo::default);
    let drag_state = use_ref(|| None::<(MouseButton, u16, u16)>);

    let set_info_clone = set_drag_info.clone();
    let state_clone = drag_state.clone();

    use_mouse(move |mouse_event| {
        match mouse_event.kind {
            MouseEventKind::Down(button) => {
                // Start drag
                state_clone.set(Some((button, mouse_event.column, mouse_event.row)));
                set_info_clone.set(DragInfo {
                    button: Some(button),
                    start: (mouse_event.column, mouse_event.row),
                    current: (mouse_event.column, mouse_event.row),
                    is_dragging: true,
                    is_start: true,
                    is_end: false,
                });
            }
            MouseEventKind::Drag(button) => {
                // Continue drag
                if let Some((drag_button, start_x, start_y)) = state_clone.get()
                    && button == drag_button
                {
                    set_info_clone.set(DragInfo {
                        button: Some(button),
                        start: (start_x, start_y),
                        current: (mouse_event.column, mouse_event.row),
                        is_dragging: true,
                        is_start: false,
                        is_end: false,
                    });
                }
            }
            MouseEventKind::Up(button) => {
                // End drag
                if let Some((drag_button, start_x, start_y)) = state_clone.get()
                    && button == drag_button
                {
                    set_info_clone.set(DragInfo {
                        button: Some(button),
                        start: (start_x, start_y),
                        current: (mouse_event.column, mouse_event.row),
                        is_dragging: false,
                        is_start: false,
                        is_end: true,
                    });
                    state_clone.set(None);
                }
            }
            _ => {}
        }
    });

    let reset = {
        let set_info = set_drag_info.clone();
        let state = drag_state.clone();
        move || {
            set_info.set(DragInfo::default());
            state.set(None);
        }
    };

    (drag_info.get(), reset)
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

/// A hook that tracks the current mouse position.
///
/// Returns a tuple `(x, y)` representing the current mouse coordinates.
/// The position is updated whenever any mouse event occurs (move, click, scroll, etc.).
///
/// # Returns
///
/// A tuple `(u16, u16)` where:
/// - First element is the column (x-coordinate)
/// - Second element is the row (y-coordinate)
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::mouse::use_mouse_position;
///
/// let (x, y) = use_mouse_position();
/// println!("Mouse is at position: ({}, {})", x, y);
/// ```
///
/// # Note
///
/// - The position starts at (0, 0) until the first mouse event
/// - Mouse capture must be enabled in the terminal
/// - The hook updates on any mouse event, including movement, clicks, and scrolling
pub fn use_mouse_position() -> (u16, u16) {
    let (position, set_position) = use_state(|| (0u16, 0u16));

    use_mouse({
        let position = position.clone();
        move |mouse_event| {
            let new_pos = (mouse_event.column, mouse_event.row);
            if new_pos != position.get() {
                set_position.set(new_pos);
            }
        }
    });

    position.get()
}

/// A hook that detects if the mouse is hovering over a specific rectangular area.
///
/// Returns `true` if the mouse cursor is currently within the specified area bounds,
/// `false` otherwise. The hover state is updated on any mouse event.
///
/// # Arguments
///
/// * `area` - A `Rect` defining the rectangular area to monitor for hover events.
///   The area is defined by its `x`, `y`, `width`, and `height` properties.
///
/// # Returns
///
/// A boolean indicating whether the mouse is currently hovering over the area.
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::mouse::use_mouse_hover;
/// use ratatui::layout::Rect;
///
/// let button_area = Rect::new(10, 5, 20, 3);
/// let is_hovering = use_mouse_hover(button_area);
///
/// if is_hovering {
///     println!("Mouse is hovering over the button!");
/// }
/// ```
///
/// # Note
///
/// - The hover detection is inclusive of the area boundaries
/// - Mouse position (x, y) is considered inside if:
///   - `x >= area.x && x < area.x + area.width`
///   - `y >= area.y && y < area.y + area.height`
/// - The hook updates on any mouse event (movement, clicks, scrolling)
/// - Mouse capture must be enabled in the terminal
pub fn use_mouse_hover(area: ratatui::layout::Rect) -> bool {
    let (is_hovering, set_hovering) = use_state(|| false);

    use_mouse({
        let is_hovering = is_hovering.clone();

        move |mouse_event| {
            let is_inside = mouse_event.column >= area.x
                && mouse_event.column < area.x + area.width
                && mouse_event.row >= area.y
                && mouse_event.row < area.y + area.height;

            if is_inside != is_hovering.get() {
                set_hovering.set(is_inside);
            }
        }
    });

    is_hovering.get()
}
