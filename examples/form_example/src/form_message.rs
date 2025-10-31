//! FormMessage Component - shadcn/ui inspired
//!
//! Filename: form_message.rs
//! Folder: /examples/form_example/src/
//!
//! A reusable message component for displaying errors and helper text.
//! Follows shadcn/ui design principles with TUI adaptations.

use reratui::prelude::*;

/// Message variant types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageVariant {
    /// Error message (red)
    Error,
    /// Helper text (gray)
    Helper,
}

impl Default for MessageVariant {
    fn default() -> Self {
        Self::Helper
    }
}

#[derive(Props)]
pub struct FormMessageProps {
    /// Message text
    pub text: String,

    /// Message variant
    pub variant: Option<MessageVariant>,

    /// Custom style
    pub style: Option<Style>,
}

/// Reusable FormMessage component
///
/// # Example
///
/// ```rust,no_run
/// use reratui::prelude::*;
///
/// #[component]
/// fn MyForm() -> Element {
///     rsx! {
///         <FormMessage
///             text={"This field is required"}
///             variant={MessageVariant::Error}
///         />
///     }
/// }
/// ```
#[component]
pub fn FormMessage(props: &FormMessageProps) -> Element {
    let variant = props.variant.unwrap_or_default();

    // Determine message style based on variant
    let (color, icon) = match variant {
        MessageVariant::Error => (Color::Red, "⚠"),
        MessageVariant::Helper => (Color::DarkGray, ""),
    };

    // Build message text with icon
    let message_text = if icon.is_empty() {
        props.text.clone()
    } else {
        format!("{} {}", icon, props.text)
    };

    // Apply custom style or default
    let message_style = if let Some(custom_style) = props.style {
        custom_style
    } else {
        let mut style = Style::default().fg(color);

        // Add modifiers based on variant
        style = match variant {
            MessageVariant::Error => style.add_modifier(Modifier::ITALIC),
            MessageVariant::Helper => style.add_modifier(Modifier::DIM),
        };

        style
    };

    rsx! {
        <Paragraph style={message_style}>
            {message_text}
        </Paragraph>
    }
}

/// FormMessage component that automatically reads from FormFieldContext
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
///             <FormInput placeholder={"Enter email"} />
///             <FormFieldMessage />  // Automatically shows error if present
///         </FormField>
///     }
/// }
/// ```
#[component]
pub fn FormFieldMessage() -> Element {
    // Get field context
    let field_ctx = use_field_context_optional();

    if let Some(ctx) = field_ctx {
        // Show error if present and touched
        if let Some(error) = ctx.error {
            if ctx.touched {
                return rsx! {
                    <FormMessage
                        text={error}
                        variant={MessageVariant::Error}
                    />
                };
            }
        }
    }

    // No message to display
    rsx! { <></> }
}

/// Optional field context hook - returns None if no field context exists
fn use_field_context_optional() -> Option<crate::form_field::FormFieldContext> {
    // Try to get field context, return None if it doesn't exist
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        use_context::<crate::form_field::FormFieldContext>()
    }))
    .ok()
}
