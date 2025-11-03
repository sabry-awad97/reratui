//! 🔍 Query Hook Example - GitHub Repository Search
//!
//! A beautiful demonstration of the `use_query` hook with:
//! - 🎯 Smart caching with automatic background refresh
//! - 🔄 Retry logic with exponential backoff
//! - 📊 Real-time loading states and error handling
//! - 🎨 Professional UI with color-coded states
//! - ⌨️ Intuitive keyboard navigation
//! - 🚀 Manual refetch and cache invalidation

use std::time::Duration;

use reratui::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct GitHubRepo {
    name: String,
    description: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
    language: Option<String>,
    html_url: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct SearchResponse {
    total_count: u32,
    items: Vec<GitHubRepo>,
}

async fn search_github_repos(query: &str) -> Result<SearchResponse, String> {
    let url = format!(
        "https://api.github.com/search/repositories?q={}&sort=stars&per_page=5",
        query
    );

    let response = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "reratui-query-example")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

struct App;

impl Component for App {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let (search_query, set_search_query) = use_state(|| String::from("rust"));
        let current_query = search_query.get();

        // Query with caching and background refresh
        let query_options = QueryOptions {
            enabled: true,
            stale_time: Duration::from_secs(30), // Refresh every 30 seconds
            cache_time: Duration::from_secs(300), // Cache for 5 minutes
            retry: true,
            retry_attempts: 3,
        };

        // Clone for the query closure
        let query_for_fetch = current_query.clone();
        let query_result = use_query(
            current_query.clone(),
            move || {
                let query = query_for_fetch.clone();
                async move { search_github_repos(&query).await }
            },
            Some(query_options),
        );

        // Clone for keyboard handler
        let refetch = query_result.refetch.clone();
        let invalidate = query_result.invalidate.clone();

        // Keyboard controls
        use_keyboard_press(move |key| match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                request_exit();
            }
            KeyCode::Char('r') => {
                refetch();
            }
            KeyCode::Char('c') => {
                invalidate();
            }
            KeyCode::Char('1') => {
                set_search_query.set(String::from("rust"));
            }
            KeyCode::Char('2') => {
                set_search_query.set(String::from("javascript"));
            }
            KeyCode::Char('3') => {
                set_search_query.set(String::from("python"));
            }
            KeyCode::Char('4') => {
                set_search_query.set(String::from("go"));
            }
            _ => {}
        });

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Search info
                Constraint::Min(10),   // Results
                Constraint::Length(5), // Status
                Constraint::Length(4), // Controls
            ])
            .split(area);

        render_title(buffer, chunks[0]);
        render_search_info(buffer, chunks[1], &current_query, &query_result);
        render_results(buffer, chunks[2], &query_result);
        render_status(buffer, chunks[3], &query_result);
        render_controls(buffer, chunks[4]);
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

    let title = Paragraph::new("🔍 GitHub Repository Search - Query Hook Demo")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(title_block);

    title.render(area, buffer);
}

fn render_search_info(
    buffer: &mut Buffer,
    area: Rect,
    query: &str,
    result: &QueryResult<SearchResponse, String>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .border_type(BorderType::Rounded);

    let status_icon = match result.status {
        QueryStatus::Idle => "⏸️",
        QueryStatus::Loading => "⏳",
        QueryStatus::Refreshing => "🔄",
        QueryStatus::Success => "✅",
        QueryStatus::Error => "❌",
    };

    let stale_indicator = if result.is_stale {
        " (refreshing...)"
    } else {
        ""
    };

    let text = Paragraph::new(format!(
        "{} Searching for: \"{}\"{}",
        status_icon, query, stale_indicator
    ))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::White))
    .block(block);

    text.render(area, buffer);
}

fn render_results(buffer: &mut Buffer, area: Rect, result: &QueryResult<SearchResponse, String>) {
    match result.status {
        QueryStatus::Idle => render_idle(buffer, area),
        QueryStatus::Loading => render_loading(buffer, area),
        QueryStatus::Refreshing => {
            if let Some(data) = &result.data {
                render_repos(buffer, area, data, true);
            } else {
                render_loading(buffer, area);
            }
        }
        QueryStatus::Success => {
            if let Some(data) = &result.data {
                render_repos(buffer, area, data, false);
            }
        }
        QueryStatus::Error => {
            if let Some(error) = &result.error {
                render_error(buffer, area, error);
            }
        }
    }
}

fn render_idle(buffer: &mut Buffer, area: Rect) {
    let block = Block::default()
        .title("💤 Idle")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let text = Paragraph::new("Waiting to search...")
        .alignment(Alignment::Center)
        .block(block);

    text.render(area, buffer);
}

fn render_loading(buffer: &mut Buffer, area: Rect) {
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
            "🔄 Searching GitHub repositories...",
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

fn render_repos(buffer: &mut Buffer, area: Rect, data: &SearchResponse, is_refreshing: bool) {
    let title = if is_refreshing {
        "🔄 Top Repositories (refreshing...)".to_string()
    } else {
        format!("✨ Top {} Repositories", data.items.len())
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    block.render(area, buffer);

    let mut lines = vec![Line::from("")];

    for (i, repo) in data.items.iter().enumerate() {
        // Repository name
        lines.push(Line::from(vec![
            Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::DarkGray)),
            Span::styled(
                &repo.name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Description
        if let Some(desc) = &repo.description {
            let truncated = if desc.len() > 60 {
                format!("{}...", &desc[..57])
            } else {
                desc.clone()
            };
            lines.push(Line::from(Span::styled(
                format!("   {}", truncated),
                Style::default().fg(Color::Gray),
            )));
        }

        // Stats
        lines.push(Line::from(vec![
            Span::styled("   ⭐ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{}", repo.stargazers_count),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled("  🍴 ", Style::default().fg(Color::Blue)),
            Span::styled(
                format!("{}", repo.forks_count),
                Style::default().fg(Color::Blue),
            ),
            Span::styled(
                format!("  {}", repo.language.as_deref().unwrap_or("Unknown")),
                Style::default().fg(Color::Magenta),
            ),
        ]));

        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    paragraph.render(inner, buffer);
}

fn render_error(buffer: &mut Buffer, area: Rect, error: &str) {
    let block = Block::default()
        .title("❌ Error")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠️  Failed to fetch repositories",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(error, Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'r' to retry",
            Style::default().fg(Color::Yellow),
        )),
    ])
    .alignment(Alignment::Center)
    .block(block);

    text.render(area, buffer);
}

fn render_status(buffer: &mut Buffer, area: Rect, result: &QueryResult<SearchResponse, String>) {
    let block = Block::default()
        .title("📊 Query Status")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    block.render(area, buffer);

    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Status
    let status_text = match result.status {
        QueryStatus::Idle => ("Idle", Color::Gray),
        QueryStatus::Loading => ("Loading", Color::Yellow),
        QueryStatus::Refreshing => ("Refreshing", Color::Cyan),
        QueryStatus::Success => ("Success", Color::Green),
        QueryStatus::Error => ("Error", Color::Red),
    };

    let status_line = Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Gray)),
        Span::styled(
            status_text.0,
            Style::default()
                .fg(status_text.1)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    Paragraph::new(vec![status_line]).render(status_chunks[0], buffer);

    // Cache info
    let cache_line = Line::from(vec![
        Span::styled("Cache: ", Style::default().fg(Color::Gray)),
        Span::styled(
            if result.data.is_some() {
                "✓ Cached"
            } else {
                "✗ No cache"
            },
            Style::default().fg(if result.data.is_some() {
                Color::Green
            } else {
                Color::Red
            }),
        ),
        Span::styled(
            if result.is_stale {
                "  (stale)"
            } else {
                "  (fresh)"
            },
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    Paragraph::new(vec![cache_line]).render(status_chunks[1], buffer);

    // Results count
    let count_line = if let Some(data) = &result.data {
        Line::from(vec![
            Span::styled("Results: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} / {}", data.items.len(), data.total_count),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("Results: ", Style::default().fg(Color::Gray)),
            Span::styled("N/A", Style::default().fg(Color::DarkGray)),
        ])
    };
    Paragraph::new(vec![count_line]).render(status_chunks[2], buffer);
}

fn render_controls(buffer: &mut Buffer, area: Rect) {
    let block = Block::default()
        .title("⌨️  Controls")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .border_type(BorderType::Rounded);

    let controls_text = vec![
        Line::from(vec![
            Span::styled(
                "1-4",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Change language  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Refetch  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "c",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Clear cache", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled(
                "Ctrl+Q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Quit  ", Style::default().fg(Color::Gray)),
            Span::styled("Auto-refresh: ", Style::default().fg(Color::DarkGray)),
            Span::styled("30s", Style::default().fg(Color::Cyan)),
        ]),
    ];

    let controls = Paragraph::new(controls_text)
        .block(block)
        .alignment(Alignment::Center);

    controls.render(area, buffer);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| App.into()).await?;
    Ok(())
}
