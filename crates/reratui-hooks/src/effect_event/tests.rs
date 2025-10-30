use super::*;
use crate::hook_context::{HookContext, set_hook_context};
use crate::state::use_state;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Helper function to set up a hook context for testing
fn setup_context() -> Rc<HookContext> {
    let context = Rc::new(HookContext::new());
    set_hook_context(context.clone());
    context
}

#[test]
fn test_effect_event_basic() {
    let context = setup_context();

    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    let event = use_effect_event(move |_: ()| {
        *called_clone.lock().unwrap() = true;
    });

    assert!(!*called.lock().unwrap());
    event.emit(());
    assert!(*called.lock().unwrap());

    context.clear();
}

#[test]
fn test_effect_event_with_arguments() {
    let context = setup_context();

    let result = Arc::new(Mutex::new(0));
    let result_clone = result.clone();

    let event = use_effect_event(move |value: i32| {
        *result_clone.lock().unwrap() = value * 2;
    });

    event.emit(5);
    assert_eq!(*result.lock().unwrap(), 10);

    event.emit(7);
    assert_eq!(*result.lock().unwrap(), 14);

    context.clear();
}

#[test]
fn test_effect_event_with_return_value() {
    let context = setup_context();

    let event = use_effect_event(|x: i32| x * 2);

    let result = event.emit(5);
    assert_eq!(result, 10);

    let result = event.emit(10);
    assert_eq!(result, 20);

    context.clear();
}

#[test]
fn test_effect_event_updates_handler() {
    let context = setup_context();

    // First render - handler returns 1
    let event = use_effect_event(|_: ()| 1);
    assert_eq!(event.emit(()), 1);

    // Simulate re-render with new handler
    context.reset_hook_index();

    // Second render - handler returns 2
    let event = use_effect_event(|_: ()| 2);
    assert_eq!(event.emit(()), 2);

    context.clear();
}

#[test]
fn test_effect_event_with_captured_state() {
    let context = setup_context();

    // First render
    let (count, set_count) = use_state(|| 0);
    let event = use_effect_event(move |_: ()| count.get());

    assert_eq!(event.emit(()), 0);

    // Update state
    set_count.set(5);

    // Simulate re-render
    context.reset_hook_index();

    // Second render - event should see new state
    let (count, _) = use_state(|| 0);
    let event = use_effect_event(move |_: ()| count.get());

    assert_eq!(event.emit(()), 5);

    context.clear();
}

#[test]
fn test_effect_event_clone() {
    let context = setup_context();

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let event = use_effect_event(move |_: ()| {
        *counter_clone.lock().unwrap() += 1;
    });

    // Clone the event
    let event_clone = event.clone();

    // Both should call the same handler
    event.emit(());
    assert_eq!(*counter.lock().unwrap(), 1);

    event_clone.emit(());
    assert_eq!(*counter.lock().unwrap(), 2);

    context.clear();
}

#[test]
fn test_effect_event_multiple_calls() {
    let context = setup_context();

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let event = use_effect_event(move |_: ()| {
        *counter_clone.lock().unwrap() += 1;
        *counter_clone.lock().unwrap()
    });

    assert_eq!(event.emit(()), 1);
    assert_eq!(event.emit(()), 2);
    assert_eq!(event.emit(()), 3);

    context.clear();
}

#[test]
fn test_effect_event_closure_captures() {
    let context = setup_context();

    let multiplier = 3;
    let event = use_effect_event(move |x: i32| x * multiplier);

    assert_eq!(event.emit(5), 15);
    assert_eq!(event.emit(10), 30);

    context.clear();
}

#[test]
fn test_effect_event_updates_closure_captures() {
    let context = setup_context();

    // First render with multiplier = 2
    let multiplier = 2;
    let event = use_effect_event(move |x: i32| x * multiplier);
    assert_eq!(event.emit(5), 10);

    // Simulate re-render with multiplier = 3
    context.reset_hook_index();
    let multiplier = 3;
    let event = use_effect_event(move |x: i32| x * multiplier);
    assert_eq!(event.emit(5), 15);

    context.clear();
}

#[test]
fn test_effect_event_with_state_pattern() {
    let context = setup_context();

    // Simulate a component with state
    let (count, set_count) = use_state(|| 0);

    // Create event that logs current count
    let log_count = use_effect_event(move |_: ()| format!("Count: {}", count.get()));

    assert_eq!(log_count.emit(()), "Count: 0");

    // Update state
    set_count.set(5);

    // Simulate re-render
    context.reset_hook_index();

    let (count, _) = use_state(|| 0);
    let log_count = use_effect_event(move |_: ()| format!("Count: {}", count.get()));

    // Event should see new state
    assert_eq!(log_count.emit(()), "Count: 5");

    context.clear();
}

#[test]
fn test_effect_event_with_option_return() {
    let context = setup_context();

    let event = use_effect_event(|value: i32| if value > 0 { Some(value * 2) } else { None });

    assert_eq!(event.emit(5), Some(10));
    assert_eq!(event.emit(-1), None);
    assert_eq!(event.emit(0), None);

    context.clear();
}

#[test]
fn test_effect_event_with_result_return() {
    let context = setup_context();

    let event = use_effect_event(|value: i32| -> Result<i32, String> {
        if value > 0 {
            Ok(value * 2)
        } else {
            Err("Value must be positive".to_string())
        }
    });

    assert_eq!(event.emit(5), Ok(10));
    assert!(event.emit(-1).is_err());

    context.clear();
}

#[test]
fn test_effect_event_tuple_args() {
    let context = setup_context();

    let event = use_effect_event(|(x, y): (i32, i32)| x + y);

    assert_eq!(event.emit((5, 3)), 8);
    assert_eq!(event.emit((10, 20)), 30);

    context.clear();
}

#[test]
fn test_effect_event_string_operations() {
    let context = setup_context();

    let prefix = "Hello, ";
    let event = use_effect_event(move |name: String| format!("{}{}", prefix, name));

    assert_eq!(event.emit("World".to_string()), "Hello, World");
    assert_eq!(event.emit("Rust".to_string()), "Hello, Rust");

    context.clear();
}
