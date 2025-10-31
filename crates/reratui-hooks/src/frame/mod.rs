//! Frame Hook - Access to the current ratatui Frame
//!
//! Filename: mod.rs
//! Folder: /crates/reratui-hooks/src/frame/
//!
//! This module provides direct access to the current ratatui Frame being rendered,
//! along with frame-related information such as:
//! - Frame count (number of frames rendered)
//! - Delta time (time since last frame)
//! - FPS (frames per second)
//! - Frame timestamp
//!
//! # Architecture
//!
//! - Renderer provides Frame pointer via `use_context_provider`
//! - Components access Frame via `use_frame()`
//! - Uses unsafe pointer casting with 'static lifetime for context storage
//! - Safe when used correctly within render scope
//!
//! # Safety
//!
//! The Frame pointer is only valid during the current render cycle.
//! Do not store the Frame reference beyond the component render.

use crate::context::use_context;
use ratatui::Frame;
use std::time::{Duration, Instant};

/// Frame context that holds both the Frame pointer and frame information
///
/// This is provided by the renderer via context and consumed by components.
///
/// # Safety
///
/// The frame_ptr is only valid during the current render cycle.
#[derive(Clone, Copy)]
pub struct FrameContext {
    /// Pointer to the current Frame (with 'static lifetime for context storage)
    frame_ptr: *mut Frame<'static>,
    /// The current frame number (starts at 0)
    pub count: u64,
    /// Time elapsed since the last frame
    pub delta: Duration,
    /// Timestamp when this frame started rendering
    pub timestamp: Instant,
}

// Safety: FrameContext is only used within a single thread during rendering
unsafe impl Send for FrameContext {}
unsafe impl Sync for FrameContext {}

impl FrameContext {
    /// Create a new FrameContext
    ///
    /// # Safety
    ///
    /// The frame pointer must be valid for the duration of the render cycle.
    pub unsafe fn new(frame: &mut Frame, count: u64, delta: Duration, timestamp: Instant) -> Self {
        // SAFETY: We're casting the frame pointer to have a 'static lifetime for storage in context.
        // This is safe because:
        // 1. The FrameContext is only used during the render cycle
        // 2. The actual Frame lifetime is managed by the renderer
        // 3. We never store FrameContext beyond the render scope
        let frame_ptr = std::ptr::from_mut(frame).cast::<Frame<'static>>();
        Self {
            frame_ptr,
            count,
            delta,
            timestamp,
        }
    }

    /// Create a new FrameContext from a raw pointer
    ///
    /// # Safety
    ///
    /// The frame pointer must be valid for the duration of the render cycle.
    /// The pointer must point to a type that is layout-compatible with Frame.
    pub unsafe fn from_raw_ptr(
        frame_ptr: *mut Frame<'static>,
        count: u64,
        delta: Duration,
        timestamp: Instant,
    ) -> Self {
        Self {
            frame_ptr,
            count,
            delta,
            timestamp,
        }
    }

    /// Get a reference to the Frame
    ///
    /// # Safety
    ///
    /// This is safe as long as the FrameContext is only used during the render cycle.
    pub fn frame(&self) -> &Frame<'static> {
        // SAFETY: The pointer is valid for the duration of the render cycle
        unsafe { &*self.frame_ptr }
    }

    /// Get a mutable reference to the Frame
    ///
    /// # Safety
    ///
    /// This is safe as long as the FrameContext is only used during the render cycle
    /// and no other mutable references exist.
    ///
    /// Note: This method takes `&mut self` to ensure exclusive access at the Rust level.
    #[allow(clippy::mut_from_ref)]
    pub fn frame_mut(&mut self) -> &mut Frame<'static> {
        // SAFETY: The pointer is valid and we have exclusive access via &mut self
        unsafe { &mut *self.frame_ptr }
    }

    /// Calculate the current FPS based on delta time
    pub fn fps(&self) -> f64 {
        if self.delta.as_secs_f64() > 0.0 {
            1.0 / self.delta.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get delta time in seconds as f64
    pub fn delta_secs(&self) -> f64 {
        self.delta.as_secs_f64()
    }

    /// Get delta time in milliseconds
    pub fn delta_millis(&self) -> u128 {
        self.delta.as_millis()
    }

    /// Check if this is the first frame
    pub fn is_first_frame(&self) -> bool {
        self.count == 0
    }

    /// Get FrameInfo (without the Frame pointer)
    pub fn frame_info(&self) -> FrameInfo {
        FrameInfo {
            count: self.count,
            delta: self.delta,
            timestamp: self.timestamp,
        }
    }
}

/// Frame information (without the Frame pointer)
///
/// This struct contains information about the current frame being rendered.
#[derive(Clone, Copy, Debug)]
pub struct FrameInfo {
    /// The current frame number (starts at 0)
    pub count: u64,
    /// Time elapsed since the last frame
    pub delta: Duration,
    /// Timestamp when this frame started rendering
    pub timestamp: Instant,
}

impl FrameInfo {
    /// Create a new FrameInfo
    pub fn new(count: u64, delta: Duration, timestamp: Instant) -> Self {
        Self {
            count,
            delta,
            timestamp,
        }
    }

    /// Calculate the current FPS based on delta time
    pub fn fps(&self) -> f64 {
        if self.delta.as_secs_f64() > 0.0 {
            1.0 / self.delta.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get delta time in seconds as f64
    pub fn delta_secs(&self) -> f64 {
        self.delta.as_secs_f64()
    }

    /// Get delta time in milliseconds
    pub fn delta_millis(&self) -> u128 {
        self.delta.as_millis()
    }

    /// Check if this is the first frame
    pub fn is_first_frame(&self) -> bool {
        self.count == 0
    }
}

/// Hook to access the current frame information
///
/// This hook retrieves frame information from the context provided by the renderer.
/// The frame info includes frame count, delta time, and timestamp.
///
/// # Panics
///
/// Panics if called outside of a component render context where the frame info
/// has been provided by the renderer.
///
/// # Use Cases
///
/// - **Animation**: Use delta time for smooth animations
/// - **Performance monitoring**: Track FPS and frame times
/// - **Frame-based logic**: Execute code on specific frames
/// - **Timing**: Calculate elapsed time since app start
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn FrameCounter() -> Element {
///     let frame_ctx = use_frame();
///     
///     rsx! {
///         <Block title="Frame Info">
///             <Paragraph>
///                 {format!("Frame: {}", frame_ctx.count)}
///             </Paragraph>
///             <Paragraph>
///                 {format!("FPS: {:.2}", frame_ctx.fps())}
///             </Paragraph>
///         </Block>
///     }
/// }
/// ```
///
/// ## Animation with Delta Time
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn AnimatedComponent() -> Element {
///     let frame_ctx = use_frame();
///     let (position, set_position) = use_state(|| 0.0f64);
///     
///     // Smooth animation using delta time
///     let speed = 10.0; // units per second
///     let new_pos = position + speed * frame_ctx.delta_secs();
///     set_position.set(new_pos);
///     
///     rsx! {
///         <Block title="Animation">
///             <Paragraph>
///                 {format!("Position: {:.2}", position)}
///             </Paragraph>
///         </Block>
///     }
/// }
/// ```
///
/// ## Performance Monitoring
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn PerformanceMonitor() -> Element {
///     let frame_ctx = use_frame();
///     
///     let fps = frame_ctx.fps();
///     let color = if fps < 30.0 {
///         Color::Red
///     } else if fps < 50.0 {
///         Color::Yellow
///     } else {
///         Color::Green
///     };
///     
///     rsx! {
///         <Block title="Performance">
///             <Paragraph style={Style::default().fg(color)}>
///                 {format!("FPS: {:.2}", fps)}
///             </Paragraph>
///             <Paragraph>
///                 {format!("Frame time: {:.2}ms", frame_ctx.delta_millis())}
///             </Paragraph>
///         </Block>
///     }
/// }
/// ```
///
/// ## Frame-based Logic
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn PeriodicUpdate() -> Element {
///     let frame_ctx = use_frame();
///     
///     // Execute every 60 frames
///     if frame_ctx.count % 60 == 0 {
///         // Do something periodically
///     }
///     
///     rsx! {
///         <Block title="Periodic">
///             <Paragraph>
///                 {format!("Updates: {}", frame_ctx.count / 60)}
///             </Paragraph>
///         </Block>
///     }
/// }
/// ```
/// Hook to access the current Frame and frame information
///
/// Returns a `FrameContext` which provides access to:
/// - The current ratatui Frame (via `.frame()` or `.frame_mut()`)
/// - Frame count, delta time, timestamp
///
/// # Examples
///
/// ```rust,no_run
/// # use reratui_hooks::frame::use_frame;
/// let frame_ctx = use_frame();
///
/// // Access frame info
/// let count = frame_ctx.count;
/// let fps = frame_ctx.fps();
///
/// // Access the Frame
/// let frame = frame_ctx.frame();
/// let area = frame.area();
/// ```
pub fn use_frame() -> FrameContext {
    use_context::<FrameContext>()
}

/// Extension trait for ratatui's Frame to access frame information
///
/// This trait extends ratatui's `Frame` with methods to access frame count,
/// delta time, FPS, and other frame-related information.
///
/// # Examples
///
/// ```rust,no_run
/// # use reratui_hooks::frame::FrameExt;
/// # use ratatui::Frame;
/// fn render_with_frame_info(frame: &mut Frame) {
///     let count = frame.frame_count();
///     let fps = frame.fps();
///     let delta = frame.delta_time();
///     
///     println!("Frame: {} @ {:.1} FPS ({}ms)", count, fps, delta.as_millis());
/// }
/// ```
pub trait FrameExt {
    /// Get the current frame count
    fn frame_count(&self) -> u64;

    /// Get the delta time since the last frame
    fn delta_time(&self) -> Duration;

    /// Get the current timestamp
    fn frame_timestamp(&self) -> Instant;

    /// Calculate the current FPS
    fn fps(&self) -> f64;

    /// Get delta time in seconds as f64
    fn delta_secs(&self) -> f64;

    /// Get delta time in milliseconds
    fn delta_millis(&self) -> u128;

    /// Check if this is the first frame
    fn is_first_frame(&self) -> bool;

    /// Get the complete FrameInfo
    fn frame_info(&self) -> FrameInfo;
}

impl FrameExt for Frame<'_> {
    fn frame_count(&self) -> u64 {
        use_frame().count
    }

    fn delta_time(&self) -> Duration {
        use_frame().delta
    }

    fn frame_timestamp(&self) -> Instant {
        use_frame().timestamp
    }

    fn fps(&self) -> f64 {
        use_frame().fps()
    }

    fn delta_secs(&self) -> f64 {
        use_frame().delta_secs()
    }

    fn delta_millis(&self) -> u128 {
        use_frame().delta_millis()
    }

    fn is_first_frame(&self) -> bool {
        use_frame().is_first_frame()
    }

    fn frame_info(&self) -> FrameInfo {
        use_frame().frame_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_component_id;
    use std::time::{Duration, Instant};

    #[test]
    fn test_frame_info_creation() {
        let now = Instant::now();
        let frame = FrameInfo::new(42, Duration::from_millis(16), now);

        assert_eq!(frame.count, 42);
        assert_eq!(frame.delta, Duration::from_millis(16));
        assert_eq!(frame.timestamp, now);
    }

    #[test]
    fn test_frame_info_fps() {
        let frame = FrameInfo::new(1, Duration::from_millis(16), Instant::now());
        let fps = frame.fps();

        // 16ms = ~62.5 FPS
        assert!((fps - 62.5).abs() < 0.1);
    }

    #[test]
    fn test_frame_info_delta_conversions() {
        let frame = FrameInfo::new(1, Duration::from_millis(16), Instant::now());

        assert_eq!(frame.delta_millis(), 16);
        assert!((frame.delta_secs() - 0.016).abs() < 0.001);
    }

    #[test]
    fn test_frame_info_is_first_frame() {
        let frame0 = FrameInfo::new(0, Duration::from_millis(16), Instant::now());
        let frame1 = FrameInfo::new(1, Duration::from_millis(16), Instant::now());

        assert!(frame0.is_first_frame());
        assert!(!frame1.is_first_frame());
    }

    #[test]
    fn test_frame_context_methods() {
        // Test FrameContext methods without needing a real Frame
        // We can't create a real Frame in tests, so we test the methods directly
        let now = Instant::now();

        // Create a mock frame pointer (never dereferenced in this test)
        let mock_ptr = std::ptr::null_mut();
        let frame_ctx = FrameContext {
            frame_ptr: mock_ptr,
            count: 100,
            delta: Duration::from_millis(16),
            timestamp: now,
        };

        assert_eq!(frame_ctx.count, 100);
        assert_eq!(frame_ctx.delta, Duration::from_millis(16));
        assert_eq!(frame_ctx.timestamp, now);

        // Test helper methods
        let fps = frame_ctx.fps();
        assert!((fps - 62.5).abs() < 0.1); // 16ms = ~62.5 FPS

        assert_eq!(frame_ctx.delta_millis(), 16);
        assert!((frame_ctx.delta_secs() - 0.016).abs() < 0.001);
        assert!(!frame_ctx.is_first_frame());
    }

    #[test]
    fn test_frame_context_first_frame() {
        let mock_ptr = std::ptr::null_mut();
        let frame_ctx = FrameContext {
            frame_ptr: mock_ptr,
            count: 0,
            delta: Duration::from_millis(16),
            timestamp: Instant::now(),
        };

        assert!(frame_ctx.is_first_frame());
    }

    #[test]
    fn test_frame_context_fps_calculation() {
        let mock_ptr = std::ptr::null_mut();

        // 60 FPS = ~16.67ms per frame
        let frame_60fps = FrameContext {
            frame_ptr: mock_ptr,
            count: 1,
            delta: Duration::from_micros(16667),
            timestamp: Instant::now(),
        };

        let fps = frame_60fps.fps();
        assert!((fps - 60.0).abs() < 1.0);
    }

    #[test]
    #[should_panic(expected = "Context value for type")]
    fn test_use_frame_without_context_panics() {
        with_component_id("ComponentWithoutContext", |_ctx| {
            let _frame = use_frame();
        });
    }
}
