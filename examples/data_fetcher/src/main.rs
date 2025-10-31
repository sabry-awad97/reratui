//! Data Fetcher Example with use_future Hook
//!
//! A beautiful async data fetching application demonstrating:
//! - use_future hook for async operations
//! - Elegant loading states with animations
//! - Modern UI with gradients and styling
//! - Multiple data sources with different loading times
//! - Global and individual refresh functionality
//! - Interactive cards with hover effects
//! - Keyboard shortcuts and mouse support
//!
//! Controls:
//! - Press 'r' to refresh all data sources
//! - Press '1' to refresh User Profile
//! - Press '2' to refresh Weather Info
//! - Press '3' to refresh Statistics
//! - Press '4' to refresh Notifications
//! - Click on any card to refresh it individually
//! - Hover over cards to see refresh hints
//! - Press 'q' to exit

use reratui::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

/// Custom hook for detecting hover state over a component's area
///
/// Returns `true` when the mouse is hovering over the component's bounding box
fn use_hover() -> bool {
    let area = use_area();
    let (is_hovered, set_hovered) = use_state(|| false);

    if let Some(event) = use_event()
        && let Event::Mouse(mouse) = event
    {
        let in_bounds = mouse.column >= area.x
            && mouse.column < area.x + area.width
            && mouse.row >= area.y
            && mouse.row < area.y + area.height;

        if in_bounds != is_hovered.get() {
            set_hovered.set(in_bounds);
        }
    }

    is_hovered.get()
}

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

/// Loading spinner component with animation
#[component]
fn LoadingSpinner() -> Element {
    let frame = use_frame();
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let idx = (frame.count / 3) as usize % spinner_chars.len();
    let spinner = spinner_chars[idx];

    rsx! {
        <Paragraph
            style={Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)}
            alignment={Alignment::Center}
        >
            {format!("{} Loading...", spinner)}
        </Paragraph>
    }
}

/// Props for the Marquee component
#[derive(Props)]
struct MarqueeProps {
    text: String,
    style: Option<Style>,
    alignment: Option<Alignment>,
    scroll_speed_ms: Option<u64>,
}

/// Marquee component with smooth scrolling animation
///
/// Uses `use_interval` for precise timing control
#[component]
fn Marquee(props: &MarqueeProps) -> Element {
    let (scroll_offset, set_scroll_offset) = use_state(|| 0usize);

    // Get scroll speed (default: 50ms per character)
    let scroll_speed = Duration::from_millis(props.scroll_speed_ms.unwrap_or(50));

    // Update scroll position on interval
    use_interval(
        {
            let set_scroll_offset = set_scroll_offset.clone();
            let text = props.text.clone();
            move || {
                let char_count = text.chars().count();
                if char_count > 0 {
                    set_scroll_offset.update(|offset| (offset + 1) % char_count);
                }
            }
        },
        scroll_speed,
    );

    // Create seamless loop by repeating the text
    let repeated_text = format!("{}{}{}", props.text, props.text, props.text);

    // Skip characters safely to avoid splitting multi-byte characters
    let displayed_text: String = repeated_text.chars().skip(scroll_offset.get()).collect();

    let style = props.style.unwrap_or_default();
    let alignment = props.alignment.unwrap_or(Alignment::Left);

    rsx! {
        <Paragraph
            style={style}
            alignment={alignment}
        >
            {displayed_text}
        </Paragraph>
    }
}

#[derive(Props)]
struct DataCardProps {
    title: String,
    future_handle: FutureHandle<String, String>,
    refresh_key: char,
    on_refresh: Callback<()>,
}

/// Data card component for displaying fetched data
#[component]
fn DataCard(props: &DataCardProps) -> Element {
    let area = use_area();
    let is_hovered = use_hover();

    // Handle click to refresh
    if let Some(event) = use_event()
        && let Event::Mouse(mouse) = event
    {
        let in_bounds = mouse.column >= area.x
            && mouse.column < area.x + area.width
            && mouse.row >= area.y
            && mouse.row < area.y + area.height;

        if in_bounds && mouse.kind == MouseEventKind::Down(MouseButton::Left) {
            props.on_refresh.emit(());
        }
    }

    // Handle keyboard shortcut for refresh
    use_keyboard_shortcut(KeyCode::Char(props.refresh_key), KeyModifiers::NONE, {
        let on_refresh = props.on_refresh.clone();
        move || {
            on_refresh.emit(());
        }
    });

    let border_color = match props.future_handle.state() {
        FutureState::Idle => Color::Gray,
        FutureState::Pending => Color::Yellow,
        FutureState::Resolved(_) => {
            if is_hovered {
                Color::Cyan
            } else {
                Color::Green
            }
        }
        FutureState::Error(_) => Color::Red,
    };

    let title_style = Style::default()
        .fg(Color::White)
        .bg(if is_hovered {
            Color::Rgb(37, 99, 235) // Brighter blue on hover
        } else {
            Color::Rgb(59, 130, 246)
        })
        .add_modifier(Modifier::BOLD);

    let title_with_hint = if is_hovered {
        format!(
            "{} [Press '{}' or Click to refresh]",
            props.title, props.refresh_key
        )
    } else {
        props.title.clone()
    };

    rsx! {
        <Block
            title={title_with_hint}
            title_style={title_style}
            borders={Borders::ALL}
            border_style={Style::default().fg(border_color)}
            style={Style::default().bg(Color::Rgb(30, 30, 46))}
        >
            {match props.future_handle.state() {
                FutureState::Idle => rsx! {
                    <Paragraph
                        style={Style::default().fg(Color::DarkGray)}
                        alignment={Alignment::Center}
                    >
                        {"⏸️  Not started"}
                    </Paragraph>
                },
                FutureState::Pending => rsx! {
                    <LoadingSpinner />
                },
                FutureState::Resolved(data) => rsx! {
                    <Paragraph
                        style={Style::default().fg(Color::White)}
                        alignment={Alignment::Left}
                    >
                        {format!("✓ {}", data)}
                    </Paragraph>
                },
                FutureState::Error(err) => rsx! {
                    <Paragraph
                        style={Style::default().fg(Color::Red)}
                        alignment={Alignment::Center}
                    >
                        {format!("✗ Error: {}", err)}
                    </Paragraph>
                },
            }}
        </Block>
    }
}

/// Main application component
#[component]
fn DataFetcherApp() -> Element {
    let frame = use_frame();

    // Refresh triggers - individual counters for each data source
    let (refresh_count, set_refresh_count) = use_state(|| 0);
    let (user_refresh, set_user_refresh) = use_state(|| 0);
    let (weather_refresh, set_weather_refresh) = use_state(|| 0);
    let (stats_refresh, set_stats_refresh) = use_state(|| 0);
    let (notifications_refresh, set_notifications_refresh) = use_state(|| 0);

    // Handle global refresh (r key)
    use_keyboard_shortcut(KeyCode::Char('r'), KeyModifiers::NONE, {
        let set_refresh_count = set_refresh_count.clone();
        move || {
            set_refresh_count.update(|c| c + 1);
        }
    });

    // Handle exit (q key)
    use_keyboard_shortcut(KeyCode::Char('q'), KeyModifiers::NONE, move || {
        request_exit()
    });

    // Fetch data from multiple sources (re-runs when either global or individual refresh changes)
    let user_data = use_future(fetch_user_data, (refresh_count.get(), user_refresh.get()));
    let weather_data = use_future(
        fetch_weather_data,
        (refresh_count.get(), weather_refresh.get()),
    );
    let stats_data = use_future(fetch_stats, (refresh_count.get(), stats_refresh.get()));
    let notifications_data = use_future(
        fetch_notifications,
        (refresh_count.get(), notifications_refresh.get()),
    );

    // Calculate overall progress
    let completed = [&user_data, &weather_data, &stats_data, &notifications_data]
        .iter()
        .filter(|h| h.is_resolved())
        .count();
    let total = 4;

    // Animated title
    let pulse = ((frame.count as f32 / 10.0).sin() * 0.5 + 0.5) * 255.0;
    let title_color = Color::Rgb((59.0 + pulse * 0.3) as u8, (130.0 + pulse * 0.2) as u8, 246);
    let title_style = Style::default()
        .fg(title_color)
        .add_modifier(Modifier::BOLD);

    // Status message for marquee
    let status_msg = if completed == total {
        format!(
            "✓ All data loaded successfully! | Press 'r' to refresh all | Press '1-4' or click cards to refresh individually | Refresh #{} ",
            refresh_count.get()
        )
    } else {
        format!(
            "⏳ Loading data from multiple sources... ({}/{}) | Press 'r' to refresh all | Press '1-4' or click cards ",
            completed, total
        )
    };

    let status_color = if completed == total {
        Color::Green
    } else {
        Color::Yellow
    };

    let status_style = Style::default()
        .fg(status_color)
        .add_modifier(Modifier::BOLD);

    rsx! {
        <Block
            title={"🚀 Async Data Fetcher Demo"}
            title_style={title_style}
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Cyan)}
            style={Style::default().bg(Color::Rgb(17, 17, 27))}
        >
            // Header with status
            <Layout
                direction={Direction::Vertical}
                constraints={vec![
                    Constraint::Length(2),
                    Constraint::Min(0),
                ]}
            >
                <Block
                    borders={Borders::BOTTOM}
                    border_style={Style::default().fg(Color::DarkGray)}
                    style={Style::default().bg(Color::Rgb(24, 24, 37))}
                >
                    <Marquee
                        text={status_msg}
                        style={status_style}
                        alignment={Alignment::Center}
                        scroll_speed_ms={250}
                    />
                </Block>

                // Data cards grid
                <Layout direction={Direction::Vertical} constraints={vec![
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ]}>
                    <DataCard
                        title={"👤 User Profile"}
                        future_handle={user_data}
                        refresh_key={'1'}
                        on_refresh={{
                            let set_user_refresh = set_user_refresh.clone();
                            move |_| set_user_refresh.update(|c| c + 1)
                        }}
                    />
                    <DataCard
                        title={"🌤️  Weather Info"}
                        future_handle={weather_data}
                        refresh_key={'2'}
                        on_refresh={{
                            let set_weather_refresh = set_weather_refresh.clone();
                            move |_| set_weather_refresh.update(|c| c + 1)
                        }}
                    />
                    <DataCard
                        title={"📊 Statistics"}
                        future_handle={stats_data}
                        refresh_key={'3'}
                        on_refresh={{
                            let set_stats_refresh = set_stats_refresh.clone();
                            move |_| set_stats_refresh.update(|c| c + 1)
                        }}
                    />
                    <DataCard
                        title={"🔔 Notifications"}
                        future_handle={notifications_data}
                        refresh_key={'4'}
                        on_refresh={{
                            let set_notifications_refresh = set_notifications_refresh.clone();
                            move |_| set_notifications_refresh.update(|c| c + 1)
                        }}
                    />
                </Layout>
            </Layout>
        </Block>
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| {
        rsx! { <DataFetcherApp /> }
    })
    .await?;
    Ok(())
}
