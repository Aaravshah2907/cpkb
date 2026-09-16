//! Language badges and developer icons for CPKB TUI.

use ratatui::style::Color;

pub struct LanguageBadge {
    pub name: &'static str,
    pub icon: &'static str,
    pub color: Color,
}

/// Return badge metadata (display name, NerdFont icon, and accent color) for a language identifier.
pub fn get_language_badge(lang: &str) -> LanguageBadge {
    match lang.to_lowercase().trim() {
        "cpp" | "c++" => LanguageBadge {
            name: "C++",
            icon: "󰌷",
            color: Color::Rgb(0, 89, 156), // C++ Blue
        },
        "c" => LanguageBadge {
            name: "C",
            icon: "",
            color: Color::Rgb(85, 85, 85),
        },
        "python" | "py" => LanguageBadge {
            name: "Python",
            icon: "",
            color: Color::Rgb(255, 212, 59), // Python Yellow
        },
        "rust" | "rs" => LanguageBadge {
            name: "Rust",
            icon: "",
            color: Color::Rgb(222, 165, 132), // Rust Orange
        },
        "tex" | "latex" | "plaintex" => LanguageBadge {
            name: "LaTeX",
            icon: "󰙩",
            color: Color::Rgb(0, 128, 128), // Teal
        },
        "javascript" | "js" => LanguageBadge {
            name: "JS",
            icon: "",
            color: Color::Rgb(247, 223, 30), // JS Yellow
        },
        "typescript" | "ts" => LanguageBadge {
            name: "TS",
            icon: "",
            color: Color::Rgb(49, 120, 198), // TS Blue
        },
        "go" | "golang" => LanguageBadge {
            name: "Go",
            icon: "󰟓",
            color: Color::Rgb(0, 173, 216), // Cyan
        },
        "java" => LanguageBadge {
            name: "Java",
            icon: "󰘐",
            color: Color::Rgb(231, 111, 0), // Java Orange
        },
        "lua" => LanguageBadge {
            name: "Lua",
            icon: "",
            color: Color::Rgb(0, 0, 128), // Blue
        },
        "bash" | "sh" | "zsh" => LanguageBadge {
            name: "Shell",
            icon: "",
            color: Color::Rgb(78, 186, 111), // Green
        },
        "markdown" | "md" => LanguageBadge {
            name: "Markdown",
            icon: "",
            color: Color::Rgb(138, 107, 246), // Purple
        },
        "sql" => LanguageBadge {
            name: "SQL",
            icon: "󰆼",
            color: Color::Rgb(227, 140, 45), // Orange
        },
        "html" | "htm" => LanguageBadge {
            name: "HTML",
            icon: "",
            color: Color::Rgb(227, 79, 38), // HTML Orange
        },
        "css" | "scss" | "sass" | "less" => LanguageBadge {
            name: "CSS",
            icon: "",
            color: Color::Rgb(21, 114, 182), // CSS Blue
        },
        "json" => LanguageBadge {
            name: "JSON",
            icon: "",
            color: Color::Rgb(203, 203, 65), // JSON Yellow
        },
        "yaml" | "yml" => LanguageBadge {
            name: "YAML",
            icon: "",
            color: Color::Rgb(203, 23, 30), // YAML Red
        },
        "toml" => LanguageBadge {
            name: "TOML",
            icon: "",
            color: Color::Rgb(156, 65, 33), // TOML Brown
        },
        "kotlin" | "kt" | "kts" => LanguageBadge {
            name: "Kotlin",
            icon: "",
            color: Color::Rgb(127, 82, 255), // Kotlin Purple
        },
        "swift" => LanguageBadge {
            name: "Swift",
            icon: "",
            color: Color::Rgb(250, 115, 67), // Swift Orange
        },
        "ruby" | "rb" => LanguageBadge {
            name: "Ruby",
            icon: "",
            color: Color::Rgb(204, 52, 45), // Ruby Red
        },
        "php" => LanguageBadge {
            name: "PHP",
            icon: "",
            color: Color::Rgb(119, 123, 180), // PHP Blue/Violet
        },
        "haskell" | "hs" => LanguageBadge {
            name: "Haskell",
            icon: "",
            color: Color::Rgb(94, 80, 134), // Haskell Violet
        },
        "zig" => LanguageBadge {
            name: "Zig",
            icon: "",
            color: Color::Rgb(247, 164, 29), // Zig Gold
        },
        "dart" => LanguageBadge {
            name: "Dart",
            icon: "",
            color: Color::Rgb(1, 117, 194), // Dart Cyan
        },
        _ => LanguageBadge {
            name: "Text",
            icon: "󰉿",
            color: Color::Rgb(148, 163, 184), // Slate Gray
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_badges_mapping() {
        assert_eq!(get_language_badge("cpp").name, "C++");
        assert_eq!(get_language_badge("python").name, "Python");
        assert_eq!(get_language_badge("rust").name, "Rust");
        assert_eq!(get_language_badge("tex").name, "LaTeX");
        assert_eq!(get_language_badge("latex").name, "LaTeX");
        assert_eq!(get_language_badge("html").name, "HTML");
        assert_eq!(get_language_badge("zig").name, "Zig");
        assert_eq!(get_language_badge("kotlin").name, "Kotlin");
        assert_eq!(get_language_badge("unknown_lang").name, "Text");
    }
}
