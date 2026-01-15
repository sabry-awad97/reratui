<p align="center">
  <img src="https://raw.githubusercontent.com/sabry-awad97/reratui/main/.github/reratui-light.png" alt="Reratui Logo" width="200"/>
</p>

<h1 align="center">Reratui</h1>

<p align="center">
  <strong>A modern, reactive TUI framework for Rust with React-inspired hooks and components</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/reratui"><img src="https://img.shields.io/crates/v/reratui.svg" alt="Crates.io"></a>
  <a href="https://crates.io/crates/reratui"><img src="https://img.shields.io/crates/d/reratui.svg" alt="Downloads"></a>
  <a href="https://docs.rs/reratui"><img src="https://docs.rs/reratui/badge.svg" alt="Documentation"></a>
  <a href="https://github.com/sabry-awad97/reratui/actions"><img src="https://github.com/sabry-awad97/reratui/workflows/CI/badge.svg" alt="CI Status"></a>
  <a href="#license"><img src="https://img.shields.io/crates/l/reratui.svg" alt="License"></a>
  <a href="https://github.com/sabry-awad97/reratui"><img src="https://img.shields.io/badge/rust-1.85%2B-orange.svg" alt="Rust Version"></a>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#examples">Examples</a> •
  <a href="#documentation">Documentation</a> •
  <a href="#contributing">Contributing</a>
</p>

---

Reratui brings React's powerful component model and hooks system to terminal user interfaces in Rust. Built on top of [ratatui](https://github.com/ratatui-org/ratatui), it provides a familiar, declarative approach to building complex TUI applications with proper state management, effects, and async support.

## Features

🎯 **React-like Component Model**

- Implement the `ComponentV2` trait to create reusable components
- Compose components naturally with Rust's type system
- Props as struct fields with full type safety

🔄 **Fiber Architecture**

- Efficient reconciliation and rendering pipeline
- Batched state updates for optimal performance
- Dirty tracking to minimize re-renders

🪝 **Comprehensive Hooks System**

- `use_state_v2` - Local state with batched updates
- `use_effect_v2` - Side effects with cleanup
- `use_context_v2` - Share data across component tree
- `use_memo_v2` / `use_callback_v2` - Memoization
- `use_reducer_v2` - Complex state management
- `use_ref_v2` - Mutable references without re-renders

⚡ **Async First**

- `use_future_v2` - Track async task state
- `use_query_v2` - Data fetching with caching & retry
- `use_mutation_v2` - Mutation state tracking
- `use_async_effect_v2` - Async side effects

🎮 **Rich Event Handling**

- `use_keyboard_v2` / `use_keyboard_shortcut_v2`
- `use_mouse_v2` / `use_mouse_hover_v2` / `use_mouse_drag_v2`
- `use_resize_v2` / `use_media_query_v2`

⏱️ **Timing & Utilities**

- `use_timeout_v2` / `use_interval_v2`
- `use_history_v2` - Undo/redo support
- `use_form_v2` - Form validation
- `use_id_v2` - Unique IDs

## Quick Start

Add Reratui to your `Cargo.toml`:

```toml
[dependencies]
reratui = "0.2.1"
tokio = { version = "1.49", features = ["full"] }
```

Create your first component:

```rust
use reratui::prelude::*;

struct Counter;

impl ComponentV2 for Counter {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // State persists across renders
        let (count, set_count) = use_state_v2(|| 0);

        // Handle keyboard input
        use_keyboard_press_v2(move |key| {
            match key.code {
                KeyCode::Up => set_count.update(|c| c + 1),
                KeyCode::Down => set_count.update(|c| c.saturating_sub(1)),
                KeyCode::Char('q') => request_exit_v2(),
                _ => {}
            }
        });

        // Render UI using ratatui widgets
        let block = Block::default()
            .title("Counter")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        Paragraph::new(format!("Count: {}", count))
            .block(block)
            .alignment(Alignment::Center)
            .render(area, buffer);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render_v2(|| Counter).await?;
    Ok(())
}
```

## Examples

The repository includes several examples demonstrating various features:

| Example               | Description                 | Run Command                               |
| --------------------- | --------------------------- | ----------------------------------------- |
| `counter_v2`          | Basic state management      | `cargo run --example counter_v2`          |
| `counter`             | Simple counter (legacy)     | `cargo run --example counter`             |
| `effect_timing_v2`    | Effect lifecycle            | `cargo run --example effect_timing_v2`    |
| `async_fetch_example` | Async data fetching         | `cargo run --example async_fetch_example` |
| `query_example`       | Data queries with caching   | `cargo run --example query_example`       |
| `mutation_example`    | CRUD operations             | `cargo run --example mutation_example`    |
| `events_showcase`     | Keyboard & mouse events     | `cargo run --example events_showcase`     |
| `command_palette`     | Complex multi-component app | `cargo run --example command_palette`     |
| `data_fetcher`        | Multiple async sources      | `cargo run --example data_fetcher`        |
| `ink_like_counter`    | Ink-style component syntax  | `cargo run --example ink_like_counter`    |

### Data Fetching Example

```rust
use reratui::prelude::*;
use reratui::hooks::{use_query_v2, QueryOptions, QueryStatus};

struct UserList;

impl ComponentV2 for UserList {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let query = use_query_v2(
            "users",
            || async { fetch_users().await },
            Some(QueryOptions {
                stale_time: Duration::from_secs(30),
                retry: true,
                retry_attempts: 3,
                ..Default::default()
            }),
        );

        match query.status {
            QueryStatus::Loading => {
                Paragraph::new("Loading...").render(area, buffer);
            }
            QueryStatus::Success => {
                if let Some(users) = &query.data {
                    render_user_list(users, area, buffer);
                }
            }
            QueryStatus::Error => {
                if let Some(err) = &query.error {
                    Paragraph::new(format!("Error: {}", err))
                        .style(Style::default().fg(Color::Red))
                        .render(area, buffer);
                }
            }
            _ => {}
        }
    }
}
```

### Keyboard Shortcuts

```rust
// Handle specific key combinations
use_keyboard_shortcut_v2(
    KeyCode::Char('s'),
    KeyModifiers::CONTROL,
    || save_document(),
);

// Handle all key presses
use_keyboard_press_v2(move |key| {
    match key.code {
        KeyCode::Up => navigate_up(),
        KeyCode::Down => navigate_down(),
        KeyCode::Enter => select_item(),
        _ => {}
    }
});
```

### Mouse Interaction

```rust
// Track hover state
let button_area = Rect::new(10, 5, 20, 3);
let is_hovering = use_mouse_hover_v2(button_area);

// Handle clicks
use_mouse_click_v2(move |button, x, y| {
    if button == MouseButton::Left && button_area.contains((x, y).into()) {
        handle_button_click();
    }
});

// Track drag operations
let (drag_info, reset_drag) = use_mouse_drag_v2();
if drag_info.is_dragging {
    // Handle drag...
}
```

## Architecture

Reratui uses a 5-phase render pipeline:

```
┌─────────────────────────────────────────────────────────┐
│                     Render Loop                          │
│                                                          │
│   ┌──────┐    ┌────────┐    ┌────────┐    ┌──────────┐ │
│   │ Poll │ → │ Render │ → │ Commit │ → │  Event   │  │
│   └──────┘    └────────┘    └────────┘    └──────────┘ │
│       ↑                                        │        │
│       │         ┌────────┐                     │        │
│       └─────────│ Effect │←────────────────────┘        │
│                 └────────┘                              │
└─────────────────────────────────────────────────────────┘
```

1. **Poll** - Wait for terminal events or scheduled updates
2. **Render** - Execute component render functions
3. **Commit** - Apply batched state updates
4. **Event** - Process terminal events
5. **Effect** - Run effects and cleanup functions

## Documentation

| Document                                                      | Description                               |
| ------------------------------------------------------------- | ----------------------------------------- |
| [Hooks Reference](crates/reratui/docs/HOOKS_REFERENCE.md)     | Complete API reference for all hooks      |
| [Component Guide](crates/reratui/docs/COMPONENT_GUIDE.md)     | How to create and compose components      |
| [Async Patterns](crates/reratui/docs/ASYNC_PATTERNS.md)       | Data fetching, queries, and mutations     |
| [Architecture](crates/reratui/docs/ARCHITECTURE.md)           | Internal fiber system and render pipeline |
| [Examples Guide](crates/reratui/docs/EXAMPLES.md)             | Walkthrough of all examples               |
| [Migration Guide](crates/reratui/MIGRATION_GUIDE.md)          | Upgrading from older versions             |
| [React Differences](crates/reratui/BEHAVIORAL_DIFFERENCES.md) | React vs Reratui comparison               |
| [Strict Mode](crates/reratui/STRICT_MODE.md)                  | Development mode for catching issues      |

## Hooks at a Glance

### State Management

```rust
let (value, set_value) = use_state_v2(|| initial);
let (state, dispatch) = use_reducer_v2(reducer, initial);
let ref_handle = use_ref_v2(|| initial);
let history = use_history_v2(|| initial);
```

### Effects

```rust
use_effect_v2(|| { /* effect */ Some(Box::new(|| { /* cleanup */ })) }, deps);
use_effect_once(|| { /* runs once on mount */ None });
use_async_effect_v2(|| async { /* async effect */ None }, deps);
```

### Context

```rust
use_context_provider_v2(|| value);
let value = use_context_v2::<T>();
let maybe_value = try_use_context_v2::<T>();
```

### Async Data

```rust
let future = use_future_v2(|| async { Ok(data) }, Some(deps));
let query = use_query_v2("key", || async { fetch() }, options);
let mutation = use_mutation_v2(|args| async { mutate(args) }, options);
```

### Events

```rust
if let Some(event) = use_event() { /* handle */ }
use_keyboard_press_v2(|key| { /* handle */ });
use_keyboard_shortcut_v2(KeyCode::Char('s'), KeyModifiers::CONTROL, || {});
use_mouse_click_v2(|button, x, y| { /* handle */ });
let is_hovering = use_mouse_hover_v2(area);
```

### Timing

```rust
let timeout = use_timeout_v2(|| { /* callback */ }, delay_ms);
let interval = use_interval_v2(|| { /* callback */ }, interval_ms);
```

### Layout

```rust
let area = use_area_v2();
let frame = use_frame_v2();
let (width, height) = use_resize_v2();
let is_narrow = use_media_query_v2(|(w, _)| w < 80);
```

## Requirements

- **Rust 1.85.0** or later (edition 2024)
- **Tokio** runtime for async support

## Project Structure

```
reratui/
├── crates/
│   ├── reratui/           # Main library
│   │   ├── src/
│   │   │   ├── hooks/     # All hook implementations
│   │   │   ├── scheduler/ # State batching & effects
│   │   │   ├── fiber.rs   # Fiber node
│   │   │   ├── fiber_tree.rs
│   │   │   ├── runtime.rs # Main render loop
│   │   │   └── ...
│   │   └── docs/          # Detailed documentation
│   └── reratui-macro/     # Procedural macros
├── examples/              # Example applications
└── ...
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development

```bash
# Run tests
cargo test

# Run a specific example
cargo run --example counter_v2

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) - The excellent TUI library this is built upon
- [React](https://react.dev/) - Inspiration for the component model and hooks system
- [Ink](https://github.com/vadimdemedes/ink) - Inspiration for bringing React patterns to terminals

---

<p align="center">
  Made with ❤️ for the Rust TUI community
</p>
