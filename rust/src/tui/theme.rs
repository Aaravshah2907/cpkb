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

pub fn get_theme_by_name(name: &str) -> Theme {
    match name.to_lowercase().trim() {
        "catppuccin" | "mocha" | "catppuccin-mocha" => THEME_CATPPUCCIN,
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
