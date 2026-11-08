use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info, warn};
use wait_timeout::ChildExt;

use crate::args::Args;

pub enum SkelletonError {
    Timeout,
    Other,
}

pub enum ExtractError {
    NothingFound,
    Timeout,
    Other,
}

#[derive(PartialEq, Eq)]
pub enum RepoDownloadState {
    Empty,
    Created,
    FilesPopulated(Vec<PathBuf>),
    KiCadExtracted(PathBuf),
}

pub struct RepoDownload {
    path: PathBuf,
    pub stage: RepoDownloadState,
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
    let file_paths = String::from_utf8_lossy(&output.stdout);
    let files: Vec<PathBuf> = file_paths
        .lines()
        .map(|a| PathBuf::from(a.trim()))
        .collect();
    download.set_stage(RepoDownloadState::FilesPopulated(files));
    Ok(download)
}

/// Extracts
pub fn extract_kicad_files(mut download: RepoDownload) -> Result<RepoDownload, ExtractError> {
    info!("Extracting kicad files from repo");
    let files = match &download.stage {
        RepoDownloadState::FilesPopulated(f) => f.clone(),
        _ => {
            error!("RepoDownload was not in FilesPopulated stage");
            return Err(ExtractError::Other);
        }
    };

    // get dir to download
    let kicad_project_path = match files
        .iter()
        .find(|path| path.extension().is_some_and(|ex| ex == "kicad_pro"))
    {
        Some(path) => path,
        None => {
            warn!("Couldn't find any .kicad_pro files in {:?}", &files);
            return Err(ExtractError::NothingFound);
        }
    };
    let kicad_project_path_str = match kicad_project_path.to_str() {
        Some(s) => s,
        None => {
            error!("Failed to create a str from kicad project path");
            return Err(ExtractError::Other);
        }
    };
    let kicad_project_parent_path_str = match kicad_project_path.parent().unwrap().to_str() {
        Some(s) => s,
        None => {
            error!("Failed to create a str from kicad project path");
            return Err(ExtractError::Other);
        }
    };
    info!("Found .kicad_pro file under {}", kicad_project_path_str);

    // set checkout to only that folder
    info!("Running git sparse-checkout");
    let sparse_checkout_output = match Command::new("git")
        .args(["sparse-checkout", "set", kicad_project_parent_path_str])
        .current_dir(download.get_path())
        .output()
    {
        Ok(out) => out,
        Err(err) => {
            error!("Failed to run git sparse-checkout, due to {}", err);
            return Err(ExtractError::Other);
        }
    };
    if !sparse_checkout_output.status.success() {
        error!(
            "Failed to git sparse-checkout: {:?}",
            sparse_checkout_output
        );
        return Err(ExtractError::Other);
    }

    // checkout
    info!("Running git checkout");
    let mut checkout_child = match Command::new("git")
        .arg("checkout")
        .current_dir(download.get_path())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            error!("Failed to spawn git checkout, due to {}", err);
            return Err(ExtractError::Other);
        }
    };

    let timeout = std::time::Duration::from_mins(2);
    match checkout_child.wait_timeout(timeout) {
        Ok(Some(output)) => {
            if !output.success() {
                error!("Failed to git checkout: {:?}", output);
                return Err(ExtractError::Other);
            }
        }
        Ok(None) => {
            warn!("git checkout timed out!");
            return Err(ExtractError::Timeout);
        }
        Err(err) => {
            error!("Failed to wait for git checkout, due to: {}", err);
            return Err(ExtractError::Other);
        }
    }

    download.set_stage(RepoDownloadState::KiCadExtracted(
        kicad_project_path.clone(),
    ));
    Ok(download)
}
