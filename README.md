# Reratui

[![Crates.io](https://img.shields.io/crates/v/reratui.svg)](https://crates.io/crates/reratui)
[![Documentation](https://docs.rs/reratui/badge.svg)](https://docs.rs/reratui)
[![License](https://img.shields.io/crates/l/reratui.svg)](https://github.com/sabry-awad97/reratui#license)
[![Build Status](https://img.shields.io/github/workflow/status/sabry-awad97/reratui/CI)](https://github.com/sabry-awad97/reratui/actions)

**A modern, reactive TUI framework for Rust, powered by Ratatui**

Reratui brings React-inspired component architecture and hooks to terminal user interfaces, enabling developers to build complex, interactive TUI applications with clean, maintainable code.

## Features

### Core Capabilities

- **Component-Based Architecture** - Build modular UIs with reusable, composable components following SOLID principles
- **Hooks System** - Manage state and side effects with React-like hooks (`use_state`, `use_effect`, `use_reducer`, etc.)
- **RSX Macro** - Declarative JSX-like syntax for intuitive UI construction with compile-time validation
- **Type-Safe Props** - Automatic prop validation and builder pattern generation with `#[derive(Props)]`
- **Hook Rules Validation** - Compile-time enforcement of the Rules of Hooks for predictable behavior
- **Async-First** - Built on Tokio with first-class async/await support throughout
- **Zero-Cost Abstractions** - Minimal runtime overhead with compile-time macro expansion

### Available Hooks

#### State Management

- `use_state` - Local component state management
- `use_reducer` - Complex state logic with actions (Redux-style)
- `use_ref` - Mutable references that persist across renders

#### Side Effects

- `use_effect` - Side effects with dependency tracking
- `use_effect_once` - Run effect only on mount
- `use_async_effect` - Async side effects with cleanup

#### Performance

- `use_callback` - Memoized callbacks to prevent unnecessary re-renders
- `use_memo` - Memoized computed values
- `use_effect_event` - Stable event handlers that always see latest values

#### Context

- `use_context` - Share data across component tree without prop drilling
- `use_context_provider` - Provide context values to child components

#### Events

- `use_event` - Generic terminal event handling
- `use_keyboard` - Keyboard event handling with stable callbacks
- `use_keyboard_press` - Only handle key press events (not releases)
- `use_keyboard_shortcut` - Handle specific key combinations
- `use_mouse` - Mouse event handling with stable callbacks
- `use_mouse_click` - Handle mouse click events
- `use_mouse_drag` - Track drag operations
- `use_double_click` - Detect double-click gestures
- `on_global_event` - Register global keyboard event handlers

#### Layout & Rendering

- `use_frame` - Access to frame timing and render context
- `use_area` - Component's rendering area information
- `use_on_resize` - Handle terminal resize events
- `use_terminal_dimensions` - Get current terminal size

## Installation

Add Reratui to your `Cargo.toml`:

```toml
[dependencies]
reratui = "0.1.0"
```

For a complete application, you typically only need the main `reratui` crate, which re-exports all necessary functionality.

## Quick Start

### Basic Counter Example

```rust
use reratui::prelude::*;

#[component]
fn Counter() -> Element {
    let (count, set_count) = use_state(|| 0);

    // Handle keyboard events
    if let Some(Event::Key(key)) = use_event()
        && key.is_press()
    {
        match key.code {
            KeyCode::Char('j') => set_count.update(|n| n + 1),
            KeyCode::Char('k') => set_count.update(|n| n - 1),
            KeyCode::Char('r') => set_count.set(0),
            _ => {}
        }
    }

    rsx! {
        <Block
            title="Counter Demo"
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Cyan)}
        >
            <Paragraph alignment={Alignment::Center}>
                {format!("Count: {}", count.get())}
            </Paragraph>
            <Paragraph alignment={Alignment::Center}>
                {"Press 'j' to increment, 'k' to decrement, 'r' to reset"}
            </Paragraph>
        </Block>
    }
}

#[reratui::main]
async fn main() -> Result<()> {
    render(|| rsx! { <Counter /> }).await?;
    Ok(())
}
```

### Component with Props

```rust
use reratui::prelude::*;

#[derive(Props)]
struct ButtonProps {
    label: String,
    on_click: Option<Callback<()>>,
}

#[component]
fn Button(props: &ButtonProps) -> Element {
    rsx! {
        <Block borders={Borders::ALL}>
            <Paragraph alignment={Alignment::Center}>
                {format!("[ {} ]", props.label)}
            </Paragraph>
        </Block>
    }
}

#[component]
fn App() -> Element {
    let (clicks, set_clicks) = use_state(|| 0);

    rsx! {
        <Layout direction={Direction::Vertical}>
            <Button
                label={format!("Clicked {} times", clicks.get())}
                on_click={move |_| set_clicks.update(|n| n + 1)}
            />
        </Layout>
    }
}
```

## Advanced Usage

### State Management with `use_reducer`

For complex state logic, use the `use_reducer` hook:

```rust
use reratui::prelude::*;

#[derive(Clone)]
enum TodoAction {
    Add(String),
    Toggle(usize),
    Remove(usize),
}

#[derive(Clone)]
struct TodoState {
    todos: Vec<Todo>,
    next_id: usize,
}

fn todo_reducer(state: TodoState, action: TodoAction) -> TodoState {
    match action {
        TodoAction::Add(text) => TodoState {
            todos: {
                let mut todos = state.todos;
                todos.push(Todo { id: state.next_id, text, completed: false });
                todos
            },
            next_id: state.next_id + 1,
        },
        TodoAction::Toggle(id) => TodoState {
            todos: state.todos.into_iter().map(|mut todo| {
                if todo.id == id {
                    todo.completed = !todo.completed;
                }
                todo
            }).collect(),
            ..state
        },
        TodoAction::Remove(id) => TodoState {
            todos: state.todos.into_iter().filter(|t| t.id != id).collect(),
            ..state
        },
    }
}

#[component]
fn TodoApp() -> Element {
    let (state, dispatch) = use_reducer(
        todo_reducer,
        TodoState { todos: vec![], next_id: 1 }
    );

    // Use state and dispatch in your component...
}
```

### Context for Global State

Share state across components without prop drilling:

```rust
#[component]
fn App() -> Element {
    let theme = use_context_provider(|| Theme::Dark);

    rsx! {
        <Layout>
            <Header />
            <Content />
        </Layout>
    }
}

#[component]
fn Header() -> Element {
    let theme = use_context::<Theme>();
    // Use theme...
}
```

## Architecture

Reratui follows a modular architecture with clear separation of concerns:

```
reratui/
├── reratui/              # Main crate - re-exports all functionality
├── reratui-core/         # Core types (Element, Component, VNode)
├── reratui-macro/        # Procedural macros (component, rsx, Props)
├── reratui-hooks/        # Hook implementations
├── reratui-runtime/      # Event loop and rendering runtime
├── reratui-ratatui/      # Ratatui backend integration
└── examples/             # Example applications
```

### Design Principles

- **SOLID Principles** - Single responsibility, open/closed, Liskov substitution, interface segregation, dependency inversion
- **Domain-Driven Design** - Clear boundaries between domains with well-defined interfaces
- **Composition over Inheritance** - Build complex UIs by composing simple components
- **Type Safety** - Leverage Rust's type system for compile-time correctness
- **Zero-Cost Abstractions** - No runtime overhead for the convenience features

## Examples

The [`examples/`](./examples) directory contains complete applications demonstrating various features:

- **counter** - Basic state management and event handling
- **rsx_demo** - Comprehensive RSX macro features and patterns
- **events_showcase** - Complete event handling demo (keyboard, mouse, resize, global events)
- **router** - Navigation and routing (coming soon)

Run an example with:

```bash
cargo run --example counter
cargo run --example events_showcase
```

## Documentation

- **[API Documentation](https://docs.rs/reratui)** - Complete API reference
- **[Examples](./examples)** - Working code examples
- **[Hooks Guide](./docs/hooks.md)** - Detailed hook usage patterns
- **[Component Patterns](./docs/patterns.md)** - Best practices and patterns

## Minimum Supported Rust Version (MSRV)

Reratui requires Rust 1.75.0 or later due to the use of:

- `let`-`else` statements
- `let` chains in `if` expressions
- Edition 2024 features

## Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) for details on:

- Code of conduct
- Development setup
- Testing requirements
- Pull request process

## Roadmap

- [x] Core component system with lifecycle hooks (`on_mount`, `on_unmount`)
- [x] Comprehensive hooks system (state, effect, reducer, context, etc.)
- [x] RSX macro with conditional rendering
- [x] Hook rules validation
- [x] Event handling (keyboard, mouse, resize)
- [x] Global event system for application-wide shortcuts
- [ ] Router with nested routes
- [ ] Form validation helpers
- [ ] Animation system
- [ ] Dev tools and debugging
- [ ] Performance profiling tools

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

- Built on top of [Ratatui](https://github.com/ratatui-org/ratatui) - A Rust library for cooking up terminal user interfaces
- Inspired by [React](https://react.dev/) - Component architecture and hooks patterns
- Inspired by [Yew](https://yew.rs/) - Rust web framework with similar patterns
