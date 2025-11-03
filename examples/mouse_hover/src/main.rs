//! 🖱️ Beautiful Mouse Hover Demo
//!
//! An elegant demonstration of mouse hover interactions with:
//! - 🎨 Modern card-based UI design
//! - ✨ Smooth hover effects with animations
//! - 🎯 Interactive buttons and panels
//! - 📊 Real-time hover statistics
//! - 🌈 Beautiful color schemes and gradients

use reratui::prelude::*;
use std::sync::{Arc, Mutex};

/// Hook for detecting if the mouse is hovering over a specific area.
///
/// Takes a rectangular area and returns a boolean indicating whether the mouse
/// is currently hovering over that area.
pub fn use_mouse_hover(area: Rect) -> bool {
    let (is_hovering, set_hovering) = use_state(|| false);

    if let Some(event) = use_event()
        && let Event::Mouse(MouseEvent { column, row, .. }) = event
    {
        let is_inside = column >= area.x
            && column < area.x + area.width
            && row >= area.y
            && row < area.y + area.height;

        if is_inside != is_hovering.get() {
            set_hovering.set(is_inside);
        }
    }

    is_hovering.get()
}

#[derive(Clone)]
struct HoverStats {
    total_hovers: Arc<Mutex<u32>>,
    current_position: Arc<Mutex<(u16, u16)>>,
}

impl HoverStats {
    fn new() -> Self {
        Self {
            total_hovers: Arc::new(Mutex::new(0)),
            current_position: Arc::new(Mutex::new((0, 0))),
        }
    }

    fn increment(&self) {
        *self.total_hovers.lock().unwrap() += 1;
    }

    fn update_position(&self, x: u16, y: u16) {
        *self.current_position.lock().unwrap() = (x, y);
    }

    fn get_count(&self) -> u32 {
        *self.total_hovers.lock().unwrap()
    }

    fn get_position(&self) -> (u16, u16) {
        *self.current_position.lock().unwrap()
    }
}

#[derive(Clone)]
struct MouseHoverDemo {
    stats: HoverStats,
}

impl MouseHoverDemo {
    fn new() -> Self {
        Self {
            stats: HoverStats::new(),
        }
    }
}

impl Component for MouseHoverDemo {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Track mouse position
        if let Some(event) = use_event()
            && let Event::Mouse(MouseEvent { column, row, .. }) = event
        {
            self.stats.update_position(column, row);
        }

        // Main layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Main content
                Constraint::Length(5), // Stats footer
            ])
            .split(area);

        // Render title
        render_title(buffer, main_chunks[0]);

        // Content area split into left and right
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[1]);

        // Left side: Interactive cards
        render_interactive_cards(buffer, content_chunks[0], &self.stats);

        // Right side: Button grid
        render_button_grid(buffer, content_chunks[1], &self.stats);

        // Footer with stats
        render_stats_footer(buffer, main_chunks[2], &self.stats);
    }
}

fn render_title(buffer: &mut Buffer, area: Rect) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .border_type(BorderType::Double);

    let title = Paragraph::new("🖱️  Interactive Mouse Hover Showcase")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .block(title_block);

    title.render(area, buffer);
}

fn render_interactive_cards(buffer: &mut Buffer, area: Rect, stats: &HoverStats) {
    let block = Block::default()
        .title("🎴 Interactive Cards")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    block.render(area, buffer);

    // Create 3 card areas
    let card_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .margin(1)
        .split(inner);

    let cards = [
        ("🚀 Performance", "Hover to boost", Color::Green),
        ("🎨 Design", "Hover to style", Color::Blue),
        ("⚡ Speed", "Hover to accelerate", Color::Yellow),
    ];

    for (i, (title, subtitle, color)) in cards.iter().enumerate() {
        let is_hovering = use_mouse_hover(card_chunks[i]);

        if is_hovering {
            let prev_count = stats.get_count();
            stats.increment();
            if stats.get_count() != prev_count {
                // Count changed
            }
        }

        let (border_style, bg_color, icon) = if is_hovering {
            (
                Style::default()
                    .fg(*color)
                    .add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK),
                Some(*color),
                "✨",
            )
        } else {
            (Style::default().fg(Color::DarkGray), None, "")
        };

        let card_block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .border_type(BorderType::Rounded);

        let card_inner = card_block.inner(card_chunks[i]);
        card_block.render(card_chunks[i], buffer);

        // Calculate vertical centering
        let content_height = 3; // 3 lines of content
        let available_height = card_inner.height as usize;
        let top_padding = (available_height.saturating_sub(content_height)) / 2;

        let mut lines = vec![];

        // Add top padding for vertical centering
        for _ in 0..top_padding {
            lines.push(Line::from(""));
        }

        lines.push(Line::from(vec![Span::styled(
            format!("{} {}", icon, title),
            Style::default()
                .fg(if is_hovering { Color::White } else { *color })
                .bg(bg_color.unwrap_or(Color::Reset))
                .add_modifier(if is_hovering {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            *subtitle,
            Style::default()
                .fg(if is_hovering {
                    Color::White
                } else {
                    Color::DarkGray
                })
                .bg(bg_color.unwrap_or(Color::Reset)),
        )));

        let card_text = Paragraph::new(lines).alignment(Alignment::Center);
        card_text.render(card_inner, buffer);
    }
}

fn render_button_grid(buffer: &mut Buffer, area: Rect, stats: &HoverStats) {
    let block = Block::default()
        .title("🔘 Button Grid")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    block.render(area, buffer);

    // Create a 3x3 grid of buttons
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .margin(1)
        .split(inner);

    let button_colors = [
        [Color::Red, Color::Green, Color::Blue],
        [Color::Yellow, Color::Magenta, Color::Cyan],
        [Color::LightRed, Color::LightGreen, Color::LightBlue],
    ];

    let button_labels = [["🔴", "🟢", "🔵"], ["🟡", "🟣", "🔷"], ["🔺", "🟩", "💠"]];

    for (row_idx, row_area) in rows.iter().enumerate() {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(*row_area);

        for (col_idx, col_area) in cols.iter().enumerate() {
            let is_hovering = use_mouse_hover(*col_area);

            if is_hovering {
                stats.increment();
            }

            let color = button_colors[row_idx][col_idx];
            let label = button_labels[row_idx][col_idx];

            let button_style = if is_hovering {
                Style::default()
                    .fg(Color::Black)
                    .bg(color)
                    .add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK)
            } else {
                Style::default().fg(color).add_modifier(Modifier::DIM)
            };

            let button_block = Block::default()
                .borders(Borders::ALL)
                .border_style(if is_hovering {
                    Style::default().fg(color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                })
                .border_type(if is_hovering {
                    BorderType::Double
                } else {
                    BorderType::Plain
                });

            let button_inner = button_block.inner(*col_area);
            button_block.render(*col_area, buffer);

            // Calculate vertical centering for button
            let available_height = button_inner.height as usize;
            let top_padding = available_height / 2;

            let mut button_lines = vec![];
            
            // Add top padding for vertical centering
            for _ in 0..top_padding {
                button_lines.push(Line::from(""));
            }
            button_lines.push(Line::from(Span::styled(label, button_style)));

            let button_text = Paragraph::new(button_lines).alignment(Alignment::Center);
            button_text.render(button_inner, buffer);
        }
    }
}

fn render_stats_footer(buffer: &mut Buffer, area: Rect, stats: &HoverStats) {
    let block = Block::default()
        .title("📊 Live Statistics")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    block.render(area, buffer);

    let (mouse_x, mouse_y) = stats.get_position();
    let hover_count = stats.get_count();

    let stats_text = vec![
        Line::from(vec![
            Span::styled("🖱️  Mouse Position: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("({}, {})", mouse_x, mouse_y),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled("✨ Total Hovers: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{}", hover_count),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "💡 Tip: Move your mouse over the cards and buttons to see the magic!",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let stats_paragraph = Paragraph::new(stats_text).alignment(Alignment::Center);
    stats_paragraph.render(inner, buffer);
}

/// Entry point for the application
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = MouseHoverDemo::new();
    render(move || app.clone().into()).await?;

    Ok(())
}
