# Migration Guide

This guide helps you migrate from older versions of Reratui to the current API.

## Migrating to v1.0.0 (V2 Suffix Removal)

Version 1.0.0 removes all `_v2` and `V2` suffixes from the API. The fiber-based architecture introduced in v0.2.x is now the standard API.

### Breaking Changes

All APIs that previously had `_v2` or `V2` suffixes now use clean names:

| Old Name (v0.2.x)  | New Name (v1.0.0) |
| ------------------ | ----------------- |
| `StateSetterV2`    | `StateSetter`     |
| `CallbackV2`       | `Callback`        |
| `DispatchV2`       | `Dispatch`        |
| `RefV2`            | `Ref`             |
| `EffectEventV2`    | `EffectEvent`     |
| `QueryResultV2`    | `QueryResult`     |
| `MutationHandleV2` | `MutationHandle`  |
| `FutureHandleV2`   | `FutureHandle`    |
| `FormHandleV2`     | `FormHandle`      |
| `FormStateV2`      | `FormState`       |
| `FormConfigV2`     | `FormConfig`      |
| `ValidatorV2`      | `Validator`       |

### Example Updates

The example directories have been renamed:

| Old Name           | New Name        |
| ------------------ | --------------- |
| `counter_v2`       | `counter_fiber` |
| `effect_timing_v2` | `effect_timing` |

### Migration Steps

1. Update your `Cargo.toml` to use the new version
2. Find and replace all `V2` suffixes in your code
3. Update any example references

## Migrating from v0.1.x to v0.2.x+ (Fiber Architecture)

Version 0.2.x introduced the fiber-based architecture with the `Component` trait. This is a significant change from the previous API.

### Component Changes

#### Before (v0.1.x)

```rust
// Old component pattern
fn my_component(frame: &mut Frame, area: Rect) {
    let count = use_state(|| 0);
    // ...
}
```

#### After (v0.2.x+)

```rust
use reratui::prelude::*;

struct MyComponent;

impl Component for MyComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let (count, set_count) = use_state(|| 0);
        // ...
    }
}
```

### State Hook Changes

#### Before

```rust
let count = use_state(|| 0);
count.set(5);
count.update(|c| *c + 1);
```

#### After

```rust
let (count, set_count) = use_state(|| 0);
set_count.set(5);
set_count.update(|c| c + 1);

// New methods
set_count.set_if_changed(5);
set_count.update_if_changed(|c| c + 1);
```

**Key Changes:**

- Returns tuple `(value, setter)` instead of handle
- Setter is separate from value
- New conditional update methods

### Effect Hook Changes

#### Before

```rust
use_effect(|| {
    println!("Effect ran");
    || println!("Cleanup")
}, &[dep]);
```

#### After

```rust
use_effect(
    move || {
        println!("Effect ran");
        Some(Box::new(|| println!("Cleanup")))
    },
    dep,
);

// Or for mount-only effects
use_effect_once(|| {
    println!("Mounted");
    Some(Box::new(|| println!("Unmounting")))
});
```

**Key Changes:**

- Cleanup is `Option<Box<dyn FnOnce()>>`
- Single dependency value (use tuples for multiple)
- `use_effect_once` for mount-only effects

### Context Hook Changes

#### Before

```rust
// Provider
provide_context(theme);

// Consumer
let theme = use_context::<Theme>();
```

#### After

```rust
// Provider
use_context_provider(|| theme.clone());

// Consumer
let theme = use_context::<Theme>();

// Optional consumer
let theme = try_use_context::<Theme>();
```

### Ref Hook Changes

#### Before

```rust
let my_ref = use_ref(|| initial_value);
my_ref.set(new_value);
let value = my_ref.get();
```

#### After

```rust
let my_ref = use_ref(|| initial_value);
my_ref.set(new_value);
let value = my_ref.get();
my_ref.update(|v| *v + 1);
```

**Key Changes:**

- Added `update` method for functional updates

### Async Hook Changes

#### Before

```rust
let data = use_async(|| async { fetch_data().await });
```

#### After

```rust
// Simple async
let handle = use_future(
    || async { fetch_data().await },
    Some(deps),
);

// With caching
let query = use_query(
    "cache-key",
    || async { fetch_data().await },
    Some(QueryOptions::default()),
);

// For mutations
let mutation = use_mutation(
    |args| async move { mutate_data(args).await },
    None,
);
```

### Event Handling Changes

#### Before

```rust
if let Some(Event::Key(key)) = get_event() {
    // Handle key
}
```

#### After

```rust
// Raw event access
if let Some(Event::Key(key)) = use_event() {
    // Handle key
}

// Or use specialized hooks
use_keyboard_press(move |key| {
    match key.code {
        KeyCode::Char('q') => request_exit(),
        _ => {}
    }
});

use_keyboard_shortcut(
    KeyCode::Char('s'),
    KeyModifiers::CONTROL,
    || save(),
);

use_mouse_click(move |button, x, y| {
    // Handle click
});
```

### Runtime Changes

#### Before

```rust
fn main() {
    run(my_app);
}
```

#### After

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| App).await?;
    Ok(())
}

// With options
render_with_options(|| App, RenderOptions {
    frame_interval_ms: 16,
    strict_mode: true,
}).await?;
```

**Key Changes:**

- Async runtime required (Tokio)
- `render` instead of `run`
- Returns `Result`
- Optional `RenderOptions`

### Exit Handling Changes

#### Before

```rust
exit();
```

#### After

```rust
request_exit();

// Check exit status
if should_exit() {
    // ...
}

// Cancel exit
reset_exit();
```

## Features Available in Current Version

### Timing Hooks

```rust
// Timeout
let timeout = use_timeout(|| println!("Fired!"), 5000);
timeout.cancel();
timeout.reset();

// Interval
let interval = use_interval(|| println!("Tick!"), 1000);
interval.pause();
interval.resume();
```

### History Hook

```rust
let history = use_history(|| String::new());
history.set("Hello".to_string());
history.undo();
history.redo();
```

### Form Hook

```rust
let form = use_form(
    FormConfig::builder()
        .field("email", "")
        .validator("email", Validator::required("Required"))
        .validator("email", Validator::email("Invalid"))
        .on_submit(|values| println!("{:?}", values))
        .build()
);
```

### Layout Hooks

```rust
let area = use_area();
let frame = use_frame();
let (width, height) = use_resize();
let is_narrow = use_media_query(|(w, _)| w < 80);
```

### Mouse Hooks

```rust
let is_hovering = use_mouse_hover(button_area);
let (drag_info, reset_drag) = use_mouse_drag();
let (x, y) = use_mouse_position();
use_double_click(Duration::from_millis(500), |btn, x, y| {});
```

## Checklist for Migration

### From v0.2.x to v1.0.0

- [ ] Update `Cargo.toml` to latest version
- [ ] Find and replace `StateSetterV2` → `StateSetter`
- [ ] Find and replace `CallbackV2` → `Callback`
- [ ] Find and replace other `V2` suffixed types
- [ ] Update example references (`counter_v2` → `counter_fiber`, etc.)

### From v0.1.x to v1.0.0

- [ ] Update `Cargo.toml` to latest version
- [ ] Add `tokio` dependency with `full` features
- [ ] Convert function components to `Component` structs
- [ ] Update state hook usage to tuple pattern
- [ ] Update effect cleanup to `Option<Box<...>>`
- [ ] Convert dependency arrays to single values/tuples
- [ ] Update event handling to use new hooks
- [ ] Update main function to async with `render`
- [ ] Replace `exit()` with `request_exit()`
- [ ] Test thoroughly with strict mode enabled

## Common Migration Issues

### Issue: State type doesn't implement required traits

```
error: the trait bound `MyType: Send` is not satisfied
```

**Solution:** Ensure your state types implement `Clone + Send + Sync + PartialEq + 'static`:

```rust
#[derive(Clone, PartialEq)]
struct MyType {
    // fields
}
```

### Issue: Effect cleanup type mismatch

```
error: expected `Option<Box<dyn FnOnce() + Send>>`
```

**Solution:** Wrap cleanup in `Some(Box::new(...))`:

```rust
use_effect(
    || {
        // effect
        Some(Box::new(|| {
            // cleanup
        }))
    },
    deps,
);
```

### Issue: Multiple dependencies

```
error: expected single value, found array
```

**Solution:** Use a tuple:

```rust
// Before
use_effect(..., &[dep1, dep2]);

// After
use_effect(..., (dep1, dep2));
```

### Issue: Missing async runtime

```
error: there is no reactor running
```

**Solution:** Use `#[tokio::main]` and add tokio dependency:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| App).await?;
    Ok(())
}
```
