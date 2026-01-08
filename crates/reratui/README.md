# reratui-fiber

React-like fiber architecture for Reratui with proper effect timing, state batching, and component lifecycle management.

[![Crates.io](https://img.shields.io/crates/v/reratui-fiber.svg)](https://crates.io/crates/reratui-fiber)
[![Documentation](https://docs.rs/reratui-fiber/badge.svg)](https://docs.rs/reratui-fiber)

## Overview

`reratui-fiber` implements React's Fiber architecture for Reratui, providing:

- **Fiber-based component instances** - Each component has isolated hook state (no global index collisions)
- **Post-commit effect execution** - Effects run after the screen updates, not during render
- **State update batching** - Multiple state updates in event handlers trigger a single re-render
- **Proper context lifecycle** - Context providers are automatically cleaned up on unmount
- **Strict mode** - Double-renders in development to catch impure components

## Why reratui-fiber?

The original Reratui hooks had several issues that didn't match React's semantics:

| Issue           | Old Behavior                       | New Behavior (v2)              |
| --------------- | ---------------------------------- | ------------------------------ |
| Effect timing   | Effects run during render          | Effects run after commit       |
| Hook identity   | Global index (corrupts on reorder) | Fiber-scoped (stable identity) |
| State batching  | Each `set_state` re-renders        | Batched within event handlers  |
| Context cleanup | Never cleaned up (memory leak)     | Cleaned up on unmount          |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
reratui-fiber = "0.2.1"
```

## Quick Start

```rust
use reratui_fiber::prelude::*;

#[component]
fn Counter() -> Element {
    // Fiber-scoped state - stable identity across renders
    let (count, set_count) = use_state_v2(|| 0);

    // Effect runs AFTER commit (screen already updated)
    use_effect_v2(|| {
        println!("Count is now: {}", count);
        None // No cleanup needed
    }, (count,));

    // Multiple updates are BATCHED - only ONE re-render
    let increment_by_3 = {
        let set_count = set_count.clone();
        move |_| {
            set_count.update(|n| n + 1); // Queued
            set_count.update(|n| n + 1); // Queued
            set_count.update(|n| n + 1); // Queued
            // All 3 updates batched into single re-render!
        }
    };

    rsx! {
        <Block title="Counter">
            <Paragraph>{format!("Count: {}", count)}</Paragraph>
        </Block>
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render_v2(|| rsx! { <Counter /> }).await
}
```

## Hooks API

### use_state_v2

State management with batching support:

```rust
let (value, setter) = use_state_v2(|| initial_value);

// Set directly
setter.set(new_value);

// Update based on previous value (receives latest state)
setter.update(|prev| prev + 1);
```

### use_effect_v2

Side effects that run after commit:

```rust
// Run when dependencies change
use_effect_v2(|| {
    println!("Count changed!");
    Some(|| println!("Cleanup")) // Optional cleanup
}, (count,));

// Run once on mount (empty deps)
use_effect_once(|| {
    println!("Mounted!");
    Some(|| println!("Unmounting"))
});

// Run every render (None deps)
use_effect_v2(|| {
    println!("Rendered!");
    None
}, None::<()>);
```

### use_async_effect_v2

Async effects with async cleanup:

```rust
use_async_effect_v2(|| {
    async move {
        let data = fetch_data().await;
        set_data.set(data);

        Some(|| async move {
            println!("Async cleanup");
        })
    }
}, (user_id,));
```

### use_context_v2 / use_context_provider_v2

Context with proper lifecycle:

```rust
// Provide context (automatically cleaned up on unmount)
let theme = use_context_provider_v2(|| Theme::default());

// Consume context (panics if no provider)
let theme = use_context_v2::<Theme>();

// Try to consume (returns None if no provider)
let maybe_theme = try_use_context_v2::<Theme>();
```

### use_memo_v2 / use_callback_v2

Memoization hooks:

```rust
// Memoize expensive computation
let expensive = use_memo_v2(|| compute_expensive(input), (input,));

// Memoize callback
let on_click = use_callback_v2(|_| {
    println!("Clicked!");
}, ());
```

## Render Pipeline

The v2 render loop follows React's 4-phase pipeline:

```
┌─────────────────────────────────────────────────┐
│              Render Pipeline (v2)               │
├─────────────────────────────────────────────────┤
│                                                 │
│  1. EVENT PHASE                                 │
│     ├─ begin_batch() - start batching           │
│     ├─ Process events through handlers          │
│     └─ end_batch() - collect dirty fibers       │
│                                                 │
│  2. RENDER PHASE (Pure, no side effects)        │
│     ├─ Execute component functions              │
│     ├─ Queue effects (don't execute)            │
│     └─ Build VNode tree                         │
│                                                 │
│  3. COMMIT PHASE                                │
│     ├─ Apply changes to terminal buffer         │
│     ├─ Process pending unmounts                 │
│     └─ terminal.draw() - flush to screen        │
│                                                 │
│  4. EFFECT PHASE (After commit)                 │
│     ├─ Run cleanup functions                    │
│     └─ Run new effects                          │
│                                                 │
└─────────────────────────────────────────────────┘
```

## Strict Mode

Enable strict mode to catch impure renders during development:

```rust
use reratui_fiber::prelude::*;

// Enable strict mode (only active in debug builds)
set_strict_mode_enabled(true);

// Or use render options
render_v2_with_options(
    || rsx! { <App /> },
    RenderOptions {
        strict_mode: true,
        ..Default::default()
    }
).await?;
```

Strict mode will:

- Double-render each component
- Run effects, cleanup, and effects again on mount
- Warn if renders produce different results

## Migration from Old API

```rust
// ❌ OLD (deprecated)
use reratui::prelude::*;

let (count, set_count) = use_state(|| 0);      // Global index
use_effect(|| { println!("rendered"); None }, ()); // Runs during render

render(|| rsx! { <Counter /> }).await?;

// ✅ NEW (recommended)
use reratui_fiber::prelude::*;

let (count, set_count) = use_state_v2(|| 0);      // Fiber-scoped
use_effect_v2(|| { println!("rendered"); None }, ()); // Runs after commit

render_v2(|| rsx! { <Counter /> }).await?;
```

## Key Types

| Type               | Description                                 |
| ------------------ | ------------------------------------------- |
| `FiberId`          | Unique identifier for a component instance  |
| `Fiber`            | A mounted component with its own hook state |
| `FiberTree`        | Global tree tracking all mounted components |
| `StateSetterV2<T>` | State setter with batching support          |
| `RenderOptions`    | Configuration for the render loop           |

## Examples

See the examples in the main repository:

- `counter-v2` - Basic counter with v2 APIs
- `effect-timing-v2` - Demonstrates effect execution timing
- `state-batching-v2` - Shows state update batching

```bash
cargo run -p counter-v2
cargo run -p effect-timing-v2
cargo run -p state-batching-v2
```

## License

Dual-licensed under Apache 2.0 or MIT at your option.
