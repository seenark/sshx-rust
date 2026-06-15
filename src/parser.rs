use std::fs;
use std::path::{Path, PathBuf};

use crate::error::SshxError;
use crate::model::*;

#[allow(dead_code)]
pub fn parse_file(path: &Path) -> Result<Vec<SSHHost>, SshxError> {
    let content = fs::read_to_string(path).map_err(|e| SshxError::ConfigFileUnreadable {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    parse_content(&content, path)
}

pub fn parse_content(content: &str, file_path: &Path) -> Result<Vec<SSHHost>, SshxError> {
    let mut hosts = Vec::new();
    let mut current_host: Option<SSHHost> = None;

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') && !trimmed.starts_with("## sshx:") {
            continue;
        }

        if trimmed.starts_with("## sshx:") {
            if let Some(ref mut host) = current_host {
                parse_annotation(host, trimmed, file_path, line_idx + 1)?;
            }
            continue;
        }

        if trimmed.starts_with("Host ") {
            if let Some(mut host) = current_host.take() {
                host.source.line_end = line_idx;
                hosts.push(host);
            }

            if let Some(stripped) = trimmed.strip_prefix("Host ") {
                let name = stripped.trim().to_string();
                let line_start = line_idx + 1;
                current_host = Some(SSHHost {
                    name: name.clone(),
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
                        file: file_path.to_path_buf(),
                        line_start,
                        line_end: 0,
                    },
                });
            }
            continue;
        }

        if let Some(ref mut host) = current_host {
            parse_option(host, trimmed);
        }
    }

    if let Some(mut host) = current_host {
        host.source.line_end = content.lines().count();
        hosts.push(host);
    }

    Ok(hosts)
}

pub fn parse_with_includes(path: &Path) -> Result<(Vec<SSHHost>, Vec<PathBuf>), SshxError> {
    let mut all_hosts = Vec::new();
    let mut all_source_files = Vec::new();
    let mut visited = std::collections::HashSet::new();
    parse_recursive(path, &mut all_hosts, &mut all_source_files, &mut visited)?;
    Ok((all_hosts, all_source_files))
}

fn resolve_include_pattern(pattern: &str, ssh_dir: &Path, home_dir: Option<&Path>) -> String {
    if pattern == "~" {
        return home_dir
            .map(|home| home.to_string_lossy().into_owned())
            .unwrap_or_else(|| pattern.to_string());
    }

    if let Some(stripped) = pattern.strip_prefix("~/") {
        return home_dir
            .map(|home| home.join(stripped).to_string_lossy().into_owned())
            .unwrap_or_else(|| pattern.to_string());
    }

    if Path::new(pattern).is_absolute() {
        return pattern.to_string();
    }

    ssh_dir.join(pattern).to_string_lossy().into_owned()
}

fn parse_recursive(
    path: &Path,
    hosts: &mut Vec<SSHHost>,
    source_files: &mut Vec<PathBuf>,
    visited: &mut std::collections::HashSet<PathBuf>,
) -> Result<(), SshxError> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical.clone()) {
        return Ok(());
    }

    if !path.exists() {
        return Err(SshxError::ConfigFileNotFound {
            path: path.to_path_buf(),
        });
    }

    let content = std::fs::read_to_string(path).map_err(|e| SshxError::ConfigFileUnreadable {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;

    source_files.push(path.to_path_buf());

    let ssh_dir = path.parent().unwrap_or(Path::new("."));
    let home_dir = dirs::home_dir();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(patterns) = trimmed.strip_prefix("Include ") {
            for pattern in patterns.split_whitespace() {
                let full_pattern = resolve_include_pattern(pattern, ssh_dir, home_dir.as_deref());
                if let Ok(entries) = glob::glob(&full_pattern) {
                    for entry in entries.flatten() {
                        if entry.is_file() {
                            parse_recursive(&entry, hosts, source_files, visited)?;
                        }
                    }
                }
            }
            // Glob pattern syntax errors result in zero entries, which is valid
            // behavior (e.g., Include *.conf when no .conf files exist).
        }
    }

    let file_hosts = parse_content(&content, path)?;
    hosts.extend(file_hosts);

    Ok(())
}

fn parse_annotation(
    host: &mut SSHHost,
    raw: &str,
    file: &Path,
    line: usize,
) -> Result<(), SshxError> {
    let content = raw.trim_start_matches("## sshx:").trim();
    let parts: Vec<&str> = content.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(SshxError::AnnotationParseError {
            file: file.to_path_buf(),
            line,
            raw: raw.to_string(),
        });
    }

    let key = parts[0].trim();
    let value = parts[1].trim().trim_matches('"');

    match key {
        "group" => host.sshx.group = Some(value.to_string()),
        "password" => host.sshx.password = Some(value.to_string()),
        "requires" => host.sshx.requires = Some(value.to_string()),
        "alias" => host.sshx.alias = Some(value.to_string()),
        "background" => host.sshx.background = value == "true" || value == "1",
        "description" => host.sshx.description = Some(value.to_string()),
        "after_connect" => host.sshx.after_connect = Some(value.to_string()),
        _ => {
            return Err(SshxError::UnknownAnnotationKey {
                file: file.to_path_buf(),
                line,
                key: key.to_string(),
            });
        }
    }
    Ok(())
}

fn parse_option(host: &mut SSHHost, line: &str) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return;
    }

    let key = parts[0];
    let value = parts[1].trim();
    let key_normalized = key.to_ascii_lowercase();

    match key_normalized.as_str() {
        "hostname" => host.hostname = value.to_string(),
        "port" => {
            // Silently ignore invalid port values (SSH-compatible behavior)
            if let Ok(port) = value.parse::<u16>() {
                host.port = Some(port);
            }
        }
        "user" => host.user = Some(value.to_string()),
        "identityfile" => {
            host.identity_file = Some(PathBuf::from(
                value.replace("~", std::env::var("HOME").unwrap_or_default().as_str()),
            ))
        }
        "localforward" => {
            if let Some(forward) = parse_local_forward(value) {
                host.local_forwards.push(forward);
            }
        }
        "stricthostkeychecking" => {
            host.strict_host_checking = match value {
                "yes" => Some(StrictHostChecking::Yes),
                "no" => Some(StrictHostChecking::No),
                "ask" => Some(StrictHostChecking::Ask),
                "accept-new" | "accept_new" => Some(StrictHostChecking::AcceptNew),
                _ => None,
            };
        }
        "userknownhostsfile" => host.user_known_hosts_file = Some(value.to_string()),
        _ => host
            .extra_options
            .push((key.to_string(), value.to_string())),
    }
}

fn parse_local_forward(value: &str) -> Option<LocalForward> {
    // SSH silently ignores malformed LocalForward entries, so we return None
    // and the caller simply doesn't add anything to local_forwards.
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let local_port: u16 = parts[0].parse().ok()?;
    let remote_parts: Vec<&str> = parts[1].split(':').collect();
    if remote_parts.len() != 2 {
        return None;
    }

    let remote_host = remote_parts[0].to_string();
    let remote_port: u16 = remote_parts[1].parse().ok()?;

    Some(LocalForward {
        local_port,
        remote_host,
        remote_port,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn test_parse_basic_config() {
        let path = fixture("basic_config");
        let hosts = parse_file(&path).unwrap();
        assert_eq!(hosts.len(), 2);
        let web = &hosts[0];
        assert_eq!(web.name, "web-server");
        assert_eq!(web.hostname, "192.168.1.10");
        assert_eq!(web.port, Some(22));
        assert_eq!(web.user.as_deref(), Some("admin"));
        let db = &hosts[1];
        assert_eq!(db.name, "db-server");
        assert_eq!(db.hostname, "10.0.0.5");
        assert_eq!(db.port, Some(5432));
    }

    #[test]
    fn test_parse_empty_config() {
        let path = fixture("empty_config");
        let hosts = parse_file(&path).unwrap();
        assert!(hosts.is_empty());
    }

    #[test]
    fn test_parse_annotated_config() {
        let path = fixture("annotated_config");
        let hosts = parse_file(&path).unwrap();
        assert_eq!(hosts.len(), 2);

        let prod = &hosts[0];
        assert_eq!(prod.name, "prod-app");
        assert_eq!(prod.hostname, "203.0.113.50");
        assert_eq!(prod.port, Some(2222));
        assert_eq!(prod.user.as_deref(), Some("deploy"));
        assert_eq!(prod.sshx.group.as_deref(), Some("production"));
        assert_eq!(prod.sshx.alias.as_deref(), Some("prod"));
        assert_eq!(prod.sshx.password.as_deref(), Some("s3cret"));
        assert_eq!(
            prod.sshx.description.as_deref(),
            Some("Main production app server")
        );
        assert_eq!(
            prod.sshx.after_connect.as_deref(),
            Some("curl -s http://localhost:8080/health")
        );
        assert_eq!(prod.local_forwards.len(), 2);
        assert_eq!(prod.strict_host_checking, Some(StrictHostChecking::No));

        let staging = &hosts[1];
        assert_eq!(staging.name, "staging-app");
        assert_eq!(staging.sshx.group.as_deref(), Some("staging"));
        assert_eq!(staging.sshx.requires.as_deref(), Some("prod-app"));
    }

    #[test]
    fn test_parse_multi_group() {
        let path = fixture("multi_group_config");
        let hosts = parse_file(&path).unwrap();
        assert_eq!(hosts.len(), 4);

        let jump = &hosts[0];
        assert_eq!(jump.name, "jump-host");
        assert_eq!(jump.sshx.group.as_deref(), Some("infra"));
        assert_eq!(jump.sshx.alias.as_deref(), Some("jh"));

        let prod_app = &hosts[1];
        assert_eq!(prod_app.name, "prod-app");
        assert_eq!(prod_app.sshx.group.as_deref(), Some("production"));
        assert_eq!(prod_app.sshx.requires.as_deref(), Some("jump-host"));

        let prod_db = &hosts[2];
        assert_eq!(prod_db.name, "prod-db");
        assert_eq!(prod_db.sshx.group.as_deref(), Some("production"));
        assert_eq!(prod_db.sshx.alias.as_deref(), Some("pdb"));

        let dev = &hosts[3];
        assert_eq!(dev.name, "dev-box");
        assert_eq!(dev.sshx.group.as_deref(), Some("development"));
    }

    #[test]
    fn test_parse_source_locations() {
        let path = fixture("annotated_config");
        let hosts = parse_file(&path).unwrap();
        assert_eq!(hosts.len(), 2);

        let prod = &hosts[0];
        let staging = &hosts[1];

        assert!(prod.source.line_start > 0);
        assert!(prod.source.line_end >= prod.source.line_start);
        assert_eq!(prod.source.file, path);
        assert!(staging.source.line_start > prod.source.line_end);
        assert!(staging.source.line_end >= staging.source.line_start);
        assert_eq!(staging.source.file, path);
    }

    #[test]
    fn test_parse_content_final_host_line_end_without_trailing_newline() {
        let path = Path::new("inline_config");
        let content = "Host one\n  HostName 1.1.1.1\nHost two\n  HostName 2.2.2.2";

        let hosts = parse_content(content, path).unwrap();

        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].name, "one");
        assert_eq!(hosts[0].source.line_end, 2);
        assert_eq!(hosts[1].name, "two");
        assert_eq!(hosts[1].source.line_start, 3);
        assert_eq!(hosts[1].source.line_end, 4);
    }

    #[test]
    fn test_parse_option_keywords_case_insensitively() {
        let path = Path::new("inline_config");
        let content = "Host github.com\n  Hostname github.com\n  User git\n  IdentityFile ~/.ssh/github/seenark\n  IdentitiesOnly yes";

        let hosts = parse_content(content, path).unwrap();

        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].hostname, "github.com");
        assert_eq!(hosts[0].user.as_deref(), Some("git"));
        assert!(hosts[0].identity_file.is_some());
        assert!(
            hosts[0]
                .extra_options
                .contains(&(String::from("IdentitiesOnly"), String::from("yes")))
        );
    }

    #[test]
    fn test_parse_strict_host_checking() {
        let path = fixture("annotated_config");
        let hosts = parse_file(&path).unwrap();
        let prod = &hosts[0];
        assert_eq!(prod.strict_host_checking, Some(StrictHostChecking::No));
        assert_eq!(prod.user_known_hosts_file.as_deref(), Some("/dev/null"));
    }

    #[test]
    fn test_resolve_include_pattern() {
        let ssh_dir = Path::new("/tmp/ssh");
        let home = Path::new("/Users/tester");

        assert_eq!(
            resolve_include_pattern("~/.ssh/github/config", ssh_dir, Some(home)),
            "/Users/tester/.ssh/github/config"
        );
        assert_eq!(
            resolve_include_pattern("~", ssh_dir, Some(home)),
            "/Users/tester"
        );
        assert_eq!(
            resolve_include_pattern("included/*.conf", ssh_dir, Some(home)),
            "/tmp/ssh/included/*.conf"
        );
        assert_eq!(
            resolve_include_pattern("/etc/ssh/config", ssh_dir, Some(home)),
            "/etc/ssh/config"
        );
        assert_eq!(
            resolve_include_pattern("~/.ssh/github/config", ssh_dir, None),
            "~/.ssh/github/config"
        );
    }

    #[test]
    fn test_parse_with_includes() {
        let path = fixture("config_with_include");
        let (hosts, source_files) = parse_with_includes(&path).unwrap();
        assert_eq!(hosts.len(), 2);
        assert_eq!(source_files.len(), 2);
        assert!(hosts.iter().any(|h| h.name == "included-host"));
    }

    #[test]
    fn test_parse_with_multiple_include_patterns() {
        let tempdir = tempdir().unwrap();
        let included_a = tempdir.path().join("included-a.conf");
        let included_b = tempdir.path().join("included-b.conf");
        let root = tempdir.path().join("config");

        std::fs::write(&included_a, "Host included-a\n    HostName 10.0.0.1\n").unwrap();
        std::fs::write(&included_b, "Host included-b\n    HostName 10.0.0.2\n").unwrap();
        std::fs::write(
            &root,
            format!(
                "Include {} {}\nHost root-host\n    HostName 10.0.0.3\n",
                included_a.display(),
                included_b.display()
            ),
        )
        .unwrap();

        let (hosts, _) = parse_with_includes(&root).unwrap();

        assert_eq!(hosts.len(), 3);
        assert!(hosts.iter().any(|host| host.name == "included-a"));
        assert!(hosts.iter().any(|host| host.name == "included-b"));
        assert!(hosts.iter().any(|host| host.name == "root-host"));
    }

    #[test]
    fn test_circular_include_prevention() {
        let temp_dir = std::env::temp_dir();
        let base = temp_dir.join("sshx_circular_test");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        let file_a = base.join("a.conf");
        let file_b = base.join("b.conf");

        std::fs::write(&file_a, "Include b.conf\nHost host-a\n    HostName 1.1.1.1").unwrap();
        std::fs::write(&file_b, "Include a.conf\nHost host-b\n    HostName 2.2.2.2").unwrap();

        let (hosts, source_files) = parse_with_includes(&file_a).unwrap();

        assert_eq!(hosts.len(), 2);
        assert!(source_files.len() <= 2);

        let _ = std::fs::remove_dir_all(&base);
    }
}
