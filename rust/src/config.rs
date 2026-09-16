//! User configuration helpers for CPKB.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub const APP_VERSION: &str = "3.0.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomTheme {
    pub primary: String,
    pub secondary: String,
    pub warning: String,
    pub error: String,
    pub success: String,
    pub accent: String,
    pub foreground: String,
    pub background: String,
    pub surface: String,
    pub panel: String,
    pub boost: String,
    pub dark: bool,
}

impl Default for CustomTheme {
    fn default() -> Self {
        Self {
            primary: "#00ffff".to_string(),
            secondary: "#3399ff".to_string(),
            warning: "#fabd2f".to_string(),
            error: "#ff5555".to_string(),
            success: "#4EBF71".to_string(),
            accent: "#00ffff".to_string(),
            foreground: "#ffffff".to_string(),
            background: "#1e1e1e".to_string(),
            surface: "#252526".to_string(),
            panel: "#2d2d30".to_string(),
            boost: "#333333".to_string(),
            dark: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayConfig {
    pub theme: String,
    pub accent_color: String,
    pub left_pane_width: u32,
    pub layout: String,
    pub border_style: String,
    pub custom_theme: CustomTheme,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            theme: "textual-dark".to_string(),
            accent_color: "cyan".to_string(),
            left_pane_width: 35,
            layout: "horizontal".to_string(),
            border_style: "solid".to_string(),
            custom_theme: CustomTheme::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorConfig {
    pub command: String,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdFormatConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<serde_json::Value>, // "auto" or integer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnippetsConfig {
    pub max_number: u32,
    pub code_language: String,
    pub default_tags: String,
    pub default_id_format: String,
    pub id_formats: BTreeMap<String, IdFormatConfig>,
}

impl Default for SnippetsConfig {
    fn default() -> Self {
        let mut id_formats = BTreeMap::new();
        id_formats.insert(
            "default".to_string(),
            IdFormatConfig {
                prefix: Some("CP".to_string()),
                width: Some(serde_json::Value::String("auto".to_string())),
                pattern: None,
            },
        );
        id_formats.insert(
            "ms".to_string(),
            IdFormatConfig {
                prefix: Some("ms_".to_string()),
                width: Some(serde_json::Value::String("auto".to_string())),
                pattern: None,
            },
        );
        id_formats.insert(
            "latex".to_string(),
            IdFormatConfig {
                prefix: None,
                width: None,
                pattern: Some("LATEX-######".to_string()),
            },
        );

        Self {
            max_number: 9999,
            code_language: "cpp".to_string(),
            default_tags: String::new(),
            default_id_format: "default".to_string(),
            id_formats,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeybindingsConfig {
    pub quit: String,
    pub refresh: String,
    pub copy_snippet: String,
    pub focus_search: String,
    pub add_snippet: String,
    pub edit_snippet: String,
    pub use_snippet: String,
    pub delete_snippet: String,
    pub edit_tags: String,
    pub settings: String,
    pub scroll_detail_down: String,
    pub scroll_detail_up: String,
    pub page_detail_down: String,
    pub page_detail_up: String,
    pub shrink_left: String,
    pub grow_left: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            quit: "ctrl+q".to_string(),
            refresh: "ctrl+r".to_string(),
            copy_snippet: "ctrl+c".to_string(),
            focus_search: "/".to_string(),
            add_snippet: "ctrl+a".to_string(),
            edit_snippet: "ctrl+e".to_string(),
            use_snippet: "ctrl+u".to_string(),
            delete_snippet: "ctrl+d".to_string(),
            edit_tags: "ctrl+t".to_string(),
            settings: "ctrl+comma".to_string(),
            scroll_detail_down: "j".to_string(),
            scroll_detail_up: "k".to_string(),
            page_detail_down: "pagedown".to_string(),
            page_detail_up: "pageup".to_string(),
            shrink_left: "left_square_bracket".to_string(),
            grow_left: "right_square_bracket".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackupsConfig {
    pub max_backups: u32,
}

impl Default for BackupsConfig {
    fn default() -> Self {
        Self { max_backups: 25 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportsConfig {
    pub load_cpp_cheatsheet_on_setup: bool,
}

impl Default for ImportsConfig {
    fn default() -> Self {
        Self {
            load_cpp_cheatsheet_on_setup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EncryptionConfig {
    pub enabled: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub config_version: u32,
    pub app_version: String,
    pub default_language: String,
    pub editor: EditorConfig,
    pub display: DisplayConfig,
    pub snippets: SnippetsConfig,
    pub keybindings: KeybindingsConfig,
    pub backups: BackupsConfig,
    pub imports: ImportsConfig,
    pub encryption: EncryptionConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_version: 1,
            app_version: APP_VERSION.to_string(),
            default_language: "cpp".to_string(),
            editor: EditorConfig::default(),
            display: DisplayConfig::default(),
            snippets: SnippetsConfig::default(),
            keybindings: KeybindingsConfig::default(),
            backups: BackupsConfig::default(),
            imports: ImportsConfig::default(),
            encryption: EncryptionConfig::default(),
        }
    }
}

/// Return the application data directory path (~/.local/share/cpkb or $CPKB_DIR).
pub fn default_app_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var("CPKB_DIR") {
        if !override_dir.trim().is_empty() {
            return PathBuf::from(override_dir);
        }
    }
    if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("cpkb")
    } else {
        PathBuf::from(".cpkb")
    }
}

/// Return the config file path for an application directory.
pub fn config_path(app_dir: &Path) -> PathBuf {
    app_dir.join("config.json")
}

/// Helper to merge JSON values, overlaying saved values on top of defaults.
fn merge_json_values(default_val: &serde_json::Value, saved_val: &serde_json::Value) -> serde_json::Value {
    match (default_val, saved_val) {
        (serde_json::Value::Object(def_map), serde_json::Value::Object(saved_map)) => {
            let mut merged = def_map.clone();
            for (k, v) in saved_map {
                if let Some(existing) = merged.get(k) {
                    merged.insert(k.clone(), merge_json_values(existing, v));
                } else {
                    merged.insert(k.clone(), v.clone());
                }
            }
            serde_json::Value::Object(merged)
        }
        (_, saved) => saved.clone(),
    }
}

/// Load configuration from `app_dir`, falling back to default values when absent or invalid.
pub fn load_config(app_dir: &Path) -> Config {
    let path = config_path(app_dir);
    if !path.exists() {
        return Config::default();
    }

    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Config::default(),
    };

    let saved_json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Config::default(),
    };

    let default_json = match serde_json::to_value(Config::default()) {
        Ok(v) => v,
        Err(_) => return Config::default(),
    };

    let merged_json = merge_json_values(&default_json, &saved_json);
    serde_json::from_value(merged_json).unwrap_or_default()
}

/// Persist `config` to `app_dir` and return the written path.
pub fn save_config(app_dir: &Path, config: &Config) -> std::io::Result<PathBuf> {
    fs::create_dir_all(app_dir)?;
    let path = config_path(app_dir);
    let json_str = serde_json::to_string_pretty(config)? + "\n";
    fs::write(&path, json_str)?;
    Ok(path)
}

/// Return the configured maximum snippet count.
pub fn max_snippets(app_dir: &Path) -> u32 {
    load_config(app_dir).snippets.max_number.max(1)
}

/// Return the configured backup retention limit.
pub fn max_backups(app_dir: &Path) -> u32 {
    load_config(app_dir).backups.max_backups
}

/// Return whether encryption is enabled in configuration.
pub fn encryption_enabled(app_dir: &Path) -> bool {
    load_config(app_dir).encryption.enabled
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.config_version, 1);
        assert_eq!(config.default_language, "cpp");
        assert_eq!(config.snippets.max_number, 9999);
        assert_eq!(config.backups.max_backups, 25);
        assert!(config.snippets.id_formats.contains_key("default"));
        assert!(config.snippets.id_formats.contains_key("ms"));
        assert!(config.snippets.id_formats.contains_key("latex"));
    }

    #[test]
    fn test_save_and_load_config_roundtrip() {
        let dir = tempdir().unwrap();
        let mut config = Config::default();
        config.default_language = "python".to_string();
        config.display.theme = "dracula".to_string();
        config.snippets.max_number = 5000;

        let path = save_config(dir.path(), &config).unwrap();
        assert!(path.exists());

        let loaded = load_config(dir.path());
        assert_eq!(loaded.default_language, "python");
        assert_eq!(loaded.display.theme, "dracula");
        assert_eq!(loaded.snippets.max_number, 5000);
    }

    #[test]
    fn test_load_partial_config_merges_defaults() {
        let dir = tempdir().unwrap();
        let partial_json = r#"{
            "default_language": "rust",
            "display": {
                "theme": "nord"
            }
        }"#;
        fs::write(config_path(dir.path()), partial_json).unwrap();

        let loaded = load_config(dir.path());
        assert_eq!(loaded.default_language, "rust");
        assert_eq!(loaded.display.theme, "nord");
        // Preserved nested defaults:
        assert_eq!(loaded.display.accent_color, "cyan");
        assert_eq!(loaded.snippets.max_number, 9999);
    }
}
