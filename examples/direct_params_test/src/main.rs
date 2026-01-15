//! Direct Parameters Test - Component with Direct Rendering
//!
//! Demonstrates using Component trait with direct widget rendering.

use reratui::prelude::*;

struct DirectParamsDemo {
    title: String,
}

impl DirectParamsDemo {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

impl Component for DirectParamsDemo {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Handle exit
        if let Some(Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            kind: KeyEventKind::Press,
            ..
        })) = use_event()
        {
            request_exit();
        }

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(5), // Footer
            ])
            .split(area);

        // Header
        let header_block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        let header = Paragraph::new("🚀 Direct Parameters + Component Demo")
            .alignment(Alignment::Center)
            .block(header_block);
        header.render(chunks[0], buffer);

        // Main content - horizontal split
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Left side - Counter info
        let left_block = Block::default()
            .title("📊 Layout Features")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        let left_content =
            Paragraph::new("✅ Nested Layouts\n✅ Direct Parameters\n✅ Component Trait")
                .block(left_block);
        left_content.render(main_chunks[0], buffer);

        // Right side - User card
        let right_block = Block::default()
            .title("👤 User Card")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let right_content =
            Paragraph::new("Name: Alice Johnson\nAge: 28\nEmail: alice@example.com")
                .block(right_block);
        right_content.render(main_chunks[1], buffer);

        // Footer - three columns
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(chunks[2]);

        let labels = [
            ("🎯 Component Types", Color::Green, "Direct Params"),
            ("🎨 Layout System", Color::Yellow, "Nested Layouts"),
            ("⚡ Performance", Color::Magenta, "Zero Runtime Cost"),
        ];

        for (i, (title, color, content)) in labels.iter().enumerate() {
            let block = Block::default()
                .title(*title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(*color));
            let para = Paragraph::new(*content)
                .alignment(Alignment::Center)
                .block(block);
            para.render(footer_chunks[i], buffer);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| DirectParamsDemo::new("✨ Direct Parameters Test ✨")).await?;
    Ok(())
}
