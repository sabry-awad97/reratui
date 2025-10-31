//! use_watch hook - Watch form field values
//!
//! Filename: use_watch.rs
//! Folder: /crates/reratui-hooks/src/form/
//!
//! Similar to React Hook Form's useWatch, allows watching specific form fields
//! and re-rendering when their values change.

use super::FormHandle;
use crate::{effect::use_effect, state::use_state};

/// Watch a single field value and re-render when it changes
///
/// # Examples
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn MyComponent() -> Element {
///     let form = use_form_context();
///     let email = use_watch(&form, "email");
///     
///     rsx! {
///         <Paragraph>{format!("Email: {}", email)}</Paragraph>
///     }
/// }
/// ```
pub fn use_watch(form: &FormHandle, field_name: &str) -> String {
    let (value, set_value) = use_state(|| form.get_value(field_name).unwrap_or_default());

    // Watch for changes to the field
    use_effect(
        {
            let form = form.clone();
            let field_name = field_name.to_string();
            let set_value = set_value.clone();
            let prev_value = value.clone();

            move || {
                let current_value = form.get_value(&field_name).unwrap_or_default();
                if current_value != prev_value.get() {
                    set_value.set(current_value);
                }
                None::<Box<dyn FnOnce() + Send>>
            }
        },
        value.get().clone(),
    );

    value.get().clone()
}

/// Watch multiple field values and re-render when any of them change
///
/// # Examples
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn MyComponent() -> Element {
///     let form = use_form_context();
///     let values = use_watch_multiple(&form, &["email", "username"]);
///     
///     rsx! {
///         <Paragraph>{format!("Email: {}, Username: {}",
///             values.get("username").unwrap_or(&String::new())
///         )}</Paragraph>
///     }
/// }
/// ```
pub fn use_watch_multiple(
    form: &FormHandle,
    field_names: &[&str],
) -> std::collections::HashMap<String, String> {
    let (values, set_values) = use_state(|| {
        let mut map = std::collections::HashMap::new();
        for name in field_names {
            map.insert(name.to_string(), form.get_value(name).unwrap_or_default());
        }
        map
    });

    // Watch for changes to any field
    use_effect(
        {
            let form = form.clone();
            let field_names: Vec<String> = field_names.iter().map(|s| s.to_string()).collect();
            let set_values = set_values.clone();
            let prev_values = values.clone();

            move || {
                let mut changed = false;
                let mut new_values = prev_values.get().clone();

                for name in &field_names {
                    let current_value = form.get_value(name).unwrap_or_default();
                    if new_values.get(name) != Some(&current_value) {
                        new_values.insert(name.clone(), current_value);
                        changed = true;
                    }
                }

                if changed {
                    set_values.set(new_values);
                }
                None::<Box<dyn FnOnce() + Send>>
            }
        },
        values.get().len(),
    );

    values.get().clone()
}

/// Watch all form values and re-render when any value changes
/// # Examples
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn FormDebugger() -> Element {
///     let form = use_form_context();
///     let all_values = use_watch_all(&form);
///     
///     rsx! {
///         <Block title={"Form Values"}>
///             {all_values.iter().map(|(key, value)| {
///                 rsx! {
///                     <Paragraph>{format!("{}: {}", key, value)}</Paragraph>
///                 }
///             })}
///         </Block>
///     }
/// }
/// ```
pub fn use_watch_all(form: &FormHandle) -> std::collections::HashMap<String, String> {
    let (values, set_values) = use_state(|| form.get_all_values());

    // Watch for changes to any field
    use_effect(
        {
            let form = form.clone();
            let set_values = set_values.clone();
            let prev_values = values.clone();

            move || {
                let current_values = form.get_all_values();
                if current_values != prev_values.get() {
                    set_values.set(current_values);
                }
                None::<Box<dyn FnOnce() + Send>>
            }
        },
        values.get().len(),
    );

    values.get().clone()
}

/// Hook to watch a field and get a callback when it changes
///
/// # Examples
///
/// ```rust,ignore
/// use reratui::prelude::*;
///
/// #[component]
/// fn MyComponent() -> Element {
///     let form = use_form_context();
///     
///     use_watch_callback(&form, "email", |value| {
///         // Do something when email changes
///         println!("Email changed to: {}", value);
///     });
///     
///     rsx! { /* ... */ }
/// }
/// ```
pub fn use_watch_callback<F>(form: &FormHandle, field_name: &str, callback: F)
where
    F: Fn(&str) + 'static,
{
    let (prev_value, set_prev_value) = use_state(|| form.get_value(field_name).unwrap_or_default());

    use_effect(
        {
            let form = form.clone();
            let field_name = field_name.to_string();
            let set_prev_value = set_prev_value.clone();
            let prev = prev_value.clone();

            move || {
                let current_value = form.get_value(&field_name).unwrap_or_default();
                if current_value != prev.get() {
                    callback(&current_value);
                    set_prev_value.set(current_value);
                }
                None::<Box<dyn FnOnce() + Send>>
            }
        },
        prev_value.get().clone(),
    );
}
