use reratui::{prelude::*, ratatui::widgets::BorderType};

use super::utils::interpolate_color;
use crate::theme::Theme;

pub struct AnimatedTitleComponent {
    title: String,
    theme: Theme,
}

impl AnimatedTitleComponent {
    pub fn new(title: String, theme: Theme) -> Self {
        Self { title, theme }
    }
}

impl ComponentV2 for AnimatedTitleComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Animation state for color cycling
        let (animation_step, set_animation_step) = use_state_v2(|| 0usize);

        // Typing animation state
        let (visible_chars, set_visible_chars) = use_state_v2(|| 0usize);
        let (typing_complete, set_typing_complete) = use_state_v2(|| false);

        // Border breathing effect state
        let (breath_value, set_breath_value) = use_state_v2(|| 0.0f32);

        // Define colors for animation
        let colors = [
            self.theme.primary,
            self.theme.accent,
            self.theme.secondary,
            self.theme.info,
            self.theme.success,
        ];

        use_interval_v2(
            {
                // Set up color cycling animation interval
                let colors_len = colors.len();
                move || {
                    // Update animation step using update() to get fresh values
                    set_animation_step.update(move |current_step| {
                        // Cycle through colors
                        (*current_step + 1) % colors_len
                    });
                }
            },
            300, // Reduced animation frequency to save CPU
        );

        use_interval_v2(
            {
                // Set up typing animation interval
                let title_len = self.title.chars().count();
                move || {
                    // Increment visible characters using update() to get fresh value
                    set_visible_chars.update(move |current| {
                        if *current < title_len {
                            *current + 1
                        } else {
                            // Mark typing as complete
                            set_typing_complete.set(true);
                            *current
                        }
                    });
                }
            },
            150, // Slower typing speed to reduce CPU usage
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

        // Get current color based on animation step for character coloring
        let char_base_color = colors[animation_step % colors.len()];

        // Create animated title spans with typing effect
        let title_spans = self
            .title
            .chars()
            .enumerate()
            .map(|(i, c)| {
                // Only show characters up to the current typing position
                if i >= visible_chars {
                    return Span::styled(
                        " ",
                        Style::default()
                            .fg(self.theme.background)
                            .bg(self.theme.background),
                    );
                }

                // Offset each character's color slightly for a wave effect
                let char_offset = (i + animation_step) % colors.len();
                let char_color = colors[char_offset];

                // Apply a slight breathing effect to the character color too
                let char_color = interpolate_color(char_color, char_base_color, breath_value * 0.3);

                Span::styled(
                    c.to_string(),
                    Style::default().fg(char_color).add_modifier(Modifier::BOLD),
                )
            })
            .collect::<Vec<_>>();

        // Add cursor at typing position if typing is not complete
        let mut final_spans = title_spans;
        if !typing_complete {
            let cursor_pos = visible_chars;
            if cursor_pos < self.title.len() {
                // Add blinking cursor
                if animation_step % 2 == 0 {
                    final_spans.push(Span::styled(
                        "▎", // Cursor character
                        Style::default()
                            .fg(self.theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
            }
        }

        // Calculate border color based on breathing effect
        let breath_factor = breath_value;

        // Interpolate between border color and accent color based on breath value
        let border_color = interpolate_color(self.theme.border, self.theme.accent, breath_factor);

        // Render title with animated styling and breathing border
        let title_widget = Paragraph::new(Line::from(final_spans))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(Span::styled(
                        " Title ",
                        Style::default()
                            .fg(self.theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color)),
            );

        title_widget.render(area, buffer);
    }
}
