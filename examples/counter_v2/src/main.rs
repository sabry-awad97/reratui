//! Counter Example with v2 APIs using ComponentV2 trait
//!
//! This example demonstrates using the ComponentV2 trait with use_event hook.
//! With position-based component identification, you can create components
//! inside the render closure and state will persist across frames (like React).
//!
//! This example uses only `reratui-fiber` - no rsx! macro needed.
//!
//! Press 'j' to increment, 'k' to decrement, 'r' to reset, 'q' to quit.

use reratui::prelude::*;

/// The main counter component - just implement render()!
struct Counter;

impl ComponentV2 for Counter {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let (count, set_count) = use_state_v2(|| 0i32);
        let component_area = try_use_context_v2::<ComponentArea>()
            .map(|ca| ca.area())
            .unwrap_or(area);

        // Handle keyboard events using use_event hook
        if let Some(Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        })) = use_event()
        {
            match code {
                KeyCode::Char('j') => {
                    set_count.update(|prev| prev + 1);
                }
                KeyCode::Char('k') => {
                    if count > 0 {
                        set_count.update(|prev| prev - 1);
                    }
                }
                KeyCode::Char('r') => {
                    set_count.set(0);
                }
                KeyCode::Char('q') => {
                    request_exit_v2();
                }
                _ => {}
            }
        }

        // Effect that runs after commit when count changes
        use_effect_v2(
            {
                move || {
                    let _ = count; // Could log: "Count changed to {count}"
                    Option::<fn()>::None
                }
            },
            Some((count,)),
        );

        let title = "✨ Counter App (v2 Trait + use_event)";

        // Create layout chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(5),
                Constraint::Length(3),
            ])
            .split(area);

        // Render title block
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Rgb(17, 24, 39)));
        let title_paragraph = Paragraph::new(title)
            .alignment(Alignment::Center)
            .block(title_block);
        title_paragraph.render(chunks[0], buffer);

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

        // Render controls info
        let controls_block = Block::default()
            .title("Controls")
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Rgb(17, 24, 39)));
        let controls_paragraph =
            Paragraph::new("[j] Increment  |  [k] Decrement  |  [r] Reset  |  [q] Quit")
                .alignment(Alignment::Center)
                .block(controls_block);
        controls_paragraph.render(chunks[2], buffer);

        // Render footer
        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(Color::Rgb(17, 24, 39)));
        let footer_text = format!(
            "Component Area: {}x{} | Using use_event hook",
            component_area.width, component_area.height
        );
        let footer_paragraph = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .block(footer_block);
        footer_paragraph.render(chunks[3], buffer);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple: just pass a closure that returns your component!
    render_v2(|| Counter).await?;
    Ok(())
}
