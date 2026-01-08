use chrono::{DateTime, Local};
use reratui_fiber::{prelude::*, ratatui::widgets::BorderType};

use super::utils::interpolate_color;
use crate::theme::Theme;

/// Notification level for the notification icon
#[derive(Clone, Copy, PartialEq)]
pub enum NotificationLevel {
    None,
    Low,
    Medium,
    High,
}

/// Application mode for the mode icon
#[derive(Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Edit,
    View,
    Command,
}

pub struct StatusIconsComponent {
    notification_level: NotificationLevel,
    app_mode: AppMode,
    theme: Theme,
}

impl StatusIconsComponent {
    pub fn new(notification_level: NotificationLevel, app_mode: AppMode, theme: Theme) -> Self {
        Self {
            notification_level,
            app_mode,
            theme,
        }
    }
}

impl ComponentV2 for StatusIconsComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Animation state for notification blinking
        let (animation_step, set_animation_step) = use_state_v2(|| 0usize);

        // Clock state
        let (current_time, set_current_time) = use_state_v2(Local::now);

        // Border breathing effect state for consistent border color with other components
        let (breath_value, set_breath_value) = use_state_v2(|| 0.0f32);

        use_interval_v2(
            {
                // Set up animation interval
                move || {
                    // Simple counter for animation using update() to get fresh value
                    set_animation_step.update(|step| *step + 1);
                }
            },
            500, // Slower animation speed to reduce CPU usage
        );

        use_interval_v2(
            {
                // Set up clock update interval
                move || {
                    // Update current time every second
                    set_current_time.set(Local::now());
                }
            },
            1000, // Update every second
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

        // Create notification and mode icons with clock
        let right_status = create_notification_and_mode_icons(
            self.notification_level,
            self.app_mode,
            &self.theme,
            animation_step,
            current_time,
        );

        // Render right status icons
        let right_status_widget = Paragraph::new(right_status)
            .alignment(Alignment::Right)
            .block(
                Block::default()
                    .title(Span::styled(
                        " Info ",
                        Style::default()
                            .fg(self.theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color)),
            );

        right_status_widget.render(area, buffer);
    }
}

/// Create notification and mode icons with clock
fn create_notification_and_mode_icons(
    notification_level: NotificationLevel,
    app_mode: AppMode,
    theme: &'_ Theme,
    animation_step: usize,
    current_time: DateTime<Local>,
) -> Line<'_> {
    // Create notification icon
    let (notif_icon, notif_style) = match notification_level {
        NotificationLevel::None => ("🔔 0 ", Style::default().fg(theme.muted)),
        NotificationLevel::Low => ("🔔 1 ", Style::default().fg(theme.info)),
        NotificationLevel::Medium => ("🔔 2 ", Style::default().fg(theme.warning)),
        NotificationLevel::High => (
            // Blinking effect for high notification level
            if animation_step.is_multiple_of(2) {
                "🔔 3!"
            } else {
                "🔔 3 "
            },
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ),
    };

    // Create mode icon
    let (mode_icon, mode_style) = match app_mode {
        AppMode::Normal => ("[N]", Style::default().fg(theme.primary)),
        AppMode::Edit => ("[E]", Style::default().fg(theme.accent)),
        AppMode::View => ("[V]", Style::default().fg(theme.info)),
        AppMode::Command => ("[C]", Style::default().fg(theme.warning)),
    };

    // Format the current time
    let time_str = current_time.format("%H:%M:%S").to_string();
    let date_str = current_time.format("%Y-%m-%d").to_string();

    // Create clock with pulsing colon for seconds
    let clock_style = Style::default()
        .fg(theme.secondary)
        .add_modifier(Modifier::BOLD);
    let date_style = Style::default().fg(theme.muted);

    Line::from(vec![
        Span::styled(notif_icon, notif_style),
        Span::raw(" "),
        Span::styled(mode_icon, mode_style),
        Span::raw(" | "),
        Span::styled(time_str, clock_style),
        Span::raw(" "),
        Span::styled(format!("({})", date_str), date_style),
    ])
}
