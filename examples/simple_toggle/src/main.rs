use reratui::prelude::*;

/// A simple toggle button component - like React's useState for booleans
#[component]
fn ToggleButton(label: String) -> Element {
    // useState equivalent - starts as false
    let (is_toggled, set_is_toggled) = use_state(|| false);
    let toggled = is_toggled.get();

    // Handle space key to toggle
    if let Some(event) = use_event()
        && let Event::Key(key) = event
        && key.kind == KeyEventKind::Press
        && key.code == KeyCode::Char(' ')
    {
        // Toggle the state: !prevState
        set_is_toggled.update(|prev| !prev);
    }

    rsx! {
        <Block
            title={label}
            borders={Borders::ALL}
            border_style={
                if toggled {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Red)
                }
            }
        >
            <Layout
                direction={Direction::Vertical}
                constraints={vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ]}
            >
                <Paragraph
                    style={
                        if toggled {
                            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                        }
                    }
                    alignment={Alignment::Center}
                >
                    {
                        if toggled {
                            "✅ ENABLED"
                        } else {
                            "❌ DISABLED"
                        }
                    }
                </Paragraph>
                <Paragraph
                    style={Style::default().fg(Color::Yellow)}
                    alignment={Alignment::Center}
                >
                    {format!("State: {}", toggled)}
                </Paragraph>
                <Paragraph
                    style={Style::default().fg(Color::Gray)}
                    alignment={Alignment::Center}
                >
                    {"Press SPACE to toggle"}
                </Paragraph>
            </Layout>
        </Block>
    }
}

/// A counter with increment/decrement toggle
#[component]
fn CounterToggle() -> Element {
    // Multiple useState hooks
    let (count, set_count) = use_state(|| 0);
    let (increment_mode, set_increment_mode) = use_state(|| true);

    let current_count = count.get();
    let is_increment = increment_mode.get();

    // Handle arrow keys and 'm' for mode toggle
    if let Some(event) = use_event()
        && let Event::Key(key) = event
        && key.kind == KeyEventKind::Press
    {
        match key.code {
            KeyCode::Up => {
                if is_increment {
                    set_count.update(|prev| prev + 1);
                } else {
                    set_count.update(|prev| prev - 1);
                }
            }
            KeyCode::Down => {
                if is_increment {
                    set_count.update(|prev| prev - 1);
                } else {
                    set_count.update(|prev| prev + 1);
                }
            }
            KeyCode::Char('m') => {
                // Toggle increment/decrement mode
                set_increment_mode.update(|prev| !prev);
            }
            _ => {}
        }
    }

    rsx! {
        <Block
            title="🔢 Counter Toggle"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Cyan)}
        >
            <Layout
                direction={Direction::Vertical}
                constraints={vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ]}
            >
                <Paragraph
                    style={Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)}
                    alignment={Alignment::Center}
                >
                    {format!("Count: {}", current_count)}
                </Paragraph>
                <Paragraph
                    style={
                        if is_increment {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::Red)
                        }
                    }
                    alignment={Alignment::Center}
                >
                    {
                        if is_increment {
                            "📈 Increment Mode"
                        } else {
                            "📉 Decrement Mode"
                        }
                    }
                </Paragraph>
                <Paragraph
                    style={Style::default().fg(Color::Yellow)}
                    alignment={Alignment::Center}
                >
                    {"↑/↓: Count"}
                </Paragraph>
                <Paragraph
                    style={Style::default().fg(Color::Gray)}
                    alignment={Alignment::Center}
                >
                    {"m: Toggle mode"}
                </Paragraph>
            </Layout>
        </Block>
    }
}

/// Main app component
struct SimpleToggleApp;

impl SimpleToggleApp {
    fn new() -> Self {
        Self
    }
}

impl Component for SimpleToggleApp {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Handle quit
        if let Some(event) = use_event()
            && let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
            && key.code == KeyCode::Char('q')
        {
            request_exit();
        }

        let layout = rsx! {
            <Layout
                direction={Direction::Vertical}
                margin={1}
                constraints={vec![
                    Constraint::Length(3),  // Header
                    Constraint::Min(0),     // Content
                    Constraint::Length(3),  // Footer
                ]}
            >
                {/* Header */}
                <Block
                    title="🔄 Simple Toggle Demo"
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)}
                >
                    <Paragraph
                        alignment={Alignment::Center}
                        style={Style::default().fg(Color::White).add_modifier(Modifier::BOLD)}
                    >
                        {"useState for Boolean Toggles"}
                    </Paragraph>
                </Block>

                {/* Main content */}
                <Layout
                    direction={Direction::Horizontal}
                    constraints={vec![
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ]}
                >
                    <ToggleButton label={"🔘 Toggle Switch".to_string()} />
                    <CounterToggle />
                </Layout>

                {/* Footer */}
                <Block
                    title="📋 Controls"
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Yellow)}
                >
                    <Paragraph
                        alignment={Alignment::Center}
                        style={Style::default().fg(Color::Yellow)}
                    >
                        {"SPACE: Toggle • ↑/↓: Count • m: Mode • q: Quit"}
                    </Paragraph>
                </Block>
            </Layout>
        };

        layout.render(area, buffer);
    }
}

/// Entry point - demonstrates simple useState toggle patterns
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Simple Toggle Demo");
    println!("Demonstrates useState patterns for boolean toggles");
    println!("Controls: SPACE, ↑/↓, m, q\n");

    if let Err(err) = render(|| SimpleToggleApp::new().into()).await {
        eprintln!("❌ Error: {:?}", err);
    } else {
        println!("✨ Simple toggle demo completed!");
    }

    Ok(())
}
