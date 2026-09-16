use cpkb::config::load_config;
use cpkb::db::init_db;
use cpkb::db::snippets::{get_snippet, list_snippets};
use tempfile::tempdir;

#[test]
fn test_cli_add_and_show_snippet() {
    let dir = tempdir().unwrap();
    let mut conn = init_db(dir.path()).unwrap();

    let id = cpkb::db::snippets::add_snippet(
        &mut conn,
        dir.path(),
        "Quicksort in Rust",
        "Divide and conquer sort",
        "Sorting slices",
        "algo, sort, rust",
        "pub fn quicksort() {}",
        Some("rust"),
        None,
    )
    .unwrap();

    assert_eq!(id, "CP0001");

    let snip = get_snippet(&conn, &id).unwrap().unwrap();
    assert_eq!(snip.title, "Quicksort in Rust");
    assert_eq!(snip.language, "rust");

    let list = list_snippets(&conn).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "CP0001");
}

#[test]
fn test_cli_id_format_management() {
    let dir = tempdir().unwrap();
    let mut config = load_config(dir.path());

    config.snippets.id_formats.insert(
        "test".to_string(),
        cpkb::config::IdFormatConfig {
            prefix: None,
            width: None,
            pattern: Some("TEST-####".to_string()),
            color: None,
        },
    );
    cpkb::config::save_config(dir.path(), &config).unwrap();

    let mut conn = init_db(dir.path()).unwrap();
    let id = cpkb::db::snippets::add_snippet(
        &mut conn,
        dir.path(),
        "Test Snippet",
        "",
        "",
        "",
        "code",
        Some("cpp"),
        Some("test"),
    )
    .unwrap();

    assert_eq!(id, "TEST-0001");
}
