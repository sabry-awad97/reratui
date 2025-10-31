use crate::theme::Theme;
use reratui::prelude::*;

/// Connection status for the status icon
#[derive(Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
}

pub struct ConnectionStatusComponent {
    status: ConnectionStatus,
    theme: Theme,
}

impl ConnectionStatusComponent {
    pub fn new(status: ConnectionStatus, theme: Theme) -> Self {
        Self { status, theme }
    }

    /// Get the spans for the connection status without rendering to a frame
    pub fn get_spans<'a>(&'a self, spans: &mut Vec<Span<'a>>) {
        let status_line = create_status_icons(self.status, &self.theme);
        spans.push(status_line.spans[0].clone());
    }
}

impl Component for ConnectionStatusComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let mut spans = Vec::new();
        self.get_spans(&mut spans);

        let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);

        paragraph.render(area, buffer);
    }
}

/// Create connection status icon
fn create_status_icons(status: ConnectionStatus, theme: &'_ Theme) -> Line<'_> {
    let (icon, style) = match status {
        ConnectionStatus::Connected => (
            "● CONNECTED",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
        ),
        ConnectionStatus::Disconnected => (
            "○ DISCONNECTED",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        ConnectionStatus::Connecting => (
            "◌ CONNECTING",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
    };

    Line::from(vec![Span::styled(icon, style)])
}
