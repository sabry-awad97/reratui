use std::sync::atomic::{AtomicBool, Ordering};

static GLOBAL_EXIT: AtomicBool = AtomicBool::new(false);

/// Request the application to exit
pub fn request_exit() {
    GLOBAL_EXIT.store(true, Ordering::Release);
}

/// Check if exit has been requested
pub fn should_exit() -> bool {
    GLOBAL_EXIT.load(Ordering::Acquire)
}

/// Reset the exit flag (useful for tests)
pub fn reset_exit() {
    GLOBAL_EXIT.store(false, Ordering::Release);
}

/// A guard that automatically resets the exit flag when dropped
pub struct ExitGuard;

impl Drop for ExitGuard {
    fn drop(&mut self) {
        reset_exit();
    }
}

/// Create a new exit guard
pub fn exit_guard() -> ExitGuard {
    ExitGuard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_flag() {
        assert!(!should_exit());
        request_exit();
        assert!(should_exit());
        reset_exit();
        assert!(!should_exit());
    }

    #[test]
    fn test_exit_guard() {
        assert!(!should_exit());
        {
            let _guard = exit_guard();
            request_exit();
            assert!(should_exit());
        }
        assert!(!should_exit());
    }
}
