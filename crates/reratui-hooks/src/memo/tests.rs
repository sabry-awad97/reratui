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
fn test_memo_basic() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            42
        },
        1,
    );

    assert_eq!(result, 42);
    assert_eq!(*call_count.lock().unwrap(), 1);

    context.clear();
}

#[test]
fn test_memo_caches_value() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    // First render
    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            42
        },
        1,
    );
    assert_eq!(result, 42);
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Simulate re-render with same dependency
    context.reset_hook_index();

    let call_count_clone = call_count.clone();
    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            42
        },
        1,
    );

    assert_eq!(result, 42);
    // Should not recompute
    assert_eq!(*call_count.lock().unwrap(), 1);

    context.clear();
}

#[test]
fn test_memo_recomputes_on_dependency_change() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    // First render with dep = 1
    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            42
        },
        1,
    );
    assert_eq!(result, 42);
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Simulate re-render with different dependency
    context.reset_hook_index();

    let call_count_clone = call_count.clone();
    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            100
        },
        2, // Different dependency
    );

    assert_eq!(result, 100);
    // Should recompute
    assert_eq!(*call_count.lock().unwrap(), 2);

    context.clear();
}

#[test]
fn test_memo_with_state() {
    let context = setup_context();

    let (count, set_count) = use_state(|| 0);

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();
    let count_clone = count.clone();

    let doubled = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            count_clone.get() * 2
        },
        count.get(),
    );

    assert_eq!(doubled, 0);
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Update state
    set_count.set(5);

    // Simulate re-render
    context.reset_hook_index();

    let (count, _) = use_state(|| 0);
    let call_count_clone = call_count.clone();
    let count_clone = count.clone();

    let doubled = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            count_clone.get() * 2
        },
        count.get(),
    );

    assert_eq!(doubled, 10);
    // Should recompute
    assert_eq!(*call_count.lock().unwrap(), 2);

    context.clear();
}

#[test]
fn test_memo_with_tuple_dependencies() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            10 + 20
        },
        (10, 20),
    );

    assert_eq!(result, 30);
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Re-render with same deps
    context.reset_hook_index();

    let call_count_clone = call_count.clone();
    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            10 + 20
        },
        (10, 20),
    );

    assert_eq!(result, 30);
    // Should not recompute
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Re-render with different deps
    context.reset_hook_index();

    let call_count_clone = call_count.clone();
    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            15 + 25
        },
        (15, 25),
    );

    assert_eq!(result, 40);
    // Should recompute
    assert_eq!(*call_count.lock().unwrap(), 2);

    context.clear();
}

#[test]
fn test_memo_once() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    let result = use_memo_once(move || {
        *call_count_clone.lock().unwrap() += 1;
        42
    });

    assert_eq!(result, 42);
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Re-render
    context.reset_hook_index();

    let call_count_clone = call_count.clone();
    let result = use_memo_once(move || {
        *call_count_clone.lock().unwrap() += 1;
        100
    });

    // Should return cached value
    assert_eq!(result, 42);
    // Should not recompute
    assert_eq!(*call_count.lock().unwrap(), 1);

    context.clear();
}

#[test]
fn test_memo_with_complex_computation() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            // Simulate expensive computation
            (1..=100).sum::<i32>()
        },
        "key",
    );

    assert_eq!(result, 5050);
    assert_eq!(*call_count.lock().unwrap(), 1);

    context.clear();
}

#[test]
fn test_memo_with_vec() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            vec![1, 2, 3, 4, 5]
        },
        1,
    );

    assert_eq!(result, vec![1, 2, 3, 4, 5]);
    assert_eq!(*call_count.lock().unwrap(), 1);

    context.clear();
}

#[test]
fn test_memo_with_string() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    let name = "World";
    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            format!("Hello, {}!", name)
        },
        name,
    );

    assert_eq!(result, "Hello, World!");
    assert_eq!(*call_count.lock().unwrap(), 1);

    context.clear();
}

#[test]
fn test_memo_multiple_memos() {
    let context = setup_context();

    let call_count1 = Arc::new(Mutex::new(0));
    let call_count2 = Arc::new(Mutex::new(0));

    let call_count1_clone = call_count1.clone();
    let result1 = use_memo(
        move || {
            *call_count1_clone.lock().unwrap() += 1;
            10
        },
        1,
    );

    let call_count2_clone = call_count2.clone();
    let result2 = use_memo(
        move || {
            *call_count2_clone.lock().unwrap() += 1;
            20
        },
        2,
    );

    assert_eq!(result1, 10);
    assert_eq!(result2, 20);
    assert_eq!(*call_count1.lock().unwrap(), 1);
    assert_eq!(*call_count2.lock().unwrap(), 1);

    context.clear();
}

#[test]
fn test_memo_with_option() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            Some(42)
        },
        true,
    );

    assert_eq!(result, Some(42));
    assert_eq!(*call_count.lock().unwrap(), 1);

    context.clear();
}

#[test]
fn test_memo_with_result() {
    let context = setup_context();

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();

    let result = use_memo(
        move || {
            *call_count_clone.lock().unwrap() += 1;
            Ok::<i32, String>(42)
        },
        1,
    );

    assert_eq!(result, Ok(42));
    assert_eq!(*call_count.lock().unwrap(), 1);

    context.clear();
}
