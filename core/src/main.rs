use std::path::PathBuf;

use clap::Parser;

use tracing::info;

use crate::git::check_valid_uri;

mod git;
mod logging;
mod output;

#[derive(Parser, Debug)]
#[command(name = "gerby-cli")]
#[command(about = "CLI for exporting PDFs of KiCad projects from a git repo URL!", long_about = None)]
struct Args {
    /// REQUIRED: URL to the git repo to export from NECESSARY
    repo_url: String,

    // REQUIRED: output path to put exported pdfs
    output: PathBuf,

    /// Flag whether schematics should be exported
    #[arg(short, long, default_value_t = false)]
    schematics: bool,

    /// Flag whether pcb should be exported
    #[arg(short, long, default_value_t = false)]
    pcb: bool,
}

fn main() {
    logging::init();
    let mut args = Args::parse();
    info!("Running with {:?}", args);

    args.repo_url = check_valid_uri(&args.repo_url).unwrap_or_else(|_| std::process::exit(1));
}
