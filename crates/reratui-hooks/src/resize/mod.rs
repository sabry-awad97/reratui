//! Resize event hook for detecting terminal size changes
//!
//! This module provides the `use_on_resize` hook for responding to terminal resize events.

use crate::{
    callback::Callback, effect_event::use_effect_event, event::use_event, state::use_state,
};
use crossterm::event::Event;

#[cfg(test)]
mod tests;

/// A hook that triggers a callback when the terminal is resized.
///
/// This hook monitors resize events and calls the provided callback with the new
/// terminal dimensions (width, height) whenever a resize occurs.
///
/// # Type Parameters
///
/// * `F` - The callback function type that takes `(u16, u16)` as parameters
///
/// # Arguments
///
/// * `callback` - A callback function that will be invoked with `(width, height)` when resize occurs
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,no_run
/// use reratui_hooks::resize::use_on_resize;
///
/// // In a component context:
/// use_on_resize(|(width, height)| {
///     println!("Terminal resized to: {}x{}", width, height);
/// });
/// ```
///
/// ## With State Updates
///
/// ```rust,no_run
/// use reratui_hooks::{resize::use_on_resize, state::use_state};
///
/// // Track terminal size in state
/// let (size, set_size) = use_state(|| (80u16, 24u16));
///
/// use_on_resize({
///     let set_size = set_size.clone();
///     move |(width, height)| {
///         set_size.set((width, height));
///     }
/// });
/// ```
///
/// ## With Closure
///
/// ```rust,no_run
/// use reratui_hooks::{
///     resize::use_on_resize,
///     state::use_state,
/// };
///
/// let (size, set_size) = use_state(|| (80u16, 24u16));
///
/// use_on_resize({
///     let set_size = set_size.clone();
///     move |(width, height): (u16, u16)| {
///         set_size.set((width, height));
///         println!("Resized to: {}x{}", width, height);
///     }
/// });
/// ```
///
/// # Implementation Details
///
/// - Monitors `Event::Resize` events from the event system
/// - Only triggers callback when actual resize events occur
/// - Callback receives `(width, height)` as a tuple
/// - Works seamlessly with other hooks like `use_state` and `use_callback`
/// - No performance overhead when no resize occurs
///
/// # Notes
///
/// - The callback is called immediately when a resize event is detected
/// - Multiple components can use this hook independently
/// - Each component will receive resize events separately
/// - Consider using `use_callback` to memoize the callback for better performance
///
/// # Note
///
/// - The callback always sees the latest state values (via effect event pattern)
/// - Each resize event is only processed once per component
/// - The callback has a stable identity across renders
/// - Only resize events trigger the callback (keyboard, mouse, etc. are ignored)
///
/// # Performance
///
/// This hook is lightweight and only processes events when they occur. It does not
/// poll or create additional overhead when the terminal is not being resized.
pub fn use_on_resize<F>(callback: F)
where
    F: Fn((u16, u16)) + Clone + Send + Sync + 'static,
{
    // Create a stable callback using effect event pattern
    let stable_handler = use_effect_event(move |dimensions: (u16, u16)| {
        callback(dimensions);
    });

    // Check for resize events
    if let Some(Event::Resize(width, height)) = use_event() {
        // Emit the event to the stable handler
        stable_handler.emit((width, height));
    }
}

/// A hook that triggers a memoized callback when the terminal is resized.
///
/// This is a variant of `use_on_resize` that accepts a `MemoizedCallback` instead
/// of a raw closure. This is useful when you want to use callbacks created with
/// `use_callback` or `use_effect_event`.
///
/// # Type Parameters
///
/// * `IN` - The input type for the callback (must be `(u16, u16)`)
/// * `OUT` - The output type of the callback
///
/// # Arguments
///
/// * `callback` - A memoized callback that will be invoked with `(width, height)`
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::{
///     resize::use_on_resize_callback,
///     callback::use_callback,
///     state::use_state,
/// };
///
/// let (size, set_size) = use_state(|| (80u16, 24u16));
///
/// let handle_resize = use_callback(
///     {
///         let set_size = set_size.clone();
///         move |(width, height): (u16, u16)| {
///             set_size.set((width, height));
///         }
///     },
///     set_size.version(),
/// );
///
/// use_on_resize_callback(handle_resize);
/// ```
pub fn use_on_resize_callback<OUT>(callback: Callback<(u16, u16), OUT>)
where
    OUT: 'static,
{
    // Check for resize events
    if let Some(Event::Resize(width, height)) = use_event() {
        // Call the memoized callback with the new dimensions
        callback.emit((width, height));
    }
}

/// A hook that returns the current terminal dimensions as a tuple.
///
/// This is a convenience hook that automatically tracks terminal size and returns
/// the current dimensions directly as a tuple.
///
/// # Returns
///
/// A tuple `(u16, u16)` containing the current terminal dimensions (width, height)
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::resize::use_terminal_dimensions;
///
/// // In a component context:
/// let (width, height) = use_terminal_dimensions();
/// println!("Terminal: {}x{}", width, height);
/// ```
///
/// ## Responsive Layout Example
///
/// ```rust,no_run
/// use reratui_hooks::resize::use_terminal_dimensions;
///
/// let (width, height) = use_terminal_dimensions();
///
/// // Adjust UI based on terminal size
/// if width < 80 {
///     // Render compact layout
/// } else {
///     // Render full layout
/// }
/// ```
///
/// # Notes
///
/// - Returns (0, 0) until the first resize event occurs
/// - Automatically updates when the terminal is resized
/// - Re-renders the component when dimensions change
/// - Most convenient API for read-only dimension access
pub fn use_terminal_dimensions() -> (u16, u16) {
    let (size, set_size) = use_state(|| (0u16, 0u16));

    use_on_resize({
        let set_size = set_size.clone();
        move |(width, height)| {
            set_size.set((width, height));
        }
    });

    size.get()
}

/// A hook that evaluates a media query predicate against terminal dimensions.
///
/// This hook allows you to create responsive layouts by checking terminal size
/// against custom conditions. The predicate is re-evaluated whenever the terminal
/// is resized.
///
/// # Type Parameters
///
/// * `F` - A function that takes `(u16, u16)` and returns `bool`
///
/// # Arguments
///
/// * `predicate` - A function that receives `(width, height)` and returns whether the condition matches
///
/// # Returns
///
/// `bool` - `true` if the predicate matches the current terminal dimensions, `false` otherwise
///
/// # Examples
///
/// ## Basic Width Query
///
/// ```rust,no_run
/// use reratui_hooks::resize::use_media_query;
///
/// // Check if terminal is narrow
/// let is_narrow = use_media_query(|(width, _)| width < 80);
///
/// if is_narrow {
///     // Render compact layout
/// } else {
///     // Render full layout
/// }
/// ```
///
/// ## Height Query
///
/// ```rust,no_run
/// use reratui_hooks::resize::use_media_query;
///
/// let is_short = use_media_query(|(_, height)| height < 24);
/// ```
///
/// ## Combined Queries
///
/// ```rust,no_run
/// use reratui_hooks::resize::use_media_query;
///
/// let is_small = use_media_query(|(width, height)| {
///     width < 80 && height < 24
/// });
///
/// let is_mobile = use_media_query(|(width, _)| width < 60);
/// let is_tablet = use_media_query(|(width, _)| width >= 60 && width < 120);
/// let is_desktop = use_media_query(|(width, _)| width >= 120);
/// ```
///
/// ## Aspect Ratio Query
///
/// ```rust,no_run
/// use reratui_hooks::resize::use_media_query;
///
/// let is_wide = use_media_query(|(width, height)| {
///     let ratio = width as f32 / height as f32;
///     ratio > 2.5
/// });
/// ```
///
/// ## Responsive Breakpoints
///
/// ```rust,no_run
/// use reratui_hooks::resize::use_media_query;
///
/// // Define breakpoints
/// let is_xs = use_media_query(|(w, _)| w < 40);
/// let is_sm = use_media_query(|(w, _)| w >= 40 && w < 80);
/// let is_md = use_media_query(|(w, _)| w >= 80 && w < 120);
/// let is_lg = use_media_query(|(w, _)| w >= 120 && w < 160);
/// let is_xl = use_media_query(|(w, _)| w >= 160);
/// ```
///
/// # Implementation Details
///
/// - Returns `false` until the first resize event occurs (dimensions are 0x0)
/// - Automatically re-evaluates when terminal is resized
/// - Triggers component re-render when the predicate result changes
/// - Predicate is only called when dimensions change
///
/// # Performance
///
/// The predicate function is called:
/// - Once on initial render
/// - Once per resize event
///
/// For expensive predicates, consider memoizing the result separately.
///
/// # Notes
///
/// - The predicate receives `(width, height)` as a tuple
/// - Multiple `use_media_query` calls can be used in the same component
/// - Each query independently tracks its own match state
/// - Useful for creating responsive TUI layouts
pub fn use_media_query<F>(predicate: F) -> bool
where
    F: Fn((u16, u16)) -> bool + Clone + Send + Sync + 'static,
{
    let (matches, set_matches) = use_state(|| false);

    use_on_resize({
        let set_matches = set_matches.clone();
        move |(width, height)| {
            let result = predicate((width, height));
            set_matches.set(result);
        }
    });

    matches.get()
}
