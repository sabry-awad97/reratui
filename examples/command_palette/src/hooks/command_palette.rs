use std::sync::Arc;

use reratui::prelude::{StateHandle, StateSetter, use_state};
use std::collections::HashMap;

/// A command that can be executed
#[derive(Clone)]
pub struct Command {
    /// The name of the command
    pub name: String,
    /// Description of what the command does
    pub description: String,
    /// The function to execute when the command is triggered
    action: Arc<dyn Fn() + Send + Sync>,
}

// Manual implementation of PartialEq that ignores the action closure
impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.description == other.description
        // We intentionally don't compare the action field
    }
}

/// Manages keyboard commands and command palette UI
pub struct CommandPalette {
    commands: StateHandle<HashMap<String, Command>>,
    set_commands: StateSetter<HashMap<String, Command>>,
    is_visible: StateHandle<bool>,
    set_is_visible: StateSetter<bool>,
    selected_index: StateHandle<usize>,
    set_selected_index: StateSetter<usize>,
    filter_text: StateHandle<String>,
    set_filter_text: StateSetter<String>,
}

impl CommandPalette {
    /// Register a new command
    pub fn register<F>(&self, name: &str, description: &str, action: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut commands = self.commands.get();
        commands.insert(
            name.to_string(),
            Command {
                name: name.to_string(),
                description: description.to_string(),
                action: Arc::new(action),
            },
        );
        self.set_commands.set(commands);
    }

    /// Execute a command by name
    pub fn execute(&self, name: &str) -> bool {
        if let Some(command) = self.commands.get().get(name) {
            (command.action)();
            true
        } else {
            false
        }
    }

    /// Show the command palette
    pub fn show_palette(&self) {
        self.set_is_visible.set(true);
        self.set_selected_index.set(0);
        self.set_filter_text.set(String::new());
    }

    /// Hide the command palette
    pub fn hide_palette(&self) {
        self.set_is_visible.set(false);
    }

    /// Check if the command palette is visible
    pub fn is_palette_visible(&self) -> bool {
        self.is_visible.get()
    }

    /// Get the filtered list of commands
    pub fn get_commands(&self) -> Vec<Command> {
        let filter = self.filter_text.get().to_lowercase();
        self.commands
            .get()
            .values()
            .filter(|cmd| {
                cmd.name.to_lowercase().contains(&filter)
                    || cmd.description.to_lowercase().contains(&filter)
            })
            .cloned() // Clone the commands to create owned values
            .collect()
    }

    /// Update the filter text
    pub fn set_filter(&self, text: String) {
        self.set_filter_text.set(text);
        self.set_selected_index.set(0);
    }

    /// Get the current filter text
    pub fn get_filter(&self) -> String {
        self.filter_text.get()
    }

    /// Get the currently selected command index
    pub fn get_selected_index(&self) -> usize {
        self.selected_index.get()
    }

    /// Set the selected command index
    pub fn set_selected_index(&self, index: usize) {
        self.set_selected_index.set(index);
    }
}

/// Hook for managing keyboard commands and command palette
pub fn use_command_palette() -> CommandPalette {
    let (commands, set_commands) = use_state(HashMap::new);
    let (is_visible, set_is_visible) = use_state(|| false);
    let (selected_index, set_selected_index) = use_state(|| 0);
    let (filter_text, set_filter_text) = use_state(String::new);

    CommandPalette {
        commands,
        set_commands,
        is_visible,
        set_is_visible,
        selected_index,
        set_selected_index,
        filter_text,
        set_filter_text,
    }
}
