//! Main Ratatui UI rendering logic for CPKB.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap,
};
use ratatui::Frame;

use crate::tui::app::{
    ActiveModal, AddSnippetModalState, App, EditSnippetModalState, SettingsModalState, SUPPORTED_LANGUAGES,
};
use crate::tui::badges::get_language_badge;
use crate::tui::theme::THEMES;

pub fn render_ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Background base
    let bg_block = Block::default().style(app.theme.style_base());
    f.render_widget(bg_block, size);

    // Vertical split: Header (3), Main Body (Min 10), Footer (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(size);

    render_header(f, app, chunks[0]);
    render_body(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);

    // Modal overlays
    if let Some(ref modal) = app.active_modal.clone() {
        match modal {
            ActiveModal::Help => render_help_modal(f, app, size),
            ActiveModal::DeleteConfirm(id) => render_delete_modal(f, app, size, id),
            ActiveModal::AddSnippet(state) => render_add_modal(f, app, size, state),
            ActiveModal::EditSnippet(state) => render_edit_modal(f, app, size, state),
            ActiveModal::Settings(state) => render_settings_modal(f, app, size, state),
        }
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    // Aligned to exact same column widths as main body
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(app.left_panel_percent),
            Constraint::Percentage(100 - app.left_panel_percent),
        ])
        .split(area);

    // Brand Block (Left)
    let brand_lines = Line::from(vec![
        Span::styled(" CPKB ", Style::default().bg(app.theme.primary).fg(app.theme.background).add_modifier(Modifier::BOLD)),
        Span::styled(" Knowledge Base ", Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
        Span::styled(format!("[{}]", app.all_snippets.len()), Style::default().fg(app.theme.secondary)),
    ]);
    let brand_p = Paragraph::new(brand_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(app.theme.style_border(false)),
    );
    f.render_widget(brand_p, header_chunks[0]);

    // Search / Filter Block (Right)
    let search_content = if app.search_active {
        Line::from(vec![
            Span::styled(" 🔍 Search: ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(&app.search_query, Style::default().fg(app.theme.text)),
            Span::styled("█", Style::default().fg(app.theme.primary)),
        ])
    } else if !app.search_query.is_empty() {
        Line::from(vec![
            Span::styled(" 🔍 Filter: ", Style::default().fg(app.theme.secondary).add_modifier(Modifier::BOLD)),
            Span::styled(&app.search_query, Style::default().fg(app.theme.text)),
            Span::styled(format!(" ({} matches)", app.filtered_indices.len()), Style::default().fg(app.theme.text_dim)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" Press ", Style::default().fg(app.theme.text_dim)),
            Span::styled("/", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(" to search snippets... ", Style::default().fg(app.theme.text_dim)),
            Span::styled(format!("({} total)", app.all_snippets.len()), Style::default().fg(app.theme.text_dim)),
        ])
    };

    let search_p = Paragraph::new(search_content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(app.theme.style_border(app.search_active)),
    );
    f.render_widget(search_p, header_chunks[1]);
}

fn render_body(f: &mut Frame, app: &mut App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(app.left_panel_percent),
            Constraint::Percentage(100 - app.left_panel_percent),
        ])
        .split(area);

    render_snippet_list(f, app, main_chunks[0]);
    render_snippet_detail(f, app, main_chunks[1]);
}

fn render_snippet_list(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = !app.search_active && app.active_modal.is_none();
    let border_style = app.theme.style_border(is_focused);

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(disp_idx, &actual_idx)| {
            let snip = &app.all_snippets[actual_idx];
            let is_selected = disp_idx == app.selected_index;

            let prefix = if is_selected { "▎ " } else { "  " };
            let title_style = if is_selected {
                Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text)
            };

            let id_style = if is_selected {
                Style::default().fg(app.theme.secondary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text_dim)
            };

            let line1 = Line::from(vec![
                Span::styled(prefix, Style::default().fg(app.theme.primary)),
                Span::styled(format!("{:<10}", snip.id), id_style),
                Span::styled(&snip.title, title_style),
            ]);

            let tag_str = if snip.tags.is_empty() {
                "—".to_string()
            } else {
                snip.tags.clone()
            };

            let line2 = Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("Tags: {}", tag_str), Style::default().fg(app.theme.text_dim)),
            ]);

            let item_style = if is_selected {
                app.theme.style_surface()
            } else {
                Style::default()
            };

            ListItem::new(vec![line1, line2]).style(item_style)
        })
        .collect();

    let list_title = format!(" Snippets ({}) ", app.filtered_indices.len());
    let list_widget = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(list_title, Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)))
                .border_style(border_style),
        )
        .highlight_style(app.theme.style_selected());

    f.render_stateful_widget(list_widget, area, &mut app.list_state);
}

fn render_snippet_detail(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ref snip) = app.active_snippet {
        let badge = get_language_badge(&snip.language);

        let detail_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(7), Constraint::Min(8)])
            .split(area);

        // Metadata Header
        let meta_lines = vec![
            Line::from(vec![
                Span::styled(format!(" {} ", snip.id), Style::default().bg(app.theme.primary).fg(app.theme.background).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(&snip.title, Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
                Span::raw("   "),
                Span::styled(format!(" {} {} ", badge.icon, badge.name), Style::default().bg(badge.color).fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(app.theme.text_dim).add_modifier(Modifier::BOLD)),
                Span::styled(if snip.description.is_empty() { "—" } else { &snip.description }, Style::default().fg(app.theme.text)),
            ]),
            Line::from(vec![
                Span::styled("Use Case:    ", Style::default().fg(app.theme.text_dim).add_modifier(Modifier::BOLD)),
                Span::styled(if snip.use_case.is_empty() { "—" } else { &snip.use_case }, Style::default().fg(app.theme.text)),
            ]),
            Line::from(vec![
                Span::styled("Tags:        ", Style::default().fg(app.theme.text_dim).add_modifier(Modifier::BOLD)),
                Span::styled(if snip.tags.is_empty() { "—" } else { &snip.tags }, Style::default().fg(app.theme.accent)),
            ]),
        ];

        let meta_p = Paragraph::new(meta_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Details ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)))
                    .border_style(app.theme.style_border(false)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(meta_p, detail_chunks[0]);

        // Highlighted Code Pane
        let highlighted_lines = app.highlighter.highlight(&snip.code, &snip.language);
        let code_title = format!(" Code Preview ({}) ", badge.name);
        let code_p = Paragraph::new(highlighted_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(code_title, Style::default().fg(badge.color).add_modifier(Modifier::BOLD)))
                    .border_style(app.theme.style_border(false)),
            )
            .style(app.theme.style_surface());

        f.render_widget(code_p, detail_chunks[1]);
    } else {
        let empty_p = Paragraph::new("No snippet selected")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(app.theme.style_border(false)),
            );
        f.render_widget(empty_p, area);
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    if let Some(status) = app.active_status() {
        let status_p = Paragraph::new(Line::from(vec![
            Span::styled(" ✔ ", Style::default().fg(app.theme.success).add_modifier(Modifier::BOLD)),
            Span::styled(status, Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
        ]));
        f.render_widget(status_p, area);
    } else {
        let keys = Line::from(vec![
            Span::styled(" a ", Style::default().bg(app.theme.surface).fg(app.theme.success).add_modifier(Modifier::BOLD)),
            Span::raw(" Add  "),
            Span::styled(" e ", Style::default().bg(app.theme.surface).fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Edit  "),
            Span::styled(" d ", Style::default().bg(app.theme.surface).fg(app.theme.error).add_modifier(Modifier::BOLD)),
            Span::raw(" Del  "),
            Span::styled(" c ", Style::default().bg(app.theme.surface).fg(app.theme.secondary).add_modifier(Modifier::BOLD)),
            Span::raw(" Copy  "),
            Span::styled(" s ", Style::default().bg(app.theme.surface).fg(app.theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" Config  "),
            Span::styled(" / ", Style::default().bg(app.theme.surface).fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Search  "),
            Span::styled(" ? ", Style::default().bg(app.theme.surface).fg(app.theme.text_dim).add_modifier(Modifier::BOLD)),
            Span::raw(" Help  "),
            Span::styled(" q ", Style::default().bg(app.theme.surface).fg(app.theme.text_dim).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ]);
        let footer_p = Paragraph::new(keys).style(app.theme.style_footer());
        f.render_widget(footer_p, area);
    }
}

fn render_add_modal(f: &mut Frame, app: &App, area: Rect, state: &AddSnippetModalState) {
    let modal_area = centered_rect(75, 75, area);
    f.render_widget(Clear, modal_area);

    let fields = [
        ("Title", &state.title),
        ("Description", &state.description),
        ("Use Case", &state.use_case),
        ("Tags (comma separated)", &state.tags),
        ("Language (cpp, rust, python...)", &state.language),
        ("Code", &state.code),
    ];

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Add New Snippet", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("  (Press Tab to navigate fields, Ctrl+e for $EDITOR, Ctrl+s to Save, Esc to Cancel)", Style::default().fg(app.theme.text_dim)),
        ]),
        Line::from(""),
    ];

    for (idx, (label, val)) in fields.iter().enumerate() {
        let is_focused = state.focus_idx == idx;
        let prefix = if is_focused { "▶ " } else { "  " };
        let label_style = if is_focused {
            Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text_dim)
        };
        let val_style = if is_focused {
            Style::default().fg(app.theme.text).bg(app.theme.background)
        } else {
            Style::default().fg(app.theme.text)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(app.theme.primary)),
            Span::styled(format!("{:<26}", label), label_style),
            Span::styled(if val.is_empty() { if is_focused { "█" } else { "—" } } else { val }, val_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" [Ctrl+s / Enter] ", Style::default().bg(app.theme.success).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" Save Snippet     "),
        Span::styled(" [Ctrl+e] ", Style::default().bg(app.theme.primary).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" Open in $EDITOR     "),
        Span::styled(" [Esc] ", Style::default().bg(app.theme.surface).fg(app.theme.text).add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel"),
    ]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Create Snippet ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)))
                .border_style(app.theme.style_border(true))
                .style(app.theme.style_surface()),
        );

    f.render_widget(p, modal_area);
}

fn render_edit_modal(f: &mut Frame, app: &App, area: Rect, state: &EditSnippetModalState) {
    let modal_area = centered_rect(75, 75, area);
    f.render_widget(Clear, modal_area);

    let fields = [
        ("Title", &state.title),
        ("Description", &state.description),
        ("Use Case", &state.use_case),
        ("Tags (comma separated)", &state.tags),
        ("Language (cpp, rust, python...)", &state.language),
        ("Code", &state.code),
    ];

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("Editing Snippet: {}", state.id), Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("  (Press Tab to navigate fields, Ctrl+e for $EDITOR, Ctrl+s to Save)", Style::default().fg(app.theme.text_dim)),
        ]),
        Line::from(""),
    ];

    for (idx, (label, val)) in fields.iter().enumerate() {
        let is_focused = state.focus_idx == idx;
        let prefix = if is_focused { "▶ " } else { "  " };
        let label_style = if is_focused {
            Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text_dim)
        };
        let val_style = if is_focused {
            Style::default().fg(app.theme.text).bg(app.theme.background)
        } else {
            Style::default().fg(app.theme.text)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(app.theme.primary)),
            Span::styled(format!("{:<26}", label), label_style),
            Span::styled(if val.is_empty() { if is_focused { "█" } else { "—" } } else { val }, val_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" [Ctrl+s / Enter] ", Style::default().bg(app.theme.success).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" Save Changes     "),
        Span::styled(" [Ctrl+e] ", Style::default().bg(app.theme.primary).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" Open in $EDITOR     "),
        Span::styled(" [Esc] ", Style::default().bg(app.theme.surface).fg(app.theme.text).add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel"),
    ]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(format!(" Edit Snippet: {} ", state.id), Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)))
                .border_style(app.theme.style_border(true))
                .style(app.theme.style_surface()),
        );

    f.render_widget(p, modal_area);
}

fn render_settings_modal(f: &mut Frame, app: &App, area: Rect, state: &SettingsModalState) {
    let modal_area = centered_rect(65, 55, area);
    f.render_widget(Clear, modal_area);

    let current_theme = THEMES[state.theme_idx].0;
    let current_lang = SUPPORTED_LANGUAGES[state.lang_idx];

    let theme_focused = state.focus_idx == 0;
    let lang_focused = state.focus_idx == 1;

    let lines = vec![
        Line::from(vec![
            Span::styled("CPKB Settings & Preferences", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(if theme_focused { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled("Color Theme:       ", if theme_focused { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) }),
            Span::styled(" ◀ ", Style::default().fg(app.theme.secondary)),
            Span::styled(format!("{:<20}", current_theme), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" ▶ ", Style::default().fg(app.theme.secondary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(if lang_focused { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled("Default Language:  ", if lang_focused { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) }),
            Span::styled(" ◀ ", Style::default().fg(app.theme.secondary)),
            Span::styled(format!("{:<20}", current_lang), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" ▶ ", Style::default().fg(app.theme.secondary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Config Storage:    ", Style::default().fg(app.theme.text_dim)),
            Span::styled(app.app_dir.join("config.json").display().to_string(), Style::default().fg(app.theme.text)),
        ]),
        Line::from(vec![
            Span::styled("Database Storage:  ", Style::default().fg(app.theme.text_dim)),
            Span::styled(app.app_dir.join("snippets.db").display().to_string(), Style::default().fg(app.theme.text)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [← / →] ", Style::default().bg(app.theme.surface).fg(app.theme.secondary).add_modifier(Modifier::BOLD)),
            Span::raw(" Change value  "),
            Span::styled(" [Tab] ", Style::default().bg(app.theme.surface).fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Switch row  "),
            Span::styled(" [Enter / Ctrl+s] ", Style::default().bg(app.theme.success).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Save & Apply  "),
            Span::styled(" [Esc] ", Style::default().bg(app.theme.surface).fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
        ]),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Settings ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)))
                .border_style(app.theme.style_border(true))
                .style(app.theme.style_surface()),
        );

    f.render_widget(p, modal_area);
}

fn render_help_modal(f: &mut Frame, app: &App, area: Rect) {
    let modal_area = centered_rect(65, 65, area);
    f.render_widget(Clear, modal_area);

    let help_text = vec![
        Line::from(vec![Span::styled("CPKB Keybindings & Commands", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(vec![Span::styled("  j / ↓         ", Style::default().fg(app.theme.secondary)), Span::raw("Select next snippet")]),
        Line::from(vec![Span::styled("  k / ↑         ", Style::default().fg(app.theme.secondary)), Span::raw("Select previous snippet")]),
        Line::from(vec![Span::styled("  g / G         ", Style::default().fg(app.theme.secondary)), Span::raw("Jump to top / bottom of list")]),
        Line::from(vec![Span::styled("  /             ", Style::default().fg(app.theme.secondary)), Span::raw("Search and fuzzy filter snippets in real-time")]),
        Line::from(vec![Span::styled("  a             ", Style::default().fg(app.theme.success)), Span::raw("Add a new snippet (modal form / $EDITOR)")]),
        Line::from(vec![Span::styled("  e / Enter     ", Style::default().fg(app.theme.primary)), Span::raw("Edit selected snippet (modal form / $EDITOR)")]),
        Line::from(vec![Span::styled("  Ctrl+e        ", Style::default().fg(app.theme.primary)), Span::raw("Directly open selected snippet in $EDITOR")]),
        Line::from(vec![Span::styled("  s / ,         ", Style::default().fg(app.theme.accent)), Span::raw("Open Settings & Theme selector")]),
        Line::from(vec![Span::styled("  c             ", Style::default().fg(app.theme.secondary)), Span::raw("Copy selected snippet code to clipboard")]),
        Line::from(vec![Span::styled("  [ / ]         ", Style::default().fg(app.theme.text_dim)), Span::raw("Decrease / increase left panel width")]),
        Line::from(vec![Span::styled("  d             ", Style::default().fg(app.theme.error)), Span::raw("Delete selected snippet")]),
        Line::from(vec![Span::styled("  Esc           ", Style::default().fg(app.theme.text_dim)), Span::raw("Clear search filter / close active modal")]),
        Line::from(vec![Span::styled("  q             ", Style::default().fg(app.theme.text_dim)), Span::raw("Quit CPKB")]),
        Line::from(""),
        Line::from(vec![Span::styled("Press Esc or ? to close this help guide.", Style::default().fg(app.theme.text_dim))]),
    ];

    let help_p = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Help & Keybindings ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)))
                .border_style(app.theme.style_border(true))
                .style(app.theme.style_surface()),
        )
        .alignment(Alignment::Left);

    f.render_widget(help_p, modal_area);
}

fn render_delete_modal(f: &mut Frame, app: &App, area: Rect, id: &str) {
    let modal_area = centered_rect(50, 25, area);
    f.render_widget(Clear, modal_area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Are you sure you want to delete ", Style::default().fg(app.theme.text)),
            Span::styled(id, Style::default().fg(app.theme.error).add_modifier(Modifier::BOLD)),
            Span::styled("?", Style::default().fg(app.theme.text)),
        ]),
        Line::from("This action cannot be undone."),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [y] ", Style::default().bg(app.theme.error).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Confirm Delete     "),
            Span::styled(" [n / Esc] ", Style::default().bg(app.theme.surface).fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
        ]),
    ];

    let p = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Confirm Deletion ", Style::default().fg(app.theme.error).add_modifier(Modifier::BOLD)))
                .border_style(Style::default().fg(app.theme.error).add_modifier(Modifier::BOLD))
                .style(app.theme.style_surface()),
        );

    f.render_widget(p, modal_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
