use crate::theme::Theme;
use rand::Rng;
use reratui_fiber::prelude::*;

pub struct SystemInfoComponent {
    theme: Theme,
}

impl SystemInfoComponent {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Get the spans for the system info without rendering to a frame
    pub fn get_spans<'a>(&'a self, spans: &mut Vec<Span<'a>>) {
        // Simulated system information state
        let (cpu_usage, set_cpu_usage) = use_state_v2(|| 30.0f32); // Start with reasonable values
        let (memory_used, set_memory_used) = use_state_v2(|| 4.0f32); // GB
        let (memory_total, _set_memory_total) = use_state_v2(|| 16.0f32); // GB

        use_interval_v2(
            {
                // Set up system info update interval
                move || {
                    // Update CPU usage with simulated variation
                    set_cpu_usage.update(|current| {
                        let mut rng = rand::rng();
                        let change: f32 = rng.random_range(-5.0..5.0);
                        (*current + change).clamp(5.0, 95.0)
                    });

                    // Simulate memory usage changes (more stable than CPU)
                    set_memory_used.update(|current| {
                        let mut rng = rand::rng();
                        let change: f32 = rng.random_range(-0.2..0.2);
                        (*current + change).clamp(2.0, 15.0)
                    });
                }
            },
            5000, // Update every 5 seconds to reduce CPU usage
        );

        // Create system information display
        let system_info = create_system_info(cpu_usage, memory_used, memory_total, &self.theme);

        // Add the spans to the output vector
        spans.extend(system_info.spans);
    }
}

impl ComponentV2 for SystemInfoComponent {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let mut spans = Vec::new();
        self.get_spans(&mut spans);

        let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);

        paragraph.render(area, buffer);
    }
}

/// Create simulated system information display
fn create_system_info(
    cpu_usage: f32,
    memory_used: f32,
    memory_total: f32,
    theme: &'_ Theme,
) -> Line<'_> {
    // Calculate memory percentage
    let memory_percentage = if memory_total > 0.0 {
        (memory_used / memory_total * 100.0).round() as u64
    } else {
        0
    };

    // Get CPU usage as integer with clamping to ensure it's never above 100%
    let global_cpu_usage = (cpu_usage.min(100.0).round() as u64).min(100);

    // Create memory usage string with appropriate color
    let memory_style = if memory_percentage < 50 {
        Style::default().fg(theme.success)
    } else if memory_percentage < 80 {
        Style::default().fg(theme.warning)
    } else {
        Style::default()
            .fg(theme.error)
            .add_modifier(Modifier::BOLD)
    };

    // Create CPU usage string with appropriate color
    let cpu_style = if global_cpu_usage < 50 {
        Style::default().fg(theme.success)
    } else if global_cpu_usage < 80 {
        Style::default().fg(theme.warning)
    } else {
        Style::default()
            .fg(theme.error)
            .add_modifier(Modifier::BOLD)
    };

    // Memory is already in GB in our simulation
    let used_gb = memory_used;
    let total_gb = memory_total;

    Line::from(vec![
        Span::styled(format!("CPU: {}%", global_cpu_usage), cpu_style),
        Span::raw(" | "),
        Span::styled(
            format!(
                "MEM: {:.1}/{:.1} GB ({}%)",
                used_gb, total_gb, memory_percentage
            ),
            memory_style,
        ),
    ])
}
