//! Color palettes and visual themes for CPKB TUI.

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub border_focused: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub text: Color,
    pub text_dim: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Theme {
    pub fn style_base(&self) -> Style {
        Style::default().bg(self.background).fg(self.text)
    }

    pub fn style_surface(&self) -> Style {
        Style::default().bg(self.surface).fg(self.text)
    }

    pub fn style_border(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.border_focused).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.border)
        }
    }

    pub fn style_selected(&self) -> Style {
        Style::default()
            .bg(self.surface)
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn style_header(&self) -> Style {
        Style::default()
            .bg(self.surface)
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn style_footer(&self) -> Style {
        Style::default()
            .bg(self.background)
            .fg(self.text_dim)
    }
}

pub const THEME_COSMERE: Theme = Theme {
    name: "Cosmere",
    background: Color::Rgb(15, 23, 42),       // Slate 900
    surface: Color::Rgb(30, 41, 59),          // Slate 800
    border: Color::Rgb(51, 65, 85),           // Slate 700
    border_focused: Color::Rgb(56, 189, 248), // Sky 400 (Honor Blue)
    primary: Color::Rgb(56, 189, 248),        // Sky 400
    secondary: Color::Rgb(245, 158, 11),      // Amber 500 (Stormlight Gold)
    accent: Color::Rgb(168, 85, 247),         // Purple 500 (Voidbringer Violet)
    text: Color::Rgb(248, 250, 252),          // Slate 50
    text_dim: Color::Rgb(148, 163, 184),      // Slate 400
    success: Color::Rgb(16, 185, 129),        // Emerald 500 (Edgedancer)
    warning: Color::Rgb(245, 158, 11),        // Amber 500
    error: Color::Rgb(239, 68, 68),           // Red 500
};

pub const THEME_CATPPUCCIN: Theme = Theme {
    name: "Catppuccin Mocha",
    background: Color::Rgb(30, 30, 46),       // Base
    surface: Color::Rgb(49, 50, 68),          // Surface 0
    border: Color::Rgb(69, 71, 90),           // Surface 1
    border_focused: Color::Rgb(137, 180, 250),// Blue
    primary: Color::Rgb(137, 180, 250),       // Blue
    secondary: Color::Rgb(249, 226, 175),     // Yellow
    accent: Color::Rgb(203, 166, 247),        // Mauve
    text: Color::Rgb(205, 214, 244),          // Text
    text_dim: Color::Rgb(166, 173, 200),      // Subtext 0
    success: Color::Rgb(166, 227, 161),       // Green
    warning: Color::Rgb(250, 179, 135),       // Peach
    error: Color::Rgb(243, 139, 168),         // Red
};

pub const THEME_DRACULA: Theme = Theme {
    name: "Dracula",
    background: Color::Rgb(40, 42, 54),
    surface: Color::Rgb(68, 71, 90),
    border: Color::Rgb(98, 114, 164),
    border_focused: Color::Rgb(189, 147, 249),
    primary: Color::Rgb(189, 147, 249),
    secondary: Color::Rgb(241, 250, 140),
    accent: Color::Rgb(255, 121, 198),
    text: Color::Rgb(248, 248, 242),
    text_dim: Color::Rgb(98, 114, 164),
    success: Color::Rgb(80, 250, 123),
    warning: Color::Rgb(255, 184, 108),
    error: Color::Rgb(255, 85, 85),
};

pub const THEME_TOKYO_NIGHT: Theme = Theme {
    name: "Tokyo Night",
    background: Color::Rgb(26, 27, 38),
    surface: Color::Rgb(36, 40, 59),
    border: Color::Rgb(65, 72, 104),
    border_focused: Color::Rgb(122, 162, 247),
    primary: Color::Rgb(122, 162, 247),
    secondary: Color::Rgb(224, 175, 104),
    accent: Color::Rgb(187, 154, 247),
    text: Color::Rgb(192, 202, 245),
    text_dim: Color::Rgb(86, 95, 137),
    success: Color::Rgb(158, 206, 106),
    warning: Color::Rgb(224, 175, 104),
    error: Color::Rgb(247, 118, 142),
};

pub const THEME_NORD: Theme = Theme {
    name: "Nord",
    background: Color::Rgb(46, 52, 64),
    surface: Color::Rgb(59, 66, 82),
    border: Color::Rgb(76, 86, 106),
    border_focused: Color::Rgb(136, 192, 208),
    primary: Color::Rgb(136, 192, 208),
    secondary: Color::Rgb(235, 203, 139),
    accent: Color::Rgb(180, 142, 173),
    text: Color::Rgb(236, 239, 244),
    text_dim: Color::Rgb(147, 158, 179),
    success: Color::Rgb(163, 190, 140),
    warning: Color::Rgb(235, 203, 139),
    error: Color::Rgb(191, 97, 106),
};

pub const THEME_GRUVBOX: Theme = Theme {
    name: "Gruvbox",
    background: Color::Rgb(40, 40, 40),
    surface: Color::Rgb(60, 56, 54),
    border: Color::Rgb(102, 92, 84),
    border_focused: Color::Rgb(250, 189, 47),
    primary: Color::Rgb(250, 189, 47),
    secondary: Color::Rgb(254, 128, 25),
    accent: Color::Rgb(211, 134, 155),
    text: Color::Rgb(235, 219, 178),
    text_dim: Color::Rgb(168, 153, 132),
    success: Color::Rgb(184, 187, 38),
    warning: Color::Rgb(250, 189, 47),
    error: Color::Rgb(251, 73, 52),
};

pub const THEMES: &[(&str, Theme)] = &[
    ("Cosmere", THEME_COSMERE),
    ("Catppuccin Mocha", THEME_CATPPUCCIN),
    ("Tokyo Night", THEME_TOKYO_NIGHT),
    ("Nord", THEME_NORD),
    ("Gruvbox", THEME_GRUVBOX),
    ("Dracula", THEME_DRACULA),
];

pub fn get_theme_by_name(name: &str) -> Theme {
    match name.to_lowercase().trim() {
        "catppuccin" | "mocha" | "catppuccin-mocha" => THEME_CATPPUCCIN,
        "tokyo-night" | "tokyonight" => THEME_TOKYO_NIGHT,
        "nord" => THEME_NORD,
        "gruvbox" => THEME_GRUVBOX,
        "dracula" => THEME_DRACULA,
        _ => THEME_COSMERE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_resolution() {
        assert_eq!(get_theme_by_name("catppuccin").name, "Catppuccin Mocha");
        assert_eq!(get_theme_by_name("dracula").name, "Dracula");
        assert_eq!(get_theme_by_name("cosmere").name, "Cosmere");
        assert_eq!(get_theme_by_name("default").name, "Cosmere");
    }

    #[test]
    fn test_theme_styles() {
        let theme = THEME_COSMERE;
        assert_eq!(theme.style_base().bg, Some(theme.background));
        assert_eq!(theme.style_surface().bg, Some(theme.surface));
    }
}
