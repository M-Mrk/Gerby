use clap::Parser;
use tracing::info;

mod args;
mod git;
mod output;
use crate::{
    args::Args,
    git::{
        ExtractError, RepoDownloadState, SkelletonError, check_repo_exists, check_valid_uri,
        clone_repo_skelleton, extract_kicad_files,
    },
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

    let extracted_download = match extract_kicad_files(skelleton_download) {
        Ok(download) => {
            let path = if let RepoDownloadState::KiCadExtracted(p) = &download.stage {
                p
            } else {
                panic!()
            };
            output_update("kicad_found", vec![("path", &(format!("{:?}", path)))]);
            download
        }
        Err(ExtractError::NothingFound) => {
            output_error("kicad_not_found", vec![]);
        }
        Err(ExtractError::Timeout) => {
            output_error("kicad_timeout", vec![]);
        }
        Err(ExtractError::Other) => {
            output_error("kicad_fail", vec![]);
        }
    };

    info!("files at {:?}", extracted_download.get_path());

    loop {
        std::thread::sleep(std::time::Duration::from_hours(1));
    }
}
