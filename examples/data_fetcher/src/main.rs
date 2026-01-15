//! Data Fetcher Example with use_future Hook
//!
//! A beautiful async data fetching application demonstrating:
//! - use_future hook for async operations
//! - Elegant loading states
//! - Multiple data sources with different loading times
//! - Global and individual refresh functionality
//!
//! Controls:
//! - Press 'r' to refresh all data sources
//! - Press '1-4' to refresh individual sources
//! - Press 'q' to exit

use reratui::hooks::{FutureState, use_future};
use reratui::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

/// Simulates fetching user data from an API
async fn fetch_user_data() -> Result<String, String> {
    sleep(Duration::from_millis(1500)).await;
    Ok("👤 John Doe | john@example.com | Premium User".to_string())
}

/// Simulates fetching weather data
async fn fetch_weather_data() -> Result<String, String> {
    sleep(Duration::from_millis(2000)).await;
    Ok("☀️ Sunny | 24°C | Humidity: 65% | Wind: 12 km/h".to_string())
}

/// Simulates fetching statistics
async fn fetch_stats() -> Result<String, String> {
    sleep(Duration::from_millis(1000)).await;
    Ok("📊 Active Users: 1,234 | Total Revenue: $45,678 | Growth: +15%".to_string())
}

/// Simulates fetching notifications
async fn fetch_notifications() -> Result<String, String> {
    sleep(Duration::from_millis(800)).await;
    Ok("🔔 3 new messages | 2 updates | 1 alert".to_string())
}

struct DataFetcherApp;

impl Component for DataFetcherApp {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Refresh triggers
        let (refresh_count, set_refresh_count) = use_state(|| 0u32);
        let (user_refresh, set_user_refresh) = use_state(|| 0u32);
        let (weather_refresh, set_weather_refresh) = use_state(|| 0u32);
        let (stats_refresh, set_stats_refresh) = use_state(|| 0u32);
        let (notifications_refresh, set_notifications_refresh) = use_state(|| 0u32);

        // Handle keyboard events
        if let Some(Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        })) = use_event()
        {
            match code {
                KeyCode::Char('r') => set_refresh_count.update(|c| c + 1),
                KeyCode::Char('1') => set_user_refresh.update(|c| c + 1),
                KeyCode::Char('2') => set_weather_refresh.update(|c| c + 1),
                KeyCode::Char('3') => set_stats_refresh.update(|c| c + 1),
                KeyCode::Char('4') => set_notifications_refresh.update(|c| c + 1),
                KeyCode::Char('q') => request_exit(),
                _ => {}
            }
        }

        // Fetch data from multiple sources
        let user_data = use_future(fetch_user_data, Some((refresh_count, user_refresh)));
        let weather_data = use_future(fetch_weather_data, Some((refresh_count, weather_refresh)));
        let stats_data = use_future(fetch_stats, Some((refresh_count, stats_refresh)));
        let notifications_data = use_future(
            fetch_notifications,
            Some((refresh_count, notifications_refresh)),
        );

        // Calculate overall progress
        let futures = [&user_data, &weather_data, &stats_data, &notifications_data];
        let completed = futures
            .iter()
            .filter(|h| matches!(h.state(), FutureState::Resolved(_)))
            .count();

        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(4), // User data
                Constraint::Length(4), // Weather data
                Constraint::Length(4), // Stats data
                Constraint::Length(4), // Notifications
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header
        let status = if completed == 4 {
            format!("✓ All data loaded! | Refresh #{}", refresh_count)
        } else {
            format!("⏳ Loading... ({}/4)", completed)
        };
        let header_block = Block::default()
            .title("🚀 Async Data Fetcher Demo")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let header = Paragraph::new(status)
            .alignment(Alignment::Center)
            .block(header_block);
        header.render(chunks[0], buffer);

        // Render data cards
        render_data_card(buffer, chunks[1], "👤 User Profile [1]", &user_data);
        render_data_card(buffer, chunks[2], "🌤️  Weather Info [2]", &weather_data);
        render_data_card(buffer, chunks[3], "📊 Statistics [3]", &stats_data);
        render_data_card(
            buffer,
            chunks[4],
            "🔔 Notifications [4]",
            &notifications_data,
        );

        // Footer
        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let footer = Paragraph::new("[r] Refresh All | [1-4] Refresh Individual | [q] Quit")
            .alignment(Alignment::Center)
            .block(footer_block);
        footer.render(chunks[5], buffer);
    }
}

fn render_data_card(
    buffer: &mut Buffer,
    area: Rect,
    title: &str,
    handle: &reratui::hooks::FutureHandle<String, String>,
) {
    let (border_color, content) = match handle.state() {
        FutureState::Idle => (Color::Gray, "⏸️  Not started".to_string()),
        FutureState::Pending => (Color::Yellow, "⏳ Loading...".to_string()),
        FutureState::Resolved(data) => (Color::Green, format!("✓ {}", data)),
        FutureState::Error(err) => (Color::Red, format!("✗ Error: {}", err)),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let paragraph = Paragraph::new(content)
        .alignment(Alignment::Left)
        .block(block);
    paragraph.render(area, buffer);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| DataFetcherApp).await?;
    Ok(())
}
