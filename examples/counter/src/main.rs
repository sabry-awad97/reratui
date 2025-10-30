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

/// A beautiful, interactive Button component with hover effect
#[component]
fn Button(props: &ButtonProps) -> Element {
    let label = props.label.clone();
    let area = use_area();
    let (is_hovered, set_hovered) = use_state(|| false);

    // Handle mouse events
    if let Some(event) = use_event()
        && let Event::Mouse(mouse) = event
    {
        let click_x = mouse.column;
        let click_y = mouse.row;

        let in_bounds = click_x >= area.x
            && click_x < area.x + area.width
            && click_y >= area.y
            && click_y < area.y + area.height;

        // Update hover state
        if in_bounds && !is_hovered.get() {
            set_hovered.set(true);
        } else if !in_bounds && is_hovered.get() {
            set_hovered.set(false);
        }

        // Handle click
        if in_bounds
            && mouse.kind == MouseEventKind::Down(MouseButton::Left)
            && let Some(callback) = &props.on_click
        {
            callback.emit(());
        }
    }

    // Beautiful styling with hover effect
    let base_style = props.style.unwrap_or_else(|| {
        Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(59, 130, 246)) // Blue-500
    });

    let hover_style = Style::default()
        .fg(Color::White)
        .bg(Color::Rgb(37, 99, 235)) // Blue-600
        .add_modifier(Modifier::BOLD);

    let style = if is_hovered.get() {
        hover_style
    } else {
        base_style
    };

    let border_style = if is_hovered.get() {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Blue)
    };

    // Add visual indicator for hover
    let display_text = if is_hovered.get() {
        format!("▶ {} ◀", label)
    } else {
        format!("  {}  ", label)
    };

    rsx! {
        <Block
            borders={Borders::ALL}
            border_style={border_style}
            style={style}
        >
            <Paragraph alignment={Alignment::Center}>
                {display_text}
            </Paragraph>
        </Block>
    }
}

#[component]
fn Counter() -> Element {
    let (count, set_count) = use_state(|| 0);
    let component_area = use_area();
    let mut frame_ctx = use_frame();

    // Process keyboard events
    if let Some(Event::Key(key)) = use_event()
        && key.is_press()
        && let KeyCode::Char('q') = key.code
    {
        request_exit()
    }

    let increment = {
        let set_count = set_count.clone();
        move |_| {
            set_count.update(|prev| prev + 1);
        }
    };

    let count = count.get();

    let decrement = {
        let set_count = set_count.clone();
        move |_| {
            if count > 0 {
                set_count.update(|prev| prev - 1);
            }
        }
    };

    let reset = {
        let set_count = set_count.clone();
        move |_| {
            set_count.set(0);
        }
    };

    // Beautiful gradient title
    let title = format!("✨ Counter App | FPS: {:.1}", frame_ctx.fps());

    // Access the ratatui Frame directly!
    let frame = frame_ctx.frame_mut();
    let frame_area = frame.area();

    // Return a layout that renders all components
    rsx! {
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(3),   // Header
                Constraint::Min(8),      // Counter display - takes remaining space
                Constraint::Length(5),   // Buttons - compact height
                Constraint::Length(3),   // Stats footer
            ]}
        >
            <Block
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)}
                style={Style::default().bg(Color::Rgb(17, 24, 39))}
            >
                <Paragraph alignment={Alignment::Center}>
                    {title}
                </Paragraph>
            </Block>

            <Block
                title="Current Count"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)}
                style={Style::default().bg(Color::Rgb(31, 41, 55))}
            >
                <Paragraph
                    alignment={Alignment::Center}
                    style={Style::default()
                        .fg(Color::Rgb(147, 197, 253))
                        .add_modifier(Modifier::BOLD)}
                >
                    {format!("╔═══════════════════╗\n║   Count: {:>6}   ║\n╚═══════════════════╝", count)}
                </Paragraph>
            </Block>

            <Block
                title="Actions"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)}
                style={Style::default().bg(Color::Rgb(17, 24, 39))}
            >
                <Layout
                    direction={Direction::Horizontal}
                    constraints={vec![
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                        Constraint::Percentage(33),
                    ]}
                >
                    <Button
                        label="➖ Decrement"
                        on_click={decrement}
                        style={Style::default().fg(Color::White).bg(Color::Rgb(239, 68, 68))}
                    />
                    <Button
                        label="🔄 Reset"
                        on_click={reset}
                        style={Style::default().fg(Color::White).bg(Color::Rgb(234, 179, 8))}
                    />
                    <Button
                        label="➕ Increment"
                        on_click={increment}
                        style={Style::default().fg(Color::White).bg(Color::Rgb(34, 197, 94))}
                    />
                </Layout>
            </Block>

            <Block
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::DarkGray)}
                style={Style::default().bg(Color::Rgb(17, 24, 39))}
            >
                <Paragraph alignment={Alignment::Center}>
                    {format!(
                        "Frame #{} | Delta: {:.2}ms | Terminal: {}x{} | Component: {}x{}",
                        frame_ctx.count,
                        frame_ctx.delta_millis(),
                        frame_area.width,
                        frame_area.height,
                        component_area.width,
                        component_area.height
                    )}
                </Paragraph>
            </Block>
        </Layout>
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
