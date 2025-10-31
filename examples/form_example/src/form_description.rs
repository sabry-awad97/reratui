use reratui::prelude::*;

#[derive(Props)]
pub struct FormDescriptionProps {
    /// Description text
    pub text: String,

    /// Custom style
    pub style: Option<Style>,
}

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
