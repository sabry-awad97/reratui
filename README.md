# 🎨 Reratui

**A reactive TUI framework for Rust, powered by Ratatui**

Reratui brings React-like component architecture and hooks to terminal user interfaces, making it easy to build complex, interactive TUI applications with clean, maintainable code.

## ✨ Features

- 🎯 **Component-Based Architecture** - Build UIs with reusable, composable components
- 🪝 **Hooks System** - Manage state and side effects with familiar React-like hooks
- 🔄 **Reactive Updates** - Efficient diffing and rendering with virtual DOM
- 🎨 **RSX Syntax** - JSX-like syntax for intuitive UI declaration
- 🧭 **Built-in Router** - Navigate between views with ease
- 📝 **Form Handling** - Simplified form state management and validation
- 📊 **Chart Components** - Ready-to-use data visualization components
- 🎭 **Icon Library** - Comprehensive icon set for TUIs
- ⚡ **Async Support** - First-class async/await support with Tokio

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
reratui = "0.1"           # Core framework
reratui-hooks = "0.1"     # Additional hooks
reratui-router = "0.1"    # Navigation/routing
reratui-forms = "0.1"     # Form handling
reratui-icons = "0.1"     # Icon components
reratui-charts = "0.1"    # Chart components
```

## 🚀 Quick Start

```rust
#[component(Counter)]
fn counter() -> Element {
    let (count, set_count) = use_state(|| 0);

    rsx! {
        <Block title="Counter" borders={Borders::ALL}>
            <Paragraph>
                "Count: {count}"
            </Paragraph>
            <Button on_click={move |_| set_count(count + 1)}>
                "Increment"
            </Button>
        </Block>
    }
}

#[reratui::main]
async fn main() -> Result<()> {
    render(|| {
        rsx! { <Counter /> }
    })
    .await;
    Ok(())
}
```

## 📚 Documentation

- [Getting Started Guide](./docs/getting-started.md)
- [Component API](./docs/components.md)
- [Hooks Reference](./docs/hooks.md)
- [Router Guide](./docs/router.md)
- [Examples](./examples)

## 🏗️ Architecture

```
reratui/
├── reratui/              # Main crate (re-exports)
├── reratui-core/         # Core runtime & VNode
├── reratui-macro/        # RSX and component macros
├── reratui-hooks/        # Hook implementations
├── reratui-ratatui/      # Ratatui backend adapter
├── reratui-router/       # Routing system
├── reratui-forms/        # Form handling
├── reratui-icons/        # Icon components
├── reratui-charts/       # Chart components
└── examples/             # Example applications
```

## 🎯 Design Principles

- **SOLID Principles** - Clean, maintainable architecture
- **Domain-Driven Design** - Clear separation of concerns
- **Dependency Injection** - Loosely coupled components
- **Composition over Inheritance** - Flexible component composition
- **Type Safety** - Leverage Rust's type system for correctness

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) for details.

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
