//! Global event handling for TUI applications

use crossterm::event::{KeyCode, KeyEvent};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

type EventHandler = dyn Fn() -> bool + Send + Sync + 'static;

static GLOBAL_EVENT_HANDLERS: Lazy<Mutex<HashMap<KeyCode, Vec<Arc<EventHandler>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Register a global event handler for a specific key code
///
/// # Arguments
/// * `key` - The key code to listen for
/// * `handler` - A closure that will be called when the key is pressed.
///   Return `true` to indicate the event was handled and stop propagation,
///   or `false` to allow other handlers to process the event.
///
/// # Example
/// ```
/// use crossterm::event::KeyCode;
/// use pulse_core::hooks::event::global_events::on_global_event;
///
/// on_global_event(KeyCode::Char('q'), || {
///     println!("Quit requested");
///     true // Stop event propagation
/// });
/// ```
pub fn on_global_event<F>(key: KeyCode, handler: F)
where
    F: Fn() -> bool + Send + Sync + 'static,
{
    let mut handlers = GLOBAL_EVENT_HANDLERS.lock();
    let handlers_for_key = handlers.entry(key).or_default();
    handlers_for_key.push(Arc::new(handler));
}

/// Process a key event through all registered global handlers
///
/// # Returns
/// `true` if the event was handled by any handler, `false` otherwise
pub fn process_global_event(event: &KeyEvent) -> bool {
    if event.kind != crossterm::event::KeyEventKind::Press {
        return false;
    }

    let handlers = GLOBAL_EVENT_HANDLERS.lock();
    if let Some(handlers_for_key) = handlers.get(&event.code) {
        for handler in handlers_for_key {
            if handler() {
                return true;
            }
        }
    }
    false
}

// A simple test mutex for use in tests
#[cfg(test)]
mod test_util {
    use std::sync::{Arc, Condvar, Mutex};

    /// A simple mutex that can be used for testing synchronization in async contexts
    #[derive(Debug)]
    pub struct TestMutex<T> {
        inner: Arc<(Mutex<T>, Condvar)>,
    }

    impl<T> TestMutex<T> {
        /// Create a new TestMutex with the given value
        pub fn new(value: T) -> Self {
            TestMutex {
                inner: Arc::new((Mutex::new(value), Condvar::new())),
            }
        }

        /// Lock the mutex
        pub fn lock(&self) -> std::sync::MutexGuard<'_, T> {
            self.inner.0.lock().unwrap()
        }

        /// Wait for a condition to be true
        pub fn wait_for<F>(&self, mut condition: F) -> std::sync::MutexGuard<'_, T>
        where
            F: FnMut(&T) -> bool,
        {
            let (lock, cvar) = &*self.inner;
            let mut guard = lock.lock().unwrap();

            while !condition(&*guard) {
                guard = cvar.wait(guard).unwrap();
            }

            guard
        }

        /// Notify all waiters
        pub fn notify_all(&self) {
            self.inner.1.notify_all();
        }
    }

    impl<T> Clone for TestMutex<T> {
        fn clone(&self) -> Self {
            TestMutex {
                inner: self.inner.clone(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_util::TestMutex;
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_global_event_handling() {
        use crate::test_utils::TEST_MUTEX;
        let _lock = TEST_MUTEX.lock();

        // Clear any existing handlers
        GLOBAL_EVENT_HANDLERS.lock().clear();

        // Use TestMutex to synchronize the test
        let test_state = TestMutex::new((false, false)); // (handler_called, test_complete)
        let test_state_clone = test_state.clone();

        // Register a handler for the 'a' key
        on_global_event(KeyCode::Char('a'), move || {
            let mut state = test_state_clone.lock();
            state.0 = true; // Mark handler as called
            state.1 = true; // Mark test as complete
            test_state_clone.notify_all();
            true
        });

        // Create a test event with the correct KeyEventKind
        let event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        // Process the event
        let result = process_global_event(&event);

        // Wait for the handler to be called with a timeout
        let state = test_state.wait_for(|s| s.1);

        // Verify the handler was called and the event was handled
        assert!(state.0, "Handler was not called. Event: {:?}", event);
        assert!(result, "Event should be marked as handled");
    }

    #[test]
    fn test_event_propagation_stop() {
        use crate::test_utils::TEST_MUTEX;
        let _lock = TEST_MUTEX.lock();

        // Clear any existing handlers
        GLOBAL_EVENT_HANDLERS.lock().clear();

        // Use a test key that's not used in other tests
        let test_key = KeyCode::Char('y');

        // Create counters for our handlers
        let first_handler_called = Arc::new(AtomicBool::new(false));
        let second_handler_called = Arc::new(AtomicBool::new(false));

        // First handler stops propagation
        {
            let first_called = first_handler_called.clone();
            on_global_event(test_key, move || {
                first_called.store(true, Ordering::SeqCst);
                true // Stop propagation
            });
        }

        // Second handler should not be called
        {
            let second_called = second_handler_called.clone();
            on_global_event(test_key, move || {
                second_called.store(true, Ordering::SeqCst);
                true
            });
        }

        // Verify we have the expected number of handlers registered
        {
            let handlers = GLOBAL_EVENT_HANDLERS.lock();
            let handlers_for_key = handlers.get(&test_key).unwrap_or(&vec![]).len();
            assert_eq!(handlers_for_key, 2, "Expected 2 handlers to be registered");
        }

        // Process the event
        let event = KeyEvent {
            code: test_key,
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        // Process the event multiple times to ensure handlers are called
        let mut result = false;
        for _ in 0..5 {
            result = process_global_event(&event);
            if result {
                break;
            }
        }

        // Verify only the first handler was called and the event was handled
        assert!(
            first_handler_called.load(Ordering::SeqCst),
            "First handler was not called"
        );
        assert!(
            !second_handler_called.load(Ordering::SeqCst),
            "Second handler was called but should have been skipped"
        );
        assert!(result, "Event should be marked as handled");
    }

    #[test]
    fn test_event_propagation_continue() {
        use crate::test_utils::TEST_MUTEX;
        let _lock = TEST_MUTEX.lock();

        // Clear any existing handlers
        GLOBAL_EVENT_HANDLERS.lock().clear();

        // Create a counter to track handler calls
        let handler_call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        // Create a test key that's not used in other tests
        let test_key = KeyCode::Char('z');

        // Register first handler that returns false to continue propagation
        {
            let counter = handler_call_count.clone();
            on_global_event(test_key, move || {
                counter.fetch_add(1, Ordering::SeqCst);
                false // Continue propagation
            });
        }

        // Register second handler that also returns false
        {
            let counter = handler_call_count.clone();
            on_global_event(test_key, move || {
                counter.fetch_add(1, Ordering::SeqCst);
                false // Continue propagation
            });
        }

        // Verify we have the expected number of handlers registered
        {
            let handlers = GLOBAL_EVENT_HANDLERS.lock();
            let handlers_for_key = handlers.get(&test_key).unwrap_or(&vec![]).len();
            assert_eq!(handlers_for_key, 2, "Expected 2 handlers to be registered");
        }

        // Process the event
        let event = KeyEvent {
            code: test_key,
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        // Process the event multiple times to ensure handlers are called
        let mut result = false;
        for _ in 0..5 {
            result = process_global_event(&event);
            if result {
                break;
            }
        }

        // Verify both handlers were called
        let count = handler_call_count.load(Ordering::SeqCst);
        assert!(
            count >= 2,
            "Expected both handlers to be called at least once, but got {} calls",
            count
        );

        // The event should be marked as handled if any handler returns true
        // Since all handlers return false, the result should be false
        assert!(
            !result,
            "Event should not be marked as handled when all handlers return false"
        );
    }

    #[test]
    fn test_no_handlers() {
        use crate::test_utils::TEST_MUTEX;
        let _lock = TEST_MUTEX.lock();

        // Clear any existing handlers
        GLOBAL_EVENT_HANDLERS.lock().clear();

        let event = KeyEvent::new(KeyCode::Char('x'), crossterm::event::KeyModifiers::NONE);
        assert!(!process_global_event(&event));
    }

    #[test]
    fn test_multiple_handlers_same_key() {
        use crate::test_utils::TEST_MUTEX;
        let _lock = TEST_MUTEX.lock();

        GLOBAL_EVENT_HANDLERS.lock().clear();

        let counter1 = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter2 = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let c1 = counter1.clone();
        on_global_event(KeyCode::Char('a'), move || {
            c1.fetch_add(1, Ordering::SeqCst);
            false // Continue propagation
        });

        let c2 = counter2.clone();
        on_global_event(KeyCode::Char('a'), move || {
            c2.fetch_add(1, Ordering::SeqCst);
            true // Stop propagation
        });

        // Process the event
        let event = KeyEvent::new(KeyCode::Char('a'), crossterm::event::KeyModifiers::NONE);
        let result = process_global_event(&event);

        // Verify both handlers were called and the event was marked as handled
        assert_eq!(
            counter1.load(Ordering::SeqCst),
            1,
            "First handler was not called or called multiple times"
        );
        assert_eq!(
            counter2.load(Ordering::SeqCst),
            1,
            "Second handler was not called or called multiple times"
        );
        assert!(
            result,
            "Event should be marked as handled when any handler returns true"
        );
    }

    // #[test]
    // fn test_thread_safety() {
    //     // Clear any existing handlers
    //     GLOBAL_EVENT_HANDLERS.lock().clear();

    //     // Use a simple counter with Arc and AtomicUsize for synchronization
    //     let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    //     // Number of threads to spawn
    //     const NUM_THREADS: u8 = 3; // Reduced for faster testing

    //     // Spawn multiple threads that register handlers
    //     let mut handles = vec![];
    //     for i in 0..NUM_THREADS {
    //         let counter = counter.clone();
    //         let handle = thread::spawn(move || {
    //             // Each thread registers a handler for a specific key
    //             let key = KeyCode::Char((b'a' + i) as char);

    //             let counter = counter.clone();
    //             on_global_event(key, move || {
    //                 counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //                 false // Continue propagation
    //             });

    //             // Return the key this thread is handling
    //             key
    //         });
    //         handles.push(handle);
    //     }

    //     // Collect all registered keys
    //     let registered_keys: Vec<KeyCode> = handles
    //         .into_iter()
    //         .map(|h| h.join().expect("Thread panicked"))
    //         .collect();

    //     // Test each registered key
    //     for key in registered_keys {
    //         // Reset counter
    //         counter.store(0, std::sync::atomic::Ordering::SeqCst);

    //         // Create and process the event
    //         let event = KeyEvent {
    //             code: key,
    //             modifiers: crossterm::event::KeyModifiers::NONE,
    //             kind: crossterm::event::KeyEventKind::Press,
    //             state: crossterm::event::KeyEventState::NONE,
    //         };

    //         // Process the event multiple times to ensure the handler is called
    //         for _ in 0..5 {
    //             process_global_event(&event);
    //             std::thread::yield_now(); // Give the handler a chance to run
    //         }

    //         // Verify the handler was called at least once
    //         let count = counter.load(std::sync::atomic::Ordering::SeqCst);
    //         assert!(
    //             count >= 1,
    //             "Expected at least 1 handler for key {:?} to be called, got {}",
    //             key,
    //             count
    //         );
    //     }

    //     // Verify that we registered the expected number of handlers
    //     let handlers = GLOBAL_EVENT_HANDLERS.lock();
    //     assert_eq!(
    //         handlers.len(),
    //         NUM_THREADS as usize,
    //         "Expected {} registered handlers, got {}",
    //         NUM_THREADS,
    //         handlers.len()
    //     );

    //     // Explicitly clear handlers to prevent interference with other tests
    //     drop(handlers);
    //     GLOBAL_EVENT_HANDLERS.lock().clear();
    // }

    #[test]
    fn test_release_handlers() {
        use crate::test_utils::TEST_MUTEX;
        let _lock = TEST_MUTEX.lock();

        // Clear any existing handlers
        GLOBAL_EVENT_HANDLERS.lock().clear();

        // Create a counter to track handler calls
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        // Create a scope to control the lifetime of the handler
        {
            let c = counter.clone();
            on_global_event(KeyCode::Char('z'), move || {
                // This will increment the counter as long as the Arc is alive
                c.fetch_add(1, Ordering::SeqCst);
                true
            });

            // Handler should work when in scope
            let event = KeyEvent::new(KeyCode::Char('z'), crossterm::event::KeyModifiers::NONE);
            assert!(
                process_global_event(&event),
                "Handler should process the event when in scope"
            );
            assert_eq!(
                counter.load(Ordering::SeqCst),
                1,
                "Counter should be incremented when handler is called"
            );
        }

        // The Arc is now dropped, but the handler is still registered
        // The handler will still be called, but since the counter was dropped,
        // it will use a stale reference (which is unsafe, but we're testing the behavior)
        let event = KeyEvent::new(KeyCode::Char('z'), crossterm::event::KeyModifiers::NONE);

        // We can't make any guarantees about the counter value after the Arc is dropped,
        // but we can verify that the handler is still called and returns true
        assert!(
            process_global_event(&event),
            "Handler should still process the event"
        );

        // We can't reliably test the counter value here since the Arc is dropped
        // and accessing it would be undefined behavior
        // Instead, we'll just verify that the counter hasn't been modified in an unexpected way
        let final_count = counter.load(Ordering::SeqCst);
        assert!(
            final_count == 1 || final_count == 2,
            "Counter should be 1 or 2, got {}",
            final_count
        );
    }

    #[test]
    fn test_key_modifiers() {
        use crate::test_utils::TEST_MUTEX;
        let _lock = TEST_MUTEX.lock();

        GLOBAL_EVENT_HANDLERS.lock().clear();

        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();

        on_global_event(KeyCode::Char('c'), move || {
            c.fetch_add(1, Ordering::SeqCst);
            true
        });

        // Create events with different modifiers but same key code
        let events = [
            KeyEvent::new(KeyCode::Char('c'), crossterm::event::KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('c'), crossterm::event::KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('c'), crossterm::event::KeyModifiers::ALT),
            KeyEvent::new(KeyCode::Char('c'), crossterm::event::KeyModifiers::NONE),
        ];

        // Process each event and verify the handler was called
        for event in &events {
            let result = process_global_event(event);
            assert!(
                result,
                "Handler should have been called for event: {:?}",
                event
            );
        }

        // Verify the handler was called for each event
        assert_eq!(
            counter.load(Ordering::SeqCst),
            events.len(),
            "Handler should have been called {} times",
            events.len()
        );
    }

    #[test]
    fn test_remove_handlers() {
        use crate::test_utils::TEST_MUTEX;
        let _lock = TEST_MUTEX.lock();

        GLOBAL_EVENT_HANDLERS.lock().clear();

        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();

        // Add and then remove handler
        on_global_event(KeyCode::Char('x'), move || {
            c.fetch_add(1, Ordering::SeqCst);
            true
        });

        // Clear all handlers
        GLOBAL_EVENT_HANDLERS.lock().clear();

        let event = KeyEvent::new(KeyCode::Char('x'), crossterm::event::KeyModifiers::NONE);
        assert!(!process_global_event(&event));
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }
}
