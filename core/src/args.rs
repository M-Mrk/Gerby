use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "gerby-cli")]
#[command(about = "CLI for exporting PDFs of KiCad projects from a git repo URL!", long_about = None)]
pub struct Args {
    /// REQUIRED: URL to the git repo to export from NECESSARY
    pub repo_url: String,

    // REQUIRED: output path to put exported pdfs
    pub output: PathBuf,

    /// Flag whether schematics should be exported
    #[arg(short, long, default_value_t = false)]
    pub schematics: bool,

    /// Flag whether pcb should be exported
    #[arg(short, long, default_value_t = false)]
    pub pcb: bool,
}
