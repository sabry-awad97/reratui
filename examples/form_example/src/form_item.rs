use reratui::prelude::*;

#[derive(Props)]
pub struct FormItemProps {
    /// Child components (Label, Input, Description, Message)
    pub children: Vec<Element>,
}

#[component]
pub fn FormItem(props: &FormItemProps) -> Element {
    // Get field context from FormField
    let field_ctx = use_context::<crate::form_field::FormFieldContext>();

    // Extract values from context
    let error = field_ctx.error.clone();
    let touched = field_ctx.touched;

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
