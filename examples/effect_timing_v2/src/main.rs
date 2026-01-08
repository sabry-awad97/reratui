//! Effect Timing Example with v2 APIs
//!
//! This example demonstrates the React-like effect timing behavior:
//! - Effects run AFTER the commit phase (not during render)
//! - Cleanup functions run BEFORE new effects
//! - Dependency tracking controls when effects re-run
//!
//! Press 'j' to increment, 'k' to decrement, 'q' to quit.

use reratui::prelude::*;

struct EffectDemo;

impl ComponentV2 for EffectDemo {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let (count, set_count) = use_state_v2(|| 0i32);
        let (effect_log, set_effect_log) = use_state_v2(Vec::<String>::new);

        // Effect that runs when count changes
        use_effect_v2(
            {
                move || {
                    set_effect_log.update(move |log| {
                        let mut new_log = log.clone();
                        new_log.push(format!("Effect ran: count = {}", count));
                        if new_log.len() > 5 {
                            new_log.remove(0);
                        }
                        new_log
                    });

                    let cleanup_count = count;
                    Some(move || {
                        let _ = cleanup_count;
                    })
                }
            },
            Some((count,)),
        );

        // Handle keyboard events
        if let Some(Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        })) = use_event()
        {
            match code {
                KeyCode::Char('j') => set_count.update(|n| n + 1),
                KeyCode::Char('k') => {
                    if count > 0 {
                        set_count.update(|n| n - 1);
                    }
                }
                KeyCode::Char('q') => request_exit_v2(),
                _ => {}
            }
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(8),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header_block = Block::default()
            .title("Effect Timing Demo (v2 APIs)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let header = Paragraph::new("Effects run AFTER commit, not during render")
            .alignment(Alignment::Center)
            .block(header_block);
        header.render(chunks[0], buffer);

        // Count display
        let count_block = Block::default()
            .title("Current Count")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let count_para = Paragraph::new(format!("Count: {}", count))
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .block(count_block);
        count_para.render(chunks[1], buffer);

        // Effect log
        let log_block = Block::default()
            .title("Effect Log (last 5)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));
        let log_para = Paragraph::new(effect_log.join("\n")).block(log_block);
        log_para.render(chunks[2], buffer);

        // Footer
        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let footer = Paragraph::new("Press 'j' to increment, 'k' to decrement, 'q' to quit")
            .alignment(Alignment::Center)
            .block(footer_block);
        footer.render(chunks[3], buffer);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render_v2(|| EffectDemo).await?;
    Ok(())
}
