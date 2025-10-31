use std::time::{Duration, SystemTime};

use chrono::Local;
use reratui::prelude::*;

use crate::components::header::{AppMode, ConnectionStatus, NotificationLevel};
use crate::components::{CommandPaletteComponent, Header, HelpBar, MessageList};
use crate::hooks::use_command_palette;
use crate::models::{Message, MessageType};
use crate::theme::Theme;

pub struct CommandPaletteApp {
    pub title: String,
}

impl CommandPaletteApp {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }

    fn format_timestamp(time: SystemTime) -> String {
        let datetime: chrono::DateTime<Local> = time.into();
        datetime.format("%H:%M:%S").to_string()
    }
}

impl Component for CommandPaletteApp {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Set up themes
        let (theme_index, set_theme_index) = use_state(|| 0usize);
        let themes = vec![
            Theme::github_dark(),
            Theme::github_light(),
            Theme::nord_dark(),
        ];

        let theme = themes[theme_index.get()].clone();

        // Set up command palette
        let palette = use_command_palette();

        // Set up status states
        let (connection_status, set_connection_status) = use_state(|| ConnectionStatus::Connected);
        let (notification_level, set_notification_level) = use_state(|| NotificationLevel::None);
        let (app_mode, set_app_mode) = use_state(|| AppMode::Normal);

        use_interval(
            {
                // Simulate connection status changes
                let connection_status = connection_status.clone();
                let set_connection_status = set_connection_status.clone();

                move || {
                    // Cycle through connection statuses for demo purposes
                    match connection_status.get() {
                        ConnectionStatus::Connected => {
                            set_connection_status.set(ConnectionStatus::Connecting)
                        }
                        ConnectionStatus::Connecting => {
                            set_connection_status.set(ConnectionStatus::Disconnected)
                        }
                        ConnectionStatus::Disconnected => {
                            set_connection_status.set(ConnectionStatus::Connected)
                        }
                    }
                }
            },
            Duration::from_secs(5), // Change every 5 seconds
        );

        // Update app mode based on palette visibility
        if palette.is_palette_visible() {
            set_app_mode.set(AppMode::Command);
        } else {
            // Only set back to normal if we're currently in command mode
            if app_mode.get() == AppMode::Command {
                set_app_mode.set(AppMode::Normal);
            }
        }

        // Register theme commands
        {
            let theme_index = theme_index.clone();
            let set_theme_index = set_theme_index.clone();
            let themes_len = themes.len();
            palette.register("theme:next", "🎨 Switch to next theme", move || {
                let next_index = (theme_index.get() + 1) % themes_len;
                set_theme_index.set(next_index);
            });
        }

        {
            let theme_index = theme_index.clone();
            let set_theme_index = set_theme_index.clone();
            let themes_len = themes.len();
            palette.register(
                "theme:previous",
                "🎨 Switch to previous theme",
                move || {
                    let prev_index = if theme_index.get() == 0 {
                        themes_len - 1
                    } else {
                        theme_index.get() - 1
                    };
                    set_theme_index.set(prev_index);
                },
            );
        }

        // Set up messages state
        let (messages, set_messages) = use_state(Vec::<Message>::new);

        // Register message commands
        {
            let set_messages = set_messages.clone();
            let messages = messages.clone();
            let set_notification_level = set_notification_level.clone();
            palette.register(
                "greet",
                "👋 Display a friendly greeting message",
                move || {
                    set_messages.set({
                        let mut msgs = messages.get();
                        msgs.push(Message {
                            text: "Hello, World! Welcome to the enhanced command palette demo!"
                                .to_string(),
                            timestamp: Local::now(),
                            message_type: MessageType::Success,
                        });
                        msgs
                    });
                    // Increment notification level
                    set_notification_level.set(NotificationLevel::Low);
                },
            );
        }

        {
            let set_messages = set_messages.clone();
            let set_notification_level = set_notification_level.clone();
            palette.register(
                "clear",
                "🧹 Clear all messages from the display",
                move || {
                    set_messages.set(Vec::new());
                    // Reset notification level
                    set_notification_level.set(NotificationLevel::None);
                },
            );
        }

        {
            let set_messages = set_messages.clone();
            let messages = messages.clone();
            palette.register(
                "add timestamp",
                "🕒 Insert current timestamp",
                move || {
                    set_messages.set({
                        let mut msgs = messages.get();
                        msgs.push(Message {
                            text: format!(
                                "Current time: {}",
                                Self::format_timestamp(SystemTime::now())
                            ),
                            timestamp: Local::now(),
                            message_type: MessageType::Info,
                        });
                        msgs
                    });
                },
            );
        }

        {
            let set_messages = set_messages.clone();
            let messages = messages.clone();
            let set_notification_level = set_notification_level.clone();
            palette.register("warning", "⚠️ Add a warning message", move || {
                set_messages.set({
                    let mut msgs = messages.get();
                    msgs.push(Message {
                        text: "This is a warning message!".to_string(),
                        timestamp: Local::now(),
                        message_type: MessageType::Warning,
                    });
                    msgs
                });
                // Set medium notification level
                set_notification_level.set(NotificationLevel::Medium);
            });
        }

        {
            let set_messages = set_messages.clone();
            let messages = messages.clone();
            let set_notification_level = set_notification_level.clone();
            palette.register("error", "❌ Add an error message", move || {
                set_messages.set({
                    let mut msgs = messages.get();
                    msgs.push(Message {
                        text: "This is an error message!".to_string(),
                        timestamp: Local::now(),
                        message_type: MessageType::Error,
                    });
                    msgs
                });
                // Set high notification level
                set_notification_level.set(NotificationLevel::High);
            });
        }

        // Handle keyboard events
        if let Some(Event::Key(key)) = use_event()
            && key.kind == KeyEventKind::Press
        {
            match (key.code, key.modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    request_exit();
                }
                (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                    palette.show_palette();
                }
                (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                    // Toggle edit mode
                    set_app_mode.set(match app_mode.get() {
                        AppMode::Edit => AppMode::Normal,
                        _ => AppMode::Edit,
                    });
                }
                (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                    // Toggle view mode
                    set_app_mode.set(match app_mode.get() {
                        AppMode::View => AppMode::Normal,
                        _ => AppMode::View,
                    });
                }
                (KeyCode::Up, _) if palette.is_palette_visible() => {
                    let commands = palette.get_commands();
                    if !commands.is_empty() {
                        let current = palette.get_selected_index();
                        let new_index = if current > 0 {
                            current - 1
                        } else {
                            commands.len() - 1
                        };
                        palette.set_selected_index(new_index);
                    }
                }
                (KeyCode::Down, _) if palette.is_palette_visible() => {
                    let commands = palette.get_commands();
                    if !commands.is_empty() {
                        let current = palette.get_selected_index();
                        let new_index = if current + 1 < commands.len() {
                            current + 1
                        } else {
                            0
                        };
                        palette.set_selected_index(new_index);
                    }
                }
                (KeyCode::Enter, _) if palette.is_palette_visible() => {
                    let commands = palette.get_commands();
                    if !commands.is_empty()
                        && palette.get_selected_index() < commands.len()
                        && let Some(command) = commands.get(palette.get_selected_index())
                    {
                        palette.execute(&command.name);
                        palette.hide_palette();
                    }
                }
                (KeyCode::Char(c), _) if palette.is_palette_visible() => {
                    let mut filter = palette.get_filter();
                    filter.push(c);
                    palette.set_filter(filter);
                }
                (KeyCode::Backspace, _) if palette.is_palette_visible() => {
                    let mut filter = palette.get_filter();
                    filter.pop();
                    palette.set_filter(filter);
                }
                (KeyCode::Esc, _) if palette.is_palette_visible() => {
                    palette.hide_palette();
                }
                _ => {}
            }
        }

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Header (1 for menu + 3 for main header + 3 for marquee)
                Constraint::Min(1),    // Message list
                Constraint::Length(3), // Help bar
            ])
            .split(area);

        // Render components
        Header {
            title: self.title.clone(),
            theme: theme.clone(),
            connection_status: connection_status.get(),
            notification_level: notification_level.get(),
            app_mode: app_mode.get(),
            marquee_text: "🔔 Welcome to the Command Palette Demo! Press Ctrl+P to open the command palette. Press Ctrl+E to toggle edit mode. Press Ctrl+V to toggle view mode. 🚀 Explore all the features and enjoy the enhanced UI! 🎉".to_string(),
        }
        .render(chunks[0], buffer);

        MessageList {
            messages: messages.get(),
            theme: theme.clone(),
        }
        .render(chunks[1], buffer);

        HelpBar {
            theme: theme.clone(),
        }
        .render(chunks[2], buffer);

        // Render all active dropdown menus (on top of other UI elements)
        let active_menus = crate::components::header::menu_bar::get_active_menus();
        for (menu, area) in active_menus {
            menu.render_dropdown(area, buffer);
        }

        // Render command palette if visible (on top of everything)
        if palette.is_palette_visible() {
            CommandPaletteComponent { palette, theme }.render(area, buffer);
        }
    }
}
