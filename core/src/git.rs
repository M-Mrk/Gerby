use std::str::FromStr;

use anyhow::{Result, bail};
use regex::Regex;
use tracing::{info, warn};

use crate::output::output_error;

pub fn check_valid_uri(uri: &str) -> Result<String> {
    let mut output = String::from(uri);

    // Make sure protocol is prefixed
    if !output.starts_with("https://") {
        output.insert_str(0, "https://");
        info!("Added https protocol to git repo uri: {}", output);
    }

    // Make sure uri doesn't contain invalid characters
    let uri_without_protocol = output.split_off(8);
    let uri_allowed_ex = Regex::new(r"^[a-zA-Z0-9./-]+$").unwrap();
    if !uri_allowed_ex.is_match(&uri_without_protocol) {
        let error = format!(
            "Git repo uri path contains invalid characters!: {}",
            uri_without_protocol
        );
        warn!(error);

        output_error("uri_forbidden", vec![("uri", &uri_without_protocol)]);

        bail!(error);
    }

    let host = uri_without_protocol
        .split("/")
        .next()
        .unwrap_or(&uri_without_protocol);
    if std::net::IpAddr::from_str(host).is_ok() {
        let error = format!("Host is an IP address!: {}", host);
        warn!(error);
        output_error("uri_forbidden", vec![("uri", &uri_without_protocol)]);
        bail!(error)
    }

    Ok(output)
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
}
