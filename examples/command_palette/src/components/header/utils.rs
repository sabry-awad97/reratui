use reratui::prelude::Color;

/// Interpolate between two colors based on a factor (0.0 to 1.0)
pub fn interpolate_color(color1: Color, color2: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0); // Clamp factor between 0.0 and 1.0

    match (color1, color2) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let r = interpolate_component(r1, r2, factor);
            let g = interpolate_component(g1, g2, factor);
            let b = interpolate_component(b1, b2, factor);
            Color::Rgb(r, g, b)
        }
        // For other color types, just return color1 for factor < 0.5, color2 otherwise
        _ => {
            if factor < 0.5 {
                color1
            } else {
                color2
            }
        }
    }
}

/// Interpolate between two color components
fn interpolate_component(c1: u8, c2: u8, factor: f32) -> u8 {
    let c1_f = f32::from(c1);
    let c2_f = f32::from(c2);
    let result = c1_f + (c2_f - c1_f) * factor;
    result.round() as u8
}
