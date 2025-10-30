use super::*;
use crate::test_utils::{with_component_id, with_test_isolate};

#[derive(Clone, Debug, PartialEq)]
enum CounterAction {
    Increment,
    Decrement,
    Reset,
    SetValue(i32),
}

fn counter_reducer(state: i32, action: CounterAction) -> i32 {
    match action {
        CounterAction::Increment => state + 1,
        CounterAction::Decrement => state - 1,
        CounterAction::Reset => 0,
        CounterAction::SetValue(value) => value,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TodoState {
    todos: Vec<Todo>,
    next_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct Todo {
    id: u32,
    text: String,
    completed: bool,
}

#[derive(Clone, Debug)]
enum TodoAction {
    AddTodo(String),
    ToggleTodo(u32),
    RemoveTodo(u32),
    ClearCompleted,
}

fn todo_reducer(state: TodoState, action: TodoAction) -> TodoState {
    match action {
        TodoAction::AddTodo(text) => TodoState {
            todos: {
                let mut todos = state.todos;
                todos.push(Todo {
                    id: state.next_id,
                    text,
                    completed: false,
                });
                todos
            },
            next_id: state.next_id + 1,
        },
        TodoAction::ToggleTodo(id) => TodoState {
            todos: state
                .todos
                .into_iter()
                .map(|mut todo| {
                    if todo.id == id {
                        todo.completed = !todo.completed;
                    }
                    todo
                })
                .collect(),
            ..state
        },
        TodoAction::RemoveTodo(id) => TodoState {
            todos: state
                .todos
                .into_iter()
                .filter(|todo| todo.id != id)
                .collect(),
            ..state
        },
        TodoAction::ClearCompleted => TodoState {
            todos: state
                .todos
                .into_iter()
                .filter(|todo| !todo.completed)
                .collect(),
            ..state
        },
    }
}

#[test]
fn test_use_reducer_basic_counter() {
    with_test_isolate(|| {
        with_component_id("CounterComponent", |_context| {
            let (state, dispatch) = use_reducer(counter_reducer, 0);

            // Initial state
            assert_eq!(state.get(), 0);

            // Increment
            dispatch.call(CounterAction::Increment);
            assert_eq!(state.get(), 1);

            // Increment again
            dispatch.call(CounterAction::Increment);
            assert_eq!(state.get(), 2);

            // Decrement
            dispatch.call(CounterAction::Decrement);
            assert_eq!(state.get(), 1);

            // Reset
            dispatch.call(CounterAction::Reset);
            assert_eq!(state.get(), 0);

            // Set specific value
            dispatch.call(CounterAction::SetValue(42));
            assert_eq!(state.get(), 42);
        });
    });
}

#[test]
fn test_use_reducer_complex_state() {
    with_test_isolate(|| {
        with_component_id("TodoComponent", |_context| {
            let initial_state = TodoState {
                todos: vec![],
                next_id: 1,
            };

            let (state, dispatch) = use_reducer(todo_reducer, initial_state);

            // Initial state
            assert_eq!(state.get().todos.len(), 0);
            assert_eq!(state.get().next_id, 1);

            // Add first todo
            dispatch.call(TodoAction::AddTodo("Learn Rust".to_string()));
            let current_state = state.get();
            assert_eq!(current_state.todos.len(), 1);
            assert_eq!(current_state.todos[0].text, "Learn Rust");
            assert_eq!(current_state.todos[0].id, 1);
            assert!(!current_state.todos[0].completed);
            assert_eq!(current_state.next_id, 2);

            // Add second todo
            dispatch.call(TodoAction::AddTodo("Build TUI app".to_string()));
            let current_state = state.get();
            assert_eq!(current_state.todos.len(), 2);
            assert_eq!(current_state.todos[1].text, "Build TUI app");
            assert_eq!(current_state.todos[1].id, 2);
            assert_eq!(current_state.next_id, 3);

            // Toggle first todo
            dispatch.call(TodoAction::ToggleTodo(1));
            let current_state = state.get();
            assert!(current_state.todos[0].completed);
            assert!(!current_state.todos[1].completed);

            // Remove second todo
            dispatch.call(TodoAction::RemoveTodo(2));
            let current_state = state.get();
            assert_eq!(current_state.todos.len(), 1);
            assert_eq!(current_state.todos[0].id, 1);

            // Add another todo and mark it completed
            dispatch.call(TodoAction::AddTodo("Test app".to_string()));
            dispatch.call(TodoAction::ToggleTodo(3));
            let current_state = state.get();
            assert_eq!(current_state.todos.len(), 2);
            assert!(current_state.todos[0].completed);
            assert!(current_state.todos[1].completed);

            // Clear completed todos
            dispatch.call(TodoAction::ClearCompleted);
            let current_state = state.get();
            assert_eq!(current_state.todos.len(), 0);
        });
    });
}

#[test]
fn test_use_reducer_state_persistence() {
    with_test_isolate(|| {
        // First render cycle
        with_component_id("PersistentReducerComponent", |_context| {
            let (state, dispatch) = use_reducer(counter_reducer, 10);
            assert_eq!(state.get(), 10);

            dispatch.call(CounterAction::Increment);
            dispatch.call(CounterAction::Increment);
            assert_eq!(state.get(), 12);
        });

        // Second render cycle (same component ID)
        with_component_id("PersistentReducerComponent", |_context| {
            let (state, dispatch) = use_reducer(counter_reducer, 0); // Initial value ignored
            assert_eq!(state.get(), 12); // Should persist from previous cycle

            dispatch.call(CounterAction::SetValue(100));
            assert_eq!(state.get(), 100);
        });

        // Third render cycle
        with_component_id("PersistentReducerComponent", |_context| {
            let (state, _) = use_reducer(counter_reducer, 0);
            assert_eq!(state.get(), 100); // Should persist
        });
    });
}

#[test]
fn test_use_reducer_field_access() {
    with_test_isolate(|| {
        with_component_id("FieldAccessComponent", |_context| {
            let initial_state = TodoState {
                todos: vec![
                    Todo {
                        id: 1,
                        text: "First".to_string(),
                        completed: false,
                    },
                    Todo {
                        id: 2,
                        text: "Second".to_string(),
                        completed: true,
                    },
                ],
                next_id: 3,
            };

            let (state, _dispatch) = use_reducer(todo_reducer, initial_state);

            // Test field access without cloning entire state
            let todo_count = state.field(|s| s.todos.len());
            assert_eq!(todo_count, 2);

            let next_id = state.field(|s| s.next_id);
            assert_eq!(next_id, 3);

            let first_todo_text = state.field(|s| s.todos[0].text.clone());
            assert_eq!(first_todo_text, "First");

            let completed_count = state.field(|s| s.todos.iter().filter(|t| t.completed).count());
            assert_eq!(completed_count, 1);
        });
    });
}

#[test]
fn test_use_reducer_multiple_instances() {
    with_test_isolate(|| {
        with_component_id("MultipleReducerComponent", |_context| {
            // Two different reducer instances in the same component
            let (counter1, dispatch1) = use_reducer(counter_reducer, 0);
            let (counter2, dispatch2) = use_reducer(counter_reducer, 100);

            // Initial states should be different
            assert_eq!(counter1.get(), 0);
            assert_eq!(counter2.get(), 100);

            // Updates should be independent
            dispatch1.call(CounterAction::Increment);
            dispatch2.call(CounterAction::Decrement);

            assert_eq!(counter1.get(), 1);
            assert_eq!(counter2.get(), 99);

            // More updates
            dispatch1.call(CounterAction::SetValue(50));
            dispatch2.call(CounterAction::Reset);

            assert_eq!(counter1.get(), 50);
            assert_eq!(counter2.get(), 0);
        });
    });
}

/// Test version tracking for change detection
#[test]
fn test_use_reducer_version_tracking() {
    with_test_isolate(|| {
        with_component_id("VersionTrackingComponent", |_context| {
            let (state, dispatch) = use_reducer(counter_reducer, 0);

            // Initial version should be 0
            assert_eq!(state.version(), 0);
            assert_eq!(state.get(), 0);

            // First dispatch should increment version to 1
            dispatch.call(CounterAction::Increment);
            assert_eq!(state.version(), 1);
            assert_eq!(state.get(), 1);

            // Second dispatch should increment version to 2
            dispatch.call(CounterAction::Increment);
            assert_eq!(state.version(), 2);
            assert_eq!(state.get(), 2);

            // Different action should still increment version
            dispatch.call(CounterAction::Reset);
            assert_eq!(state.version(), 3);
            assert_eq!(state.get(), 0);

            // SetValue action should increment version
            dispatch.call(CounterAction::SetValue(42));
            assert_eq!(state.version(), 4);
            assert_eq!(state.get(), 42);

            // Multiple actions should continue incrementing
            dispatch.call(CounterAction::Decrement);
            dispatch.call(CounterAction::Decrement);
            assert_eq!(state.version(), 6);
            assert_eq!(state.get(), 40);
        });
    });
}
