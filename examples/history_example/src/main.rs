//! 📝 History Management Example
//!
//! A beautiful demonstration of the `use_history` hook with undo/redo functionality.
//! Features:
//! - 🎨 Rich text editor with history tracking
//! - ⏮️ Undo/Redo operations with keyboard shortcuts
//! - 📊 Visual history timeline
//! - 🎯 Real-time state visualization
//! - ⌨️ Intuitive keyboard controls

use reratui::prelude::*;

#[derive(Clone, PartialEq, Debug)]
struct EditorState {
    content: String,
    cursor_position: usize,
    operation: String,
}

impl EditorState {
    fn new() -> Self {
        Self {
            content: String::from(
                "Welcome to the History Example!\nTry typing and using Ctrl+Z/Ctrl+Y for undo/redo.",
            ),
            cursor_position: 0,
            operation: String::from("Initial"),
        }
    }
}

struct App;

impl Component for App {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let history = use_history(EditorState::new(), 50);

        // Clone history for use in the keyboard handler
        let history_for_handler = history.clone();

        // Keyboard controls
        use_keyboard_press(move |key| {
            let current_state = history_for_handler.current();
            let mut new_state = current_state.clone();

            match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    request_exit();
                }
                KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    history_for_handler.undo();
                }
                KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    history_for_handler.redo();
                }
                KeyCode::Char(c) => {
                    new_state.content.push(c);
                    new_state.cursor_position += 1;
                    new_state.operation = format!("Added '{}'", c);
                    history_for_handler.push(new_state);
                }
                KeyCode::Backspace => {
                    if !new_state.content.is_empty() && new_state.cursor_position > 0 {
                        new_state.content.pop();
                        new_state.cursor_position = new_state.cursor_position.saturating_sub(1);
                        new_state.operation = String::from("Deleted character");
                        history_for_handler.push(new_state);
                    }
                }
                KeyCode::Enter => {
                    new_state.content.push('\n');
                    new_state.cursor_position += 1;
                    new_state.operation = String::from("New line");
                    history_for_handler.push(new_state);
                }
                _ => {}
            }
        });

        // Get current state after setting up keyboard handler
        let current_state = history.current();

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Editor
                Constraint::Length(5), // Status
                Constraint::Length(4), // Controls
            ])
            .split(area);

        // Render title
        render_title(buffer, chunks[0]);

        // Render editor
        render_editor(buffer, chunks[1], &current_state);

        // Render status
        render_status(buffer, chunks[2], &history, &current_state);

        // Render controls
        render_controls(buffer, chunks[3]);
    }
}

fn render_title(buffer: &mut Buffer, area: Rect) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .border_type(BorderType::Double);

    let title = Paragraph::new("📝 History Management - Text Editor with Undo/Redo")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(title_block);

    title.render(area, buffer);
}

fn render_editor(buffer: &mut Buffer, area: Rect, state: &EditorState) {
    let editor_block = Block::default()
        .title("✏️  Editor")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .border_type(BorderType::Rounded);

    // Split content into lines for display
    let lines: Vec<Line> = state
        .content
        .lines()
        .map(|line| {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::White),
            ))
        })
        .collect();

    let editor = Paragraph::new(lines)
        .block(editor_block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));

    editor.render(area, buffer);
}

fn render_status<T: Clone + 'static>(
    buffer: &mut Buffer,
    area: Rect,
    history: &HistoryManager<T>,
    state: &EditorState,
) {
    let status_block = Block::default()
        .title("📊 Status")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .border_type(BorderType::Rounded);

    let inner = status_block.inner(area);
    status_block.render(area, buffer);

    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Last operation
    let operation_line = Line::from(vec![
        Span::styled("Last Operation: ", Style::default().fg(Color::Gray)),
        Span::styled(
            &state.operation,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let operation = Paragraph::new(vec![operation_line]);
    operation.render(status_chunks[0], buffer);

    // Character count
    let char_count_line = Line::from(vec![
        Span::styled("Characters: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", state.content.len()),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let char_count = Paragraph::new(vec![char_count_line]);
    char_count.render(status_chunks[1], buffer);

    // History status
    let undo_status = if history.can_undo() {
        Span::styled("✓ Available", Style::default().fg(Color::Green))
    } else {
        Span::styled("✗ Unavailable", Style::default().fg(Color::Red))
    };

    let redo_status = if history.can_redo() {
        Span::styled("✓ Available", Style::default().fg(Color::Green))
    } else {
        Span::styled("✗ Unavailable", Style::default().fg(Color::Red))
    };

    let history_line = Line::from(vec![
        Span::styled("Undo: ", Style::default().fg(Color::Gray)),
        undo_status,
        Span::styled("  |  Redo: ", Style::default().fg(Color::Gray)),
        redo_status,
    ]);
    let history_status = Paragraph::new(vec![history_line]);
    history_status.render(status_chunks[2], buffer);
}

fn render_controls(buffer: &mut Buffer, area: Rect) {
    let controls_block = Block::default()
        .title("⌨️  Controls")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .border_type(BorderType::Rounded);

    let controls_text = vec![
        Line::from(vec![
            Span::styled(
                "Ctrl+Z",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Undo  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Ctrl+Y",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Redo  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Ctrl+Q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Quit", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled(
                "Type",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to add text  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Backspace",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to delete", Style::default().fg(Color::Gray)),
        ]),
    ];

    let controls = Paragraph::new(controls_text)
        .block(controls_block)
        .alignment(Alignment::Center);

    controls.render(area, buffer);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| App.into()).await?;
    Ok(())
}
