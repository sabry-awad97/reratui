//! Layout Demo
//!
//! This example demonstrates the Layout component with nested components
//! using direct parameters, showcasing the RSX macro's support for the
//! Layout syntax.

use reratui::prelude::*;

/// A simple component that displays a message with a color
#[component]
fn ColoredMessage(message: String, color: Color) -> Element {
    rsx! {
        <Block
            title="Colored Message"
            borders={Borders::ALL}
            border_style={Style::default().fg(color)}
        >
            <Paragraph
                alignment={Alignment::Center}
                style={Style::default().fg(color).add_modifier(Modifier::BOLD)}
            >
                {message}
            </Paragraph>
        </Block>
    }
}

/// A component that shows a counter with a specific value
#[component]
fn Counter(value: i32, label: String) -> Element {
    rsx! {
        <Block
            title={label}
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Cyan)}
        >
            <Paragraph
                alignment={Alignment::Center}
                style={Style::default().fg(Color::White).add_modifier(Modifier::BOLD)}
            >
                {format!("Count: {}", value)}
            </Paragraph>
        </Block>
    }
}

/// A component that displays user information
#[component]
fn UserInfo(name: String, age: u32, role: String) -> Element {
    rsx! {
        <Block
            title="User Information"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Green)}
        >
            <Layout
                direction={Direction::Vertical}
                constraints={vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]}
            >
                <Paragraph>
                    {format!("Name: {}", name)}
                </Paragraph>
                <Paragraph>
                    {format!("Age: {}", age)}
                </Paragraph>
                <Paragraph>
                    {format!("Role: {}", role)}
                </Paragraph>
            </Layout>
        </Block>
    }
}

/// Main layout demo component
struct LayoutDemo {
    title: String,
}

impl LayoutDemo {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

impl Component for LayoutDemo {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // This demonstrates the exact Layout syntax you wanted to check
        let layout = rsx! {
            <Layout
                direction={Direction::Vertical}
                margin={1}
                constraints={vec![
                    Constraint::Length(3),  // Header
                    Constraint::Min(0),     // Content
                    Constraint::Length(3),  // Footer
                ]}
            >
                {/* Header */}
                <Block
                    title={self.title.clone()}
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)}
                >
                    <Paragraph
                        alignment={Alignment::Center}
                        style={Style::default().fg(Color::White).add_modifier(Modifier::BOLD)}
                    >
                        {"🚀 Layout Demo with Direct Parameter Components"}
                    </Paragraph>
                </Block>

                {/* Main content - This is the Layout syntax you wanted to test */}
                <Layout
                    direction={Direction::Horizontal}
                    constraints={vec![
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                        Constraint::Percentage(33),
                    ]}
                >
                    <ColoredMessage
                        message={"Success!".to_string()}
                        color={Color::Green}
                    />
                    <Counter
                        value={42}
                        label={"Demo Counter".to_string()}
                    />
                    <UserInfo
                        name={"Alice".to_string()}
                        age={30}
                        role={"Developer".to_string()}
                    />
                </Layout>

                {/* Footer */}
                <Block
                    title="📝 Features Demonstrated"
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Green)}
                >
                    <Paragraph
                        alignment={Alignment::Center}
                        style={Style::default().fg(Color::White)}
                    >
                        {"✅ Layout Components | ✅ Direct Parameters | ✅ RSX Nesting"}
                    </Paragraph>
                </Block>
            </Layout>
        };

        // Render the layout
        layout.render(area, buffer);
    }
}

/// Entry point for the layout demo
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Layout Demo...");
    println!("This demonstrates:");
    println!("  • Layout components with percentage constraints");
    println!("  • Direct parameter components");
    println!("  • RSX macro nesting support");
    println!("  • The exact Layout syntax you requested");
    println!();

    // Render the application
    if let Err(err) = render(|| LayoutDemo::new("✨ Layout Demo ✨").into()).await {
        eprintln!("❌ Application error: {:?}", err);
    } else {
        println!("✨ Layout demo completed successfully!");
        println!("🎯 Demonstrated: Layout components with direct parameter components");
    }

    Ok(())
}
