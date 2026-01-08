# Strict Mode Guide

Strict mode is a development tool that helps you find common bugs in your Reratui components. It intentionally double-renders components and double-executes effects to help you catch impure renders and missing cleanup functions.

## Table of Contents

1. [What is Strict Mode?](#what-is-strict-mode)
2. [Enabling Strict Mode](#enabling-strict-mode)
3. [What Strict Mode Does](#what-strict-mode-does)
4. [Common Issues Detected](#common-issues-detected)
5. [Best Practices](#best-practices)
6. [FAQ](#faq)

---

## What is Strict Mode?

Strict mode is a development-only feature (disabled in release builds) that helps you write more robust components by:

1. **Double-rendering components** - Renders each component twice to detect impure renders
2. **Double-executing effects on mount** - Runs effects twice on mount to detect missing cleanup
3. **Warning on render differences** - Logs warnings if two renders produce different results

This mirrors React's StrictMode behavior and helps catch bugs that might otherwise only appear in production.

---

## Enabling Strict Mode

### Method 1: Global Flag

```rust
use reratui_fiber::prelude::*;

fn main() {
    // Enable strict mode globally
    set_strict_mode_enabled(true);

    // Your app code...
}
```

### Method 2: Render Options (Recommended)

```rust
use reratui_fiber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render_v2_with_options(
        || rsx! { <App /> },
        RenderOptions {
            strict_mode: true,
            ..Default::default()
        }
    ).await?;

    Ok(())
}
```

### Method 3: Check Current State

```rust
use reratui_fiber::prelude::*;

fn debug_info() {
    if is_strict_mode_enabled() {
        println!("Strict mode is ON");
    } else {
        println!("Strict mode is OFF");
    }
}
```

---

## What Strict Mode Does

### 1. Double Rendering

When strict mode is enabled, each component renders twice:

```
Component Render Flow (Strict Mode):
┌─────────────────────────────────────┐
│  1. First render (discarded)        │
│     └─ Reset hook index             │
│  2. Second render (kept)            │
│     └─ Compare with first render    │
│  3. If different → Log warning      │
└─────────────────────────────────────┘
```

This helps detect:

- Components that depend on external mutable state
- Components that produce different output on each render
- Side effects during render (which violate React's rules)

### 2. Effect Double-Execution on Mount

When a component mounts, effects run twice:

```
Effect Mount Flow (Strict Mode):
┌─────────────────────────────────────┐
│  1. Run effect (first time)         │
│  2. Run cleanup (if returned)       │
│  3. Run effect (second time)        │
│  4. Keep second cleanup             │
└─────────────────────────────────────┘
```

This helps detect:

- Effects that don't properly clean up
- Effects that assume they only run once
- Resource leaks (subscriptions, timers, etc.)

### 3. Render Difference Warnings

If two renders produce different results, strict mode logs a warning:

```
[WARN] Strict mode: Component rendered different results!
       This indicates an impure render.
       First: Element { ... }, Second: Element { ... }
```

---

## Common Issues Detected

### Issue 1: Impure Renders

**Problem:** Component produces different output on each render.

```rust
// ❌ BAD: Impure render - uses external mutable state
static mut COUNTER: i32 = 0;

#[component]
fn BadComponent() -> Element {
    unsafe {
        COUNTER += 1;  // Side effect during render!
    }
    rsx! { <Text text={format!("Render #{}", unsafe { COUNTER })} /> }
}
```

**Solution:** Use hooks for state management.

```rust
// ✅ GOOD: Pure render - uses hooks for state
#[component]
fn GoodComponent() -> Element {
    let (count, set_count) = use_state_v2(|| 0);

    // State changes happen through setters, not during render
    rsx! { <Text text={format!("Count: {}", count)} /> }
}
```

### Issue 2: Missing Effect Cleanup

**Problem:** Effect doesn't clean up resources.

```rust
// ❌ BAD: No cleanup - subscription leaks
#[component]
fn BadSubscriber() -> Element {
    use_effect_v2(|| {
        let subscription = subscribe_to_updates();
        // Missing cleanup! Subscription leaks on unmount
        None
    }, ());

    rsx! { <Text text="Subscribed" /> }
}
```

**Solution:** Return a cleanup function.

```rust
// ✅ GOOD: Proper cleanup
#[component]
fn GoodSubscriber() -> Element {
    use_effect_v2(|| {
        let subscription = subscribe_to_updates();

        // Cleanup runs on unmount or before next effect
        Some(move || {
            unsubscribe(subscription);
        })
    }, ());

    rsx! { <Text text="Subscribed" /> }
}
```

### Issue 3: Effects That Assume Single Execution

**Problem:** Effect assumes it only runs once.

```rust
// ❌ BAD: Assumes effect runs once
#[component]
fn BadInitializer() -> Element {
    let (data, set_data) = use_state_v2(|| None);

    use_effect_v2(|| {
        // This will run twice in strict mode!
        // If fetch_data has side effects, they'll happen twice
        let result = fetch_data_sync();
        set_data.set(Some(result));
        None
    }, ());

    rsx! { <Text text={format!("{:?}", data)} /> }
}
```

**Solution:** Make effects idempotent or use proper cleanup.

```rust
// ✅ GOOD: Idempotent effect with cleanup
#[component]
fn GoodInitializer() -> Element {
    let (data, set_data) = use_state_v2(|| None);

    use_async_effect_v2(|| {
        let set_data = set_data.clone();
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancelled_clone = cancelled.clone();

        async move {
            let result = fetch_data().await;

            // Check if we were cancelled before updating state
            if !cancelled_clone.load(std::sync::atomic::Ordering::SeqCst) {
                set_data.set(Some(result));
            }

            // Cleanup: mark as cancelled
            Some(move || async move {
                cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
            })
        }
    }, ());

    rsx! { <Text text={format!("{:?}", data)} /> }
}
```

### Issue 4: Side Effects During Render

**Problem:** Performing side effects during the render phase.

```rust
// ❌ BAD: Side effect during render
#[component]
fn BadLogger() -> Element {
    let (count, _) = use_state_v2(|| 0);

    // This runs during render - BAD!
    println!("Rendering with count: {}", count);

    rsx! { <Text text={count.to_string()} /> }
}
```

**Solution:** Use effects for side effects.

```rust
// ✅ GOOD: Side effect in effect hook
#[component]
fn GoodLogger() -> Element {
    let (count, _) = use_state_v2(|| 0);

    // This runs after commit - GOOD!
    use_effect_v2(|| {
        println!("Rendered with count: {}", count);
        None
    }, (count,));

    rsx! { <Text text={count.to_string()} /> }
}
```

---

## Best Practices

### 1. Enable Strict Mode During Development

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable strict mode in development
    #[cfg(debug_assertions)]
    let options = RenderOptions {
        strict_mode: true,
        ..Default::default()
    };

    #[cfg(not(debug_assertions))]
    let options = RenderOptions::default();

    render_v2_with_options(|| rsx! { <App /> }, options).await
}
```

### 2. Write Pure Render Functions

```rust
// ✅ Pure render - same input always produces same output
#[component]
fn PureComponent(count: i32) -> Element {
    // No side effects, no external state
    let doubled = count * 2;
    rsx! { <Text text={format!("Doubled: {}", doubled)} /> }
}
```

### 3. Always Provide Cleanup for Resources

```rust
// ✅ Always clean up resources
use_effect_v2(|| {
    let timer = start_timer();
    let subscription = subscribe();

    Some(move || {
        stop_timer(timer);
        unsubscribe(subscription);
    })
}, ());
```

### 4. Make Effects Idempotent

```rust
// ✅ Idempotent effect - safe to run multiple times
use_effect_v2(|| {
    // Setting state is idempotent
    set_initialized.set(true);
    None
}, ());
```

### 5. Use Refs for Mutable Values That Don't Trigger Re-renders

```rust
// ✅ Use refs for values that shouldn't trigger re-renders
#[component]
fn ComponentWithRef() -> Element {
    let render_count = use_ref_v2(|| 0);

    // Mutating ref doesn't cause re-render
    *render_count.borrow_mut() += 1;

    rsx! { <Text text="Hello" /> }
}
```

---

## FAQ

### Q: Why does my component render twice?

**A:** This is intentional in strict mode! It helps detect impure renders. Your component should produce the same output both times. If it doesn't, you have a bug.

### Q: Why does my effect run twice on mount?

**A:** Strict mode runs effects twice to verify cleanup works correctly. This simulates the component mounting, unmounting, and remounting - which can happen in real apps due to:

- React Suspense
- Fast refresh during development
- Component remounting due to key changes

### Q: Will strict mode affect my production app?

**A:** No! Strict mode is automatically disabled in release builds (`#[cfg(not(debug_assertions))]`). It has zero overhead in production.

### Q: How do I fix "different render results" warnings?

**A:** Your component is impure. Common causes:

- Using `rand()` or `Instant::now()` during render
- Reading from mutable global state
- Side effects during render

Move these to effects or use deterministic values.

### Q: Should I always use strict mode?

**A:** Yes, during development! It catches bugs early. Disable it only if you have a specific reason (e.g., debugging a specific issue where double-render interferes).

### Q: Does strict mode slow down my app?

**A:** In debug builds, yes - components render twice. But this is only during development. Release builds are unaffected.

### Q: How do I know if strict mode caught a bug?

**A:** Look for warnings in your console:

- "Component rendered different results!" - Impure render detected
- "Effect executed during render phase!" - Side effect during render

### Q: Can I disable strict mode for specific components?

**A:** Currently, strict mode is global. If you need to disable it for debugging, use `set_strict_mode_enabled(false)` temporarily.

---

## Summary

Strict mode is your friend during development:

| Feature                 | What It Does                    | What It Catches                 |
| ----------------------- | ------------------------------- | ------------------------------- |
| Double render           | Renders twice, compares results | Impure renders, external state  |
| Effect double-execution | Runs effects twice on mount     | Missing cleanup, resource leaks |
| Render warnings         | Logs when renders differ        | Non-deterministic components    |

Enable it early, fix the warnings, and ship more robust code! 🚀
