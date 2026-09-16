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
use crate::tui::theme::{get_theme_by_name, Theme, THEMES};

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

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsModalState {
    pub theme_idx: usize,
    pub lang_idx: usize,
    pub focus_idx: usize, // 0=Theme, 1=Language
}

pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "cpp", "c", "python", "rust", "tex", "javascript", "typescript", "go", "java", "lua", "bash", "markdown", "sql", "text",
];

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveModal {
    Help,
    DeleteConfirm(String), // Snippet ID to delete
    AddSnippet(AddSnippetModalState),
    EditSnippet(EditSnippetModalState),
    Settings(SettingsModalState),
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
    pub theme: Theme,
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
        let theme = get_theme_by_name(&config.display.theme);
        let all_snippets = list_snippets(conn).unwrap_or_default();
        let filtered_indices: Vec<usize> = (0..all_snippets.len()).collect();

        let mut list_state = ListState::default();
        if !filtered_indices.is_empty() {
            list_state.select(Some(0));
        }

        let mut app = Self {
            app_dir: app_dir.to_path_buf(),
            config,
            all_snippets,
            filtered_indices,
            selected_index: 0,
            list_state,
            active_snippet: None,
            search_query: String::new(),
            search_active: false,
            theme,
            status_message: None,
            should_quit: false,
            active_modal: None,
            left_panel_percent: 38,
            highlighter: SyntaxHighlighter::new(),
            matcher: SkimMatcherV2::default(),
        };

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

    /// Apply fuzzy / keyword search filter across snippets.
    pub fn apply_filter(&mut self) {
        if self.search_query.trim().is_empty() {
            self.filtered_indices = (0..self.all_snippets.len()).collect();
        } else {
            let mut matches = Vec::new();
            for (idx, snip) in self.all_snippets.iter().enumerate() {
                let target = format!("{} {} {}", snip.id, snip.title, snip.tags);
                if let Some(score) = self.matcher.fuzzy_match(&target, &self.search_query) {
                    matches.push((score, idx));
                }
            }
            matches.sort_by(|a, b| b.0.cmp(&a.0));
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
        }
    }

    pub fn decrease_panel_width(&mut self) {
        if self.left_panel_percent > 20 {
            self.left_panel_percent -= 5;
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
        let current_theme_name = self.theme.name;
        let theme_idx = THEMES.iter().position(|(name, _)| *name == current_theme_name).unwrap_or(0);

        let current_lang = self.config.default_language.to_lowercase();
        let lang_idx = SUPPORTED_LANGUAGES.iter().position(|l| *l == current_lang).unwrap_or(0);

        self.active_modal = Some(ActiveModal::Settings(SettingsModalState {
            theme_idx,
            lang_idx,
            focus_idx: 0,
        }));
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
        let (theme_name, theme) = &THEMES[state.theme_idx];
        let lang = SUPPORTED_LANGUAGES[state.lang_idx];

        self.theme = theme.clone();
        self.config.display.theme = theme_name.to_lowercase().replace(' ', "-");
        self.config.default_language = lang.to_string();

        if let Err(e) = save_config(&self.app_dir, &self.config) {
            self.set_status_message(format!("Failed to save config: {}", e));
        } else {
            self.set_status_message(format!("Settings saved: Theme = {}, Default Lang = {}", theme_name, lang));
        }
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
}
