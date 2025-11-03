//! Tests for the resize hook

use super::*;
use crate::{
    callback::use_callback,
    event::set_current_event,
    test_utils::{with_component_id, with_test_isolate},
};
use crossterm::event::Event;
use parking_lot::Mutex;
use std::sync::{Arc, LazyLock};

// Test mutex to prevent parallel test execution from interfering with global event state
static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

// Note: These tests verify that the hook correctly detects resize events.
// The hook uses use_event() internally which tracks event processing per hook index.

#[test]
fn test_use_on_resize_basic() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let size = Arc::new(Mutex::new((0u16, 0u16)));

        let called_clone = called.clone();
        let size_clone = size.clone();

        // Set a resize event
        set_current_event(Some(Arc::new(Event::Resize(100, 50))));

        // First render with the hook
        with_component_id("ResizeTest", |_ctx| {
            use_on_resize(move |(width, height)| {
                *called_clone.lock() = true;
                *size_clone.lock() = (width, height);
            });
        });

        // Verify callback was called with correct dimensions
        assert!(*called.lock(), "Callback should have been called");
        assert_eq!(*size.lock(), (100, 50), "Size should be (100, 50)");
    });
}

#[test]
fn test_use_on_resize_no_event() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // Clear any previous events
        set_current_event(None);

        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Use the hook
        with_component_id("ResizeNoEventTest", |_ctx| {
            use_on_resize(move |_| {
                *called_clone.lock() = true;
            });
        });

        // Verify callback was NOT called
        assert!(
            !*called.lock(),
            "Callback should not be called without resize event"
        );
    });
}

#[test]
fn test_use_on_resize_ignores_other_events() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Set a key event (not resize)
        set_current_event(Some(Arc::new(Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('a'),
            crossterm::event::KeyModifiers::NONE,
        )))));

        // Use the hook
        with_component_id("ResizeIgnoreOtherTest", |_ctx| {
            use_on_resize(move |_| {
                *called_clone.lock() = true;
            });
        });

        // Verify callback was NOT called
        assert!(
            !*called.lock(),
            "Callback should not be called for non-resize events"
        );
    });
}

#[test]
fn test_use_on_resize_multiple_calls() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let call_count = Arc::new(Mutex::new(0));
        let sizes = Arc::new(Mutex::new(Vec::new()));

        let count_clone = call_count.clone();
        let sizes_clone = sizes.clone();

        // First resize
        set_current_event(Some(Arc::new(Event::Resize(80, 24))));

        with_component_id("ResizeMultipleTest", |_ctx| {
            use_on_resize({
                let count = count_clone.clone();
                let sizes = sizes_clone.clone();
                move |(w, h)| {
                    *count.lock() += 1;
                    sizes.lock().push((w, h));
                }
            });
        });

        assert_eq!(*call_count.lock(), 1);
        assert_eq!(sizes.lock()[0], (80, 24));
    });
}

#[test]
fn test_use_on_resize_with_state() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // Set a resize event
        set_current_event(Some(Arc::new(Event::Resize(120, 40))));

        with_component_id("ResizeWithStateTest", |_ctx| {
            let (size, set_size) = use_state(|| (0u16, 0u16));

            // Use the hook with state update
            use_on_resize({
                let set_size = set_size.clone();
                move |(width, height)| {
                    set_size.set((width, height));
                }
            });

            // Verify state was updated
            assert_eq!(
                size.get(),
                (120, 40),
                "State should be updated to (120, 40)"
            );
        });
    });
}

#[test]
fn test_use_on_resize_callback() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let called = Arc::new(Mutex::new(false));
        let size = Arc::new(Mutex::new((0u16, 0u16)));

        let called_clone = called.clone();
        let size_clone = size.clone();

        // Set a resize event
        set_current_event(Some(Arc::new(Event::Resize(150, 60))));

        with_component_id("ResizeCallbackTest", |_ctx| {
            // Create a memoized callback
            let callback = use_callback(
                move |(width, height): (u16, u16)| {
                    *called_clone.lock() = true;
                    *size_clone.lock() = (width, height);
                },
                (),
            );

            // Use the hook with memoized callback
            use_on_resize_callback(callback);
        });

        // Verify callback was called
        assert!(*called.lock(), "Callback should have been called");
        assert_eq!(*size.lock(), (150, 60), "Size should be (150, 60)");
    });
}

#[test]
fn test_use_on_resize_zero_dimensions() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let size = Arc::new(Mutex::new((100u16, 100u16)));
        let size_clone = size.clone();

        // Set a resize event with zero dimensions
        set_current_event(Some(Arc::new(Event::Resize(0, 0))));

        with_component_id("ResizeZeroTest", |_ctx| {
            use_on_resize(move |(width, height)| {
                *size_clone.lock() = (width, height);
            });
        });

        // Verify callback was called with zero dimensions
        assert_eq!(*size.lock(), (0, 0), "Should handle zero dimensions");
    });
}

#[test]
fn test_use_on_resize_large_dimensions() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let size = Arc::new(Mutex::new((0u16, 0u16)));
        let size_clone = size.clone();

        // Set a resize event with large dimensions
        set_current_event(Some(Arc::new(Event::Resize(u16::MAX, u16::MAX))));

        with_component_id("ResizeLargeTest", |_ctx| {
            use_on_resize(move |(width, height)| {
                *size_clone.lock() = (width, height);
            });
        });

        // Verify callback was called with large dimensions
        assert_eq!(
            *size.lock(),
            (u16::MAX, u16::MAX),
            "Should handle maximum dimensions"
        );
    });
}

#[test]
fn test_use_on_resize_persistence_across_renders() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        let call_count = Arc::new(Mutex::new(0));
        let count_clone = call_count.clone();

        // First render with resize event
        set_current_event(Some(Arc::new(Event::Resize(80, 24))));

        with_component_id("ResizePersistenceTest", |_ctx| {
            use_on_resize({
                let count = count_clone.clone();
                move |_| {
                    *count.lock() += 1;
                }
            });
        });

        assert_eq!(*call_count.lock(), 1, "Should be called once");

        // Second render without event (simulating re-render)
        set_current_event(None);

        with_component_id("ResizePersistenceTest", |_ctx| {
            use_on_resize({
                let count = count_clone.clone();
                move |_| {
                    *count.lock() += 1;
                }
            });
        });

        // Count should still be 1 (not called again)
        assert_eq!(
            *call_count.lock(),
            1,
            "Should not be called on re-render without event"
        );
    });
}

#[test]
fn test_use_terminal_dimensions() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // First render - no event
        set_current_event(None);

        with_component_id("TerminalDimensionsTest", |_ctx| {
            let (width, height) = use_terminal_dimensions();
            assert_eq!(
                (width, height),
                (0, 0),
                "Initial dimensions should be (0, 0)"
            );
        });

        // Set a resize event
        set_current_event(Some(Arc::new(Event::Resize(120, 40))));

        // Second render - with event
        with_component_id("TerminalDimensionsTest", |_ctx| {
            let (width, height) = use_terminal_dimensions();
            assert_eq!(
                (width, height),
                (120, 40),
                "Dimensions should be updated to (120, 40)"
            );
        });
    });
}

#[test]
fn test_use_terminal_dimensions_responsive() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // Test narrow terminal
        set_current_event(Some(Arc::new(Event::Resize(60, 20))));

        with_component_id("TerminalDimensionsResponsiveTest", |_ctx| {
            let (width, _) = use_terminal_dimensions();
            assert!(width < 80, "Should detect narrow terminal");
        });

        // Test wide terminal
        set_current_event(Some(Arc::new(Event::Resize(120, 40))));

        with_component_id("TerminalDimensionsResponsiveTest", |_ctx| {
            let (width, _) = use_terminal_dimensions();
            assert!(width >= 80, "Should detect wide terminal");
        });
    });
}

#[test]
fn test_use_terminal_dimensions_updates() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // First size
        set_current_event(Some(Arc::new(Event::Resize(80, 24))));

        with_component_id("TerminalDimensionsUpdatesTest", |_ctx| {
            let (w1, h1) = use_terminal_dimensions();
            assert_eq!((w1, h1), (80, 24));
        });

        // Second size
        set_current_event(Some(Arc::new(Event::Resize(100, 30))));

        with_component_id("TerminalDimensionsUpdatesTest", |_ctx| {
            let (w2, h2) = use_terminal_dimensions();
            assert_eq!((w2, h2), (100, 30));
        });
    });
}

#[test]
fn test_use_media_query_basic() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // No event initially
        set_current_event(None);

        with_component_id("MediaQueryBasicTest", |_ctx| {
            let is_narrow = use_media_query(|(width, _)| width < 80);
            assert!(!is_narrow, "Should be false initially (0x0)");
        });

        // Set narrow terminal
        set_current_event(Some(Arc::new(Event::Resize(60, 24))));

        with_component_id("MediaQueryBasicTest", |_ctx| {
            let is_narrow = use_media_query(|(width, _)| width < 80);
            assert!(is_narrow, "Should match narrow terminal");
        });

        // Set wide terminal
        set_current_event(Some(Arc::new(Event::Resize(120, 40))));

        with_component_id("MediaQueryBasicTest", |_ctx| {
            let is_narrow = use_media_query(|(width, _)| width < 80);
            assert!(!is_narrow, "Should not match wide terminal");
        });
    });
}

#[test]
fn test_use_media_query_height() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        set_current_event(Some(Arc::new(Event::Resize(80, 20))));

        with_component_id("MediaQueryHeightTest", |_ctx| {
            let is_short = use_media_query(|(_, height)| height < 24);
            assert!(is_short, "Should detect short terminal");
        });

        set_current_event(Some(Arc::new(Event::Resize(80, 40))));

        with_component_id("MediaQueryHeightTest", |_ctx| {
            let is_short = use_media_query(|(_, height)| height < 24);
            assert!(!is_short, "Should not match tall terminal");
        });
    });
}

#[test]
fn test_use_media_query_combined() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // Small terminal
        set_current_event(Some(Arc::new(Event::Resize(60, 20))));

        with_component_id("MediaQueryCombinedTest", |_ctx| {
            let is_small = use_media_query(|(width, height)| width < 80 && height < 24);
            assert!(is_small, "Should match small terminal");
        });

        // Large terminal
        set_current_event(Some(Arc::new(Event::Resize(120, 40))));

        with_component_id("MediaQueryCombinedTest", |_ctx| {
            let is_small = use_media_query(|(width, height)| width < 80 && height < 24);
            assert!(!is_small, "Should not match large terminal");
        });
    });
}

#[test]
fn test_use_media_query_breakpoints() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // Extra small
        set_current_event(Some(Arc::new(Event::Resize(30, 20))));

        with_component_id("MediaQueryBreakpointsTest", |_ctx| {
            let is_xs = use_media_query(|(w, _)| w < 40);
            let is_sm = use_media_query(|(w, _)| (40..80).contains(&w));
            let is_md = use_media_query(|(w, _)| (80..120).contains(&w));

            assert!(is_xs, "Should match xs breakpoint");
            assert!(!is_sm, "Should not match sm breakpoint");
            assert!(!is_md, "Should not match md breakpoint");
        });

        // Small
        set_current_event(Some(Arc::new(Event::Resize(60, 24))));

        with_component_id("MediaQueryBreakpointsTest", |_ctx| {
            let is_xs = use_media_query(|(w, _)| w < 40);
            let is_sm = use_media_query(|(w, _)| (40..80).contains(&w));
            let is_md = use_media_query(|(w, _)| (80..120).contains(&w));

            assert!(!is_xs, "Should not match xs breakpoint");
            assert!(is_sm, "Should match sm breakpoint");
            assert!(!is_md, "Should not match md breakpoint");
        });

        // Medium
        set_current_event(Some(Arc::new(Event::Resize(100, 30))));

        with_component_id("MediaQueryBreakpointsTest", |_ctx| {
            let is_xs = use_media_query(|(w, _)| w < 40);
            let is_sm = use_media_query(|(w, _)| (40..80).contains(&w));
            let is_md = use_media_query(|(w, _)| (80..120).contains(&w));

            assert!(!is_xs, "Should not match xs breakpoint");
            assert!(!is_sm, "Should not match sm breakpoint");
            assert!(is_md, "Should match md breakpoint");
        });
    });
}

#[test]
fn test_use_media_query_aspect_ratio() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // Wide aspect ratio (3:1)
        set_current_event(Some(Arc::new(Event::Resize(120, 40))));

        with_component_id("MediaQueryAspectTest", |_ctx| {
            let is_wide = use_media_query(|(width, height)| {
                let ratio = width as f32 / height as f32;
                ratio > 2.5
            });
            assert!(is_wide, "Should detect wide aspect ratio");
        });

        // Normal aspect ratio (2:1)
        set_current_event(Some(Arc::new(Event::Resize(80, 40))));

        with_component_id("MediaQueryAspectTest", |_ctx| {
            let is_wide = use_media_query(|(width, height)| {
                let ratio = width as f32 / height as f32;
                ratio > 2.5
            });
            assert!(!is_wide, "Should not match normal aspect ratio");
        });
    });
}

#[test]
fn test_use_media_query_multiple_queries() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        set_current_event(Some(Arc::new(Event::Resize(100, 30))));

        with_component_id("MediaQueryMultipleTest", |_ctx| {
            let is_mobile = use_media_query(|(w, _)| w < 60);
            let is_tablet = use_media_query(|(w, _)| (60..120).contains(&w));
            let is_desktop = use_media_query(|(w, _)| w >= 120);

            assert!(!is_mobile, "Should not be mobile");
            assert!(is_tablet, "Should be tablet");
            assert!(!is_desktop, "Should not be desktop");
        });
    });
}

#[test]
fn test_use_media_query_updates_on_resize() {
    let _lock = TEST_MUTEX.lock();
    with_test_isolate(|| {
        // Start with narrow
        set_current_event(Some(Arc::new(Event::Resize(60, 24))));

        with_component_id("MediaQueryUpdatesTest", |_ctx| {
            let is_narrow = use_media_query(|(w, _)| w < 80);
            assert!(is_narrow, "Should start narrow");
        });

        // Resize to wide
        set_current_event(Some(Arc::new(Event::Resize(120, 40))));

        with_component_id("MediaQueryUpdatesTest", |_ctx| {
            let is_narrow = use_media_query(|(w, _)| w < 80);
            assert!(!is_narrow, "Should update to wide");
        });
    });
}
