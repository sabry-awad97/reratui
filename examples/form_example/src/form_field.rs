use reratui::prelude::*;

#[derive(Props)]
pub struct FormFieldProps {
    /// Field name in the form
    pub name: String,

    /// Render callback for custom layout
    pub render: Callback<FormFieldContext, Element>,
}

#[component]
pub fn FormField(props: &FormFieldProps) -> Element {
    // Access form from context
    let form = use_form_context();

    // Access field registry and auto-register this field
    let registry = use_context::<crate::form::FieldRegistry>();
    let field_index = registry.register_field(&props.name);

    // Check if this field is focused
    let is_focused = registry.get_focused_index() == field_index;

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

    // Create field context
    let field_context = FormFieldContext {
        value,
        error: error.clone(),
        touched,
        is_focused,
    };

    // Provide context to children
    use_context_provider(|| field_context.clone());

    // Use render callback to generate the UI
    props.render.emit(field_context)
}

/// Context data provided by FormField to Input
#[derive(Clone)]
pub struct FormFieldContext {
    pub value: String,
    pub error: Option<String>,
    pub touched: bool,
    pub is_focused: bool,
}
