use clap::Parser;
use tracing::info;

mod args;
mod git;
mod output;
use crate::{
    args::Args,
    git::{check_repo_exists, check_valid_uri},
    output::{output_error, output_update},
};

mod logging;

fn main() {
    logging::init();
    let mut args = Args::parse();
    info!("Running with {:?}", args);

    args.repo_url = check_valid_uri(&args.repo_url).unwrap_or_else(|uri_err| {
        output_error("uri_invalid", vec![("uri", &uri_err)]);
        std::process::exit(1)
    });

    if check_repo_exists(&args.repo_url).is_err() {
        output_error("repo_not_found", vec![("uri", &args.repo_url)]);
    } else {
        output_update("repo_found", vec![]);
    }
}
