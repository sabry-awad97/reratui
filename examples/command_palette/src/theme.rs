use reratui::prelude::*;

/// Modern theme system with carefully crafted color palettes
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    // Core colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub surface: Color,
    pub foreground: Color,
    pub muted: Color,
    pub border: Color,

    // Semantic colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // Interactive states
    pub hover: Color,
    pub active: Color,
    pub inactive: Color,

    // Text hierarchy
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_inverse: Color,

    // Modifiers and effects
    pub title_modifier: Modifier,
    pub highlight_modifier: Modifier,
    pub shadow_modifier: Modifier,
}

impl Theme {
    pub fn github_dark() -> Self {
        Self {
            name: "GitHub Dark".to_string(),
            is_dark: true,
            primary: Color::Rgb(88, 166, 255),
            secondary: Color::Rgb(139, 148, 158),
            accent: Color::Rgb(246, 146, 255),
            background: Color::Rgb(13, 17, 23),
            surface: Color::Rgb(22, 27, 34),
            foreground: Color::Rgb(230, 237, 243),
            muted: Color::Rgb(88, 96, 105),
            border: Color::Rgb(48, 54, 61),

            success: Color::Rgb(46, 160, 67),
            warning: Color::Rgb(187, 128, 9),
            error: Color::Rgb(248, 81, 73),
            info: Color::Rgb(88, 166, 255),

            hover: Color::Rgb(33, 38, 45),
            active: Color::Rgb(88, 166, 255),
            inactive: Color::Rgb(88, 96, 105),

            text_primary: Color::Rgb(230, 237, 243),
            text_secondary: Color::Rgb(139, 148, 158),
            text_tertiary: Color::Rgb(88, 96, 105),
            text_inverse: Color::Rgb(13, 17, 23),

            title_modifier: Modifier::BOLD,
            highlight_modifier: Modifier::REVERSED,
            shadow_modifier: Modifier::DIM,
        }
    }

    pub fn github_light() -> Self {
        Self {
            name: "GitHub Light".to_string(),
            is_dark: false,
            primary: Color::Rgb(0, 92, 197),
            secondary: Color::Rgb(87, 96, 106),
            accent: Color::Rgb(188, 28, 255),
            background: Color::Rgb(255, 255, 255),
            surface: Color::Rgb(246, 248, 250),
            foreground: Color::Rgb(36, 41, 47),
            muted: Color::Rgb(87, 96, 106),
            border: Color::Rgb(208, 215, 222),

            success: Color::Rgb(40, 167, 69),
            warning: Color::Rgb(219, 171, 9),
            error: Color::Rgb(215, 58, 73),
            info: Color::Rgb(0, 92, 197),

            hover: Color::Rgb(246, 248, 250),
            active: Color::Rgb(0, 92, 197),
            inactive: Color::Rgb(149, 157, 165),

            text_primary: Color::Rgb(36, 41, 47),
            text_secondary: Color::Rgb(87, 96, 106),
            text_tertiary: Color::Rgb(149, 157, 165),
            text_inverse: Color::Rgb(255, 255, 255),

            title_modifier: Modifier::BOLD,
            highlight_modifier: Modifier::REVERSED,
            shadow_modifier: Modifier::DIM,
        }
    }

    pub fn nord_dark() -> Self {
        Self {
            name: "Nord Dark".to_string(),
            is_dark: true,
            primary: Color::Rgb(136, 192, 208),
            secondary: Color::Rgb(129, 161, 193),
            accent: Color::Rgb(180, 142, 173),
            background: Color::Rgb(46, 52, 64),
            surface: Color::Rgb(59, 66, 82),
            foreground: Color::Rgb(229, 233, 240),
            muted: Color::Rgb(94, 129, 172),
            border: Color::Rgb(76, 86, 106),

            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            info: Color::Rgb(136, 192, 208),

            hover: Color::Rgb(67, 76, 94),
            active: Color::Rgb(136, 192, 208),
            inactive: Color::Rgb(76, 86, 106),

            text_primary: Color::Rgb(229, 233, 240),
            text_secondary: Color::Rgb(216, 222, 233),
            text_tertiary: Color::Rgb(129, 161, 193),
            text_inverse: Color::Rgb(46, 52, 64),

            title_modifier: Modifier::BOLD,
            highlight_modifier: Modifier::REVERSED,
            shadow_modifier: Modifier::DIM,
        }
    }

    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text_primary).bg(self.background)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted).bg(self.background)
    }

    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent).bg(self.background)
    }

    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info).bg(self.background)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success).bg(self.background)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning).bg(self.background)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error).bg(self.background)
    }
}
