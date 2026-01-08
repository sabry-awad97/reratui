//! Integration tests for ComponentV2 trait and fiber integration.
//!
//! These tests verify that ComponentV2 components work correctly within
//! the render_v2 context with proper hook isolation and lifecycle management.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use reratui_fiber::element::Element;
use reratui_fiber::prelude::*;
use std::sync::Arc;

/// A simple counter component for testing - no Clone needed!
struct CounterComponent {
    render_count: Arc<std::sync::Mutex<u32>>,
}

impl ComponentV2 for CounterComponent {
    fn render(&self, _area: Rect, _buffer: &mut Buffer) {
        // Increment render count
        let mut count = self.render_count.lock().unwrap();
        *count += 1;

        // Use hooks to verify they work
        let (_state, _set_state) = use_state_v2(|| 0i32);
    }
}

/// Test that ComponentV2 can be rendered within a fiber context
#[test]
fn test_component_v2_within_fiber_context() {
    use reratui_fiber::context_stack::clear_context_stack;
    use reratui_fiber::fiber_tree::{clear_fiber_tree, set_fiber_tree};

    // Setup fiber tree
    let mut tree = FiberTree::new();
    let root_fiber = tree.mount(None, None);
    tree.begin_render(root_fiber);
    set_fiber_tree(tree);

    let render_count = Arc::new(std::sync::Mutex::new(0));
    let component = CounterComponent {
        render_count: render_count.clone(),
    };

    let area = Rect::new(0, 0, 80, 24);
    let mut buffer = Buffer::empty(area);

    // Render the component using Element::component
    let element = Element::component(component);
    element.render(area, &mut buffer);

    // Verify render was called
    assert_eq!(*render_count.lock().unwrap(), 1);

    // Cleanup
    clear_fiber_tree();
    clear_context_stack();
}

/// Test that hooks work correctly within ComponentV2
#[test]
fn test_component_v2_hooks_work() {
    use reratui_fiber::context_stack::clear_context_stack;
    use reratui_fiber::fiber_tree::{clear_fiber_tree, set_fiber_tree};

    // Setup fiber tree
    let mut tree = FiberTree::new();
    let root_fiber = tree.mount(None, None);
    tree.begin_render(root_fiber);
    set_fiber_tree(tree);

    struct HookTestComponent {
        state_value: Arc<std::sync::Mutex<Option<i32>>>,
    }

    impl ComponentV2 for HookTestComponent {
        fn render(&self, _area: Rect, _buffer: &mut Buffer) {
            let (state, _set_state) = use_state_v2(|| 42i32);
            *self.state_value.lock().unwrap() = Some(state);
        }
    }

    let state_value = Arc::new(std::sync::Mutex::new(None));
    let component = HookTestComponent {
        state_value: state_value.clone(),
    };

    let area = Rect::new(0, 0, 80, 24);
    let mut buffer = Buffer::empty(area);

    Element::component(component).render(area, &mut buffer);

    // Verify state hook worked
    assert_eq!(*state_value.lock().unwrap(), Some(42));

    // Cleanup
    clear_fiber_tree();
    clear_context_stack();
}

/// Test that fiber isolation works between different ComponentV2 instances
#[test]
fn test_component_v2_fiber_isolation() {
    use reratui_fiber::context_stack::clear_context_stack;
    use reratui_fiber::fiber_tree::{clear_fiber_tree, set_fiber_tree};

    // Setup fiber tree
    let mut tree = FiberTree::new();
    let root_fiber = tree.mount(None, None);
    tree.begin_render(root_fiber);
    set_fiber_tree(tree);

    struct IsolationTestComponent {
        initial_value: i32,
        observed_value: Arc<std::sync::Mutex<Option<i32>>>,
    }

    impl ComponentV2 for IsolationTestComponent {
        fn render(&self, _area: Rect, _buffer: &mut Buffer) {
            let initial = self.initial_value;
            let (state, _set_state) = use_state_v2(move || initial);
            *self.observed_value.lock().unwrap() = Some(state);
        }
    }

    let value1 = Arc::new(std::sync::Mutex::new(None));
    let value2 = Arc::new(std::sync::Mutex::new(None));

    let component1 = IsolationTestComponent {
        initial_value: 100,
        observed_value: value1.clone(),
    };

    let component2 = IsolationTestComponent {
        initial_value: 200,
        observed_value: value2.clone(),
    };

    let area = Rect::new(0, 0, 80, 24);
    let mut buffer = Buffer::empty(area);

    // Render both components
    Element::component(component1).render(area, &mut buffer);
    Element::component(component2).render(area, &mut buffer);

    // Verify each component has its own isolated state
    assert_eq!(*value1.lock().unwrap(), Some(100));
    assert_eq!(*value2.lock().unwrap(), Some(200));

    // Cleanup
    clear_fiber_tree();
    clear_context_stack();
}

/// Test nested ComponentV2 components
#[test]
fn test_nested_component_v2() {
    use reratui_fiber::context_stack::clear_context_stack;
    use reratui_fiber::fiber_tree::{clear_fiber_tree, set_fiber_tree};

    // Setup fiber tree
    let mut tree = FiberTree::new();
    let root_fiber = tree.mount(None, None);
    tree.begin_render(root_fiber);
    set_fiber_tree(tree);

    struct ChildComponent {
        render_called: Arc<std::sync::atomic::AtomicBool>,
    }

    impl ComponentV2 for ChildComponent {
        fn render(&self, _area: Rect, _buffer: &mut Buffer) {
            self.render_called
                .store(true, std::sync::atomic::Ordering::SeqCst);

            // Verify we can access ComponentArea context from parent
            let area_ctx = try_use_context_v2::<ComponentArea>();
            assert!(
                area_ctx.is_some(),
                "Child should have access to ComponentArea"
            );
        }
    }

    // Parent component that renders a child
    struct ParentComponent {
        child_render_called: Arc<std::sync::atomic::AtomicBool>,
    }

    impl ComponentV2 for ParentComponent {
        fn render(&self, area: Rect, buffer: &mut Buffer) {
            // Parent renders child using Element::component
            let child = ChildComponent {
                render_called: self.child_render_called.clone(),
            };
            Element::component(child).render(area, buffer);
        }
    }

    let child_render_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let parent = ParentComponent {
        child_render_called: child_render_called.clone(),
    };

    let area = Rect::new(0, 0, 80, 24);
    let mut buffer = Buffer::empty(area);

    Element::component(parent).render(area, &mut buffer);

    // Verify child was rendered
    assert!(
        child_render_called.load(std::sync::atomic::Ordering::SeqCst),
        "Child component should have been rendered"
    );

    // Cleanup
    clear_fiber_tree();
    clear_context_stack();
}

/// Test ComponentArea context is correctly provided
#[test]
fn test_component_area_context() {
    use reratui_fiber::context_stack::clear_context_stack;
    use reratui_fiber::fiber_tree::{clear_fiber_tree, set_fiber_tree};

    // Setup fiber tree
    let mut tree = FiberTree::new();
    let root_fiber = tree.mount(None, None);
    tree.begin_render(root_fiber);
    set_fiber_tree(tree);

    struct AreaTestComponent {
        received_area: Arc<std::sync::Mutex<Option<Rect>>>,
    }

    impl ComponentV2 for AreaTestComponent {
        fn render(&self, area: Rect, _buffer: &mut Buffer) {
            // Get ComponentArea from context
            let area_ctx = try_use_context_v2::<ComponentArea>();
            if let Some(ctx) = area_ctx {
                *self.received_area.lock().unwrap() = Some(ctx.area());
                // Verify context area matches render area
                assert_eq!(ctx.area(), area);
            }
        }
    }

    let received_area = Arc::new(std::sync::Mutex::new(None));
    let component = AreaTestComponent {
        received_area: received_area.clone(),
    };

    let area = Rect::new(10, 20, 100, 50);
    let mut buffer = Buffer::empty(area);

    Element::component(component).render(area, &mut buffer);

    // Verify area was received
    let received = received_area.lock().unwrap();
    assert!(received.is_some());
    assert_eq!(received.unwrap(), area);

    // Cleanup
    clear_fiber_tree();
    clear_context_stack();
}
