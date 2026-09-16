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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::db::{get_in_memory_conn, snippets::insert_snippet_with_id};

    fn seed(conn: &mut rusqlite::Connection, id: &str, title: &str, lang: &str, code: &str) {
        insert_snippet_with_id(conn, id, title, "desc", "use", "tags", code, lang, None, None)
            .expect("seed insert_snippet_with_id failed");
    }

    #[test]
    fn test_export_markdown_empty_db() {
        let conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();
        let (count, _path) = export_markdown(&conn, dir.path()).unwrap();
        assert_eq!(count, 0, "empty DB should export 0 snippets");
    }

    #[test]
    fn test_export_markdown_writes_file_and_counts() {
        let mut conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();
        seed(&mut conn, "CP0001", "Alpha Sort",  "cpp",    "void a() {}");
        seed(&mut conn, "CP0002", "Beta Search", "python", "def b(): pass");
        seed(&mut conn, "CP0003", "Gamma Tree",  "rust",   "fn g() {}");

        let (count, path) = export_markdown(&conn, dir.path()).unwrap();
        assert_eq!(count, 3);
        assert!(path.exists(), "output file must be created");
        let content = std::fs::read_to_string(&path).unwrap();
        for title in &["Alpha Sort", "Beta Search", "Gamma Tree"] {
            assert!(content.contains(title), "file must contain title '{title}'");
        }
        for lang in &["cpp", "python", "rust"] {
            assert!(
                content.contains(&format!("```{lang}")),
                "file must contain ```{lang} fence"
            );
        }
    }

    #[test]
    fn test_export_markdown_language_fence() {
        let mut conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();
        seed(&mut conn, "CP0001", "Python Example", "python", "print('hi')");

        let (_n, path) = export_markdown(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("```python"), "must have ```python fence:\n{content}");
        assert!(content.contains("print('hi')"));
    }

    #[test]
    fn test_export_markdown_id_in_header() {
        let mut conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();
        seed(&mut conn, "LATEX-000001", "Euler", "tex", r"\e^{i\pi}");

        let (_n, path) = export_markdown(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        // Header must be: ## Title (ID)
        assert!(content.contains("## Euler (LATEX-000001)"),
            "header must contain ID in parens:\n{content}");
    }

    #[test]
    fn test_export_markdown_empty_lang_produces_bare_fence() {
        let mut conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();
        seed(&mut conn, "CP0001", "No Lang", "", "x = 1");

        let (_n, path) = export_markdown(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        // Opening fence must be ``` with nothing after it (no lang suffix)
        assert!(content.contains("```\n"), "expected bare ``` fence:\n{content}");
    }
}
