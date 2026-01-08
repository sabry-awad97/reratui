use reratui::prelude::*;
use reratui::hooks::{use_interval_v2, use_keyboard_press_v2};
use reratui::ratatui::widgets::BorderType;

/// A React-like Counter component that mimics the Ink example
///
/// This component demonstrates:
/// - useState equivalent with use_state_v2
/// - useEffect equivalent with use_interval_v2
/// - Component composition with ComponentV2
/// - Automatic cleanup on unmount
struct Counter;

impl ComponentV2 for Counter {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // useState equivalent - initialize counter to 0
        let (counter_value, set_counter) = use_state_v2(|| 0i32);

        // useEffect equivalent - setInterval that increments counter every 100ms
        use_interval_v2(
            {
                move || {
                    // setCounter(previousCounter => previousCounter + 1)
                    set_counter.update(|counter| counter + 1);
                }
            },
            100, // 100ms interval like the React example
        );

        // Render - equivalent to: <Text color="green">{counter} tests passed</Text>
        let paragraph = Paragraph::new(format!("{} tests passed", counter_value))
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);

        paragraph.render(area, buffer);
    }
}

/// A more elaborate version with multiple counters and styling
struct EnhancedCounter;

impl ComponentV2 for EnhancedCounter {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Multiple state hooks - like multiple useState calls
        let (tests_passed_value, set_tests_passed) = use_state_v2(|| 0i32);
        let (tests_failed_value, set_tests_failed) = use_state_v2(|| 0i32);
        let (uptime_seconds_value, set_uptime_seconds) = use_state_v2(|| 0i32);

        // Fast counter for tests passed (every 100ms like React example)
        use_interval_v2(
            {
                move || {
                    set_tests_passed.update(|tests_passed| tests_passed + 1);
                }
            },
            100,
        );

        // Slower counter for failed tests (every 500ms)
        use_interval_v2(
            {
                move || {
                    if tests_failed_value < 5 {
                        set_tests_failed.update(|tests_failed| tests_failed + 1);
                    }
                }
            },
            500,
        );

        // Uptime counter (every second)
        use_interval_v2(
            {
                move || {
                    set_uptime_seconds.update(|uptime_seconds| uptime_seconds + 1);
                }
            },
            1000,
        );

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        // Tests Passed - Green like the original
        let passed_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title("✅ Tests Passed");
        let passed_inner = passed_block.inner(chunks[0]);
        passed_block.render(chunks[0], buffer);

        let passed_text = Paragraph::new(format!("{} tests passed", tests_passed_value))
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        passed_text.render(passed_inner, buffer);

        // Tests Failed - Red
        let failed_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title("❌ Tests Failed");
        let failed_inner = failed_block.inner(chunks[1]);
        failed_block.render(chunks[1], buffer);

        let failed_text = Paragraph::new(format!("{} tests failed", tests_failed_value))
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        failed_text.render(failed_inner, buffer);

        // Uptime - Blue
        let uptime_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title("⏱️ Uptime");
        let uptime_inner = uptime_block.inner(chunks[2]);
        uptime_block.render(chunks[2], buffer);

        let uptime_text = Paragraph::new(format!("{}s uptime", uptime_seconds_value))
            .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        uptime_text.render(uptime_inner, buffer);

        // Instructions
        let instructions_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title("📝 Instructions");
        let instructions_inner = instructions_block.inner(chunks[3]);
        instructions_block.render(chunks[3], buffer);

        let instructions_text = Paragraph::new("Press 'q' to quit • React-like hooks in Rust TUI")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        instructions_text.render(instructions_inner, buffer);
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

impl ComponentV2 for ReactLikeApp {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Handle events (like event listeners in React)
        use_keyboard_press_v2(|key| {
            if key.code == KeyCode::Char('q') {
                request_exit_v2();
            }
        });

        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Counter content
            ])
            .split(area);

        // Header
        let header_block = Block::default()
            .title("🚀 React-like Counter in Rust TUI")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Double);
        let header_inner = header_block.inner(chunks[0]);
        header_block.render(chunks[0], buffer);

        let header_text = Paragraph::new(self.title.clone())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
        header_text.render(header_inner, buffer);

        // Counter Component
        if self.enhanced_mode {
            EnhancedCounter.render(chunks[1], buffer);
        } else {
            let counter_block = Block::default()
                .title("🧪 Test Runner")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green));
            let counter_inner = counter_block.inner(chunks[1]);
            counter_block.render(chunks[1], buffer);

            Counter.render(counter_inner, buffer);
        }
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
    let title = if enhanced_mode {
        "🎨 Enhanced Multi-Counter Demo"
    } else {
        "⚡ Simple Counter (React/Ink Style)"
    };
    
    if let Err(err) = render_v2(move || {
        ReactLikeApp::new(title, enhanced_mode)
    })
    .await
    {
        eprintln!("❌ Application error: {:?}", err);
    } else {
        println!("✨ React-like counter demo completed successfully!");
        println!("🎯 Demonstrated: useState → use_state_v2, useEffect → use_interval_v2");
    }

    Ok(())
}
