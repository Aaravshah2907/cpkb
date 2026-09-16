//! Comprehensive integration and unit tests for CPKB Ratatui TUI.

use tempfile::tempdir;
use cpkb::config::load_config;
use cpkb::db::init_db;
use cpkb::db::snippets::{add_snippet, get_snippet};
use cpkb::tui::app::{ActiveModal, AddSnippetModalState, App, EditSnippetModalState, SettingsModalState};
use cpkb::tui::badges::get_language_badge;
use cpkb::tui::syntax::SyntaxHighlighter;
use cpkb::tui::theme::{get_theme_by_name, THEMES};

#[test]
fn test_tui_app_creation_and_empty_state() {
    let dir = tempdir().unwrap();
    let conn = init_db(dir.path()).unwrap();

    let app = App::new(&conn, dir.path());
    assert_eq!(app.all_snippets.len(), 0);
    assert_eq!(app.filtered_indices.len(), 0);
    assert!(app.active_snippet.is_none());
    assert_eq!(app.selected_index, 0);
    assert!(!app.search_active);
    assert!(app.active_modal.is_none());
}

#[test]
fn test_tui_app_with_snippets_and_navigation() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id1 = add_snippet(&mut conn, dir.path(), "Binary Search", "desc1", "use1", "search", "code1", Some("cpp"), None).unwrap();
    let id2 = add_snippet(&mut conn, dir.path(), "Merge Sort", "desc2", "use2", "sort", "code2", Some("rust"), None).unwrap();
    let id3 = add_snippet(&mut conn, dir.path(), "Dijkstra", "desc3", "use3", "graph", "code3", Some("python"), None).unwrap();

    let mut app = App::new(&conn, dir.path());
    assert_eq!(app.all_snippets.len(), 3);
    assert_eq!(app.filtered_indices.len(), 3);
    // Newest first by default
    assert_eq!(app.active_snippet.as_ref().unwrap().id, id3);

    // Navigate next
    app.next(&conn);
    assert_eq!(app.selected_index, 1);
    assert_eq!(app.active_snippet.as_ref().unwrap().id, id2);

    app.next(&conn);
    assert_eq!(app.selected_index, 2);
    assert_eq!(app.active_snippet.as_ref().unwrap().id, id1);

    // Wrap around to top
    app.next(&conn);
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.active_snippet.as_ref().unwrap().id, id3);

    // Wrap around to bottom with previous
    app.previous(&conn);
    assert_eq!(app.selected_index, 2);
    assert_eq!(app.active_snippet.as_ref().unwrap().id, id1);

    // Jump to first and last
    app.select_first(&conn);
    assert_eq!(app.selected_index, 0);
    app.select_last(&conn);
    assert_eq!(app.selected_index, 2);
}

#[test]
fn test_tui_add_modal_save_flow() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();
    let mut app = App::new(&conn, dir.path());

    app.open_add_modal();
    assert!(matches!(app.active_modal, Some(ActiveModal::AddSnippet(_))));

    let new_state = AddSnippetModalState {
        title: "Trie Tree".to_string(),
        description: "Prefix tree for string search".to_string(),
        use_case: "Autocomplete".to_string(),
        tags: "string, tree, trie".to_string(),
        language: "cpp".to_string(),
        code: "struct TrieNode {};".to_string(),
        focus_idx: 0,
    };

    app.save_new_snippet(&mut conn, new_state);
    assert!(app.active_modal.is_none());
    assert_eq!(app.all_snippets.len(), 1);
    assert_eq!(app.all_snippets[0].title, "Trie Tree");
    assert!(app.active_status().unwrap().contains("Created snippet"));
}

#[test]
fn test_tui_add_modal_empty_validation() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();
    let mut app = App::new(&conn, dir.path());

    let invalid_state = AddSnippetModalState {
        title: "".to_string(),
        description: "".to_string(),
        use_case: "".to_string(),
        tags: "".to_string(),
        language: "cpp".to_string(),
        code: "".to_string(),
        focus_idx: 0,
    };

    app.save_new_snippet(&mut conn, invalid_state);
    assert!(app.active_status().unwrap().contains("cannot be empty"));
    assert_eq!(app.all_snippets.len(), 0);
}

#[test]
fn test_tui_edit_modal_save_flow() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();
    let id = add_snippet(&mut conn, dir.path(), "Old Title", "old desc", "old use", "old tag", "old code", Some("cpp"), None).unwrap();

    let mut app = App::new(&conn, dir.path());
    app.open_edit_modal();
    assert!(matches!(app.active_modal, Some(ActiveModal::EditSnippet(_))));

    let edit_state = EditSnippetModalState {
        id: id.clone(),
        title: "Updated Title".to_string(),
        description: "Updated Description".to_string(),
        use_case: "Updated Use Case".to_string(),
        tags: "updated, tags".to_string(),
        language: "rust".to_string(),
        code: "fn updated() {}".to_string(),
        focus_idx: 0,
    };

    app.save_edited_snippet(&mut conn, edit_state);
    assert!(app.active_modal.is_none());

    let snip = get_snippet(&conn, &id).unwrap().unwrap();
    assert_eq!(snip.title, "Updated Title");
    assert_eq!(snip.language, "rust");
    assert_eq!(snip.code, "fn updated() {}");
}

#[test]
fn test_tui_settings_modal_flow() {
    let dir = tempdir().unwrap();
    let conn = init_db(dir.path()).unwrap();
    let mut app = App::new(&conn, dir.path());

    app.open_settings_modal();
    assert!(matches!(app.active_modal, Some(ActiveModal::Settings(_))));

    // Select Catppuccin Mocha (index 1) and Python (index 2)
    let settings_state = SettingsModalState {
        theme_idx: 1, // Catppuccin Mocha
        lang_idx: 2,  // python
        focus_idx: 0,
    };

    app.save_settings(settings_state);
    assert!(app.active_modal.is_none());
    assert_eq!(app.theme.name, "Catppuccin Mocha");
    assert_eq!(app.config.default_language, "python");

    // Verify persisted config on disk
    let saved_cfg = load_config(dir.path());
    assert_eq!(saved_cfg.display.theme, "catppuccin-mocha");
    assert_eq!(saved_cfg.default_language, "python");
}

#[test]
fn test_tui_delete_modal_flow() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();
    let id = add_snippet(&mut conn, dir.path(), "To Delete", "", "", "", "del()", Some("cpp"), None).unwrap();

    let mut app = App::new(&conn, dir.path());
    assert_eq!(app.all_snippets.len(), 1);

    app.confirm_delete_current();
    assert!(matches!(app.active_modal, Some(ActiveModal::DeleteConfirm(_))));

    app.execute_delete(&conn, &id);
    assert_eq!(app.all_snippets.len(), 0);
    assert!(app.active_status().unwrap().contains("Deleted snippet"));
}

#[test]
fn test_tui_panel_resizing_bounds() {
    let dir = tempdir().unwrap();
    let conn = init_db(dir.path()).unwrap();
    let mut app = App::new(&conn, dir.path());

    app.left_panel_percent = 65;
    app.increase_panel_width();
    assert_eq!(app.left_panel_percent, 70);
    app.increase_panel_width();
    assert_eq!(app.left_panel_percent, 70); // Max bound

    app.left_panel_percent = 25;
    app.decrease_panel_width();
    assert_eq!(app.left_panel_percent, 20);
    app.decrease_panel_width();
    assert_eq!(app.left_panel_percent, 20); // Min bound
}

#[test]
fn test_tui_language_badges_all() {
    let langs = ["cpp", "c", "python", "rust", "tex", "javascript", "typescript", "go", "java", "lua", "bash", "markdown", "sql", "text"];
    for lang in langs {
        let badge = get_language_badge(lang);
        assert!(!badge.name.is_empty());
        assert!(!badge.icon.is_empty());
    }
}

#[test]
fn test_tui_syntect_syntax_highlighter_multiple_languages() {
    let highlighter = SyntaxHighlighter::new();

    let cpp_code = "#include <iostream>\nint main() { return 0; }";
    let cpp_lines = highlighter.highlight(cpp_code, "cpp");
    assert_eq!(cpp_lines.len(), 2);

    let py_code = "def solve():\n    print('ans')";
    let py_lines = highlighter.highlight(py_code, "python");
    assert_eq!(py_lines.len(), 2);

    let rs_code = "fn main() {\n    let x = 42;\n}";
    let rs_lines = highlighter.highlight(rs_code, "rust");
    assert_eq!(rs_lines.len(), 3);
}

#[test]
fn test_tui_all_themes_present() {
    assert_eq!(THEMES.len(), 6);
    assert_eq!(get_theme_by_name("cosmere").name, "Cosmere");
    assert_eq!(get_theme_by_name("tokyo-night").name, "Tokyo Night");
    assert_eq!(get_theme_by_name("nord").name, "Nord");
    assert_eq!(get_theme_by_name("gruvbox").name, "Gruvbox");
    assert_eq!(get_theme_by_name("dracula").name, "Dracula");
}
