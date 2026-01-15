# Migration Guide

This guide helps you migrate from older versions of Reratui to the current v2 API.

## Migrating to v0.2.x (Fiber Architecture)

Version 0.2.x introduced the fiber-based architecture with the `ComponentV2` trait and `_v2` hooks. This is a significant change from the previous API.

### Component Changes

#### Before (v0.1.x)

```rust
// Old component pattern
fn my_component(frame: &mut Frame, area: Rect) {
    let count = use_state(|| 0);
    // ...
}
```

#### After (v0.2.x)

```rust
use reratui::prelude::*;

struct MyComponent;

impl ComponentV2 for MyComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let (count, set_count) = use_state_v2(|| 0);
        // ...
    }
}
```

### Hook Changes

All hooks have been renamed with the `_v2` suffix and updated signatures:

| Old Hook       | New Hook          |
| -------------- | ----------------- |
| `use_state`    | `use_state_v2`    |
| `use_effect`   | `use_effect_v2`   |
| `use_context`  | `use_context_v2`  |
| `use_ref`      | `use_ref_v2`      |
| `use_memo`     | `use_memo_v2`     |
| `use_callback` | `use_callback_v2` |
| `use_reducer`  | `use_reducer_v2`  |

### State Hook Changes

#### Before

```rust
let count = use_state(|| 0);
count.set(5);
count.update(|c| *c + 1);
```

#### After

```rust
let (count, set_count) = use_state_v2(|| 0);
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
use_effect_v2(
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
use_context_provider_v2(|| theme.clone());

// Consumer
let theme = use_context_v2::<Theme>();

// Optional consumer
let theme = try_use_context_v2::<Theme>();
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
let my_ref = use_ref_v2(|| initial_value);
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
let handle = use_future_v2(
    || async { fetch_data().await },
    Some(deps),
);

// With caching
let query = use_query_v2(
    "cache-key",
    || async { fetch_data().await },
    Some(QueryOptions::default()),
);

// For mutations
let mutation = use_mutation_v2(
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
use_keyboard_press_v2(move |key| {
    match key.code {
        KeyCode::Char('q') => request_exit_v2(),
        _ => {}
    }
});

use_keyboard_shortcut_v2(
    KeyCode::Char('s'),
    KeyModifiers::CONTROL,
    || save(),
);

use_mouse_click_v2(move |button, x, y| {
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
    render_v2(|| App).await?;
    Ok(())
}

// With options
render_v2_with_options(|| App, RenderOptions {
    frame_interval_ms: 16,
    strict_mode: true,
}).await?;
```

**Key Changes:**

- Async runtime required (Tokio)
- `render_v2` instead of `run`
- Returns `Result`
- Optional `RenderOptions`

### Exit Handling Changes

#### Before

```rust
exit();
```

#### After

```rust
request_exit_v2();

// Check exit status
if should_exit_v2() {
    // ...
}

// Cancel exit
reset_exit_v2();
```

## New Features in v0.2.x

### Timing Hooks

```rust
// Timeout
let timeout = use_timeout_v2(|| println!("Fired!"), 5000);
timeout.cancel();
timeout.reset();

// Interval
let interval = use_interval_v2(|| println!("Tick!"), 1000);
interval.pause();
interval.resume();
```

### History Hook

```rust
let history = use_history_v2(|| String::new());
history.set("Hello".to_string());
history.undo();
history.redo();
```

### Form Hook

```rust
let form = use_form_v2(
    FormConfigV2::builder()
        .field("email", "")
        .validator("email", ValidatorV2::required("Required"))
        .validator("email", ValidatorV2::email("Invalid"))
        .on_submit(|values| println!("{:?}", values))
        .build()
);
```

### Layout Hooks

```rust
let area = use_area_v2();
let frame = use_frame_v2();
let (width, height) = use_resize_v2();
let is_narrow = use_media_query_v2(|(w, _)| w < 80);
```

### Mouse Hooks

```rust
let is_hovering = use_mouse_hover_v2(button_area);
let (drag_info, reset_drag) = use_mouse_drag_v2();
let (x, y) = use_mouse_position_v2();
use_double_click_v2(Duration::from_millis(500), |btn, x, y| {});
```

## Checklist for Migration

- [ ] Update `Cargo.toml` to latest version
- [ ] Add `tokio` dependency with `full` features
- [ ] Convert function components to `ComponentV2` structs
- [ ] Update all hook calls to `_v2` versions
- [ ] Update state hook usage to tuple pattern
- [ ] Update effect cleanup to `Option<Box<...>>`
- [ ] Convert dependency arrays to single values/tuples
- [ ] Update event handling to use new hooks
- [ ] Update main function to async with `render_v2`
- [ ] Replace `exit()` with `request_exit_v2()`
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
use_effect_v2(
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
use_effect_v2(..., (dep1, dep2));
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
    render_v2(|| App).await?;
    Ok(())
}
```
