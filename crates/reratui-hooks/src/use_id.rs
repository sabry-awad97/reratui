//! use_id hook - Generates unique IDs for components
//!
//! Filename: use_id.rs
//! Folder: /crates/reratui-hooks/src/
//!
//! Similar to React's useId hook, generates stable unique IDs for accessibility
//! and component identification.

use crate::state::use_state;
use uuid::Uuid;

/// Generates a unique, stable ID for the component
///
/// The ID is generated once per component instance and remains stable across re-renders.
/// This hook is implemented using `use_state` internally, ensuring the ID persists
/// across component re-renders.
///
/// This is useful for:
/// - Accessibility attributes (aria-labelledby, aria-describedby)
/// - Form field associations
/// - Component identification
///
/// # Examples
///
/// ## Basic usage
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn MyComponent() -> Element {
///     let id = use_id();
///     
///     rsx! {
///         <Block title={format!("Component {}", id)}>
///             <Paragraph>{format!("My unique ID is: {}", id)}</Paragraph>
///         </Block>
///     }
/// }
/// ```
///
/// ## With use_state (multiple hooks)
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn Counter() -> Element {
///     let id = use_id(); // Stable across re-renders
///     let (count, set_count) = use_state(|| 0);
///     
///     rsx! {
///         <Block title={format!("Counter {}", id)}>
///             <Paragraph>{format!("Count: {}", count)}</Paragraph>
///         </Block>
///     }
/// }
/// ```
///
/// # Returns
///
/// Returns a `String` containing a UUID that is unique and stable across re-renders.
pub fn use_id() -> String {
    // Use use_state to store the ID, initialized with a new UUID
    let (id, _) = use_state(Uuid::now_v7);
    id.get().to_string()
}

/// Generates a unique ID with a custom prefix
///
/// This is useful when you want more readable IDs for debugging or logging.
///
/// # Examples
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn FormField() -> Element {
///     let field_id = use_id_with_prefix("field");
///     
///     rsx! {
///         <Block title={field_id}>
///             <Paragraph>{"Field content"}</Paragraph>
///         </Block>
///     }
/// }
/// ```
pub fn use_id_with_prefix(prefix: &str) -> String {
    let id = use_id();
    format!("{}_{}", prefix, id)
}
