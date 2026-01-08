use reratui_fiber::prelude::*;
use reratui_fiber::ratatui::widgets::BorderType;
use std::sync::{Arc, LazyLock, Mutex};

use crate::theme::Theme;

// Global debug log storage
static DEBUG_LOGS: LazyLock<Arc<Mutex<Vec<String>>>> = LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

/// Add a debug log message
pub fn debug_log(msg: impl Into<String>) {
    if let Ok(mut logs) = DEBUG_LOGS.lock() {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        logs.push(format!("[{}] {}", timestamp, msg.into()));
        // Keep only last 50 logs
        if logs.len() > 50 {
            logs.remove(0);
        }
    }
}

/// Get all debug logs
pub fn get_debug_logs() -> Vec<String> {
    DEBUG_LOGS
        .lock()
        .map(|logs| logs.clone())
        .unwrap_or_default()
}

/// Clear debug logs
pub fn clear_debug_logs() {
    if let Ok(mut logs) = DEBUG_LOGS.lock() {
        logs.clear();
    }
}

pub struct DebugPanel {
    pub theme: Theme,
}

impl ComponentV2 for DebugPanel {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        // Force refresh by tracking render count
        let (render_count, set_render_count) = use_state_v2(|| 0usize);

        // Log each render
        debug_log(format!("DebugPanel render #{}", render_count));

        // Set up interval to force re-render and check for new logs
        use_interval_v2(
            move || {
                debug_log("Interval tick!");
                set_render_count.update(|c| *c + 1);
            },
            1000, // Every second
        );

        let logs = get_debug_logs();

        // Create log lines
        let log_lines: Vec<Line> = logs
            .iter()
            .rev() // Show newest first
            .take(area.height.saturating_sub(2) as usize) // Fit in area
            .map(|log| {
                Line::from(Span::styled(
                    log.clone(),
                    Style::default().fg(self.theme.muted),
                ))
            })
            .collect();

        let paragraph = Paragraph::new(log_lines)
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" 🐛 Debug Logs (renders: {}) ", render_count),
                        Style::default()
                            .fg(self.theme.warning)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(self.theme.warning)),
            )
            .style(Style::default().bg(self.theme.background));

        paragraph.render(area, buffer);
    }
}
