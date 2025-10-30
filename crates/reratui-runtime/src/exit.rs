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
}
