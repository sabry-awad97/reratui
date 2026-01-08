//! Counter Example with ComponentV2 and use_state_v2 Hook
//!
//! A simple counter application demonstrating the Reratui fiber framework with:
//! - ComponentV2 trait for components
//! - use_state_v2 hook for reactive state management with batching
//! - Direct widget rendering (no rsx! macro)
//!
//! Press 'j' to increment, 'k' to decrement, 'r' to reset, 'q' to quit.

use reratui::prelude::*;

struct Counter;

impl ComponentV2 for Counter {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let (count, set_count) = use_state_v2(|| 0i32);

        // Handle keyboard events
        if let Some(Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        })) = use_event()
        {
            match code {
                KeyCode::Char('j') | KeyCode::Up => {
                    set_count.update(|prev| prev + 1);
                }
                KeyCode::Char('k') | KeyCode::Down => {
                    if count > 0 {
                        set_count.update(|prev| prev - 1);
                    }
                }
                KeyCode::Char('r') => {
                    set_count.set(0);
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    request_exit_v2();
                }
                _ => {}
            }
        }

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(8),    // Counter display
                Constraint::Length(5), // Controls
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render header
        let title = "✨ Counter App (Fiber Architecture)";
        let header_block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Rgb(17, 24, 39)));
        let header = Paragraph::new(title)
            .alignment(Alignment::Center)
            .block(header_block);
        header.render(chunks[0], buffer);

        // Render count display
        let count_block = Block::default()
            .title("Current Count")
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Rgb(31, 41, 55)));
        let count_text = format!(
            "╔═══════════════════╗\n║   Count: {:>6}   ║\n╚═══════════════════╝",
            count
        );
        let count_paragraph = Paragraph::new(count_text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Rgb(147, 197, 253))
                    .add_modifier(Modifier::BOLD),
            )
            .block(count_block);
        count_paragraph.render(chunks[1], buffer);

        // Render controls
        let controls_block = Block::default()
            .title("Controls")
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Rgb(17, 24, 39)));
        let controls_text = "[j/↑] Increment  |  [k/↓] Decrement  |  [r] Reset  |  [q/Esc] Quit";
        let controls = Paragraph::new(controls_text)
            .alignment(Alignment::Center)
            .block(controls_block);
        controls.render(chunks[2], buffer);

        // Render footer
        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(Color::Rgb(17, 24, 39)));
        let footer_text = format!("Area: {}x{}", area.width, area.height);
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .block(footer_block);
        footer.render(chunks[3], buffer);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render_v2(|| Counter).await?;
    Ok(())
}
