//! Runtime and event loop for Reratui TUI framework
//!
//! This module provides the core runtime functionality for Reratui applications,
//! including terminal management, event handling, and the render loop.

mod exit;
mod managed_terminal;

pub use exit::{request_exit, reset_exit, should_exit};
pub use managed_terminal::{ManagedTerminal, restore_terminal, setup_terminal};

use anyhow::Result;
use crossterm::event::Event;
use reratui_core::Element;
use reratui_hooks::frame::FrameContext;
use reratui_hooks::hook_context::HookContext;
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

/// Renders a component-based TUI application with hooks support
///
/// This function sets up a hook context and manages the component lifecycle
/// including state persistence between renders.
///
/// # Arguments
/// * `app_fn` - A closure that returns an Element (supports both components and RSX)
///
/// # Example
/// ```no_run,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn Counter() -> Element {
///     let (count, set_count) = use_state(|| 0);
///     rsx! { <Text text={format!("Count: {}", count)} /> }
/// }
///
/// # async fn example() {
/// // Direct component
/// render(|| Counter()).await.unwrap();
///
/// // Or with RSX
/// render(|| {
///     rsx! { <Counter /> }
/// }).await.unwrap();
/// # }
/// ```
pub async fn render<F>(initializer: F) -> Result<()>
where
    F: Fn() -> Element + 'static,
{
    // Initialize panic handler
    reratui_panic::setup_panic_handler();

    // Initialize terminal backend
    let mut terminal = setup_terminal()?;

    // Create a new hook context for this component tree
    let hook_context = Rc::new(HookContext::new());

    // Set the hook context for this thread
    reratui_hooks::hook_context::set_hook_context(hook_context.clone());

    // Create the element
    let element = initializer();

    // Frame tracking
    let mut frame_count: u64 = 0;
    let mut last_frame_time = Instant::now();

    // Create async event stream
    use crossterm::event::EventStream;
    use tokio_stream::StreamExt;
    let mut events = EventStream::new();

    // Main render loop with continuous rendering
    loop {
        // Calculate frame timing
        let current_time = Instant::now();
        let delta = current_time.duration_since(last_frame_time);
        last_frame_time = current_time;

        // Reset hook index before each render
        hook_context.reset_hook_index();

        // Poll for events with timeout (allows continuous rendering)
        let timeout = tokio::time::sleep(Duration::from_millis(16));
        tokio::pin!(timeout);

        tokio::select! {
            Some(Ok(event)) = events.next() => {
                // Process key events through global event system
                let processed = if let Event::Key(key_event) = &event {
                    reratui_hooks::event::global_events::process_global_event(key_event)
                } else {
                    false
                };

                // If not processed as a global event, make it available to components
                if !processed {
                    reratui_hooks::event::set_current_event(Some(std::sync::Arc::new(event)));
                }
            }
            _ = &mut timeout => {
                // Timeout - clear event and continue rendering
                reratui_hooks::event::set_current_event(None);
            }
        }

        // Check for exit
        if should_exit() {
            break;
        }

        // Render the element
        terminal.draw(|frame| {
            // SAFETY: The FrameContext is only used within this render scope
            // and the frame pointer remains valid for the duration of the draw call
            let frame_ctx = unsafe { FrameContext::new(frame, frame_count, delta, current_time) };

            // Provide frame context for components
            let _frame_context = reratui_hooks::context::use_context_provider(|| frame_ctx);

            let area = frame.area();
            element.render(area, frame.buffer_mut());
        })?;

        // Clean up unmounted components after render
        reratui_core::component::cleanup_unmounted();

        // Increment frame counter
        frame_count += 1;
    }

    // Clear the current event
    reratui_hooks::event::set_current_event(None);

    // Clean up the hook context
    reratui_hooks::hook_context::clear_hook_context();

    // Restore terminal state
    restore_terminal()?;

    Ok(())
}
