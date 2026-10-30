//! Interactive Ratatui Terminal User Interface (TUI) for CPKB.

pub mod app;
pub mod badges;
pub mod syntax;
pub mod theme;
pub mod ui;

use std::io::{self, stdout};
use std::path::Path;
use std::time::Duration;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rusqlite::Connection;

use crate::cli::editor::{open_snippet_creator, open_snippet_editor};
use crate::db::snippets::{add_snippet, update_snippet};
use crate::tui::app::{ActiveModal, AddSnippetModalState, App, EditSnippetModalState, SUPPORTED_LANGUAGES};
use crate::tui::theme::get_available_themes;
use crate::tui::ui::render_ui;

/// Start and run the CPKB TUI application loop.
pub fn run_tui(conn: &mut Connection, app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(conn, app_dir);

    let res = run_app(&mut terminal, &mut app, conn);

    // Restore terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("TUI runtime error: {err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    conn: &mut Connection,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| render_ui(f, app))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handle_key_event(terminal, app, conn, key.code, key.modifiers)?;
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => move_tag_selection(app, 1),
                    MouseEventKind::ScrollUp => move_tag_selection(app, -1),
                    _ => {}
                },
                _ => {}
            }
        }
    }
    Ok(())
}

fn move_tag_selection(app: &mut App, delta: isize) {
    if let Some(ActiveModal::TagSelect(mut state)) = app.active_modal.clone() {
        if !state.tags.is_empty() {
            state.selected_idx = if delta > 0 {
                (state.selected_idx + 1).min(state.tags.len() - 1)
            } else {
                state.selected_idx.saturating_sub(1)
            };
            app.active_modal = Some(ActiveModal::TagSelect(state));
        }
    }
}

fn add_text_field(state: &mut AddSnippetModalState, idx: usize) -> Option<&mut String> {
    match idx {
        0 => Some(&mut state.title),
        1 => Some(&mut state.description),
        2 => Some(&mut state.use_case),
        3 => Some(&mut state.tags),
        6 => Some(&mut state.code),
        _ => None,
    }
}

fn edit_text_field(state: &mut EditSnippetModalState, idx: usize) -> Option<&mut String> {
    match idx {
        0 => Some(&mut state.title),
        1 => Some(&mut state.description),
        2 => Some(&mut state.use_case),
        3 => Some(&mut state.tags),
        4 => Some(&mut state.language),
        6 => Some(&mut state.code),
        _ => None,
    }
}

fn handle_cursor_key(value: &mut String, cursor: &mut usize, field_idx: usize, code: KeyCode, modifiers: KeyModifiers) -> bool {
    let multiline = matches!(field_idx, 1 | 2 | 6);
    *cursor = (*cursor).min(value.chars().count());
    match code {
        KeyCode::Char(c) if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            let byte = value.char_indices().nth(*cursor).map(|(i, _)| i).unwrap_or(value.len());
            value.insert(byte, c);
            *cursor += 1;
            true
        }
        KeyCode::Backspace => {
            if *cursor > 0 {
                let start = value.char_indices().nth(*cursor - 1).map(|(i, _)| i).unwrap();
                let end = value.char_indices().nth(*cursor).map(|(i, _)| i).unwrap_or(value.len());
                value.replace_range(start..end, "");
                *cursor -= 1;
            }
            true
        }
        KeyCode::Delete => {
            if let Some(start) = value.char_indices().nth(*cursor).map(|(i, _)| i) {
                let end = value.char_indices().nth(*cursor + 1).map(|(i, _)| i).unwrap_or(value.len());
                value.replace_range(start..end, "");
            }
            true
        }
        KeyCode::Left => { *cursor = cursor.saturating_sub(1); true }
        KeyCode::Right => { *cursor = (*cursor + 1).min(value.chars().count()); true }
        KeyCode::Home => {
            let before = value.chars().take(*cursor).collect::<String>();
            *cursor = before.rfind('\n').map(|idx| before[..idx].chars().count() + 1).unwrap_or(0);
            true
        }
        KeyCode::End => {
            let before = value.chars().take(*cursor).collect::<String>();
            let line_len = value.chars().skip(*cursor).take_while(|c| *c != '\n').count();
            *cursor += line_len;
            if before.is_empty() { *cursor = line_len; }
            true
        }
        KeyCode::Up if multiline => { move_cursor_vertical(value, cursor, -1); true }
        KeyCode::Down if multiline => { move_cursor_vertical(value, cursor, 1); true }
        KeyCode::Enter if multiline => {
            let byte = value.char_indices().nth(*cursor).map(|(i, _)| i).unwrap_or(value.len());
            value.insert(byte, '\n');
            *cursor += 1;
            true
        }
        _ => false,
    }
}

fn move_cursor_vertical(value: &str, cursor: &mut usize, delta: isize) {
    let chars: Vec<char> = value.chars().collect();
    let current = (*cursor).min(chars.len());
    let line_start = chars[..current].iter().rposition(|c| *c == '\n').map(|i| i + 1).unwrap_or(0);
    let column = current - line_start;
    let line_end = chars[current..].iter().position(|c| *c == '\n').map(|i| current + i).unwrap_or(chars.len());
    let (target_start, target_end) = if delta < 0 {
        if line_start == 0 { return; }
        let end = line_start - 1;
        let start = chars[..end].iter().rposition(|c| *c == '\n').map(|i| i + 1).unwrap_or(0);
        (start, end)
    } else {
        if line_end == chars.len() { return; }
        let start = line_end + 1;
        let end = chars[start..].iter().position(|c| *c == '\n').map(|i| start + i).unwrap_or(chars.len());
        (start, end)
    };
    *cursor = target_start + column.min(target_end - target_start);
}

fn handle_key_event<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    conn: &mut Connection,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> io::Result<()> {
    // Universal quit on Ctrl+C
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        app.should_quit = true;
        return Ok(());
    }

    // Modal state handling
    if let Some(modal) = app.active_modal.clone() {
        match modal {
            ActiveModal::Help => match code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter => {
                    app.active_modal = None;
                }
                _ => {}
            },
            ActiveModal::TagSelect(mut state) => match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.active_modal = None;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if !state.tags.is_empty() {
                        state.selected_idx = (state.selected_idx + 1) % state.tags.len();
                        app.active_modal = Some(ActiveModal::TagSelect(state));
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if !state.tags.is_empty() {
                        state.selected_idx = if state.selected_idx == 0 {
                            state.tags.len() - 1
                        } else {
                            state.selected_idx - 1
                        };
                        app.active_modal = Some(ActiveModal::TagSelect(state));
                    }
                }
                KeyCode::Enter => {
                    if !state.tags.is_empty() {
                        let selected_tag = state.tags[state.selected_idx].0.clone();
                        app.apply_tag_filter(Some(selected_tag), conn);
                    }
                }
                KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Backspace | KeyCode::Delete => {
                    app.apply_tag_filter(None, conn);
                }
                _ => {}
            },
            ActiveModal::DeleteConfirm(id) => match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    app.execute_delete(conn, &id);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                    app.active_modal = None;
                }
                _ => {}
            },
            ActiveModal::IdFormatBlocked(_) => match code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => app.open_id_formats_modal(),
                _ => {}
            },
            ActiveModal::IdFormats(mut state) => match code {
                KeyCode::Esc => app.open_settings_modal(),
                KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                    let names: Vec<String> = app.config.snippets.id_formats.keys().cloned().collect();
                    if !names.is_empty() { let idx = state.selected_idx.min(names.len() - 1); app.remove_selected_id_format(conn, &names[idx]); }
                }
                KeyCode::Tab => { state.focus_idx = (state.focus_idx + 1) % 2; app.active_modal = Some(ActiveModal::IdFormats(state)); }
                KeyCode::BackTab => { state.focus_idx = if state.focus_idx == 0 { 1 } else { 0 }; app.active_modal = Some(ActiveModal::IdFormats(state)); }
                KeyCode::Left | KeyCode::Right if state.focus_idx < 2 => { state.focus_idx = if state.focus_idx == 0 { 1 } else { 0 }; app.active_modal = Some(ActiveModal::IdFormats(state)); }
                KeyCode::Up => { state.selected_idx = state.selected_idx.saturating_sub(1); app.active_modal = Some(ActiveModal::IdFormats(state)); }
                KeyCode::Down => { state.selected_idx += 1; app.active_modal = Some(ActiveModal::IdFormats(state)); }
                KeyCode::Enter if state.focus_idx == 0 => { state.focus_idx = 1; app.active_modal = Some(ActiveModal::IdFormats(state)); }
                KeyCode::Enter => app.save_id_format(state),
                KeyCode::Backspace => { if state.focus_idx == 0 { state.name.pop(); } else { state.pattern.pop(); } app.active_modal = Some(ActiveModal::IdFormats(state)); }
                KeyCode::Char(c) if c.is_ascii_alphanumeric() || "_-#.@".contains(c) => { if state.focus_idx == 0 { state.name.push(c); } else { state.pattern.push(c); } app.active_modal = Some(ActiveModal::IdFormats(state)); }
                KeyCode::Delete => {
                    let names: Vec<String> = app.config.snippets.id_formats.keys().cloned().collect();
                    if !names.is_empty() { let idx = state.selected_idx.min(names.len()-1); app.remove_selected_id_format(conn, &names[idx]); }
                }
                _ => {}
            },
            ActiveModal::Settings(mut state) => {
                if (modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('e'))
                    || code == KeyCode::Char('e')
                    || code == KeyCode::Char('E')
                {
                    launch_editor_config(terminal, app)?;
                    return Ok(());
                }

                if code == KeyCode::Char('c') || code == KeyCode::Char('C') {
                    app.open_custom_theme_modal();
                    return Ok(());
                }

                match code {
                    KeyCode::Esc => {
                        app.active_modal = None;
                    }
                    KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
                        state.focus_idx = (state.focus_idx + 1) % 7;
                        app.active_modal = Some(ActiveModal::Settings(state));
                    }
                    KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
                        state.focus_idx = if state.focus_idx == 0 { 6 } else { state.focus_idx - 1 };
                        app.active_modal = Some(ActiveModal::Settings(state));
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        let available_themes = get_available_themes(Some(&app.config.display.custom_theme));
                        match state.focus_idx {
                            0 => {
                                state.theme_idx = if state.theme_idx == 0 {
                                    available_themes.len() - 1
                                } else {
                                    state.theme_idx - 1
                                };
                                app.theme = available_themes[state.theme_idx].1.clone(); // Live theme preview
                            }
                            1 => {
                                state.lang_idx = if state.lang_idx == 0 {
                                    SUPPORTED_LANGUAGES.len() - 1
                                } else {
                                    state.lang_idx - 1
                                };
                            }
                            2 => {
                                state.sort_idx = if state.sort_idx == 0 { 2 } else { state.sort_idx - 1 };
                            }
                            3 => {
                                state.layout_idx = if state.layout_idx == 0 { 1 } else { 0 };
                            }
                            4 => {
                                state.border_idx = if state.border_idx == 0 { 3 } else { state.border_idx - 1 };
                            }
                            6 => { app.open_id_formats_modal(); return Ok(()); }
                            _ => {}
                        }
                        app.active_modal = Some(ActiveModal::Settings(state));
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        let available_themes = get_available_themes(Some(&app.config.display.custom_theme));
                        match state.focus_idx {
                            0 => {
                                state.theme_idx = (state.theme_idx + 1) % available_themes.len();
                                app.theme = available_themes[state.theme_idx].1.clone(); // Live theme preview
                            }
                            1 => {
                                state.lang_idx = (state.lang_idx + 1) % SUPPORTED_LANGUAGES.len();
                            }
                            2 => {
                                state.sort_idx = (state.sort_idx + 1) % 3;
                            }
                            3 => {
                                state.layout_idx = (state.layout_idx + 1) % 2;
                            }
                            4 => {
                                state.border_idx = (state.border_idx + 1) % 4;
                            }
                            6 => { app.open_id_formats_modal(); return Ok(()); }
                            _ => {}
                        }
                        app.active_modal = Some(ActiveModal::Settings(state));
                    }
                    KeyCode::Enter => {
                        if state.focus_idx == 5 {
                            app.open_custom_theme_modal();
                        } else if state.focus_idx == 6 {
                            app.open_id_formats_modal();
                        } else {
                            app.save_settings(state);
                        }
                    }
                    _ => {}
                }
            }
            ActiveModal::CustomTheme(mut state) => {
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('s') {
                    app.save_custom_theme(state);
                    return Ok(());
                }
                match code {
                    KeyCode::Esc => {
                        app.active_modal = None;
                    }
                    KeyCode::Tab | KeyCode::Down => {
                        state.focus_idx = (state.focus_idx + 1) % 11;
                        app.active_modal = Some(ActiveModal::CustomTheme(state));
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        state.focus_idx = if state.focus_idx == 0 { 10 } else { state.focus_idx - 1 };
                        app.active_modal = Some(ActiveModal::CustomTheme(state));
                    }
                    KeyCode::Enter => {
                        if state.focus_idx == 10 {
                            app.save_custom_theme(state);
                        } else {
                            state.focus_idx += 1;
                            app.active_modal = Some(ActiveModal::CustomTheme(state));
                        }
                    }
                    KeyCode::Backspace => {
                        state.get_value_mut(state.focus_idx).pop();
                        app.active_modal = Some(ActiveModal::CustomTheme(state));
                    }
                    KeyCode::Char(c) => {
                        if c.is_alphanumeric() || c == '#' || c == '-' || c == '_' {
                            state.get_value_mut(state.focus_idx).push(c);
                            app.active_modal = Some(ActiveModal::CustomTheme(state));
                        }
                    }
                    _ => {}
                }
            }
            ActiveModal::AddSnippet(mut state) => {
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('e') {
                    // Launch $EDITOR for new snippet
                    launch_editor_add(terminal, app, conn)?;
                    return Ok(());
                }
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('s') {
                    app.save_new_snippet(conn, state);
                    return Ok(());
                }
                let field_idx = state.focus_idx;
                if let Some(field) = add_text_field(&mut state, field_idx) {
                    if handle_cursor_key(field, &mut app.form_cursors[field_idx], field_idx, code, modifiers) {
                        if field_idx == 3 { app.tag_suggestion_idx = 0; }
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                        return Ok(());
                    }
                }

                match code {
                    KeyCode::Esc => {
                        app.active_modal = None;
                    }
                    KeyCode::Up if state.focus_idx == 3 => {
                        app.tag_suggestion_idx = app.tag_suggestion_idx.saturating_sub(1);
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    KeyCode::Down if state.focus_idx == 3 && !app.tag_suggestions(&state.tags).is_empty() => {
                        let count = app.tag_suggestions(&state.tags).len();
                        if count > 0 { app.tag_suggestion_idx = (app.tag_suggestion_idx + 1) % count; }
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    KeyCode::Enter if state.focus_idx == 3 => {
                        app.complete_tag(&mut state.tags);
                        app.form_cursors[3] = state.tags.chars().count();
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    KeyCode::Enter => { state.focus_idx = (state.focus_idx + 1) % 7; app.active_modal = Some(ActiveModal::AddSnippet(state)); }
                    KeyCode::Tab => {
                        state.focus_idx = (state.focus_idx + 1) % 7;
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    KeyCode::Down => { state.focus_idx = (state.focus_idx + 1) % 7; app.active_modal = Some(ActiveModal::AddSnippet(state)); }
                    KeyCode::BackTab | KeyCode::Up => {
                        state.focus_idx = if state.focus_idx == 0 { 6 } else { state.focus_idx - 1 };
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    KeyCode::Left | KeyCode::Right if state.focus_idx == 5 => {
                        let available_formats: Vec<String> = app.config.snippets.id_formats.keys().cloned().collect();
                        if !available_formats.is_empty() {
                            let current_idx = available_formats.iter().position(|k| k == &state.id_format).unwrap_or(0);
                            let next_idx = if code == KeyCode::Right {
                                (current_idx + 1) % available_formats.len()
                            } else {
                                if current_idx == 0 { available_formats.len() - 1 } else { current_idx - 1 }
                            };
                            state.id_format = available_formats[next_idx].clone();
                        }
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    KeyCode::Left | KeyCode::Right if state.focus_idx == 4 => {
                        let current = SUPPORTED_LANGUAGES.iter().position(|l| *l == state.language.to_lowercase()).unwrap_or(0);
                        let next = if code == KeyCode::Right { (current + 1) % SUPPORTED_LANGUAGES.len() } else if current == 0 { SUPPORTED_LANGUAGES.len()-1 } else { current-1 };
                        state.language = SUPPORTED_LANGUAGES[next].to_string();
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    _ => {}
                }
            }
            ActiveModal::EditSnippet(mut state) => {
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('e') {
                    // Launch $EDITOR for editing snippet
                    launch_editor_edit(terminal, app, conn)?;
                    return Ok(());
                }
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('s') {
                    app.save_edited_snippet(conn, state);
                    return Ok(());
                }
                let field_idx = state.focus_idx;
                if let Some(field) = edit_text_field(&mut state, field_idx) {
                    if handle_cursor_key(field, &mut app.form_cursors[field_idx], field_idx, code, modifiers) {
                        if field_idx == 3 { app.tag_suggestion_idx = 0; }
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                        return Ok(());
                    }
                }

                match code {
                    KeyCode::Esc => {
                        app.active_modal = None;
                    }
                    KeyCode::Up if state.focus_idx == 3 => {
                        app.tag_suggestion_idx = app.tag_suggestion_idx.saturating_sub(1);
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                    }
                    KeyCode::Down if state.focus_idx == 3 && !app.tag_suggestions(&state.tags).is_empty() => {
                        let count = app.tag_suggestions(&state.tags).len();
                        if count > 0 { app.tag_suggestion_idx = (app.tag_suggestion_idx + 1) % count; }
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                    }
                    KeyCode::Enter if state.focus_idx == 3 => {
                        app.complete_tag(&mut state.tags);
                        app.form_cursors[3] = state.tags.chars().count();
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                    }
                    KeyCode::Enter => { state.focus_idx = (state.focus_idx + 1) % 7; app.active_modal = Some(ActiveModal::EditSnippet(state)); }
                    KeyCode::Tab => {
                        state.focus_idx = (state.focus_idx + 1) % 7;
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                    }
                    KeyCode::Down => { state.focus_idx = (state.focus_idx + 1) % 7; app.active_modal = Some(ActiveModal::EditSnippet(state)); }
                    KeyCode::BackTab | KeyCode::Up => {
                        state.focus_idx = if state.focus_idx == 0 { 6 } else { state.focus_idx - 1 };
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                    }
                    KeyCode::Left | KeyCode::Right if state.focus_idx == 5 => {
                        let available_formats: Vec<String> = app.config.snippets.id_formats.keys().cloned().collect();
                        if !available_formats.is_empty() {
                            let current_idx = available_formats.iter().position(|k| k == &state.id_format).unwrap_or(0);
                            let next_idx = if code == KeyCode::Right {
                                (current_idx + 1) % available_formats.len()
                            } else {
                                if current_idx == 0 { available_formats.len() - 1 } else { current_idx - 1 }
                            };
                            state.id_format = available_formats[next_idx].clone();
                        }
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                    }
                    _ => {}
                }
            }
        }
        return Ok(());
    }

    // Search input mode
    if app.search_active {
        match code {
            KeyCode::Esc => {
                app.search_active = false;
                app.search_query.clear();
                app.apply_filter();
                app.reload_active_snippet(conn);
            }
            KeyCode::Enter => {
                app.search_active = false;
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.apply_filter();
                app.reload_active_snippet(conn);
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.apply_filter();
                app.reload_active_snippet(conn);
            }
            _ => {}
        }
        return Ok(());
    }

    // Normal navigation mode
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('e') {
        launch_editor_edit(terminal, app, conn)?;
        return Ok(());
    }

    match code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.next(conn);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.previous(conn);
        }
        KeyCode::Char('g') | KeyCode::Home => {
            app.select_first(conn);
        }
        KeyCode::Char('G') | KeyCode::End => {
            app.select_last(conn);
        }
        KeyCode::Char('/') => {
            app.search_active = true;
        }
        KeyCode::Char('a') => {
            app.open_add_modal();
        }
        KeyCode::Char('e') | KeyCode::Enter => {
            app.open_edit_modal();
        }
        KeyCode::Char('s') | KeyCode::Char(',') => {
            app.open_settings_modal();
        }
        KeyCode::Char('o') | KeyCode::Char('O') => {
            app.toggle_sort(conn);
        }
        KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.copy_current_code();
        }
        KeyCode::Char('L') => {
            app.cycle_language_filter(conn);
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            app.open_tag_modal();
        }
        KeyCode::Char('d') => {
            app.confirm_delete_current();
        }
        KeyCode::Char('[') => {
            app.decrease_panel_width();
        }
        KeyCode::Char(']') => {
            app.increase_panel_width();
        }
        KeyCode::Char('?') => {
            app.active_modal = Some(ActiveModal::Help);
        }
        KeyCode::Esc => {
            app.clear_all_filters(conn);
        }
        _ => {}
    }

    Ok(())
}

fn launch_editor_add<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    conn: &mut Connection,
) -> io::Result<()> {
    // Suspend TUI
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let default_lang = if app.config.default_language.is_empty() { "cpp" } else { &app.config.default_language };
    if let Ok(Some(edited)) = open_snippet_creator(&app.app_dir, default_lang) {
        if let Ok(id) = add_snippet(
            conn,
            &app.app_dir,
            &edited.title,
            &edited.description,
            &edited.use_case,
            &edited.tags,
            &edited.code,
            Some(&edited.language),
            None,
        ) {
            app.set_status_message(format!("Created snippet {} via $EDITOR!", id));
            app.reload_snippets(conn);
        }
    }

    // Resume TUI
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    terminal.clear()?;
    app.active_modal = None;

    Ok(())
}

fn launch_editor_edit<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    conn: &mut Connection,
) -> io::Result<()> {
    if let Some(ref snip) = app.active_snippet {
        let snip_clone = snip.clone();

        // Suspend TUI
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        if let Ok(Some(edited)) = open_snippet_editor(&app.app_dir, &snip_clone) {
            if let Ok(true) = update_snippet(
                conn,
                &snip_clone.id,
                &edited.title,
                &edited.description,
                &edited.use_case,
                &edited.tags,
                &edited.code,
                &edited.language,
            ) {
                app.set_status_message(format!("Updated snippet {} via $EDITOR!", snip_clone.id));
                app.reload_snippets(conn);
            }
        }

        // Resume TUI
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        terminal.clear()?;
        app.active_modal = None;
    }

    Ok(())
}

fn launch_editor_config<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let config_path = app.app_dir.join("config.json");
    if !config_path.exists() {
        let _ = crate::config::save_config(&app.app_dir, &app.config);
    }

    // Suspend TUI
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let editor_cmd = crate::cli::editor::configured_editor_command(&app.app_dir);
    let _ = std::process::Command::new(&editor_cmd[0])
        .args(&editor_cmd[1..])
        .arg(&config_path)
        .status();

    // Reload config & theme
    app.config = crate::config::load_config(&app.app_dir);
    app.theme = crate::tui::theme::get_theme_by_name(&app.config.display.theme);
    app.set_status_message("Configuration reloaded from config.json!".to_string());

    // Resume TUI
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    terminal.clear()?;
    app.active_modal = None;

    Ok(())
}
