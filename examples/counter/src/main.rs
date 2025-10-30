//! Counter Example with RSX and use_state Hook
//!
//! A simple counter application demonstrating the Reratui framework with:
//! - RSX syntax for declarative UI
//! - use_state hook for reactive state management
//! - Automatic re-rendering on state changes
//!
//! Press any key to increment the counter.
//! Press Esc or Ctrl+C to exit.

use reratui::prelude::*;

/// Props for the Button component
#[derive(Props)]
struct ButtonProps {
    /// Button label text
    label: String,
    /// Optional click handler callback
    on_click: Option<Callback<()>>,
    /// Optional styling
    style: Option<Style>,
}

/// A reusable Button component
///
/// # Examples
/// ```rust,no_run
/// rsx! {
///     <Button label="Click Me" on_click={|| println!("Clicked!")} />
/// }
/// ```
#[component]
fn Button(props: &ButtonProps) -> Element {
    let label = props.label.clone();
    let style = props
        .style
        .unwrap_or_else(|| Style::default().fg(Color::Black).bg(Color::Green));

    rsx! {
        <Block borders={Borders::ALL} style={style}>
            <Paragraph>
                {format!("[ {} ]", label)}
            </Paragraph>
        </Block>
    }
}

#[component]
fn Counter() -> Element {
    let (count, set_count) = use_state(|| 0);

    let increment = move |_| {
        set_count.update(|prev| prev + 1);
    };

    rsx! {
        <Block title="Counter" borders={Borders::ALL}>
            <Paragraph>
                {format!("Count: {}", count)}
            </Paragraph>
            <Button
                label="Increment"
                on_click={increment}
            />
        </Block>
    }
}

#[reratui::main]
async fn main() -> Result<()> {
    render(|| {
        rsx! { <Counter /> }
    })
    .await?;
    Ok(())
}
