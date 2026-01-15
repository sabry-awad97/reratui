use reratui::prelude::*;

use crate::theme::Theme;

use super::dropdown_menu::{DropdownMenu, MenuAction, MenuItem};

pub struct MenuBar {
    pub theme: Theme,
}

impl MenuBar {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }
}

// Store active menus for later rendering
thread_local! {
    static ACTIVE_MENUS: std::cell::RefCell<Vec<(DropdownMenu, Rect)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Clear all active menus
pub fn clear_active_menus() {
    ACTIVE_MENUS.with(|cell| {
        cell.borrow_mut().clear();
    });
}

/// Get all active menus
pub fn get_active_menus() -> Vec<(DropdownMenu, Rect)> {
    let mut result = Vec::new();
    ACTIVE_MENUS.with(|cell| {
        // Clone each item individually
        for (menu, area) in cell.borrow().iter() {
            result.push((menu.clone(), *area));
        }
    });
    result
}

/// Add an active menu for later rendering
fn add_active_menu(menu: DropdownMenu, area: Rect) {
    ACTIVE_MENUS.with(|cell| {
        cell.borrow_mut().push((menu, area));
    });
}

/// Helper function to render just the menu title
fn render_menu_title(menu: &DropdownMenu, area: Rect, buffer: &mut Buffer) {
    let active_submenu = super::dropdown_menu::get_active_submenu();
    let is_active = active_submenu.as_ref() == Some(&menu.title);

    let title_style = if is_active {
        Style::default()
            .fg(menu.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(menu.theme.secondary)
    };

    let title_widget = Paragraph::new(Line::from(vec![Span::styled(
        menu.title.clone(),
        title_style,
    )]))
    .alignment(Alignment::Left);

    title_widget.render(area, buffer);
}

impl Component for MenuBar {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Clear any previously stored menus
        clear_active_menus();
        // No need for border effects in the menu bar

        // For the menu bar, we don't need a border to save space
        // Create the inner area for the menus - use the full area
        let inner_area = area;

        // Create a layout for the menus
        let menu_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(15), // File menu
                Constraint::Length(15), // Edit menu
                Constraint::Length(15), // View menu
                Constraint::Length(15), // Help menu
                Constraint::Min(0),     // Remaining space
            ])
            .split(inner_area);

        // Create the File menu
        let file_menu = DropdownMenu::new(
            "File",
            vec![
                MenuItem {
                    label: "New".to_string(),
                    shortcut: "N".to_string(),
                    action: MenuAction::Custom("new".to_string()),
                },
                MenuItem {
                    label: "Open".to_string(),
                    shortcut: "O".to_string(),
                    action: MenuAction::Custom("open".to_string()),
                },
                MenuItem {
                    label: "Save".to_string(),
                    shortcut: "S".to_string(),
                    action: MenuAction::Custom("save".to_string()),
                },
                MenuItem {
                    label: "Recent Files".to_string(),
                    shortcut: "R".to_string(),
                    action: MenuAction::OpenSubmenu("recent_files".to_string()),
                },
                MenuItem {
                    label: "Exit".to_string(),
                    shortcut: "Q".to_string(),
                    action: MenuAction::Exit,
                },
            ],
            self.theme.clone(),
            KeyCode::Char('f'),
            KeyModifiers::ALT,
        );

        // Create the Recent Files submenu
        let recent_files_submenu = DropdownMenu::new_submenu(
            "Recent Files",
            vec![
                MenuItem {
                    label: "Document1.txt".to_string(),
                    shortcut: "1".to_string(),
                    action: MenuAction::Custom("open_recent_1".to_string()),
                },
                MenuItem {
                    label: "Project.rs".to_string(),
                    shortcut: "2".to_string(),
                    action: MenuAction::Custom("open_recent_2".to_string()),
                },
                MenuItem {
                    label: "Notes.md".to_string(),
                    shortcut: "3".to_string(),
                    action: MenuAction::Custom("open_recent_3".to_string()),
                },
                MenuItem {
                    label: "Clear Recent Files".to_string(),
                    shortcut: "C".to_string(),
                    action: MenuAction::Custom("clear_recent".to_string()),
                },
                MenuItem {
                    label: "Back".to_string(),
                    shortcut: "Esc".to_string(),
                    action: MenuAction::CloseMenu,
                },
            ],
            self.theme.clone(),
            "recent_files".to_string(),
        );

        // Create the Edit menu
        let edit_menu = DropdownMenu::new(
            "Edit",
            vec![
                MenuItem {
                    label: "Cut".to_string(),
                    shortcut: "X".to_string(),
                    action: MenuAction::Custom("cut".to_string()),
                },
                MenuItem {
                    label: "Copy".to_string(),
                    shortcut: "C".to_string(),
                    action: MenuAction::Custom("copy".to_string()),
                },
                MenuItem {
                    label: "Paste".to_string(),
                    shortcut: "V".to_string(),
                    action: MenuAction::Custom("paste".to_string()),
                },
            ],
            self.theme.clone(),
            KeyCode::Char('e'),
            KeyModifiers::ALT,
        );

        // Create the View menu
        let view_menu = DropdownMenu::new(
            "View",
            vec![
                MenuItem {
                    label: "Zoom In".to_string(),
                    shortcut: "+".to_string(),
                    action: MenuAction::Custom("zoom_in".to_string()),
                },
                MenuItem {
                    label: "Zoom Out".to_string(),
                    shortcut: "-".to_string(),
                    action: MenuAction::Custom("zoom_out".to_string()),
                },
                MenuItem {
                    label: "Toggle Sidebar".to_string(),
                    shortcut: "B".to_string(),
                    action: MenuAction::Custom("toggle_sidebar".to_string()),
                },
            ],
            self.theme.clone(),
            KeyCode::Char('v'),
            KeyModifiers::ALT,
        );

        // Create the Help menu
        let help_menu = DropdownMenu::new(
            "Help",
            vec![
                MenuItem {
                    label: "Documentation".to_string(),
                    shortcut: "D".to_string(),
                    action: MenuAction::Custom("documentation".to_string()),
                },
                MenuItem {
                    label: "About".to_string(),
                    shortcut: "A".to_string(),
                    action: MenuAction::Custom("about".to_string()),
                },
            ],
            self.theme.clone(),
            KeyCode::Char('h'),
            KeyModifiers::ALT,
        );

        // Store menus for later rendering (just render the titles now)
        add_active_menu(file_menu.clone(), menu_layout[0]);
        add_active_menu(edit_menu.clone(), menu_layout[1]);
        add_active_menu(view_menu.clone(), menu_layout[2]);
        add_active_menu(help_menu.clone(), menu_layout[3]);

        // Render just the menu titles
        render_menu_title(&file_menu, menu_layout[0], buffer);
        render_menu_title(&edit_menu, menu_layout[1], buffer);
        render_menu_title(&view_menu, menu_layout[2], buffer);
        render_menu_title(&help_menu, menu_layout[3], buffer);

        // Store submenu for later rendering if active
        let active_submenu = super::dropdown_menu::get_active_submenu();
        if let Some(submenu_id) = active_submenu
            && submenu_id == "recent_files"
        {
            // Position the submenu next to the parent menu
            let submenu_area = Rect {
                x: menu_layout[0].x + 15, // Position to the right of the File menu
                y: menu_layout[0].y + 4,  // Position below the "Recent Files" item
                width: 30,                // Width of the submenu
                height: 10,               // Height of the submenu
            };
            add_active_menu(recent_files_submenu.clone(), submenu_area);
        }
    }
}
