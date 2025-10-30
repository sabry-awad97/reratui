use super::*;
use crate::hook_context::{HookContext, set_hook_context};
use std::rc::Rc;
use std::sync::Arc as StdArc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Helper function to set up a hook context for testing
fn setup_context() -> Rc<HookContext> {
    let context = Rc::new(HookContext::new());
    set_hook_context(context.clone());
    context
}

#[test]
fn test_ref_container_creation() {
    let container = RefContainer::new(|| 42);
    assert_eq!(container.get(), 42);
}

#[test]
fn test_ref_container_set() {
    let container = RefContainer::new(|| 0);
    assert_eq!(container.get(), 0);

    container.set(100);
    assert_eq!(container.get(), 100);
}

#[test]
fn test_ref_container_update() {
    let container = RefContainer::new(|| 10);

    container.update(|value| *value += 5);
    assert_eq!(container.get(), 15);

    container.update(|value| *value *= 2);
    assert_eq!(container.get(), 30);
}

#[test]
fn test_ref_container_with() {
    #[derive(Clone)]
    struct Data {
        count: i32,
        name: String,
    }

    let container = RefContainer::new(|| Data {
        count: 42,
        name: "test".to_string(),
    });

    let count = container.with(|data| data.count);
    assert_eq!(count, 42);

    let name = container.with(|data| data.name.clone());
    assert_eq!(name, "test");
}

#[test]
fn test_ref_container_with_mut() {
    #[derive(Clone)]
    struct Data {
        count: i32,
        name: String,
    }

    let container = RefContainer::new(|| Data {
        count: 0,
        name: String::new(),
    });

    container.with_mut(|data| {
        data.count = 100;
        data.name = "updated".to_string();
    });

    assert_eq!(container.get().count, 100);
    assert_eq!(container.get().name, "updated");
}

#[test]
fn test_ref_container_replace() {
    let container = RefContainer::new(|| 42);
    let old_value = container.replace(100);

    assert_eq!(old_value, 42);
    assert_eq!(container.get(), 100);
}

#[test]
fn test_ref_container_take() {
    let container = RefContainer::new(|| 42);
    let value = container.take();

    assert_eq!(value, 42);
    assert_eq!(container.get(), 0); // Default for i32
}

#[test]
fn test_ref_handle_creation() {
    let handle = RefHandle::new(|| "hello");
    assert_eq!(handle.get(), "hello");
}

#[test]
fn test_ref_handle_set() {
    let handle = RefHandle::new(|| 0);
    handle.set(42);
    assert_eq!(handle.get(), 42);
}

#[test]
fn test_ref_handle_update() {
    let handle = RefHandle::new(|| vec![1, 2, 3]);
    handle.update(|vec| vec.push(4));
    assert_eq!(handle.get(), vec![1, 2, 3, 4]);
}

#[test]
fn test_ref_handle_clone() {
    let handle1 = RefHandle::new(|| 42);
    let handle2 = handle1.clone();

    handle1.set(100);
    assert_eq!(handle2.get(), 100); // Both point to same container
}

#[test]
fn test_use_ref_basic() {
    let context = setup_context();

    let counter = use_ref(|| 0);
    assert_eq!(counter.get(), 0);

    counter.set(42);
    assert_eq!(counter.get(), 42);

    // Reset for next test
    context.clear();
}

#[test]
fn test_use_ref_persistence_across_renders() {
    let context = setup_context();

    // First render
    let counter = use_ref(|| 0);
    counter.set(10);
    assert_eq!(counter.get(), 10);

    // Simulate re-render by resetting hook index
    context.reset_hook_index();

    // Second render - should get the same ref
    let counter = use_ref(|| 0);
    assert_eq!(counter.get(), 10); // Value persists!

    context.clear();
}

#[test]
fn test_use_ref_multiple_refs() {
    let context = setup_context();

    let ref1 = use_ref(|| 1);
    let ref2 = use_ref(|| 2);
    let ref3 = use_ref(|| 3);

    assert_eq!(ref1.get(), 1);
    assert_eq!(ref2.get(), 2);
    assert_eq!(ref3.get(), 3);

    ref1.set(10);
    ref2.set(20);
    ref3.set(30);

    assert_eq!(ref1.get(), 10);
    assert_eq!(ref2.get(), 20);
    assert_eq!(ref3.get(), 30);

    context.clear();
}

#[test]
fn test_use_ref_complex_type() {
    let context = setup_context();

    #[derive(Clone, Debug, PartialEq)]
    struct Config {
        count: i32,
        name: String,
        enabled: bool,
    }

    let config_ref = use_ref(|| Config {
        count: 0,
        name: "initial".to_string(),
        enabled: false,
    });

    config_ref.with_mut(|cfg| {
        cfg.count = 42;
        cfg.name = "updated".to_string();
        cfg.enabled = true;
    });

    let config = config_ref.get();
    assert_eq!(config.count, 42);
    assert_eq!(config.name, "updated");
    assert!(config.enabled);

    context.clear();
}

#[test]
fn test_use_ref_with_option() {
    let context = setup_context();

    let maybe_value = use_ref(|| Option::<i32>::None);
    assert_eq!(maybe_value.get(), None);

    maybe_value.set(Some(42));
    assert_eq!(maybe_value.get(), Some(42));

    let value = maybe_value.take();
    assert_eq!(value, Some(42));
    assert_eq!(maybe_value.get(), None);

    context.clear();
}

#[test]
fn test_use_ref_tracking_previous_value() {
    let context = setup_context();

    let current = use_ref(|| 10);
    let previous = use_ref(|| 0);

    // Simulate state change
    let old_value = previous.replace(current.get());
    current.set(20);

    assert_eq!(old_value, 0); // First time, previous was 0
    assert_eq!(previous.get(), 10); // Now stores old current
    assert_eq!(current.get(), 20); // Current is updated

    context.clear();
}

#[test]
fn test_use_ref_render_counter() {
    let context = setup_context();

    let render_count = use_ref(|| 0);

    // Simulate multiple renders
    for i in 1..=5 {
        render_count.update(|count| *count += 1);
        assert_eq!(render_count.get(), i);
        context.reset_hook_index();
    }

    context.clear();
}

#[test]
fn test_use_ref_thread_safety() {
    let context = setup_context();

    let counter = use_ref(|| StdArc::new(AtomicUsize::new(0)));

    // Clone the ref handle for use in another thread
    let counter_clone = counter.clone();

    // Simulate concurrent access
    let handle = std::thread::spawn(move || {
        counter_clone.with(|arc| {
            arc.fetch_add(1, Ordering::SeqCst);
        });
    });

    handle.join().unwrap();

    let value = counter.with(|arc| arc.load(Ordering::SeqCst));
    assert_eq!(value, 1);

    context.clear();
}

#[test]
fn test_use_ref_with_vec_operations() {
    let context = setup_context();

    let items = use_ref(Vec::<String>::new);

    items.update(|vec| vec.push("first".to_string()));
    items.update(|vec| vec.push("second".to_string()));
    items.update(|vec| vec.push("third".to_string()));

    assert_eq!(items.get().len(), 3);
    assert_eq!(items.get()[0], "first");
    assert_eq!(items.get()[2], "third");

    items.with_mut(|vec| {
        vec.retain(|s| s != "second");
    });

    assert_eq!(items.get().len(), 2);
    assert_eq!(items.get(), vec!["first", "third"]);

    context.clear();
}

#[test]
fn test_use_ref_caching_pattern() {
    let context = setup_context();

    #[derive(Clone)]
    struct Cache {
        value: Option<String>,
        computed: bool,
    }

    let cache = use_ref(|| Cache {
        value: None,
        computed: false,
    });

    // First access - compute
    let result1 = cache.with_mut(|c| {
        if !c.computed {
            c.value = Some("expensive result".to_string());
            c.computed = true;
        }
        c.value.clone().unwrap()
    });

    assert_eq!(result1, "expensive result");

    // Second access - use cached value
    let result2 = cache.with_mut(|c| {
        if !c.computed {
            panic!("Should not recompute!");
        }
        c.value.clone().unwrap()
    });

    assert_eq!(result2, "expensive result");

    context.clear();
}

#[test]
fn test_use_ref_timer_handle() {
    let context = setup_context();

    use std::time::Instant;

    let start_time = use_ref(Instant::now);

    // Simulate some work
    std::thread::sleep(std::time::Duration::from_millis(10));

    let elapsed = start_time.with(|time| time.elapsed());
    assert!(elapsed.as_millis() >= 10);

    context.clear();
}

#[test]
fn test_use_ref_does_not_trigger_rerender() {
    let context = setup_context();

    // This is a conceptual test - in a real component,
    // mutating a ref should NOT trigger a re-render

    // First render - create the ref
    let counter = use_ref(|| 0);

    // Mutate the ref multiple times
    counter.set(1);
    counter.set(2);
    counter.set(3);

    // Verify the value changed
    assert_eq!(counter.get(), 3);

    // Simulate a new render cycle
    context.reset_hook_index();

    // Second render - get the same ref
    let counter = use_ref(|| 0);

    // The value should persist (proving it's the same ref)
    assert_eq!(counter.get(), 3);

    // The key point: mutations don't trigger re-renders
    // (unlike use_state which would trigger re-renders)

    context.clear();
}

#[test]
fn test_use_ref_with_custom_type() {
    let context = setup_context();

    #[derive(Clone, Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }

    impl Point {
        fn distance_from_origin(&self) -> f64 {
            (self.x * self.x + self.y * self.y).sqrt()
        }
    }

    let point_ref = use_ref(|| Point { x: 0.0, y: 0.0 });

    point_ref.with_mut(|p| {
        p.x = 3.0;
        p.y = 4.0;
    });

    let distance = point_ref.with(|p| p.distance_from_origin());
    assert_eq!(distance, 5.0);

    context.clear();
}

#[test]
fn test_use_ref_field_access() {
    let context = setup_context();

    #[derive(Clone)]
    struct User {
        id: u64,
        name: String,
        #[allow(dead_code)]
        email: String,
    }

    let user_ref = use_ref(|| User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    });

    // Access specific fields without cloning entire struct
    let id = user_ref.with(|u| u.id);
    let name = user_ref.with(|u| u.name.clone());

    assert_eq!(id, 1);
    assert_eq!(name, "Alice");

    context.clear();
}

#[test]
fn test_use_ref_swap_pattern() {
    let context = setup_context();

    let ref1 = use_ref(|| "first");
    let ref2 = use_ref(|| "second");

    let temp = ref1.replace(ref2.get());
    ref2.set(temp);

    assert_eq!(ref1.get(), "second");
    assert_eq!(ref2.get(), "first");

    context.clear();
}
