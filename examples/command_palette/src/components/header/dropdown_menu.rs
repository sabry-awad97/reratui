use crate::theme::Theme;
use reratui::prelude::*;

/// Represents a menu item in a dropdown menu
#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: String,
    pub action: MenuAction,
}

/// Actions that can be performed when a menu item is selected
#[derive(Clone)]
pub enum MenuAction {
    /// Custom action with a unique identifier
    Custom(String),
    /// Open a submenu
    OpenSubmenu(String),
    /// Close the current menu
    CloseMenu,
    /// Exit the application
    Exit,
}

/// A dropdown menu component
#[derive(Clone)]
pub struct DropdownMenu {
    pub title: String,
    pub items: Vec<MenuItem>,
    pub theme: Theme,
    pub shortcut: KeyCode,
    pub modifier: KeyModifiers,
    pub parent: Option<String>, // Parent menu ID if this is a submenu
}

impl DropdownMenu {
    pub fn new(
        title: &str,
        items: Vec<MenuItem>,
        theme: Theme,
        shortcut: KeyCode,
        modifier: KeyModifiers,
    ) -> Self {
        Self {
            title: title.to_string(),
            items,
            theme,
            shortcut,
            modifier,
            parent: None, // Default to no parent
        }
    }

    /// Create a new submenu with a parent
    pub fn new_submenu(title: &str, items: Vec<MenuItem>, theme: Theme, parent: String) -> Self {
        Self {
            title: title.to_string(),
            items,
            theme,
            shortcut: KeyCode::Null, // Submenus don't have keyboard shortcuts
            modifier: KeyModifiers::NONE,
            parent: Some(parent),
        }
    }

    /// Check if the given key event matches this menu's shortcut
    pub fn matches_shortcut(&self, key: &KeyEvent) -> bool {
        key.code == self.shortcut && key.modifiers == self.modifier
    }
}

// Shared state for menu actions and active submenus
thread_local! {
    static LAST_MENU_ACTION: std::cell::RefCell<Option<MenuAction>> = const { std::cell::RefCell::new(None) };
    static ACTIVE_SUBMENU: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Get the last menu action that was triggered
pub fn get_last_menu_action() -> Option<MenuAction> {
    let mut result = None;
    LAST_MENU_ACTION.with(|cell| {
        if let Some(action) = cell.borrow().clone() {
            result = Some(action);
            // Clear the action after retrieving it
            *cell.borrow_mut() = None;
        }
    });
    result
}

/// Set the last menu action
fn set_last_menu_action(action: MenuAction) {
    LAST_MENU_ACTION.with(|cell| {
        *cell.borrow_mut() = Some(action);
    });
}

/// Get the currently active submenu
pub fn get_active_submenu() -> Option<String> {
    let mut result = None;
    ACTIVE_SUBMENU.with(|cell| {
        if let Some(submenu) = cell.borrow().clone() {
            result = Some(submenu);
        }
    });
    result
}

/// Set the active submenu
pub fn set_active_submenu(submenu: Option<String>) {
    ACTIVE_SUBMENU.with(|cell| {
        *cell.borrow_mut() = submenu;
    });
}

impl DropdownMenu {
    /// Render just the dropdown part (for z-index control)
    pub fn render_dropdown(&self, area: Rect, buffer: &mut Buffer) {
        // State for the currently selected item
        let (selected_index, _set_selected_index) = use_state(|| 0);

        // Check if this menu should be open based on active submenu
        let active_submenu = get_active_submenu();
        let should_be_open = match &self.parent {
            // This is a submenu, open if it matches the active submenu
            Some(parent) => active_submenu.as_ref() == Some(parent),
            // This is a main menu, open if it's the active one
            None => active_submenu.as_ref() == Some(&self.title),
        };

        // Only render the dropdown if it should be open
        if should_be_open {
            // Calculate the dropdown area
            let dropdown_height = self.items.len() as u16 + 2; // +2 for borders
            let dropdown_width = 25; // Narrower width

            let dropdown_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: dropdown_width,
                height: dropdown_height,
            };

            // Create a clear widget to ensure the dropdown is visible
            Clear.render(dropdown_area, buffer);

            // Create list items from menu items
            let items: Vec<ListItem> = self
                .items
                .iter()
                .map(|item| {
                    let mut spans = vec![
                        Span::styled(item.label.clone(), Style::default().fg(self.theme.primary)),
                        Span::raw(" "),
                    ];

                    // Add submenu indicator or shortcut based on action type
                    match &item.action {
                        MenuAction::OpenSubmenu(_) => {
                            spans.push(Span::styled(
                                "►", // Submenu indicator
                                Style::default().fg(self.theme.accent),
                            ));
                        }
                        _ => {
                            spans.push(Span::styled(
                                format!("({})", item.shortcut),
                                Style::default().fg(self.theme.muted),
                            ));
                        }
                    }

                    ListItem::new(Line::from(spans))
                })
                .collect();

            // Create a list state with the selected item
            let mut list_state = ListState::default();
            list_state.select(Some(selected_index.get()));

            // Create the list widget
            let list = List::new(items)
                .block(
                    Block::default()
                        .title(self.title.clone())
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(self.theme.border)),
                )
                .highlight_style(
                    Style::default()
                        .bg(self.theme.accent)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            let mut frame_ctx = use_frame();
            let frame = frame_ctx.frame_mut();
            // Render the stateful list
            frame.render_stateful_widget(list, dropdown_area, &mut list_state);
        }
    }
}

impl Component for DropdownMenu {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // State for whether the menu is open
        let (is_open, set_is_open) = use_state(|| false);

        // State for the currently selected item
        let (selected_index, set_selected_index) = use_state(|| 0);

        // Check if this menu should be open based on active submenu
        let active_submenu = get_active_submenu();
        let should_be_open = match &self.parent {
            // This is a submenu, open if it matches the active submenu
            Some(parent) => active_submenu.as_ref() == Some(parent),
            // This is a main menu, open if no submenu is active or if it's already open
            None => is_open.get() || active_submenu.is_none(),
        };

        // Handle keyboard events
        if let Some(Event::Key(key)) = use_event()
            && key.kind == KeyEventKind::Press
        {
            // Toggle menu open/closed with shortcut (only for main menus)
            if self.parent.is_none() && self.matches_shortcut(&key) {
                if is_open.get() {
                    set_is_open.set(false);
                    set_active_submenu(None); // Close any active submenu
                } else {
                    set_is_open.set(true);
                    // Close other menus by setting this as the active one
                    set_active_submenu(Some(self.title.clone()));
                }
            }

            // If menu is open or should be open (submenu), handle navigation and selection
            if is_open.get() || should_be_open {
                match key.code {
                    KeyCode::Esc => {
                        set_is_open.set(false);
                    }
                    KeyCode::Up => {
                        let new_index = if selected_index.get() > 0 {
                            selected_index.get() - 1
                        } else {
                            self.items.len() - 1
                        };
                        set_selected_index.set(new_index);
                    }
                    KeyCode::Down => {
                        let new_index = (selected_index.get() + 1) % self.items.len();
                        set_selected_index.set(new_index);
                    }
                    KeyCode::Enter => {
                        // Execute the selected item's action
                        if let Some(item) = self.items.get(selected_index.get()) {
                            match &item.action {
                                MenuAction::CloseMenu => {
                                    set_is_open.set(false);
                                    set_active_submenu(None);
                                }
                                MenuAction::OpenSubmenu(submenu_id) => {
                                    // Open the submenu
                                    set_active_submenu(Some(submenu_id.clone()));
                                }
                                _ => {
                                    // Store the action in thread-local storage
                                    set_last_menu_action(item.action.clone());
                                    set_is_open.set(false);
                                    set_active_submenu(None);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Render the menu title (always visible) - more compact
        let title_style = if is_open.get() {
            Style::default()
                .fg(self.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.secondary)
        };

        // Simplified shortcut display
        let title_text = self.title.clone();

        let title_widget = Paragraph::new(Line::from(vec![Span::styled(title_text, title_style)]))
            .alignment(Alignment::Left);

        title_widget.render(area, buffer);

        // If the menu is open, set it as the active menu
        if is_open.get() {
            set_active_submenu(Some(self.title.clone()));
        }
    }
}
