use reratui::prelude::*;

mod demo_tabs;
mod demo_utils;

use demo_tabs::*;
use demo_utils::*;

/// Props for the Button component
#[derive(Props)]
struct TitleProps {
    /// Button label text
    content: String,
}

/// A reusable Button component with area awareness
#[component]
fn Title(props: &TitleProps) -> Element {
    rsx! {
        <Block
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::White)}
        >
            <Paragraph
                alignment={Alignment::Center}
                style={Style::default().fg(Color::Cyan)}
            >
                {props.content.clone()}
            </Paragraph>
        </Block>

    }
}

/// A comprehensive component that demonstrates all RSX macro features
struct RsxDemoImpl {
    title: String,
}

impl RsxDemoImpl {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

impl Component for RsxDemoImpl {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Create state for interactive demo (using hooks for now)
        let (counter, set_counter) = use_state(|| 0);
        let (selected_tab, set_selected_tab) = use_state(|| 0);
        let (show_debug, set_show_debug) = use_state(|| true);
        let (theme_mode, set_theme_mode) = use_state(|| 0);

        // Process keyboard events
        if let Some(Event::Key(key)) = use_event()
            && key.is_press()
        {
            match key.code {
                KeyCode::Char('j') => set_counter.update(|prev| prev + 1),
                KeyCode::Char('k') => {
                    if counter.get() > 0 {
                        set_counter.update(|prev| prev - 1);
                    }
                }
                KeyCode::Tab => {
                    set_selected_tab.set((selected_tab.get() + 1) % get_tab_titles().len())
                }
                KeyCode::Char('d') => set_show_debug.set(!show_debug.get()),
                KeyCode::Char('t') => set_theme_mode.set((theme_mode.get() + 1) % 3),
                KeyCode::Char('r') => {
                    set_counter.set(0);
                    set_show_debug.set(true);
                    set_theme_mode.set(0);
                }
                _ => {}
            }
        }

        // Create demo state for passing to tab renderers
        let demo_state = DemoState {
            counter: counter.get(),
            show_debug: show_debug.get(),
            theme_mode: theme_mode.get(),
        };

        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Tabs
                Constraint::Min(5),    // Content
                Constraint::Length(3), // Help
            ])
            .split(area);

        // Render the title using the modular method
        let title_vnode = rsx!(
            <Title
                content={self.title.clone()}
            />
        );
        title_vnode.render(chunks[0], buffer);

        // Render tabs using rsx!
        let tabs_vnode = rsx!(
            <Tabs
                titles={get_tab_titles()}
                select={selected_tab.get()}
                style={Style::default().fg(Color::White)}
                highlight_style={Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)}
                block={Block::default().borders(Borders::ALL)}
            />
        );
        tabs_vnode.render(chunks[1], buffer);

        // Render content based on selected tab using modular components
        let content_vnode = match selected_tab.get() {
            0 => render_overview_tab(&demo_state),
            1 => render_match_tab(&demo_state),
            2 => render_logical_and_tab(&demo_state),
            3 => render_if_else_tab(&demo_state),
            4 => render_mixed_conditionals_tab(&demo_state),
            5 => render_nested_layouts_tab(&demo_state),
            6 => render_help_tab(&demo_state),
            _ => render_overview_tab(&demo_state), // Fallback
        };
        content_vnode.render(chunks[2], buffer);

        // Render help footer
        let help_vnode = rsx!(
            <Block
                title="💡 Quick Help"
                borders={Borders::ALL}
                border_style={Style::default().fg(Color::Gray)}
            >
                <Paragraph alignment={Alignment::Center}>
                    {get_control_instructions()}
                </Paragraph>
            </Block>
        );
        help_vnode.render(chunks[3], buffer);
    }
}

#[component]
fn RsxDemo() -> Element {
    RsxDemoImpl::new("🚀 RSX Macro Demo - Modular & Reusable").into()
}

/// Entry point for the application
#[reratui::main]
async fn main() -> Result<()> {
    render(|| rsx! { <RsxDemo /> }).await?;
    Ok(())
}
