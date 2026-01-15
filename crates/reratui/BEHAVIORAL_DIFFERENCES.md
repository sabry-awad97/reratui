# Behavioral Differences: React vs Reratui

This document outlines the key differences between React and Reratui for developers familiar with React.

## Overview

Reratui is inspired by React but adapted for terminal user interfaces in Rust. While the mental model is similar, there are important differences due to the different runtime environment and language.

## Component Model

### React

```jsx
function Counter({ initialCount }) {
  const [count, setCount] = useState(initialCount);
  return <div>Count: {count}</div>;
}
```

### Reratui

```rust
struct Counter {
    initial_count: i32,
}

impl ComponentV2 for Counter {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let (count, set_count) = use_state_v2(|| self.initial_count);
        Paragraph::new(format!("Count: {}", count))
            .render(area, buffer);
    }
}
```

**Key Differences:**

- Components are structs implementing `ComponentV2` trait
- Props are struct fields
- Render receives `area` and `buffer` instead of returning JSX
- Direct rendering to buffer instead of virtual DOM

## State Management

### State Updates

| React                        | Reratui                           |
| ---------------------------- | --------------------------------- |
| `setState(value)`            | `set_state.set(value)`            |
| `setState(prev => prev + 1)` | `set_state.update(\|c\| c + 1)`   |
| Automatic batching           | Explicit batching in render cycle |

### Reratui-Specific Methods

```rust
// Only update if value changed (avoids unnecessary re-renders)
set_count.set_if_changed(new_value);
set_count.update_if_changed(|c| c + 1);
```

### State Type Requirements

React: Any JavaScript value
Reratui: `T: Clone + Send + Sync + PartialEq + 'static`

## Effects

### React

```jsx
useEffect(() => {
  console.log("Effect ran");
  return () => console.log("Cleanup");
}, [dep]);
```

### Reratui

```rust
use_effect_v2(
    move || {
        println!("Effect ran");
        Some(Box::new(|| println!("Cleanup")))
    },
    dep,
);
```

**Key Differences:**

- Cleanup is `Option<Box<dyn FnOnce()>>` instead of optional return
- Dependencies are a single value (use tuples for multiple)
- No dependency array - single dependency or tuple

### Dependency Comparison

| React                  | Reratui                            |
| ---------------------- | ---------------------------------- |
| `[]` (empty array)     | `use_effect_once`                  |
| `[dep]`                | `use_effect_v2(..., dep)`          |
| `[dep1, dep2]`         | `use_effect_v2(..., (dep1, dep2))` |
| No deps (every render) | Not directly supported             |

## Context

### React

```jsx
// Provider
<ThemeContext.Provider value={theme}>
  <App />
</ThemeContext.Provider>;

// Consumer
const theme = useContext(ThemeContext);
```

### Reratui

```rust
// Provider (inside component)
use_context_provider_v2(|| theme.clone());

// Consumer
let theme = use_context_v2::<Theme>();
```

**Key Differences:**

- No separate Context object creation
- Provider is a hook, not a component wrapper
- Type-based lookup instead of context object
- `try_use_context_v2` for optional context

## Refs

### React

```jsx
const ref = useRef(initialValue);
ref.current = newValue;
console.log(ref.current);
```

### Reratui

```rust
let ref_handle = use_ref_v2(|| initial_value);
ref_handle.set(new_value);
let value = ref_handle.get();
```

**Key Differences:**

- Methods instead of `.current` property
- `get()` returns cloned value
- `update(fn)` for functional updates

## Memoization

### React

```jsx
const memoized = useMemo(() => expensive(), [dep]);
const callback = useCallback(() => doSomething(), [dep]);
```

### Reratui

```rust
let memoized = use_memo_v2(|| expensive(), dep);
let callback = use_callback_v2(|| do_something(), dep);
```

**Key Differences:**

- Single dependency value (use tuples for multiple)
- Callback returns `CallbackV2<F>` wrapper

## Event Handling

### React

```jsx
<button onClick={(e) => handleClick(e)}>Click</button>
```

### Reratui

```rust
use_keyboard_press_v2(move |key| {
    if key.code == KeyCode::Enter {
        handle_click();
    }
});

use_mouse_click_v2(move |button, x, y| {
    if button == MouseButton::Left {
        handle_click();
    }
});
```

**Key Differences:**

- No JSX event props
- Hooks for event handling
- Separate hooks for keyboard and mouse
- Terminal events instead of DOM events

## Async Data Fetching

### React (with React Query)

```jsx
const { data, isLoading, error, refetch } = useQuery({
  queryKey: ["users"],
  queryFn: fetchUsers,
  staleTime: 30000,
});
```

### Reratui

```rust
let query = use_query_v2(
    "users",
    || async { fetch_users().await },
    Some(QueryOptions {
        stale_time: Duration::from_secs(30),
        ..Default::default()
    }),
);

// Access: query.data, query.status, query.error, query.refetch()
```

**Key Differences:**

- Built-in (no separate library)
- `QueryStatus` enum instead of boolean flags
- Duration types instead of milliseconds

## Rendering

### React

- Virtual DOM diffing
- Reconciliation algorithm
- Automatic re-renders on state change

### Reratui

- Direct buffer rendering
- Fiber-based architecture
- 5-phase render pipeline:
  1. Poll (wait for events)
  2. Render (execute components)
  3. Commit (apply state updates)
  4. Event (process terminal events)
  5. Effect (run effects)

## Lifecycle

### React Lifecycle

```
Mount → Update → Unmount
```

### Reratui Lifecycle

```
Mount → Render Loop → Unmount
         ↓
    Poll → Render → Commit → Event → Effect
         ↑___________________________|
```

## No Direct Equivalents

### React Features Not in Reratui

| React               | Reratui Alternative                        |
| ------------------- | ------------------------------------------ |
| JSX                 | Direct widget rendering                    |
| Suspense            | Manual loading states                      |
| Error Boundaries    | Manual error handling                      |
| Portals             | Not applicable (single buffer)             |
| Fragments           | Not needed                                 |
| forwardRef          | Not applicable                             |
| useImperativeHandle | Not applicable                             |
| useLayoutEffect     | `use_effect_v2` (all effects are "layout") |
| useDeferredValue    | Not available                              |
| useTransition       | Not available                              |
| Server Components   | Not applicable                             |

### Reratui-Specific Features

| Feature              | Description               |
| -------------------- | ------------------------- |
| `use_keyboard_v2`    | Terminal keyboard events  |
| `use_mouse_v2`       | Terminal mouse events     |
| `use_area_v2`        | Component render area     |
| `use_frame_v2`       | Frame timing info         |
| `use_resize_v2`      | Terminal resize events    |
| `use_media_query_v2` | Terminal size breakpoints |
| `use_history_v2`     | Undo/redo state           |
| `use_form_v2`        | Form validation           |
| `use_timeout_v2`     | Timeout with handle       |
| `use_interval_v2`    | Interval with handle      |

## Threading Model

### React

- Single-threaded (main thread)
- Concurrent features for interruptible rendering

### Reratui

- Async runtime (Tokio)
- State must be `Send + Sync`
- Effects can spawn async tasks

## Type Safety

### React

- Runtime prop validation (PropTypes) or TypeScript
- Hooks can return any type

### Reratui

- Compile-time type checking
- Strict type requirements on hooks
- Generic constraints enforced

## Performance Considerations

### React

- Virtual DOM overhead
- Reconciliation cost
- Memoization for optimization

### Reratui

- Direct buffer rendering (no VDOM)
- Fiber-based dirty tracking
- State batching
- `set_if_changed` / `update_if_changed` for optimization

## Migration Tips

1. **Think in traits** - Components are trait implementations, not functions
2. **Embrace ownership** - Clone when needed, use `Arc` for shared state
3. **Single dependency** - Use tuples for multiple effect dependencies
4. **Direct rendering** - No JSX, render directly to buffer
5. **Terminal events** - Use keyboard/mouse hooks instead of event props
6. **Type constraints** - Ensure state types meet trait bounds
7. **Async runtime** - Tokio is required for async features
