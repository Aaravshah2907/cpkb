//! Clap command line argument parser and dispatcher for CPKB.

pub mod commands;
pub mod completions;
pub mod editor;

use std::path::Path;
use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
use rusqlite::Connection;

#[derive(Parser, Debug)]
#[command(name = "cpkb", version, about = "Competitive Programming Knowledge Base", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch interactive Terminal User Interface (TUI)
    Tui,

    /// Add a new snippet with interactive prompts or options
    Add {
        /// Programming language for the snippet (e.g. cpp, python, rust, tex, md)
        #[arg(short, long)]
        language: Option<String>,

        /// Configured ID format pattern name (e.g. default, ms, Algo, latex)
        #[arg(long)]
        id_format: Option<String>,
    },

    /// List all snippets in the knowledge base
    List {
        /// Sort snippets by field: id, name, or date
        #[arg(short, long, default_value = "date")]
        sort: String,

        /// Output snippets formatted as JSON
        #[arg(long)]
        json: bool,
    },


    /// Show snippet details, metadata, and code
    Show {
        /// Snippet ID (e.g., CP0001, ms_00001, LATEX-000014)
        id: String,

        /// Output snippet metadata and code formatted as JSON
        #[arg(long)]
        json: bool,
    },

    /// Search snippets by keywords across title, description, tags, and code
    Search {
        /// Search query (multiple keywords are combined with AND logic)
        query: String,

        /// Output search results formatted as JSON
        #[arg(long)]
        json: bool,
    },

    /// Search snippets and output pipeline-friendly delimited rows (id | title)
    Query {
        /// Search query string (default: empty to return all snippets)
        #[arg(default_value = "")]
        query: String,

        /// Maximum number of results to return (default: 5)
        #[arg(long, default_value_t = 5)]
        limit: u32,
    },

    /// Record the usage of a snippet in an external source file or problem
    Use {
        /// Snippet ID
        id: String,

        /// File path or problem identifier where the snippet was used
        file: String,
    },

    /// List all recorded usage records and problem references for a snippet
    Usages {
        /// Snippet ID
        id: String,
    },

    /// Show knowledge base summary statistics (snippets, usages, tags, languages)
    Stats,

    /// Fetch and display a randomly selected snippet
    Random,

    /// Edit snippet metadata, language, and code in default $EDITOR
    Edit {
        /// Snippet ID to edit
        id: String,
    },

    /// Edit a specific usage record in default $EDITOR
    EditUsage {
        /// Usage record integer ID
        id: i64,
    },

    /// Delete a snippet and its associated tags/usages
    Delete {
        /// Snippet ID to delete
        id: String,
    },

    /// Add a tag to a snippet
    TagAdd {
        /// Snippet ID
        id: String,

        /// Tag keyword to add
        tag: String,
    },

    /// Remove a tag from a snippet
    TagRemove {
        /// Snippet ID
        id: String,

        /// Tag keyword to remove
        tag: String,
    },

    /// Show the most recently created or updated snippets
    Recent {
        /// Number of snippets to display (default: 10)
        #[arg(short = 'n', long, default_value_t = 10)]
        limit: u32,
    },

    /// Copy a snippet to the system clipboard or append to a file
    Copy {
        /// Snippet ID to copy
        id: String,

        /// File to append to (optional)
        #[arg(short, long)]
        file: Option<String>,
    },

    /// Display active configuration settings and storage paths
    Config,

    /// Create a manual timestamped backup of the database
    Backup,

    /// Manage custom sequential snippet ID formats and patterns
    IdFormat(IdFormatArgs),

    /// Launch an interactive spaced-repetition revision session (SM-2)
    Revise,

    /// Show spaced-repetition revision statistics and retention schedule
    SrsStats,

    /// Export all snippets to a Markdown file (mirrors Python `export`)
    Export,

    /// Export all snippets to a JSON file
    ExportJson,

    /// Export all snippets to a styled HTML file
    ExportHtml,

    /// Export the raw SQLite database file to the exports directory
    ExportDb,

    /// Import snippets from a Markdown, JSON, HTML, or SQLite DB file (or bundled defaults)
    Import {
        /// Path to the import source file
        source: Option<String>,

        /// Import bundled standard C++ STL and algorithm cheatsheets
        #[arg(long)]
        defaults: bool,

        /// Regenerate IDs instead of preserving source IDs
        #[arg(long)]
        regenerate_ids: bool,

        /// Override the ID format for newly generated IDs (e.g. default, ms, latex)
        #[arg(long)]
        id_format: Option<String>,
    },

    /// Generate shell completion scripts (bash, zsh, fish, powershell, elvish)
    Completions {
        /// Target shell type
        shell: Shell,
    },

    /// Automatically detect active shell and install completion script
    InstallCompletions,
}

#[derive(Args, Debug)]
pub struct IdFormatArgs {
    #[command(subcommand)]
    pub command: IdFormatCommands,
}

#[derive(Subcommand, Debug)]
pub enum IdFormatCommands {
    /// List configured ID formats and patterns
    List,

    /// Add or update a custom ID format pattern
    Add {
        /// Format name identifier (e.g. Algo, latex, ms, custom)
        name: String,

        /// ID pattern with # placeholders (e.g. NOTE-###, ALG_####, LATEX-######)
        #[arg(long)]
        pattern: Option<String>,

        /// Legacy ID prefix string (e.g. NOTE-)
        #[arg(long)]
        prefix: Option<String>,

        /// Set this format as the default for new snippets
        #[arg(long)]
        default: bool,
    },

    /// Set the default ID format for new snippets
    Default {
        /// Configured format name to set as default
        name: String,
    },
}

/// Execute CLI subcommand against active database and application directory.
pub fn run_cli(cli: Cli, conn: &mut Connection, app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = cli.command.unwrap_or(Commands::Tui);
    match cmd {
        Commands::Tui => crate::tui::run_tui(conn, app_dir),
        Commands::List { sort, json } => commands::cmd_list(conn, &sort, json),
        Commands::Recent { limit } => commands::cmd_recent(conn, limit),

        Commands::Search { query, json } => commands::cmd_search(conn, &query, json),
        Commands::Query { query, limit } => commands::cmd_query(conn, &query, limit),
        Commands::Show { id, json } => commands::cmd_show(conn, &id, json),
        Commands::Add { language, id_format } => {
            commands::cmd_add(conn, app_dir, language.as_deref(), id_format.as_deref())
        }
        Commands::Edit { id } => commands::cmd_edit(conn, app_dir, &id),
        Commands::Delete { id } => commands::cmd_delete(conn, &id),
        Commands::TagAdd { id, tag } => commands::cmd_tag_add(conn, &id, &tag),
        Commands::TagRemove { id, tag } => commands::cmd_tag_remove(conn, &id, &tag),
        Commands::Use { id, file } => commands::cmd_use(conn, &id, &file),
        Commands::Usages { id } => commands::cmd_usages(conn, &id),
        Commands::EditUsage { id } => commands::cmd_edit_usage(conn, app_dir, id),
        Commands::Copy { id, file } => commands::cmd_copy(conn, &id, file.as_deref()),
        Commands::Stats => commands::cmd_stats(conn),
        Commands::Random => commands::cmd_random(conn),
        Commands::Backup => commands::cmd_backup(app_dir),
        Commands::Config => commands::cmd_config(app_dir),
        Commands::Revise => commands::cmd_revise(conn),
        Commands::SrsStats => commands::cmd_srs_stats(conn),
        Commands::IdFormat(args) => match args.command {
            IdFormatCommands::List => commands::cmd_id_format_list(app_dir),
            IdFormatCommands::Add { name, pattern, prefix, default } => {
                commands::cmd_id_format_add(app_dir, &name, pattern.as_deref(), prefix.as_deref(), default)
            }
            IdFormatCommands::Default { name } => commands::cmd_id_format_default(app_dir, &name),
        },
        Commands::Export => commands::cmd_export(conn, app_dir),
        Commands::ExportJson => commands::cmd_export_json(conn, app_dir),
        Commands::ExportHtml => commands::cmd_export_html(conn, app_dir),
        Commands::ExportDb => commands::cmd_export_db(app_dir),
        Commands::Import { source, defaults, regenerate_ids, id_format } => {
            commands::cmd_import(conn, source.as_deref(), defaults, !regenerate_ids, id_format.as_deref())
        }
        Commands::Completions { shell } => {
            completions::generate_completions(shell);
            Ok(())
        }
        Commands::InstallCompletions => {
            completions::install_completions(app_dir)
        }
    }
}
