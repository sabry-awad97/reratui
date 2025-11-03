use std::time::Duration;

use reratui::prelude::*;

/// A React-like Counter component that mimics the Ink example
///
/// This component demonstrates:
/// - useState equivalent with use_state
/// - useEffect equivalent with use_interval
/// - Component composition with rsx!
/// - Automatic cleanup on unmount
#[component]
fn Counter() -> Element {
    // useState equivalent - initialize counter to 0
    let (counter, set_counter) = use_state(|| 0);
    let counter_value = counter.get();

    // useEffect equivalent - setInterval that increments counter every 100ms
    use_interval(
        {
            move || {
                // setCounter(previousCounter => previousCounter + 1)
                set_counter.update(|counter| counter + 1);
            }
        },
        Duration::from_millis(100), // 100ms interval like the React example
    );

    // Return JSX-like rsx! - equivalent to: <Text color="green">{counter} tests passed</Text>
    rsx! {
        <Paragraph
            style={Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)}
            alignment={Alignment::Center}
        >
            {format!("{} tests passed", counter_value)}
        </Paragraph>
    }
}

/// A more elaborate version with multiple counters and styling
#[component]
fn EnhancedCounter() -> Element {
    // Multiple state hooks - like multiple useState calls
    let (tests_passed, set_tests_passed) = use_state(|| 0);
    let (tests_failed, set_tests_failed) = use_state(|| 0);
    let (uptime_seconds, set_uptime_seconds) = use_state(|| 0);

    // Get current values
    let tests_passed_value = tests_passed.get();
    let tests_failed_value = tests_failed.get();
    let uptime_seconds_value = uptime_seconds.get();

    // Fast counter for tests passed (every 100ms like React example)
    use_interval(
        {
            move || {
                set_tests_passed.update(|tests_passed| tests_passed + 1);
            }
        },
        Duration::from_millis(100),
    );

    // Slower counter for failed tests (every 500ms)
    use_interval(
        {
            move || {
                if tests_failed.get() < 5 {
                    set_tests_failed.update(|tests_failed| tests_failed + 1);
                }
            }
        },
        Duration::from_millis(500),
    );

    // Uptime counter (every second)
    use_interval(
        {
            move || {
                set_uptime_seconds.update(|uptime_seconds| uptime_seconds + 1);
            }
        },
        Duration::from_secs(1),
    );

    rsx! {
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]}
        >
            {/* Tests Passed - Green like the original */}
            <Block
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Green)}
                title="✅ Tests Passed"
            >
                <Paragraph
                    style={Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)}
                    alignment={Alignment::Center}
                >
                    {format!("{} tests passed", tests_passed_value)}
                </Paragraph>
            </Block>

            {/* Tests Failed - Red */}
            <Block
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Red)}
                title="❌ Tests Failed"
            >
                <Paragraph
                    style={Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)}
                    alignment={Alignment::Center}
                >
                    {format!("{} tests failed", tests_failed_value)}
                </Paragraph>
            </Block>

            {/* Uptime - Blue */}
            <Block
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Blue)}
                title="⏱️ Uptime"
            >
                <Paragraph
                    style={Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)}
                    alignment={Alignment::Center}
                >
                    {format!("{}s uptime", uptime_seconds_value)}
                </Paragraph>
            </Block>

            {/* Instructions */}
            <Block
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Yellow)}
                title="📝 Instructions"
            >
                <Paragraph
                    style={Style::default().fg(Color::Yellow)}
                    alignment={Alignment::Center}
                >
                    {"Press 'q' to quit • React-like hooks in Rust TUI"}
                </Paragraph>
            </Block>
        </Layout>
    }
}

/// Main App component that handles events and renders the counter
struct ReactLikeApp {
    title: String,
    enhanced_mode: bool,
}

impl ReactLikeApp {
    fn new(title: &str, enhanced_mode: bool) -> Self {
        Self {
            title: title.to_string(),
            enhanced_mode,
        }
    }
}

impl Component for ReactLikeApp {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Handle events (like event listeners in React)
        if let Some(event) = use_event()
            && let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') => {
                    request_exit();
                }
                KeyCode::Char('e') => {
                    // In a real app, you'd use state for this
                }
                _ => {}
            }
        }

        // Create the layout
        let layout = rsx! {
            <Layout
                direction={Direction::Vertical}
                margin={1}
                constraints={vec![
                    Constraint::Length(3),  // Header
                    Constraint::Min(0),     // Counter content
                ]}
            >
                {/* Header */}
                <Block
                    title="🚀 React-like Counter in Rust TUI"
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)}
                >
                    <Paragraph
                        alignment={Alignment::Center}
                        style={Style::default().fg(Color::White).add_modifier(Modifier::BOLD)}
                    >
                        {self.title.clone()}
                    </Paragraph>
                </Block>

                {/* Counter Component */}
                {
                    if self.enhanced_mode {
                        <EnhancedCounter />
                    } else {
                        <Block
                            title="🧪 Test Runner"
                            borders={Borders::ALL}
                            border_style={Style::default().fg(Color::Green)}
                        >
                            <Counter />
                        </Block>
                    }
                }
            </Layout>
        };

        // Render the layout
        layout.render(area, buffer);
    }
}

/// Entry point - equivalent to render(<Counter />) in React/Ink
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting React-like Counter Demo...");
    println!("This mimics the React/Ink example with TUI framework hooks!");
    println!("Press 'q' to quit\n");

    // Choose which version to run
    let enhanced_mode = std::env::args().any(|arg| arg == "--enhanced");

    // render(<ReactLikeApp />) - equivalent to the React render call
    if let Err(err) = render(move || {
        ReactLikeApp::new(
            if enhanced_mode {
                "🎨 Enhanced Multi-Counter Demo"
            } else {
                "⚡ Simple Counter (React/Ink Style)"
            },
            enhanced_mode,
        )
        .into()
    })
    .await
    {
        eprintln!("❌ Application error: {:?}", err);
    } else {
        println!("✨ React-like counter demo completed successfully!");
        println!("🎯 Demonstrated: useState → use_state, useEffect → use_interval");
    }

    Ok(())
}
