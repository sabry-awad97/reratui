//! 🎨 Beautiful Async Fetch Example - Direct Rendering Version
//!
//! A modern demonstration of `use_future` hook without RSX/VNode

use reratui::prelude::*;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
struct Post {
    id: u32,
    title: String,
    body: String,
    #[serde(rename = "userId")]
    user_id: u32,
}

struct App;

impl Component for App {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // State management
        let (post_id, set_post_id) = use_state(|| 1u32);
        let current_post_id = post_id.get();

        // Fetch post data with automatic refetch on ID change
        let future_handle = use_future(
            move || async move {
                // Simulate network delay for better UX demonstration
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                let url = format!(
                    "https://jsonplaceholder.typicode.com/posts/{}",
                    current_post_id
                );

                let response = reqwest::get(&url)
                    .await
                    .map_err(|e| format!("Network error: {}", e))?;

                if !response.status().is_success() {
                    return Err(format!("API error: {}", response.status()));
                }

                let post: Post = response
                    .json()
                    .await
                    .map_err(|e| format!("JSON parsing error: {}", e))?;

                Ok::<Post, String>(post)
            },
            current_post_id,
        );

        let future_state = future_handle.state();

        // Keyboard navigation
        use_keyboard_press(move |key| match key.code {
            KeyCode::Char('q') => {
                request_exit();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if current_post_id > 1 {
                    set_post_id.set(current_post_id - 1);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if current_post_id < 100 {
                    set_post_id.set(current_post_id + 1);
                }
            }
            KeyCode::Home => {
                set_post_id.set(1);
            }
            KeyCode::End => {
                set_post_id.set(100);
            }
            _ => {}
        });

        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render header
        render_header(buffer, chunks[0], current_post_id);

        // Render content based on state
        render_content(buffer, chunks[1], &future_state);

        // Render footer
        render_footer(buffer, chunks[2], current_post_id);
    }
}

fn render_header(buffer: &mut Buffer, area: Rect, post_id: u32) {
    let block = Block::default()
        .title("📡 Async Data Fetching Demo")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let text = Paragraph::new(format!("Post #{} / 100", post_id))
        .alignment(Alignment::Center)
        .block(block);

    text.render(area, buffer);
}

fn render_content(buffer: &mut Buffer, area: Rect, state: &FutureState<Post, String>) {
    match state {
        FutureState::Idle => {
            let block = Block::default()
                .title("⏸️  Idle")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray));

            let text = Paragraph::new("Waiting to start...")
                .alignment(Alignment::Center)
                .block(block);

            text.render(area, buffer);
        }
        FutureState::Pending => {
            let block = Block::default()
                .title("⏳ Loading...")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

            let text = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "🔄 Fetching data from API...",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Please wait...",
                    Style::default().fg(Color::Gray),
                )),
            ])
            .alignment(Alignment::Center)
            .block(block);

            text.render(area, buffer);
        }
        FutureState::Resolved(post) => {
            let block = Block::default()
                .title(format!("✅ Post #{}", post.id))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                );

            let text = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    &post.title,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )),
                Line::from(""),
                Line::from(Span::styled(&post.body, Style::default().fg(Color::White))),
                Line::from(""),
                Line::from(vec![
                    Span::styled("User ID: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{}", post.user_id),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
            ])
            .alignment(Alignment::Left)
            .block(block);

            text.render(area, buffer);
        }
        FutureState::Error(error) => {
            let block = Block::default()
                .title("❌ Error")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

            let text = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "⚠️  Failed to fetch data",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(error, Style::default().fg(Color::Gray))),
                Line::from(""),
                Line::from(Span::styled(
                    "Try navigating to another post",
                    Style::default().fg(Color::Yellow),
                )),
            ])
            .alignment(Alignment::Center)
            .block(block);

            text.render(area, buffer);
        }
    }
}

fn render_footer(buffer: &mut Buffer, area: Rect, post_id: u32) {
    let nav_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let text_style = Style::default().fg(Color::Gray);

    let can_go_prev = post_id > 1;
    let can_go_next = post_id < 100;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    // Create inner layout for three columns
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(inner);

    // Render block first
    block.render(area, buffer);

    // Left column
    let left_line = Line::from(vec![
        Span::styled("← h", if can_go_prev { nav_style } else { text_style }),
        Span::styled(" Previous  ", text_style),
        Span::styled("Home", nav_style),
        Span::styled(" First", text_style),
    ]);
    let left_para = Paragraph::new(vec![left_line]).alignment(Alignment::Left);
    left_para.render(chunks[0], buffer);

    // Center column
    let center_line = Line::from(vec![
        Span::styled("q", nav_style),
        Span::styled(" Quit", text_style),
    ]);
    let center_para = Paragraph::new(vec![center_line]).alignment(Alignment::Center);
    center_para.render(chunks[1], buffer);

    // Right column
    let right_line = Line::from(vec![
        Span::styled("Next ", text_style),
        Span::styled("l →  ", if can_go_next { nav_style } else { text_style }),
        Span::styled("Last ", text_style),
        Span::styled("End", nav_style),
    ]);
    let right_para = Paragraph::new(vec![right_line]).alignment(Alignment::Right);
    right_para.render(chunks[2], buffer);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| App.into()).await?;
    Ok(())
}
