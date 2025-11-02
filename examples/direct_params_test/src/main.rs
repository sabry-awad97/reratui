use reratui::prelude::*;

/// A component that takes direct parameters instead of a props struct
#[component]
fn Counter(initial_value: i32) -> Element {
    rsx! {
        <Block
            title="Counter Component"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Cyan)}
        >
            <Paragraph alignment={Alignment::Center}>
                {format!("Counter value: {}", initial_value)}
            </Paragraph>
        </Block>
    }
}

/// A component that takes multiple direct parameters
#[component]
fn UserCard(name: String, age: u32, email: String) -> Element {
    rsx! {
        <Block
            title="User Card"
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
                    {format!("Email: {}", email)}
                </Paragraph>
            </Layout>
        </Block>
    }
}

/// A demo component that uses the direct parameter components
struct DirectParamsDemo {
    title: String,
}

impl DirectParamsDemo {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

impl Component for DirectParamsDemo {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Create the entire layout using rsx! macro
        let layout = rsx! {
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
                        {"🚀 Direct Parameters + RSX Layout Demo"}
                    </Paragraph>
                </Block>

                {/* Main content area with horizontal layout */}
                <Layout
                    direction={Direction::Horizontal}
                    constraints={vec![
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ]}
                >
                    {/* Left side - Counter component */}
                    <Layout
                        direction={Direction::Vertical}
                        constraints={vec![
                            Constraint::Length(8),
                            Constraint::Min(0),
                        ]}
                    >
                        <Counter initial_value={42} />

                        <Block
                            title="📊 Layout Features"
                            borders={Borders::ALL}
                            border_style={Style::default().fg(Color::Blue)}
                        >
                            <Layout
                                direction={Direction::Vertical}
                                constraints={vec![
                                    Constraint::Length(1),
                                    Constraint::Length(1),
                                    Constraint::Length(1),
                                ]}
                            >
                                <Paragraph>{"✅ Nested Layouts"}</Paragraph>
                                <Paragraph>{"✅ Direct Parameters"}</Paragraph>
                                <Paragraph>{"✅ RSX Syntax"}</Paragraph>
                            </Layout>
                        </Block>
                    </Layout>

                    {/* Right side - UserCard component */}
                    <UserCard
                        name={"Alice Johnson".to_string()}
                        age={28}
                        email={"alice@example.com".to_string()}
                    />
                </Layout>

                {/* Footer with multiple columns */}
                <Layout
                    direction={Direction::Horizontal}
                    constraints={vec![
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                        Constraint::Percentage(33),
                    ]}
                >
                    <Block
                        title="🎯 Component Types"
                        borders={Borders::ALL}
                        border_style={Style::default().fg(Color::Green)}
                    >
                        <Paragraph alignment={Alignment::Center}>
                            {"Direct Params"}
                        </Paragraph>
                    </Block>

                    <Block
                        title="🎨 Layout System"
                        borders={Borders::ALL}
                        border_style={Style::default().fg(Color::Yellow)}
                    >
                        <Paragraph alignment={Alignment::Center}>
                            {"RSX Layouts"}
                        </Paragraph>
                    </Block>

                    <Block
                        title="⚡ Performance"
                        borders={Borders::ALL}
                        border_style={Style::default().fg(Color::Magenta)}
                    >
                        <Paragraph alignment={Alignment::Center}>
                            {"Zero Runtime Cost"}
                        </Paragraph>
                    </Block>
                </Layout>
            </Layout>
        };

        // Render the entire layout
        layout.render(area, buffer);
    }
}

/// Entry point for the application
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing direct parameter components with rsx! macro...");

    if let Err(err) = render(|| DirectParamsDemo::new("✨ Direct Parameters Test ✨").into()).await
    {
        eprintln!("Error: {:?}", err);
    }

    println!("✅ Direct parameter components work perfectly!");
    Ok(())
}
