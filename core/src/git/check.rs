use regex::Regex;
use std::process::Command;
use std::str::FromStr;
use tracing::{error, info, warn};

/// Checks whether URI contains invalid characters or is an IP address and prefixes https if needed
pub fn check_valid_uri(uri: &str) -> Result<String, String> {
    let mut output = String::from(uri);

    // Make sure protocol is prefixed
    if !output.starts_with("https://") {
        output.insert_str(0, "https://");
        info!("Added https protocol to git repo uri: {}", output);
    }

    // Make sure uri doesn't contain invalid characters
    let uri_without_protocol = output.clone().split_off(8);
    let uri_allowed_ex = Regex::new(r"^[a-zA-Z0-9./-]+$").unwrap();
    if !uri_allowed_ex.is_match(&uri_without_protocol) {
        warn!(
            "Git repo uri path contains invalid characters!: {}",
            uri_without_protocol
        );
        return Err(output);
    }

    let host = uri_without_protocol
        .split("/")
        .next()
        .unwrap_or(&uri_without_protocol);
    if std::net::IpAddr::from_str(host).is_ok() {
        warn!("Host is an IP address!: {}", host);
        return Err(uri.to_string());
    }

    Ok(output)
}

/// Checks whether repo exists and can be accessed using git ls-remote
pub fn check_repo_exists(uri: &str) -> Result<(), ()> {
    let output = match Command::new("git")
        .arg("ls-remote")
        .arg("--")
        .arg(uri)
        .env("GIT_TERMINAL_PROMPT", "0") // fail if user input needed
        .output()
    {
        Ok(out) => out,
        Err(err) => {
            error!("{:?}", err);
            return Err(());
        }
    };

    if !output.status.success() {
        warn!("git ls-remote failed and returned: {:?}", output);
        return Err(());
    }

    info!("git ls-remote succeeded and returned: {:?}", output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_check_real_repo() {
        assert!(check_valid_uri("github.com/M-Mrk/gerby").is_ok());
        assert!(check_valid_uri("gitlab.com/gitlab-org/gitlab").is_ok());
        assert!(check_valid_uri("codeberg.org/forgejo/forgejo").is_ok());
    }

    #[test]
    fn test_uri_check_real_repo_w_https() {
        assert!(check_valid_uri("https://github.com/M-Mrk/gerby").is_ok());
    }

    #[test]
    fn test_uri_check_malicous() {
        assert!(check_valid_uri("github.com/john/project && echo 'evil'").is_err());
        assert!(check_valid_uri("file://evil-file").is_err());
        assert!(check_valid_uri("192.168.2.1").is_err());
        assert!(check_valid_uri("[::1]").is_err());
    }

    #[test]
    fn test_uri_check_empty() {
        assert!(check_valid_uri("").is_err());
    }

    #[test]
    fn test_existing_repo() {
        assert!(check_repo_exists("https://github.com/M-Mrk/gerby").is_ok())
    }

    #[test]
    fn test_non_existing_repo() {
        assert!(check_repo_exists("https://github.com/M-Mrk/evil-test-repo").is_err())
    }
}
