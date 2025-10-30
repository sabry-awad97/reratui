/// Helper function to get current mode based on counter
pub fn get_mode(counter: i32) -> &'static str {
    match counter {
        0..=3 => "startup",
        4..=7 => "normal",
        8..=12 => "active",
        13..=20 => "turbo",
        _ => "extreme",
    }
}

/// Helper function to get status description
pub fn get_status_description(mode: &str, counter: i32) -> String {
    match (mode, counter % 3) {
        ("startup", 0) => "🌱 Starting up - Ready to grow!".to_string(),
        ("startup", _) => "🌱 Starting up - Building momentum...".to_string(),
        ("normal", 0) => "⚡ Normal mode - Perfectly balanced!".to_string(),
        ("normal", _) => "⚡ Normal mode - Steady progress...".to_string(),
        ("active", 0) => "🚀 Active mode - Peak performance!".to_string(),
        ("active", _) => "🚀 Active mode - High energy!".to_string(),
        ("turbo", 0) => "🔥 Turbo mode - Maximum efficiency!".to_string(),
        ("turbo", _) => "🔥 Turbo mode - Blazing fast!".to_string(),
        ("extreme", _) => "💥 Extreme mode - Beyond limits!".to_string(),
        _ => "🤔 Unknown state".to_string(),
    }
}

/// Helper function to check if a number is prime
pub fn is_prime(n: i32) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    for i in (3..=(n as f64).sqrt() as i32).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

/// Get theme name based on theme mode
pub fn get_theme_name(theme_mode: usize) -> &'static str {
    match theme_mode {
        0 => "Light",
        1 => "Dark",
        _ => "Colorful",
    }
}

/// Get tab titles for the demo
pub fn get_tab_titles() -> Vec<&'static str> {
    vec![
        "🏠 Overview",
        "🎯 Match Expressions",
        "⚡ Logical AND",
        "🔀 If-Else",
        "🎨 Mixed Conditionals",
        "🏗️ Nested Layouts",
        "📚 Help",
    ]
}

/// Demo state structure to hold all the demo state
#[derive(Clone)]
pub struct DemoState {
    pub counter: i32,
    pub show_debug: bool,
    pub theme_mode: usize,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            counter: 0,
            show_debug: true,
            theme_mode: 0,
        }
    }

    pub fn get_current_mode(&self) -> &'static str {
        get_mode(self.counter)
    }

    pub fn get_status_description(&self) -> String {
        get_status_description(self.get_current_mode(), self.counter)
    }

    pub fn get_theme_name(&self) -> &'static str {
        get_theme_name(self.theme_mode)
    }
}

impl Default for DemoState {
    fn default() -> Self {
        Self::new()
    }
}

/// Control instructions for the demo
pub fn get_control_instructions() -> &'static str {
    "j/k: Counter ±1 | Tab: Next section | d: Toggle debug | t: Change theme | r: Reset"
}

/// Get features list for overview
pub fn get_features_list() -> &'static str {
    "🚀 Features Demonstrated:\n✅ Match expressions\n✅ Logical AND (&&)\n✅ If-else conditionals\n✅ Else if chains\n✅ Nested layouts\n✅ Dynamic content\n✅ Function calls in conditions"
}
