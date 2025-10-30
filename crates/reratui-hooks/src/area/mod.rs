//! Component Area Hook - Access to component's render area
//!
//! This module provides a professional context-based API for accessing the current
//! component's render area (Rect). The renderer provides the area via context,
//! and components can consume it using `use_area()`.
//!
//! # Architecture
//!
//! - Renderer provides area via `use_context_provider`
//! - Components consume area via `use_area()` (which calls `use_context`)
//! - Clean separation of concerns following SOLID principles
//!
//! # Use Cases
//!
//! - Layout calculations
//! - Responsive design decisions
//! - Scroll calculations
//! - Mouse event handling within component bounds

use crate::context::use_context;
use ratatui::layout::Rect;
use std::ops::Deref;

/// Context type for component render area
///
/// This is provided by the renderer and consumed by components via `use_area()`.
///
/// Implements `Deref<Target = Rect>` so you can access Rect methods directly:
///
/// ```rust,ignore
/// use reratui::prelude::*;
/// use ratatui::layout::Margin;
///
/// let area = use_area();
/// let width = area.width;  // Direct access to Rect fields
/// let height = area.height;
/// let inner = area.inner(&Margin::new(1, 1));  // Call Rect methods
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentArea(pub Rect);

impl Deref for ComponentArea {
    type Target = Rect;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Hook to access the current component's render area
///
/// This hook retrieves the component's render area from the context provided
/// by the renderer. The area represents the Rect that the component is being
/// rendered into.
///
/// # Panics
///
/// Panics if called outside of a component render context where the area
/// has been provided by the renderer.
///
/// # Use Cases
///
/// - **Layout calculations**: Determine available space for child components
/// - **Responsive design**: Adjust rendering based on available space
/// - **Positioning**: Calculate absolute positions within the component
/// - **Scroll handling**: Track viewport dimensions for scrollable content
/// - **Mouse events**: Determine if mouse events are within component bounds
///
/// # Examples
///
/// ## Basic Usage - Direct Field Access
///
/// ```rust,ignore
/// use reratui::prelude::*;
/// #[component]
/// fn MyComponent() -> Element {
///     let area = use_area();
///     
///     // Direct access to Rect fields via Deref
///     println!("Width: {}, Height: {}", area.width, area.height);
///     println!("Position: ({}, {})", area.x, area.y);
///     
///     rsx! {
///         <Block title="My Component">
///             <Paragraph>
///                 {format!("Size: {}x{}", area.width, area.height)}
///             </Paragraph>
///         </Block>
///     }
/// }
/// ```
///
/// ## Responsive Layout
///
/// ```rust,ignore
/// use reratui::prelude::*;
/// #[component]
/// fn AdaptiveLayout() -> Element {
///     let area = use_area();
///     
///     // Direct field access - no .0 needed!
///     let layout_mode = if area.width < 80 {
///         "compact"
///     } else {
///         "wide"
///     };
///     
///     rsx! {
///         <Block title={format!("Layout: {}", layout_mode)}>
///             <Paragraph>
///                 {format!("Adaptive content")}
///             </Paragraph>
///         </Block>
///     }
/// }
/// ```
///
/// ## Scroll Viewport
///
/// ```rust,ignore
/// use reratui::prelude::*;
/// #[component]
/// fn ScrollableList() -> Element {
///     let area = use_area();
///     let (scroll_offset, set_scroll) = use_state(|| 0u16);
///     
///     // Calculate visible items based on area height
///     let visible_count = area.height.saturating_sub(2);
///     
///     rsx! {
///         <Block title="Scrollable List">
///             <Paragraph>
///                 {format!("Showing {} items", visible_count)}
///             </Paragraph>
///         </Block>
///     }
/// }
/// ```
///
/// ## Using Rect Methods
///
/// ```rust,ignore
/// use reratui::prelude::*;
/// use ratatui::layout::Margin;
/// #[component]
/// fn LayoutComponent() -> Element {
///     let area = use_area();
///     
///     // Call Rect methods directly
///     let inner = area.inner(&Margin::new(1, 1));
///     let is_empty = area.is_empty();
///     let area_size = area.area();
///     
///     rsx! {
///         <Block title="Layout">
///             <Paragraph>
///                 {format!("Inner: {}x{}", inner.width, inner.height)}
///             </Paragraph>
///         </Block>
///     }
/// }
/// ```
pub fn use_area() -> ComponentArea {
    use_context::<ComponentArea>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::use_context_provider;
    use crate::test_utils::with_component_id;
    use ratatui::layout::Rect;

    #[test]
    fn test_use_area_with_context() {
        // Simulate renderer providing area context
        with_component_id("RendererComponent", |_ctx| {
            let test_rect = Rect::new(10, 20, 100, 50);
            let _area = use_context_provider(|| ComponentArea(test_rect));

            // Simulate child component consuming area
            with_component_id("ChildComponent", |_ctx| {
                let area = use_area();

                // Direct field access via Deref
                assert_eq!(*area, test_rect);
                assert_eq!(area.width, 100);
                assert_eq!(area.height, 50);
                assert_eq!(area.x, 10);
                assert_eq!(area.y, 20);
            });
        });
    }

    #[test]
    fn test_use_area_responsive_layout() {
        // Test responsive behavior with different sizes
        with_component_id("ResponsiveComponent", |_ctx| {
            // Small screen
            let small_rect = Rect::new(0, 0, 60, 20);
            let _area = use_context_provider(|| ComponentArea(small_rect));

            with_component_id("ChildComponent", |_ctx| {
                let area = use_area();
                // Direct field access - no .0 needed!
                let is_compact = area.width < 80;
                assert!(is_compact);
            });
        });

        with_component_id("ResponsiveComponent", |_ctx| {
            // Large screen
            let large_rect = Rect::new(0, 0, 120, 40);
            let _area = use_context_provider(|| ComponentArea(large_rect));

            with_component_id("ChildComponent", |_ctx| {
                let area = use_area();
                // Direct field access
                let is_compact = area.width < 80;
                assert!(!is_compact);
            });
        });
    }

    #[test]
    fn test_use_area_deref() {
        // Test that Deref works correctly
        with_component_id("DerefTestComponent", |_ctx| {
            let test_rect = Rect::new(5, 10, 80, 24);
            let _area = use_context_provider(|| ComponentArea(test_rect));

            with_component_id("ChildComponent", |_ctx| {
                let area = use_area();

                // Test direct field access
                assert_eq!(area.width, 80);
                assert_eq!(area.height, 24);

                // Test calling Rect methods
                let area_size = area.area();
                assert_eq!(area_size, 80 * 24);

                let is_empty = area.is_empty();
                assert!(!is_empty);
            });
        });
    }

    #[test]
    #[should_panic(expected = "Context value for type")]
    fn test_use_area_without_context_panics() {
        // Calling use_area without context should panic
        with_component_id("ComponentWithoutContext", |_ctx| {
            let _area = use_area();
        });
    }
}
