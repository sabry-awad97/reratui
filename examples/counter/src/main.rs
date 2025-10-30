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

/// A reusable Button component with area awareness
#[component]
fn Button(props: &ButtonProps) -> Element {
    let label = props.label.clone();
    let area = use_area();

    // Demonstrate responsive styling based on available space
    let is_compact = area.width < 50;

    let style = props
        .style
        .unwrap_or_else(|| Style::default().fg(Color::Black).bg(Color::Green));

    let display_text = if is_compact {
        // Compact mode: just show first letter
        format!("[{}]", label.chars().next().unwrap_or('?'))
    } else {
        // Normal mode: show full label
        format!("[ {} ]", label)
    };

    rsx! {
        <Block borders={Borders::ALL} style={style}>
            <Paragraph>
                {display_text}
            </Paragraph>
        </Block>
    }
}

#[component]
fn Counter() -> Element {
    let (count, set_count) = use_state(|| 0);
    let component_area = use_area();
    let frame_ctx = use_frame();

    let increment = move |_| {
        set_count.update(|prev| prev + 1);
    };

    // Access the ratatui Frame directly!
    let frame = frame_ctx.frame();
    let frame_area = frame.area();

    // Show component dimensions and frame info in the title
    let title = format!(
        "Counter ({}x{}) | Frame: {} | FPS: {:.1} | Terminal: {}x{}",
        component_area.width,
        component_area.height,
        frame_ctx.count,
        frame_ctx.fps(),
        frame_area.width,
        frame_area.height
    );

    rsx! {
        <Block title={title} borders={Borders::ALL}>
            <Paragraph>
                {format!("Count: {}", count)}
            </Paragraph>
            <Paragraph>
                {format!("Delta: {:.2}ms | Frame #{}", frame_ctx.delta_millis(), frame_ctx.count)}
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
