//! Form management hook with validation
//!
//! Provides a comprehensive form handling solution with:
//! - Field registration and value management
//! - Validation with custom rules
//! - Error tracking per field
//! - Dirty/touched state tracking
//! - Submit handling with async support
//! - Context API for sharing form state without prop drilling

use crate::context::{use_context, use_context_provider};
use crate::state::use_state;
use std::collections::HashMap;

mod types;
mod validation;

#[cfg(test)]
mod tests;

pub use types::*;
pub use validation::*;

/// Form hook for managing form state, validation, and submission
///
/// Automatically provides the form to child components via context,
/// allowing them to access it using `use_form_context()`.
///
/// # Example
///
/// ```rust,no_run
/// use reratui::prelude::*;
///
/// #[component]
/// fn MyForm() -> Element {
///     // Form is automatically provided to child components
///     let form = use_form(
///         FormConfig::builder()
///             .field("email", "")
///             .field("password", "")
///             .validator("email", Validator::required("Email is required"))
///             .validator("email", Validator::email("Invalid email format"))
///             .validator("password", Validator::required("Password is required"))
///             .validator("password", Validator::min_length(8, "Min 8 characters"))
///             .on_submit(|values| {
///                 println!("Form submitted: {:?}", values);
///             })
///             .build()
///     );
///
///     // You can use the form directly
///     let email_reg = form.register("email");
///
///     rsx! {
///         <Block>
///             // Or child components can access it via use_form_context()
///             <FormField field_name={"email"} />
///         </Block>
///     }
/// }
///
/// #[component]
/// fn FormField(field_name: &str) -> Element {
///     // Access form from context - no props needed!
///     let form = use_form_context();
///     let registration = form.register(field_name);
///     
///     rsx! { <></> }
/// }
/// ```
pub fn use_form(config: FormConfig) -> FormHandle {
    let (values, set_values) = use_state(|| config.initial_values.clone());
    let (errors, set_errors) = use_state(HashMap::<String, String>::new);
    let (touched, set_touched) = use_state(HashMap::<String, bool>::new);
    let (is_submitting, set_is_submitting) = use_state(|| false);
    let (is_valid, set_is_valid) = use_state(|| true);

    let form = FormHandle {
        values,
        set_values,
        errors,
        set_errors,
        touched,
        set_touched,
        is_submitting,
        set_is_submitting,
        is_valid,
        set_is_valid,
        validators: config.validators,
        on_submit: config.on_submit,
    };

    // Automatically provide form to child components
    use_context_provider(|| form.clone());

    form
}

/// Retrieves the form context from a parent component
///
/// This hook allows child components to access the form state without
/// having to pass it through props. The form is automatically provided
/// by `use_form()` in a parent component.
///
/// Similar to React Hook Form's `useFormContext`.
///
/// # Panics
///
/// Panics if called outside of a component that has a `use_form()` ancestor.
///
/// # Example
///
/// ```rust,no_run
/// use reratui::prelude::*;
///
/// #[component]
/// fn ParentForm() -> Element {
///     // Form is automatically provided to children
///     let _form = use_form(
///         FormConfig::builder()
///             .field("username", "")
///             .on_submit(|_| {})
///             .build()
///     );
///
///     rsx! {
///         <FormField field_name={"username"} />
///     }
/// }
///
/// #[component]
/// fn FormField(field_name: &str) -> Element {
///     // Access form from context - no props needed!
///     let form = use_form_context();
///     let registration = form.register(field_name);
///     
///     rsx! {
///         <Block>
///             <Paragraph>{registration.value}</Paragraph>
///         </Block>
///     }
/// }
/// ```
pub fn use_form_context() -> FormHandle {
    use_context::<FormHandle>()
}
