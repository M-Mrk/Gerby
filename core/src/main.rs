use clap::Parser;

use tracing::info;

mod logging;

#[derive(Parser, Debug)]
#[command(name = "gerby-cli")]
#[command(about = "CLI for exporting PDFs of KiCad projects from a git repo URL!", long_about = None)]
struct Args {
    /// REQUIRED: URL to the git repo to export from NECESSARY
    #[arg(short = 'u', long, required = true)]
    repo_url: String,

    /// Flag whether schematics should be exported
    #[arg(short, long, default_value_t = false)]
    schematics: bool,

    /// Flag whether pcb should be exported
    #[arg(short, long, default_value_t = false)]
    pcb: bool,
}

fn main() {
    logging::init();
    let args = Args::parse();

    info!("{:?}", args);
}
