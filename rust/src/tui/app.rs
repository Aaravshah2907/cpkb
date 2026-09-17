//! Application state machine and business logic for CPKB TUI.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::widgets::ListState;
use rusqlite::Connection;

use crate::clipboard::copy_to_clipboard;
use crate::config::{load_config, save_config, Config};
use crate::db::snippets::{add_snippet, delete_snippet, get_snippet, list_snippets, update_snippet, Snippet, SnippetSummary};
use crate::tui::syntax::SyntaxHighlighter;
use crate::tui::theme::{get_available_themes, get_theme, Theme};

#[derive(Debug, Clone, PartialEq)]
pub struct AddSnippetModalState {
    pub title: String,
    pub description: String,
    pub use_case: String,
    pub tags: String,
    pub language: String,
    pub code: String,
    pub focus_idx: usize, // 0..=5
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditSnippetModalState {
    pub id: String,
    pub title: String,
    pub description: String,
    pub use_case: String,
    pub tags: String,
    pub language: String,
    pub code: String,
    pub focus_idx: usize, // 0..=5
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Date,
    Id,
    Name,
}

impl SortOrder {
    pub fn label(&self) -> &'static str {
        match self {
            SortOrder::Date => "Date",
            SortOrder::Id => "ID",
            SortOrder::Name => "Name",
        }
    }
}

pub const SORT_OPTIONS: &[(&str, &str)] = &[
    ("date", "Recent / Date"),
    ("id", "Snippet ID"),
    ("name", "Snippet Name"),
];

/// Compare two strings naturally, parsing numeric sequences as numbers so CP1 < CP2 < CP10.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    while a_chars.peek().is_some() || b_chars.peek().is_some() {
        match (a_chars.peek(), b_chars.peek()) {
            (Some(ac), Some(bc)) if ac.is_ascii_digit() && bc.is_ascii_digit() => {
                let mut a_num = 0u64;
                while let Some(&c) = a_chars.peek() {
                    if let Some(d) = c.to_digit(10) {
                        a_num = a_num.saturating_mul(10).saturating_add(d as u64);
                        a_chars.next();
                    } else {
                        break;
                    }
                }
                let mut b_num = 0u64;
                while let Some(&c) = b_chars.peek() {
                    if let Some(d) = c.to_digit(10) {
                        b_num = b_num.saturating_mul(10).saturating_add(d as u64);
                        b_chars.next();
                    } else {
                        break;
                    }
                }
                if a_num != b_num {
                    return a_num.cmp(&b_num);
                }
            }
            (Some(ac), Some(bc)) => {
                let ac_lower = ac.to_lowercase().next().unwrap_or(*ac);
                let bc_lower = bc.to_lowercase().next().unwrap_or(*bc);
                if ac_lower != bc_lower {
                    return ac_lower.cmp(&bc_lower);
                }
                a_chars.next();
                b_chars.next();
            }
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (None, None) => break,
        }
    }
    std::cmp::Ordering::Equal
}

pub const LAYOUT_OPTIONS: &[(&str, &str)] = &[
    ("horizontal", "Horizontal (Side-by-side)"),
    ("vertical", "Vertical (Stacked)"),
];

pub const BORDER_OPTIONS: &[(&str, &str)] = &[
    ("round", "Rounded"),
    ("solid", "Solid / Plain"),
    ("double", "Double"),
    ("thick", "Thick / Heavy"),
];

pub const CUSTOM_THEME_FIELDS: &[(&str, &str)] = &[
    ("primary", "Primary (focused borders, headers)"),
    ("secondary", "Secondary (accents, tags)"),
    ("warning", "Warning (warnings, alerts)"),
    ("error", "Error (errors, delete markers)"),
    ("success", "Success (confirmations, added)"),
    ("accent", "Accent (highlights, badges)"),
    ("foreground", "Foreground (main text)"),
    ("background", "Background (main window bg)"),
    ("surface", "Surface (card/panel surface)"),
    ("panel", "Panel (borders, dividers)"),
    ("boost", "Boost (dimmed text, subtitles)"),
];

#[derive(Debug, Clone, PartialEq)]
pub struct CustomThemeModalState {
    pub primary: String,
    pub secondary: String,
    pub warning: String,
    pub error: String,
    pub success: String,
    pub accent: String,
    pub foreground: String,
    pub background: String,
    pub surface: String,
    pub panel: String,
    pub boost: String,
    pub focus_idx: usize, // 0..=10
}

impl CustomThemeModalState {
    pub fn get_value(&self, idx: usize) -> &str {
        match idx {
            0 => &self.primary,
            1 => &self.secondary,
            2 => &self.warning,
            3 => &self.error,
            4 => &self.success,
            5 => &self.accent,
            6 => &self.foreground,
            7 => &self.background,
            8 => &self.surface,
            9 => &self.panel,
            10 => &self.boost,
            _ => "",
        }
    }

    pub fn get_value_mut(&mut self, idx: usize) -> &mut String {
        match idx {
            0 => &mut self.primary,
            1 => &mut self.secondary,
            2 => &mut self.warning,
            3 => &mut self.error,
            4 => &mut self.success,
            5 => &mut self.accent,
            6 => &mut self.foreground,
            7 => &mut self.background,
            8 => &mut self.surface,
            9 => &mut self.panel,
            10 => &mut self.boost,
            _ => &mut self.primary,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsModalState {
    pub theme_idx: usize,
    pub lang_idx: usize,
    pub sort_idx: usize,
    pub layout_idx: usize,
    pub border_idx: usize,
    pub focus_idx: usize, // 0=Theme, 1=Language, 2=Sort, 3=Layout, 4=Border, 5=Edit Custom Theme
}

pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "cpp", "c", "python", "rust", "tex", "javascript", "typescript", "go", "java", "kotlin", "lua", "bash",
    "markdown", "sql", "html", "css", "json", "yaml", "toml", "swift", "ruby", "php", "zig", "text",
];

#[derive(Debug, Clone, PartialEq)]
pub struct TagSelectModalState {
    pub tags: Vec<(String, usize)>,
    pub selected_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveModal {
    Help,
    DeleteConfirm(String), // Snippet ID to delete
    AddSnippet(AddSnippetModalState),
    EditSnippet(EditSnippetModalState),
    Settings(SettingsModalState),
    CustomTheme(CustomThemeModalState),
    TagSelect(TagSelectModalState),
}


pub struct App {
    pub app_dir: PathBuf,
    pub config: Config,
    pub all_snippets: Vec<SnippetSummary>,
    pub filtered_indices: Vec<usize>,
    pub selected_index: usize,
    pub list_state: ListState,
    pub active_snippet: Option<Snippet>,
    pub search_query: String,
    pub search_active: bool,
    pub language_filter: Option<String>,
    pub tag_filter: Option<String>,
    pub theme: Theme,
    pub sort_order: SortOrder,
    pub status_message: Option<(String, Instant)>,
    pub should_quit: bool,
    pub active_modal: Option<ActiveModal>,
    pub left_panel_percent: u16,
    pub highlighter: SyntaxHighlighter,
    matcher: SkimMatcherV2,
}

impl App {
    pub fn new(conn: &Connection, app_dir: &Path) -> Self {
        let config = load_config(app_dir);
        let theme = get_theme(&config.display.theme, Some(&config.display.custom_theme));
        let all_snippets = list_snippets(conn).unwrap_or_default();

        let sort_order = match config.display.sort_by.to_lowercase().as_str() {
            "id" => SortOrder::Id,
            "name" | "title" => SortOrder::Name,
            _ => SortOrder::Date,
        };

        // Load persisted left panel width from config, clamped to a sane range
        let left_panel_percent = (config.display.left_pane_width as u16).clamp(20, 70);

        let mut app = Self {
            app_dir: app_dir.to_path_buf(),
            config,
            all_snippets,
            filtered_indices: Vec::new(),
            selected_index: 0,
            list_state: ListState::default(),
            active_snippet: None,
            search_query: String::new(),
            search_active: false,
            language_filter: None,
            tag_filter: None,
            theme,
            sort_order,
            status_message: None,
            should_quit: false,
            active_modal: None,
            left_panel_percent,
            highlighter: SyntaxHighlighter::new(),
            matcher: SkimMatcherV2::default(),
        };

        app.apply_filter();
        app.reload_active_snippet(conn);
        app
    }

    /// Refresh the list of snippets from the database.
    pub fn reload_snippets(&mut self, conn: &Connection) {
        self.all_snippets = list_snippets(conn).unwrap_or_default();
        self.apply_filter();
        self.reload_active_snippet(conn);
    }

    /// Fetch and cache full metadata/code for currently selected snippet.
    pub fn reload_active_snippet(&mut self, conn: &Connection) {
        if self.filtered_indices.is_empty() {
            self.active_snippet = None;
            self.list_state.select(None);
            return;
        }

        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = self.filtered_indices.len().saturating_sub(1);
        }

        self.list_state.select(Some(self.selected_index));
        let actual_idx = self.filtered_indices[self.selected_index];
        let summary = &self.all_snippets[actual_idx];
        self.active_snippet = get_snippet(conn, &summary.id).ok().flatten();
    }

    /// Apply fuzzy / keyword search filter, language filter, tag filter, and active sort order across snippets.
    pub fn apply_filter(&mut self) {
        let is_eligible = |snip: &SnippetSummary| -> bool {
            if let Some(ref lang) = self.language_filter {
                if &snip.language.to_lowercase().trim() != lang {
                    return false;
                }
            }
            if let Some(ref tag) = self.tag_filter {
                let tag_lower = tag.to_lowercase();
                let matches_tag = snip
                    .tags
                    .split(',')
                    .any(|t| t.trim().to_lowercase() == tag_lower);
                if !matches_tag {
                    return false;
                }
            }
            true
        };

        if self.search_query.trim().is_empty() {
            let mut indices: Vec<usize> = (0..self.all_snippets.len())
                .filter(|&idx| is_eligible(&self.all_snippets[idx]))
                .collect();
            match self.sort_order {
                SortOrder::Id => {
                    indices.sort_by(|&a, &b| natural_cmp(&self.all_snippets[a].id, &self.all_snippets[b].id));
                }
                SortOrder::Name => {
                    indices.sort_by(|&a, &b| self.all_snippets[a].title.to_lowercase().cmp(&self.all_snippets[b].title.to_lowercase()));
                }
                SortOrder::Date => {
                    // all_snippets is already sorted by date descending from SQLite
                }
            }
            self.filtered_indices = indices;
        } else {
            let mut matches = Vec::new();
            for (idx, snip) in self.all_snippets.iter().enumerate() {
                if !is_eligible(snip) {
                    continue;
                }
                let target = format!("{} {} {}", snip.id, snip.title, snip.tags);
                if let Some(score) = self.matcher.fuzzy_match(&target, &self.search_query) {
                    matches.push((score, idx));
                }
            }
            matches.sort_by(|a, b| {
                b.0.cmp(&a.0).then_with(|| match self.sort_order {
                    SortOrder::Id => natural_cmp(&self.all_snippets[a.1].id, &self.all_snippets[b.1].id),
                    SortOrder::Name => self.all_snippets[a.1].title.to_lowercase().cmp(&self.all_snippets[b.1].title.to_lowercase()),
                    SortOrder::Date => a.1.cmp(&b.1),
                })
            });
            self.filtered_indices = matches.into_iter().map(|(_, idx)| idx).collect();
        }

        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = 0;
        }
        self.list_state.select(if self.filtered_indices.is_empty() { None } else { Some(self.selected_index) });
    }


    pub fn next(&mut self, conn: &Connection) {
        if !self.filtered_indices.is_empty() {
            if self.selected_index + 1 < self.filtered_indices.len() {
                self.selected_index += 1;
            } else {
                self.selected_index = 0; // Wrap around
            }
            self.reload_active_snippet(conn);
        }
    }

    pub fn previous(&mut self, conn: &Connection) {
        if !self.filtered_indices.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.filtered_indices.len().saturating_sub(1); // Wrap around
            }
            self.reload_active_snippet(conn);
        }
    }

    pub fn select_first(&mut self, conn: &Connection) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = 0;
            self.reload_active_snippet(conn);
        }
    }

    pub fn select_last(&mut self, conn: &Connection) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = self.filtered_indices.len().saturating_sub(1);
            self.reload_active_snippet(conn);
        }
    }

    pub fn increase_panel_width(&mut self) {
        if self.left_panel_percent < 70 {
            self.left_panel_percent += 5;
            // Persist so the width survives restarts
            self.config.display.left_pane_width = self.left_panel_percent as u32;
        }
    }

    pub fn decrease_panel_width(&mut self) {
        if self.left_panel_percent > 20 {
            self.left_panel_percent -= 5;
            // Persist so the width survives restarts
            self.config.display.left_pane_width = self.left_panel_percent as u32;
        }
    }

    pub fn copy_current_code(&mut self) {
        if let Some(ref snip) = self.active_snippet {
            match copy_to_clipboard(&snip.code) {
                Ok(_) => {
                    self.set_status_message(format!("Copied {} code to clipboard!", snip.id));
                }
                Err(e) => {
                    self.set_status_message(format!("Clipboard copy failed: {}", e));
                }
            }
        }
    }

    pub fn open_add_modal(&mut self) {
        let default_lang = self.config.default_language.clone();
        self.active_modal = Some(ActiveModal::AddSnippet(AddSnippetModalState {
            title: String::new(),
            description: String::new(),
            use_case: String::new(),
            tags: String::new(),
            language: if default_lang.is_empty() { "cpp".to_string() } else { default_lang },
            code: String::new(),
            focus_idx: 0,
        }));
    }

    pub fn open_edit_modal(&mut self) {
        if let Some(ref snip) = self.active_snippet {
            self.active_modal = Some(ActiveModal::EditSnippet(EditSnippetModalState {
                id: snip.id.clone(),
                title: snip.title.clone(),
                description: snip.description.clone(),
                use_case: snip.use_case.clone(),
                tags: snip.tags.clone(),
                language: snip.language.clone(),
                code: snip.code.clone(),
                focus_idx: 0,
            }));
        }
    }

    pub fn open_settings_modal(&mut self) {
        let available_themes = get_available_themes(Some(&self.config.display.custom_theme));
        let current_theme_name = self.theme.name;
        let theme_idx = available_themes.iter().position(|(name, _)| *name == current_theme_name).unwrap_or(0);

        let current_lang = self.config.default_language.to_lowercase();
        let lang_idx = SUPPORTED_LANGUAGES.iter().position(|l| *l == current_lang).unwrap_or(0);

        let sort_idx = match self.sort_order {
            SortOrder::Date => 0,
            SortOrder::Id => 1,
            SortOrder::Name => 2,
        };

        let layout_idx = match self.config.display.layout.to_lowercase().as_str() {
            "vertical" => 1,
            _ => 0,
        };
        let border_idx = match self.config.display.border_style.to_lowercase().as_str() {
            "solid" | "plain" => 1,
            "double" => 2,
            "thick" | "heavy" => 3,
            _ => 0,
        };

        self.active_modal = Some(ActiveModal::Settings(SettingsModalState {
            theme_idx,
            lang_idx,
            sort_idx,
            layout_idx,
            border_idx,
            focus_idx: 0,
        }));
    }

    pub fn open_custom_theme_modal(&mut self) {
        let ct = &self.config.display.custom_theme;
        self.active_modal = Some(ActiveModal::CustomTheme(CustomThemeModalState {
            primary: ct.primary.clone(),
            secondary: ct.secondary.clone(),
            warning: ct.warning.clone(),
            error: ct.error.clone(),
            success: ct.success.clone(),
            accent: ct.accent.clone(),
            foreground: ct.foreground.clone(),
            background: ct.background.clone(),
            surface: ct.surface.clone(),
            panel: ct.panel.clone(),
            boost: ct.boost.clone(),
            focus_idx: 0,
        }));
    }

    pub fn save_custom_theme(&mut self, state: CustomThemeModalState) {
        self.config.display.custom_theme.primary = state.primary;
        self.config.display.custom_theme.secondary = state.secondary;
        self.config.display.custom_theme.warning = state.warning;
        self.config.display.custom_theme.error = state.error;
        self.config.display.custom_theme.success = state.success;
        self.config.display.custom_theme.accent = state.accent;
        self.config.display.custom_theme.foreground = state.foreground;
        self.config.display.custom_theme.background = state.background;
        self.config.display.custom_theme.surface = state.surface;
        self.config.display.custom_theme.panel = state.panel;
        self.config.display.custom_theme.boost = state.boost;

        // If Custom theme is active, reload theme colors immediately
        if self.config.display.theme.to_lowercase() == "custom" || self.theme.name == "Custom" {
            self.theme = crate::tui::theme::from_custom(&self.config.display.custom_theme);
        }

        if let Err(e) = save_config(&self.app_dir, &self.config) {
            self.set_status_message(format!("Failed to save custom theme: {}", e));
        } else {
            self.set_status_message("Custom theme colors saved successfully!".to_string());
        }
        self.active_modal = None;
    }

    pub fn toggle_sort(&mut self, conn: &Connection) {
        self.sort_order = match self.sort_order {
            SortOrder::Date => SortOrder::Id,
            SortOrder::Id => SortOrder::Name,
            SortOrder::Name => SortOrder::Date,
        };
        self.config.display.sort_by = match self.sort_order {
            SortOrder::Date => "date".to_string(),
            SortOrder::Id => "id".to_string(),
            SortOrder::Name => "name".to_string(),
        };
        self.set_status_message(format!("Sorted by: {}", self.sort_order.label()));
        self.apply_filter();
        self.reload_active_snippet(conn);
    }

    pub fn cycle_language_filter(&mut self, conn: &Connection) {
        let mut languages: Vec<String> = self
            .all_snippets
            .iter()
            .map(|s| s.language.to_lowercase().trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        languages.sort();
        languages.dedup();

        if languages.is_empty() {
            self.set_status_message("No snippets available to filter.".to_string());
            return;
        }

        self.language_filter = match &self.language_filter {
            None => Some(languages[0].clone()),
            Some(curr) => {
                if let Some(pos) = languages.iter().position(|l| l == curr) {
                    if pos + 1 < languages.len() {
                        Some(languages[pos + 1].clone())
                    } else {
                        None // Wrap back to All
                    }
                } else {
                    None
                }
            }
        };

        self.apply_filter();
        self.reload_active_snippet(conn);
        if let Some(ref l) = self.language_filter {
            self.set_status_message(format!("Filtered by language: {}", l));
        } else {
            self.set_status_message("Language filter: All (cleared)".to_string());
        }
    }

    pub fn open_tag_modal(&mut self) {
        use std::collections::HashMap;
        let mut counts: HashMap<String, usize> = HashMap::new();
        for s in &self.all_snippets {
            for t in s.tags.split(',') {
                let tag = t.trim();
                if !tag.is_empty() {
                    *counts.entry(tag.to_string()).or_insert(0) += 1;
                }
            }
        }
        let mut tags: Vec<(String, usize)> = counts.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        if tags.is_empty() {
            self.set_status_message("No tags found in knowledge base.".to_string());
            return;
        }

        let selected_idx = if let Some(ref current_tag) = self.tag_filter {
            tags.iter().position(|(t, _)| t == current_tag).unwrap_or(0)
        } else {
            0
        };

        self.active_modal = Some(ActiveModal::TagSelect(TagSelectModalState {
            tags,
            selected_idx,
        }));
    }

    pub fn apply_tag_filter(&mut self, tag: Option<String>, conn: &Connection) {
        self.tag_filter = tag.clone();
        self.active_modal = None;
        self.apply_filter();
        self.reload_active_snippet(conn);
        if let Some(t) = tag {
            self.set_status_message(format!("Filtered by tag: {}", t));
        } else {
            self.set_status_message("Tag filter cleared.".to_string());
        }
    }

    pub fn clear_all_filters(&mut self, conn: &Connection) {
        let had_filter = self.language_filter.is_some() || self.tag_filter.is_some() || !self.search_query.is_empty();
        self.language_filter = None;
        self.tag_filter = None;
        self.search_query.clear();
        self.apply_filter();
        self.reload_active_snippet(conn);
        if had_filter {
            self.set_status_message("All search, language, and tag filters cleared.".to_string());
        }
    }


    pub fn confirm_delete_current(&mut self) {
        if let Some(ref snip) = self.active_snippet {
            self.active_modal = Some(ActiveModal::DeleteConfirm(snip.id.clone()));
        }
    }

    pub fn execute_delete(&mut self, conn: &Connection, id: &str) {
        match delete_snippet(conn, id) {
            Ok(true) => {
                self.set_status_message(format!("Deleted snippet {}", id));
                self.reload_snippets(conn);
            }
            Ok(false) => {
                self.set_status_message(format!("Snippet {} not found", id));
            }
            Err(e) => {
                self.set_status_message(format!("Failed to delete {}: {}", id, e));
            }
        }
        self.active_modal = None;
    }

    pub fn save_new_snippet(&mut self, conn: &mut Connection, state: AddSnippetModalState) {
        if state.title.trim().is_empty() || state.code.trim().is_empty() {
            self.set_status_message("Title and Code cannot be empty!".to_string());
            return;
        }

        match add_snippet(
            conn,
            &self.app_dir,
            &state.title,
            &state.description,
            &state.use_case,
            &state.tags,
            &state.code,
            Some(&state.language),
            None,
        ) {
            Ok(id) => {
                self.set_status_message(format!("Created snippet {} successfully!", id));
                self.active_modal = None;
                self.reload_snippets(conn);
            }
            Err(e) => {
                self.set_status_message(format!("Failed to create snippet: {}", e));
            }
        }
    }

    pub fn save_edited_snippet(&mut self, conn: &mut Connection, state: EditSnippetModalState) {
        if state.title.trim().is_empty() || state.code.trim().is_empty() {
            self.set_status_message("Title and Code cannot be empty!".to_string());
            return;
        }

        match update_snippet(
            conn,
            &state.id,
            &state.title,
            &state.description,
            &state.use_case,
            &state.tags,
            &state.code,
            &state.language,
        ) {
            Ok(true) => {
                self.set_status_message(format!("Updated snippet {} successfully!", state.id));
                self.active_modal = None;
                self.reload_snippets(conn);
            }
            Ok(false) => {
                self.set_status_message(format!("Snippet {} not found", state.id));
            }
            Err(e) => {
                self.set_status_message(format!("Failed to update snippet: {}", e));
            }
        }
    }

    pub fn save_settings(&mut self, state: SettingsModalState) {
        let available_themes = get_available_themes(Some(&self.config.display.custom_theme));
        let (theme_name, theme) = &available_themes[state.theme_idx.min(available_themes.len() - 1)];
        let lang = SUPPORTED_LANGUAGES[state.lang_idx.min(SUPPORTED_LANGUAGES.len() - 1)];
        let sort_opt = match state.sort_idx {
            1 => SortOrder::Id,
            2 => SortOrder::Name,
            _ => SortOrder::Date,
        };
        let layout = LAYOUT_OPTIONS[state.layout_idx.min(LAYOUT_OPTIONS.len() - 1)].0;
        let border = BORDER_OPTIONS[state.border_idx.min(BORDER_OPTIONS.len() - 1)].0;

        self.theme = theme.clone();
        self.sort_order = sort_opt;
        self.config.display.theme = theme_name.to_lowercase().replace(' ', "-");
        self.config.default_language = lang.to_string();
        self.config.display.sort_by = match sort_opt {
            SortOrder::Date => "date".to_string(),
            SortOrder::Id => "id".to_string(),
            SortOrder::Name => "name".to_string(),
        };
        self.config.display.layout = layout.to_string();
        self.config.display.border_style = border.to_string();

        if let Err(e) = save_config(&self.app_dir, &self.config) {
            self.set_status_message(format!("Failed to save config: {}", e));
        } else {
            self.set_status_message(format!("Settings saved: Theme = {}, Lang = {}, Sort = {}", theme_name, lang, sort_opt.label()));
        }
        self.apply_filter();
        self.active_modal = None;
    }



    pub fn set_status_message(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn active_status(&self) -> Option<&str> {
        if let Some((ref msg, instant)) = self.status_message {
            if instant.elapsed() < Duration::from_secs(4) {
                return Some(msg.as_str());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::db::init_db;
    use crate::db::snippets::add_snippet;

    #[test]
    fn test_app_initialization_and_navigation() {
        let dir = tempdir().unwrap();
        let mut conn = init_db(dir.path()).unwrap();

        add_snippet(&mut conn, dir.path(), "Alpha", "", "", "tag1", "a()", Some("cpp"), None).unwrap();
        add_snippet(&mut conn, dir.path(), "Beta", "", "", "tag2", "b()", Some("rust"), None).unwrap();

        let mut app = App::new(&conn, dir.path());
        assert_eq!(app.all_snippets.len(), 2);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.active_snippet.as_ref().unwrap().title, "Beta"); // newest first

        app.next(&conn);
        assert_eq!(app.selected_index, 1);
        assert_eq!(app.active_snippet.as_ref().unwrap().title, "Alpha");

        // Wrap around
        app.next(&conn);
        assert_eq!(app.selected_index, 0);

        app.previous(&conn);
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_app_fuzzy_filtering() {
        let dir = tempdir().unwrap();
        let mut conn = init_db(dir.path()).unwrap();

        add_snippet(&mut conn, dir.path(), "Dijkstra", "", "", "graph", "d()", Some("cpp"), None).unwrap();
        add_snippet(&mut conn, dir.path(), "Segment Tree", "", "", "range", "st()", Some("cpp"), None).unwrap();

        let mut app = App::new(&conn, dir.path());
        assert_eq!(app.filtered_indices.len(), 2);

        app.search_query = "dijk".to_string();
        app.apply_filter();
        assert_eq!(app.filtered_indices.len(), 1);

        app.search_query.clear();
        app.apply_filter();
        assert_eq!(app.filtered_indices.len(), 2);
    }

    #[test]
    fn test_app_panel_resizing() {
        let dir = tempdir().unwrap();
        let conn = init_db(dir.path()).unwrap();
        let mut app = App::new(&conn, dir.path());

        let initial = app.left_panel_percent;
        app.increase_panel_width();
        assert_eq!(app.left_panel_percent, initial + 5);

        app.decrease_panel_width();
        assert_eq!(app.left_panel_percent, initial);
    }

    #[test]
    fn test_natural_cmp() {
        assert_eq!(natural_cmp("CP1", "CP2"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("CP2", "CP10"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("CP0001", "CP0002"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("CP10", "CP2"), std::cmp::Ordering::Greater);
        assert_eq!(natural_cmp("ALG.1", "ALG.2"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("same", "SAME"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_app_sorting_by_id_and_name() {
        let dir = tempdir().unwrap();
        let mut conn = init_db(dir.path()).unwrap();

        let id1 = add_snippet(&mut conn, dir.path(), "Zebra", "", "", "", "z()", Some("cpp"), None).unwrap();
        let id2 = add_snippet(&mut conn, dir.path(), "Apple", "", "", "", "a()", Some("rust"), None).unwrap();

        let mut app = App::new(&conn, dir.path());

        // Sort by ID
        app.sort_order = SortOrder::Id;
        app.apply_filter();
        let first_id = &app.all_snippets[app.filtered_indices[0]].id;
        let second_id = &app.all_snippets[app.filtered_indices[1]].id;
        assert_eq!(first_id, &id1); // id1 created first, so lower number
        assert_eq!(second_id, &id2);

        // Sort by Name
        app.sort_order = SortOrder::Name;
        app.apply_filter();
        let first_title = &app.all_snippets[app.filtered_indices[0]].title;
        let second_title = &app.all_snippets[app.filtered_indices[1]].title;
        assert_eq!(first_title, "Apple");
        assert_eq!(second_title, "Zebra");

        // Toggle sort cycles Date -> Id -> Name -> Date
        app.sort_order = SortOrder::Date;
        app.toggle_sort(&conn);
        assert_eq!(app.sort_order, SortOrder::Id);
        app.toggle_sort(&conn);
        assert_eq!(app.sort_order, SortOrder::Name);
        app.toggle_sort(&conn);
        assert_eq!(app.sort_order, SortOrder::Date);
    }
}
