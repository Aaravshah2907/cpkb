use clap::Parser;
use cpkb::cli::{run_cli, Cli};
use cpkb::config::default_app_dir;
use cpkb::db::init_db;

fn main() {
    let cli = Cli::parse();
    let app_dir = default_app_dir();
    let mut conn = match init_db(&app_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error initializing database: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = run_cli(cli, &mut conn, &app_dir) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
