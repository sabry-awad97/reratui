use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

thread_local! {
    static HOOK_CONTEXT: RefCell<Option<Rc<HookContext>>> = const { RefCell::new(None) };
}

/// A hook context that manages state for components
pub struct HookContext {
    pub(crate) states: RefCell<HashMap<usize, Box<dyn Any>>>,
    current_hook: RefCell<usize>,
}

impl HookContext {
    /// Create a new hook context
    pub fn new() -> Self {
        Self {
            states: RefCell::new(HashMap::new()),
            current_hook: RefCell::new(0),
        }
    }

    /// Get the current hook index
    pub fn current_hook_index(&self) -> usize {
        *self.current_hook.borrow()
    }

    /// Get the current hook index and increment it
    pub fn next_hook_index(&self) -> usize {
        let mut current = self.current_hook.borrow_mut();
        let index = *current;
        *current += 1;
        index
    }

    /// Reset the hook index for a new render cycle
    pub fn reset_hook_index(&self) {
        *self.current_hook.borrow_mut() = 0;
    }

    /// Get state for a specific hook index
    pub fn get_state<T: 'static + Clone>(&self, index: usize) -> Option<T> {
        self.states
            .borrow()
            .get(&index)
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .cloned()
    }

    /// Set state for a specific hook index
    pub fn set_state<T: 'static>(&self, index: usize, value: T) {
        self.states.borrow_mut().insert(index, Box::new(value));
    }

    /// Get or initialize state for a specific hook index
    pub fn get_or_init_state<T: 'static, F>(
        &self,
        index: usize,
        init: F,
    ) -> std::rc::Rc<std::cell::RefCell<T>>
    where
        F: FnOnce() -> T,
    {
        use std::cell::RefCell;
        use std::rc::Rc;

        let mut states = self.states.borrow_mut();

        if let Some(existing) = states.get(&index) {
            // Try to downcast to the expected type
            if let Some(typed_state) = existing.downcast_ref::<Rc<RefCell<T>>>() {
                return typed_state.clone();
            }
        }

        // Initialize new state
        let new_state = Rc::new(RefCell::new(init()));
        states.insert(index, Box::new(new_state.clone()));
        new_state
    }

    /// Check if state exists for a hook index
    pub fn has_state(&self, index: usize) -> bool {
        self.states.borrow().contains_key(&index)
    }

    /// Clear all state (useful for cleanup)
    pub fn clear(&self) {
        self.states.borrow_mut().clear();
        self.reset_hook_index();
    }
}

impl Default for HookContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Set the current hook context for the thread
pub fn set_hook_context(context: Rc<HookContext>) {
    HOOK_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(context);
    });
}

/// Get the current hook context for the thread
pub fn get_hook_context() -> Option<Rc<HookContext>> {
    HOOK_CONTEXT.with(|ctx| ctx.borrow().clone())
}

/// Clear the hook context for the thread
pub fn clear_hook_context() {
    HOOK_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
}

/// Get the current hook context
pub fn with_hook_context<R>(f: impl FnOnce(&HookContext) -> R) -> R {
    let context =
        get_hook_context().expect("with_hook_context must be called within a hook context");
    f(&context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_context_creation() {
        let context = HookContext::new();
        assert_eq!(*context.current_hook.borrow(), 0);
        assert!(context.states.borrow().is_empty());
    }

    #[test]
    fn test_hook_index_management() {
        let context = HookContext::new();

        // Test incrementing hook index
        assert_eq!(context.next_hook_index(), 0);
        assert_eq!(context.next_hook_index(), 1);
        assert_eq!(context.next_hook_index(), 2);

        // Test reset
        context.reset_hook_index();
        assert_eq!(context.next_hook_index(), 0);
    }

    #[test]
    fn test_state_management() {
        let context = HookContext::new();

        // Test setting and getting state
        context.set_state(0, 42i32);
        assert_eq!(context.get_state::<i32>(0), Some(42));

        // Test different types
        context.set_state(1, "hello".to_string());
        assert_eq!(context.get_state::<String>(1), Some("hello".to_string()));

        // Test non-existent state
        assert_eq!(context.get_state::<i32>(99), None);
    }

    #[test]
    fn test_has_state() {
        let context = HookContext::new();

        assert!(!context.has_state(0));

        context.set_state(0, 42i32);
        assert!(context.has_state(0));
        assert!(!context.has_state(1));
    }

    #[test]
    fn test_clear_state() {
        let context = HookContext::new();

        // Add some state
        context.set_state(0, 42i32);
        context.set_state(1, "test".to_string());
        context.next_hook_index(); // Advance hook index

        assert!(context.has_state(0));
        assert!(context.has_state(1));

        // Clear all state
        context.clear();

        assert!(!context.has_state(0));
        assert!(!context.has_state(1));
        assert_eq!(*context.current_hook.borrow(), 0);
    }

    #[test]
    fn test_thread_local_context_management() {
        let context = Rc::new(HookContext::new());

        // Initially no context
        assert!(get_hook_context().is_none());

        // Set context
        set_hook_context(context.clone());
        assert!(get_hook_context().is_some());

        // Clear context
        clear_hook_context();
        assert!(get_hook_context().is_none());
    }

    #[test]
    fn test_with_hook_context() {
        let context = Rc::new(HookContext::new());
        context.set_state(0, 100i32);

        // Set up the context first
        set_hook_context(context.clone());

        let result = with_hook_context(|ctx| {
            // State should be available
            assert_eq!(ctx.get_state::<i32>(0), Some(100));
            "test_result"
        });

        assert_eq!(result, "test_result");

        // Clean up
        clear_hook_context();
    }

    #[test]
    fn test_type_safety() {
        let context = HookContext::new();

        // Set state as i32
        context.set_state(0, 42i32);

        // Try to get as different type - should return None
        assert_eq!(context.get_state::<String>(0), None);

        // Get as correct type - should work
        assert_eq!(context.get_state::<i32>(0), Some(42));
    }

    #[test]
    fn test_default_implementation() {
        let context = HookContext::default();
        assert_eq!(*context.current_hook.borrow(), 0);
        assert!(context.states.borrow().is_empty());
    }
}
