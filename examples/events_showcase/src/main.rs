//! Events Showcase Demo
//!
//! Demonstrates all event hooks working together:
//! - use_keyboard
//! - use_mouse
//! - use_on_resize
//!
//! Features:
//! - Unified event dashboard
//! - Real-time event visualization
//! - Interactive drawing canvas
//! - Terminal resize handling

use reratui::prelude::*;

// State structure for the entire showcase
#[derive(Clone)]
struct ShowcaseState {
    // Terminal
    terminal_size: (u16, u16),
    resize_count: u32,
    // Keyboard
    last_key: String,
    key_count: u32,
    // Mouse
    mouse_pos: (u16, u16),
    mouse_event_type: String,
    click_count: u32,
    // Canvas
    canvas_pixels: Vec<(u16, u16)>,
    drawing_mode: bool,
}

// Actions for state updates
#[derive(Clone)]
enum ShowcaseAction {
    // Terminal actions
    Resize { width: u16, height: u16 },
    // Keyboard actions
    KeyPress { description: String },
    ToggleDrawing,
    ClearCanvas,
    // Mouse actions
    MouseMove { x: u16, y: u16, event_type: String },
    MouseClick,
    AddPixel { x: u16, y: u16 },
}

// Reducer function
fn showcase_reducer(state: ShowcaseState, action: ShowcaseAction) -> ShowcaseState {
    match action {
        ShowcaseAction::Resize { width, height } => ShowcaseState {
            terminal_size: (width, height),
            resize_count: state.resize_count + 1,
            ..state
        },
        ShowcaseAction::KeyPress { description } => ShowcaseState {
            last_key: description,
            key_count: state.key_count + 1,
            ..state
        },
        ShowcaseAction::ToggleDrawing => ShowcaseState {
            drawing_mode: !state.drawing_mode,
            ..state
        },
        ShowcaseAction::ClearCanvas => ShowcaseState {
            canvas_pixels: Vec::new(),
            ..state
        },
        ShowcaseAction::MouseMove { x, y, event_type } => ShowcaseState {
            mouse_pos: (x, y),
            mouse_event_type: event_type,
            ..state
        },
        ShowcaseAction::MouseClick => ShowcaseState {
            click_count: state.click_count + 1,
            ..state
        },
        ShowcaseAction::AddPixel { x, y } => {
            let mut new_pixels = state.canvas_pixels.clone();
            new_pixels.push((x, y));
            // Keep only last 500 pixels
            if new_pixels.len() > 500 {
                new_pixels.drain(0..100);
            }
            ShowcaseState {
                canvas_pixels: new_pixels,
                ..state
            }
        }
    }
}

#[derive(Clone)]
struct EventsShowcase;

impl Component for EventsShowcase {
    fn on_mount(&self) {
        // Set up exit handlers
        on_global_event(KeyCode::Char('q'), || {
            request_exit();
            true // Stop event propagation - event is handled
        });

        on_global_event(KeyCode::Esc, || {
            request_exit();
            true // Stop event propagation - event is handled
        });
    }

    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Initialize state with reducer
        let initial_state = ShowcaseState {
            terminal_size: (area.width, area.height),
            resize_count: 0,
            last_key: String::from("None"),
            key_count: 0,
            mouse_pos: (0, 0),
            mouse_event_type: String::from("None"),
            click_count: 0,
            canvas_pixels: Vec::new(),
            drawing_mode: false,
        };

        let (state, dispatch) = use_reducer(showcase_reducer, initial_state);

        // Handle resize events
        use_on_resize({
            let dispatch = dispatch.clone();
            move |(width, height)| {
                dispatch.call(ShowcaseAction::Resize { width, height });
            }
        });

        // Handle keyboard events
        use_keyboard_press({
            let dispatch = dispatch.clone();

            move |key_event| {
                // Handle special keys first
                if let KeyCode::Char(c) = key_event.code {
                    if c == 'd' {
                        dispatch.call(ShowcaseAction::ToggleDrawing);
                    }
                    if c == 'c' {
                        dispatch.call(ShowcaseAction::ClearCanvas);
                    }
                }

                // Build key description
                let key_desc = match key_event.code {
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
                    _ => format!("{:?}", key_event.code),
                };

                let mut modifiers = Vec::new();
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    modifiers.push("Ctrl");
                }
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    modifiers.push("Alt");
                }
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    modifiers.push("Shift");
                }

                let full_desc = if modifiers.is_empty() {
                    key_desc
                } else {
                    format!("{} + {}", modifiers.join(" + "), key_desc)
                };

                dispatch.call(ShowcaseAction::KeyPress {
                    description: full_desc,
                });
            }
        });

        // Handle mouse events
        use_mouse({
            let dispatch = dispatch.clone();
            let drawing = state.field(|s| s.drawing_mode);

            move |mouse_event| {
                let event_type = match mouse_event.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        dispatch.call(ShowcaseAction::MouseClick);
                        "Left Click"
                    }
                    MouseEventKind::Down(MouseButton::Right) => "Right Click",
                    MouseEventKind::Down(MouseButton::Middle) => "Middle Click",
                    MouseEventKind::Up(_) => "Button Up",
                    MouseEventKind::Drag(MouseButton::Left) => {
                        // Add pixel to canvas if in drawing mode
                        if drawing {
                            dispatch.call(ShowcaseAction::AddPixel {
                                x: mouse_event.column,
                                y: mouse_event.row,
                            });
                        }
                        "Dragging"
                    }
                    MouseEventKind::Drag(_) => "Dragging",
                    MouseEventKind::Moved => "Moved",
                    MouseEventKind::ScrollDown => "Scroll Down",
                    MouseEventKind::ScrollUp => "Scroll Up",
                    MouseEventKind::ScrollLeft => "Scroll Left",
                    MouseEventKind::ScrollRight => "Scroll Right",
                };

                dispatch.call(ShowcaseAction::MouseMove {
                    x: mouse_event.column,
                    y: mouse_event.row,
                    event_type: event_type.to_string(),
                });
            }
        });

        // Create layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Main content
                Constraint::Length(8), // Instructions
            ])
            .split(area);

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[1]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Terminal info
                Constraint::Length(7), // Keyboard info
                Constraint::Min(5),    // Mouse info
            ])
            .split(content_chunks[0]);

        // Title
        let title = Paragraph::new("🎯 Event Hooks Showcase - Keyboard + Mouse + Resize")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        title.render(main_chunks[0], buffer);

        // Terminal info
        let (width, height) = state.field(|s| s.terminal_size);
        let resize_count = state.field(|s| s.resize_count);
        let terminal_info = Paragraph::new(vec![
            Line::from(Span::styled(
                "Terminal Size",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}x{}", width, height),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Resizes: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", resize_count),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("use_on_resize")
                .border_style(Style::default().fg(Color::Green)),
        );
        terminal_info.render(left_chunks[0], buffer);

        // Keyboard info
        let last_key = state.field(|s| s.last_key.clone());
        let key_count = state.field(|s| s.key_count);
        let keyboard_info = Paragraph::new(vec![
            Line::from(Span::styled(
                "Keyboard Events",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Last Key: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    last_key,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Count: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}", key_count), Style::default().fg(Color::Cyan)),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("use_keyboard")
                .border_style(Style::default().fg(Color::Yellow)),
        );
        keyboard_info.render(left_chunks[1], buffer);

        // Mouse info
        let (mx, my) = state.field(|s| s.mouse_pos);
        let mouse_event_type = state.field(|s| s.mouse_event_type.clone());
        let click_count = state.field(|s| s.click_count);
        let mouse_info = Paragraph::new(vec![
            Line::from(Span::styled(
                "Mouse Events",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Position: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("({}, {})", mx, my),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Event: ", Style::default().fg(Color::Gray)),
                Span::styled(mouse_event_type, Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("Clicks: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}", click_count), Style::default().fg(Color::Red)),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("use_mouse")
                .border_style(Style::default().fg(Color::Magenta)),
        );
        mouse_info.render(left_chunks[2], buffer);

        // Canvas area
        let drawing_mode = state.field(|s| s.drawing_mode);
        let canvas_pixels = state.field(|s| s.canvas_pixels.clone());

        let canvas_title = if drawing_mode {
            "🎨 Drawing Canvas (ACTIVE - Drag to draw!)"
        } else {
            "🎨 Drawing Canvas (Press 'd' to activate)"
        };

        let canvas_border_color = if drawing_mode {
            Color::Green
        } else {
            Color::Blue
        };

        let mut canvas_lines = vec![Line::from("")];

        // Show pixel count
        canvas_lines.push(Line::from(vec![
            Span::styled("Pixels: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", canvas_pixels.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        canvas_lines.push(Line::from(""));

        if drawing_mode {
            canvas_lines.push(Line::from(Span::styled(
                "Click and drag to draw!",
                Style::default().fg(Color::Yellow),
            )));
        } else {
            canvas_lines.push(Line::from(Span::styled(
                "Press 'd' to enable drawing",
                Style::default().fg(Color::Gray),
            )));
        }

        let canvas = Paragraph::new(canvas_lines)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .title(canvas_title)
                    .border_style(Style::default().fg(canvas_border_color)),
            );
        canvas.render(content_chunks[1], buffer);

        // Render canvas pixels
        let canvas_area = content_chunks[1].inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        for (px, py) in canvas_pixels.iter() {
            if *px >= canvas_area.x
                && *px < canvas_area.x + canvas_area.width
                && *py >= canvas_area.y
                && *py < canvas_area.y + canvas_area.height
            {
                let pixel = Paragraph::new("●").style(Style::default().fg(Color::Cyan));
                pixel.render(
                    Rect {
                        x: *px,
                        y: *py,
                        width: 1,
                        height: 1,
                    },
                    buffer,
                );
            }
        }

        // Instructions
        let instructions = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "🎯 All Event Hooks Active!",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "'d'",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " to toggle drawing mode  |  ",
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    "'c'",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to clear canvas", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "'q'",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" or ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Esc",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to quit  |  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Resize",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " the terminal to see it tracked!",
                    Style::default().fg(Color::Gray),
                ),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Instructions")
                .border_style(Style::default().fg(Color::Yellow)),
        );
        instructions.render(main_chunks[2], buffer);
    }
}

/// Entry point for the application
#[reratui::main]
async fn main() -> Result<()> {
    render(|| EventsShowcase.into()).await?;
    Ok(())
}
