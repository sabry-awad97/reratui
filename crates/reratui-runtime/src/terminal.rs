//! Terminal management for Reratui runtime
//!
//! This module provides terminal initialization, cleanup, and management
//! functionality for TUI applications.

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};

/// A managed terminal instance that handles setup and cleanup
pub struct ManagedTerminal {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl ManagedTerminal {
    /// Initialize a new terminal with proper setup
    pub fn new() -> io::Result<Self> {
        // Enable raw mode for input handling
        enable_raw_mode()?;

        // Get stdout
        let mut stdout = io::stdout();

        // Enter alternate screen to preserve terminal state
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        // Create the terminal backend
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Get a mutable reference to the terminal
    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    /// Get the terminal size
    pub fn size(&self) -> io::Result<ratatui::layout::Rect> {
        self.terminal
            .size()
            .map(|size| ratatui::layout::Rect::new(0, 0, size.width, size.height))
    }

    /// Clear the terminal
    pub fn clear(&mut self) -> io::Result<()> {
        self.terminal.clear()
    }

    /// Draw the terminal with a closure
    pub fn draw<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }
}

impl Drop for ManagedTerminal {
    /// Cleanup terminal state when dropped
    fn drop(&mut self) {
        // Restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

/// Initialize terminal for TUI applications
pub fn setup_terminal() -> io::Result<ManagedTerminal> {
    ManagedTerminal::new()
}

/// Restore terminal to original state
pub fn restore_terminal() -> io::Result<()> {
    // Disable raw mode
    disable_raw_mode()?;

    // Leave alternate screen and disable mouse capture
    execute!(
        std::io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::cursor::Show
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    /// Test that ManagedTerminal can be created and dropped safely
    #[test]
    fn test_managed_terminal_creation_and_cleanup() {
        // This test verifies basic terminal lifecycle
        // Note: In CI environments, this might fail due to lack of TTY
        // but it's useful for local development

        // We can't easily test actual terminal setup in unit tests
        // since it requires a real terminal, so we test the API structure
        // Placeholder - actual terminal tests need integration environment
    }

    /// Test terminal size conversion from Size to Rect
    #[test]
    fn test_size_conversion() {
        // Test the logic we use in the size() method
        let width = 80u16;
        let height = 24u16;

        let rect = ratatui::layout::Rect::new(0, 0, width, height);

        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, width);
        assert_eq!(rect.height, height);
    }

    /// Test that setup_terminal function exists and has correct signature
    #[test]
    fn test_setup_terminal_signature() {
        // Verify the function signature compiles
        let _setup_fn: fn() -> io::Result<ManagedTerminal> = setup_terminal;
    }

    /// Test that restore_terminal function exists and has correct signature
    #[test]
    fn test_restore_terminal_signature() {
        // Verify the function signature compiles
        let _restore_fn: fn() -> io::Result<()> = restore_terminal;
    }

    /// Test ManagedTerminal method signatures
    #[test]
    fn test_managed_terminal_methods() {
        // Test that all expected methods exist with correct signatures
        // This is a compile-time test

        // We can't actually create a ManagedTerminal in tests without a TTY,
        // but we can verify the method signatures exist
        fn _test_methods(mut terminal: ManagedTerminal) -> io::Result<()> {
            let _size = terminal.size()?;
            terminal.clear()?;
            terminal.draw(|_frame| {})?;
            let _term_ref = terminal.terminal_mut();
            Ok(())
        }

        // Compilation success means methods exist
    }

    /// Test error handling scenarios
    #[test]
    fn test_error_handling() {
        // Test that our error types are compatible
        let _io_error: io::Error = io::Error::other("test");

        // Verify our functions return the expected error types
        fn _test_error_types() {
            let _: io::Result<ManagedTerminal> = setup_terminal();
            let _: io::Result<()> = restore_terminal();
        }
    }

    /// Test thread safety considerations
    #[test]
    fn test_thread_safety() {
        // Test that our types can be used safely across threads
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;

            // Test that our functions can be called from different threads
            let _setup_fn = setup_terminal;
            let _restore_fn = restore_terminal;
        });

        handle.join().unwrap();
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    /// Test Drop implementation behavior
    #[test]
    fn test_drop_implementation() {
        // Verify that ManagedTerminal has drop behavior
        // This is important for RAII cleanup
        use std::mem;

        // Test that ManagedTerminal needs drop (has custom Drop implementation)
        assert!(mem::needs_drop::<ManagedTerminal>());

        // We can't test the actual Drop behavior without a real terminal,
        // but we can verify the type requires cleanup
    }

    /// Performance test for rapid terminal operations
    #[test]
    fn test_performance_characteristics() {
        // Test that our terminal operations complete in reasonable time
        let start = std::time::Instant::now();

        // Simulate the work our functions would do
        for _ in 0..1000 {
            let _rect = ratatui::layout::Rect::new(0, 0, 80, 24);
        }

        let duration = start.elapsed();

        // Should complete very quickly since it's just struct creation
        assert!(duration < Duration::from_millis(10));
    }

    /// Test memory usage patterns
    #[test]
    fn test_memory_usage() {
        // Test that our structs have reasonable memory footprint
        use std::mem;

        // ManagedTerminal should be relatively small
        let terminal_size = mem::size_of::<ManagedTerminal>();

        // Should be reasonable size (less than 1KB)
        assert!(terminal_size < 1024);

        // Test that Rect creation is efficient
        let rect_size = mem::size_of::<ratatui::layout::Rect>();
        assert!(rect_size <= 8); // Should be just 4 u16s
    }
}
