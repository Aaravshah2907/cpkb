//! Markdown export: writes all snippets to a timestamped `.md` file.

use std::path::Path;
use chrono::Utc;
use rusqlite::Connection;

use crate::db::snippets::list_all_snippets;

/// Export every snippet to `<app_dir>/exports/snippets_<timestamp>.md`.
///
/// Returns the number of snippets written and the output path.
pub fn export_markdown(conn: &Connection, app_dir: &Path) -> anyhow::Result<(usize, std::path::PathBuf)> {
    let export_dir = app_dir.join("exports");
    std::fs::create_dir_all(&export_dir)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let out_path = export_dir.join(format!("snippets_{timestamp}.md"));

    let snippets = list_all_snippets(conn)?;
    let count = snippets.len();

    if count == 0 {
        return Ok((0, out_path));
    }

    let mut lines = Vec::with_capacity(count * 10);
    for s in &snippets {
        // Header: ## Title (ID)
        lines.push(format!("## {} ({})", s.title, s.id));
        lines.push(format!("**Description:** {}", s.description.as_deref().unwrap_or("")));
        lines.push(format!("**Use case:** {}", s.use_case.as_deref().unwrap_or("")));
        lines.push(format!("**Tags:** {}", s.tags.as_deref().unwrap_or("")));
        // Language-tagged code fence
        let lang = s.language.as_deref().unwrap_or("");
        lines.push(format!("\n```{lang}"));
        lines.push(s.code.clone());
        lines.push("```\n".to_string());
    }

    std::fs::write(&out_path, lines.join("\n"))?;
    Ok((count, out_path))
}
