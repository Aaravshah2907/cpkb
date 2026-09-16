//! HTML export: writes all snippets to a self-contained, styled `.html` file.
//!
//! Matches the structure from Python's `cmd_export_html` but adds basic
//! syntax highlighting via a `<style>` block and `<code>` tags.

use std::path::Path;
use chrono::Utc;
use rusqlite::Connection;

use crate::db::snippets::list_all_snippets;

/// Escape HTML special characters to prevent XSS in the generated file.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Export every snippet to `<app_dir>/exports/snippets_<timestamp>.html`.
///
/// Returns the number of snippets written and the output path.
pub fn export_html(conn: &Connection, app_dir: &Path) -> anyhow::Result<(usize, std::path::PathBuf)> {
    let export_dir = app_dir.join("exports");
    std::fs::create_dir_all(&export_dir)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let out_path = export_dir.join(format!("snippets_{timestamp}.html"));

    let snippets = list_all_snippets(conn)?;
    let count = snippets.len();

    let mut html = String::with_capacity(count * 512 + 2048);

    // Minimal but clean stylesheet
    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>CPKB Export</title>
<style>
  body { font-family: system-ui, sans-serif; max-width: 900px; margin: 2rem auto; padding: 0 1rem; color: #222; }
  h1   { border-bottom: 2px solid #333; padding-bottom: .4rem; }
  h2   { margin-top: 2rem; color: #333; }
  .meta { font-size: .9rem; color: #555; margin-bottom: .5rem; }
  .tag { display: inline-block; background: #eef; border-radius: 4px; padding: 0 6px; margin-right: 4px; font-size: .8rem; }
  pre  { background: #1e1e2e; color: #cdd6f4; border-radius: 6px; padding: 1rem; overflow-x: auto; }
  code { font-family: "JetBrains Mono", "Fira Code", monospace; font-size: .9rem; }
  hr   { border: none; border-top: 1px solid #ddd; }
</style>
</head>
<body>
<h1>CPKB Knowledge Base</h1>
"#);

    html.push_str(&format!("<p class=\"meta\">Exported {} snippets on {}</p>\n", count, Utc::now().format("%Y-%m-%d %H:%M UTC")));

    for s in &snippets {
        let lang = s.language.as_deref().unwrap_or("");
        let tags_html: String = s
            .tags
            .as_deref()
            .unwrap_or("")
            .split(',')
            .map(|t| format!("<span class=\"tag\">{}</span>", escape_html(t.trim())))
            .collect::<Vec<_>>()
            .join(" ");

        html.push_str(&format!(
            "<section>\n\
             <h2>{} <small>({})</small></h2>\n\
             <div class=\"meta\">\
               <strong>Description:</strong> {} | \
               <strong>Use case:</strong> {} | \
               <strong>Language:</strong> {}\
             </div>\n\
             <div class=\"meta\"><strong>Tags:</strong> {}</div>\n\
             <pre><code class=\"language-{}\">{}</code></pre>\n\
             </section><hr/>\n",
            escape_html(&s.title),
            escape_html(&s.id),
            escape_html(s.description.as_deref().unwrap_or("")),
            escape_html(s.use_case.as_deref().unwrap_or("")),
            escape_html(lang),
            tags_html,
            escape_html(lang),
            escape_html(&s.code),
        ));
    }

    html.push_str("</body>\n</html>\n");
    std::fs::write(&out_path, html)?;
    Ok((count, out_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::db::{get_in_memory_conn, snippets::insert_snippet_with_id};

    /// Helper: insert a snippet with a given title (all other fields left minimal).
    fn insert(conn: &mut rusqlite::Connection, id: &str, title: &str) {
        insert_snippet_with_id(conn, id, title, "", "", "", "// code", "text", None, None)
            .expect("insert_snippet_with_id failed");
    }

    #[test]
    fn test_export_html_creates_file() {
        let mut conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();

        insert(&mut conn, "h1", "Snippet One");
        insert(&mut conn, "h2", "Snippet Two");

        let (count, path) = export_html(&conn, dir.path()).unwrap();

        assert_eq!(count, 2, "should report 2 exported snippets");
        assert!(path.exists(), "HTML output file must be created");

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("<!DOCTYPE html>"),
            "output must begin with a valid HTML5 doctype"
        );
    }

    #[test]
    fn test_export_html_escapes_special_chars() {
        let mut conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();

        // Title contains characters that must be HTML-escaped
        insert_snippet_with_id(
            &mut conn,
            "esc1",
            "A & B < C",
            "",
            "",
            "",
            "code",
            "text",
            None,
            None,
        )
        .unwrap();

        let (_count, path) = export_html(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();

        assert!(
            content.contains("&amp;"),
            "ampersand must be escaped to &amp; in HTML output"
        );
        assert!(
            content.contains("&lt;"),
            "less-than must be escaped to &lt; in HTML output"
        );
    }

    #[test]
    fn test_export_html_contains_snippet_titles() {
        let mut conn = get_in_memory_conn().unwrap();
        let dir = tempdir().unwrap();

        insert(&mut conn, "t1", "Alpha");
        insert(&mut conn, "t2", "Beta");

        let (_count, path) = export_html(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();

        assert!(
            content.contains("Alpha"),
            "HTML output must contain snippet title 'Alpha'"
        );
        assert!(
            content.contains("Beta"),
            "HTML output must contain snippet title 'Beta'"
        );
    }
}

