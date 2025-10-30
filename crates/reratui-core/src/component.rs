use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::collections::HashMap;

thread_local! {
    // Track mounted component instances and their mount states
    static MOUNT_STATE: std::cell::RefCell<MountState> = Default::default();
}

// Store cleanup callbacks for unmounting
type CleanupFn = Box<dyn Fn() + 'static>;

#[derive(Default)]
struct MountState {
    // Tracks all currently mounted components by their ID hash
    mounted: std::collections::HashSet<usize>,
    // Components that were mounted in the last render
    current_render: std::collections::HashSet<usize>,
    // Cleanup functions for each mounted component
    cleanup_fns: HashMap<usize, CleanupFn>,
}

impl MountState {
    fn track_mount<F>(&mut self, id_hash: usize, cleanup_fn: F) -> bool
    where
        F: Fn() + 'static,
    {
        self.current_render.insert(id_hash);

        // Returns true if this is the first time mounting (newly inserted)
        let is_new = self.mounted.insert(id_hash);

        if is_new {
            // Store cleanup function
            self.cleanup_fns.insert(id_hash, Box::new(cleanup_fn));
        }

        is_new
    }

    fn cleanup_unmounted(&mut self) {
        // Find components that were mounted before but not in current render
        let unmounted: Vec<_> = self
            .mounted
            .difference(&self.current_render)
            .cloned()
            .collect();

        // Call cleanup functions and remove unmounted components
        for &id_hash in &unmounted {
            if let Some(cleanup_fn) = self.cleanup_fns.remove(&id_hash) {
                cleanup_fn(); // Call on_unmount
            }
            self.mounted.remove(&id_hash);
        }

        // Prepare for next render
        self.current_render.clear();
    }
}

pub trait Component: 'static {
    /// Called once when the component is first mounted
    fn on_mount(&self) {}

    /// Called when the component is about to be unmounted
    fn on_unmount(&self) {}

    /// Called on every render
    fn render(&self, area: Rect, frame: &mut Buffer);

    /// Gets a unique identifier for this component instance
    fn component_id(&self) -> String {
        // Default implementation uses the type name
        std::any::type_name::<Self>().to_string()
    }

    /// Clone the component into a Box
    /// This method makes the trait object-safe while still allowing cloning
    fn clone_box(&self) -> Box<dyn Component>
    where
        Self: Clone,
    {
        Box::new(self.clone())
    }

    /// Renders the component with mount/unmount lifecycle tracking
    fn render_with_mount(&self, area: Rect, frame: &mut Frame)
    where
        Self: Clone,
    {
        let component_id = self.component_id();
        let id_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            component_id.hash(&mut hasher);
            hasher.finish() as usize
        };

        // Create cleanup function that captures a clone of self
        let self_clone = self.clone();
        let cleanup_fn = move || {
            self_clone.on_unmount();
        };

        // Track this component in the current render
        let is_first_render = MOUNT_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.track_mount(id_hash, cleanup_fn)
        });

        // Call on_mount on first render
        if is_first_render {
            self.on_mount();
        }

        // Call the actual render method
        self.render(area, frame.buffer_mut());
    }
}

/// Cleans up any components that were unmounted in the last render cycle
/// This should be called after each render cycle
pub fn cleanup_unmounted() {
    MOUNT_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.cleanup_unmounted();
    });
}
