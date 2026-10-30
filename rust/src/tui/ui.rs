//! Main Ratatui UI rendering logic for CPKB.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
use ratatui::Frame;

use crate::tui::app::{
    ActiveModal, AddSnippetModalState, App, CustomThemeModalState, EditSnippetModalState,
    SettingsModalState, TagSelectModalState, IdFormatModalState, BORDER_OPTIONS, CUSTOM_THEME_FIELDS, LAYOUT_OPTIONS, SUPPORTED_LANGUAGES,
};
use crate::tui::badges::get_language_badge;
use crate::tui::theme::{get_available_themes, parse_color, resolve_id_color};

/// Convert a config `border_style` string to a ratatui `BorderType`.
fn border_type_from_config(style: &str) -> BorderType {
    match style.to_lowercase().trim() {
        "solid" | "plain" => BorderType::Plain,
        "heavy" | "thick" => BorderType::Thick,
        "double" => BorderType::Double,
        "dash" | "dashed" => BorderType::Rounded, // closest available fallback
        _ => BorderType::Rounded,
    }
}

/// Resolve the effective primary color, allowing `accent_color` from config to override the theme primary.
fn effective_primary(app: &App) -> ratatui::style::Color {
    let accent = app.config.display.accent_color.trim();
    if !accent.is_empty() && accent != "cyan" && accent != "default" {
        if let Some(c) = parse_color(accent) {
            return c;
        }
    }
    app.theme.primary
}


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
            ActiveModal::CustomTheme(state) => render_custom_theme_modal(f, app, size, &state),
            ActiveModal::TagSelect(state) => render_tag_select_modal(f, app, size, &state),
            ActiveModal::IdFormats(state) => render_id_formats_modal(f, app, size, state),
            ActiveModal::IdFormatBlocked(message) => render_id_format_blocked(f, app, size, message),
        }
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let bt = border_type_from_config(&app.config.display.border_style);
    let primary = effective_primary(app);

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
        Span::styled(" CPKB ", Style::default().bg(primary).fg(app.theme.background).add_modifier(Modifier::BOLD)),
        Span::styled(" Knowledge Base ", Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
        Span::styled(format!("[{}]", app.all_snippets.len()), Style::default().fg(app.theme.secondary)),
    ]);
    let brand_p = Paragraph::new(brand_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(bt)
            .border_style(app.theme.style_border(false)),
    );
    f.render_widget(brand_p, header_chunks[0]);

    // Search / Filter Block (Right)
    let search_content = if app.search_active {
        Line::from(vec![
            Span::styled(" 🔍 Search: ", Style::default().fg(primary).add_modifier(Modifier::BOLD)),
            Span::styled(&app.search_query, Style::default().fg(app.theme.text)),
            Span::styled("█", Style::default().fg(primary)),
        ])
    } else if !app.search_query.is_empty() {
        Line::from(vec![
            Span::styled(" 🔍 Filter: ", Style::default().fg(app.theme.secondary).add_modifier(Modifier::BOLD)),
            Span::styled(&app.search_query, Style::default().fg(app.theme.text)),
            Span::styled(format!(" ({} matches)", app.filtered_indices.len()), Style::default().fg(app.theme.text_dim)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" Search snippets ", Style::default().fg(app.theme.text_dim)),
            Span::styled(format!("({} total)", app.all_snippets.len()), Style::default().fg(app.theme.text_dim)),
        ])
    };

    let search_p = Paragraph::new(search_content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(bt)
            .border_style(app.theme.style_border(app.search_active)),
    );
    f.render_widget(search_p, header_chunks[1]);
}


fn render_body(f: &mut Frame, app: &mut App, area: Rect) {
    let is_vertical = app.config.display.layout.to_lowercase() == "vertical";
    let main_chunks = if is_vertical {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(app.left_panel_percent),
                Constraint::Percentage(100 - app.left_panel_percent),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(app.left_panel_percent),
                Constraint::Percentage(100 - app.left_panel_percent),
            ])
            .split(area)
    };

    render_snippet_list(f, app, main_chunks[0]);
    render_snippet_detail(f, app, main_chunks[1]);
}


fn render_snippet_list(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = !app.search_active && app.active_modal.is_none();
    let border_style = app.theme.style_border(is_focused);
    let bt = border_type_from_config(&app.config.display.border_style);
    let primary = effective_primary(app);

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(disp_idx, &actual_idx)| {
            let snip = &app.all_snippets[actual_idx];
            let is_selected = disp_idx == app.selected_index;

            let prefix = if is_selected { "▎ " } else { "  " };
            // Resolve per-format config color if available, else deterministic hash
            let id_color = resolve_id_color(&snip.id, &app.config.snippets.id_formats);
            let badge = get_language_badge(&snip.language);

            let title_style = if is_selected {
                Style::default().fg(primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text)
            };

            let id_style = if is_selected {
                Style::default().fg(id_color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(id_color)
            };

            let line1 = Line::from(vec![
                Span::styled(prefix, Style::default().fg(primary)),
                Span::styled(format!("{:<8} ", snip.id), id_style),
                Span::styled(format!("{} ", badge.icon), Style::default().fg(badge.color)),
                Span::styled(&snip.title, title_style),
            ]);

            let tag_str = if snip.tags.is_empty() {
                "—".to_string()
            } else {
                snip.tags.clone()
            };

            let line2 = Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{:<7} ", badge.name), Style::default().fg(badge.color).add_modifier(Modifier::DIM)),
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

    let sort_label = app.sort_order.label();
    let mut filter_tags = Vec::new();
    filter_tags.push(format!("Sort: {}", sort_label));
    if let Some(ref l) = app.language_filter {
        filter_tags.push(format!("Lang: {}", l));
    }
    if let Some(ref t) = app.tag_filter {
        filter_tags.push(format!("Tag: {}", t));
    }
    let list_title = format!(" Snippets ({}) [{}] ", app.filtered_indices.len(), filter_tags.join(" | "));
    let list_widget = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(bt)
                .title(Span::styled(list_title, Style::default().fg(primary).add_modifier(Modifier::BOLD)))
                .border_style(border_style),
        )
        .highlight_style(app.theme.style_selected());


    f.render_stateful_widget(list_widget, area, &mut app.list_state);
}


fn render_snippet_detail(f: &mut Frame, app: &App, area: Rect) {
    let bt = border_type_from_config(&app.config.display.border_style);
    let primary = effective_primary(app);

    if let Some(ref snip) = app.active_snippet {
        let badge = get_language_badge(&snip.language);
        let id_color = resolve_id_color(&snip.id, &app.config.snippets.id_formats);

        let detail_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(7), Constraint::Min(8)])
            .split(area);

        // Metadata Header
        let meta_lines = vec![
            Line::from(vec![
                Span::styled(format!(" {} ", snip.id), Style::default().bg(id_color).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(&snip.title, Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
                Span::raw("   "),
                Span::styled(format!(" {} {} ", badge.icon, badge.name), Style::default().bg(badge.color).fg(Color::Black).add_modifier(Modifier::BOLD)),
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
                    .border_type(bt)
                    .title(Span::styled(" Details ", Style::default().fg(primary).add_modifier(Modifier::BOLD)))
                    .border_style(app.theme.style_border(false)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(meta_p, detail_chunks[0]);

        // Highlighted Code Pane
        let highlighted_lines = app.highlighter.highlight(&snip.code, &snip.language);
        let code_title = format!(" Code Preview ({} {}) ", badge.icon, badge.name);
        let code_p = Paragraph::new(highlighted_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(bt)
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
                    .border_type(bt)
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
            Span::raw(" Add "),
            Span::styled(" e ", Style::default().bg(app.theme.surface).fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Edit "),
            Span::styled(" d ", Style::default().bg(app.theme.surface).fg(app.theme.error).add_modifier(Modifier::BOLD)),
            Span::raw(" Del "),
            Span::styled(" y ", Style::default().bg(app.theme.surface).fg(app.theme.secondary).add_modifier(Modifier::BOLD)),
            Span::raw(" Yank "),
            Span::styled(" L ", Style::default().bg(app.theme.surface).fg(app.theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" Lang "),
            Span::styled(" t ", Style::default().bg(app.theme.surface).fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Tag "),
            Span::styled(" o ", Style::default().bg(app.theme.surface).fg(app.theme.secondary).add_modifier(Modifier::BOLD)),
            Span::raw(" Sort "),
            Span::styled(" s ", Style::default().bg(app.theme.surface).fg(app.theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" Config "),
            Span::styled(" / ", Style::default().bg(app.theme.surface).fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Search "),
            Span::styled(" ? ", Style::default().bg(app.theme.surface).fg(app.theme.text_dim).add_modifier(Modifier::BOLD)),
            Span::raw(" Help "),
            Span::styled(" q ", Style::default().bg(app.theme.surface).fg(app.theme.text_dim).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ]);
        let footer_p = Paragraph::new(keys).style(app.theme.style_footer());
        f.render_widget(footer_p, area);
    }
}

fn render_add_modal(f: &mut Frame, app: &App, area: Rect, state: &AddSnippetModalState) {
    render_snippet_form(f, app, area, state.focus_idx, &app.form_cursors, "Add New Snippet", &state.title, &state.description, &state.use_case, &state.tags, &state.language, &state.id_format, &state.code, false);
}

fn render_edit_modal(f: &mut Frame, app: &App, area: Rect, state: &EditSnippetModalState) {
    render_snippet_form(f, app, area, state.focus_idx, &app.form_cursors, &format!("Edit Snippet: {}", state.id), &state.title, &state.description, &state.use_case, &state.tags, &state.language, &state.id_format, &state.code, true);
}

fn text_cursor_location(text: &str, cursor: usize) -> (usize, usize) {
    let prefix: Vec<char> = text.chars().take(cursor).collect();
    let line = prefix.iter().filter(|c| **c == '\n').count();
    let column = prefix.iter().rposition(|c| *c == '\n').map(|idx| prefix.len() - idx - 1).unwrap_or(prefix.len());
    (line, column)
}

fn cursor_spans(text: &str, cursor: usize, visible: bool, primary: Color) -> Vec<Span<'static>> {
    if !visible {
        return vec![Span::raw(text.to_string())];
    }
    let chars: Vec<char> = text.chars().collect();
    let at = cursor.min(chars.len());
    let left: String = chars[..at].iter().collect();
    let right: String = chars[at..].iter().collect();
    vec![
        Span::raw(left),
        Span::styled("▏", Style::default().fg(primary).add_modifier(Modifier::BOLD)),
        Span::raw(right),
    ]
}

#[allow(clippy::too_many_arguments)]
fn render_snippet_form(f: &mut Frame, app: &App, area: Rect, focus: usize, cursors: &[usize; 7], heading: &str, title: &str, description: &str, use_case: &str, tags: &str, language: &str, id_format: &str, code: &str, language_editable: bool) {
    let modal_area = centered_rect(92, 94, area);
    f.render_widget(Clear, modal_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(modal_area);
    let mut lines = Vec::new();
    let mut field_rows = [0usize; 7];
    let mut cursor_rows = [0usize; 7];
    for idx in 0..7 {
        field_rows[idx] = lines.len();
        let active = focus == idx;
        let marker = if active { "▶ " } else { "  " };
        let label_style = if active { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) };
        let value = match idx { 0 => title, 1 => description, 2 => use_case, 3 => tags, 4 => language, 5 => id_format, _ => code };
        if idx == 0 {
            cursor_rows[idx] = field_rows[idx];
            let mut spans = vec![Span::styled(marker, label_style), Span::styled("Title: ", label_style)];
            spans.extend(cursor_spans(value, cursors[idx], active, app.theme.primary));
            lines.push(Line::from(spans));
        } else if idx == 1 || idx == 2 {
            let label_txt = if idx == 1 { "Description" } else { "Use Case" };
            lines.push(Line::from(vec![Span::styled(marker, label_style), Span::styled(format!("{label_txt} (Enter for newline):"), label_style)]));
            let content: Vec<&str> = value.split('\n').collect();
            let (cursor_line, cursor_col) = text_cursor_location(value, cursors[idx]);
            cursor_rows[idx] = field_rows[idx] + 1 + cursor_line;
            let min_lines = if idx == 1 { 3 } else { 2 };
            for line_idx in 0..content.len().max(min_lines) {
                let text = content.get(line_idx).copied().unwrap_or("");
                let mut spans = vec![Span::raw("    ")];
                spans.extend(cursor_spans(text, cursor_col, active && cursor_line == line_idx, app.theme.primary));
                lines.push(Line::from(spans));
            }
        } else if idx == 3 {
            cursor_rows[idx] = field_rows[idx];
            let mut spans = vec![Span::styled(marker, label_style), Span::styled("Tags (comma separated): ", label_style)];
            spans.extend(cursor_spans(value, cursors[idx], active, app.theme.primary));
            lines.push(Line::from(spans));
            if active {
                let suggestions = app.tag_suggestions(value);
                if suggestions.is_empty() {
                    let token = value.rsplit(',').next().unwrap_or("").trim();
                    if !token.is_empty() { lines.push(Line::from(vec![Span::raw("    + New tag: "), Span::styled(token, Style::default().fg(app.theme.success))])); }
                } else {
                    let visible_count = suggestions.len().min(4);
                    let start = app.tag_suggestion_idx.saturating_sub(visible_count - 1).min(suggestions.len() - visible_count);
                    for (suggestion_idx, suggestion) in suggestions.iter().enumerate().skip(start).take(visible_count) {
                        let marker = if suggestion_idx == app.tag_suggestion_idx { "  > " } else { "    " };
                        let stale = if suggestion.retired { "*" } else { "" };
                        lines.push(Line::from(vec![Span::raw(marker), Span::styled(format!("{}{}  ({})", suggestion.tag, stale, suggestion.usage_count), Style::default().fg(if suggestion.retired { app.theme.text_dim } else { app.theme.secondary }))]));
                    }
                }
                lines.push(Line::from(Span::styled("    ↑/↓ choose • Enter complete", Style::default().fg(app.theme.text_dim))));
            }
        } else if idx == 4 {
            let badge = get_language_badge(language);
            let mut spans = vec![Span::styled(marker, label_style), Span::styled("Language: ", label_style)];
            spans.extend(cursor_spans(language, cursors[idx], active && language_editable, app.theme.primary));
            spans.extend([Span::raw("  "), Span::styled(format!("{} {}", badge.icon, badge.name), Style::default().bg(badge.color).fg(Color::Black).add_modifier(Modifier::BOLD))]);
            if !language_editable { spans.push(Span::raw("  ←/→")); }
            cursor_rows[idx] = field_rows[idx];
            lines.push(Line::from(spans));
        } else if idx == 5 {
            lines.push(Line::from(vec![Span::styled(marker, label_style), Span::styled("ID Format: ", label_style), Span::raw(id_format), Span::raw("  ←/→") ]));
        } else {
            lines.push(Line::from(vec![Span::styled(marker, label_style), Span::styled(format!("Code ({} lines; Enter for newline):", code.lines().count().max(1)), label_style)]));
            let code_lines: Vec<&str> = code.split('\n').collect();
            let (cursor_line, cursor_col) = text_cursor_location(code, cursors[idx]);
            cursor_rows[idx] = field_rows[idx] + 1 + cursor_line;
            for line_idx in 0..code_lines.len().max(12) {
                let text = code_lines.get(line_idx).copied().unwrap_or("");
                let mut spans = vec![Span::styled(format!("{:>3} │ ", line_idx + 1), Style::default().fg(app.theme.text_dim))];
                spans.extend(cursor_spans(text, cursor_col, active && cursor_line == line_idx, app.theme.primary));
                lines.push(Line::from(spans));
            }
        }
    }
    let inner_height = chunks[0].height.saturating_sub(2) as usize;
    let scroll = cursor_rows[focus].saturating_sub(inner_height / 2);
    let p = Paragraph::new(lines).scroll((scroll as u16, 0)).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(Span::styled(heading, Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD))).border_style(app.theme.style_border(true)).style(app.theme.style_surface()));
    f.render_widget(p, chunks[0]);
    let controls = "Text: ←/→ cursor • ↑/↓ line • Home/End line edge\nTab: next field • Enter: newline / advance • Backspace/Delete\nCtrl+S: save • Ctrl+E: editor • Esc: cancel • Tags: ↑/↓";
    f.render_widget(Paragraph::new(controls).alignment(Alignment::Center).style(app.theme.style_surface()), chunks[1]);
}

fn render_settings_modal(f: &mut Frame, app: &App, area: Rect, state: &SettingsModalState) {
    let modal_area = centered_rect(68, 75, area);
    f.render_widget(Clear, modal_area);

    let available_themes = get_available_themes(Some(&app.config.display.custom_theme));
    let current_theme = available_themes[state.theme_idx.min(available_themes.len() - 1)].0;
    let current_lang = SUPPORTED_LANGUAGES[state.lang_idx.min(SUPPORTED_LANGUAGES.len() - 1)];
    let lang_badge = get_language_badge(current_lang);

    let theme_focused = state.focus_idx == 0;
    let lang_focused = state.focus_idx == 1;
    let sort_focused = state.focus_idx == 2;
    let layout_focused = state.focus_idx == 3;
    let border_focused = state.focus_idx == 4;
    let custom_focused = state.focus_idx == 5;
    let formats_focused = state.focus_idx == 6;

    let sort_name = match state.sort_idx {
        1 => "Snippet ID",
        2 => "Snippet Name",
        _ => "Recent / Date",
    };
    let layout_name = LAYOUT_OPTIONS[state.layout_idx.min(LAYOUT_OPTIONS.len() - 1)].1;
    let border_name = BORDER_OPTIONS[state.border_idx.min(BORDER_OPTIONS.len() - 1)].1;

    let mut lines = vec![
        Line::from(vec![
            Span::styled("CPKB Settings & Preferences", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(if theme_focused { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled("Color Theme:       ", if theme_focused { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) }),
            Span::styled(" ◀ ", Style::default().fg(app.theme.secondary)),
            Span::styled(format!("{:<18}", current_theme), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" ▶ ", Style::default().fg(app.theme.secondary)),
            Span::styled("  Preview: ", Style::default().fg(app.theme.text_dim)),
            Span::styled("●", Style::default().fg(app.theme.primary)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(app.theme.secondary)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(app.theme.accent)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(app.theme.success)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(app.theme.warning)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(app.theme.error)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(if lang_focused { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled("Default Language:  ", if lang_focused { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) }),
            Span::styled(" ◀ ", Style::default().fg(app.theme.secondary)),
            Span::styled(format!("{:<18}", current_lang), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" ▶ ", Style::default().fg(app.theme.secondary)),
            Span::styled(format!(" {} {} ", lang_badge.icon, lang_badge.name), Style::default().bg(lang_badge.color).fg(Color::Black).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(if sort_focused { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled("Default Sort:      ", if sort_focused { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) }),
            Span::styled(" ◀ ", Style::default().fg(app.theme.secondary)),
            Span::styled(format!("{:<18}", sort_name), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" ▶ ", Style::default().fg(app.theme.secondary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(if layout_focused { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled("Display Layout:    ", if layout_focused { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) }),
            Span::styled(" ◀ ", Style::default().fg(app.theme.secondary)),
            Span::styled(format!("{:<26}", layout_name), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" ▶ ", Style::default().fg(app.theme.secondary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(if border_focused { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled("Border Style:      ", if border_focused { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) }),
            Span::styled(" ◀ ", Style::default().fg(app.theme.secondary)),
            Span::styled(format!("{:<26}", border_name), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" ▶ ", Style::default().fg(app.theme.secondary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(if custom_focused { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled("🎨 Edit Custom Theme Colors", if custom_focused { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) }),
            Span::raw("  "),
            Span::styled("[Press Enter or 'c' to edit hex colors]", Style::default().fg(app.theme.secondary)),
        ]),
        Line::from(vec![
            Span::styled(if formats_focused { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled("Manage Snippet ID Formats", if formats_focused { Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.text_dim) }),
            Span::styled("  [Enter / ← / →]", Style::default().fg(app.theme.secondary)),
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
    ];

    if current_theme == "Custom" {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" [Custom Theme Active] ", Style::default().bg(app.theme.accent).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" Colors loaded from config.json -> display.custom_theme", Style::default().fg(app.theme.text_dim)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" [← / →] ", Style::default().bg(app.theme.surface).fg(app.theme.secondary).add_modifier(Modifier::BOLD)),
        Span::raw(" Value  "),
        Span::styled(" [Tab] ", Style::default().bg(app.theme.surface).fg(app.theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw(" Row  "),
        Span::styled(" [Ctrl+e / E] ", Style::default().bg(app.theme.primary).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" Edit in $EDITOR  "),
        Span::styled(" [Enter] ", Style::default().bg(app.theme.success).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" Save  "),
        Span::styled(" [Esc] ", Style::default().bg(app.theme.surface).fg(app.theme.text).add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel"),
    ]));

    let bt = border_type_from_config(&app.config.display.border_style);
    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(bt)
                .title(Span::styled(" Settings ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)))
                .border_style(app.theme.style_border(true))
                .style(app.theme.style_surface()),
        );

    f.render_widget(p, modal_area);
}

fn render_id_formats_modal(f: &mut Frame, app: &App, area: Rect, state: &IdFormatModalState) {
    let modal = centered_rect(70, 70, area);
    f.render_widget(Clear, modal);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(4)])
        .split(modal);
    let mut lines = Vec::new();
    for (idx, (label, value)) in [("Name", &state.name), ("Pattern", &state.pattern)].iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(if state.focus_idx == idx { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::styled(format!("{label}: "), Style::default().fg(app.theme.secondary)),
            Span::styled(if value.is_empty() { "(type here)" } else { value.as_str() }, Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Configured formats:"));
    for (idx, (name, fmt)) in app.config.snippets.id_formats.iter().enumerate() {
        let pattern = fmt.pattern.clone().unwrap_or_else(|| format!("{}####", fmt.prefix.as_deref().unwrap_or("CP")));
        lines.push(Line::from(vec![
            Span::styled(if idx == state.selected_idx { "▶ " } else { "  " }, Style::default().fg(app.theme.primary)),
            Span::raw(format!("{name:<16} {pattern}{}", if name == &app.config.snippets.default_id_format { " [default]" } else { "" })),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Examples: ALG_####  NOTE-###  LATEX-######"));
    f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" ID Formats ").border_style(app.theme.style_border(true)).style(app.theme.style_surface())), chunks[0]);
    let controls = "Tab / Shift+Tab / ←→: switch fields\nEnter: advance / save • Backspace: edit • Esc: Settings\n↑/↓: select format • Ctrl+D: remove";
    f.render_widget(Paragraph::new(controls).alignment(Alignment::Center).style(app.theme.style_surface()), chunks[1]);
}

fn render_id_format_blocked(f: &mut Frame, app: &App, area: Rect, message: &str) {
    let modal = centered_rect(65, 35, area);
    f.render_widget(Clear, modal);
    let lines = vec![Line::from("Cannot remove ID format"), Line::from(""), Line::from(message), Line::from(""), Line::from("Press Enter or Esc to continue")];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }).block(Block::default().borders(Borders::ALL).title(" ID Format In Use ").border_style(Style::default().fg(app.theme.error)).style(app.theme.style_surface())), modal);
}

fn render_custom_theme_modal(f: &mut Frame, app: &App, area: Rect, state: &CustomThemeModalState) {
    let modal_area = centered_rect(75, 80, area);
    f.render_widget(Clear, modal_area);

    let bt = border_type_from_config(&app.config.display.border_style);
    let primary = effective_primary(app);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("🎨 Edit Custom Theme Hex Colors", Style::default().fg(primary).add_modifier(Modifier::BOLD)),
            Span::styled("  (Type hex code or named color, Tab to next field)", Style::default().fg(app.theme.text_dim)),
        ]),
        Line::from(""),
    ];

    for (idx, &(field_name, field_desc)) in CUSTOM_THEME_FIELDS.iter().enumerate() {
        let is_focused = state.focus_idx == idx;
        let val = state.get_value(idx);
        let parsed = parse_color(val);

        let prefix = if is_focused { "▶ " } else { "  " };
        let label_style = if is_focused {
            Style::default().fg(primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text_dim)
        };
        let val_style = if is_focused {
            Style::default().fg(app.theme.text).bg(app.theme.background).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text)
        };

        let mut row_spans = vec![
            Span::styled(prefix, Style::default().fg(primary)),
            Span::styled(format!("{:<14}", field_name), label_style),
            Span::styled(if val.is_empty() { if is_focused { "█" } else { "—" } } else { val }, val_style),
            Span::raw("  "),
        ];

        if let Some(color) = parsed {
            row_spans.push(Span::styled("● ", Style::default().fg(color)));
            row_spans.push(Span::styled(format!("({field_desc})"), Style::default().fg(app.theme.text_dim)));
        } else {
            row_spans.push(Span::styled("✖ ", Style::default().fg(app.theme.error)));
            row_spans.push(Span::styled(format!("({field_desc})"), Style::default().fg(app.theme.text_dim)));
        }

        lines.push(Line::from(row_spans));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" [Enter / Ctrl+s] ", Style::default().bg(app.theme.success).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" Save Theme Colors    "),
        Span::styled(" [Tab / ↑ / ↓] ", Style::default().bg(app.theme.surface).fg(primary).add_modifier(Modifier::BOLD)),
        Span::raw(" Navigate Fields    "),
        Span::styled(" [Esc] ", Style::default().bg(app.theme.surface).fg(app.theme.text).add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel"),
    ]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(bt)
                .title(Span::styled(" Custom Theme Palette ", Style::default().fg(primary).add_modifier(Modifier::BOLD)))
                .border_style(app.theme.style_border(true))
                .style(app.theme.style_surface()),
        );

    f.render_widget(p, modal_area);
}


fn render_help_modal(f: &mut Frame, app: &App, area: Rect) {
    let modal_area = centered_rect(65, 70, area);
    f.render_widget(Clear, modal_area);

    let help_text = vec![
        Line::from(vec![Span::styled("CPKB Keybindings & Commands", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(vec![Span::styled("  j / ↓         ", Style::default().fg(app.theme.secondary)), Span::raw("Select next snippet")]),
        Line::from(vec![Span::styled("  k / ↑         ", Style::default().fg(app.theme.secondary)), Span::raw("Select previous snippet")]),
        Line::from(vec![Span::styled("  g / G         ", Style::default().fg(app.theme.secondary)), Span::raw("Jump to top / bottom of list")]),
        Line::from(vec![Span::styled("  /             ", Style::default().fg(app.theme.secondary)), Span::raw("Search and fuzzy filter snippets in real-time")]),
        Line::from(vec![Span::styled("  o             ", Style::default().fg(app.theme.secondary)), Span::raw("Cycle sort order (Snippet ID, Name, Date)")]),
        Line::from(vec![Span::styled("  y / c         ", Style::default().fg(app.theme.secondary)), Span::raw("Copy / Yank selected snippet code to clipboard")]),
        Line::from(vec![Span::styled("  L             ", Style::default().fg(app.theme.accent)), Span::raw("Cycle language filter (All → cpp → python → rust → ...)" )]),
        Line::from(vec![Span::styled("  t             ", Style::default().fg(app.theme.primary)), Span::raw("Open Tag selector modal (filter by tag)")]),
        Line::from(vec![Span::styled("  a             ", Style::default().fg(app.theme.success)), Span::raw("Add a new snippet (modal form / $EDITOR)")]),
        Line::from(vec![Span::styled("  e / Enter     ", Style::default().fg(app.theme.primary)), Span::raw("Edit selected snippet (modal form / $EDITOR)")]),
        Line::from(vec![Span::styled("  Ctrl+e        ", Style::default().fg(app.theme.primary)), Span::raw("Directly open selected snippet in $EDITOR")]),
        Line::from(vec![Span::styled("  s / ,         ", Style::default().fg(app.theme.accent)), Span::raw("Open Settings & Theme selector")]),
        Line::from(vec![Span::styled("  [ / ]         ", Style::default().fg(app.theme.text_dim)), Span::raw("Decrease / increase left panel width")]),
        Line::from(vec![Span::styled("  d             ", Style::default().fg(app.theme.error)), Span::raw("Delete selected snippet")]),
        Line::from(vec![Span::styled("  Esc           ", Style::default().fg(app.theme.text_dim)), Span::raw("Clear search, language & tag filters / close modal")]),
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

fn render_tag_select_modal(f: &mut Frame, app: &App, area: Rect, state: &TagSelectModalState) {
    let modal_area = centered_rect(76, 72, area);
    f.render_widget(Clear, modal_area);

    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(modal_area);

    let bt = border_type_from_config(&app.config.display.border_style);
    let primary = effective_primary(app);

    let items: Vec<ListItem> = state
        .tags
        .iter()
        .enumerate()
        .map(|(idx, (tag, count))| {
            let is_selected = state.selected_idx == idx;
            let prefix = if is_selected { "▶ " } else { "  " };
            let tag_style = if is_selected {
                Style::default().fg(primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text)
            };
            let count_style = if is_selected {
                Style::default().fg(app.theme.secondary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text_dim)
            };

            let item_line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(primary)),
                Span::styled(format!("{:<28}", tag), tag_style),
                Span::styled(format!("({} snippet{})", count, if *count == 1 { "" } else { "s" }), count_style),
            ]);
            let bg_style = if is_selected {
                app.theme.style_selected()
            } else {
                Style::default()
            };
            ListItem::new(item_line).style(bg_style)
        })
        .collect();

    let tag_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(bt)
                .title(Span::styled(" 🏷️ Filter by Tag ", Style::default().fg(primary).add_modifier(Modifier::BOLD)))
                .border_style(app.theme.style_border(true))
                .style(app.theme.style_surface()),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_idx.min(state.tags.len().saturating_sub(1))));
    f.render_stateful_widget(tag_list, modal_chunks[0], &mut list_state);
    let controls = Paragraph::new("↑/↓ or wheel: select • Enter: apply\nc: clear • Esc: close")
        .style(app.theme.style_surface())
        .alignment(Alignment::Center);
    f.render_widget(controls, modal_chunks[1]);
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
