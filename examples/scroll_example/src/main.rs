//! Scroll Example - Demonstrates scrollable content in reratui
//!
//! This example shows how to use the scroll hooks to create scrollable lists.
//! NOTE: Requires the react-event-semantics fix to work properly with multiple
//! event handlers (quit + scroll).
//!
//! Controls:
//! - j/Down: Scroll down
//! - k/Up: Scroll up
//! - g/Home: Scroll to top
//! - G/End: Scroll to bottom
//! - Ctrl+d/PageDown: Page down
//! - Ctrl+u/PageUp: Page up
//! - q: Quit

use reratui::hooks::use_scroll_keyboard;
use reratui::prelude::*;

/// Main app showing scrollable list
struct App;

impl Component for App {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Generate sample data
        let items: Vec<String> = (1..=100)
            .map(|i| format!("Item {:>3} - This is a sample list item with some content to demonstrate scrolling", i))
            .collect();

        // Calculate content area (with border)
        let block = Block::default()
            .title(" Scrollable List (j/k to scroll, q to quit) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buffer);

        // Use scroll hook with automatic keyboard bindings
        // This will work once react-event-semantics is implemented
        let scroll = use_scroll_keyboard(items.len(), inner.height as usize);

        // Handle quit separately - will work after event fix
        if let Some(Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            kind: KeyEventKind::Press,
            ..
        })) = use_event()
        {
            request_exit();
        }

        // Render visible items
        for (i, item) in items
            .iter()
            .skip(scroll.offset)
            .take(inner.height as usize)
            .enumerate()
        {
            let y = inner.y + i as u16;

            // Alternate row colors for readability
            let style = if (scroll.offset + i).is_multiple_of(2) {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };

            buffer.set_string(inner.x, y, item, style);
        }

        // Show scroll indicators
        if scroll.has_overflow() {
            // Progress percentage
            let progress = format!(" {:.0}% ", scroll.scroll_progress() * 100.0);
            let x = area.x + area.width.saturating_sub(progress.len() as u16 + 1);
            buffer.set_string(x, area.y, &progress, Style::default().fg(Color::Cyan));

            // "More above" indicator
            if scroll.can_scroll_up() {
                buffer.set_string(
                    inner.x + inner.width.saturating_sub(8),
                    inner.y,
                    "▲ more",
                    Style::default().fg(Color::DarkGray),
                );
            }

            // "More below" indicator
            if scroll.can_scroll_down() {
                buffer.set_string(
                    inner.x + inner.width.saturating_sub(8),
                    inner.y + inner.height.saturating_sub(1),
                    "▼ more",
                    Style::default().fg(Color::DarkGray),
                );
            }

            // Scrollbar
            let scrollbar_x = inner.x + inner.width;
            let scrollbar_height = inner.height as f64;
            let thumb_size =
                (scrollbar_height * (inner.height as f64 / items.len() as f64)).max(1.0) as u16;
            let thumb_pos =
                (scroll.scroll_progress() * (scrollbar_height - thumb_size as f64)) as u16;

            for y in 0..inner.height {
                let char = if y >= thumb_pos && y < thumb_pos + thumb_size {
                    "█"
                } else {
                    "░"
                };
                buffer.set_string(
                    scrollbar_x,
                    inner.y + y,
                    char,
                    Style::default().fg(Color::Cyan),
                );
            }
        }

        // Status line
        let status = format!(
            " Lines {}-{} of {} ",
            scroll.offset + 1,
            (scroll.offset + inner.height as usize).min(items.len()),
            items.len()
        );
        buffer.set_string(
            area.x + 2,
            area.y + area.height.saturating_sub(1),
            &status,
            Style::default().fg(Color::Cyan),
        );
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    render(|| App).await?;
    Ok(())
}
