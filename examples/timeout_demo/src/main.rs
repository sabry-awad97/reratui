//! Timeout Hook Demo
//!
//! Demonstrates the use_timeout hook for scheduling one-time callbacks.
//! Shows auto-hide notifications, delayed actions, and controlled timeouts.

use reratui::prelude::*;
use std::time::Duration;

/// Main app component demonstrating different timeout patterns
#[component]
fn TimeoutDemo() -> Element {
    // Auto-hide notification after 5 seconds
    let (notification, set_notification) =
        use_state(|| Some("Welcome! This message will disappear in 5 seconds...".to_string()));

    use_timeout(
        {
            let set_notification = set_notification.clone();
            move || {
                set_notification.set(None);
            }
        },
        Duration::from_secs(5),
    );

    // Delayed loading state
    let (loading, set_loading) = use_state(|| true);

    use_timeout(
        {
            let set_loading = set_loading.clone();
            move || {
                set_loading.set(false);
            }
        },
        Duration::from_secs(2),
    );

    rsx! {
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(3),
                Constraint::Min(0),
            ]}
        >
            {/* Header */}
            <Block
                title="⏱️ Timeout Hook Demo"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Cyan)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"Demonstrating one-time delayed callbacks"}
                </Paragraph>
            </Block>

            {/* Main content */}
            <Layout
                direction={Direction::Vertical}
                margin={1}
                constraints={vec![
                    Constraint::Length(5),
                    Constraint::Length(7),
                    Constraint::Length(7),
                    Constraint::Min(0),
                ]}
            >
                {/* Auto-hide notification */}
                <NotificationCard notification={notification.get()} />

                {/* Loading state */}
                <LoadingCard loading={loading.get()} />

                {/* Resettable timeout */}
                <ResettableTimeoutCard />

                {/* Controlled timeout */}
                <ControlledTimeoutCard />
            </Layout>
        </Layout>
    }
}

/// Auto-hide notification card
#[component]
fn NotificationCard(notification: Option<String>) -> Element {
    rsx! {
        <Block
            title="📢 Auto-Hide Notification"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Yellow)}
        >
            <Layout margin={1}>
                <Paragraph>
                    {if let Some(msg) = notification {
                        msg
                    } else {
                        "Notification hidden!".to_string()
                    }}
                </Paragraph>
            </Layout>
        </Block>
    }
}

/// Loading state card
#[component]
fn LoadingCard(loading: bool) -> Element {
    rsx! {
        <Block
            title="⏳ Delayed Loading State"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Blue)}
        >
            <Layout
                direction={Direction::Vertical}
                margin={1}
                constraints={vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]}
            >
                <Paragraph>
                    {if loading {
                        "Status: Loading... (2 seconds)"
                    } else {
                        "Status: ✅ Loaded!"
                    }}
                </Paragraph>
                <Paragraph style={Style::default().fg(Color::Gray)}>
                    {"Simulates async data loading"}
                </Paragraph>
            </Layout>
        </Block>
    }
}

/// Resettable timeout card
#[component]
fn ResettableTimeoutCard() -> Element {
    let (message, set_message) = use_state(|| "Waiting for timeout...".to_string());
    let (reset_count, set_reset_count) = use_state(|| 0);

    let reset = use_timeout_with_reset(
        {
            let set_message = set_message.clone();
            move || {
                set_message.set("⏰ Timeout triggered!".to_string());
            }
        },
        Duration::from_secs(3),
    );

    // Simulate user activity resetting the timer
    use_interval(
        {
            let reset = reset.clone();
            let set_reset_count = set_reset_count.clone();
            let set_message = set_message.clone();
            move || {
                reset();
                set_reset_count.update(|c| c + 1);
                set_message.set(format!("Timer reset {} times", reset_count.get() + 1));
            }
        },
        Duration::from_millis(1500),
    );

    rsx! {
        <Block
            title="🔄 Resettable Timeout"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Green)}
        >
            <Layout
                direction={Direction::Vertical}
                margin={1}
                constraints={vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]}
            >
                <Paragraph>
                    {message.get()}
                </Paragraph>
                <Paragraph style={Style::default().fg(Color::Gray)}>
                    {"Auto-resets every 1.5s (simulating user activity)"}
                </Paragraph>
            </Layout>
        </Block>
    }
}

/// Controlled timeout card
#[component]
fn ControlledTimeoutCard() -> Element {
    let (status, set_status) = use_state(|| "Idle".to_string());

    let (start, cancel, is_active) = use_timeout_controlled(
        {
            let set_status = set_status.clone();
            move || {
                set_status.set("✅ Completed!".to_string());
            }
        },
        Duration::from_secs(4),
    );

    // Auto-start after 1 second
    use_timeout(
        {
            let start = start.clone();
            let set_status = set_status.clone();
            move || {
                start();
                set_status.set("⏳ Running... (4 seconds)".to_string());
            }
        },
        Duration::from_secs(1),
    );

    // Auto-cancel after 2.5 seconds (before completion)
    use_timeout(
        {
            let cancel = cancel.clone();
            let set_status = set_status.clone();
            move || {
                if is_active() {
                    cancel();
                    set_status.set("❌ Cancelled!".to_string());
                }
            }
        },
        Duration::from_millis(2500),
    );

    rsx! {
        <Block
            title="🎮 Controlled Timeout"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Magenta)}
        >
            <Layout
                direction={Direction::Vertical}
                margin={1}
                constraints={vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]}
            >
                <Paragraph>
                    {format!("Status: {}", status.get())}
                </Paragraph>
                <Paragraph style={Style::default().fg(Color::Gray)}>
                    {"Starts at 1s, cancelled at 2.5s"}
                </Paragraph>
            </Layout>
        </Block>
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| rsx! { <TimeoutDemo /> }).await?;
    Ok(())
}
