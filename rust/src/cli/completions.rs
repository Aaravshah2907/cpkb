//! Shell completions generator and auto-installer for CPKB CLI.

use std::fs;
use std::io;
use std::path::Path;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::cli::Cli;

/// Generate shell completion script to stdout for the specified shell.
pub fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let bin_name = "cpkb";
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}

/// Automatically detect active shell and install completion script.
pub fn install_completions(app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let shell_env = std::env::var("SHELL").unwrap_or_default();
    let (shell_name, shell_type) = if shell_env.contains("zsh") {
        ("zsh", Shell::Zsh)
    } else if shell_env.contains("bash") {
        ("bash", Shell::Bash)
    } else if shell_env.contains("fish") {
        ("fish", Shell::Fish)
    } else {
        eprintln!("Unsupported or undetected shell: {}", shell_env);
        eprintln!("Please generate completions manually using: cpkb completions <bash|zsh|fish>");
        return Ok(());
    };

    println!("Detected active shell: {}", shell_name);

    // Prepare completions directory: ~/.local/share/cpkb/completions
    let comp_dir = app_dir.join("completions");
    fs::create_dir_all(&comp_dir)?;
    let comp_file = comp_dir.join(format!("cpkb.{}", shell_name));

    // Generate script into the file
    let mut file = fs::File::create(&comp_file)?;
    let mut cmd = Cli::command();
    generate(shell_type, &mut cmd, "cpkb", &mut file);

    let home = dirs::home_dir().ok_or("Could not determine user home directory")?;

    match shell_name {
        "zsh" => {
            let rc_file = home.join(".zshrc");
            let source_line = format!("source \"{}\"", comp_file.display());
            let current_content = fs::read_to_string(&rc_file).unwrap_or_default();

            if current_content.contains(&comp_file.to_string_lossy().to_string())
                || current_content.contains("cpkb completions zsh")
            {
                println!("Completions are already sourced in {}", rc_file.display());
            } else {
                use std::io::Write;
                let mut rc = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&rc_file)?;
                writeln!(rc, "\n# CPKB shell completions\n{}", source_line)?;
                println!("Successfully added completions to {}", rc_file.display());
                println!("Run this command or restart your shell to enable:");
                println!("  source {}", rc_file.display());
            }
        }
        "bash" => {
            let rc_file = home.join(".bashrc");
            let source_line = format!("source \"{}\"", comp_file.display());
            let current_content = fs::read_to_string(&rc_file).unwrap_or_default();

            if current_content.contains(&comp_file.to_string_lossy().to_string())
                || current_content.contains("cpkb completions bash")
            {
                println!("Completions are already sourced in {}", rc_file.display());
            } else {
                use std::io::Write;
                let mut rc = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&rc_file)?;
                writeln!(rc, "\n# CPKB shell completions\n{}", source_line)?;
                println!("Successfully added completions to {}", rc_file.display());
                println!("Run this command or restart your shell to enable:");
                println!("  source {}", rc_file.display());
            }
        }
        "fish" => {
            let fish_comp_dir = home.join(".config").join("fish").join("completions");
            fs::create_dir_all(&fish_comp_dir)?;
            let fish_target = fish_comp_dir.join("cpkb.fish");
            fs::copy(&comp_file, &fish_target)?;
            println!("Installed fish completions directly to {}", fish_target.display());
            println!("Completions are active immediately in new fish shells.");
        }
        _ => {}
    }

    Ok(())
}
