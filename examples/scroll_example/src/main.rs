//! Scroll Example - Demonstrates scrollable content in reratui
//!
//! This example shows two approaches to scrolling:
//! 1. Generic widget scrolling (ScrollView) - works with ANY widget
//! 2. Item-based scrolling (ScrollViewItems) - simpler API for lists
//!
//! Controls:
//! - j/Down: Scroll down
//! - k/Up: Scroll up
//! - g/Home: Scroll to top
//! - G/End: Scroll to bottom
//! - PageDown/PageUp: Page scroll
//! - Tab: Switch between examples
//! - q: Quit

use ratatui::widgets::Tabs;
use reratui::components::{
    ScrollIndicator, ScrollView, ScrollViewItemProps, ScrollViewItems, ScrollViewProps,
};
use reratui::prelude::*;

/// Main app with tabbed scroll examples
struct App;

impl Component for App {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Tab state
        let (tab, set_tab) = use_state(|| 0usize);

        // Handle tab switching and quit
        if let Some(Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        })) = use_event()
        {
            match code {
                KeyCode::Char('q') => request_exit(),
                KeyCode::Tab => set_tab.update(|t| (t + 1) % 2),
                _ => {}
            }
        }

        // Layout: tabs at top, content below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Render tab bar
        let tab_titles = vec!["Generic Widgets", "Item List"];
        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Scroll Examples (Tab to switch, q to quit) "),
            )
            .select(tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        tabs.render(chunks[0], buffer);

        // Render selected example
        match tab {
            0 => render_generic_scroll(chunks[1], buffer),
            1 => render_item_scroll(chunks[1], buffer),
            _ => {}
        }
    }
}

/// Example 1: Generic widget scrolling
///
/// This demonstrates scrolling ANY widgets - paragraphs, tables, blocks, etc.
/// The content is rendered to a virtual buffer and then the visible portion
/// is copied to the screen.
fn render_generic_scroll(area: Rect, buffer: &mut Buffer) {
    // Total content height: 10 sections × 8 lines each = 80 lines
    let section_height = 8u16;
    let section_count = 10u16;
    let content_height = section_height * section_count;

    let props = ScrollViewProps::new(content_height, move |content_area, buf| {
        // Render multiple widget sections - ALL of them, the ScrollView handles visibility
        for i in 0..section_count {
            let section_y = i * section_height;
            let section_area = Rect::new(
                content_area.x,
                content_area.y + section_y,
                content_area.width,
                section_height,
            );

            // Each section is a bordered block with content
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(match i % 4 {
                    0 => Color::Cyan,
                    1 => Color::Green,
                    2 => Color::Yellow,
                    _ => Color::Magenta,
                }))
                .title(format!(" Section {} ", i + 1));

            let inner = block.inner(section_area);
            block.render(section_area, buf);

            // Render some content inside each section
            let lines = [
                format!("This is section {} of {}", i + 1, section_count),
                String::new(),
                "This demonstrates generic widget scrolling.".to_string(),
                "Any widget can be scrolled, not just text!".to_string(),
                format!("Section starts at line {}", section_y),
            ];

            for (j, line) in lines.iter().enumerate() {
                if j < inner.height as usize {
                    buf.set_string(
                        inner.x,
                        inner.y + j as u16,
                        line,
                        Style::default().fg(Color::White),
                    );
                }
            }
        }
    })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Generic Widget Scroll (j/k to scroll) ")
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .indicators(ScrollIndicator {
        show_scrollbar: true,
        show_more_above: true,
        show_more_below: true,
        track_color: Color::DarkGray,
        thumb_color: Color::Cyan,
    });

    ScrollView::new(props).render(area, buffer);
}

/// Example 2: Item-based scrolling
///
/// This is a simpler API for when you have a list of items where each
/// item is one line. More efficient than the generic approach for simple lists.
fn render_item_scroll(area: Rect, buffer: &mut Buffer) {
    // Generate sample items
    let items: Vec<String> = (1..=100)
        .map(|i| format!(" {:>3}. Item {} - Sample list item with content", i, i))
        .collect();

    let item_count = items.len();

    let props = ScrollViewItemProps::new(
        item_count,
        move |content_area, buf, start_idx, visible_count| {
            // Only render visible items - more efficient!
            for (i, item) in items.iter().skip(start_idx).take(visible_count).enumerate() {
                let y = content_area.y + i as u16;

                // Alternate row colors
                let style = if (start_idx + i) % 2 == 0 {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };

                buf.set_string(content_area.x, y, item, style);
            }
        },
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Item List Scroll (j/k to scroll) ")
            .border_style(Style::default().fg(Color::Green)),
    )
    .indicators(ScrollIndicator {
        show_scrollbar: true,
        show_more_above: true,
        show_more_below: true,
        track_color: Color::DarkGray,
        thumb_color: Color::Green,
    });

    ScrollViewItems::new(props).render(area, buffer);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    render(|| App).await?;
    Ok(())
}
