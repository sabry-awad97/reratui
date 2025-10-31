use std::time::Duration;

use reratui::prelude::*;

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

impl Component for AnimatedTitleComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Animation state for color cycling
        let (animation_step, set_animation_step) = use_state(|| 0usize);
        let (pulse_direction, set_pulse_direction) = use_state(|| true); // true = increasing, false = decreasing

        // Typing animation state
        let (visible_chars, set_visible_chars) = use_state(|| 0usize);
        let (typing_complete, set_typing_complete) = use_state(|| false);

        // Border breathing effect state
        let (breath_value, set_breath_value) = use_state(|| 0.0f32);
        let (breath_increasing, set_breath_increasing) = use_state(|| true);

        // Define colors for animation
        let colors = [
            self.theme.primary,
            self.theme.accent,
            self.theme.secondary,
            self.theme.info,
            self.theme.success,
        ];

        use_interval(
            {
                // Set up color cycling animation interval
                let animation_step = animation_step.clone();
                let set_animation_step = set_animation_step.clone();
                let pulse_direction = pulse_direction.clone();
                let set_pulse_direction = set_pulse_direction.clone();

                move || {
                    // Update animation step
                    if pulse_direction.get() {
                        // Increasing brightness
                        let next_step = animation_step.get() + 1;
                        if next_step >= colors.len() {
                            set_pulse_direction.set(false);
                            set_animation_step.set(colors.len() - 2);
                        } else {
                            set_animation_step.set(next_step);
                        }
                    } else {
                        // Decreasing brightness
                        if animation_step.get() <= 1 {
                            set_pulse_direction.set(true);
                            set_animation_step.set(0);
                        } else {
                            set_animation_step.set(animation_step.get() - 1);
                        }
                    }
                }
            },
            Duration::from_millis(300), // Reduced animation frequency to save CPU
        );

        use_interval(
            {
                // Set up typing animation interval
                let visible_chars = visible_chars.clone();
                let set_visible_chars = set_visible_chars.clone();
                let typing_complete = typing_complete.clone();
                let set_typing_complete = set_typing_complete.clone();
                let title_len = self.title.chars().count();
                move || {
                    // Only run typing animation if not complete
                    if !typing_complete.get() {
                        let current_visible = visible_chars.get();

                        if current_visible < title_len {
                            // Increment visible characters
                            set_visible_chars.set(current_visible + 1);
                        } else {
                            // Mark typing as complete
                            set_typing_complete.set(true);
                        }
                    }
                }
            },
            Duration::from_millis(150), // Slower typing speed to reduce CPU usage
        );

        use_interval(
            {
                // Set up border breathing effect
                let breath_value = breath_value.clone();
                let set_breath_value = set_breath_value.clone();
                let breath_increasing = breath_increasing.clone();
                let set_breath_increasing = set_breath_increasing.clone();

                move || {
                    let current_value = breath_value.get();
                    let is_increasing = breath_increasing.get();

                    // Update breathing value
                    if is_increasing {
                        let new_value = current_value + 0.05;
                        if new_value >= 1.0 {
                            set_breath_increasing.set(false);
                            set_breath_value.set(1.0);
                        } else {
                            set_breath_value.set(new_value);
                        }
                    } else {
                        let new_value = current_value - 0.05;
                        if new_value <= 0.0 {
                            set_breath_increasing.set(true);
                            set_breath_value.set(0.0);
                        } else {
                            set_breath_value.set(new_value);
                        }
                    }
                }
            },
            Duration::from_millis(200), // Slower breathing speed to reduce CPU usage
        );

        // Get current color based on animation step for character coloring
        let char_base_color = colors[animation_step.get() % colors.len()];

        // Create animated title spans with typing effect
        let title_spans = self
            .title
            .chars()
            .enumerate()
            .map(|(i, c)| {
                // Only show characters up to the current typing position
                if i >= visible_chars.get() {
                    return Span::styled(
                        " ",
                        Style::default()
                            .fg(self.theme.background)
                            .bg(self.theme.background),
                    );
                }

                // Offset each character's color slightly for a wave effect
                let char_offset = (i + animation_step.get()) % colors.len();
                let char_color = colors[char_offset];

                // Apply a slight breathing effect to the character color too
                let char_color =
                    interpolate_color(char_color, char_base_color, breath_value.get() * 0.3);

                Span::styled(
                    c.to_string(),
                    Style::default().fg(char_color).add_modifier(Modifier::BOLD),
                )
            })
            .collect::<Vec<_>>();

        // Add cursor at typing position if typing is not complete
        let mut final_spans = title_spans;
        if !typing_complete.get() {
            let cursor_pos = visible_chars.get();
            if cursor_pos < self.title.len() {
                // Add blinking cursor
                if animation_step.get() % 2 == 0 {
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
        let breath_factor = breath_value.get();

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
