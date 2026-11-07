use std::path::PathBuf;
use std::process::Command;
use std::{fs, path::Path};
use tracing::{error, info, warn};
use wait_timeout::ChildExt;

use crate::args::Args;

pub enum SkelletonError {
    Timeout,
    Other,
}

pub enum ExtractError {
    Other,
}

#[derive(PartialEq, Eq)]
pub enum RepoDownloadState {
    Empty,
    Created,
    SkelletonDownloaded,
}

pub struct RepoDownload {
    path: PathBuf,
    stage: RepoDownloadState,
}
impl RepoDownload {
    pub fn new(process_id: &str) -> Self {
        Self {
            path: std::env::temp_dir().join(format!("Gerby-core-{}", process_id,)),
            stage: RepoDownloadState::Empty,
        }
    }

    pub fn init(&mut self) -> Result<(), ()> {
        if let Err(err) = fs::create_dir_all(self.path.as_path()) {
            error!(
                "Failed to create temp dir for cloning repo, due to: {}",
                err
            );
            return Err(());
        }
        self.stage = RepoDownloadState::Created;
        Ok(())
    }
    pub fn delete(&self) -> Result<(), ()> {
        if self.stage == RepoDownloadState::Empty {
            return Ok(());
        }
        info!("Deleting downloaded repo path at {:?}", self.path);

        if let Err(err) = fs::remove_dir_all(self.path.as_path()) {
            error!("Failed to delete repo download, due to {}", err);
            return Err(());
        }
        Ok(())
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn set_stage(&mut self, stage: RepoDownloadState) {
        self.stage = stage;
    }
}
impl Drop for RepoDownload {
    fn drop(&mut self) {
        self.delete()
            .expect("Failed to delete repo when RepoDownload was dropped");
    }
}

/// Clone a skelleton of a git repo (file layout but no data). Returns RepoDownload with path to
/// clone in a temporary directory
pub fn clone_repo_skelleton(args: &Args, process_id: &str) -> Result<RepoDownload, SkelletonError> {
    let mut download = RepoDownload::new(process_id);
    download.init().map_err(|_| SkelletonError::Other)?;

    // clone repo .git data for newest version
    info!("Running git clone to clone repo skelleton");
    let mut git_clone = match Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--filter=blob:none",
            "--no-checkout",
            &args.repo_url,
            download.get_path().to_str().unwrap_or_default(),
        ])
        .env("GIT_TERMINAL_PROMPT", "0") // fail if user input needed
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            warn!("Failed to spawn git clone, due to {}", err);
            return Err(SkelletonError::Other);
        }
    };
    match git_clone.wait_timeout(std::time::Duration::from_secs(45)) {
        Ok(Some(output)) => {
            if !output.success() {
                warn!("Cloning repo skelleton failed, output: {}", output);
                return Err(SkelletonError::Other);
            }
        }
        Ok(None) => {
            warn!("Cloning repo skelleton timed out!");
            return Err(SkelletonError::Timeout);
        }
        Err(err) => {
            error!("Failed to wait for git clone, due to {}", err);
            return Err(SkelletonError::Other);
        }
    }

    // create skelleton using .git
    // Get all file names
    info!("Creating skelleton with filenames and directories");
    let output = match Command::new("git")
        .args(["ls-tree", "-r", "--name-only", "HEAD"])
        .current_dir(download.get_path().to_str().unwrap_or_default())
        .output()
    {
        Ok(out) => out,
        Err(err) => {
            error!("Failed to run git ls-tree, due to: {}", err);
            return Err(SkelletonError::Other);
        }
    };
    if !output.status.success() {
        error!("Git ls-tree failed: {:?}", output);
        return Err(SkelletonError::Other);
    }

    // Create files
    let files = String::from_utf8_lossy(&output.stdout);
    for file_path in files.lines() {
        let path = download.get_path().join(Path::new(file_path.trim()));

        // create parent dirs for file
        if let Some(parent) = path.parent()
            && !parent.exists()
            && let Err(err) = fs::create_dir_all(parent)
        {
            error!(
                "Failed to create parent dir for skelleton at '{:?}', due to  {}",
                parent, err
            );
            return Err(SkelletonError::Other);
        }

        // create file
        if let Err(err) = fs::File::create(&path) {
            error!(
                "Failed to create file for skelleton at '{:?}', due to  {}",
                path, err
            );
            return Err(SkelletonError::Other);
        }
    }

    download.set_stage(RepoDownloadState::SkelletonDownloaded);
    Ok(download)
}
