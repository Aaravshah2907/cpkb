//! External editor buffer invocations for snippet and usage editing.

use std::fs;
use std::path::Path;
use std::process::Command;
use crate::config::load_config;
use crate::db::snippets::Snippet;
use crate::db::usages::Usage;

pub fn configured_editor_command(app_dir: &Path) -> Vec<String> {
    let config = load_config(app_dir);
    let configured = config.editor.command.trim();
    if !configured.is_empty() {
        return configured.split_whitespace().map(|s| s.to_string()).collect();
    }
    if let Ok(visual) = std::env::var("VISUAL") {
        if !visual.trim().is_empty() {
            return visual.split_whitespace().map(|s| s.to_string()).collect();
        }
    }
    if let Ok(editor) = std::env::var("EDITOR") {
        if !editor.trim().is_empty() {
            return editor.split_whitespace().map(|s| s.to_string()).collect();
        }
    }
    if cfg!(windows) {
        vec!["notepad".to_string()]
    } else {
        vec!["nano".to_string()]
    }
}

pub struct EditedSnippet {
    pub title: String,
    pub description: String,
    pub use_case: String,
    pub tags: String,
    pub language: String,
    pub code: String,
}

pub fn open_snippet_editor(app_dir: &Path, snip: &Snippet) -> Result<Option<EditedSnippet>, Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().join("snippet_edit.md");

    let initial_content = format!(
        "Title: {}\nDescription: {}\nUse case: {}\nTags: {}\nLanguage: {}\n---\n{}\n",
        snip.title, snip.description, snip.use_case, snip.tags, snip.language, snip.code
    );
    fs::write(&temp_path, initial_content)?;

    let editor_cmd = configured_editor_command(app_dir);
    let status = Command::new(&editor_cmd[0])
        .args(&editor_cmd[1..])
        .arg(&temp_path)
        .status()?;

    if !status.success() {
        return Err("Editor exited with an error".into());
    }

    let modified = fs::read_to_string(&temp_path)?;
    let parts: Vec<&str> = modified.splitn(2, "---").collect();
    if parts.len() != 2 {
        return Err("Invalid format after edit: missing '---' separator.".into());
    }

    let meta_lines = parts[0].lines();
    let new_code = parts[1].trim().to_string();

    let mut new_title = String::new();
    let mut new_desc = String::new();
    let mut new_use_case = String::new();
    let mut new_tags = String::new();
    let mut new_lang = snip.language.clone();

    for line in meta_lines {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if lower.starts_with("title:") {
            new_title = trimmed[6..].trim().to_string();
        } else if lower.starts_with("description:") {
            new_desc = trimmed[12..].trim().to_string();
        } else if lower.starts_with("use case:") {
            new_use_case = trimmed[9..].trim().to_string();
        } else if lower.starts_with("tags:") {
            new_tags = trimmed[5..].trim().to_string();
        } else if lower.starts_with("language:") {
            new_lang = trimmed[9..].trim().to_string();
        }
    }

    if new_title.is_empty() || new_code.is_empty() {
        return Err("Title and code cannot be empty.".into());
    }

    Ok(Some(EditedSnippet {
        title: new_title,
        description: new_desc,
        use_case: new_use_case,
        tags: new_tags,
        language: new_lang,
        code: new_code,
    }))
}

pub fn open_snippet_creator(app_dir: &Path, default_lang: &str) -> Result<Option<EditedSnippet>, Box<dyn std::error::Error>> {
    let dummy = Snippet {
        id: "".to_string(),
        title: "".to_string(),
        description: "".to_string(),
        use_case: "".to_string(),
        tags: "".to_string(),
        code: "".to_string(),
        language: default_lang.to_string(),
        created_at: "".to_string(),
        updated_at: "".to_string(),
    };
    open_snippet_editor(app_dir, &dummy)
}

pub struct EditedUsage {
    pub file_path: String,
    pub problem_name: String,
    pub notes: String,
}

pub fn open_usage_editor(app_dir: &Path, usage: &Usage) -> Result<Option<EditedUsage>, Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().join("usage_edit.md");

    let initial_content = format!(
        "File: {}\nProblem: {}\n---\n{}\n",
        usage.file_path, usage.problem_name, usage.notes
    );
    fs::write(&temp_path, initial_content)?;

    let editor_cmd = configured_editor_command(app_dir);
    let status = Command::new(&editor_cmd[0])
        .args(&editor_cmd[1..])
        .arg(&temp_path)
        .status()?;

    if !status.success() {
        return Err("Editor exited with an error".into());
    }

    let modified = fs::read_to_string(&temp_path)?;
    let parts: Vec<&str> = modified.splitn(2, "---").collect();
    if parts.len() != 2 {
        return Err("Invalid format after edit: missing '---' separator.".into());
    }

    let meta_lines = parts[0].lines();
    let new_notes = parts[1].trim().to_string();

    let mut new_file = usage.file_path.clone();
    let mut new_prob = usage.problem_name.clone();

    for line in meta_lines {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if lower.starts_with("file:") {
            new_file = trimmed[5..].trim().to_string();
        } else if lower.starts_with("problem:") {
            new_prob = trimmed[8..].trim().to_string();
        }
    }

    Ok(Some(EditedUsage {
        file_path: new_file,
        problem_name: new_prob,
        notes: new_notes,
    }))
}
