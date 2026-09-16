//! Full-text and multi-keyword search queries for CPKB.

use rusqlite::{Connection, Result};
use crate::db::snippets::SnippetSummary;

/// Multi-keyword AND search returning snippet summaries.
pub fn search_snippets(conn: &Connection, query: &str) -> Result<Vec<SnippetSummary>> {
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.is_empty() {
        return crate::db::snippets::list_snippets(conn);
    }

    let mut sql = "SELECT id, title, tags, language FROM snippets WHERE ".to_string();
    let mut clauses = Vec::new();
    for i in 1..=words.len() {
        clauses.push(format!(
            "(id LIKE ?{i} OR title LIKE ?{i} OR description LIKE ?{i} OR use_case LIKE ?{i} OR tags LIKE ?{i} OR code LIKE ?{i})"
        ));
    }
    sql.push_str(&clauses.join(" AND "));
    sql.push_str(" ORDER BY created_at DESC");

    let patterns: Vec<String> = words.into_iter().map(|w| format!("%{}%", w)).collect();
    let params_ref: Vec<&dyn rusqlite::ToSql> = patterns.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_ref.as_slice(), |row| {
        Ok(SnippetSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            tags: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            language: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Scripting-friendly search returning pairs of (id, title) limited by `limit`.
pub fn query_snippets(conn: &Connection, query: &str, limit: u32) -> Result<Vec<(String, String)>> {
    let summaries = search_snippets(conn, query)?;
    Ok(summaries
        .into_iter()
        .take(limit as usize)
        .map(|s| (s.id, s.title))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{get_in_memory_conn, snippets::insert_snippet_with_id};

    fn seed(conn: &mut rusqlite::Connection, id: &str, title: &str, desc: &str, tags: &str, code: &str) {
        insert_snippet_with_id(conn, id, title, desc, "", tags, code, "cpp", None, None).unwrap();
    }

    #[test]
    fn test_search_empty_query_returns_all() {
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "S1", "Alpha", "", "tag1", "a()");
        seed(&mut conn, "S2", "Beta",  "", "tag2", "b()");
        // Empty query should fall through to list_snippets
        let results = search_snippets(&conn, "").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_title() {
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "S1", "Binary Search", "search in sorted array", "search,algo", "bs()");
        seed(&mut conn, "S2", "Merge Sort",    "stable sort",            "sort,algo",   "ms()");

        let results = search_snippets(&conn, "binary").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "S1");
    }

    #[test]
    fn test_search_by_tag() {
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "S1", "Dijkstra", "", "graph,shortest_path", "dijkstra()");
        seed(&mut conn, "S2", "BFS",      "", "graph,bfs",           "bfs()");

        let results = search_snippets(&conn, "shortest_path").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "S1");
    }

    #[test]
    fn test_search_by_code() {
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "S1", "Snippet", "", "", "VERY_UNIQUE_IDENTIFIER_xyz()");

        let results = search_snippets(&conn, "VERY_UNIQUE_IDENTIFIER_xyz").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_multi_keyword_and_logic() {
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "S1", "Quick Sort Impl", "fast in-place", "sort", "qs()");
        seed(&mut conn, "S2", "Binary Search",   "",              "algo", "bs()");

        // Both "quick" and "sort" must match the same snippet
        let results = search_snippets(&conn, "quick sort").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "S1");
    }

    #[test]
    fn test_search_multi_keyword_no_partial_match() {
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "S1", "Fibonacci", "dp table", "dp", "fib()");

        // "fibonacci" matches but "xyz" doesn't → no results
        let results = search_snippets(&conn, "fibonacci xyz").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_no_results() {
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "S1", "Trie", "", "string,trie", "trie()");

        let results = search_snippets(&conn, "nonexistent_keyword_abc123").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_query_snippets_respects_limit() {
        let mut conn = get_in_memory_conn().unwrap();
        for i in 1..=8 {
            seed(&mut conn, &format!("S{i}"), &format!("Algo {i}"), "", "algo", "x()");
        }
        let results = query_snippets(&conn, "algo", 3).unwrap();
        assert_eq!(results.len(), 3);
        // Each entry is a (id, title) tuple
        assert!(!results[0].0.is_empty());
        assert!(results[0].1.starts_with("Algo"));
    }

    #[test]
    fn test_query_snippets_limit_larger_than_results() {
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "S1", "Only One", "", "unique_q", "x()");

        let results = query_snippets(&conn, "unique_q", 100).unwrap();
        assert_eq!(results.len(), 1);
    }
}

