//! Shell completions generator for CPKB CLI (Bash, Zsh, Fish, PowerShell, Elvish).

use std::io;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::cli::Cli;

/// Generate shell completion script to stdout for the specified shell.
pub fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let bin_name = "cpkb";
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}
