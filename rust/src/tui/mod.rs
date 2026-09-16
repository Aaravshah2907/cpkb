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

use crate::tui::app::{ActiveModal, App};
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
                    handle_key_event(app, conn, key.code, key.modifiers);
                }
            }
        }
    }
    Ok(())
}

fn handle_key_event(app: &mut App, conn: &mut Connection, code: KeyCode, modifiers: KeyModifiers) {
    // Universal quit on Ctrl+C
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    // Modal state handling
    if let Some(ref modal) = app.active_modal.clone() {
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
        }
        return;
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
        return;
    }

    // Normal navigation mode
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
}
