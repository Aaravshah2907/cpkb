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

    /// Helper: insert a single snippet with minimal boilerplate.
    fn insert(conn: &mut rusqlite::Connection, id: &str, title: &str, lang: &str, code: &str) {
        insert_snippet_with_id(conn, id, title, "", "", "", code, lang, None, None)
            .expect("insert_snippet_with_id failed");
    }

    #[test]
    fn test_export_markdown_empty_db() {
        let conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();

        let (count, _path) = export_markdown(&conn, dir.path()).unwrap();

        assert_eq!(count, 0, "empty DB should export 0 snippets");
    }

    #[test]
    fn test_export_markdown_writes_file() {
        let mut conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();

        insert(&mut conn, "id1", "Alpha Snippet", "rust", "fn main() {}");
        insert(&mut conn, "id2", "Beta Snippet",  "go",   "package main");
        insert(&mut conn, "id3", "Gamma Snippet", "js",   "console.log(1)");

        let (count, path) = export_markdown(&conn, dir.path()).unwrap();

        assert_eq!(count, 3, "should export exactly 3 snippets");
        assert!(path.exists(), "output file must be created");

        let content = std::fs::read_to_string(&path).unwrap();
        for title in &["Alpha Snippet", "Beta Snippet", "Gamma Snippet"] {
            assert!(content.contains(title), "file should contain title '{title}'");
        }
        // Every snippet must have at least one language-tagged code fence
        for lang in &["rust", "go", "js"] {
            assert!(
                content.contains(&format!("```{lang}")),
                "file should contain ```{lang} fence"
            );
        }
    }

    #[test]
    fn test_export_markdown_language_fence() {
        let mut conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();

        insert(&mut conn, "py1", "Python Example", "python", "print('hi')");

        let (_count, path) = export_markdown(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();

        assert!(
            content.contains("```python"),
            "exported markdown must contain a ```python code fence"
        );
    }
}

#[cfg(test)]
mod tests_preexisting {
    use super::*;
    use tempfile::tempdir;
    use crate::db::{get_in_memory_conn, snippets::insert_snippet_with_id};

    fn seed(conn: &mut rusqlite::Connection, id: &str, title: &str, lang: &str, code: &str) {
        insert_snippet_with_id(conn, id, title, "desc", "use", "tag", code, lang, None, None).unwrap();
    }

    #[test]
    fn test_export_markdown_empty_db_preexisting() {
        let dir = tempdir().unwrap();
        let conn = get_in_memory_conn().unwrap();
        let (count, _path) = export_markdown(&conn, dir.path()).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_export_markdown_writes_file_and_counts() {
        let dir = tempdir().unwrap();
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "CP0001", "Alpha Sort",   "cpp",    "void a() {}");
        seed(&mut conn, "CP0002", "Beta Search",  "python", "def b(): pass");
        seed(&mut conn, "CP0003", "Gamma Tree",   "rust",   "fn g() {}");

        let (count, path) = export_markdown(&conn, dir.path()).unwrap();
        assert_eq!(count, 3);
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Alpha Sort"));
        assert!(content.contains("Beta Search"));
        assert!(content.contains("Gamma Tree"));
    }

    #[test]
    fn test_export_markdown_language_fence_preexisting() {
        let dir = tempdir().unwrap();
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "CP0001", "My Snippet", "python", "print('hi')");

        let (_n, path) = export_markdown(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("```python"), "Expected ```python fence:\n{content}");
        assert!(content.contains("print('hi')"));
    }

    #[test]
    fn test_export_markdown_id_in_header() {
        let dir = tempdir().unwrap();
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "LATEX-000001", "Euler", "tex", r"\e^{i\pi}");

        let (_n, path) = export_markdown(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        // Header must be: ## Title (ID)
        assert!(content.contains("## Euler (LATEX-000001)"));
    }

    #[test]
    fn test_export_markdown_empty_lang_produces_bare_fence() {
        let dir = tempdir().unwrap();
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "CP0001", "No Lang", "", "x = 1");

        let (_n, path) = export_markdown(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        // Opening fence should be ``` with nothing after it (no lang tag)
        assert!(content.contains("```\n"), "Expected bare ``` fence:\n{content}");
    }
}
