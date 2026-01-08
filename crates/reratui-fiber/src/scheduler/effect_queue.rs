//! Effect queue for post-commit effect execution.

use std::any::Any;
use std::cell::RefCell;

use crate::fiber::{AsyncCleanupFn, AsyncPendingEffect, CleanupFn, FiberId, PendingEffect};
use crate::fiber_tree::FiberTree;

thread_local! {
    /// Thread-local effect queue for the current render context
    static EFFECT_QUEUE: RefCell<EffectQueue> = RefCell::new(EffectQueue::new());
}

/// State stored for each effect hook instance
pub struct EffectHookState {
    /// Dependencies from the last render (boxed for type erasure)
    pub deps: Option<Box<dyn Any + Send>>,
    /// Cleanup function from the last effect execution
    pub cleanup: Option<CleanupFn>,
}

impl EffectHookState {
    /// Create a new effect hook state
    pub fn new() -> Self {
        Self {
            deps: None,
            cleanup: None,
        }
    }

    /// Create with initial deps
    pub fn with_deps<D: Any + Send + 'static>(deps: D) -> Self {
        Self {
            deps: Some(Box::new(deps)),
            cleanup: None,
        }
    }

    /// Check if deps have changed
    pub fn deps_changed<D: PartialEq + 'static>(&self, new_deps: &D) -> bool {
        match &self.deps {
            None => true, // No previous deps means first render
            Some(boxed) => {
                match boxed.downcast_ref::<D>() {
                    Some(old_deps) => old_deps != new_deps,
                    None => true, // Type mismatch, treat as changed
                }
            }
        }
    }

    /// Update deps
    pub fn set_deps<D: Any + Send + 'static>(&mut self, deps: D) {
        self.deps = Some(Box::new(deps));
    }

    /// Take the cleanup function (removes it from state)
    pub fn take_cleanup(&mut self) -> Option<CleanupFn> {
        self.cleanup.take()
    }

    /// Set the cleanup function
    pub fn set_cleanup(&mut self, cleanup: CleanupFn) {
        self.cleanup = Some(cleanup);
    }
}

impl Default for EffectHookState {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue of effects to run after commit phase
pub struct EffectQueue {
    /// Effects queued during current render, grouped by fiber
    pending: Vec<(FiberId, PendingEffect)>,
    /// Async effects queued during current render, grouped by fiber
    pending_async: Vec<(FiberId, AsyncPendingEffect)>,
    /// Cleanups to run before new effects
    cleanups_to_run: Vec<CleanupFn>,
    /// Async cleanups to run before new effects
    async_cleanups_to_run: Vec<AsyncCleanupFn>,
}

impl EffectQueue {
    /// Create a new empty effect queue
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            pending_async: Vec::new(),
            cleanups_to_run: Vec::new(),
            async_cleanups_to_run: Vec::new(),
        }
    }

    /// Queue an effect for post-commit execution
    pub fn queue_effect(&mut self, fiber_id: FiberId, effect: PendingEffect) {
        self.pending.push((fiber_id, effect));
    }

    /// Queue an async effect for post-commit execution
    pub fn queue_async_effect(&mut self, fiber_id: FiberId, effect: AsyncPendingEffect) {
        self.pending_async.push((fiber_id, effect));
    }

    /// Queue a cleanup function to run before new effects
    pub fn queue_cleanup(&mut self, cleanup: CleanupFn) {
        self.cleanups_to_run.push(cleanup);
    }

    /// Queue an async cleanup function to run before new effects
    pub fn queue_async_cleanup(&mut self, cleanup: AsyncCleanupFn) {
        self.async_cleanups_to_run.push(cleanup);
    }

    /// Execute all queued effects (called after commit)
    /// Note: This only runs synchronous effects. Use flush_async for async effects.
    pub fn flush(&mut self, tree: &mut FiberTree) {
        // 1. Run cleanups in reverse order
        while let Some(cleanup) = self.cleanups_to_run.pop() {
            cleanup();
        }

        // 2. Run effects in declaration order
        for (fiber_id, pending) in self.pending.drain(..) {
            if let Some(fiber) = tree.get_mut(fiber_id)
                && let Some(cleanup) = (pending.effect)()
            {
                // Store cleanup indexed by hook_index for proper cleanup ordering
                fiber.cleanup_by_hook.insert(pending.hook_index, cleanup);
            }
        }
    }

    /// Execute all queued async effects (called after commit)
    /// This is an async function that handles async cleanups and effects.
    pub async fn flush_async(&mut self, tree: &mut FiberTree) {
        // 1. Run sync cleanups in reverse order first
        while let Some(cleanup) = self.cleanups_to_run.pop() {
            cleanup();
        }

        // 2. Run async cleanups in reverse order
        while let Some(async_cleanup) = self.async_cleanups_to_run.pop() {
            async_cleanup().await;
        }

        // 3. Run sync effects in declaration order
        for (fiber_id, pending) in self.pending.drain(..) {
            if let Some(fiber) = tree.get_mut(fiber_id)
                && let Some(cleanup) = (pending.effect)()
            {
                // Store cleanup indexed by hook_index for proper cleanup ordering
                fiber.cleanup_by_hook.insert(pending.hook_index, cleanup);
            }
        }

        // 4. Run async effects in declaration order
        for (_fiber_id, pending) in self.pending_async.drain(..) {
            if let Some(async_cleanup) = (pending.effect)().await {
                // Store async cleanup for next flush
                // Note: We store it in the queue for the next cycle
                // In a real implementation, you might want to store it in the fiber
                self.async_cleanups_to_run.push(async_cleanup);
            }
        }
    }

    /// Check if there are pending effects
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
            || !self.pending_async.is_empty()
            || !self.cleanups_to_run.is_empty()
            || !self.async_cleanups_to_run.is_empty()
    }

    /// Check if there are pending async effects
    pub fn has_pending_async(&self) -> bool {
        !self.pending_async.is_empty() || !self.async_cleanups_to_run.is_empty()
    }

    /// Clear all pending effects and cleanups
    pub fn clear(&mut self) {
        self.pending.clear();
        self.pending_async.clear();
        self.cleanups_to_run.clear();
        self.async_cleanups_to_run.clear();
    }

    /// Get the number of pending effects
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get the number of pending async effects
    pub fn pending_async_count(&self) -> usize {
        self.pending_async.len()
    }

    /// Drain all pending async effects for testing or manual execution
    /// Returns the pending async effects as a vector
    pub fn drain_async_effects(&mut self) -> Vec<(FiberId, AsyncPendingEffect)> {
        self.pending_async.drain(..).collect()
    }

    /// Get the number of pending cleanups
    pub fn cleanup_count(&self) -> usize {
        self.cleanups_to_run.len()
    }

    /// Get the number of pending async cleanups
    pub fn async_cleanup_count(&self) -> usize {
        self.async_cleanups_to_run.len()
    }
}

impl Default for EffectQueue {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local access functions

/// Queue an effect to the thread-local effect queue
pub fn queue_effect(fiber_id: FiberId, effect: PendingEffect) {
    EFFECT_QUEUE.with(|q| {
        q.borrow_mut().queue_effect(fiber_id, effect);
    });
}

/// Queue an async effect to the thread-local effect queue
pub fn queue_async_effect(fiber_id: FiberId, effect: AsyncPendingEffect) {
    EFFECT_QUEUE.with(|q| {
        q.borrow_mut().queue_async_effect(fiber_id, effect);
    });
}

/// Queue a cleanup to the thread-local effect queue
pub fn queue_cleanup(cleanup: CleanupFn) {
    EFFECT_QUEUE.with(|q| {
        q.borrow_mut().queue_cleanup(cleanup);
    });
}

/// Queue an async cleanup to the thread-local effect queue
pub fn queue_async_cleanup(cleanup: AsyncCleanupFn) {
    EFFECT_QUEUE.with(|q| {
        q.borrow_mut().queue_async_cleanup(cleanup);
    });
}

/// Flush the thread-local effect queue with a provided tree
pub fn flush_effects_with_tree(tree: &mut FiberTree) {
    EFFECT_QUEUE.with(|q| {
        q.borrow_mut().flush(tree);
    });
}

/// Flush the thread-local effect queue using the thread-local fiber tree
pub fn flush_effects() {
    crate::fiber_tree::with_fiber_tree_mut(|tree| {
        EFFECT_QUEUE.with(|q| {
            q.borrow_mut().flush(tree);
        });
    });
}

/// Flush async effects from the thread-local effect queue using the thread-local fiber tree
/// This handles async cleanups and async effects with tokio.
pub async fn flush_async_effects() {
    // We need to handle the async flush carefully due to thread-local borrowing
    // First, check if there are any async effects to process
    let has_async = has_pending_async_effects();

    if has_async {
        // Drain the async effects and cleanups to process them outside the borrow
        let (async_effects, async_cleanups) = EFFECT_QUEUE.with(|q| {
            let mut queue = q.borrow_mut();
            let effects = queue.pending_async.drain(..).collect::<Vec<_>>();
            let cleanups = queue.async_cleanups_to_run.drain(..).collect::<Vec<_>>();
            (effects, cleanups)
        });

        // Run async cleanups in reverse order
        for async_cleanup in async_cleanups.into_iter().rev() {
            async_cleanup().await;
        }

        // Run async effects in declaration order
        for (_fiber_id, pending) in async_effects {
            if let Some(async_cleanup) = (pending.effect)().await {
                // Store async cleanup for next flush
                EFFECT_QUEUE.with(|q| {
                    q.borrow_mut().async_cleanups_to_run.push(async_cleanup);
                });
            }
        }
    }
}

/// Check if the thread-local effect queue has pending work
pub fn has_pending_effects() -> bool {
    EFFECT_QUEUE.with(|q| q.borrow().has_pending())
}

/// Check if the thread-local effect queue has pending async work
pub fn has_pending_async_effects() -> bool {
    EFFECT_QUEUE.with(|q| q.borrow().has_pending_async())
}

/// Clear the thread-local effect queue
pub fn clear_effect_queue() {
    EFFECT_QUEUE.with(|q| {
        q.borrow_mut().clear();
    });
}

/// Execute a closure with the thread-local effect queue
pub fn with_effect_queue<R, F: FnOnce(&mut EffectQueue) -> R>(f: F) -> R {
    EFFECT_QUEUE.with(|q| f(&mut q.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_effect_queue_creation() {
        let queue = EffectQueue::new();
        assert!(!queue.has_pending());
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.cleanup_count(), 0);
    }

    #[test]
    fn test_queue_effect() {
        let mut queue = EffectQueue::new();
        let fiber_id = FiberId(1);

        let effect = PendingEffect {
            effect: Box::new(|| None),
            hook_index: 0,
        };

        queue.queue_effect(fiber_id, effect);
        assert!(queue.has_pending());
        assert_eq!(queue.pending_count(), 1);
    }

    #[test]
    fn test_queue_cleanup() {
        let mut queue = EffectQueue::new();

        let cleanup: CleanupFn = Box::new(|| {});
        queue.queue_cleanup(cleanup);

        assert!(queue.has_pending());
        assert_eq!(queue.cleanup_count(), 1);
    }

    #[test]
    fn test_flush_runs_cleanups_before_effects() {
        let mut tree = FiberTree::new();
        let fiber_id = tree.mount(None, None);

        let execution_order = Arc::new(Mutex::new(Vec::new()));

        let mut queue = EffectQueue::new();

        // Queue a cleanup
        let order_clone = execution_order.clone();
        let cleanup: CleanupFn = Box::new(move || {
            order_clone.lock().unwrap().push("cleanup");
        });
        queue.queue_cleanup(cleanup);

        // Queue an effect
        let order_clone = execution_order.clone();
        let effect = PendingEffect {
            effect: Box::new(move || {
                order_clone.lock().unwrap().push("effect");
                None
            }),
            hook_index: 0,
        };
        queue.queue_effect(fiber_id, effect);

        // Flush
        queue.flush(&mut tree);

        // Verify order: cleanups run before effects
        let order = execution_order.lock().unwrap();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], "cleanup");
        assert_eq!(order[1], "effect");
    }

    #[test]
    fn test_flush_runs_cleanups_in_reverse_order() {
        let mut tree = FiberTree::new();
        let _ = tree.mount(None, None);

        let execution_order = Arc::new(Mutex::new(Vec::new()));

        let mut queue = EffectQueue::new();

        // Queue multiple cleanups
        for i in 1..=3 {
            let order_clone = execution_order.clone();
            let cleanup: CleanupFn = Box::new(move || {
                order_clone.lock().unwrap().push(i);
            });
            queue.queue_cleanup(cleanup);
        }

        queue.flush(&mut tree);

        // Verify reverse order: 3, 2, 1
        let order = execution_order.lock().unwrap();
        assert_eq!(*order, vec![3, 2, 1]);
    }

    #[test]
    fn test_flush_runs_effects_in_declaration_order() {
        let mut tree = FiberTree::new();
        let fiber_id = tree.mount(None, None);

        let execution_order = Arc::new(Mutex::new(Vec::new()));

        let mut queue = EffectQueue::new();

        // Queue multiple effects
        for i in 1..=3 {
            let order_clone = execution_order.clone();
            let effect = PendingEffect {
                effect: Box::new(move || {
                    order_clone.lock().unwrap().push(i);
                    None
                }),
                hook_index: i,
            };
            queue.queue_effect(fiber_id, effect);
        }

        queue.flush(&mut tree);

        // Verify declaration order: 1, 2, 3
        let order = execution_order.lock().unwrap();
        assert_eq!(*order, vec![1, 2, 3]);
    }

    #[test]
    fn test_effect_returns_cleanup() {
        let mut tree = FiberTree::new();
        let fiber_id = tree.mount(None, None);

        let mut queue = EffectQueue::new();

        let effect = PendingEffect {
            effect: Box::new(|| Some(Box::new(|| {}) as CleanupFn)),
            hook_index: 0,
        };
        queue.queue_effect(fiber_id, effect);

        queue.flush(&mut tree);

        // Verify cleanup was stored in fiber by hook_index
        let fiber = tree.get(fiber_id).unwrap();
        assert_eq!(fiber.cleanup_by_hook.len(), 1);
        assert!(fiber.cleanup_by_hook.contains_key(&0));
    }

    #[test]
    fn test_clear_queue() {
        let mut queue = EffectQueue::new();
        let fiber_id = FiberId(1);

        queue.queue_effect(
            fiber_id,
            PendingEffect {
                effect: Box::new(|| None),
                hook_index: 0,
            },
        );
        queue.queue_cleanup(Box::new(|| {}));

        assert!(queue.has_pending());

        queue.clear();

        assert!(!queue.has_pending());
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.cleanup_count(), 0);
    }

    #[test]
    fn test_effect_hook_state_creation() {
        let state = EffectHookState::new();
        assert!(state.deps.is_none());
        assert!(state.cleanup.is_none());
    }

    #[test]
    fn test_effect_hook_state_with_deps() {
        let state = EffectHookState::with_deps((1, 2, 3));
        assert!(state.deps.is_some());
    }

    #[test]
    fn test_effect_hook_state_deps_changed() {
        let mut state = EffectHookState::new();

        // First render - no deps, should be "changed"
        assert!(state.deps_changed(&(1, 2)));

        // Set deps
        state.set_deps((1, 2));

        // Same deps - not changed
        assert!(!state.deps_changed(&(1, 2)));

        // Different deps - changed
        assert!(state.deps_changed(&(1, 3)));
    }

    #[test]
    fn test_effect_hook_state_cleanup() {
        let mut state = EffectHookState::new();

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        state.set_cleanup(Box::new(move || {
            called_clone.store(true, Ordering::SeqCst);
        }));

        assert!(state.cleanup.is_some());

        let cleanup = state.take_cleanup();
        assert!(state.cleanup.is_none());

        cleanup.unwrap()();
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_thread_local_queue_effect() {
        // Clear any existing state
        clear_effect_queue();

        let fiber_id = FiberId(1);
        queue_effect(
            fiber_id,
            PendingEffect {
                effect: Box::new(|| None),
                hook_index: 0,
            },
        );

        assert!(has_pending_effects());

        clear_effect_queue();
        assert!(!has_pending_effects());
    }
}
