//! Responsive Layout Example
//!
//! Demonstrates the use_media_query hook for creating responsive TUI layouts
//! that adapt to different terminal sizes.

use reratui::prelude::*;

/// Main app component with responsive layout
#[component]
fn ResponsiveApp() -> Element {
    // Define breakpoints using media queries
    let is_mobile = use_media_query(|(w, _)| w < 60);
    let is_tablet = use_media_query(|(w, _)| (60..120).contains(&w));
    let is_desktop = use_media_query(|(w, _)| w >= 120);

    // Get actual dimensions for display
    let (width, height) = use_terminal_dimensions();

    // Determine layout based on screen size
    let layout_name = if is_mobile {
        "Mobile"
    } else if is_tablet {
        "Tablet"
    } else if is_desktop {
        "Desktop"
    } else {
        "Unknown"
    };

    rsx! {
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]}
        >
            {/* Header */}
            <Header layout_name={layout_name.to_string()} />

            {/* Main content - changes based on screen size */}
            {if is_mobile {
                rsx! { <MobileLayout width={width} height={height} /> }
            } else if is_tablet {
                rsx! { <TabletLayout width={width} height={height} /> }
            } else {
                rsx! { <DesktopLayout width={width} height={height} /> }
            }}

            {/* Footer */}
            <Footer width={width} height={height} />
        </Layout>
    }
}

/// Header component
#[component]
fn Header(layout_name: String) -> Element {
    rsx! {
        <Block
            title={format!("📱 Responsive Layout Demo - {}", layout_name)}
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Cyan)}
        >
            <Paragraph alignment={Alignment::Center}>
                {"Resize your terminal to see the layout adapt!"}
            </Paragraph>
        </Block>
    }
}

/// Mobile layout (< 60 columns)
#[component]
fn MobileLayout(width: u16, height: u16) -> Element {
    rsx! {
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Percentage(100),
            ]}
        >
            <Block
                title="📱 Mobile View"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Yellow)}
            >
                <Layout
                    direction={Direction::Vertical}
                    margin={1}
                    constraints={vec![
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ]}
                >
                    <Paragraph>
                        {format!("Terminal: {}x{}", width, height)}
                    </Paragraph>
                    <Paragraph>
                        {"Layout: Single column"}
                    </Paragraph>
                    <Paragraph>
                        {"Navigation: Stacked"}
                    </Paragraph>
                    <Paragraph>
                        {"Content: Full width"}
                    </Paragraph>
                </Layout>
            </Block>
        </Layout>
    }
}

/// Tablet layout (60-119 columns)
#[component]
fn TabletLayout(width: u16, height: u16) -> Element {
    rsx! {
        <Layout
            direction={Direction::Horizontal}
            constraints={vec![
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ]}
        >
            <Block
                title="📋 Sidebar"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Green)}
            >
                <Layout
                    direction={Direction::Vertical}
                    margin={1}
                    constraints={vec![
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ]}
                >
                    <Paragraph>
                        {"• Home"}
                    </Paragraph>
                    <Paragraph>
                        {"• Settings"}
                    </Paragraph>
                </Layout>
            </Block>

            <Block
                title="📄 Main Content"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Blue)}
            >
                <Layout
                    direction={Direction::Vertical}
                    margin={1}
                    constraints={vec![
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ]}
                >
                    <Paragraph>
                        {format!("Terminal: {}x{}", width, height)}
                    </Paragraph>
                    <Paragraph>
                        {"Layout: Two columns"}
                    </Paragraph>
                    <Paragraph>
                        {"Sidebar: 30% width"}
                    </Paragraph>
                </Layout>
            </Block>
        </Layout>
    }
}

/// Desktop layout (>= 120 columns)
#[component]
fn DesktopLayout(width: u16, height: u16) -> Element {
    rsx! {
        <Layout
            direction={Direction::Horizontal}
            constraints={vec![
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]}
        >
            <Block
                title="📋 Left Panel"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Green)}
            >
                <Layout
                    direction={Direction::Vertical}
                    margin={1}
                    constraints={vec![
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ]}
                >
                    <Paragraph>
                        {"• Dashboard"}
                    </Paragraph>
                    <Paragraph>
                        {"• Projects"}
                    </Paragraph>
                    <Paragraph>
                        {"• Settings"}
                    </Paragraph>
                </Layout>
            </Block>

            <Block
                title="📄 Main Content"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Blue)}
            >
                <Layout
                    direction={Direction::Vertical}
                    margin={1}
                    constraints={vec![
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ]}
                >
                    <Paragraph>
                        {format!("Terminal: {}x{}", width, height)}
                    </Paragraph>
                    <Paragraph>
                        {"Layout: Three columns"}
                    </Paragraph>
                    <Paragraph>
                        {"Left: 20% | Center: 60% | Right: 20%"}
                    </Paragraph>
                    <Paragraph style={Style::default().fg(Color::Green)}>
                        {"✨ Full desktop experience"}
                    </Paragraph>
                </Layout>
            </Block>

            <Block
                title="ℹ️ Info Panel"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Magenta)}
            >
                <Layout
                    direction={Direction::Vertical}
                    margin={1}
                    constraints={vec![
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ]}
                >
                    <Paragraph>
                        {"Stats"}
                    </Paragraph>
                    <Paragraph>
                        {"Notifications"}
                    </Paragraph>
                </Layout>
            </Block>
        </Layout>
    }
}

/// Footer component
#[component]
fn Footer(width: u16, height: u16) -> Element {
    // Check if terminal is very small
    let is_very_small = use_media_query(|(w, h)| w < 40 || h < 15);

    rsx! {
        <Block
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Gray)}
        >
            <Paragraph alignment={Alignment::Center}>
                {if is_very_small {
                    format!("{}x{} - Resize for better view", width, height)
                } else {
                    format!("Terminal: {}x{} | Press 'q' to quit", width, height)
                }}
            </Paragraph>
        </Block>
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| rsx! { <ResponsiveApp /> }).await?;
    Ok(())
}
