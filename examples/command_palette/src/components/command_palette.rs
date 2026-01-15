use reratui::prelude::*;
use reratui::ratatui::widgets::{BorderType, Clear, List, ListItem};

use crate::hooks::CommandPalette;
use crate::theme::Theme;
use crate::utils::centered_rect;

pub struct CommandPaletteComponent {
    pub palette: CommandPalette,
    pub theme: Theme,
}

impl Component for CommandPaletteComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let (_frame_count, set_frame_count) = use_state(|| 0usize);
        let (cursor_visible, set_cursor_visible) = use_state(|| true);

        use_interval(
            move || {
                set_frame_count.update(|count| *count + 1);
                // Toggle cursor visibility every 10 frames
                set_cursor_visible.update(|visible| !*visible);
            },
            500, // milliseconds - slower for cursor blink
        );

        let popup_area = centered_rect(60, 40, area);

        let block = Block::default()
            .title("⌘ Command Palette")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.theme.accent))
            .style(Style::default().bg(self.theme.background));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(2),
            ])
            .split(popup_area);

        let filter = self.palette.get_filter();
        let cursor = if self.theme.is_dark {
            if cursor_visible { "▎" } else { "│" }
        } else if cursor_visible {
            "┃"
        } else {
            "│"
        };

        let input = Paragraph::new(format!("{}{}", filter, cursor))
            .style(self.theme.text_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Search")
                    .border_style(Style::default().fg(self.theme.muted)),
            );

        Clear.render(popup_area, buffer);
        block.render(popup_area, buffer);
        input.render(chunks[0], buffer);

        let commands = self.palette.get_commands();
        let items: Vec<ListItem> = commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let is_selected = i == self.palette.get_selected_index();
                let (icon, style) = if is_selected {
                    (
                        "➤ ",
                        Style::default()
                            .fg(self.theme.background)
                            .bg(self.theme.accent)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ("  ", Style::default().fg(self.theme.foreground))
                };

                ListItem::new(vec![Line::from(vec![
                    Span::styled(icon, style),
                    Span::styled(&cmd.name, style.add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(&cmd.description, style),
                ])])
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Available Commands")
                .border_style(Style::default().fg(self.theme.muted)),
        );

        Widget::render(list, chunks[1], buffer);

        let help = Paragraph::new(vec![Line::from(vec![
            Span::styled("↑/↓", self.theme.accent_style()),
            Span::raw(": Navigate  "),
            Span::styled("Enter", self.theme.accent_style()),
            Span::raw(": Execute  "),
            Span::styled("Esc", self.theme.accent_style()),
            Span::raw(": Close"),
        ])])
        .alignment(Alignment::Center)
        .style(Style::default().fg(self.theme.muted));

        help.render(chunks[2], buffer);
    }
}
