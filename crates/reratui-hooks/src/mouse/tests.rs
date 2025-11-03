//! Tests for the mouse hook

use super::*;
use crate::{
    event::set_current_event,
    state::use_state,
    test_utils::{with_component_id, with_test_isolate},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use parking_lot::Mutex;
use std::sync::{Arc, LazyLock};

// Test mutex to prevent parallel test execution
static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
fn test_use_mouse_basic() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let captured_mouse = Arc::new(Mutex::new(None));

        let called_clone = called.clone();
        let mouse_clone = captured_mouse.clone();

        // Set a mouse click event
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseTest", |_ctx| {
            use_mouse(move |mouse| {
                *called_clone.lock() = true;
                *mouse_clone.lock() = Some(mouse);
            });
        });

        assert!(*called.lock(), "Callback should have been called");
        let captured = captured_mouse.lock().unwrap();
        assert_eq!(captured.column, 10, "Should capture correct column");
        assert_eq!(captured.row, 5, "Should capture correct row");
    });
}

#[test]
fn test_use_mouse_no_event() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        set_current_event(None);

        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        with_component_id("MouseNoEventTest", |_ctx| {
            use_mouse(move |_| {
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
fn test_use_mouse_ignores_non_mouse_events() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Set a keyboard event (not mouse)
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("MouseIgnoreTest", |_ctx| {
            use_mouse(move |_| {
                *called_clone.lock() = true;
            });
        });

        assert!(
            !*called.lock(),
            "Callback should not be called for non-mouse events"
        );
    });
}

#[test]
fn test_use_mouse_with_state() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 20,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseStateTest", |_ctx| {
            let (count, set_count) = use_state(|| 0);

            use_mouse({
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
fn test_use_mouse_click_detection() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let click_type = Arc::new(Mutex::new(String::new()));
        let click_clone = click_type.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 15,
            row: 8,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseClickTest", |_ctx| {
            use_mouse(move |mouse| match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    *click_clone.lock() = "left_down".to_string()
                }
                MouseEventKind::Down(MouseButton::Right) => {
                    *click_clone.lock() = "right_down".to_string()
                }
                MouseEventKind::Up(_) => *click_clone.lock() = "up".to_string(),
                _ => {}
            });
        });

        assert_eq!(*click_type.lock(), "left_down", "Should detect left click");
    });
}

#[test]
fn test_use_mouse_movement() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let position = Arc::new(Mutex::new((0u16, 0u16)));
        let pos_clone = position.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 42,
            row: 24,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseMoveTest", |_ctx| {
            use_mouse(move |mouse| {
                if matches!(mouse.kind, MouseEventKind::Moved) {
                    *pos_clone.lock() = (mouse.column, mouse.row);
                }
            });
        });

        assert_eq!(*position.lock(), (42, 24), "Should track mouse movement");
    });
}

#[test]
fn test_use_mouse_scroll() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let scroll_direction = Arc::new(Mutex::new(String::new()));
        let scroll_clone = scroll_direction.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseScrollTest", |_ctx| {
            use_mouse(move |mouse| {
                let dir = match mouse.kind {
                    MouseEventKind::ScrollUp => "up",
                    MouseEventKind::ScrollDown => "down",
                    MouseEventKind::ScrollLeft => "left",
                    MouseEventKind::ScrollRight => "right",
                    _ => "none",
                };
                *scroll_clone.lock() = dir.to_string();
            });
        });

        assert_eq!(
            *scroll_direction.lock(),
            "down",
            "Should detect scroll direction"
        );
    });
}

#[test]
fn test_use_mouse_with_modifiers() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let has_ctrl = Arc::new(Mutex::new(false));
        let ctrl_clone = has_ctrl.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 10,
            modifiers: KeyModifiers::CONTROL,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseModifiersTest", |_ctx| {
            use_mouse(move |mouse| {
                *ctrl_clone.lock() = mouse.modifiers.contains(KeyModifiers::CONTROL);
            });
        });

        assert!(*has_ctrl.lock(), "Should detect Ctrl modifier");
    });
}

#[test]
fn test_use_mouse_right_click() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let right_clicked = Arc::new(Mutex::new(false));
        let click_clone = right_clicked.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseRightClickTest", |_ctx| {
            use_mouse(move |mouse| {
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Right)) {
                    *click_clone.lock() = true;
                }
            });
        });

        assert!(*right_clicked.lock(), "Should detect right click");
    });
}

#[test]
fn test_use_mouse_effect_event_pattern() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // Verify that the callback sees the latest state
        let state_value = Arc::new(Mutex::new(0));
        let state_clone = state_value.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseEffectEventTest", |_ctx| {
            let (count, set_count) = use_state(|| 0);

            // Update state before mouse handler runs
            set_count.set(99);

            use_mouse({
                let state = state_clone.clone();
                let count_val = count.get();
                move |_| {
                    *state.lock() = count_val;
                }
            });
        });

        // The callback should see the updated state value (99)
        assert_eq!(
            *state_value.lock(),
            99,
            "Callback should see latest state via effect event pattern"
        );
    });
}

// Tests for use_mouse_click
#[test]
fn test_use_mouse_click_left_button() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let clicked = Arc::new(Mutex::new(false));
        let position = Arc::new(Mutex::new((0u16, 0u16)));
        let clicked_clone = clicked.clone();
        let pos_clone = position.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 15,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseClickTest", |_ctx| {
            use_mouse_click(move |button, x, y| {
                if button == MouseButton::Left {
                    *clicked_clone.lock() = true;
                    *pos_clone.lock() = (x, y);
                }
            });
        });

        assert!(*clicked.lock(), "Left click should be detected");
        assert_eq!(*position.lock(), (15, 10), "Click position should match");
    });
}

#[test]
fn test_use_mouse_click_ignores_movement() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 20,
            row: 20,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseClickIgnoreMovementTest", |_ctx| {
            use_mouse_click(move |_, _, _| {
                *called_clone.lock() = true;
            });
        });

        assert!(!*called.lock(), "Movement should not trigger click handler");
    });
}

#[test]
fn test_use_mouse_click_ignores_drag() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 25,
            row: 25,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseClickIgnoreDragTest", |_ctx| {
            use_mouse_click(move |_, _, _| {
                *called_clone.lock() = true;
            });
        });

        assert!(!*called.lock(), "Drag should not trigger click handler");
    });
}

#[test]
fn test_use_mouse_click_right_button() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let button_type = Arc::new(Mutex::new(None));
        let button_clone = button_type.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseClickRightTest", |_ctx| {
            use_mouse_click(move |button, _, _| {
                *button_clone.lock() = Some(button);
            });
        });

        assert_eq!(
            *button_type.lock(),
            Some(MouseButton::Right),
            "Right button should be detected"
        );
    });
}

// Tests for use_mouse_drag
#[test]
fn test_use_mouse_drag_start() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let drag_started = Arc::new(Mutex::new(false));
        let start_pos = Arc::new(Mutex::new((0u16, 0u16)));
        let started_clone = drag_started.clone();
        let pos_clone = start_pos.clone();

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseDragStartTest", |_ctx| {
            let (drag_info, _reset) = use_mouse_drag();

            if drag_info.is_start {
                *started_clone.lock() = true;
                *pos_clone.lock() = drag_info.start;
            }
        });

        assert!(*drag_started.lock(), "Drag should start");
        assert_eq!(*start_pos.lock(), (5, 5), "Start position should match");
    });
}

#[test]
fn test_use_mouse_drag_movement() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let drag_moving = Arc::new(Mutex::new(false));
        let current_pos = Arc::new(Mutex::new((0u16, 0u16)));

        // First, start drag
        let down_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(down_event))));

        let moving_clone = drag_moving.clone();
        let pos_clone = current_pos.clone();
        with_component_id("MouseDragMovementTest", |_ctx| {
            let (drag_info, _reset) = use_mouse_drag();

            if drag_info.is_dragging && !drag_info.is_start && !drag_info.is_end {
                *moving_clone.lock() = true;
                *pos_clone.lock() = drag_info.current;
            }
        });

        // Then drag
        let drag_event = MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(drag_event))));

        let moving_clone = drag_moving.clone();
        let pos_clone = current_pos.clone();
        with_component_id("MouseDragMovementTest", |_ctx| {
            let (drag_info, _reset) = use_mouse_drag();

            if drag_info.is_dragging && !drag_info.is_start && !drag_info.is_end {
                *moving_clone.lock() = true;
                *pos_clone.lock() = drag_info.current;
            }
        });

        assert!(*drag_moving.lock(), "Drag movement should be detected");
        assert_eq!(
            *current_pos.lock(),
            (10, 10),
            "Current position should match"
        );
    });
}

#[test]
fn test_use_mouse_drag_end() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let drag_ended = Arc::new(Mutex::new(false));

        // Start drag first
        let down_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(down_event))));

        let ended_clone = drag_ended.clone();
        with_component_id("MouseDragEndTest", |_ctx| {
            let (drag_info, _reset) = use_mouse_drag();

            if drag_info.is_end {
                *ended_clone.lock() = true;
            }
        });

        // End drag
        let up_event = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 15,
            row: 15,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(up_event))));

        let ended_clone = drag_ended.clone();
        with_component_id("MouseDragEndTest", |_ctx| {
            let (drag_info, _reset) = use_mouse_drag();

            if drag_info.is_end {
                *ended_clone.lock() = true;
            }
        });

        assert!(*drag_ended.lock(), "Drag should end");
    });
}

// Tests for use_double_click
#[test]
fn test_use_double_click_detected() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let double_clicked = Arc::new(Mutex::new(false));

        // First click
        let first_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(first_click))));

        let clicked_clone = double_clicked.clone();
        with_component_id("DoubleClickTest", |_ctx| {
            use_double_click(std::time::Duration::from_millis(500), move |_, _, _| {
                *clicked_clone.lock() = true;
            });
        });

        // Second click (immediately after, within window)
        let second_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(second_click))));

        let clicked_clone = double_clicked.clone();
        with_component_id("DoubleClickTest", |_ctx| {
            use_double_click(std::time::Duration::from_millis(500), move |_, _, _| {
                *clicked_clone.lock() = true;
            });
        });

        assert!(*double_clicked.lock(), "Double-click should be detected");
    });
}

#[test]
fn test_use_double_click_different_position() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let double_clicked = Arc::new(Mutex::new(false));

        // First click at (10, 10)
        let first_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(first_click))));

        let clicked_clone = double_clicked.clone();
        with_component_id("DoubleClickDifferentPosTest", |_ctx| {
            use_double_click(std::time::Duration::from_millis(500), move |_, _, _| {
                *clicked_clone.lock() = true;
            });
        });

        // Second click at different position (20, 20)
        let second_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 20,
            row: 20,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(second_click))));

        let clicked_clone = double_clicked.clone();
        with_component_id("DoubleClickDifferentPosTest", |_ctx| {
            use_double_click(std::time::Duration::from_millis(500), move |_, _, _| {
                *clicked_clone.lock() = true;
            });
        });

        assert!(
            !*double_clicked.lock(),
            "Different position should not trigger double-click"
        );
    });
}

#[test]
fn test_use_double_click_different_button() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let double_clicked = Arc::new(Mutex::new(false));

        // First click with left button
        let first_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(first_click))));

        let clicked_clone = double_clicked.clone();
        with_component_id("DoubleClickDifferentButtonTest", |_ctx| {
            use_double_click(std::time::Duration::from_millis(500), move |_, _, _| {
                *clicked_clone.lock() = true;
            });
        });

        // Second click with right button
        let second_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(second_click))));

        let clicked_clone = double_clicked.clone();
        with_component_id("DoubleClickDifferentButtonTest", |_ctx| {
            use_double_click(std::time::Duration::from_millis(500), move |_, _, _| {
                *clicked_clone.lock() = true;
            });
        });

        assert!(
            !*double_clicked.lock(),
            "Different button should not trigger double-click"
        );
    });
}

// Tests for use_mouse_position
#[test]
fn test_use_mouse_position_initial() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        set_current_event(None);

        with_component_id("MousePositionInitialTest", |_ctx| {
            let (x, y) = use_mouse_position();
            assert_eq!((x, y), (0, 0), "Initial position should be (0, 0)");
        });
    });
}

#[test]
fn test_use_mouse_position_updates() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 42,
            row: 24,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MousePositionUpdateTest", |_ctx| {
            let (x, y) = use_mouse_position();
            assert_eq!((x, y), (42, 24), "Position should update to (42, 24)");
        });
    });
}

#[test]
fn test_use_mouse_position_on_click() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 15,
            row: 8,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MousePositionClickTest", |_ctx| {
            let (x, y) = use_mouse_position();
            assert_eq!(
                (x, y),
                (15, 8),
                "Position should update on click to (15, 8)"
            );
        });
    });
}

#[test]
fn test_use_mouse_position_on_scroll() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let mouse_event = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 30,
            row: 20,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MousePositionScrollTest", |_ctx| {
            let (x, y) = use_mouse_position();
            assert_eq!(
                (x, y),
                (30, 20),
                "Position should update on scroll to (30, 20)"
            );
        });
    });
}

#[test]
fn test_use_mouse_position_ignores_non_mouse() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // Set a keyboard event
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        set_current_event(Some(Arc::new(Event::Key(key_event))));

        with_component_id("MousePositionIgnoreKeyTest", |_ctx| {
            let (x, y) = use_mouse_position();
            assert_eq!(
                (x, y),
                (0, 0),
                "Position should remain (0, 0) for non-mouse events"
            );
        });
    });
}

// Tests for use_mouse_hover
#[test]
fn test_use_mouse_hover_inside_area() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        use ratatui::layout::Rect;

        let area = Rect::new(10, 5, 20, 10);
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 15, // Inside area (10 <= 15 < 30)
            row: 8,     // Inside area (5 <= 8 < 15)
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseHoverInsideTest", |_ctx| {
            let is_hovering = super::use_mouse_hover(area);
            assert!(is_hovering, "Should detect hover inside area");
        });
    });
}

#[test]
fn test_use_mouse_hover_outside_area() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        use ratatui::layout::Rect;

        let area = Rect::new(10, 5, 20, 10);
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 5, // Outside area (< 10)
            row: 8,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseHoverOutsideTest", |_ctx| {
            let is_hovering = super::use_mouse_hover(area);
            assert!(!is_hovering, "Should not detect hover outside area");
        });
    });
}

#[test]
fn test_use_mouse_hover_on_boundary() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        use ratatui::layout::Rect;

        let area = Rect::new(10, 5, 20, 10);

        // Test left boundary (inclusive)
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 10, // On left boundary
            row: 8,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseHoverBoundaryTest", |_ctx| {
            let is_hovering = super::use_mouse_hover(area);
            assert!(is_hovering, "Should detect hover on left boundary");
        });
    });
}

#[test]
fn test_use_mouse_hover_right_boundary() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        use ratatui::layout::Rect;

        let area = Rect::new(10, 5, 20, 10);

        // Test right boundary (exclusive)
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 30, // On right boundary (10 + 20 = 30, exclusive)
            row: 8,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseHoverRightBoundaryTest", |_ctx| {
            let is_hovering = super::use_mouse_hover(area);
            assert!(
                !is_hovering,
                "Should not detect hover on right boundary (exclusive)"
            );
        });
    });
}

#[test]
fn test_use_mouse_hover_click_inside() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        use ratatui::layout::Rect;

        let area = Rect::new(10, 5, 20, 10);
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 20,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseHoverClickTest", |_ctx| {
            let is_hovering = super::use_mouse_hover(area);
            assert!(is_hovering, "Should detect hover on click inside area");
        });
    });
}

#[test]
fn test_use_mouse_hover_no_event() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        use ratatui::layout::Rect;

        set_current_event(None);
        let area = Rect::new(10, 5, 20, 10);

        with_component_id("MouseHoverNoEventTest", |_ctx| {
            let is_hovering = super::use_mouse_hover(area);
            assert!(!is_hovering, "Should not detect hover without event");
        });
    });
}

#[test]
fn test_use_mouse_hover_corner_cases() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        use ratatui::layout::Rect;

        let area = Rect::new(10, 5, 20, 10);

        // Test bottom-right corner (exclusive)
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 29, // Just inside right boundary
            row: 14,    // Just inside bottom boundary
            modifiers: KeyModifiers::NONE,
        };
        set_current_event(Some(Arc::new(Event::Mouse(mouse_event))));

        with_component_id("MouseHoverCornerTest", |_ctx| {
            let is_hovering = super::use_mouse_hover(area);
            assert!(
                is_hovering,
                "Should detect hover at bottom-right corner (inside)"
            );
        });
    });
}
