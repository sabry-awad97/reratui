//! Input Component - shadcn/ui inspired
//!
//! Filename: input.rs
//! Folder: /crates/reratui-components/src/
//!
//! A reusable, accessible input component with variants and states.
//! Follows shadcn/ui design principles with TUI adaptations.

use reratui::prelude::*;

/// Input variant styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputVariant {
    /// Default input style
    Default,
    /// Outlined input style
    Outlined,
}

impl Default for InputVariant {
    fn default() -> Self {
        Self::Default
    }
}

/// Input size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSize {
    /// Medium input (height: 4) - default
    Md,
}

impl Default for InputSize {
    fn default() -> Self {
        Self::Md
    }
}

/// Input state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputState {
    /// Normal state
    Normal,
    /// Focused state
    Focused,
    /// Error state
    Error,
    /// Success state
    Success,
    /// Disabled state
    Disabled,
}

impl Default for InputState {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Props)]
pub struct FormInputProps {
    /// Placeholder text
    pub placeholder: Option<String>,

    /// Input variant
    pub variant: Option<InputVariant>,

    /// Input size
    pub size: Option<InputSize>,

    /// Whether the input is disabled
    pub disabled: Option<bool>,

    /// Helper text
    pub helper_text: Option<String>,

    /// Whether to show as password (masked)
    pub password: Option<bool>,

    /// Icon to display (prefix)
    pub icon: Option<String>,

    /// Custom class/style modifier
    pub class: Option<String>,
}

/// Reusable Input component with shadcn/ui styling
/// Should be wrapped in FormField for form integration.
///
/// # Example
///
/// ```rust,no_run
/// use reratui::prelude::*;
/// fn MyForm() -> Element {
///     rsx! {
///         <FormField name={"email"} field_index={0}>
///             <FormInput
///                 placeholder={"Enter your email"}
///                 variant={InputVariant::Outlined}
///                 icon={"📧"}
///             />
///         </FormField>
///     }
/// }
/// ```
#[component]
pub fn FormInput(props: &FormInputProps) -> Element {
    // Get field context from FormField (if wrapped)
    let field_ctx = use_field_context_optional();

    // Get input state
    let variant = props.variant.unwrap_or_default();
    let disabled = props.disabled.unwrap_or(false);
    let is_password = props.password.unwrap_or(false);

    // Get value, error, touched, and focus from field context
    let (value, error, touched, is_focused) = if let Some(ctx) = field_ctx {
        (ctx.value, ctx.error, ctx.touched, ctx.is_focused)
    } else {
        (String::new(), None, false, false)
    };

    // Determine input state
    let state = if disabled {
        InputState::Disabled
    } else if is_focused {
        InputState::Focused
    } else if error.is_some() && touched {
        InputState::Error
    } else if touched && !value.is_empty() {
        InputState::Success
    } else {
        InputState::Normal
    };

    // Get colors based on state and variant
    let (border_color, fg_color, bg_color) = get_input_colors(state, variant);

    // Display value (masked if password)
    let display_value = if is_password && !value.is_empty() {
        "•".repeat(value.len())
    } else if value.is_empty() {
        props.placeholder.clone().unwrap_or_default()
    } else {
        value.clone()
    };

    // Value style
    let value_style = if value.is_empty() {
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(fg_color)
    };

    rsx! {
        <Block
            borders={Borders::ALL}
            border_style={Style::default().fg(border_color)}
            style={Style::default().bg(bg_color)}
        >
            // Icon and value row
            <Paragraph style={value_style}>
                {if let Some(ref icon) = props.icon {
                    format!("{} {}", icon, display_value)
                } else {
                    display_value
                }}
            </Paragraph>
        </Block>
    }
}

/// Helper function to get colors based on state and variant
fn get_input_colors(state: InputState, variant: InputVariant) -> (Color, Color, Color) {
    match (state, variant) {
        // Focused state
        (InputState::Focused, InputVariant::Default) => {
            (Color::Cyan, Color::White, Color::Rgb(30, 30, 46))
        }
        (InputState::Focused, InputVariant::Outlined) => {
            (Color::Cyan, Color::White, Color::Rgb(17, 17, 27))
        }
        // Error state
        (InputState::Error, _) => (Color::Red, Color::White, Color::Rgb(30, 30, 46)),

        // Success state
        (InputState::Success, _) => (Color::Green, Color::White, Color::Rgb(30, 30, 46)),

        // Disabled state
        (InputState::Disabled, _) => (Color::DarkGray, Color::DarkGray, Color::Rgb(20, 20, 30)),

        // Normal state
        (InputState::Normal, InputVariant::Default) => {
            (Color::DarkGray, Color::White, Color::Rgb(30, 30, 46))
        }
        (InputState::Normal, InputVariant::Outlined) => {
            (Color::Gray, Color::White, Color::Rgb(17, 17, 27))
        }
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
