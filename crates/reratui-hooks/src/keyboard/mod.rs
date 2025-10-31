//! Keyboard event hook
//!
//! Provides a convenient hook for handling keyboard events with stable callbacks.

use crate::{effect_event::use_effect_event, event::use_event};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[cfg(test)]
mod tests;

/// A hook that handles keyboard events with a stable callback.
///
/// This hook uses `use_effect_event` internally to ensure the callback always
/// sees the latest captured values while maintaining a stable identity.
///
/// # Type Parameters
///
/// * `F` - A function that takes a `KeyEvent` and returns nothing
///
/// # Arguments
///
/// * `handler` - A callback function that will be invoked when a key event occurs
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::keyboard::use_keyboard;
/// use reratui_hooks::state::use_state;
///
/// // Track key press count
/// let (count, set_count) = use_state(|| 0);
///
/// use_keyboard(move |key_event| {
///     println!("Key pressed: {:?}", key_event);
///     set_count.update(|c| *c + 1);
/// });
/// ```
///
/// # Note
///
/// - The callback always sees the latest state values (via effect event pattern)
/// - Each key event is only processed once per component
/// - The callback has a stable identity across renders
/// - Only keyboard events trigger the callback (mouse, resize, etc. are ignored)
pub fn use_keyboard<F>(handler: F)
where
    F: Fn(KeyEvent) + Clone + Send + Sync + 'static,
{
    // Create a stable callback using effect event pattern
    let stable_handler = use_effect_event(move |key_event: KeyEvent| {
        handler(key_event);
    });

    // Check for keyboard events
    if let Some(Event::Key(key_event)) = use_event() {
        // Emit the event to the stable handler
        stable_handler.emit(key_event);
    }
}

/// A hook that handles keyboard press events only (filters out release events).
///
/// This is a convenience wrapper around `use_keyboard` that only triggers the callback
/// when a key is pressed down, ignoring key release and repeat events.
///
/// # Type Parameters
///
/// * `F` - A function that takes a `KeyEvent` and returns nothing
///
/// # Arguments
///
/// * `handler` - A callback function that will be invoked when a key is pressed
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::keyboard::use_keyboard_press;
/// use reratui_hooks::state::use_state;
///
/// // Only track actual key presses, not releases
/// let (count, set_count) = use_state(|| 0);
///
/// use_keyboard_press(move |key_event| {
///     println!("Key pressed: {:?}", key_event.code);
///     set_count.update(|c| *c + 1);
/// });
/// ```
///
/// # Note
///
/// - Only triggers on `KeyEventKind::Press` events
/// - Filters out `KeyEventKind::Release` and `KeyEventKind::Repeat`
/// - The callback always sees the latest state values (via effect event pattern)
/// - The callback has a stable identity across renders
pub fn use_keyboard_press<F>(handler: F)
where
    F: Fn(KeyEvent) + Clone + Send + Sync + 'static,
{
    use_keyboard(move |key_event| {
        // Only handle press events, ignore release and repeat
        if key_event.is_press() {
            handler(key_event);
        }
    });
}

/// A hook that handles keyboard shortcuts with specific key and modifier combinations.
///
/// This is a high-level convenience hook for detecting keyboard shortcuts like Ctrl+S,
/// Alt+F4, etc. It only triggers on key press events.
///
/// # Type Parameters
///
/// * `F` - A function that takes no arguments and returns nothing
///
/// # Arguments
///
/// * `key_code` - The key code to match (e.g., `KeyCode::Char('s')`)
/// * `modifiers` - The required modifiers (e.g., `KeyModifiers::CONTROL`)
/// * `handler` - A callback function that will be invoked when the shortcut is pressed
///
/// # Examples
///
/// ```rust,no_run
/// use reratui_hooks::keyboard::use_keyboard_shortcut;
/// use reratui_hooks::state::use_state;
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// // Ctrl+S to save
/// let (saved, set_saved) = use_state(|| false);
/// use_keyboard_shortcut(KeyCode::Char('s'), KeyModifiers::CONTROL, {
///     let set_saved = set_saved.clone();
///     move || {
///         println!("Save triggered!");
///         set_saved.set(true);
///     }
/// });
///
/// // Alt+Q to quit
/// use_keyboard_shortcut(KeyCode::Char('q'), KeyModifiers::ALT, || {
///     println!("Quit triggered!");
/// });
///
/// // No modifiers - just Enter
/// use_keyboard_shortcut(KeyCode::Enter, KeyModifiers::NONE, || {
///     println!("Enter pressed!");
/// });
///
/// // Ctrl+Shift+P for command palette
/// use_keyboard_shortcut(
///     KeyCode::Char('p'),
///     KeyModifiers::CONTROL | KeyModifiers::SHIFT,
///     || {
///         println!("Command palette opened!");
///     }
/// );
/// ```
///
/// # Note
///
/// - Only triggers on exact matches of key code AND modifiers
/// - Uses `use_keyboard_press` internally (only press events, no release/repeat)
/// - The callback always sees the latest state values (via effect event pattern)
/// - The callback has a stable identity across renders
/// - For multiple modifiers, use bitwise OR: `KeyModifiers::CONTROL | KeyModifiers::SHIFT`
pub fn use_keyboard_shortcut<F>(key_code: KeyCode, modifiers: KeyModifiers, handler: F)
where
    F: Fn() + Clone + Send + Sync + 'static,
{
    use_keyboard_press(move |key_event| {
        // Check if both key code and modifiers match
        if key_event.code == key_code && key_event.modifiers == modifiers {
            handler();
        }
    });
}
