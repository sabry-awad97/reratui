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
pub struct InputProps {
    /// Input value
    pub value: Option<String>,

    /// Placeholder text
    pub placeholder: Option<String>,

    /// Input variant
    pub variant: Option<InputVariant>,

    /// Input size
    pub size: Option<InputSize>,

    /// Whether the input is disabled
    pub disabled: Option<bool>,

    /// Whether the input is focused
    pub focused: Option<bool>,

    /// Whether the input has an error
    pub error: Option<bool>,

    /// Helper text
    pub helper_text: Option<String>,

    /// Whether to show as password (masked)
    pub password: Option<bool>,

    /// Icon to display (prefix)
    pub icon: Option<String>,

    /// Custom class/style modifier
    pub class: Option<String>,
}

#[component]
pub fn Input(props: &InputProps) -> Element {
    // Get input state from props
    let variant = props.variant.unwrap_or_default();
    let disabled = props.disabled.unwrap_or(false);
    let is_password = props.password.unwrap_or(false);
    let is_focused = props.focused.unwrap_or(false);
    let has_error = props.error.unwrap_or(false);
    let value = props.value.clone().unwrap_or_default();

    // Determine input state
    let state = if disabled {
        InputState::Disabled
    } else if is_focused {
        InputState::Focused
    } else if has_error {
        InputState::Error
    } else if !value.is_empty() {
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
