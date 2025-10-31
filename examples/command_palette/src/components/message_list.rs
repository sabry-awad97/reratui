use reratui::prelude::*;

use crate::models::Message;
use crate::theme::Theme;

pub struct MessageList {
    pub messages: Vec<Message>,
    pub theme: Theme,
}

impl Component for MessageList {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Use the theme passed from the parent component
        let theme = &self.theme;

        // Render messages with timestamps and styling
        let messages_text: Vec<Line> = self
            .messages
            .iter()
            .map(|msg| {
                let (style, prefix) = match msg.message_type {
                    crate::models::MessageType::Info => (theme.info_style(), "ℹ"),
                    crate::models::MessageType::Success => (theme.success_style(), "✓"),
                    crate::models::MessageType::Warning => (theme.warning_style(), "⚠"),
                    crate::models::MessageType::Error => (theme.error_style(), "✗"),
                };

                Line::from(vec![
                    Span::styled(
                        format!("[{}] ", msg.timestamp.format("%H:%M:%S")),
                        theme.muted_style(),
                    ),
                    Span::styled(format!("{} ", prefix), style),
                    Span::styled(&msg.text, style),
                ])
            })
            .collect();

        let messages_widget = Paragraph::new(messages_text).block(
            Block::default()
                .title("Messages")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border)),
        );

        messages_widget.render(area, buffer);
    }
}
