//! Integration tests for the CLI layer — exercises the full stack from
//! argument parsing through DB reads and command output.

use cpkb::db::init_db;
use cpkb::db::snippets::{
    add_snippet, count_snippets, delete_snippet, get_snippet, list_all_snippets,
    list_snippets, recent_snippets, snippet_exists, update_snippet,
};
use cpkb::db::tags::{add_tag, remove_tag};
use cpkb::db::usages::{add_usage, get_usage, get_usages, update_usage};
use cpkb::db::search::{query_snippets, search_snippets};
use cpkb::export::import::{
    detect_format, import_snippets, load_from_path, load_json, load_markdown,
    ImportFormat, ImportRecord,
};
use tempfile::tempdir;

// ─── Snippet CRUD helpers ───────────────────────────────────────────────────

#[test]
fn test_add_and_retrieve_snippet() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(),
        "Merge Sort", "Stable divide-and-conquer", "sorting", "algo,sort",
        "void merge() {}", Some("cpp"), None,
    ).unwrap();

    assert_eq!(id, "CP0001");
    let snip = get_snippet(&conn, &id).unwrap().unwrap();
    assert_eq!(snip.title, "Merge Sort");
    assert_eq!(snip.language, "cpp");
    assert!(!snip.created_at.is_empty());
}

#[test]
fn test_snippet_sequential_ids() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    for i in 1..=5 {
        let id = add_snippet(
            &mut conn, dir.path(),
            &format!("Snippet {i}"), "", "", "", "code();", Some("cpp"), None,
        ).unwrap();
        assert_eq!(id, format!("CP000{i}"));
    }
}

#[test]
fn test_update_snippet() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(), "Old Title", "old desc", "", "", "v1", Some("rust"), None,
    ).unwrap();

    let updated = update_snippet(
        &mut conn, &id, "New Title", "new desc", "use case", "tag1", "v2", "rust",
    ).unwrap();
    assert!(updated);

    let snip = get_snippet(&conn, &id).unwrap().unwrap();
    assert_eq!(snip.title, "New Title");
    assert_eq!(snip.code, "v2");
}

#[test]
fn test_delete_snippet() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(), "To Delete", "", "", "", "x", Some("rust"), None,
    ).unwrap();

    assert!(snippet_exists(&conn, &id).unwrap());
    let deleted = delete_snippet(&conn, &id).unwrap();
    assert!(deleted);
    assert!(!snippet_exists(&conn, &id).unwrap());
    assert!(get_snippet(&conn, &id).unwrap().is_none());
}

#[test]
fn test_delete_nonexistent_snippet_returns_false() {
    let dir = tempdir().unwrap();
    let conn = init_db(dir.path()).unwrap();
    let deleted = delete_snippet(&conn, "GHOST-9999").unwrap();
    assert!(!deleted);
}

#[test]
fn test_count_snippets() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();
    assert_eq!(count_snippets(&conn).unwrap(), 0);

    for i in 1..=3 {
        add_snippet(
            &mut conn, dir.path(), &format!("S{i}"), "", "", "", "c", None, None,
        ).unwrap();
    }
    assert_eq!(count_snippets(&conn).unwrap(), 3);
}

#[test]
fn test_snippet_exists() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();
    assert!(!snippet_exists(&conn, "CP0001").unwrap());

    add_snippet(&mut conn, dir.path(), "X", "", "", "", "y", None, None).unwrap();
    assert!(snippet_exists(&conn, "CP0001").unwrap());
    assert!(!snippet_exists(&conn, "CP0002").unwrap());
}

#[test]
fn test_list_snippets_ordering() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    // Inserted in order A, B, C — list returns newest first
    for title in &["Alpha", "Beta", "Gamma"] {
        add_snippet(&mut conn, dir.path(), title, "", "", "", "code", None, None).unwrap();
    }

    let list = list_snippets(&conn).unwrap();
    assert_eq!(list.len(), 3);
    // list_snippets orders by created_at DESC → most recently inserted first
    assert_eq!(list[0].title, "Gamma");
    assert_eq!(list[2].title, "Alpha");
}

#[test]
fn test_list_all_snippets_ascending() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    for title in &["First", "Second"] {
        add_snippet(&mut conn, dir.path(), title, "", "", "", "x", None, None).unwrap();
    }

    // list_all_snippets orders ASC for export
    let all = list_all_snippets(&conn).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].title, "First");
    assert_eq!(all[1].title, "Second");
}

#[test]
fn test_recent_snippets_limit() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    for i in 1..=10 {
        add_snippet(
            &mut conn, dir.path(), &format!("Snip {i}"), "", "", "", "x", None, None,
        ).unwrap();
    }

    let recent = recent_snippets(&conn, 3).unwrap();
    assert_eq!(recent.len(), 3);
    // Most recent first
    assert_eq!(recent[0].title, "Snip 10");
}

#[test]
fn test_latex_auto_language() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(), "Euler Formula", "", "", "", r"\e^{i\pi} + 1 = 0", None, Some("latex"),
    ).unwrap();

    assert!(id.starts_with("LATEX-"));
    let snip = get_snippet(&conn, &id).unwrap().unwrap();
    // LaTeX snippets should be assigned language = "tex"
    assert_eq!(snip.language, "tex");
}

// ─── Tags ───────────────────────────────────────────────────────────────────

#[test]
fn test_tag_add_and_remove() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(), "Tagged Snippet", "", "", "algo", "x", None, None,
    ).unwrap();

    let merged = add_tag(&mut conn, &id, "dp").unwrap();
    assert!(merged.contains("dp"));

    // Adding duplicate should be idempotent
    let merged2 = add_tag(&mut conn, &id, "DP").unwrap(); // case-insensitive
    assert_eq!(merged, merged2, "Duplicate tag should not be re-added");

    let after_remove = remove_tag(&mut conn, &id, "dp").unwrap();
    assert!(after_remove.is_some());
    assert!(!after_remove.unwrap().contains("dp"));
}

#[test]
fn test_remove_nonexistent_tag_returns_none() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(), "Untagged", "", "", "", "code", None, None,
    ).unwrap();

    let result = remove_tag(&mut conn, &id, "ghost").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_add_tag_empty_fails() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(), "S", "", "", "", "x", None, None,
    ).unwrap();

    let result = add_tag(&mut conn, &id, "   ");
    assert!(result.is_err());
}

// ─── Search ─────────────────────────────────────────────────────────────────

#[test]
fn test_search_by_title_keyword() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    add_snippet(&mut conn, dir.path(), "Binary Search", "", "", "search,algo", "bs()", None, None).unwrap();
    add_snippet(&mut conn, dir.path(), "Merge Sort", "", "", "sort,algo", "ms()", None, None).unwrap();

    let results = search_snippets(&conn, "binary").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Binary Search");
}

#[test]
fn test_search_multiple_keywords_and_logic() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    add_snippet(&mut conn, dir.path(), "Quick Sort Impl", "fast in-place", "", "sort", "qs()", None, None).unwrap();
    add_snippet(&mut conn, dir.path(), "Binary Search Tree", "", "", "tree", "bst()", None, None).unwrap();

    // Both keywords must match the same snippet
    let results = search_snippets(&conn, "quick sort").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Quick Sort Impl");
}

#[test]
fn test_search_no_results() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    add_snippet(&mut conn, dir.path(), "Fibonacci", "", "", "dp", "fib()", None, None).unwrap();
    let results = search_snippets(&conn, "nonexistent_keyword_xyz").unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_search_in_code_field() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    add_snippet(
        &mut conn, dir.path(), "Unique Snippet", "", "", "", "VERY_UNIQUE_FUNCTION_NAME()", None, None,
    ).unwrap();

    let results = search_snippets(&conn, "VERY_UNIQUE_FUNCTION_NAME").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_query_snippets_limit() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    for i in 1..=10 {
        add_snippet(
            &mut conn, dir.path(), &format!("Algo {i}"), "", "", "algo", "x", None, None,
        ).unwrap();
    }

    let results = query_snippets(&conn, "algo", 3).unwrap();
    assert_eq!(results.len(), 3);
    // Returns (id, title) pairs
    assert!(!results[0].0.is_empty());
    assert!(results[0].1.starts_with("Algo"));
}

// ─── Usages ─────────────────────────────────────────────────────────────────

#[test]
fn test_add_and_get_usage() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(), "DFS", "", "", "graph", "dfs()", None, None,
    ).unwrap();

    let usage_id = add_usage(&conn, &id, "contest/A.cpp", "Problem A", "Used for traversal").unwrap();
    assert!(usage_id > 0);

    let usages = get_usages(&conn, &id).unwrap();
    assert_eq!(usages.len(), 1);
    assert_eq!(usages[0].file_path, "contest/A.cpp");
    assert_eq!(usages[0].problem_name, "Problem A");
    assert_eq!(usages[0].notes, "Used for traversal");
}

#[test]
fn test_update_usage() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(), "BFS", "", "", "graph", "bfs()", None, None,
    ).unwrap();
    let usage_id = add_usage(&conn, &id, "old/path.cpp", "Old Problem", "").unwrap();

    let updated = update_usage(&conn, usage_id, "new/path.cpp", "New Problem", "updated note").unwrap();
    assert!(updated);

    let usage = get_usage(&conn, usage_id).unwrap().unwrap();
    assert_eq!(usage.file_path, "new/path.cpp");
    assert_eq!(usage.problem_name, "New Problem");
    assert_eq!(usage.notes, "updated note");
}

#[test]
fn test_get_usages_empty() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = add_snippet(
        &mut conn, dir.path(), "Unused Snippet", "", "", "", "x", None, None,
    ).unwrap();

    let usages = get_usages(&conn, &id).unwrap();
    assert!(usages.is_empty());
}

#[test]
fn test_get_nonexistent_usage_returns_none() {
    let dir = tempdir().unwrap();
    let conn = init_db(dir.path()).unwrap();
    let usage = get_usage(&conn, 9999).unwrap();
    assert!(usage.is_none());
}

// ─── Import engine ─────────────────────────────────────────────────────────

#[test]
fn test_detect_format_case_insensitive() {
    assert_eq!(detect_format("FILE.JSON"), ImportFormat::Json);
    assert_eq!(detect_format("FILE.MD"), ImportFormat::Markdown);
    assert_eq!(detect_format("FILE.HTML"), ImportFormat::Html);
    assert_eq!(detect_format("FILE.DB"), ImportFormat::Db);
    assert_eq!(detect_format("FILE.SQLITE"), ImportFormat::Db);
    assert_eq!(detect_format("noext"), ImportFormat::Markdown);
}

#[test]
fn test_load_markdown_multiple_snippets() {
    let md = "\
## Alpha (CP0001)
**Description:** First snippet
**Use case:** testing
**Tags:** tag1

```rust
fn alpha() {}
```

## Beta (CP0002)
**Description:** Second snippet
**Use case:** more testing
**Tags:** tag2

```python
def beta(): pass
```
";
    let records = load_markdown(md);
    assert_eq!(records.len(), 2);

    assert_eq!(records[0].id.as_deref(), Some("CP0001"));
    assert_eq!(records[0].title, "Alpha");
    assert_eq!(records[0].language.as_deref(), Some("rust"));
    assert!(records[0].code.contains("alpha"));

    assert_eq!(records[1].id.as_deref(), Some("CP0002"));
    assert_eq!(records[1].title, "Beta");
    assert_eq!(records[1].language.as_deref(), Some("python"));
    assert!(records[1].code.contains("beta"));
}

#[test]
fn test_load_markdown_no_id_in_header() {
    // Headers without a parenthesized ID should still parse
    let md = "## My Snippet\n**Tags:** x\n\n```cpp\nvoid foo() {}\n```\n";
    let records = load_markdown(md);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].title, "My Snippet");
    assert!(records[0].id.is_none());
    assert!(records[0].code.contains("foo"));
}

#[test]
fn test_load_markdown_empty_input() {
    let records = load_markdown("");
    assert!(records.is_empty());
}

#[test]
fn test_load_json_multiple_snippets() {
    let json = r#"[
        {"id": "CP0001", "title": "Alpha", "code": "a()", "language": "python"},
        {"id": "CP0002", "title": "Beta",  "code": "b()", "language": "rust"}
    ]"#;
    let records = load_json(json.as_bytes()).unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].id.as_deref(), Some("CP0001"));
    assert_eq!(records[1].language.as_deref(), Some("rust"));
}

#[test]
fn test_load_json_missing_optional_fields() {
    // Only id, title, code are required; everything else defaults
    let json = r#"[{"id":"X1","title":"Minimal","code":"x()"}]"#;
    let records = load_json(json.as_bytes()).unwrap();
    assert_eq!(records.len(), 1);
    assert!(records[0].description.is_none());
    assert!(records[0].tags.is_none());
}

#[test]
fn test_load_json_invalid_returns_err() {
    let bad = b"not json at all {{{";
    assert!(load_json(bad).is_err());
}

#[test]
fn test_import_preserves_ids() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let records = vec![
        ImportRecord {
            id: Some("MYFORMAT-001".to_string()),
            title: "Preserved".to_string(),
            code: "p()".to_string(),
            ..Default::default()
        },
    ];

    let result = import_snippets(&mut conn, records, true, None).unwrap();
    assert_eq!(result.imported, 1);
    // ID should be preserved as-is
    assert!(snippet_exists(&conn, "MYFORMAT-001").unwrap());
}

#[test]
fn test_import_regenerates_on_collision() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    // Pre-insert a snippet with a known ID
    add_snippet(&mut conn, dir.path(), "Existing", "", "", "", "x", None, None).unwrap();
    // CP0001 now exists — importing another with CP0001 should regenerate its ID
    let records = vec![ImportRecord {
        id: Some("CP0001".to_string()),
        title: "Collision".to_string(),
        code: "collision()".to_string(),
        ..Default::default()
    }];

    let result = import_snippets(&mut conn, records, true, None).unwrap();
    assert_eq!(result.imported, 1);
    // The map should show the ID was regenerated (src != dst)
    let new_id = result.id_map.get("CP0001").unwrap();
    assert_ne!(new_id, "CP0001");
    assert_eq!(count_snippets(&conn).unwrap(), 2);
}

#[test]
fn test_import_skips_blank_title_or_code() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let records = vec![
        ImportRecord { title: "   ".to_string(), code: "x()".to_string(), ..Default::default() },
        ImportRecord { title: "Good".to_string(), code: "  ".to_string(), ..Default::default() },
        ImportRecord { title: "Valid".to_string(), code: "ok()".to_string(), ..Default::default() },
    ];

    let result = import_snippets(&mut conn, records, false, None).unwrap();
    assert_eq!(result.imported, 1);
    assert_eq!(result.skipped, 2);
}

#[test]
fn test_import_with_regenerate_ids() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let records = vec![
        ImportRecord {
            id: Some("ORIG-001".to_string()),
            title: "Regen".to_string(),
            code: "r()".to_string(),
            ..Default::default()
        },
    ];
    // preserve_ids=false → always regenerate
    let result = import_snippets(&mut conn, records, false, None).unwrap();
    assert_eq!(result.imported, 1);
    assert!(!snippet_exists(&conn, "ORIG-001").unwrap());
}

// ─── Export → Import roundtrip ──────────────────────────────────────────────

#[test]
fn test_markdown_export_import_roundtrip() {
    use cpkb::export::markdown::export_markdown;

    let src_dir = tempdir().unwrap();
    let mut conn = init_db(src_dir.path()).unwrap();

    add_snippet(&mut conn, src_dir.path(), "Alpha Func", "desc A", "use A", "tag_a", "fn alpha() {}", Some("rust"), None).unwrap();
    add_snippet(&mut conn, src_dir.path(), "Beta Func",  "desc B", "use B", "tag_b", "def beta(): pass", Some("python"), None).unwrap();

    let (count, md_path) = export_markdown(&conn, src_dir.path()).unwrap();
    assert_eq!(count, 2);
    assert!(md_path.exists());

    // Import back into a fresh DB
    let dst_dir = tempdir().unwrap();
    let mut dst_conn = init_db(dst_dir.path()).unwrap();
    let records = load_from_path(&md_path).unwrap();
    assert_eq!(records.len(), 2);

    let result = import_snippets(&mut dst_conn, records, true, None).unwrap();
    assert_eq!(result.imported, 2);
    assert_eq!(count_snippets(&dst_conn).unwrap(), 2);
}

#[test]
fn test_json_export_import_roundtrip() {
    use cpkb::export::json::export_json;

    let src_dir = tempdir().unwrap();
    let mut conn = init_db(src_dir.path()).unwrap();

    add_snippet(&mut conn, src_dir.path(), "Gamma", "d", "u", "t", "g()", Some("cpp"), None).unwrap();

    let (count, json_path) = export_json(&conn, src_dir.path()).unwrap();
    assert_eq!(count, 1);

    let dst_dir = tempdir().unwrap();
    let mut dst_conn = init_db(dst_dir.path()).unwrap();
    let records = load_from_path(&json_path).unwrap();
    let result = import_snippets(&mut dst_conn, records, true, None).unwrap();
    assert_eq!(result.imported, 1);

    let snip = get_snippet(&dst_conn, "CP0001").unwrap().unwrap();
    assert_eq!(snip.title, "Gamma");
    assert_eq!(snip.language, "cpp");
}

// ─── DB path helper ─────────────────────────────────────────────────────────

#[test]
fn test_db_path_uses_snippets_db() {
    use std::path::Path;
    use cpkb::db::db_path;
    let path = db_path(Path::new("/tmp/cpkb_test"));
    assert_eq!(path.file_name().unwrap(), "snippets.db");
}
