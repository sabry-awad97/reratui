mod animated_title;
mod connection_status;
pub mod dropdown_menu;
mod marquee;
pub mod menu_bar;
mod status_icons;
mod system_info;
mod utils;

pub use animated_title::AnimatedTitleComponent;
pub use connection_status::{ConnectionStatus, ConnectionStatusComponent};
pub use dropdown_menu::MenuAction;
pub use marquee::MarqueeComponent;
pub use menu_bar::MenuBar;
pub use status_icons::{AppMode, NotificationLevel, StatusIconsComponent};
pub use system_info::SystemInfoComponent;

use crate::theme::Theme;

use reratui::prelude::*;

pub struct Header {
    pub title: String,
    pub theme: Theme,
    pub connection_status: ConnectionStatus,
    pub notification_level: NotificationLevel,
    pub app_mode: AppMode,
    pub marquee_text: String,
}

impl Component for Header {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Create layout for header with menu bar, status icons, title, and marquee
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Menu bar - fixed height
                Constraint::Length(3), // Main header area - fixed height
                Constraint::Length(3), // Marquee area - fixed height
            ])
            .split(area);

        // Create and render the menu bar
        let menu_bar = MenuBar::new(self.theme.clone());
        menu_bar.render(main_chunks[0], buffer);

        // Check for menu actions
        if let Some(action) = dropdown_menu::get_last_menu_action() {
            match action {
                MenuAction::Exit => {
                    // Handle exit action
                    request_exit();
                }
                MenuAction::Custom(action_name) => {
                    // Handle custom actions
                    println!("Menu action: {}", action_name);
                }
                _ => {}
            }
        }

        // Split the main header area horizontally
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Left status icons and system info
                Constraint::Percentage(50), // Center title
                Constraint::Percentage(25), // Right status icons and clock
            ])
            .split(main_chunks[1]);

        // Create and render the system info and connection status component
        let system_info = SystemInfoComponent::new(self.theme.clone());
        let connection_status =
            ConnectionStatusComponent::new(self.connection_status, self.theme.clone());

        // Render the left side (connection status + system info)
        let left_component = LeftHeaderComponent {
            system_info,
            connection_status,
            theme: self.theme.clone(),
        };
        left_component.render(header_chunks[0], buffer);

        // Create and render the animated title component
        let animated_title = AnimatedTitleComponent::new(self.title.clone(), self.theme.clone());
        animated_title.render(header_chunks[1], buffer);

        // Create and render the status icons component
        let status_icons =
            StatusIconsComponent::new(self.notification_level, self.app_mode, self.theme.clone());
        status_icons.render(header_chunks[2], buffer);

        // Create and render the marquee component
        let marquee = MarqueeComponent::new(self.marquee_text.clone(), self.theme.clone());
        marquee.render(main_chunks[2], buffer);
    }
}

/// Component that combines system info and connection status
struct LeftHeaderComponent {
    system_info: SystemInfoComponent,
    connection_status: ConnectionStatusComponent,
    theme: Theme,
}

impl Component for LeftHeaderComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Get the rendered content from both components
        let mut system_info_spans = Vec::new();
        let mut connection_status_spans = Vec::new();

        // We need to capture the spans without rendering directly to the frame
        self.system_info.get_spans(&mut system_info_spans);
        self.connection_status
            .get_spans(&mut connection_status_spans);

        // Combine the spans
        let mut combined_spans = Vec::new();
        combined_spans.extend(connection_status_spans);
        combined_spans.push(Span::raw(" | "));
        combined_spans.extend(system_info_spans);

        // Create the combined widget
        let left_status_widget = Paragraph::new(Line::from(combined_spans))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .title(Span::styled(
                        " Status ",
                        Style::default()
                            .fg(self.theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(self.theme.border)),
            );

        // Render the combined widget
        left_status_widget.render(area, buffer);
    }
}
