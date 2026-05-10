use std::path::PathBuf;

use console::style;

#[derive(Debug, thiserror::Error)]
pub enum SshxError {
    #[error("SSH config file not found")]
    ConfigFileNotFound { path: PathBuf },

    #[error("SSH config file unreadable")]
    ConfigFileUnreadable { path: PathBuf, reason: String },

    #[error("Annotation parse error")]
    AnnotationParseError { file: PathBuf, line: usize, raw: String },

    #[error("Unknown annotation key")]
    UnknownAnnotationKey { file: PathBuf, line: usize, key: String },

    #[error("Duplicate alias")]
    DuplicateAlias { alias: String, host1: String, host2: String },

    #[error("Host not found")]
    HostNotFound { input: String },

    #[error("Alias ambiguous")]
    AliasAmbiguous { input: String, matches: Vec<String> },

    #[error("Required host not found")]
    RequiresHostNotFound { host: String, requires: String },

    #[error("Circular requires dependency")]
    CircularRequires { host: String },

    #[error("Tunnel spawn failed")]
    TunnelSpawnFailed { jump_host: String, reason: String },

    #[error("Tunnel port busy")]
    TunnelPortBusy { port: u16 },

    #[error("Tunnel timeout")]
    TunnelTimeout { jump_host: String, timeout_s: u64 },

    #[error("Tunnel died early")]
    TunnelDiedEarly { jump_host: String, exit_code: Option<i32> },

    #[error("sshpass not found in PATH")]
    SshpassNotFound,

    #[error("ssh not found in PATH")]
    SshNotFound,

    #[error("SSH command failed")]
    SshCommandFailed { exit_code: Option<i32> },

    #[error("Clipboard unavailable")]
    ClipboardUnavailable { reason: String },

    #[error("Config write failed")]
    ConfigWriteFailed { path: PathBuf, reason: String },

    #[error("Host already exists")]
    HostAlreadyExists { name: String },

    #[error("Invalid host name")]
    InvalidHostName { name: String },

    #[error("Invalid port")]
    InvalidPort { input: String },

    #[error("SSHX config parse failed")]
    SshxConfigParseFailed { path: PathBuf, reason: String },

    #[error("SSHX config write failed")]
    SshxConfigWriteFailed { path: PathBuf, reason: String },
}

impl SshxError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ConfigFileNotFound { .. } => "E001",
            Self::ConfigFileUnreadable { .. } => "E002",
            Self::AnnotationParseError { .. } => "E003",
            Self::UnknownAnnotationKey { .. } => "E004",
            Self::DuplicateAlias { .. } => "E005",
            Self::HostNotFound { .. } => "E010",
            Self::AliasAmbiguous { .. } => "E011",
            Self::RequiresHostNotFound { .. } => "E012",
            Self::CircularRequires { .. } => "E013",
            Self::TunnelSpawnFailed { .. } => "E020",
            Self::TunnelPortBusy { .. } => "E021",
            Self::TunnelTimeout { .. } => "E022",
            Self::TunnelDiedEarly { .. } => "E023",
            Self::SshpassNotFound { .. } => "E030",
            Self::SshNotFound { .. } => "E031",
            Self::SshCommandFailed { .. } => "E032",
            Self::ClipboardUnavailable { .. } => "E040",
            Self::ConfigWriteFailed { .. } => "E050",
            Self::HostAlreadyExists { .. } => "E051",
            Self::InvalidHostName { .. } => "E052",
            Self::InvalidPort { .. } => "E053",
            Self::SshxConfigParseFailed { .. } => "E060",
            Self::SshxConfigWriteFailed { .. } => "E061",
        }
    }

    pub fn title(&self) -> String {
        match self {
            Self::ConfigFileNotFound { path } => format!("SSH config file not found: {}", path.display()),
            Self::ConfigFileUnreadable { path, reason } => format!("SSH config file unreadable: {} — {}", path.display(), reason),
            Self::AnnotationParseError { file, line, raw } => format!("Annotation parse error in {}:{}: `{}`", file.display(), line, raw),
            Self::UnknownAnnotationKey { file, line, key } => format!("Unknown annotation key `{}` in {}:{}", key, file.display(), line),
            Self::DuplicateAlias { alias, host1, host2 } => format!("Duplicate alias `{alias}` used by both {host1} and {host2}"),
            Self::HostNotFound { input } => format!("Host not found: {input}"),
            Self::AliasAmbiguous { input, matches } => format!("Alias ambiguous: `{input}` matches [{}]", matches.join(", ")),
            Self::RequiresHostNotFound { host, requires } => format!("Required host not found: `{requires}` required by {host}"),
            Self::CircularRequires { host } => format!("Circular requires dependency: {host}"),
            Self::TunnelSpawnFailed { jump_host, reason } => format!("Tunnel spawn failed: {jump_host} — {reason}"),
            Self::TunnelPortBusy { port } => format!("Tunnel port {port} is already in use"),
            Self::TunnelTimeout { jump_host, timeout_s } => format!("Tunnel timeout — {jump_host} did not become ready in {timeout_s}s"),
            Self::TunnelDiedEarly { jump_host, exit_code } => format!("Tunnel died early: {jump_host} (exit code: {:?})", exit_code),
            Self::SshpassNotFound => "sshpass not found in PATH".to_string(),
            Self::SshNotFound => "ssh not found in PATH".to_string(),
            Self::SshCommandFailed { exit_code } => format!("SSH command failed (exit code: {:?})", exit_code),
            Self::ClipboardUnavailable { reason } => format!("Clipboard unavailable: {reason}"),
            Self::ConfigWriteFailed { path, reason } => format!("Config write failed: {} — {}", path.display(), reason),
            Self::HostAlreadyExists { name } => format!("Host already exists: {name}"),
            Self::InvalidHostName { name } => format!("Invalid host name: {name}"),
            Self::InvalidPort { input } => format!("Invalid port: {input}"),
            Self::SshxConfigParseFailed { path, reason } => format!("SSHX config parse failed: {} — {}", path.display(), reason),
            Self::SshxConfigWriteFailed { path, reason } => format!("SSHX config write failed: {} — {}", path.display(), reason),
        }
    }

    fn context_label(&self) -> Option<&'static str> {
        match self {
            Self::ConfigFileNotFound { .. } => Some("Path"),
            Self::ConfigFileUnreadable { .. } => Some("Path"),
            Self::AnnotationParseError { .. } => Some("File"),
            Self::UnknownAnnotationKey { .. } => Some("File"),
            Self::DuplicateAlias { .. } => Some("Alias"),
            Self::HostNotFound { .. } => Some("Input"),
            Self::AliasAmbiguous { .. } => Some("Input"),
            Self::RequiresHostNotFound { .. } => Some("Host"),
            Self::CircularRequires { .. } => Some("Host"),
            Self::TunnelSpawnFailed { .. } => Some("Host"),
            Self::TunnelPortBusy { .. } => Some("Port"),
            Self::TunnelTimeout { .. } => Some("Host"),
            Self::TunnelDiedEarly { .. } => Some("Host"),
            Self::SshpassNotFound => None,
            Self::SshNotFound => None,
            Self::SshCommandFailed { .. } => Some("Exit code"),
            Self::ClipboardUnavailable { .. } => Some("Reason"),
            Self::ConfigWriteFailed { .. } => Some("Path"),
            Self::HostAlreadyExists { .. } => Some("Name"),
            Self::InvalidHostName { .. } => Some("Name"),
            Self::InvalidPort { .. } => Some("Input"),
            Self::SshxConfigParseFailed { .. } => Some("Path"),
            Self::SshxConfigWriteFailed { .. } => Some("Path"),
        }
    }

    fn context_value(&self) -> Option<String> {
        match self {
            Self::ConfigFileNotFound { path } => Some(path.display().to_string()),
            Self::ConfigFileUnreadable { path, .. } => Some(path.display().to_string()),
            Self::AnnotationParseError { file, line, .. } => Some(format!("{}:{}", file.display(), line)),
            Self::UnknownAnnotationKey { file, line, .. } => Some(format!("{}:{}", file.display(), line)),
            Self::DuplicateAlias { alias, .. } => Some(alias.clone()),
            Self::HostNotFound { input } => Some(input.clone()),
            Self::AliasAmbiguous { input, matches } => Some(format!("{} (matches: {})", input, matches.join(", "))),
            Self::RequiresHostNotFound { host, .. } => Some(host.clone()),
            Self::CircularRequires { host } => Some(host.clone()),
            Self::TunnelSpawnFailed { jump_host, .. } => Some(jump_host.clone()),
            Self::TunnelPortBusy { port } => Some(port.to_string()),
            Self::TunnelTimeout { jump_host, .. } => Some(jump_host.clone()),
            Self::TunnelDiedEarly { jump_host, .. } => Some(jump_host.clone()),
            Self::SshpassNotFound => None,
            Self::SshNotFound => None,
            Self::SshCommandFailed { exit_code } => Some(format!("{:?}", exit_code)),
            Self::ClipboardUnavailable { reason } => Some(reason.clone()),
            Self::ConfigWriteFailed { path, .. } => Some(path.display().to_string()),
            Self::HostAlreadyExists { name } => Some(name.clone()),
            Self::InvalidHostName { name } => Some(name.clone()),
            Self::InvalidPort { input } => Some(input.clone()),
            Self::SshxConfigParseFailed { path, .. } => Some(path.display().to_string()),
            Self::SshxConfigWriteFailed { path, .. } => Some(path.display().to_string()),
        }
    }

    pub fn hint(&self) -> &'static str {
        match self {
            Self::ConfigFileNotFound { .. } => "Create the SSH config file or use --config PATH to specify a custom location",
            Self::ConfigFileUnreadable { .. } => "Check file permissions and ensure the file exists",
            Self::AnnotationParseError { .. } => "Ensure annotation is in format: ## sshx: key = value",
            Self::UnknownAnnotationKey { .. } => "Check for typos. Valid keys: group, password, requires, alias, background, description, after_connect",
            Self::DuplicateAlias { .. } => "Remove one of the duplicate aliases from your SSH config",
            Self::HostNotFound { .. } => "Check the host name or alias exists in your SSH config",
            Self::AliasAmbiguous { .. } => "Use a more specific host name or alias",
            Self::RequiresHostNotFound { .. } => "Add the required jump host to your SSH config first",
            Self::CircularRequires { .. } => "Remove the circular requires dependency in your SSH config",
            Self::TunnelSpawnFailed { .. } => "Check that the jump host is reachable and SSH keys are configured",
            Self::TunnelPortBusy { .. } => "Choose a different local port or stop the process using the current port",
            Self::TunnelTimeout { .. } => "Check that the host is reachable and the tunnel port is not blocked",
            Self::TunnelDiedEarly { .. } => "Check SSH connection to the jump host and verify credentials",
            Self::SshpassNotFound { .. } => "Install sshpass: brew install hudochenkov/sshpass/sshpass or apt install sshpass",
            Self::SshNotFound { .. } => "Install OpenSSH client",
            Self::SshCommandFailed { .. } => "Verify SSH connection and credentials",
            Self::ClipboardUnavailable { .. } => "Use --no-clipboard flag to print the command instead",
            Self::ConfigWriteFailed { .. } => "Check file permissions and disk space",
            Self::HostAlreadyExists { .. } => "Choose a different host name or edit the existing host",
            Self::InvalidHostName { .. } => "Host names must not contain spaces or special characters",
            Self::InvalidPort { .. } => "Port must be a number between 1 and 65535",
            Self::SshxConfigParseFailed { .. } => "Fix the TOML syntax in your SSHX config file",
            Self::SshxConfigWriteFailed { .. } => "Check file permissions and disk space",
        }
    }

    pub fn display_full(&self) -> String {
        let code = self.code();
        let title = self.title();
        let hint = self.hint();
        let context_label = self.context_label();
        let context_value = self.context_value();

        match (context_label, context_value) {
            (Some(label), Some(value)) => format!(
                "{} [{}] {}\n  {}:    {}\n  Hint:    {}",
                style("✗").red(),
                style(code).bold(),
                title,
                label,
                value,
                hint
            ),
            (None, None) => format!(
                "{} [{}] {}\n  Hint:    {}",
                style("✗").red(),
                style(code).bold(),
                title,
                hint
            ),
            _ => unreachable!("exhaustive error handling"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_variants_have_codes() {
        let errors = vec![
            SshxError::ConfigFileNotFound { path: PathBuf::from("/path") },
            SshxError::ConfigFileUnreadable { path: PathBuf::from("/path"), reason: "test".to_string() },
            SshxError::AnnotationParseError { file: PathBuf::from("/path"), line: 1, raw: "test".to_string() },
            SshxError::UnknownAnnotationKey { file: PathBuf::from("/path"), line: 1, key: "test".to_string() },
            SshxError::DuplicateAlias { alias: "test".to_string(), host1: "h1".to_string(), host2: "h2".to_string() },
            SshxError::HostNotFound { input: "test".to_string() },
            SshxError::AliasAmbiguous { input: "test".to_string(), matches: vec!["a".to_string(), "b".to_string()] },
            SshxError::RequiresHostNotFound { host: "test".to_string(), requires: "req".to_string() },
            SshxError::CircularRequires { host: "test".to_string() },
            SshxError::TunnelSpawnFailed { jump_host: "test".to_string(), reason: "test".to_string() },
            SshxError::TunnelPortBusy { port: 22 },
            SshxError::TunnelTimeout { jump_host: "test".to_string(), timeout_s: 10 },
            SshxError::TunnelDiedEarly { jump_host: "test".to_string(), exit_code: Some(1) },
            SshxError::SshpassNotFound,
            SshxError::SshNotFound,
            SshxError::SshCommandFailed { exit_code: Some(1) },
            SshxError::ClipboardUnavailable { reason: "test".to_string() },
            SshxError::ConfigWriteFailed { path: PathBuf::from("/path"), reason: "test".to_string() },
            SshxError::HostAlreadyExists { name: "test".to_string() },
            SshxError::InvalidHostName { name: "test".to_string() },
            SshxError::InvalidPort { input: "test".to_string() },
            SshxError::SshxConfigParseFailed { path: PathBuf::from("/path"), reason: "test".to_string() },
            SshxError::SshxConfigWriteFailed { path: PathBuf::from("/path"), reason: "test".to_string() },
        ];

        for error in errors {
            let code = error.code();
            assert!(!code.is_empty(), "Error {:?} has empty code", error);
        }
    }

    #[test]
    fn test_display_full_format() {
        let error = SshxError::TunnelTimeout {
            jump_host: "test-host".to_string(),
            timeout_s: 10,
        };

        let display = error.display_full();
        assert!(display.starts_with("✗"), "Should start with ✗");
        assert!(display.contains("[E022]"), "Should contain code E022");
        assert!(display.contains("Tunnel timeout"), "Should contain title");
        assert!(display.contains("Hint:"), "Should contain hint");
    }

    #[test]
    fn test_all_codes_unique() {
        let errors = vec![
            SshxError::ConfigFileNotFound { path: PathBuf::from("/path") },
            SshxError::ConfigFileUnreadable { path: PathBuf::from("/path"), reason: "test".to_string() },
            SshxError::AnnotationParseError { file: PathBuf::from("/path"), line: 1, raw: "test".to_string() },
            SshxError::UnknownAnnotationKey { file: PathBuf::from("/path"), line: 1, key: "test".to_string() },
            SshxError::DuplicateAlias { alias: "test".to_string(), host1: "h1".to_string(), host2: "h2".to_string() },
            SshxError::HostNotFound { input: "test".to_string() },
            SshxError::AliasAmbiguous { input: "test".to_string(), matches: vec!["a".to_string(), "b".to_string()] },
            SshxError::RequiresHostNotFound { host: "test".to_string(), requires: "req".to_string() },
            SshxError::CircularRequires { host: "test".to_string() },
            SshxError::TunnelSpawnFailed { jump_host: "test".to_string(), reason: "test".to_string() },
            SshxError::TunnelPortBusy { port: 22 },
            SshxError::TunnelTimeout { jump_host: "test".to_string(), timeout_s: 10 },
            SshxError::TunnelDiedEarly { jump_host: "test".to_string(), exit_code: Some(1) },
            SshxError::SshpassNotFound,
            SshxError::SshNotFound,
            SshxError::SshCommandFailed { exit_code: Some(1) },
            SshxError::ClipboardUnavailable { reason: "test".to_string() },
            SshxError::ConfigWriteFailed { path: PathBuf::from("/path"), reason: "test".to_string() },
            SshxError::HostAlreadyExists { name: "test".to_string() },
            SshxError::InvalidHostName { name: "test".to_string() },
            SshxError::InvalidPort { input: "test".to_string() },
            SshxError::SshxConfigParseFailed { path: PathBuf::from("/path"), reason: "test".to_string() },
            SshxError::SshxConfigWriteFailed { path: PathBuf::from("/path"), reason: "test".to_string() },
        ];

        let codes: Vec<_> = errors.iter().map(|e| e.code()).collect();
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(codes.len(), unique_codes.len(), "All error codes should be unique");
    }
}
