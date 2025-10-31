use crate::theme::Theme;
use rand::Rng;
use reratui::prelude::*;
use std::time::Duration;

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
        let (cpu_usage, set_cpu_usage) = use_state(|| 30.0f32); // Start with reasonable values
        let (memory_used, set_memory_used) = use_state(|| 4.0f32); // GB
        let (memory_total, set_memory_total) = use_state(|| 16.0f32); // GB

        // CPU usage history for smoothing
        let (cpu_history, set_cpu_history) = use_state(|| vec![30.0f32; 5]);

        use_interval(
            {
                // Set up system info update interval
                let cpu_usage = cpu_usage.clone();
                let memory_used = memory_used.clone();
                let set_cpu_usage = set_cpu_usage.clone();
                let set_memory_used = set_memory_used.clone();
                let set_memory_total = set_memory_total.clone();

                move || {
                    // Only update system info every 5 seconds to reduce CPU usage
                    static mut COUNTER: u8 = 0;
                    unsafe {
                        COUNTER = (COUNTER + 1) % 5;
                        if COUNTER == 0 {
                            // Generate simulated CPU usage that varies realistically
                            let mut rng = rand::rng();
                            let current_cpu =
                                (cpu_usage.get() + rng.random_range(-5.0..5.0)).clamp(5.0, 95.0);

                            // Update CPU history for smoothing
                            let mut history = cpu_history.get();
                            history.remove(0); // Remove oldest value
                            history.push(current_cpu); // Add new value
                            set_cpu_history.set(history.clone());

                            // Calculate average CPU usage for smoother display
                            let avg_cpu = history.iter().sum::<f32>() / history.len() as f32;

                            // Simulate memory usage changes (more stable than CPU)
                            let memory_total = 16.0; // 16 GB total memory
                            let memory_used = (memory_used.get() + rng.random_range(-0.2..0.2))
                                .clamp(2.0, memory_total - 1.0);

                            // Update system information state
                            set_cpu_usage.set(avg_cpu);
                            set_memory_used.set(memory_used);
                            set_memory_total.set(memory_total);
                        }
                    }
                }
            },
            Duration::from_secs(1), // Update every second
        );

        // Create system information display
        let system_info = create_system_info(
            cpu_usage.get(),
            memory_used.get(),
            memory_total.get(),
            &self.theme,
        );

        // Add the spans to the output vector
        spans.extend(system_info.spans);
    }
}

impl Component for SystemInfoComponent {
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
