//! Tests for the keyboard hook

use super::*;
use crate::{
    event::set_current_event,
    state::use_state,
    test_utils::{with_component_id, with_test_isolate},
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use parking_lot::Mutex;
use std::sync::{Arc, LazyLock};

// Test mutex to prevent parallel test execution
static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
fn test_use_keyboard_basic() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let captured_key = Arc::new(Mutex::new(None));

        let called_clone = called.clone();
        let key_clone = captured_key.clone();

        // Set a key event
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardTest", |_ctx| {
            use_keyboard(move |key| {
                *called_clone.lock() = true;
                *key_clone.lock() = Some(key);
            });
        });

        assert!(*called.lock(), "Callback should have been called");
        assert_eq!(
            captured_key.lock().unwrap().code,
            KeyCode::Char('a'),
            "Should capture correct key"
        );
    });
}

#[test]
fn test_use_keyboard_no_event() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        set_current_event(None);

        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        with_component_id("KeyboardNoEventTest", |_ctx| {
            use_keyboard(move |_| {
                *called_clone.lock() = true;
            });
        });

        assert!(
            !*called.lock(),
            "Callback should not be called without event"
        );
    });
}

#[test]
fn test_use_keyboard_ignores_non_keyboard_events() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Set a resize event (not keyboard)
        set_current_event(Some(Arc::new(Event::Resize(80, 24))));

        with_component_id("KeyboardIgnoreTest", |_ctx| {
            use_keyboard(move |_| {
                *called_clone.lock() = true;
            });
        });

        assert!(
            !*called.lock(),
            "Callback should not be called for non-keyboard events"
        );
    });
}

#[test]
fn test_use_keyboard_with_state() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardStateTest", |_ctx| {
            let (count, set_count) = use_state(|| 0);

            use_keyboard({
                let set_count = set_count.clone();
                move |_| {
                    set_count.update(|c| *c + 1);
                }
            });

            assert_eq!(count.get(), 1, "State should be updated");
        });
    });
}

#[test]
fn test_use_keyboard_multiple_keys() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let keys = Arc::new(Mutex::new(Vec::new()));
        let keys_clone = keys.clone();

        // First key
        let key1 = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        set_current_event(Some(Arc::new(Event::Key(key1))));

        with_component_id("KeyboardMultipleTest", |_ctx| {
            use_keyboard({
                let keys = keys_clone.clone();
                move |key| {
                    keys.lock().push(key.code);
                }
            });
        });

        assert_eq!(keys.lock().len(), 1);
        assert_eq!(keys.lock()[0], KeyCode::Char('x'));
    });
}

#[test]
fn test_use_keyboard_with_modifiers() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let captured_modifiers = Arc::new(Mutex::new(KeyModifiers::NONE));
        let mod_clone = captured_modifiers.clone();

        // Key with Ctrl modifier
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardModifiersTest", |_ctx| {
            use_keyboard(move |key| {
                *mod_clone.lock() = key.modifiers;
            });
        });

        assert_eq!(
            *captured_modifiers.lock(),
            KeyModifiers::CONTROL,
            "Should capture modifiers"
        );
    });
}

#[test]
fn test_use_keyboard_special_keys() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let captured_key = Arc::new(Mutex::new(None));
        let key_clone = captured_key.clone();

        // Special key (Escape)
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardSpecialTest", |_ctx| {
            use_keyboard(move |key| {
                *key_clone.lock() = Some(key.code);
            });
        });

        assert_eq!(
            captured_key.lock().unwrap(),
            KeyCode::Esc,
            "Should handle special keys"
        );
    });
}

#[test]
fn test_use_keyboard_effect_event_pattern() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // This test verifies that the callback sees the latest state
        // even though it's created with an old value
        let state_value = Arc::new(Mutex::new(0));
        let state_clone = state_value.clone();

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardEffectEventTest", |_ctx| {
            let (count, set_count) = use_state(|| 0);

            // Update state before keyboard handler runs
            set_count.set(42);

            use_keyboard({
                let state = state_clone.clone();
                let count_val = count.get();
                move |_| {
                    *state.lock() = count_val;
                }
            });
        });

        // The callback should see the updated state value (42)
        assert_eq!(
            *state_value.lock(),
            42,
            "Callback should see latest state via effect event pattern"
        );
    });
}

#[test]
fn test_use_keyboard_arrow_keys() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let direction = Arc::new(Mutex::new(String::new()));
        let dir_clone = direction.clone();

        let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardArrowTest", |_ctx| {
            use_keyboard(move |key| {
                let dir = match key.code {
                    KeyCode::Up => "up",
                    KeyCode::Down => "down",
                    KeyCode::Left => "left",
                    KeyCode::Right => "right",
                    _ => "other",
                };
                *dir_clone.lock() = dir.to_string();
            });
        });

        assert_eq!(*direction.lock(), "up", "Should handle arrow keys");
    });
}

#[test]
fn test_use_keyboard_function_keys() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let captured = Arc::new(Mutex::new(false));
        let cap_clone = captured.clone();

        let key_event = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardFunctionTest", |_ctx| {
            use_keyboard(move |key| {
                if matches!(key.code, KeyCode::F(_)) {
                    *cap_clone.lock() = true;
                }
            });
        });

        assert!(*captured.lock(), "Should handle function keys");
    });
}

#[test]
fn test_use_keyboard_press_only_press_events() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let press_count = Arc::new(Mutex::new(0));
        let count_clone = press_count.clone();

        // Create a Press event
        let key_event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardPressTest", |_ctx| {
            use_keyboard_press(move |_| {
                *count_clone.lock() += 1;
            });
        });

        assert_eq!(*press_count.lock(), 1, "Should handle press events");
    });
}

#[test]
fn test_use_keyboard_press_ignores_release() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Create a Release event
        let key_event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardPressIgnoreReleaseTest", |_ctx| {
            use_keyboard_press(move |_| {
                *called_clone.lock() = true;
            });
        });

        assert!(!*called.lock(), "Should not handle release events");
    });
}

#[test]
fn test_use_keyboard_press_ignores_repeat() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Create a Repeat event
        let key_event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Repeat,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardPressIgnoreRepeatTest", |_ctx| {
            use_keyboard_press(move |_| {
                *called_clone.lock() = true;
            });
        });

        assert!(!*called.lock(), "Should not handle repeat events");
    });
}

#[test]
fn test_use_keyboard_press_with_state() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let key_event = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardPressStateTest", |_ctx| {
            let (count, set_count) = use_state(|| 0);

            use_keyboard_press({
                let set_count = set_count.clone();
                move |_| {
                    set_count.update(|c| *c + 1);
                }
            });

            assert_eq!(count.get(), 1, "State should be updated on press");
        });
    });
}

#[test]
fn test_use_keyboard_press_effect_event_pattern() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let state_value = Arc::new(Mutex::new(0));
        let state_clone = state_value.clone();

        let key_event = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("KeyboardPressEffectEventTest", |_ctx| {
            let (count, set_count) = use_state(|| 0);

            // Update state before handler runs
            set_count.set(77);

            use_keyboard_press({
                let state = state_clone.clone();
                let count_val = count.get();
                move |_| {
                    *state.lock() = count_val;
                }
            });
        });

        assert_eq!(
            *state_value.lock(),
            77,
            "Callback should see latest state via effect event pattern"
        );
    });
}

#[test]
fn test_use_keyboard_shortcut_ctrl_s() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Create Ctrl+S event
        let key_event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("ShortcutCtrlSTest", |_ctx| {
            use_keyboard_shortcut(KeyCode::Char('s'), KeyModifiers::CONTROL, move || {
                *called_clone.lock() = true;
            });
        });

        assert!(*called.lock(), "Ctrl+S shortcut should be triggered");
    });
}

#[test]
fn test_use_keyboard_shortcut_no_match() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Create Ctrl+A event (but we're listening for Ctrl+S)
        let key_event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("ShortcutNoMatchTest", |_ctx| {
            use_keyboard_shortcut(KeyCode::Char('s'), KeyModifiers::CONTROL, move || {
                *called_clone.lock() = true;
            });
        });

        assert!(!*called.lock(), "Wrong shortcut should not trigger");
    });
}

#[test]
fn test_use_keyboard_shortcut_wrong_modifier() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Create Alt+S event (but we're listening for Ctrl+S)
        let key_event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("ShortcutWrongModifierTest", |_ctx| {
            use_keyboard_shortcut(KeyCode::Char('s'), KeyModifiers::CONTROL, move || {
                *called_clone.lock() = true;
            });
        });

        assert!(!*called.lock(), "Wrong modifier should not trigger");
    });
}

#[test]
fn test_use_keyboard_shortcut_no_modifier() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Create plain Enter event (no modifiers)
        let key_event = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("ShortcutNoModifierTest", |_ctx| {
            use_keyboard_shortcut(KeyCode::Enter, KeyModifiers::NONE, move || {
                *called_clone.lock() = true;
            });
        });

        assert!(*called.lock(), "Enter without modifiers should trigger");
    });
}

#[test]
fn test_use_keyboard_shortcut_multiple_modifiers() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Create Ctrl+Shift+P event
        let key_event = KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("ShortcutMultipleModifiersTest", |_ctx| {
            use_keyboard_shortcut(
                KeyCode::Char('p'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                move || {
                    *called_clone.lock() = true;
                },
            );
        });

        assert!(*called.lock(), "Ctrl+Shift+P shortcut should be triggered");
    });
}

#[test]
fn test_use_keyboard_shortcut_with_state() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let key_event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("ShortcutStateTest", |_ctx| {
            let (saved, set_saved) = use_state(|| false);

            use_keyboard_shortcut(KeyCode::Char('s'), KeyModifiers::CONTROL, {
                let set_saved = set_saved.clone();
                move || {
                    set_saved.set(true);
                }
            });

            assert!(saved.get(), "State should be updated by shortcut");
        });
    });
}
