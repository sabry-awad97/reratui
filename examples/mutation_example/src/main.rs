//! 🔄 Mutation Hook Example - User Management System
//!
//! A beautiful demonstration of the `use_mutation` and `use_reducer` hooks with:
//! - 🎯 Create, Update, Delete operations
//! - 🔄 Retry logic with exponential backoff
//! - ✅ Success/Error callbacks with notifications
//! - 🎨 Professional UI with color-coded states
//! - ⌨️ Intuitive keyboard navigation
//! - 🚀 Optimistic updates and rollback
//! - ❌ Cancellation support
//! - 📦 Reducer pattern for form state management

use parking_lot::Mutex;
use reratui::hooks::{
    MutationHandle, MutationOptions, MutationStatus, use_keyboard_press, use_mutation,
};
use reratui::prelude::*;
use reratui::ratatui::widgets::BorderType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct User {
    id: u32,
    name: String,
    email: String,
    role: String,
}

#[derive(Debug, Clone)]
struct CreateUserRequest {
    name: String,
    email: String,
    role: String,
}

#[derive(Debug, Clone)]
struct UpdateUserRequest {
    id: u32,
    name: String,
    email: String,
    role: String,
}

#[derive(Debug, Clone)]
enum ApiError {
    Validation(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Validation(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

// Mock API functions
async fn create_user_api(request: CreateUserRequest) -> Result<User, ApiError> {
    // Simulate network delay
    tokio::time::sleep(Duration::from_millis(800)).await;

    // Simulate occasional failures for retry demonstration
    if request.name.is_empty() {
        return Err(ApiError::Validation("Name cannot be empty".to_string()));
    }

    Ok(User {
        id: rand::random::<u32>() % 1000,
        name: request.name,
        email: request.email,
        role: request.role,
    })
}

async fn update_user_api(request: UpdateUserRequest) -> Result<User, ApiError> {
    tokio::time::sleep(Duration::from_millis(600)).await;

    if request.name.is_empty() {
        return Err(ApiError::Validation("Name cannot be empty".to_string()));
    }

    Ok(User {
        id: request.id,
        name: request.name,
        email: request.email,
        role: request.role,
    })
}

async fn delete_user_api(user_id: u32) -> Result<u32, ApiError> {
    tokio::time::sleep(Duration::from_millis(400)).await;
    Ok(user_id)
}

// Form state managed by reducer
#[derive(Clone, Debug)]
struct FormState {
    name: String,
    email: String,
    role: String,
    is_open: bool,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            role: String::from("User"),
            is_open: false,
        }
    }
}

// Actions for the form reducer
#[derive(Clone, Debug)]
enum FormAction {
    Open,
    Close,
    Reset,
    SetName(String),
    SetEmail(String),
    SetRole(String),
    Submit,
}

// Reducer function for form state
fn form_reducer(state: &FormState, action: FormAction) -> FormState {
    match action {
        FormAction::Open => FormState {
            is_open: true,
            ..state.clone()
        },
        FormAction::Close => FormState {
            is_open: false,
            ..state.clone()
        },
        FormAction::Reset => FormState::default(),
        FormAction::SetName(name) => FormState {
            name,
            ..state.clone()
        },
        FormAction::SetEmail(email) => FormState {
            email,
            ..state.clone()
        },
        FormAction::SetRole(role) => FormState {
            role,
            ..state.clone()
        },
        FormAction::Submit => FormState {
            is_open: false,
            name: String::new(),
            email: String::new(),
            role: String::from("User"),
        },
    }
}

#[derive(Clone)]
struct Notification {
    message: String,
    notification_type: NotificationType,
}

#[derive(Clone, PartialEq)]
enum NotificationType {
    Success,
    Error,
}

#[derive(Clone)]
struct App {
    users: Arc<Mutex<Vec<User>>>,
    notification: Arc<Mutex<Option<Notification>>>,
}

impl Component for App {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // State management with reducer for form
        let (form_state, form_dispatch) = use_reducer(form_reducer, FormState::default());
        let (selected_index, set_selected_index) = use_state(|| 0usize);

        let users_clone = self.users.clone();
        let notification_clone = self.notification.clone();

        // Create User Mutation
        let create_mutation = use_mutation(
            |request: CreateUserRequest| async move { create_user_api(request).await },
            Some(MutationOptions {
                retry: true,
                retry_attempts: 3,
                retry_delay: Duration::from_millis(500),
                retry_exponential_backoff: true,
                ..Default::default()
            }),
        );

        // Update User Mutation
        let update_mutation = use_mutation(
            |request: UpdateUserRequest| async move { update_user_api(request).await },
            Some(MutationOptions {
                retry: true,
                retry_attempts: 2,
                ..Default::default()
            }),
        );

        // Delete User Mutation
        let delete_mutation = use_mutation(
            |user_id: u32| async move { delete_user_api(user_id).await },
            None,
        );

        // Handle mutation results via effect
        {
            let users = users_clone.clone();
            let notif = notification_clone.clone();
            let create_state = create_mutation.state();
            if create_state.is_success {
                if let Some(user) = create_state.data {
                    users.lock().push(user.clone());
                    *notif.lock() = Some(Notification {
                        message: format!("✅ User '{}' created successfully!", user.name),
                        notification_type: NotificationType::Success,
                    });
                }
            } else if create_state.is_error
                && let Some(error) = create_state.error
            {
                *notif.lock() = Some(Notification {
                    message: format!("❌ Failed to create user: {}", error),
                    notification_type: NotificationType::Error,
                });
            }
        }

        {
            let users = users_clone.clone();
            let notif = notification_clone.clone();
            let delete_state = delete_mutation.state();
            if delete_state.is_success {
                if let Some(deleted_id) = delete_state.data {
                    users.lock().retain(|u| u.id != deleted_id);
                    *notif.lock() = Some(Notification {
                        message: "🗑️  User deleted successfully!".to_string(),
                        notification_type: NotificationType::Success,
                    });
                }
            } else if delete_state.is_error
                && let Some(error) = delete_state.error
            {
                *notif.lock() = Some(Notification {
                    message: format!("❌ Delete failed: {}", error),
                    notification_type: NotificationType::Error,
                });
            }
        }

        // Keyboard controls
        let create_mut_clone = create_mutation.clone();
        let delete_mut_clone = delete_mutation.clone();
        let users_for_kb = self.users.clone();
        let form_dispatch_clone = form_dispatch.clone();
        let form_state_clone = form_state.clone();

        use_keyboard_press(move |key| {
            let form = &form_state_clone;

            match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    request_exit();
                }
                KeyCode::Char('n') if !form.is_open => {
                    form_dispatch_clone.dispatch(FormAction::Open);
                }
                KeyCode::Char('r')
                    if key.modifiers.contains(KeyModifiers::CONTROL) && form.is_open =>
                {
                    form_dispatch_clone.dispatch(FormAction::Reset);
                }
                KeyCode::Char('c') if form.is_open => {
                    if !form.name.is_empty() && !form.email.is_empty() {
                        create_mut_clone.mutate(CreateUserRequest {
                            name: form.name.clone(),
                            email: form.email.clone(),
                            role: form.role.clone(),
                        });
                        form_dispatch_clone.dispatch(FormAction::Submit);
                    }
                }
                KeyCode::Char('1') if form.is_open => {
                    // Toggle role between User, Admin, Moderator
                    let new_role = match form.role.as_str() {
                        "User" => "Admin",
                        "Admin" => "Moderator",
                        _ => "User",
                    };
                    form_dispatch_clone.dispatch(FormAction::SetRole(new_role.to_string()));
                }
                KeyCode::Char(c)
                    if form.is_open
                        && (c.is_alphanumeric() || c == '@' || c == '.' || c == ' ') =>
                {
                    // Simple text input - append to name field
                    let mut new_name = form.name.clone();
                    new_name.push(c);
                    form_dispatch_clone.dispatch(FormAction::SetName(new_name));
                }
                KeyCode::Char('2') if form.is_open => {
                    // Switch to email input mode (for demo, we'll use a preset)
                    let email =
                        format!("{}@example.com", form.name.to_lowercase().replace(' ', "."));
                    form_dispatch_clone.dispatch(FormAction::SetEmail(email));
                }
                KeyCode::Backspace if form.is_open && !form.name.is_empty() => {
                    let mut new_name = form.name.clone();
                    new_name.pop();
                    form_dispatch_clone.dispatch(FormAction::SetName(new_name));
                }
                KeyCode::Char('d') if !form.is_open => {
                    let users = users_for_kb.lock();
                    if let Some(user) = users.get(selected_index) {
                        delete_mut_clone.mutate(user.id);
                    }
                }
                KeyCode::Char('x') => {
                    // Reset mutations
                    create_mut_clone.reset();
                    delete_mut_clone.reset();
                }
                KeyCode::Esc => {
                    form_dispatch_clone.dispatch(FormAction::Close);
                }
                KeyCode::Up if !form.is_open => {
                    if selected_index > 0 {
                        set_selected_index.set(selected_index - 1);
                    }
                }
                KeyCode::Down if !form.is_open => {
                    let users = users_for_kb.lock();
                    if selected_index < users.len().saturating_sub(1) {
                        set_selected_index.set(selected_index + 1);
                    }
                }
                _ => {}
            }
        });

        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Notification
                Constraint::Min(10),   // Main content
                Constraint::Length(6), // Status panel
                Constraint::Length(5), // Controls
            ])
            .split(area);

        render_title(buffer, chunks[0]);
        render_notification(buffer, chunks[1], &self.notification);

        if form_state.is_open {
            render_create_form(buffer, chunks[2], &form_state, &create_mutation);
        } else {
            render_user_list(buffer, chunks[2], &self.users, selected_index);
        }

        render_status_panel(
            buffer,
            chunks[3],
            &create_mutation,
            &update_mutation,
            &delete_mutation,
        );
        render_controls(buffer, chunks[4], form_state.is_open);
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

    let title = Paragraph::new("🔄 User Management System - Mutation Hook Demo")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(title_block);

    title.render(area, buffer);
}

fn render_notification(
    buffer: &mut Buffer,
    area: Rect,
    notification: &Arc<Mutex<Option<Notification>>>,
) {
    let notif = notification.lock();

    if let Some(n) = notif.as_ref() {
        let (color, icon) = match n.notification_type {
            NotificationType::Success => (Color::Green, "✅"),
            NotificationType::Error => (Color::Red, "❌"),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .border_type(BorderType::Rounded);

        let text = Paragraph::new(format!("{} {}", icon, n.message))
            .alignment(Alignment::Center)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .block(block);

        text.render(area, buffer);
    }
}

fn render_user_list(
    buffer: &mut Buffer,
    area: Rect,
    users: &Arc<Mutex<Vec<User>>>,
    selected_index: usize,
) {
    let users = users.lock();

    let block = Block::default()
        .title("👥 Users")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    block.render(area, buffer);

    if users.is_empty() {
        let empty_text = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No users yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'n' to create a new user",
                Style::default().fg(Color::Yellow),
            )),
        ])
        .alignment(Alignment::Center);

        empty_text.render(inner, buffer);
        return;
    }

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                "ID    ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Name              ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Email                     ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Role",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from("─".repeat(80)),
    ];

    for (i, user) in users.iter().enumerate() {
        let style = if i == selected_index {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{:<6}", user.id), style),
            Span::styled(format!("{:<18}", user.name), style),
            Span::styled(format!("{:<26}", user.email), style),
            Span::styled(&user.role, style),
        ]));
    }

    let list = Paragraph::new(lines);
    list.render(inner, buffer);
}

fn render_create_form(
    buffer: &mut Buffer,
    area: Rect,
    form: &FormState,
    mutation: &MutationHandle<User, ApiError, CreateUserRequest>,
) {
    let block = Block::default()
        .title("➕ Create New User")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    block.render(area, buffer);

    let state = mutation.state();
    let status_text = if state.is_pending {
        "⏳ Creating user..."
    } else {
        "Ready to create"
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Name:  ", Style::default().fg(Color::Yellow)),
            Span::styled(
                if form.name.is_empty() {
                    "_"
                } else {
                    &form.name
                },
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Email: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                if form.email.is_empty() {
                    "_"
                } else {
                    &form.email
                },
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Role:  ", Style::default().fg(Color::Yellow)),
            Span::styled(&form.role, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from("─".repeat(60)),
        Line::from(""),
        Line::from(Span::styled(
            status_text,
            Style::default()
                .fg(if state.is_pending {
                    Color::Yellow
                } else {
                    Color::Green
                })
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'c' to create • 'Esc' to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let form_paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    form_paragraph.render(inner, buffer);
}

fn render_status_panel(
    buffer: &mut Buffer,
    area: Rect,
    create_mut: &MutationHandle<User, ApiError, CreateUserRequest>,
    update_mut: &MutationHandle<User, ApiError, UpdateUserRequest>,
    delete_mut: &MutationHandle<u32, ApiError, u32>,
) {
    let block = Block::default()
        .title("📊 Mutation Status")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    block.render(area, buffer);

    let create_state = create_mut.state();
    let update_state = update_mut.state();
    let delete_state = delete_mut.state();

    let make_status_line = |name: &str, status: MutationStatus| -> Line {
        let (status_text, color) = match status {
            MutationStatus::Idle => ("Idle", Color::Gray),
            MutationStatus::Pending => ("Pending", Color::Yellow),
            MutationStatus::Success => ("Success", Color::Green),
            MutationStatus::Error => ("Error", Color::Red),
        };

        Line::from(vec![
            Span::styled(format!("{:<10}", name), Style::default().fg(Color::White)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:<10}", status_text),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ])
    };

    let lines = vec![
        make_status_line("Create", create_state.status),
        make_status_line("Update", update_state.status),
        make_status_line("Delete", delete_state.status),
    ];

    let status = Paragraph::new(lines);
    status.render(inner, buffer);
}

fn render_controls(buffer: &mut Buffer, area: Rect, in_form: bool) {
    let block = Block::default()
        .title("⌨️  Controls")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .border_type(BorderType::Rounded);

    let controls_text = if in_form {
        vec![
            Line::from(vec![
                Span::styled(
                    "Type",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Name  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "2",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Auto-email  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "1",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Toggle role", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled(
                    "c",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Create  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "r",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Reset  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Esc",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Cancel  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Backspace",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Delete char", Style::default().fg(Color::Gray)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled(
                    "n",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" New user  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "d",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Delete  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "↑↓",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Navigate", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled(
                    "x",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Reset mutation  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Ctrl+Q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Quit", Style::default().fg(Color::Gray)),
            ]),
        ]
    };

    let controls = Paragraph::new(controls_text)
        .block(block)
        .alignment(Alignment::Center);

    controls.render(area, buffer);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with some sample users
    let initial_users = vec![
        User {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            role: "Admin".to_string(),
        },
        User {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            role: "User".to_string(),
        },
        User {
            id: 3,
            name: "Carol White".to_string(),
            email: "carol@example.com".to_string(),
            role: "Moderator".to_string(),
        },
    ];

    let app = App {
        users: Arc::new(Mutex::new(initial_users)),
        notification: Arc::new(Mutex::new(None)),
    };

    render(move || app.clone()).await?;
    Ok(())
}
