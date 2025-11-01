use std::time::Duration;

use reratui::prelude::*;

/// A component that displays warning information about the panic test
#[component]
fn WarningCard() -> Element {
    rsx! {
        <Block
            title="⚠️ Warning"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Yellow)}
        >
            <Layout
                direction={Direction::Vertical}
                constraints={vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]}
            >
                <Paragraph>
                    {"This demo will panic after 3 seconds..."}
                </Paragraph>
                <Paragraph>
                    {""}
                </Paragraph>
                <Paragraph style={Style::default().fg(Color::Green)}>
                    {"The terminal should be properly restored"}
                </Paragraph>
                <Paragraph style={Style::default().fg(Color::Green)}>
                    {"after the panic."}
                </Paragraph>
            </Layout>
        </Block>
    }
}

/// A component that displays detailed information about the panic test
#[component]
fn PanicDetailsCard() -> Element {
    rsx! {
        <Block
            title="🔥 Panic Details"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Red)}
        >
            <Layout
                direction={Direction::Vertical}
                constraints={vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]}
            >
                <Paragraph style={Style::default().fg(Color::Red)}>
                    {"Type: Test Panic"}
                </Paragraph>
                <Paragraph style={Style::default().fg(Color::Yellow)}>
                    {"Trigger: 3 second timer"}
                </Paragraph>
                <Paragraph style={Style::default().fg(Color::Cyan)}>
                    {"Purpose: Terminal restoration test"}
                </Paragraph>
                <Paragraph style={Style::default().fg(Color::Magenta)}>
                    {"Expected: Clean exit"}
                </Paragraph>
            </Layout>
        </Block>
    }
}

/// A component that displays status information in the footer
#[component]
fn StatusFooter() -> Element {
    rsx! {
        <Layout
            direction={Direction::Horizontal}
            constraints={vec![
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ]}
        >
            <Block
                title="⏱️ Timer"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Blue)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"3 seconds"}
                </Paragraph>
            </Block>

            <Block
                title="🎯 Test Goal"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Green)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"Terminal Cleanup"}
                </Paragraph>
            </Block>

            <Block
                title="⚡ Framework"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Magenta)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"RSX Layout"}
                </Paragraph>
            </Block>
        </Layout>
    }
}

/// A component that demonstrates panic handling
struct PanicDemo {
    title: String,
}

impl PanicDemo {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

impl Component for PanicDemo {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Set up an effect to panic after 3 seconds
        use_interval(
            move || {
                // Panic!
                panic!("This is a test panic to verify terminal restoration");
            },
            Duration::from_secs(3),
        );

        // Create the entire layout using rsx! macro with component composition
        let ui = rsx! {
            <Layout
                direction={Direction::Vertical}
                margin={1}
                constraints={vec![
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(5),
                ]}
            >
                {/* Header */}
                <Block
                    title={self.title.clone()}
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)}
                >
                    <Paragraph alignment={Alignment::Center}>
                        {"🚨 Panic Test Demo"}
                    </Paragraph>
                </Block>

                {/* Main content area with component composition */}
                <Layout
                    direction={Direction::Horizontal}
                    constraints={vec![
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ]}
                >
                    <WarningCard />
                    <PanicDetailsCard />
                </Layout>

                {/* Footer using component */}
                <StatusFooter />
            </Layout>
        };

        // Render the entire layout
        ui.render(area, buffer);
    }
}

/// Entry point for the application
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(err) = render(|| PanicDemo::new("✨ Panic Test Demo ✨").into()).await {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}
