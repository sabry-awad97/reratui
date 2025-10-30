use ratatui::{buffer::Buffer, layout::Rect};

/// A trait that models a terminal UI component with React-style lifecycle methods.
///
/// This trait mimics the lifecycle of a React class component, allowing fine-grained control
/// over initialization, rendering, updates, and cleanup in a terminal UI environment using `ratatui`.
pub trait Component {
    /// Called immediately after the component is mounted into the UI tree.
    ///
    /// This is the place to perform any side-effects or data loading.
    fn on_mount(&mut self) {}

    /// Called when the component is about to receive new props or state.
    /// Returns `true` if the component should re-render; `false` otherwise.
    ///
    /// Useful for preventing unnecessary renders.
    fn should_update(&self) -> bool {
        true
    }

    /// Handle an event.
    ///
    /// # Arguments
    /// * `event` - The event to handle
    ///
    /// # Returns
    /// * `true` if the event was handled and the app should continue running
    /// * `false` if the app should exit
    fn on_event(&mut self, _event: &crossterm::event::Event) -> bool {
        // Default implementation: ignore the event and continue running
        true
    }

    /// Called immediately after an update (after `should_update` returns true).
    ///
    /// Use this to trigger additional side-effects or post-update computations.
    fn on_update(&mut self) {}

    /// Called immediately before the component is removed from the UI tree.
    ///
    /// Useful for cleanup, cancelling tasks, or releasing resources.
    fn on_unmount(&mut self) {}

    /// Called to render the component into the specified portion of the terminal frame.
    ///
    /// # Arguments
    /// * `area` - The rectangular region to render within.
    /// * `buffer` - The terminal buffer to draw on.
    fn render(&self, area: Rect, buffer: &mut Buffer);
}
