//! FormDescription Component - shadcn/ui inspired
//!
//! Filename: form_description.rs
//! Folder: /examples/form_example/src/
//!
//! A reusable description/helper text component for form fields.
//! Follows shadcn/ui design principles with TUI adaptations.

use reratui::prelude::*;

#[derive(Props)]
pub struct FormDescriptionProps {
    /// Description text
    pub text: String,

    /// Custom style
    pub style: Option<Style>,
}

/// Reusable FormDescription component for helper text
///
/// # Example
///
/// ```rust,no_run
/// use reratui::prelude::*;
///
/// #[component]
/// fn MyForm() -> Element {
///     rsx! {
///         <FormField name={"email"} field_index={0}>
///             <FormLabel text={"Email"} required={true} />
///             <FormInput placeholder={"Enter your email"} />
///             <FormDescription text={"We'll never share your email."} />
///             <FormFieldMessage />
///         </FormField>
///     }
/// }
/// ```
#[component]
pub fn FormDescription(props: &FormDescriptionProps) -> Element {
    // Apply custom style or default helper text style
    let description_style = if let Some(custom_style) = props.style {
        custom_style
    } else {
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM)
    };

    rsx! {
        <Paragraph style={description_style}>
            {props.text.clone()}
        </Paragraph>
    }
}
