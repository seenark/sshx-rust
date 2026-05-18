use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct SSHHost {
    pub name: String,
    pub hostname: String,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub identity_file: Option<PathBuf>,
    pub local_forwards: Vec<LocalForward>,
    pub strict_host_checking: Option<StrictHostChecking>,
    pub user_known_hosts_file: Option<String>,
    pub extra_options: Vec<(String, String)>,
    pub sshx: SSHXAnnotations,
    pub source: SourceLocation,
}

impl Default for SSHHost {
    fn default() -> Self {
        Self {
            name: String::new(),
            hostname: String::new(),
            port: None,
            user: None,
            identity_file: None,
            local_forwards: Vec::new(),
            strict_host_checking: None,
            user_known_hosts_file: None,
            extra_options: Vec::new(),
            sshx: SSHXAnnotations::default(),
            source: SourceLocation {
                file: PathBuf::new(),
                line_start: 0,
                line_end: 0,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalForward {
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StrictHostChecking {
    Yes,
    No,
    Ask,
    AcceptNew,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SSHXAnnotations {
    pub group: Option<String>,
    pub password: Option<String>,
    pub requires: Option<String>,
    pub alias: Option<String>,
    pub background: bool,
    pub description: Option<String>,
    pub after_connect: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct TunnelProcess {
    pub jump_host: String,
    pub pid: u32,
    pub local_ports: Vec<u16>,
    pub started_at: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TunnelStatus {
    NotRunning,
    Running(TunnelProcess),
    PortConflict(u16),
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_host_default() {
        let host = SSHHost::default();
        assert!(host.name.is_empty());
        assert!(host.hostname.is_empty());
        assert!(host.port.is_none());
        assert!(host.user.is_none());
        assert!(host.identity_file.is_none());
        assert!(host.local_forwards.is_empty());
        assert!(host.strict_host_checking.is_none());
        assert!(host.user_known_hosts_file.is_none());
        assert!(host.extra_options.is_empty());
        assert_eq!(host.sshx, SSHXAnnotations::default());
        assert!(host.source.file.as_os_str().is_empty());
        assert_eq!(host.source.line_start, 0);
        assert_eq!(host.source.line_end, 0);
    }

    #[test]
    fn test_local_forward() {
        let forward = LocalForward {
            local_port: 8080,
            remote_host: String::from("localhost"),
            remote_port: 3000,
        };
        assert_eq!(forward.local_port, 8080);
        assert_eq!(forward.remote_host, "localhost");
        assert_eq!(forward.remote_port, 3000);
    }

    #[test]
    fn test_annotations_default() {
        let annotations = SSHXAnnotations::default();
        assert!(annotations.group.is_none());
        assert!(annotations.password.is_none());
        assert!(annotations.requires.is_none());
        assert!(annotations.alias.is_none());
        assert!(!annotations.background);
        assert!(annotations.description.is_none());
        assert!(annotations.after_connect.is_none());
    }
}
