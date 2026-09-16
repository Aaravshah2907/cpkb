//! CLI Subcommand handlers matching Python CPKB behavior.

use std::io::{self, Read, Write};
use std::path::Path;
use colored::*;
use rusqlite::Connection;

use crate::clipboard::copy_to_clipboard;
use crate::config::{load_config, save_config};
use crate::db::snippets::{add_snippet, delete_snippet, get_snippet, list_snippets, recent_snippets, update_snippet, SnippetSummary};
use crate::db::search::{query_snippets, search_snippets};
use crate::db::tags::{add_tag, remove_tag};
use crate::db::usages::{add_usage, get_usage, get_usages, update_usage};
use crate::db::srs::{get_due_snippet, get_random_snippet, get_srs_stats, get_unreviewed_snippet, upsert_review};
use crate::db::backup_db;
use crate::cli::editor::{open_snippet_editor, open_usage_editor};

pub fn print_summaries(rows: &[SnippetSummary]) {
    if rows.is_empty() {
        println!("No snippets found.");
        return;
    }
    println!("{:<10} {:<40} Tags", "ID", "Title");
    println!("{}", "-".repeat(70));
    for r in rows {
        let title = if r.title.chars().count() > 38 {
            format!("{}...", r.title.chars().take(35).collect::<String>())
        } else {
            r.title.clone()
        };
        println!("{:<10} {:<40} {}", r.id, title, r.tags);
    }
}

pub fn cmd_list(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let rows = list_snippets(conn)?;
    print_summaries(&rows);
    Ok(())
}

pub fn cmd_recent(conn: &Connection, limit: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("Showing {} most recent snippets:", limit);
    let rows = recent_snippets(conn, limit)?;
    print_summaries(&rows);
    Ok(())
}

pub fn cmd_search(conn: &Connection, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let rows = search_snippets(conn, query)?;
    println!("Found {} snippet(s) matching '{}':", rows.len(), query);
    print_summaries(&rows);
    Ok(())
}

pub fn cmd_query(conn: &Connection, query: &str, limit: u32) -> Result<(), Box<dyn std::error::Error>> {
    let results = query_snippets(conn, query, limit)?;
    for (id, title) in results {
        println!("{} | {}", id, title);
    }
    Ok(())
}

pub fn cmd_show(conn: &Connection, id: &str, as_json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let snip = match get_snippet(conn, id)? {
        Some(s) => s,
        None => {
            eprintln!("Error: Snippet with ID {} not found.", id);
            std::process::exit(1);
        }
    };

    if as_json {
        println!("{}", serde_json::to_string_pretty(&snip)?);
        return Ok(());
    }

    println!("ID:          {}", snip.id);
    println!("Title:       {}", snip.title);
    println!("Description: {}", snip.description);
    println!("Use Case:    {}", snip.use_case);
    println!("Tags:        {}", snip.tags);
    println!("Language:    {}", snip.language);
    println!("Created At:  {}", snip.created_at);
    println!("Updated At:  {}", snip.updated_at);
    println!("\n--- Code ---\n");
    println!("{}", snip.code);
    println!("\n------------\n");

    let usages = get_usages(conn, id)?;
    if usages.is_empty() {
        println!("Usages (0): None recorded yet.");
    } else {
        println!("Usages ({}):", usages.len());
        for (i, u) in usages.iter().enumerate() {
            println!("  {}. File: {} | Problem: {} ({})", i + 1, u.file_path, u.problem_name, u.created_at);
            if !u.notes.is_empty() {
                println!("     Notes: {}", u.notes);
            }
        }
    }
    Ok(())
}

fn prompt_line(prompt: &str, default: Option<&str>) -> String {
    if let Some(def) = default {
        print!("{} [{}]: ", prompt, def);
    } else {
        print!("{}: ", prompt);
    }
    io::stdout().flush().ok();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim();
        if trimmed.is_empty() && default.is_some() {
            return default.unwrap().to_string();
        }
        return trimmed.to_string();
    }
    default.unwrap_or_default().to_string()
}

pub fn cmd_add(
    conn: &mut Connection,
    app_dir: &Path,
    language_opt: Option<&str>,
    id_format_opt: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(app_dir);
    println!("Adding a new snippet...");

    let title = prompt_line("Title", None);
    let desc = prompt_line("Description", None);
    let use_case = prompt_line("Use case", None);
    let tags_in = prompt_line("Tags (comma separated)", None);

    let default_tags = config.snippets.default_tags.trim();
    let tags = if !default_tags.is_empty() && !tags_in.is_empty() {
        format!("{}, {}", default_tags, tags_in)
    } else if !tags_in.is_empty() {
        tags_in
    } else {
        default_tags.to_string()
    };

    let default_lang = language_opt.unwrap_or(&config.snippets.code_language);
    let language = if language_opt.is_some() {
        language_opt.unwrap().to_string()
    } else {
        prompt_line("Language", Some(default_lang))
    };

    println!("Enter the code (Ctrl+D on an empty line to finish):");
    let mut code = String::new();
    io::stdin().read_to_string(&mut code)?;
    let code_trimmed = code.trim();

    if title.is_empty() || code_trimmed.is_empty() {
        eprintln!("Error: Title and code are required.");
        std::process::exit(1);
    }

    let snippet_id = add_snippet(
        conn,
        app_dir,
        &title,
        &desc,
        &use_case,
        &tags,
        code_trimmed,
        Some(&language),
        id_format_opt,
    )?;

    println!("\nSnippet added successfully! ID: {}", snippet_id);
    Ok(())
}

pub fn cmd_edit(
    conn: &mut Connection,
    app_dir: &Path,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let snip = match get_snippet(conn, id)? {
        Some(s) => s,
        None => {
            eprintln!("Error: Snippet {} not found.", id);
            return Ok(());
        }
    };

    if let Some(edited) = open_snippet_editor(app_dir, &snip)? {
        update_snippet(
            conn,
            id,
            &edited.title,
            &edited.description,
            &edited.use_case,
            &edited.tags,
            &edited.code,
            &edited.language,
        )?;
        println!("Snippet {} updated successfully!", id);
    }
    Ok(())
}

pub fn cmd_delete(conn: &Connection, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let snip = match get_snippet(conn, id)? {
        Some(s) => s,
        None => {
            eprintln!("Error: Snippet {} not found.", id);
            return Ok(());
        }
    };

    let confirm = prompt_line(&format!("Are you sure you want to delete '{}' ({})? [y/N]", snip.title, id), Some("N"));
    if matches!(confirm.to_lowercase().as_str(), "y" | "yes") {
        delete_snippet(conn, id)?;
        println!("Snippet {} deleted.", id);
    } else {
        println!("Aborted.");
    }
    Ok(())
}

pub fn cmd_tag_add(conn: &mut Connection, id: &str, tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    match add_tag(conn, id, tag) {
        Ok(tags) => println!("Tag added. Updated tags for {}: {}", id, tags),
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}

pub fn cmd_tag_remove(conn: &mut Connection, id: &str, tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    match remove_tag(conn, id, tag) {
        Ok(Some(tags)) => println!("Tag removed. Updated tags for {}: {}", id, tags),
        Ok(None) => println!("Tag '{}' was not attached to snippet {}.", tag, id),
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}

pub fn cmd_use(conn: &Connection, id: &str, file: &str) -> Result<(), Box<dyn std::error::Error>> {
    if get_snippet(conn, id)?.is_none() {
        eprintln!("Error: Snippet {} not found.", id);
        return Ok(());
    }

    let problem = prompt_line("Problem name / URL (optional)", None);
    let notes = prompt_line("Notes (optional)", None);

    let usage_id = add_usage(conn, id, file, &problem, &notes)?;
    println!("Recorded usage #{} for snippet {}.", usage_id, id);
    Ok(())
}

pub fn cmd_usages(conn: &Connection, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let usages = get_usages(conn, id)?;
    if usages.is_empty() {
        println!("No usages recorded for snippet {}.", id);
        return Ok(());
    }
    println!("Usages for snippet {} ({} total):", id, usages.len());
    println!("{:<6} {:<30} {:<25} Date", "ID", "File", "Problem");
    println!("{}", "-".repeat(75));
    for u in usages {
        println!("{:<6} {:<30} {:<25} {}", u.id, u.file_path, u.problem_name, u.created_at);
    }
    Ok(())
}

pub fn cmd_edit_usage(conn: &Connection, app_dir: &Path, usage_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    let usage = match get_usage(conn, usage_id)? {
        Some(u) => u,
        None => {
            eprintln!("Error: Usage record #{} not found.", usage_id);
            return Ok(());
        }
    };

    if let Some(edited) = open_usage_editor(app_dir, &usage)? {
        update_usage(conn, usage_id, &edited.file_path, &edited.problem_name, &edited.notes)?;
        println!("Usage #{} updated successfully!", usage_id);
    }
    Ok(())
}

pub fn cmd_copy(conn: &Connection, id: &str, file_opt: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let snip = match get_snippet(conn, id)? {
        Some(s) => s,
        None => {
            eprintln!("Error: Snippet {} not found.", id);
            std::process::exit(1);
        }
    };

    if let Some(file_path) = file_opt {
        use std::fs::OpenOptions;
        let mut f = OpenOptions::new().create(true).append(true).open(file_path)?;
        writeln!(f, "\n{}", snip.code)?;
        println!("Appended snippet {} to {}", id, file_path);
    } else {
        copy_to_clipboard(&snip.code)?;
        println!("Copied snippet {} to clipboard! ({})", id, snip.title);
    }
    Ok(())
}

pub fn cmd_stats(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let count: u32 = conn.query_row("SELECT COUNT(*) FROM snippets", [], |r| r.get(0))?;
    let usages: u32 = conn.query_row("SELECT COUNT(*) FROM usages", [], |r| r.get(0))?;
    let tags: u32 = conn.query_row("SELECT COUNT(DISTINCT tag) FROM tags", [], |r| r.get(0))?;

    println!("CPKB Knowledge Base Statistics");
    println!("{}", "=".repeat(35));
    println!("Total Snippets:  {}", count.to_string().cyan());
    println!("Recorded Usages: {}", usages.to_string().green());
    println!("Distinct Tags:   {}", tags.to_string().yellow());
    Ok(())
}

pub fn cmd_random(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(snip) = get_random_snippet(conn)? {
        cmd_show(conn, &snip.id, false)?;
    } else {
        println!("No snippets found. Add some with 'cpkb add'.");
    }
    Ok(())
}

pub fn cmd_backup(app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let path = backup_db(app_dir, "manual")?;
    println!("Database backup created at: {}", path.display());
    Ok(())
}

pub fn cmd_config(app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(app_dir);
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub fn cmd_id_format_list(app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(app_dir);
    let default_fmt = &config.snippets.default_id_format;
    println!("{:<18} {:<24} Default", "Name", "Pattern");
    println!("{}", "-".repeat(52));

    for (name, fmt) in &config.snippets.id_formats {
        let marker = if name == default_fmt { "*" } else { "" };
        let pattern_str = if let Some(ref p) = fmt.pattern {
            p.clone()
        } else {
            let pfx = fmt.prefix.as_deref().unwrap_or("CP");
            format!("{}####", pfx)
        };
        println!("{:<18} {:<24} {}", name, pattern_str, marker);
    }
    println!("\nPattern guide: <ID_BEG_KEY><_.-@><#######>");
    println!("Examples: NOTE-###, ALG_####, ID@#######");
    Ok(())
}

pub fn cmd_id_format_add(
    app_dir: &Path,
    name: &str,
    pattern: Option<&str>,
    prefix: Option<&str>,
    is_default: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config(app_dir);
    if let Some(pat) = pattern {
        if !pat.contains('#') {
            eprintln!("Error: --pattern must include at least one # placeholder.");
            return Ok(());
        }
        config.snippets.id_formats.insert(
            name.to_string(),
            crate::config::IdFormatConfig {
                prefix: None,
                width: None,
                pattern: Some(pat.to_string()),
            },
        );
    } else if let Some(pfx) = prefix {
        config.snippets.id_formats.insert(
            name.to_string(),
            crate::config::IdFormatConfig {
                prefix: Some(pfx.to_string()),
                width: Some(serde_json::Value::String("auto".to_string())),
                pattern: None,
            },
        );
    } else {
        eprintln!("Error: Provide either --pattern or --prefix.");
        return Ok(());
    }

    if is_default {
        config.snippets.default_id_format = name.to_string();
    }

    let path = save_config(app_dir, &config)?;
    println!("Saved ID format '{}' to {}", name, path.display());
    if is_default {
        println!("Default ID format set to '{}'.", name);
    }
    Ok(())
}

pub fn cmd_id_format_default(app_dir: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config(app_dir);
    if !config.snippets.id_formats.contains_key(name) {
        let available: Vec<&String> = config.snippets.id_formats.keys().collect();
        eprintln!(
            "Error: Unknown ID format '{}'. Available formats: {}",
            name,
            available.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        );
        return Ok(());
    }

    config.snippets.default_id_format = name.to_string();
    let path = save_config(app_dir, &config)?;
    println!("Default ID format set to '{}' in {}", name, path.display());
    Ok(())
}

pub fn cmd_revise(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    let (snip, source) = if let Some(s) = get_due_snippet(conn)? {
        (s, "due for review")
    } else if let Some(s) = get_unreviewed_snippet(conn)? {
        (s, "never reviewed")
    } else if let Some(s) = get_random_snippet(conn)? {
        (s, "random")
    } else {
        println!("No snippets found. Add some with 'cpkb add'.");
        return Ok(());
    };

    println!("\n📖  Revise this snippet ({})  [ID: {}]", source, snip.id);
    println!("Title:       {}", snip.title);
    if !snip.description.is_empty() {
        println!("Description: {}", snip.description);
    }

    print!("\nPress Enter to reveal the code...");
    io::stdout().flush().ok();
    let mut wait = String::new();
    io::stdin().read_line(&mut wait).ok();

    println!("\n--- Code ---\n");
    println!("{}", snip.code);
    println!("\n------------\n");

    println!("Rate your recall quality:");
    println!("  0 — Complete blackout");
    println!("  1 — Wrong, but recognised the answer");
    println!("  2 — Wrong, but answer felt familiar");
    println!("  3 — Correct, but with serious difficulty");
    println!("  4 — Correct, with some hesitation");
    println!("  5 — Perfect recall");

    let quality_str = prompt_line("Score (0-5)", Some("5"));
    let quality: u8 = quality_str.parse().unwrap_or(5).min(5);

    let record = upsert_review(conn, &snip.id, quality)?;
    println!(
        "Schedule updated: interval = {} day(s), ease = {:.2}, next review = {}",
        record.interval, record.ease_factor, record.next_review
    );
    Ok(())
}

pub fn cmd_srs_stats(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let stats = get_srs_stats(conn)?;
    println!("Spaced Repetition (SM-2) Statistics");
    println!("{}", "=".repeat(40));
    println!("Total Snippets:      {}", stats.total);
    println!("Reviewed Snippets:   {}", stats.reviewed);
    println!("Unreviewed Snippets: {}", stats.unreviewed);
    println!("Due for Review Now:  {}", stats.due);
    println!("Average Ease Factor: {:.2}", stats.avg_ease_factor);
    Ok(())
}

// ─── Export / Import ─────────────────────────────────────────────────────────

pub fn cmd_export(conn: &Connection, app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use crate::export::markdown::export_markdown;
    match export_markdown(conn, app_dir) {
        Ok((0, _)) => println!("No snippets to export."),
        Ok((n, path)) => println!("Exported {} snippet(s) to {}", n, path.display()),
        Err(e) => eprintln!("Export failed: {e}"),
    }
    Ok(())
}

pub fn cmd_export_json(conn: &Connection, app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use crate::export::json::export_json;
    match export_json(conn, app_dir) {
        Ok((0, _)) => println!("No snippets to export."),
        Ok((n, path)) => println!("Exported {} snippet(s) to {}", n, path.display()),
        Err(e) => eprintln!("Export failed: {e}"),
    }
    Ok(())
}

pub fn cmd_export_html(conn: &Connection, app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use crate::export::html::export_html;
    match export_html(conn, app_dir) {
        Ok((0, _)) => println!("No snippets to export."),
        Ok((n, path)) => println!("Exported {} snippet(s) to {}", n, path.display()),
        Err(e) => eprintln!("Export failed: {e}"),
    }
    Ok(())
}

/// Copy the raw SQLite DB into `<app_dir>/exports/snippets_<timestamp>.db`.
pub fn cmd_export_db(app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use crate::db::db_path;
    use chrono::Utc;

    let src = db_path(app_dir);
    if !src.exists() {
        eprintln!("No database found at {}", src.display());
        return Ok(());
    }

    let export_dir = app_dir.join("exports");
    std::fs::create_dir_all(&export_dir)?;
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let dst = export_dir.join(format!("snippets_{timestamp}.db"));
    std::fs::copy(&src, &dst)?;
    println!("Database exported to {}", dst.display());
    Ok(())
}

/// Import snippets from a file (Markdown, JSON, HTML, or raw SQLite DB) or bundled defaults.
pub fn cmd_import(
    conn: &mut Connection,
    source: Option<&str>,
    defaults: bool,
    preserve_ids: bool,
    id_format: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::export::import::import_snippets;

    let records = if defaults {
        crate::defaults::default_snippets()
    } else if let Some(src) = source {
        let path = std::path::Path::new(src);
        if !path.exists() {
            eprintln!("Error: File not found: {src}");
            return Ok(());
        }

        match crate::export::import::load_from_path(path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to load import file: {e}");
                return Ok(());
            }
        }
    } else {
        eprintln!("Import failed: provide a source file path or use --defaults.");
        return Ok(());
    };

    match import_snippets(conn, records, preserve_ids, id_format) {
        Ok(result) => {
            println!(
                "Imported {} snippet(s); skipped {}.",
                result.imported, result.skipped
            );
            let collisions: usize = result
                .id_map
                .iter()
                .filter(|(src, dst)| src != dst)
                .count();
            if collisions > 0 {
                println!("Regenerated {} colliding ID(s).", collisions);
            }
        }
        Err(e) => eprintln!("Import failed: {e}"),
    }
    Ok(())
}

