//! FormLabel Component - shadcn/ui inspired
//!
//! Filename: form_label.rs
//! Folder: /examples/form_example/src/
//!
//! A reusable label component for form fields.
//! Follows shadcn/ui design principles with TUI adaptations.

use reratui::prelude::*;

#[derive(Props)]
pub struct FormLabelProps {
    /// Label text
    pub text: String,

    /// Whether the field is required
    pub required: Option<bool>,

    /// Custom style
    pub style: Option<Style>,
}

/// Reusable FormLabel component
/// # Example
///
/// ```rust,no_run
/// use reratui::prelude::*;
///
/// #[component]
/// fn MyForm() -> Element {
///     rsx! {
///         <FormLabel
///             text={"Email"}
///             required={true}
///             focused={true}
///         />
///     }
/// }
/// ```
#[component]
pub fn FormLabel(props: &FormLabelProps) -> Element {
    let required = props.required.unwrap_or(false);

    // Try to get field context for state-based styling
    let field_ctx = use_field_context_optional();

    let (focused, error) = if let Some(ctx) = field_ctx {
        (ctx.is_focused, ctx.error.is_some() && ctx.touched)
    } else {
        (false, false)
    };

    // Determine label color based on state
    let color = if error {
        Color::Red
    } else if focused {
        Color::Cyan
    } else {
        Color::Gray
    };

    // Build label text with required indicator
    let label_text = if required {
        format!("{} *", props.text)
    } else {
        props.text.clone()
    };

    // Apply custom style or default
    let label_style = if let Some(custom_style) = props.style {
        custom_style
    } else {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    };

    rsx! {
        <Paragraph style={label_style}>
            {label_text}
        </Paragraph>
    }
}

/// Optional field context hook - returns None if no field context exists
fn use_field_context_optional() -> Option<crate::form_field::FormFieldContext> {
    // Try to get field context, return None if it doesn't exist
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        use_context::<crate::form_field::FormFieldContext>()
    }))
    .ok()
}
