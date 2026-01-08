# Migration Guide: From Old APIs to reratui-fiber

This guide walks you through migrating your Reratui application from the deprecated APIs in `reratui-hooks` and `reratui-runtime` to the new React-like APIs in `reratui-fiber`.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Step-by-Step Migration](#step-by-step-migration)
4. [API Reference](#api-reference)
5. [Common Patterns](#common-patterns)
6. [Troubleshooting](#troubleshooting)
7. [FAQ](#faq)

---

## Overview

### Why Migrate?

The new `reratui-fiber` crate provides significant improvements:

| Feature            | Old API                            | New API (v2)                |
| ------------------ | ---------------------------------- | --------------------------- |
| Effect timing      | During render (blocking)           | After commit (non-blocking) |
| Hook identity      | Global index (corrupts on reorder) | Fiber-scoped (stable)       |
| State batching     | None (each update re-renders)      | Automatic batching          |
| Context cleanup    | Never (memory leak)                | Automatic on unmount        |
| Functional updates | Stale closures                     | Latest state value          |

### Migration Strategy

You can migrate incrementally - old and new APIs can coexist:

1. Update your `Cargo.toml` to include `reratui-fiber`
2. Migrate one component at a time
3. Test each component after migration
4. Eventually remove deprecated API usage

---

## Quick Start

### 1. Add Dependency

```toml
[dependencies]
reratui-fiber = "0.2.1"
```

### 2. Update Imports

```rust
// ❌ OLD
use reratui::prelude::*;

// ✅ NEW
use reratui::prelude::*;
use reratui_fiber::prelude::*;  // Add this for v2 hooks
```

### 3. Update Render Function

```rust
// ❌ OLD
render(|| rsx! { <App /> }).await?;

// ✅ NEW
render_v2(|| rsx! { <App /> }).await?;
```

### 4. Update Hooks

```rust
// ❌ OLD
let (count, set_count) = use_state(|| 0);
use_effect(|| { println!("rendered"); None }, ());

// ✅ NEW
let (count, set_count) = use_state_v2(|| 0);
use_effect_v2(|| { println!("rendered"); None }, ());
```

---

## Step-by-Step Migration

### Step 1: Migrate the Render Function

The render function is the entry point. Start here.

```rust
// Before
use reratui::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| rsx! { <App /> }).await?;
    Ok(())
}

// After
use reratui::prelude::*;
use reratui_fiber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render_v2(|| rsx! { <App /> }).await?;
    Ok(())
}
```

**Note:** `render_v2` uses the 4-phase pipeline (event → render → commit → effect).

### Step 2: Migrate State Hooks

Replace `use_state` with `use_state_v2`:

```rust
// Before
#[component]
fn Counter() -> Element {
    let (count, set_count) = use_state(|| 0);

    let increment = {
        let set_count = set_count.clone();
        move |_| set_count.update(|n| n + 1)
    };

    rsx! { <Text text={count.get().to_string()} /> }
}

// After
#[component]
fn Counter() -> Element {
    let (count, set_count) = use_state_v2(|| 0);

    let increment = {
        let set_count = set_count.clone();
        move |_| set_count.update(|n| n + 1)
    };

    // Note: count is now the value directly, not a handle
    rsx! { <Text text={count.to_string()} /> }
}
```

**Key differences:**

- `use_state_v2` returns `(T, StateSetterV2<T>)` instead of `(StateHandle<T>, StateSetter<T>)`
- No need to call `.get()` on the value - it's the value directly
- Multiple updates in the same event handler are batched automatically

### Step 3: Migrate Effect Hooks

Replace `use_effect` with `use_effect_v2`:

```rust
// Before
#[component]
fn Logger() -> Element {
    let (count, _) = use_state(|| 0);

    // ❌ This runs DURING render (blocks screen update)
    use_effect(|| {
        println!("Count changed to: {}", count.get());
        None
    }, (count.get(),));

    rsx! { <Text text="Logger" /> }
}

// After
#[component]
fn Logger() -> Element {
    let (count, _) = use_state_v2(|| 0);

    // ✅ This runs AFTER commit (screen already updated)
    use_effect_v2(|| {
        println!("Count changed to: {}", count);
        None
    }, (count,));

    rsx! { <Text text="Logger" /> }
}
```

**Key differences:**

- Effects run after the screen is updated, not during render
- Cleanup functions run before new effects, not during render
- Use `use_effect_once` for mount-only effects (empty deps)

### Step 4: Migrate Context Hooks

Replace context hooks with v2 versions:

```rust
// Before
#[derive(Clone)]
struct Theme { primary: Color }

#[component]
fn ThemeProvider() -> Element {
    // ❌ Context never cleaned up (memory leak)
    let theme = use_context_provider(|| Theme { primary: Color::Cyan });
    rsx! { <Child /> }
}

#[component]
fn Child() -> Element {
    let theme = use_context::<Theme>();
    rsx! { <Block style={Style::default().fg(theme.primary)} /> }
}

// After
#[component]
fn ThemeProvider() -> Element {
    // ✅ Context automatically cleaned up on unmount
    let theme = use_context_provider_v2(|| Theme { primary: Color::Cyan });
    rsx! { <Child /> }
}

#[component]
fn Child() -> Element {
    let theme = use_context_v2::<Theme>();
    rsx! { <Block style={Style::default().fg(theme.primary)} /> }
}
```

**Key differences:**

- Context is automatically cleaned up when provider unmounts
- Nested providers properly shadow parent values
- Use `try_use_context_v2` for optional context (returns `Option<T>`)

### Step 5: Migrate Exit Requests

Replace `request_exit` with `request_exit_v2`:

```rust
// Before
if let Some(Event::Key(key)) = use_event() && key.code == KeyCode::Char('q') {
    request_exit();
}

// After
if let Some(Event::Key(key)) = use_event() && key.code == KeyCode::Char('q') {
    request_exit_v2();
}
```

---

## API Reference

### State Management

| Old API             | New API              | Notes                              |
| ------------------- | -------------------- | ---------------------------------- |
| `use_state(init)`   | `use_state_v2(init)` | Returns value directly, not handle |
| `state.get()`       | `value`              | No getter needed                   |
| `setter.set(val)`   | `setter.set(val)`    | Same API                           |
| `setter.update(fn)` | `setter.update(fn)`  | Now receives latest state          |

### Effects

| Old API                      | New API                         | Notes                      |
| ---------------------------- | ------------------------------- | -------------------------- |
| `use_effect(fn, deps)`       | `use_effect_v2(fn, deps)`       | Runs after commit          |
| `use_effect(fn, ())`         | `use_effect_once(fn)`           | Convenience for mount-only |
| `use_async_effect(fn, deps)` | `use_async_effect_v2(fn, deps)` | Async cleanup support      |

### Context

| Old API                    | New API                       | Notes               |
| -------------------------- | ----------------------------- | ------------------- |
| `use_context_provider(fn)` | `use_context_provider_v2(fn)` | Auto cleanup        |
| `use_context::<T>()`       | `use_context_v2::<T>()`       | Proper scoping      |
| N/A                        | `try_use_context_v2::<T>()`   | Returns `Option<T>` |

### Memoization

| Old API                  | New API                     | Notes        |
| ------------------------ | --------------------------- | ------------ |
| `use_memo(fn, deps)`     | `use_memo_v2(fn, deps)`     | Fiber-scoped |
| `use_callback(fn, deps)` | `use_callback_v2(fn, deps)` | Fiber-scoped |

### Runtime

| Old API             | New API                | Notes                |
| ------------------- | ---------------------- | -------------------- |
| `render(component)` | `render_v2(component)` | 4-phase pipeline     |
| `request_exit()`    | `request_exit_v2()`    | Works with render_v2 |

---

## Common Patterns

### Pattern 1: Batched State Updates

```rust
// Multiple updates in one handler = one re-render
let handle_submit = {
    let set_name = set_name.clone();
    let set_email = set_email.clone();
    let set_submitted = set_submitted.clone();
    move |_| {
        set_name.set(String::new());      // Queued
        set_email.set(String::new());     // Queued
        set_submitted.set(true);          // Queued
        // All three updates batched into ONE re-render
    }
};
```

### Pattern 2: Functional Updates Chain

```rust
// Each update receives the result of the previous
let increment_by_5 = {
    let set_count = set_count.clone();
    move |_| {
        set_count.update(|n| n + 1);  // 0 → 1
        set_count.update(|n| n + 1);  // 1 → 2
        set_count.update(|n| n + 1);  // 2 → 3
        set_count.update(|n| n + 1);  // 3 → 4
        set_count.update(|n| n + 1);  // 4 → 5
        // Result: 5 (correct!)
    }
};
```

### Pattern 3: Effect with Cleanup

```rust
use_effect_v2(|| {
    // Setup: runs after commit
    let subscription = subscribe_to_updates();

    // Cleanup: runs before next effect or on unmount
    Some(move || {
        unsubscribe(subscription);
    })
}, (dependency,));
```

### Pattern 4: Async Data Fetching

```rust
use_async_effect_v2(|| {
    let set_data = set_data.clone();
    let set_loading = set_loading.clone();

    async move {
        set_loading.set(true);

        match fetch_data(user_id).await {
            Ok(data) => set_data.set(Some(data)),
            Err(e) => set_error.set(Some(e.to_string())),
        }

        set_loading.set(false);

        // Optional async cleanup
        Some(|| async move {
            cancel_pending_requests();
        })
    }
}, (user_id,));
```

### Pattern 5: Nested Context Providers

```rust
#[component]
fn App() -> Element {
    let _theme = use_context_provider_v2(|| Theme::light());

    rsx! {
        <Layout>
            <Header />  // Gets light theme
            <DarkSection />  // Gets dark theme (shadowed)
        </Layout>
    }
}

#[component]
fn DarkSection() -> Element {
    // Shadow parent's theme for this subtree
    let _theme = use_context_provider_v2(|| Theme::dark());

    rsx! {
        <Content />  // Gets dark theme
    }
}
```

---

## Troubleshooting

### Issue: "Effect runs at wrong time"

**Symptom:** Effect seems to run before screen updates.

**Cause:** You're still using `use_effect` instead of `use_effect_v2`.

**Solution:**

```rust
// ❌ Wrong
use_effect(|| { /* runs during render */ None }, deps);

// ✅ Correct
use_effect_v2(|| { /* runs after commit */ None }, deps);
```

### Issue: "State updates don't batch"

**Symptom:** Multiple state updates cause multiple re-renders.

**Cause:** You're using `render` instead of `render_v2`.

**Solution:**

```rust
// ❌ Wrong - no batching
render(|| rsx! { <App /> }).await?;

// ✅ Correct - batching enabled
render_v2(|| rsx! { <App /> }).await?;
```

### Issue: "Functional update receives stale value"

**Symptom:** `set_count.update(|n| n + 1)` called twice only increments by 1.

**Cause:** You're using `use_state` instead of `use_state_v2`.

**Solution:**

```rust
// ❌ Wrong - stale closures
let (count, set_count) = use_state(|| 0);

// ✅ Correct - latest state
let (count, set_count) = use_state_v2(|| 0);
```

### Issue: "Context not found after provider unmounts"

**Symptom:** `use_context_v2` panics after parent unmounts.

**Cause:** This is correct behavior! Context is properly scoped now.

**Solution:** Move the provider to a component that stays mounted, or use `try_use_context_v2`:

```rust
// Safe - returns None if no provider
let maybe_theme = try_use_context_v2::<Theme>();
```

### Issue: "Deprecation warnings everywhere"

**Symptom:** Compiler shows deprecation warnings for old APIs.

**Cause:** You're using deprecated APIs.

**Solution:** This is intentional! Follow this guide to migrate. You can temporarily suppress warnings:

```rust
#[allow(deprecated)]
fn legacy_component() { /* ... */ }
```

---

## FAQ

### Q: Can I mix old and new APIs?

**A:** Yes, but with caveats:

- Use `render_v2` for the main loop (required for batching)
- Old hooks work but don't get batching benefits
- Migrate incrementally, one component at a time

### Q: Do I need to migrate all at once?

**A:** No! The migration can be gradual:

1. Start with `render_v2`
2. Migrate leaf components first
3. Work your way up to parent components
4. Old APIs continue to work (with deprecation warnings)

### Q: What about performance?

**A:** The new APIs are generally faster:

- State batching reduces re-renders
- Post-commit effects don't block rendering
- Fiber-scoped hooks have better cache locality

### Q: Will the old APIs be removed?

**A:** Eventually, yes. The deprecation timeline:

- v0.2.x: Deprecated with warnings
- v0.3.x: Deprecated with stronger warnings
- v1.0.0: Old APIs removed

### Q: How do I enable strict mode?

**A:** Strict mode helps catch bugs during development:

```rust
use reratui_fiber::prelude::*;

// Enable globally
set_strict_mode_enabled(true);

// Or per-render
render_v2_with_options(
    || rsx! { <App /> },
    RenderOptions { strict_mode: true, ..Default::default() }
).await?;
```

### Q: What does strict mode do?

**A:** In debug builds, strict mode:

- Double-renders each component (catches impure renders)
- Runs effects twice on mount (catches missing cleanup)
- Warns if renders produce different results

---

## Complete Migration Example

### Before (Old APIs)

```rust
use reratui::prelude::*;

#[derive(Clone)]
struct AppState {
    user: Option<User>,
}

#[component]
fn App() -> Element {
    let _state = use_context_provider(|| AppState { user: None });

    rsx! {
        <Layout direction={Direction::Vertical}>
            <Header />
            <Content />
        </Layout>
    }
}

#[component]
fn Header() -> Element {
    let state = use_context::<AppState>();
    let (count, set_count) = use_state(|| 0);

    use_effect(|| {
        println!("Header rendered with count: {}", count.get());
        None
    }, (count.get(),));

    if let Some(Event::Key(key)) = use_event() && key.code == KeyCode::Char('q') {
        request_exit();
    }

    rsx! {
        <Block title="Header">
            <Paragraph>{format!("Count: {}", count.get())}</Paragraph>
        </Block>
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| rsx! { <App /> }).await?;
    Ok(())
}
```

### After (New APIs)

```rust
use reratui::prelude::*;
use reratui_fiber::prelude::*;

#[derive(Clone)]
struct AppState {
    user: Option<User>,
}

#[component]
fn App() -> Element {
    // ✅ Context automatically cleaned up on unmount
    let _state = use_context_provider_v2(|| AppState { user: None });

    rsx! {
        <Layout direction={Direction::Vertical}>
            <Header />
            <Content />
        </Layout>
    }
}

#[component]
fn Header() -> Element {
    let state = use_context_v2::<AppState>();
    // ✅ Returns value directly, batching enabled
    let (count, set_count) = use_state_v2(|| 0);

    // ✅ Runs after commit, not during render
    use_effect_v2(|| {
        println!("Header rendered with count: {}", count);
        None
    }, (count,));

    if let Some(Event::Key(key)) = use_event() && key.code == KeyCode::Char('q') {
        request_exit_v2();  // ✅ Works with render_v2
    }

    rsx! {
        <Block title="Header">
            <Paragraph>{format!("Count: {}", count)}</Paragraph>
        </Block>
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ✅ 4-phase pipeline with batching
    render_v2(|| rsx! { <App /> }).await?;
    Ok(())
}
```

---

## Need Help?

- Check the [BEHAVIORAL_DIFFERENCES.md](./BEHAVIORAL_DIFFERENCES.md) for detailed behavior changes
- See the [README.md](./README.md) for API documentation
- Look at the examples: `counter_v2`, `effect_timing_v2`, `state_batching_v2`
- File an issue on GitHub if you encounter problems

Happy migrating! 🚀
