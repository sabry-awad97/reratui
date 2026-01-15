//! Events Showcase Demo
//!
//! Demonstrates event handling with Component:
//! - Keyboard events
//! - Mouse events
//! - Terminal resize handling
//!
//! Press 'q' or Esc to exit

use crossterm::event::{MouseButton, MouseEventKind};
use reratui::prelude::*;

struct EventsShowcase;

impl Component for EventsShowcase {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // State for tracking events
        let (last_key, set_last_key) = use_state(|| "None".to_string());
        let (key_count, set_key_count) = use_state(|| 0u32);
        let (mouse_pos, set_mouse_pos) = use_state(|| (0u16, 0u16));
        let (mouse_event, set_mouse_event) = use_state(|| "None".to_string());
        let (click_count, set_click_count) = use_state(|| 0u32);

        // Handle events
        if let Some(event) = use_event() {
            match event {
                Event::Key(KeyEvent {
                    code,
                    kind: KeyEventKind::Press,
                    modifiers,
                    ..
                }) => {
                    // Exit on q or Esc
                    if matches!(code, KeyCode::Char('q') | KeyCode::Esc) {
                        request_exit();
                    }

                    // Build key description
                    let key_desc = match code {
                        KeyCode::Char(c) => {
                            if c == ' ' {
                                "Space".to_string()
                            } else {
                                format!("'{}'", c)
                            }
                        }
                        KeyCode::Enter => "Enter".to_string(),
                        KeyCode::Backspace => "Backspace".to_string(),
                        KeyCode::Tab => "Tab".to_string(),
                        KeyCode::Up => "↑".to_string(),
                        KeyCode::Down => "↓".to_string(),
                        KeyCode::Left => "←".to_string(),
                        KeyCode::Right => "→".to_string(),
                        _ => format!("{:?}", code),
                    };

                    let mut mods = Vec::new();
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        mods.push("Ctrl");
                    }
                    if modifiers.contains(KeyModifiers::ALT) {
                        mods.push("Alt");
                    }
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        mods.push("Shift");
                    }

                    let full_desc = if mods.is_empty() {
                        key_desc
                    } else {
                        format!("{} + {}", mods.join(" + "), key_desc)
                    };

                    set_last_key.set(full_desc);
                    set_key_count.update(|c| c + 1);
                }
                Event::Mouse(mouse) => {
                    set_mouse_pos.set((mouse.column, mouse.row));

                    let event_type = match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            set_click_count.update(|c| c + 1);
                            "Left Click"
                        }
                        MouseEventKind::Down(MouseButton::Right) => "Right Click",
                        MouseEventKind::Down(MouseButton::Middle) => "Middle Click",
                        MouseEventKind::Up(_) => "Button Up",
                        MouseEventKind::Drag(_) => "Dragging",
                        MouseEventKind::Moved => "Moved",
                        MouseEventKind::ScrollDown => "Scroll Down",
                        MouseEventKind::ScrollUp => "Scroll Up",
                        MouseEventKind::ScrollLeft => "Scroll Left",
                        MouseEventKind::ScrollRight => "Scroll Right",
                    };
                    set_mouse_event.set(event_type.to_string());
                }
                Event::Resize(_, _) => {
                    // Resize is handled automatically
                }
                _ => {}
            }
        }

        // Layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Main content
                Constraint::Length(5), // Instructions
            ])
            .split(area);

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[1]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(7), Constraint::Min(5)])
            .split(content_chunks[0]);

        // Title
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let title = Paragraph::new("🎯 Event Hooks Showcase - Keyboard + Mouse")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(title_block);
        title.render(main_chunks[0], buffer);

        // Terminal info
        let terminal_block = Block::default()
            .title("Terminal Size")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let terminal_info = Paragraph::new(format!("\nSize: {}x{}\n", area.width, area.height))
            .alignment(Alignment::Center)
            .block(terminal_block);
        terminal_info.render(left_chunks[0], buffer);

        // Keyboard info
        let keyboard_block = Block::default()
            .title("Keyboard Events")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let keyboard_info =
            Paragraph::new(format!("\nLast Key: {}\nCount: {}\n", last_key, key_count))
                .alignment(Alignment::Center)
                .block(keyboard_block);
        keyboard_info.render(left_chunks[1], buffer);

        // Mouse info
        let mouse_block = Block::default()
            .title("Mouse Events")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));
        let mouse_info = Paragraph::new(format!(
            "\nPosition: ({}, {})\nEvent: {}\nClicks: {}\n",
            mouse_pos.0, mouse_pos.1, mouse_event, click_count
        ))
        .alignment(Alignment::Center)
        .block(mouse_block);
        mouse_info.render(content_chunks[1], buffer);

        // Instructions
        let instructions_block = Block::default()
            .title("Instructions")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let instructions = Paragraph::new(
            "Press any key to see it tracked | Move/click mouse | Press 'q' or Esc to quit",
        )
        .alignment(Alignment::Center)
        .block(instructions_block);
        instructions.render(main_chunks[2], buffer);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| EventsShowcase).await?;
    Ok(())
}
