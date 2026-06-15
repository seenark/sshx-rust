use crate::model::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SSHCommand {
    pub host: String,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub identity_file: Option<PathBuf>,
    pub local_forwards: Vec<LocalForward>,
    pub background: bool,
    pub extra_args: Vec<String>,
    pub password: Option<String>,
}

impl SSHCommand {
    pub fn from_host(host: &SSHHost) -> Self {
        let mut extra_args = Vec::new();

        if let Some(ref shc) = host.strict_host_checking {
            let val = match shc {
                StrictHostChecking::Yes => "yes",
                StrictHostChecking::No => "no",
                StrictHostChecking::Ask => "ask",
                StrictHostChecking::AcceptNew => "accept-new",
            };
            extra_args.push(format!("-o StrictHostKeyChecking={}", val));
        }
        if let Some(ref ukhf) = host.user_known_hosts_file {
            extra_args.push(format!("-o UserKnownHostsFile={}", ukhf));
        }
        for (k, v) in &host.extra_options {
            extra_args.push(format!("-o {}={}", k, v));
        }

        Self {
            host: host.hostname.clone(),
            port: host.port,
            user: host.user.clone(),
            identity_file: host.identity_file.clone(),
            local_forwards: host.local_forwards.clone(),
            background: host.sshx.background,
            extra_args,
            password: host.sshx.password.clone(),
        }
    }

    pub fn build_parts(&self) -> Vec<String> {
        let mut parts = match &self.password {
            Some(password) => vec![
                "sshpass".to_string(),
                "-p".to_string(),
                password.clone(),
                "ssh".to_string(),
            ],
            None => vec!["ssh".to_string()],
        };

        if let Some(port) = self.port {
            parts.push("-p".to_string());
            parts.push(port.to_string());
        }

        if let Some(ref identity_file) = self.identity_file {
            parts.push("-i".to_string());
            parts.push(identity_file.to_string_lossy().to_string());
        }

        for forward in &self.local_forwards {
            parts.push("-L".to_string());
            parts.push(format!(
                "{}:{}:{}",
                forward.local_port, forward.remote_host, forward.remote_port
            ));
        }

        if self.background {
            parts.push("-f".to_string());
            parts.push("-N".to_string());
        }

        parts.extend(self.extra_args.clone());

        let user_host = match &self.user {
            Some(user) => format!("{}@{}", user, self.host),
            None => self.host.clone(),
        };
        parts.push(user_host);

        parts
    }

    pub fn build(&self) -> String {
        shell_words::join(self.build_parts())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_basic_ssh_command() {
        let host = SSHHost {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            port: Some(22),
            user: Some("alice".to_string()),
            identity_file: None,
            local_forwards: Vec::new(),
            strict_host_checking: None,
            user_known_hosts_file: None,
            extra_options: Vec::new(),
            sshx: SSHXAnnotations::default(),
            source: SourceLocation::default(),
        };

        let cmd = SSHCommand::from_host(&host);
        assert_eq!(cmd.host, "example.com");
        assert_eq!(cmd.port, Some(22));
        assert_eq!(cmd.user, Some("alice".to_string()));
        assert!(cmd.identity_file.is_none());
        assert!(cmd.local_forwards.is_empty());
        assert!(!cmd.background);
        assert!(cmd.password.is_none());
    }

    #[test]
    fn test_command_with_local_forwards() {
        let host = SSHHost {
            name: "forwarded".to_string(),
            hostname: "example.com".to_string(),
            port: None,
            user: None,
            identity_file: None,
            local_forwards: vec![
                LocalForward {
                    local_port: 8080,
                    remote_host: "localhost".to_string(),
                    remote_port: 3000,
                },
                LocalForward {
                    local_port: 9090,
                    remote_host: "remote.db".to_string(),
                    remote_port: 5432,
                },
            ],
            strict_host_checking: None,
            user_known_hosts_file: None,
            extra_options: Vec::new(),
            sshx: SSHXAnnotations::default(),
            source: SourceLocation::default(),
        };

        let cmd = SSHCommand::from_host(&host);
        let parts = cmd.build_parts();

        assert_eq!(parts[0], "ssh");
        assert!(parts.contains(&"-L".to_string()));
        assert!(parts.contains(&"8080:localhost:3000".to_string()));
        assert!(parts.contains(&"9090:remote.db:5432".to_string()));
        assert!(parts.contains(&"example.com".to_string()));
    }

    #[test]
    fn test_command_with_password_uses_sshpass() {
        let host = SSHHost {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            port: None,
            user: None,
            identity_file: None,
            local_forwards: Vec::new(),
            strict_host_checking: None,
            user_known_hosts_file: None,
            extra_options: Vec::new(),
            sshx: SSHXAnnotations {
                password: Some("secret123".to_string()),
                ..Default::default()
            },
            source: SourceLocation::default(),
        };

        let cmd = SSHCommand::from_host(&host);
        let parts = cmd.build_parts();

        assert_eq!(parts[0..4], ["sshpass", "-p", "secret123", "ssh"]);
        assert!(parts.contains(&"example.com".to_string()));
    }

    #[test]
    fn test_background_mode() {
        let host = SSHHost {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            port: None,
            user: None,
            identity_file: None,
            local_forwards: Vec::new(),
            strict_host_checking: None,
            user_known_hosts_file: None,
            extra_options: Vec::new(),
            sshx: SSHXAnnotations {
                background: true,
                ..Default::default()
            },
            source: SourceLocation::default(),
        };

        let cmd = SSHCommand::from_host(&host);
        let parts = cmd.build_parts();

        assert!(parts.contains(&"-f".to_string()));
        assert!(parts.contains(&"-N".to_string()));
        assert!(parts.contains(&"example.com".to_string()));
    }

    #[test]
    fn test_command_orders_ssh_port_and_user() {
        let host = SSHHost {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            port: Some(2222),
            user: Some("alice".to_string()),
            identity_file: None,
            local_forwards: Vec::new(),
            strict_host_checking: None,
            user_known_hosts_file: None,
            extra_options: Vec::new(),
            sshx: SSHXAnnotations::default(),
            source: SourceLocation::default(),
        };

        let cmd = SSHCommand::from_host(&host);

        assert_eq!(
            cmd.build_parts(),
            vec!["ssh", "-p", "2222", "alice@example.com"]
        );
    }

    #[test]
    fn test_build_returns_string() {
        let host = SSHHost {
            name: "test".to_string(),
            hostname: "example.com".to_string(),
            port: Some(2222),
            user: Some("bob".to_string()),
            identity_file: Some(PathBuf::from("/path/to/key")),
            local_forwards: vec![LocalForward {
                local_port: 8080,
                remote_host: "localhost".to_string(),
                remote_port: 3000,
            }],
            strict_host_checking: None,
            user_known_hosts_file: None,
            extra_options: Vec::new(),
            sshx: SSHXAnnotations::default(),
            source: SourceLocation::default(),
        };

        let cmd = SSHCommand::from_host(&host);
        let built = cmd.build();

        assert!(built.starts_with("ssh "));
        assert!(!built.contains("sshpass"));
        assert!(built.contains("-p"));
        assert!(built.contains("2222"));
        assert!(built.contains("-i"));
        assert!(built.contains("/path/to/key"));
        assert!(built.contains("-L"));
        assert!(built.contains("8080:localhost:3000"));
        assert!(built.contains("bob@example.com"));
    }
}
