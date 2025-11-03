use reratui::prelude::*;

/// A simple component that demonstrates the use_state hook
struct StateDemo {
    title: String,
}

impl StateDemo {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

impl Component for StateDemo {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Create a counter state with initial value 0
        let (counter, set_counter) = use_state(|| 0);

        // Create a text state with initial value
        let (message, set_message) = use_state(|| String::from("Welcome to the State Demo!"));

        // Create a color state with initial value
        let (color, set_color) = use_state(|| Color::Cyan);

        // Create a boolean state for toggling
        let (show_help, set_show_help) = use_state(|| true);

        let event = use_event();
        // Process the current event if available
        if let Some(ref event) = event
            && let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                // Counter controls
                KeyCode::Char('j') => {
                    set_counter.update(|counter| counter + 1);
                }
                KeyCode::Char('k') => {
                    let current = counter.get();
                    if current > 0 {
                        set_counter.update(|counter| counter - 1);
                    }
                }

                // Color cycling
                KeyCode::Char('c') => {
                    set_color.update(|color| match color {
                        Color::Cyan => Color::Magenta,
                        Color::Magenta => Color::Yellow,
                        Color::Yellow => Color::Red,
                        Color::Red => Color::Green,
                        Color::Green => Color::Blue,
                        _ => Color::Cyan,
                    });
                }

                // Message changing
                KeyCode::Char('m') => {
                    let messages = [
                        "Welcome to the State Demo!",
                        "State management is easy with hooks!",
                        "Try changing the counter and color too!",
                        "Hooks make TUI apps more reactive!",
                        "This message is stored in state!",
                    ];

                    let current_idx = messages
                        .iter()
                        .position(|&m| m == message.get())
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % messages.len();
                    set_message.set(String::from(messages[next_idx]));
                }

                // Toggle help
                KeyCode::Char('h') => {
                    set_show_help.update(|show_help| !show_help);
                }

                _ => {}
            }
        }

        // Create the main layout with proper spacing
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(5), // Counter
                Constraint::Length(5), // Message
                Constraint::Length(5), // Color demo
                Constraint::Length(7), // Help
                Constraint::Length(5), // Current event display
            ])
            .split(area);

        // Render the title
        let title_block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));
        title_block.render(chunks[0], buffer);

        // Render the counter with better styling
        let counter_value = counter.get();
        let counter_text = format!("Counter: {}", counter_value);
        let counter_widget = Paragraph::new(counter_text)
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("State Example 1: Number")
                    .title_style(
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
        counter_widget.render(chunks[1], buffer);

        // Render the message with better styling
        let message_widget = Paragraph::new(message.get())
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("State Example 2: Text")
                    .title_style(
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            );
        message_widget.render(chunks[2], buffer);

        // Render the color demo with better styling
        let current_color = color.get();
        let color_name = match current_color {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
            Color::Cyan => "Cyan",
            Color::Magenta => "Magenta",
            Color::Yellow => "Yellow",
            _ => "Other",
        };

        let color_text = format!("Current color: {}", color_name);
        let color_widget = Paragraph::new(color_text)
            .style(
                Style::default()
                    .fg(current_color)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("State Example 3: Color")
                    .title_style(
                        Style::default()
                            .fg(current_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(current_color)),
            );
        color_widget.render(chunks[3], buffer);

        // Render help with better styling
        if show_help.get() {
            let help_text = vec![
                Line::from(vec![
                    Span::styled(
                        "j/k",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Increment/decrement counter  "),
                    Span::styled(
                        "c",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Change color"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "m",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Change message  "),
                    Span::styled(
                        "h",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Toggle help"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "q",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Quit"),
                ]),
            ];

            let help_widget = Paragraph::new(help_text)
                .style(Style::default())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title("Help")
                        .title_style(
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White)),
                );
            help_widget.render(chunks[4], buffer);
        }

        // Render current event information with better styling
        let event_text = if let Some(ref event) = event {
            format!("Current event: {:?}", event)
        } else {
            "No current event".to_string()
        };

        let event_widget = Paragraph::new(event_text)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("Event Info")
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        event_widget.render(chunks[5], buffer);
    }
}

/// Entry point for the application
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| StateDemo::new("✨ useState Hook Demo ✨").into()).await?;

    Ok(())
}
