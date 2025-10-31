//! FormField Component - shadcn/ui inspired
//!
//! Filename: form_field.rs
//! Folder: /examples/form_example/src/
//!
//! A wrapper component that provides form context to Input components.
//! Follows shadcn/ui composition pattern: <FormField><Input /></FormField>

use reratui::prelude::*;

#[derive(Props)]
pub struct FormFieldProps {
    /// Field name in the form
    pub name: String,

    /// Field index for focus management
    pub field_index: usize,

    /// The input component to render
    pub children: Vec<Element>,
}

/// FormField component that provides form integration to child Input
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
///             <Input
///                 label={"Email"}
///                 placeholder={"Enter your email"}
///                 variant={InputVariant::Outlined}
///             />
///         </FormField>
///     }
/// }
/// ```
#[component]
pub fn FormField(props: &FormFieldProps) -> Element {
    // Access form from context
    let form = use_form_context();

    // Access focused field from context
    let focused_field = use_context::<StateHandle<usize>>();

    // Check if this field is focused
    let is_focused = focused_field.get() == props.field_index;

    // Get form state for this field
    let value = form.get_value(&props.name).unwrap_or_default();
    let error = form.get_error(&props.name);
    let touched = form.is_touched(&props.name);

    // Handle keyboard input if focused
    let event = use_event();
    if is_focused {
        if let Some(Event::Key(key)) = event {
            if key.is_press() {
                match key.code {
                    KeyCode::Char(c) => {
                        let mut new_value = value.clone();
                        new_value.push(c);
                        form.set_value(&props.name, new_value);
                        form.set_touched(&props.name, true);
                    }
                    KeyCode::Backspace => {
                        let mut new_value = value.clone();
                        new_value.pop();
                        form.set_value(&props.name, new_value);
                        form.set_touched(&props.name, true);
                    }
                    KeyCode::Enter => {
                        form.submit();
                    }
                    _ => {}
                }
            }
        }
    }

    // Create a context provider for field-specific data
    use_context_provider(|| FormFieldContext {
        value,
        error: error.clone(),
        touched,
        is_focused,
    });

    // Render the child input with context
    // Dynamically allocate space based on what's present
    let child_count = props.children.len();
    let has_error_to_show = error.is_some() && touched;

    let constraints = match (child_count, has_error_to_show) {
        // 4 children (label, input, description, message) with error
        (4, true) => vec![
            Constraint::Length(1), // Label
            Constraint::Min(3),    // Input
            Constraint::Length(1), // Description
            Constraint::Length(1), // Message (error visible)
        ],
        // 4 children without error - description visible, no message space needed
        (4, false) => vec![
            Constraint::Length(1), // Label
            Constraint::Min(3),    // Input
            Constraint::Length(1), // Description
            Constraint::Length(0), // Message (hidden)
        ],
        // 3 children (label, input, message) with error
        (3, true) => vec![
            Constraint::Length(1), // Label
            Constraint::Min(3),    // Input
            Constraint::Length(1), // Message (error visible)
        ],
        // 3 children without error - no message space needed
        (3, false) => vec![
            Constraint::Length(1), // Label
            Constraint::Min(3),    // Input
            Constraint::Length(0), // Message (hidden)
        ],
        // Fallback for any other case
        _ => vec![
            Constraint::Length(1), // Label
            Constraint::Min(3),    // Input
            Constraint::Length(1), // Message
        ],
    };

    rsx! {
        <Layout
            direction={Direction::Vertical}
            constraints={constraints}
        >
            {props.children.clone()}
        </Layout>
    }
}

/// Context data provided by FormField to Input
#[derive(Clone)]
pub struct FormFieldContext {
    pub value: String,
    pub error: Option<String>,
    pub touched: bool,
    pub is_focused: bool,
}
