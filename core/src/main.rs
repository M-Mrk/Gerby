use clap::Parser;
use tracing::info;

mod args;
mod git;
mod output;
use crate::{
    args::Args,
    git::{SkelletonError, check_repo_exists, check_valid_uri, clone_repo_skelleton},
    output::{output_error, output_update},
};

mod logging;

fn main() {
    let process_id = uuid::Uuid::new_v4().to_string();
    logging::init(&process_id);
    let mut args = Args::parse();
    info!("Running with {:?} and id {}", args, process_id);
    output_update("process_id", vec![("id", &process_id)]);

    args.repo_url = check_valid_uri(&args.repo_url).unwrap_or_else(|uri_err| {
        output_error("uri_invalid", vec![("uri", &uri_err)]);
    });

    if check_repo_exists(&args.repo_url).is_err() {
        output_error("repo_not_found", vec![("uri", &args.repo_url)]);
    } else {
        output_update("repo_found", vec![]);
    }

    let skelleton_download = match clone_repo_skelleton(&args, &process_id) {
        Ok(download) => download,
        Err(SkelletonError::Timeout) => output_error("skelleton_timeout", vec![]),
        Err(SkelletonError::Other) => output_error("skelleton_fail", vec![]),
    };

    info!("{:?}", skelleton_download.get_files());
}
