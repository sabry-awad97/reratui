use reratui::{prelude::*, ratatui::widgets::BorderType};

use super::utils::interpolate_color;
use crate::theme::Theme;

pub struct MarqueeComponent {
    text: String,
    theme: Theme,
}

impl MarqueeComponent {
    pub fn new(text: String, theme: Theme) -> Self {
        Self { text, theme }
    }
}

impl ComponentV2 for MarqueeComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Border breathing effect state for consistent border color with other components
        let (breath_value, set_breath_value) = use_state_v2(|| 0.0f32);

        // Marquee state
        let (marquee_offset, set_marquee_offset) = use_state_v2(|| 0usize);

        use_interval_v2(
            {
                // Set up marquee animation
                move || {
                    // Update marquee position
                    set_marquee_offset.update(|prev| *prev + 1);
                }
            },
            250, // Slower marquee speed to reduce CPU usage
        );

        use_interval_v2(
            {
                // Set up border breathing effect
                move || {
                    // Update breathing value using update() to get fresh value
                    set_breath_value.update(move |current| {
                        // Use a simple sine-wave-like oscillation
                        let new_value = *current + 0.05;
                        if new_value >= 1.0 {
                            0.0 // Reset to start
                        } else {
                            new_value
                        }
                    });
                }
            },
            200, // Slower breathing speed to reduce CPU usage
        );

        // Calculate border color based on breathing effect
        let breath_factor = breath_value;

        // Interpolate between border color and accent color based on breath value
        let border_color = interpolate_color(self.theme.border, self.theme.accent, breath_factor);

        // Create marquee widget
        let marquee = create_marquee(
            &self.text,
            area.width as usize - 4, // Account for borders
            marquee_offset,
            &self.theme,
        );

        let marquee_widget = Paragraph::new(marquee).alignment(Alignment::Left).block(
            Block::default()
                .title(Span::styled(
                    " 📢 Announcements 📢 ",
                    Style::default()
                        .fg(self.theme.accent)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        );

        marquee_widget.render(area, buffer);
    }
}

/// Create a marquee text that scrolls from right to left
fn create_marquee<'a>(text: &'a str, width: usize, offset: usize, theme: &'a Theme) -> Line<'a> {
    // Convert text to a vector of characters to handle Unicode correctly
    let text_chars: Vec<char> = text.chars().collect();
    let padding_chars: Vec<char> = " ".repeat(width.min(20)).chars().collect();

    // Create a circular buffer of characters
    let mut all_chars = Vec::with_capacity(text_chars.len() * 2 + padding_chars.len());
    all_chars.extend(text_chars.iter());
    all_chars.extend(padding_chars.iter());
    all_chars.extend(text_chars.iter());

    // Calculate the starting position for the visible portion
    let start_pos = offset % (text_chars.len() + padding_chars.len());

    // Get the visible characters
    let visible_chars: Vec<char> = all_chars
        .iter()
        .cycle()
        .skip(start_pos)
        .take(width)
        .cloned()
        .collect();

    // Create a gradient effect for the marquee text
    let spans: Vec<Span> = visible_chars
        .iter()
        .enumerate()
        .map(|(i, c)| {
            // Create a gradient from primary to accent color
            let factor = i as f32 / width as f32;
            let color = interpolate_color(theme.primary, theme.accent, factor);
            Span::styled(c.to_string(), Style::default().fg(color))
        })
        .collect();

    Line::from(spans)
}
