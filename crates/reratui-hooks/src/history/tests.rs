//! Tests for use_history hook

use super::*;
use crate::test_utils::{with_component_id, with_test_isolate};

#[test]
fn test_history_initial_state() {
    with_test_isolate(|| {
        with_component_id("HistoryInitialTest", |_ctx| {
            let history = use_history(0, 10);

            assert_eq!(history.current(), 0, "Initial state should be 0");
            assert!(!history.can_undo(), "Should not be able to undo initially");
            assert!(!history.can_redo(), "Should not be able to redo initially");
        });
    });
}

#[test]
fn test_history_push() {
    with_test_isolate(|| {
        with_component_id("HistoryPushTest", |_ctx| {
            let history = use_history(0, 10);

            history.push(1);
            assert_eq!(history.current(), 1, "Current state should be 1 after push");
            assert!(history.can_undo(), "Should be able to undo after push");
            assert!(!history.can_redo(), "Should not be able to redo after push");

            history.push(2);
            assert_eq!(
                history.current(),
                2,
                "Current state should be 2 after second push"
            );
            assert!(
                history.can_undo(),
                "Should be able to undo after second push"
            );
        });
    });
}

#[test]
fn test_history_undo() {
    with_test_isolate(|| {
        with_component_id("HistoryUndoTest", |_ctx| {
            let history = use_history(0, 10);

            history.push(1);
            history.push(2);
            history.push(3);

            assert_eq!(history.current(), 3, "Current should be 3");

            history.undo();
            assert_eq!(history.current(), 2, "After undo, current should be 2");
            assert!(history.can_undo(), "Should still be able to undo");
            assert!(history.can_redo(), "Should be able to redo");

            history.undo();
            assert_eq!(
                history.current(),
                1,
                "After second undo, current should be 1"
            );

            history.undo();
            assert_eq!(
                history.current(),
                0,
                "After third undo, current should be 0"
            );
            assert!(!history.can_undo(), "Should not be able to undo anymore");
        });
    });
}

#[test]
fn test_history_redo() {
    with_test_isolate(|| {
        with_component_id("HistoryRedoTest", |_ctx| {
            let history = use_history(0, 10);

            history.push(1);
            history.push(2);
            history.push(3);

            history.undo();
            history.undo();

            assert_eq!(history.current(), 1, "After two undos, current should be 1");

            history.redo();
            assert_eq!(history.current(), 2, "After redo, current should be 2");
            assert!(history.can_redo(), "Should still be able to redo");

            history.redo();
            assert_eq!(
                history.current(),
                3,
                "After second redo, current should be 3"
            );
            assert!(!history.can_redo(), "Should not be able to redo anymore");
        });
    });
}

#[test]
fn test_history_push_clears_redo() {
    with_test_isolate(|| {
        with_component_id("HistoryPushClearsRedoTest", |_ctx| {
            let history = use_history(0, 10);

            history.push(1);
            history.push(2);
            history.undo();

            assert!(history.can_redo(), "Should be able to redo before push");

            history.push(3);
            assert!(
                !history.can_redo(),
                "Redo stack should be cleared after push"
            );
            assert_eq!(history.current(), 3, "Current should be 3");
        });
    });
}

#[test]
fn test_history_max_size() {
    with_test_isolate(|| {
        with_component_id("HistoryMaxSizeTest", |_ctx| {
            let history = use_history(0, 3);

            // Push more than max_history items
            for i in 1..=5 {
                history.push(i);
            }

            assert_eq!(history.current(), 5, "Current should be 5");

            // Undo all the way
            history.undo(); // 5 -> 4
            history.undo(); // 4 -> 3
            history.undo(); // 3 -> 2

            assert_eq!(
                history.current(),
                2,
                "Should only be able to undo 3 times (max_history)"
            );
            assert!(!history.can_undo(), "Should not be able to undo further");
        });
    });
}

#[test]
fn test_history_with_strings() {
    with_test_isolate(|| {
        with_component_id("HistoryStringsTest", |_ctx| {
            let history = use_history(String::from("initial"), 10);

            assert_eq!(history.current(), "initial");

            history.push(String::from("first"));
            history.push(String::from("second"));

            assert_eq!(history.current(), "second");

            history.undo();
            assert_eq!(history.current(), "first");

            history.redo();
            assert_eq!(history.current(), "second");
        });
    });
}

#[test]
fn test_history_with_complex_type() {
    with_test_isolate(|| {
        with_component_id("HistoryComplexTypeTest", |_ctx| {
            #[derive(Clone, PartialEq, Debug)]
            struct AppState {
                counter: i32,
                text: String,
            }

            let initial = AppState {
                counter: 0,
                text: String::from("initial"),
            };

            let history = use_history(initial, 10);

            let state1 = AppState {
                counter: 1,
                text: String::from("first"),
            };
            history.push(state1.clone());

            assert_eq!(history.current().counter, 1);
            assert_eq!(history.current().text, "first");

            history.undo();
            assert_eq!(history.current().counter, 0);
            assert_eq!(history.current().text, "initial");
        });
    });
}

#[test]
fn test_history_undo_without_history() {
    with_test_isolate(|| {
        with_component_id("HistoryUndoWithoutHistoryTest", |_ctx| {
            let history = use_history(42, 10);

            // Try to undo when there's no history
            history.undo();

            // Should remain at initial state
            assert_eq!(history.current(), 42);
            assert!(!history.can_undo());
        });
    });
}

#[test]
fn test_history_redo_without_future() {
    with_test_isolate(|| {
        with_component_id("HistoryRedoWithoutFutureTest", |_ctx| {
            let history = use_history(42, 10);

            history.push(43);

            // Try to redo when there's no future
            history.redo();

            // Should remain at current state
            assert_eq!(history.current(), 43);
            assert!(!history.can_redo());
        });
    });
}

#[test]
fn test_history_multiple_undo_redo_cycles() {
    with_test_isolate(|| {
        with_component_id("HistoryMultipleCyclesTest", |_ctx| {
            let history = use_history(0, 10);

            // First cycle
            history.push(1);
            history.push(2);
            history.undo();
            history.redo();
            assert_eq!(history.current(), 2);

            // Second cycle
            history.push(3);
            history.undo();
            history.undo();
            assert_eq!(history.current(), 1);

            history.redo();
            history.redo();
            assert_eq!(history.current(), 3);
        });
    });
}

#[test]
fn test_history_max_size_one() {
    with_test_isolate(|| {
        with_component_id("HistoryMaxSizeOneTest", |_ctx| {
            let history = use_history(0, 1);

            history.push(1);
            history.push(2);

            // Should only remember one previous state
            history.undo();
            assert_eq!(history.current(), 1);

            history.undo();
            // Should not be able to undo further
            assert_eq!(history.current(), 1);
            assert!(!history.can_undo());
        });
    });
}

#[test]
fn test_history_sequence_operations() {
    with_test_isolate(|| {
        with_component_id("HistorySequenceTest", |_ctx| {
            let history = use_history(vec![1], 5);

            history.push(vec![1, 2]);
            history.push(vec![1, 2, 3]);
            history.push(vec![1, 2, 3, 4]);

            assert_eq!(history.current(), vec![1, 2, 3, 4]);

            history.undo();
            assert_eq!(history.current(), vec![1, 2, 3]);

            history.push(vec![1, 2, 3, 5]);
            assert_eq!(history.current(), vec![1, 2, 3, 5]);
            assert!(!history.can_redo(), "Redo should be cleared after push");

            history.undo();
            history.undo();
            assert_eq!(history.current(), vec![1, 2]);
        });
    });
}
