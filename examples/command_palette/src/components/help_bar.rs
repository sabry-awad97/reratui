use reratui::prelude::*;

use crate::theme::Theme;

pub struct HelpBar {
    pub theme: Theme,
}

impl Component for HelpBar {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Use the theme passed from the parent component
        let theme = &self.theme;

        // Render help text
        let help = Paragraph::new(vec![Line::from(vec![
            Span::styled("Ctrl+P", theme.accent_style()),
            Span::raw(": Show Command Palette  "),
            Span::styled("Ctrl+C", theme.accent_style()),
            Span::raw(": Exit"),
        ])])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border)),
        );

        help.render(area, buffer);
    }
}
