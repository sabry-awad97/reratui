use crate::demo_utils::*;
use reratui::prelude::*;

/// A Container component that can hold and render children
pub fn render_overview_tab(state: &DemoState) -> Element {
    let current_mode = state.get_current_mode();
    let status_description = state.get_status_description();

    // For now, return a simple block - children rendering would need more complex implementation
    rsx! {
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(6),
                Constraint::Min(0),
                Constraint::Length(4),
            ]}
            margin={1}
        >
            <Block
                title="🎉 RSX Macro Comprehensive Demo"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)}
            >
                <Layout
                    direction={Direction::Horizontal}
                    constraints={vec![Constraint::Percentage(50), Constraint::Percentage(50)]}
                >
                    <Paragraph
                        alignment={Alignment::Left}
                        style={Style::default().fg(Color::Yellow)}
                    >
                        {format!("✨ Current State:\n📊 Counter: {}\n🎯 Mode: {}\n🔧 Debug: {}\n🎨 Theme: {}",
                            state.counter,
                            current_mode,
                            if state.show_debug { "ON" } else { "OFF" },
                            state.get_theme_name()
                        )}
                    </Paragraph>
                    <Paragraph
                        alignment={Alignment::Left}
                        style={Style::default().fg(Color::Green)}
                    >
                        {get_features_list()}
                    </Paragraph>
                </Layout>
            </Block>

            {/* Interactive status display section */}
            <Block
                title="🎯 Interactive Status Display"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Magenta)}
            >
                <Layout
                    direction={Direction::Vertical}
                    constraints={vec![Constraint::Length(3), Constraint::Min(0)]}
                >
                    {/* Main status message */}
                    <Paragraph
                        alignment={Alignment::Center}
                        style={Style::default().fg(Color::White).add_modifier(Modifier::BOLD)}
                    >
                        {status_description}
                    </Paragraph>

                    {/* Debug panel - only shown when debug mode is enabled */}
                    {state.show_debug && (
                        <Block
                            title="🐛 Debug Information"
                            borders={Borders::ALL}
                            border_style={Style::default().fg(Color::Blue)}
                        >
                            <Layout
                                direction={Direction::Horizontal}
                                constraints={vec![Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)]}
                            >
                                {/* Mathematical properties display */}
                                <Paragraph alignment={Alignment::Center}>
                                    {format!("Even: {}", state.counter % 2 == 0)}
                                </Paragraph>
                                <Paragraph alignment={Alignment::Center}>
                                    {format!("Div by 3: {}", state.counter % 3 == 0)}
                                </Paragraph>
                                <Paragraph alignment={Alignment::Center}>
                                    {format!("Prime: {}", is_prime(state.counter))}
                                </Paragraph>
                            </Layout>
                        </Block>
                    )}
                </Layout>
            </Block>

            <Block
                title="🎮 Controls"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Gray)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {get_control_instructions()}
                </Paragraph>
            </Block>
        </Layout>
    }
}

/// Render the match expressions tab content
pub fn render_match_tab(state: &DemoState) -> Element {
    let current_mode = state.get_current_mode();

    rsx!(
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(5),
                Constraint::Min(0),
                Constraint::Length(3),
            ]}
            margin={1}
        >
            <Block
                title="🎯 Match Expressions - All Syntax Variations"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"Demonstrating match expressions with patterns, guards, and function calls\nBoth rsx! { } and rsx!({ }) syntax supported"}
                </Paragraph>
            </Block>

            <Layout
                direction={Direction::Horizontal}
                constraints={vec![Constraint::Percentage(50), Constraint::Percentage(50)]}
            >
                <Block
                    title="🔢 Simple Pattern Matching"
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Green)}
                >
                    <Layout direction={Direction::Vertical}>
                        {match state.counter {
                            0 => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"🎯 Zero - The beginning!"}
                                </Paragraph>
                            ),
                            1..=5 => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"🌱 Low range (1-5)"}
                                </Paragraph>
                            ),
                            6..=10 => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"⚡ Medium range (6-10)"}
                                </Paragraph>
                            ),
                            11..=20 => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"🚀 High range (11-20)"}
                                </Paragraph>
                            ),
                            _ => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"💥 Extreme range (21+)"}
                                </Paragraph>
                            ),
                        }}

                        <Paragraph alignment={Alignment::Center}>
                            {format!("Current: {}", state.counter)}
                        </Paragraph>
                    </Layout>
                </Block>

                <Block
                    title="🎨 Complex Patterns & Guards"
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Magenta)}
                >
                    <Layout direction={Direction::Vertical}>
                        {match (current_mode, state.counter) {
                            ("startup", n) if n % 2 == 0 => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"🌱 Startup + Even"}
                                </Paragraph>
                            ),
                            ("startup", _) => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"🌱 Startup + Odd"}
                                </Paragraph>
                            ),
                            ("normal", n) if n > 5 => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"⚡ Normal + High"}
                                </Paragraph>
                            ),
                            ("active", n) if is_prime(n) => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"🚀 Active + Prime!"}
                                </Paragraph>
                            ),
                            (mode, n) if n % 3 == 0 => (
                                <Paragraph alignment={Alignment::Center}>
                                    {format!("🎯 {} + Div by 3", mode)}
                                </Paragraph>
                            ),
                            _ => (
                                <Paragraph alignment={Alignment::Center}>
                                    {"🤔 Other combination"}
                                </Paragraph>
                            ),
                        }}

                        <Paragraph alignment={Alignment::Center}>
                            {format!("Mode: {} | Value: {}", current_mode, state.counter)}
                        </Paragraph>
                    </Layout>
                </Block>
            </Layout>

            <Block
                title="💡 Match Expression Features"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Cyan)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"✅ Range patterns (1..=5) | ✅ Guards (if condition) | ✅ Tuple destructuring | ✅ Function calls | ✅ Nested in Layout"}
                </Paragraph>
            </Block>
        </Layout>
    )
}

/// Render the logical AND tab content
pub fn render_logical_and_tab(state: &DemoState) -> Element {
    rsx!(
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(3),
            ]}
            margin={1}
        >
            <Block
                title="⚡ Logical AND (&&) Conditional Rendering"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"Show/hide components based on conditions using && operator\nComponents only render when condition is true"}
                </Paragraph>
            </Block>

            <Layout
                direction={Direction::Vertical}
                constraints={vec![Constraint::Length(4), Constraint::Length(4), Constraint::Length(4), Constraint::Min(0)]}
            >
                {state.counter > 0 && (
                    <Block
                        title="✅ Counter > 0"
                        borders={Borders::ALL}
                        border_style={Style::default().fg(Color::Green)}
                    >
                        <Paragraph alignment={Alignment::Center}>
                            {format!("This appears when counter > 0 (currently {})", state.counter)}
                        </Paragraph>
                    </Block>
                )}

                {state.counter % 2 == 0 && (
                    <Block
                        title="🎯 Even Numbers Only"
                        borders={Borders::ALL}
                        border_style={Style::default().fg(Color::Blue)}
                    >
                        <Paragraph alignment={Alignment::Center}>
                            {format!("Even number detected: {}", state.counter)}
                        </Paragraph>
                    </Block>
                )}

                {state.counter > 5 && state.show_debug && (
                    <Block
                        title="🔥 Complex Condition"
                        borders={Borders::ALL}
                        border_style={Style::default().fg(Color::Red)}
                    >
                        <Paragraph alignment={Alignment::Center}>
                            {"Both counter > 5 AND debug mode enabled!"}
                        </Paragraph>
                    </Block>
                )}

                {is_prime(state.counter) && (
                    <Block
                        title="🌟 Prime Number!"
                        borders={Borders::ALL}
                        border_style={Style::default().fg(Color::Yellow)}
                    >
                        <Paragraph alignment={Alignment::Center}>
                            {format!("Prime number detected: {}!", state.counter)}
                        </Paragraph>
                    </Block>
                )}
            </Layout>

            <Block
                title="💡 Logical AND Features"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Cyan)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"✅ Simple conditions | ✅ Complex expressions | ✅ Function calls | ✅ Multiple conditions | ✅ Nested in Layout"}
                </Paragraph>
            </Block>
        </Layout>
    )
}

/// Render the if-else tab content
pub fn render_if_else_tab(state: &DemoState) -> Element {
    rsx!(
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(3),
            ]}
            margin={1}
        >
            <Block
                title="🔀 If-Else Conditional Rendering"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"Choose between different components based on conditions\nSupports if, else if, and else branches"}
                </Paragraph>
            </Block>

            <Layout
                direction={Direction::Horizontal}
                constraints={vec![Constraint::Percentage(50), Constraint::Percentage(50)]}
            >
                <Block
                    title="🎯 Simple If-Else"
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Green)}
                >
                    <Layout
                        direction={Direction::Vertical}
                        constraints={vec![Constraint::Length(2), Constraint::Length(2), Constraint::Min(0)]}
                    >
                        {if state.counter % 2 == 0 {
                            <Paragraph alignment={Alignment::Center}>
                                {"✅ Even number branch"}
                            </Paragraph>
                        } else {
                            <Paragraph alignment={Alignment::Center}>
                                {"🔄 Odd number branch"}
                            </Paragraph>
                        }}

                        {if state.show_debug {
                            <Paragraph alignment={Alignment::Center}>
                                {"🐛 Debug mode active"}
                            </Paragraph>
                        } else {
                            <Paragraph alignment={Alignment::Center}>
                                {"🔒 Debug mode off"}
                            </Paragraph>
                        }}

                        <Paragraph alignment={Alignment::Center}>
                            {format!("Counter: {} | Debug: {}", state.counter, state.show_debug)}
                        </Paragraph>
                    </Layout>
                </Block>

                <Block
                    title="🌈 Complex If-Else Chain"
                    borders={Borders::ALL}
                    border_style={Style::default().fg(Color::Magenta)}
                >
                    <Layout
                        direction={Direction::Vertical}
                        constraints={vec![Constraint::Length(2), Constraint::Length(2), Constraint::Min(0)]}
                    >
                        {if state.counter == 0 {
                            <Paragraph alignment={Alignment::Center}>
                                {"🎯 Exactly zero!"}
                            </Paragraph>
                        } else if state.counter < 5 {
                            <Paragraph alignment={Alignment::Center}>
                                {"🌱 Small number (1-4)"}
                            </Paragraph>
                        } else if state.counter < 10 {
                            <Paragraph alignment={Alignment::Center}>
                                {"⚡ Medium number (5-9)"}
                            </Paragraph>
                        } else if state.counter < 20 {
                            <Paragraph alignment={Alignment::Center}>
                                {"🚀 Large number (10-19)"}
                            </Paragraph>
                        } else {
                            <Paragraph alignment={Alignment::Center}>
                                {"💥 Huge number (20+)"}
                            </Paragraph>
                        }}

                        {if is_prime(state.counter) && state.counter > 1 {
                            <Paragraph alignment={Alignment::Center}>
                                {"🌟 Prime number!"}
                            </Paragraph>
                        } else if state.counter > 1 {
                            <Paragraph alignment={Alignment::Center}>
                                {"🔢 Composite number"}
                            </Paragraph>
                        } else {
                            <Paragraph alignment={Alignment::Center}>
                                {"🎯 Special case (0 or 1)"}
                            </Paragraph>
                        }}

                        <Paragraph alignment={Alignment::Center}>
                            {format!("Range check: {} | Prime: {}",
                                if state.counter == 0 { "Zero" }
                                else if state.counter < 5 { "Small" }
                                else if state.counter < 10 { "Medium" }
                                else if state.counter < 20 { "Large" }
                                else { "Huge" },
                                is_prime(state.counter)
                            )}
                        </Paragraph>
                    </Layout>
                </Block>
            </Layout>

            <Block
                title="💡 If-Else Features"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Cyan)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"✅ If-else chains | ✅ Complex conditions | ✅ Function calls | ✅ Nested if statements | ✅ Both rsx! syntaxes"}
                </Paragraph>
            </Block>
        </Layout>
    )
}

/// Render the mixed conditionals tab content
pub fn render_mixed_conditionals_tab(state: &DemoState) -> Element {
    let current_mode = state.get_current_mode();

    rsx!(
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(3),
            ]}
            margin={1}
        >
            <Block
                title="🎨 Mixed Conditionals - All Types Together"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"Combining match expressions, logical AND, and if-else in one layout\nDemonstrates real-world usage patterns"}
                </Paragraph>
            </Block>

            <Layout
                direction={Direction::Vertical}
                constraints={vec![Constraint::Length(6), Constraint::Length(6), Constraint::Min(0)]}
            >
                {match current_mode {
                    "startup" => (
                        <Block title="🌱 Startup Mode" borders={Borders::ALL} border_style={Style::default().fg(Color::Green)}>
                            <Layout direction={Direction::Horizontal}>
                                {state.counter > 0 && (
                                    <Paragraph alignment={Alignment::Center}>
                                        {"✅ Counter active"}
                                    </Paragraph>
                                )}

                                {if state.show_debug {
                                    <Paragraph alignment={Alignment::Center}>
                                        {"🐛 Debug enabled"}
                                    </Paragraph>
                                } else {
                                    <Paragraph alignment={Alignment::Center}>
                                        {"🔒 Debug disabled"}
                                    </Paragraph>
                                }}
                            </Layout>
                        </Block>
                    ),
                    "normal" => (
                        <Block title="⚡ Normal Mode" borders={Borders::ALL} border_style={Style::default().fg(Color::Blue)}>
                            <Layout direction={Direction::Horizontal}>
                                {state.counter % 2 == 0 && <Paragraph alignment={Alignment::Center}>
                                    {"🎯 Even number"}
                                </Paragraph>}

                                {is_prime(state.counter) && <Paragraph alignment={Alignment::Center}>
                                    {"🌟 Prime!"}
                                </Paragraph>}
                            </Layout>
                        </Block>
                    ),
                    "active" => (
                        <Block title="🚀 Active Mode" borders={Borders::ALL} border_style={Style::default().fg(Color::Yellow)}>
                            <Paragraph alignment={Alignment::Center}>
                                {if state.counter > 10 {
                                    "🔥 High performance mode!"
                                } else {
                                    "⚡ Standard active mode"
                                }}
                            </Paragraph>
                        </Block>
                    ),
                    _ => (
                        <Block title="💥 Advanced Mode" borders={Borders::ALL} border_style={Style::default().fg(Color::Magenta)}>
                            <Paragraph alignment={Alignment::Center}>
                                {"🚀 Beyond normal limits!"}
                            </Paragraph>
                        </Block>
                    ),
                }}

                {state.show_debug && (
                    <Block title="🐛 Debug Panel" borders={Borders::ALL} border_style={Style::default().fg(Color::Cyan)}>
                        <Layout direction={Direction::Horizontal}>
                            <Paragraph alignment={Alignment::Center}>
                                {format!("Mode: {}", current_mode)}
                            </Paragraph>
                            <Paragraph alignment={Alignment::Center}>
                                {format!("Counter: {}", state.counter)}
                            </Paragraph>
                            <Paragraph alignment={Alignment::Center}>
                                {format!("Theme: {}", state.get_theme_name())}
                            </Paragraph>
                        </Layout>
                    </Block>
                )}

                {state.counter > 15 && (
                    <Block title="🎉 Achievement Unlocked!" borders={Borders::ALL} border_style={Style::default().fg(Color::Green)}>
                        <Paragraph alignment={Alignment::Center}>
                            {"You've reached the high counter zone! 🏆"}
                        </Paragraph>
                    </Block>
                )}
            </Layout>

            <Block
                title="💡 Mixed Conditionals Features"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Cyan)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"✅ Match + AND + If-Else | ✅ Nested conditions | ✅ Real-world patterns | ✅ Complex state logic"}
                </Paragraph>
            </Block>
        </Layout>
    )
}

/// Render the nested layouts tab content
pub fn render_nested_layouts_tab(state: &DemoState) -> Element {
    rsx!(
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(3),
            ]}
            margin={1}
        >
            <Block
                title="🏗️ Nested Layouts - Complex UI Structures"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"Demonstrating deeply nested layouts with RSX\nShows how to build complex UI hierarchies"}
                </Paragraph>
            </Block>

            <Layout
                direction={Direction::Horizontal}
                constraints={vec![Constraint::Percentage(50), Constraint::Percentage(50)]}
            >
                <Layout
                    direction={Direction::Vertical}
                    constraints={vec![Constraint::Percentage(50), Constraint::Percentage(50)]}
                >
                    <Block title="📊 Metrics" borders={Borders::ALL} border_style={Style::default().fg(Color::Blue)}>
                        <Layout direction={Direction::Horizontal}>
                            <Paragraph alignment={Alignment::Center}>
                                {format!("Count: {}", state.counter)}
                            </Paragraph>
                            <Paragraph alignment={Alignment::Center}>
                                {format!("Squared: {}", state.counter * state.counter)}
                            </Paragraph>
                        </Layout>
                    </Block>

                    <Block title="🎯 Status" borders={Borders::ALL} border_style={Style::default().fg(Color::Yellow)}>
                        <Layout direction={Direction::Vertical}>
                            <Paragraph alignment={Alignment::Center}>
                                {format!("Mode: {}", state.get_current_mode())}
                            </Paragraph>
                            <Paragraph alignment={Alignment::Center}>
                                {format!("Theme: {}", state.get_theme_name())}
                            </Paragraph>
                        </Layout>
                    </Block>
                </Layout>

                <Layout
                    direction={Direction::Vertical}
                    constraints={vec![Constraint::Length(4), Constraint::Length(4), Constraint::Min(0)]}
                >
                    {state.counter % 2 == 0 && <Block title="🎯 Even" borders={Borders::ALL} border_style={Style::default().fg(Color::Green)}>
                        <Paragraph alignment={Alignment::Center}>
                            {"Even number detected"}
                        </Paragraph>
                    </Block>}

                    {is_prime(state.counter) && <Block title="🌟 Prime" borders={Borders::ALL} border_style={Style::default().fg(Color::Magenta)}>
                        <Paragraph alignment={Alignment::Center}>
                            {"Prime number!"}
                        </Paragraph>
                    </Block>}

                    <Block title="🔧 Controls" borders={Borders::ALL} border_style={Style::default().fg(Color::Cyan)}>
                        <Layout direction={Direction::Horizontal}>
                            {state.counter > 0 && <Paragraph alignment={Alignment::Center}>
                                {"↑"}
                            </Paragraph>}
                            {state.counter % 3 == 0 && <Paragraph alignment={Alignment::Center}>
                                {"🎯"}
                            </Paragraph>}
                            {state.show_debug && <Paragraph alignment={Alignment::Center}>
                                {"🐛"}
                            </Paragraph>}
                        </Layout>
                    </Block>
                </Layout>
            </Layout>

            <Block
                title="💡 Nested Layout Features"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Cyan)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"✅ Deep nesting | ✅ Mixed directions | ✅ Conditional layouts | ✅ Complex hierarchies | ✅ Responsive design"}
                </Paragraph>
            </Block>
        </Layout>
    )
}

/// Render the help tab content
pub fn render_help_tab(_state: &DemoState) -> Element {
    rsx!(
        <Layout
            direction={Direction::Vertical}
            constraints={vec![
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(3),
            ]}
            margin={1}
        >
            <Block
                title="📚 Help - RSX Macro Guide"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"Complete guide to using the RSX macro system\nLearn all the features and syntax patterns"}
                </Paragraph>
            </Block>

            <Layout
                direction={Direction::Horizontal}
                constraints={vec![Constraint::Percentage(50), Constraint::Percentage(50)]}
            >
                <Block title="🎮 Controls" borders={Borders::ALL} border_style={Style::default().fg(Color::Green)}>
                    <Layout direction={Direction::Vertical}>
                        <Paragraph alignment={Alignment::Left}>
                            {"j/k - Increment/Decrement counter"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"Tab - Switch between tabs"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"d - Toggle debug mode"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"t - Change theme"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"r - Reset all values"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"q/Esc - Quit application"}
                        </Paragraph>
                    </Layout>
                </Block>

                <Block title="🚀 RSX Features" borders={Borders::ALL} border_style={Style::default().fg(Color::Yellow)}>
                    <Layout direction={Direction::Vertical}>
                        <Paragraph alignment={Alignment::Left}>
                            {"✅ Match expressions with guards"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"✅ Logical AND (&&) conditions"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"✅ If-else-if conditional rendering"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"✅ Nested layout structures"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"✅ Function calls in expressions"}
                        </Paragraph>
                        <Paragraph alignment={Alignment::Left}>
                            {"✅ Dynamic content generation"}
                        </Paragraph>
                    </Layout>
                </Block>
            </Layout>

            <Block
                title="💡 About This Demo"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Magenta)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {"This demo showcases the complete RSX macro system with modular, reusable components. Each tab demonstrates different conditional rendering patterns and layout techniques."}
                </Paragraph>
            </Block>
        </Layout>
    )
}
