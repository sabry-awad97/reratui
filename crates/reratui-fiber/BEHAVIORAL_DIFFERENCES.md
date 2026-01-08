# Behavioral Differences: Old vs New APIs

This document details the specific behavioral differences between the deprecated APIs in `reratui-hooks` and `reratui-runtime` and the new React-like APIs in `reratui-fiber`.

## Overview

The new `reratui-fiber` crate implements React's Fiber architecture to provide proper component lifecycle management, effect timing, and state batching. This results in several important behavioral changes that developers need to understand when migrating.

## Critical Behavioral Changes

### 1. Effect Execution Timing

**The most important difference** - effects now run at the correct time.

#### Old Behavior (`use_effect`)

```rust
use reratui::prelude::*;

#[component]
fn OldComponent() -> Element {
    let (count, set_count) = use_state(|| 0);

    // ❌ PROBLEM: Effect runs DURING render phase
    use_effect(|| {
        println!("Count: {}", count); // Prints BEFORE screen updates
        // This blocks rendering and can cause performance issues
        None
    }, (count,));

    rsx! { <Text text={count.to_string()} /> }
}
```

**Issues:**

- Effects run synchronously during component render
- Screen hasn't been updated yet when effect runs
- Can block rendering and cause performance problems
- Side effects during render violate React's principles

#### New Behavior (`use_effect_v2`)

```rust
use reratui_fiber::prelude::*;

#[component]
fn NewComponent() -> Element {
    let (count, set_count) = use_state_v2(|| 0);

    // ✅ CORRECT: Effect runs AFTER commit phase
    use_effect_v2(|| {
        println!("Count: {}", count); // Prints AFTER screen shows new count
        // Screen has already been updated when this runs
        None
    }, (count,));

    rsx! { <Text text={count.to_string()} /> }
}
```

**Benefits:**

- Effects run after the screen has been updated
- Non-blocking - render phase is pure and fast
- Matches React's behavior exactly
- Better performance and user experience

### 2. Hook Identity and State Isolation

**Critical for conditional rendering and component reordering.**

#### Old Behavior (Global Hook Index)

```rust
use reratui::prelude::*;

#[component]
fn OldConditional() -> Element {
    let (show_details, set_show) = use_state(|| false); // Hook index 0

    rsx! {
        <Layout>
            <Button on_click={move |_| set_show.update(|v| !v)} />

            // ❌ PROBLEM: Conditional rendering corrupts hook indices
            {if show_details {
                rsx! { <Details /> }  // Details' hooks start at index 1
            } else {
                rsx! { <Summary /> }  // Summary's hooks also start at index 1
            }}
        </Layout>
    }
}

#[component]
fn Details() -> Element {
    let (expanded, set_expanded) = use_state(|| false); // Hook index 1
    // When Details unmounts and Summary mounts, Summary gets Details' state!
    rsx! { <Block /> }
}

#[component]
fn Summary() -> Element {
    let (count, set_count) = use_state(|| 0); // Also hook index 1 - COLLISION!
    // Gets leftover state from Details component
    rsx! { <Block /> }
}
```

**Problems:**

- Global hook index shared across all components
- Conditional rendering causes hook index collisions
- Component reordering corrupts state
- State "bleeds" between different components

#### New Behavior (Fiber-Scoped Hooks)

```rust
use reratui_fiber::prelude::*;

#[component]
fn NewConditional() -> Element {
    let (show_details, set_show) = use_state_v2(|| false); // Fiber-scoped

    rsx! {
        <Layout>
            <Button on_click={move |_| set_show.update(|v| !v)} />

            // ✅ CORRECT: Each component has isolated hook state
            {if show_details {
                rsx! { <Details /> }  // Details has its own Fiber
            } else {
                rsx! { <Summary /> }  // Summary has its own Fiber
            }}
        </Layout>
    }
}

#[component]
fn Details() -> Element {
    let (expanded, set_expanded) = use_state_v2(|| false); // Isolated to Details Fiber
    rsx! { <Block /> }
}

#[component]
fn Summary() -> Element {
    let (count, set_count) = use_state_v2(|| 0); // Isolated to Summary Fiber
    rsx! { <Block /> }
}
```

**Benefits:**

- Each component instance has its own Fiber with isolated hook state
- Conditional rendering is safe - no hook collisions
- Component reordering preserves individual state
- Matches React's component isolation

### 3. State Update Batching

**Multiple state updates now batch into a single re-render.**

#### Old Behavior (Immediate Updates)

```rust
use reratui::prelude::*;

#[component]
fn OldBatching() -> Element {
    let (count, set_count) = use_state(|| 0);
    let (name, set_name) = use_state(|| String::new());

    let handle_click = {
        let set_count = set_count.clone();
        let set_name = set_name.clone();
        move |_| {
            // ❌ PROBLEM: Each update triggers immediate re-render
            set_count.set(count + 1);    // Re-render #1
            set_count.set(count + 2);    // Re-render #2
            set_name.set("Updated".into()); // Re-render #3
            // Total: 3 re-renders for one user action!
        }
    };

    rsx! { <Button on_click={handle_click} /> }
}
```

**Problems:**

- Each `set_state` call triggers immediate re-render
- Multiple updates in same event handler cause multiple re-renders
- Poor performance and flickering UI
- Intermediate states are visible to user

#### New Behavior (Batched Updates)

```rust
use reratui_fiber::prelude::*;

#[component]
fn NewBatching() -> Element {
    let (count, set_count) = use_state_v2(|| 0);
    let (name, set_name) = use_state_v2(|| String::new());

    let handle_click = {
        let set_count = set_count.clone();
        let set_name = set_name.clone();
        move |_| {
            // ✅ CORRECT: All updates batched into single re-render
            set_count.set(count + 1);       // Queued
            set_count.set(count + 2);       // Queued (overwrites previous)
            set_name.set("Updated".into()); // Queued
            // Total: 1 re-render for all updates!
        }
    };

    rsx! { <Button on_click={handle_click} /> }
}
```

**Benefits:**

- Multiple state updates in same event handler are batched
- Only one re-render per user action
- Better performance and smoother UI
- Matches React's batching behavior

### 4. Functional State Updates

**Functional updates now receive the latest state value.**

#### Old Behavior (Stale Closures)

```rust
use reratui::prelude::*;

#[component]
fn OldFunctionalUpdate() -> Element {
    let (count, set_count) = use_state(|| 0);

    let increment_twice = {
        let set_count = set_count.clone();
        move |_| {
            // ❌ PROBLEM: Both updates see the same stale value
            set_count.update(|n| n + 1); // n = 0, sets to 1
            set_count.update(|n| n + 1); // n = 0 again!, sets to 1
            // Result: count = 1 (should be 2)
        }
    };

    rsx! { <Button on_click={increment_twice} /> }
}
```

**Problems:**

- Functional updates see stale state from closure capture
- Multiple functional updates don't chain properly
- Final state is incorrect

#### New Behavior (Latest State)

```rust
use reratui_fiber::prelude::*;

#[component]
fn NewFunctionalUpdate() -> Element {
    let (count, set_count) = use_state_v2(|| 0);

    let increment_twice = {
        let set_count = set_count.clone();
        move |_| {
            // ✅ CORRECT: Each update receives latest state
            set_count.update(|n| n + 1); // n = 0, queues update to 1
            set_count.update(|n| n + 1); // n = 1, queues update to 2
            // Result: count = 2 (correct!)
        }
    };

    rsx! { <Button on_click={increment_twice} /> }
}
```

**Benefits:**

- Functional updates receive the latest state value
- Multiple functional updates chain correctly
- Predictable state transitions

### 5. Context Provider Lifecycle

**Context values are now properly scoped and cleaned up.**

#### Old Behavior (Memory Leak)

```rust
use reratui::prelude::*;

#[component]
fn OldContextProvider() -> Element {
    // ❌ PROBLEM: Context value never cleaned up
    let _theme = use_context_provider(|| Theme::default());
    // When component unmounts, theme stays in global context stack
    // Memory leak and incorrect scoping for nested providers

    rsx! { <Child /> }
}
```

**Problems:**

- Context values pushed to global stack but never popped
- Memory leak as values accumulate
- Nested providers don't shadow correctly
- Context persists after provider unmounts

#### New Behavior (Proper Lifecycle)

```rust
use reratui_fiber::prelude::*;

#[component]
fn NewContextProvider() -> Element {
    // ✅ CORRECT: Context automatically cleaned up on unmount
    let _theme = use_context_provider_v2(|| Theme::default());
    // When component unmounts, theme is automatically removed from stack

    rsx! { <Child /> }
}
```

**Benefits:**

- Context values automatically cleaned up when provider unmounts
- No memory leaks
- Proper scoping for nested providers
- Matches React's context lifecycle

### 6. Render Phase Purity

**The render phase is now pure and side-effect free.**

#### Old Behavior (Side Effects During Render)

```rust
use reratui::prelude::*;

#[component]
fn OldRenderPurity() -> Element {
    let (count, set_count) = use_state(|| 0);

    // ❌ PROBLEM: Side effects run during render
    use_effect(|| {
        // This runs during render phase, blocking screen updates
        println!("Side effect during render!");
        fetch_data(); // Network call blocks rendering!
        None
    }, ());

    rsx! { <Text text={count.to_string()} /> }
}
```

**Problems:**

- Side effects run during render phase
- Blocks screen updates
- Can't be interrupted or aborted
- Violates React's render purity principles

#### New Behavior (Pure Render Phase)

```rust
use reratui_fiber::prelude::*;

#[component]
fn NewRenderPurity() -> Element {
    let (count, set_count) = use_state_v2(|| 0);

    // ✅ CORRECT: Effects queued during render, executed after commit
    use_effect_v2(|| {
        // This runs AFTER screen has been updated
        println!("Side effect after commit!");
        fetch_data(); // Doesn't block rendering
        None
    }, ());

    rsx! { <Text text={count.to_string()} /> }
}
```

**Benefits:**

- Render phase is pure - only computes output
- Screen updates are not blocked by side effects
- Render can be interrupted or aborted if needed
- Better performance and responsiveness

## Render Pipeline Differences

### Old Pipeline (Immediate)

```
┌─────────────────────────────────────────┐
│           Old Render Pipeline           │
├─────────────────────────────────────────┤
│                                         │
│  1. Process Events                      │
│  2. Execute Components                  │
│     ├─ Run effects immediately          │
│     ├─ Update state immediately         │
│     └─ Side effects block rendering     │
│  3. Draw to terminal                    │
│                                         │
└─────────────────────────────────────────┘
```

### New Pipeline (React-like)

```
┌─────────────────────────────────────────┐
│           New Render Pipeline           │
├─────────────────────────────────────────┤
│                                         │
│  1. EVENT PHASE                         │
│     ├─ begin_batch()                    │
│     ├─ Process events                   │
│     └─ end_batch() → dirty fibers       │
│                                         │
│  2. RENDER PHASE (Pure)                 │
│     ├─ Execute components               │
│     ├─ Queue effects (don't run)        │
│     └─ Build VNode tree                 │
│                                         │
│  3. COMMIT PHASE                        │
│     ├─ Apply changes to terminal        │
│     ├─ Process unmounts                 │
│     └─ terminal.draw()                  │
│                                         │
│  4. EFFECT PHASE                        │
│     ├─ Run cleanup functions            │
│     └─ Run queued effects               │
│                                         │
└─────────────────────────────────────────┘
```

## Hook Comparison Table

| Feature              | Old API                      | New API                         | Key Difference               |
| -------------------- | ---------------------------- | ------------------------------- | ---------------------------- |
| **State**            | `use_state(init)`            | `use_state_v2(init)`            | Batching, functional updates |
| **Effects**          | `use_effect(fn, deps)`       | `use_effect_v2(fn, deps)`       | Post-commit execution        |
| **Async Effects**    | `use_async_effect(fn, deps)` | `use_async_effect_v2(fn, deps)` | Post-commit + async cleanup  |
| **Context Provider** | `use_context_provider(fn)`   | `use_context_provider_v2(fn)`   | Automatic cleanup            |
| **Context Consumer** | `use_context::<T>()`         | `use_context_v2::<T>()`         | Proper scoping               |
| **Memo**             | `use_memo(fn, deps)`         | `use_memo_v2(fn, deps)`         | Fiber-scoped                 |
| **Callback**         | `use_callback(fn, deps)`     | `use_callback_v2(fn, deps)`     | Fiber-scoped                 |
| **Render**           | `render(component)`          | `render_v2(component)`          | 4-phase pipeline             |

## Migration Checklist

When migrating from old to new APIs:

### ✅ Safe Migrations (No Behavior Change)

- [ ] Replace `use_state` with `use_state_v2`
- [ ] Replace `use_memo` with `use_memo_v2`
- [ ] Replace `use_callback` with `use_callback_v2`
- [ ] Replace `render` with `render_v2`

### ⚠️ Behavior Changes (Review Required)

- [ ] Replace `use_effect` with `use_effect_v2`
  - **Review:** Effects now run after commit, not during render
  - **Action:** Verify timing assumptions are still correct
- [ ] Replace `use_async_effect` with `use_async_effect_v2`
  - **Review:** Same timing change as effects
  - **Action:** Check async effect dependencies
- [ ] Replace `use_context_provider` with `use_context_provider_v2`
  - **Review:** Context now cleaned up on unmount
  - **Action:** Verify nested provider behavior
- [ ] Replace `use_context` with `use_context_v2`
  - **Review:** Proper scoping may change behavior
  - **Action:** Test context consumption in nested components

### 🔍 Test After Migration

- [ ] **Effect timing:** Verify effects run at expected times
- [ ] **State batching:** Check that multiple updates batch correctly
- [ ] **Conditional rendering:** Test components that render conditionally
- [ ] **Context scoping:** Verify nested providers work correctly
- [ ] **Component reordering:** Test dynamic component lists

## Common Migration Issues

### Issue 1: Effect Timing Assumptions

```rust
// ❌ Old code that assumes effect runs during render
use_effect(|| {
    // This used to run before screen update
    assert_eq!(get_screen_content(), old_content); // May fail now
    None
}, (state,));

// ✅ Updated code that works with post-commit timing
use_effect_v2(|| {
    // This runs after screen update
    assert_eq!(get_screen_content(), new_content); // Correct
    None
}, (state,));
```

### Issue 2: State Update Expectations

```rust
// ❌ Old code expecting immediate state updates
let handle_click = move |_| {
    set_count.set(5);
    println!("Count: {}", count); // Still old value
};

// ✅ Updated code that works with batching
let handle_click = move |_| {
    set_count.set(5);
    // State update is queued, will be applied before next render
    // Use effect to observe new value:
};

use_effect_v2(|| {
    println!("Count: {}", count); // New value
    None
}, (count,));
```

### Issue 3: Context Scoping Changes

```rust
// ❌ Old code that relied on context leaking
#[component]
fn Parent() -> Element {
    let _theme = use_context_provider(|| Theme::dark());
    rsx! { <Child /> }
}

#[component]
fn Child() -> Element {
    // This component unmounts but context stays
    rsx! { <GrandChild /> }
}

#[component]
fn GrandChild() -> Element {
    let theme = use_context::<Theme>(); // Used to work even after Child unmounted
    rsx! { <Block /> }
}

// ✅ Updated code with proper scoping
// Move context provider to appropriate level
#[component]
fn App() -> Element {
    let _theme = use_context_provider_v2(|| Theme::dark());
    rsx! {
        <Parent />
        <GrandChild /> // Now properly scoped
    }
}
```

## Performance Implications

### Old API Performance Issues

- Effects run during render → blocking
- No state batching → excessive re-renders
- Global hook index → cache misses
- Context leaks → memory growth

### New API Performance Benefits

- Effects run after commit → non-blocking
- State batching → fewer re-renders
- Fiber-scoped hooks → better cache locality
- Proper cleanup → stable memory usage

## Debugging Differences

### Old API Debugging

```rust
// Limited debugging - effects run immediately
use_effect(|| {
    println!("Effect ran"); // Prints during render
    None
}, ());
```

### New API Debugging

```rust
// Better debugging with strict mode
#[cfg(debug_assertions)]
set_strict_mode_enabled(true);

use_effect_v2(|| {
    println!("Effect ran"); // Prints after commit
    None
}, ());

// Strict mode will:
// - Double-render components to catch impure renders
// - Run effects twice on mount to catch missing cleanup
// - Warn about render inconsistencies
```

## Conclusion

The new `reratui-fiber` APIs provide significant behavioral improvements that align with React's proven patterns:

1. **Correct effect timing** prevents render blocking
2. **Fiber-scoped hooks** eliminate state corruption
3. **State batching** improves performance
4. **Proper context lifecycle** prevents memory leaks
5. **Pure render phase** enables future optimizations

While migration requires careful review of timing assumptions, the new APIs provide a more robust and performant foundation for building TUI applications.
