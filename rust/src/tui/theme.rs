//! Color palettes and visual themes for CPKB TUI.

use ratatui::style::{Color, Modifier, Style};
use crate::config::CustomTheme;

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

/// Parse a hex code (#rrggbb or #rgb) or named CSS color into a ratatui Color.
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some(Color::Rgb(r, g, b));
        }
    }
    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" | "purple" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" | "dark_gray" => Some(Color::DarkGray),
        "lightred" | "light_red" => Some(Color::LightRed),
        "lightgreen" | "light_green" => Some(Color::LightGreen),
        "lightyellow" | "light_yellow" => Some(Color::LightYellow),
        "lightblue" | "light_blue" => Some(Color::LightBlue),
        "lightmagenta" | "light_magenta" => Some(Color::LightMagenta),
        "lightcyan" | "light_cyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => None,
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

pub const THEME_SOLARIZED_DARK: Theme = Theme {
    name: "Solarized Dark",
    background: Color::Rgb(0, 43, 54),
    surface: Color::Rgb(7, 54, 66),
    border: Color::Rgb(88, 110, 117),
    border_focused: Color::Rgb(38, 139, 210),
    primary: Color::Rgb(38, 139, 210),      // Blue
    secondary: Color::Rgb(181, 137, 0),     // Yellow
    accent: Color::Rgb(211, 54, 130),       // Magenta
    text: Color::Rgb(131, 148, 150),
    text_dim: Color::Rgb(88, 110, 117),
    success: Color::Rgb(133, 153, 0),       // Green
    warning: Color::Rgb(203, 75, 22),       // Orange
    error: Color::Rgb(220, 50, 47),         // Red
};

pub const THEME_SYNTHWAVE: Theme = Theme {
    name: "Synthwave",
    background: Color::Rgb(36, 20, 51),
    surface: Color::Rgb(49, 27, 70),
    border: Color::Rgb(80, 44, 115),
    border_focused: Color::Rgb(255, 126, 219), // Neon Pink
    primary: Color::Rgb(255, 126, 219),
    secondary: Color::Rgb(54, 241, 205),       // Neon Cyan
    accent: Color::Rgb(254, 222, 93),          // Neon Yellow
    text: Color::Rgb(255, 255, 255),
    text_dim: Color::Rgb(160, 140, 180),
    success: Color::Rgb(54, 241, 205),
    warning: Color::Rgb(254, 222, 93),
    error: Color::Rgb(254, 68, 80),
};

pub fn from_custom(custom: &CustomTheme) -> Theme {
    let background = parse_color(&custom.background).unwrap_or(Color::Rgb(30, 30, 30));
    let surface = parse_color(&custom.surface).unwrap_or(Color::Rgb(45, 45, 48));
    let primary = parse_color(&custom.primary).unwrap_or(Color::Rgb(0, 255, 255));
    let secondary = parse_color(&custom.secondary).unwrap_or(Color::Rgb(51, 153, 255));
    let accent = parse_color(&custom.accent).unwrap_or(Color::Rgb(0, 255, 255));
    let text = parse_color(&custom.foreground).unwrap_or(Color::Rgb(255, 255, 255));
    let warning = parse_color(&custom.warning).unwrap_or(Color::Rgb(250, 189, 47));
    let error = parse_color(&custom.error).unwrap_or(Color::Rgb(255, 85, 85));
    let success = parse_color(&custom.success).unwrap_or(Color::Rgb(78, 191, 113));
    let border = parse_color(&custom.panel).unwrap_or(Color::Rgb(70, 70, 75));
    let border_focused = primary;
    let text_dim = parse_color(&custom.boost).unwrap_or(Color::Rgb(150, 150, 150));

    Theme {
        name: "Custom",
        background,
        surface,
        border,
        border_focused,
        primary,
        secondary,
        accent,
        text,
        text_dim,
        success,
        warning,
        error,
    }
}

pub const THEMES: &[(&str, Theme)] = &[
    ("Cosmere", THEME_COSMERE),
    ("Catppuccin Mocha", THEME_CATPPUCCIN),
    ("Tokyo Night", THEME_TOKYO_NIGHT),
    ("Nord", THEME_NORD),
    ("Gruvbox", THEME_GRUVBOX),
    ("Dracula", THEME_DRACULA),
    ("Solarized Dark", THEME_SOLARIZED_DARK),
    ("Synthwave", THEME_SYNTHWAVE),
];

pub fn get_available_themes(custom: Option<&CustomTheme>) -> Vec<(&'static str, Theme)> {
    let mut list = Vec::with_capacity(THEMES.len() + 1);
    for &(name, ref t) in THEMES {
        list.push((name, t.clone()));
    }
    if let Some(c) = custom {
        list.push(("Custom", from_custom(c)));
    } else {
        list.push(("Custom", from_custom(&CustomTheme::default())));
    }
    list
}

pub fn get_theme_by_name(name: &str) -> Theme {
    get_theme(name, None)
}

pub fn get_theme(name: &str, custom: Option<&CustomTheme>) -> Theme {
    match name.to_lowercase().trim() {
        "custom" => {
            if let Some(c) = custom {
                from_custom(c)
            } else {
                from_custom(&CustomTheme::default())
            }
        }
        "catppuccin" | "mocha" | "catppuccin-mocha" => THEME_CATPPUCCIN,
        "tokyo-night" | "tokyonight" => THEME_TOKYO_NIGHT,
        "nord" => THEME_NORD,
        "gruvbox" => THEME_GRUVBOX,
        "dracula" => THEME_DRACULA,
        "solarized" | "solarized-dark" => THEME_SOLARIZED_DARK,
        "synthwave" | "cyberpunk" => THEME_SYNTHWAVE,
        _ => THEME_COSMERE,
    }
}

/// Deterministic, high-contrast, beautiful palette for snippet IDs across all terminal themes.
pub const ID_PALETTE: &[Color] = &[
    Color::Rgb(56, 189, 248),  // Sky Blue
    Color::Rgb(52, 211, 153),  // Emerald Green
    Color::Rgb(251, 191, 36),  // Amber Gold
    Color::Rgb(192, 132, 252), // Violet Purple
    Color::Rgb(251, 113, 133), // Rose Coral
    Color::Rgb(251, 146, 60),  // Vivid Orange
    Color::Rgb(45, 212, 191),  // Teal Mint
    Color::Rgb(129, 140, 248), // Indigo Blue
    Color::Rgb(232, 121, 249), // Fuchsia Pink
    Color::Rgb(163, 230, 53),  // Lime Green
    Color::Rgb(94, 234, 212),  // Aquamarine
    Color::Rgb(252, 165, 165), // Soft Peach
];

/// Return a deterministic distinct color for any snippet ID.
pub fn get_id_color(id: &str) -> Color {
    if id.is_empty() {
        return ID_PALETTE[0];
    }
    // Deterministic djb2-like polynomial hash for uniform color distribution
    let hash = id.bytes().fold(5381u32, |acc, b| {
        acc.wrapping_mul(33).wrapping_add(b as u32)
    });
    let idx = (hash as usize) % ID_PALETTE.len();
    ID_PALETTE[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_resolution() {
        assert_eq!(get_theme_by_name("catppuccin").name, "Catppuccin Mocha");
        assert_eq!(get_theme_by_name("dracula").name, "Dracula");
        assert_eq!(get_theme_by_name("cosmere").name, "Cosmere");
        assert_eq!(get_theme_by_name("solarized").name, "Solarized Dark");
        assert_eq!(get_theme_by_name("synthwave").name, "Synthwave");
        assert_eq!(get_theme_by_name("custom").name, "Custom");
        assert_eq!(get_theme_by_name("default").name, "Cosmere");
    }

    #[test]
    fn test_theme_styles() {
        let theme = THEME_COSMERE;
        assert_eq!(theme.style_base().bg, Some(theme.background));
        assert_eq!(theme.style_surface().bg, Some(theme.surface));
    }

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#00ff00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_color("#f00"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("cyan"), Some(Color::Cyan));
        assert_eq!(parse_color("invalid_color"), None);
    }

    #[test]
    fn test_id_colors_deterministic() {
        let color1 = get_id_color("CP001");
        let color1_again = get_id_color("CP001");
        assert_eq!(color1, color1_again);

        let color2 = get_id_color("CP002");
        let color3 = get_id_color("ms_001");
        // Ensure they hash to valid palette colors
        assert!(ID_PALETTE.contains(&color1));
        assert!(ID_PALETTE.contains(&color2));
        assert!(ID_PALETTE.contains(&color3));
    }
}

