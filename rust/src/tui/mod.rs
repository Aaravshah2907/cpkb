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
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rusqlite::Connection;

use crate::cli::editor::{open_snippet_creator, open_snippet_editor};
use crate::db::snippets::{add_snippet, update_snippet};
use crate::tui::app::{ActiveModal, App, SUPPORTED_LANGUAGES};
use crate::tui::theme::get_available_themes;
use crate::tui::ui::render_ui;

/// Start and run the CPKB TUI application loop.
pub fn run_tui(conn: &mut Connection, app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(conn, app_dir);

    let res = run_app(&mut terminal, &mut app, conn);

    // Restore terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
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
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(terminal, app, conn, key.code, key.modifiers)?;
                }
            }
        }
    }
    Ok(())
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
            ActiveModal::DeleteConfirm(id) => match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    app.execute_delete(conn, &id);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                    app.active_modal = None;
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

                match code {
                    KeyCode::Esc => {
                        app.active_modal = None;
                    }
                    KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
                        state.focus_idx = (state.focus_idx + 1) % 2;
                        app.active_modal = Some(ActiveModal::Settings(state));
                    }
                    KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
                        state.focus_idx = if state.focus_idx == 0 { 1 } else { 0 };
                        app.active_modal = Some(ActiveModal::Settings(state));
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        let available_themes = get_available_themes(Some(&app.config.display.custom_theme));
                        if state.focus_idx == 0 {
                            state.theme_idx = if state.theme_idx == 0 {
                                available_themes.len() - 1
                            } else {
                                state.theme_idx - 1
                            };
                            app.theme = available_themes[state.theme_idx].1.clone(); // Live theme preview
                        } else {
                            state.lang_idx = if state.lang_idx == 0 {
                                SUPPORTED_LANGUAGES.len() - 1
                            } else {
                                state.lang_idx - 1
                            };
                        }
                        app.active_modal = Some(ActiveModal::Settings(state));
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        let available_themes = get_available_themes(Some(&app.config.display.custom_theme));
                        if state.focus_idx == 0 {
                            state.theme_idx = (state.theme_idx + 1) % available_themes.len();
                            app.theme = available_themes[state.theme_idx].1.clone(); // Live theme preview
                        } else {
                            state.lang_idx = (state.lang_idx + 1) % SUPPORTED_LANGUAGES.len();
                        }
                        app.active_modal = Some(ActiveModal::Settings(state));
                    }
                    KeyCode::Enter => {
                        app.save_settings(state);
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
                if (modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('s')) || (code == KeyCode::Enter && state.focus_idx == 5) {
                    app.save_new_snippet(conn, state);
                    return Ok(());
                }

                match code {
                    KeyCode::Esc => {
                        app.active_modal = None;
                    }
                    KeyCode::Tab | KeyCode::Down => {
                        state.focus_idx = (state.focus_idx + 1) % 6;
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        state.focus_idx = if state.focus_idx == 0 { 5 } else { state.focus_idx - 1 };
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    KeyCode::Backspace => {
                        let target = match state.focus_idx {
                            0 => &mut state.title,
                            1 => &mut state.description,
                            2 => &mut state.use_case,
                            3 => &mut state.tags,
                            4 => &mut state.language,
                            _ => &mut state.code,
                        };
                        target.pop();
                        app.active_modal = Some(ActiveModal::AddSnippet(state));
                    }
                    KeyCode::Char(c) => {
                        let target = match state.focus_idx {
                            0 => &mut state.title,
                            1 => &mut state.description,
                            2 => &mut state.use_case,
                            3 => &mut state.tags,
                            4 => &mut state.language,
                            _ => &mut state.code,
                        };
                        target.push(c);
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
                if (modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('s')) || (code == KeyCode::Enter && state.focus_idx == 5) {
                    app.save_edited_snippet(conn, state);
                    return Ok(());
                }

                match code {
                    KeyCode::Esc => {
                        app.active_modal = None;
                    }
                    KeyCode::Tab | KeyCode::Down => {
                        state.focus_idx = (state.focus_idx + 1) % 6;
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        state.focus_idx = if state.focus_idx == 0 { 5 } else { state.focus_idx - 1 };
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                    }
                    KeyCode::Backspace => {
                        let target = match state.focus_idx {
                            0 => &mut state.title,
                            1 => &mut state.description,
                            2 => &mut state.use_case,
                            3 => &mut state.tags,
                            4 => &mut state.language,
                            _ => &mut state.code,
                        };
                        target.pop();
                        app.active_modal = Some(ActiveModal::EditSnippet(state));
                    }
                    KeyCode::Char(c) => {
                        let target = match state.focus_idx {
                            0 => &mut state.title,
                            1 => &mut state.description,
                            2 => &mut state.use_case,
                            3 => &mut state.tags,
                            4 => &mut state.language,
                            _ => &mut state.code,
                        };
                        target.push(c);
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
        KeyCode::Char('c') => {
            app.copy_current_code();
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
            if !app.search_query.is_empty() {
                app.search_query.clear();
                app.apply_filter();
                app.reload_active_snippet(conn);
            }
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
