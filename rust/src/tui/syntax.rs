//! Syntax highlighting engine using syntect for Ratatui code preview.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub struct SyntaxHighlighter {
    pub ps: SyntaxSet,
    pub ts: ThemeSet,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
        }
    }

    /// Highlight `code` using `language` syntax into Ratatui `Line`s.
    pub fn highlight(&self, code: &str, language: &str) -> Vec<Line<'static>> {
        let syntax = self
            .ps
            .find_syntax_by_token(language)
            .or_else(|| self.ps.find_syntax_by_extension(language))
            .unwrap_or_else(|| self.ps.find_syntax_plain_text());

        let theme = &self.ts.themes["base16-ocean.dark"];
        let mut h = HighlightLines::new(syntax, theme);

        let mut lines = Vec::new();
        for line_str in code.lines() {
            let mut spans = Vec::new();
            if let Ok(ranges) = h.highlight_line(&format!("{}\n", line_str), &self.ps) {
                for (style, text) in ranges {
                    let text_clean = text.trim_end_matches('\n').trim_end_matches('\r');
                    if !text_clean.is_empty() {
                        let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        spans.push(Span::styled(text_clean.to_string(), Style::default().fg(fg)));
                    }
                }
            } else {
                spans.push(Span::raw(line_str.to_string()));
            }

            if spans.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(spans));
            }
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlighter_rust() {
        let highlighter = SyntaxHighlighter::new();
        let code = "pub fn main() {\n    println!(\"Hello\");\n}";
        let lines = highlighter.highlight(code, "rust");
        assert_eq!(lines.len(), 3);
        assert!(!lines[0].spans.is_empty());
    }

    #[test]
    fn test_syntax_highlighter_plain_fallback() {
        let highlighter = SyntaxHighlighter::new();
        let code = "plain text line 1\nline 2";
        let lines = highlighter.highlight(code, "unknown_lang_extension");
        assert_eq!(lines.len(), 2);
    }
}
