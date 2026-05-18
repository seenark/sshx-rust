use crate::error::SshxError;
use crate::model::*;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ConfigIndex {
    pub hosts: Vec<SSHHost>,
    pub groups: IndexMap<String, Vec<usize>>,
    pub aliases: HashMap<String, usize>,
    pub source_files: Vec<PathBuf>,
}

impl ConfigIndex {
    pub fn build(hosts: Vec<SSHHost>) -> Result<Self, SshxError> {
        let mut groups: IndexMap<String, Vec<usize>> = IndexMap::new();
        let mut aliases: HashMap<String, usize> = HashMap::new();

        for (idx, host) in hosts.iter().enumerate() {
            if let Some(ref group) = host.sshx.group {
                groups.entry(group.clone()).or_default().push(idx);
            }

            if let Some(ref alias) = host.sshx.alias {
                if let Some(existing_idx) = aliases.get(alias) {
                    let existing_host = &hosts[*existing_idx];
                    return Err(SshxError::DuplicateAlias {
                        alias: alias.clone(),
                        host1: existing_host.name.clone(),
                        host2: host.name.clone(),
                    });
                }
                aliases.insert(alias.clone(), idx);
            }
        }

        for host in &hosts {
            if let Some(ref requires) = host.sshx.requires {
                let requires_exists = hosts.iter().any(|h| h.name == *requires);
                if !requires_exists {
                    return Err(SshxError::RequiresHostNotFound {
                        host: host.name.clone(),
                        requires: requires.clone(),
                    });
                }
            }
        }

        let name_to_idx: HashMap<&str, usize> = hosts
            .iter()
            .enumerate()
            .map(|(i, h)| (h.name.as_str(), i))
            .collect();

        for (idx, host) in hosts.iter().enumerate() {
            if let Some(ref requires) = host.sshx.requires {
                let mut visited = std::collections::HashSet::new();
                visited.insert(idx);
                let mut current_requires = Some(requires.as_str());
                while let Some(req) = current_requires {
                    if let Some(&req_idx) = name_to_idx.get(req) {
                        let req_host = &hosts[req_idx];
                        if let Some(ref req_requires) = req_host.sshx.requires {
                            let req_requires_idx = *name_to_idx
                                .get(req_requires.as_str())
                                .expect("requires target must exist");
                            if visited.contains(&req_requires_idx) {
                                return Err(SshxError::CircularRequires {
                                    host: host.name.clone(),
                                });
                            }
                            visited.insert(req_requires_idx);
                            current_requires = Some(req_requires.as_str());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(ConfigIndex {
            hosts,
            groups,
            aliases,
            source_files: Vec::new(),
        })
    }

    pub fn resolve_alias(&self, input: &str) -> Option<&SSHHost> {
        if let Some(&idx) = self.aliases.get(input) {
            return self.hosts.get(idx);
        }
        self.find_host(input)
    }

    pub fn find_host(&self, name: &str) -> Option<&SSHHost> {
        self.hosts.iter().find(|h| h.name == name)
    }

    pub fn hosts_in_group(&self, group: &str) -> Vec<&SSHHost> {
        self.groups
            .get(group)
            .map(|indices| indices.iter().filter_map(|&i| self.hosts.get(i)).collect())
            .unwrap_or_default()
    }

    pub fn jump_host_for(&self, host: &SSHHost) -> Option<&SSHHost> {
        host.sshx
            .requires
            .as_ref()
            .and_then(|req| self.find_host(req))
    }

    pub fn load(ssh_config_path: Option<&PathBuf>) -> Result<Self, SshxError> {
        let sshx_config = crate::config::SshxConfig::load().unwrap_or_default();
        let config_path = ssh_config_path
            .cloned()
            .unwrap_or_else(|| sshx_config.ssh_config_path());

        let (hosts, source_files) = crate::parser::parse_with_includes(&config_path)?;

        let mut index = Self::build(hosts)?;
        index.source_files = source_files;
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use std::path::PathBuf;

    fn fixture_hosts(name: &str) -> Vec<SSHHost> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name);
        parser::parse_file(&path).unwrap()
    }

    #[test]
    fn test_build_index_basic() {
        let hosts = fixture_hosts("basic_config");
        let index = ConfigIndex::build(hosts).unwrap();
        assert_eq!(index.hosts.len(), 2);
        assert!(index.groups.is_empty());
        assert!(index.aliases.is_empty());
    }

    #[test]
    fn test_find_host_by_name() {
        let hosts = fixture_hosts("basic_config");
        let index = ConfigIndex::build(hosts).unwrap();
        let web = index.find_host("web-server");
        assert!(web.is_some());
        assert_eq!(web.unwrap().hostname, "192.168.1.10");
    }

    #[test]
    fn test_resolve_alias() {
        let hosts = fixture_hosts("annotated_config");
        let index = ConfigIndex::build(hosts).unwrap();
        let prod = index.resolve_alias("prod");
        assert!(prod.is_some());
        assert_eq!(prod.unwrap().name, "prod-app");
    }

    #[test]
    fn test_hosts_in_group() {
        let hosts = fixture_hosts("multi_group_config");
        let index = ConfigIndex::build(hosts).unwrap();
        let production_hosts = index.hosts_in_group("production");
        assert_eq!(production_hosts.len(), 2);
        let names: Vec<&str> = production_hosts.iter().map(|h| h.name.as_str()).collect();
        assert!(names.contains(&"prod-app"));
        assert!(names.contains(&"prod-db"));
    }

    #[test]
    fn test_jump_host_for() {
        let hosts = fixture_hosts("multi_group_config");
        let index = ConfigIndex::build(hosts).unwrap();
        let prod_app = index.find_host("prod-app").unwrap();
        let jump = index.jump_host_for(prod_app);
        assert!(jump.is_some());
        assert_eq!(jump.unwrap().name, "jump-host");
    }

    #[test]
    fn test_duplicate_alias_error() {
        let hosts = vec![
            SSHHost {
                name: "host1".to_string(),
                hostname: "192.168.1.1".to_string(),
                sshx: SSHXAnnotations {
                    alias: Some("same".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            SSHHost {
                name: "host2".to_string(),
                hostname: "192.168.1.2".to_string(),
                sshx: SSHXAnnotations {
                    alias: Some("same".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];
        let result = ConfigIndex::build(hosts);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            SshxError::DuplicateAlias {
                alias,
                host1,
                host2,
            } => {
                assert_eq!(alias, "same");
                assert_eq!(host1, "host1");
                assert_eq!(host2, "host2");
            }
            _ => panic!("Expected DuplicateAlias error"),
        }
    }

    #[test]
    fn test_requires_not_found_error() {
        let hosts = vec![SSHHost {
            name: "myhost".to_string(),
            hostname: "192.168.1.1".to_string(),
            sshx: SSHXAnnotations {
                requires: Some("nonexistent".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }];
        let result = ConfigIndex::build(hosts);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            SshxError::RequiresHostNotFound { host, requires } => {
                assert_eq!(host, "myhost");
                assert_eq!(requires, "nonexistent");
            }
            _ => panic!("Expected RequiresHostNotFound error"),
        }
    }

    #[test]
    fn test_circular_requires_error() {
        let hosts = vec![
            SSHHost {
                name: "host-a".to_string(),
                hostname: "192.168.1.1".to_string(),
                sshx: SSHXAnnotations {
                    requires: Some("host-b".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            SSHHost {
                name: "host-b".to_string(),
                hostname: "192.168.1.2".to_string(),
                sshx: SSHXAnnotations {
                    requires: Some("host-a".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];
        let result = ConfigIndex::build(hosts);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            SshxError::CircularRequires { host } => {
                assert_eq!(host, "host-a");
            }
            _ => panic!("Expected CircularRequires error"),
        }
    }

    #[test]
    fn test_full_load_pipeline() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("multi_group_config");
        let config_path = Some(&path);
        let index = ConfigIndex::load(config_path).unwrap();
        assert_eq!(index.hosts.len(), 4);
        assert_eq!(index.groups.len(), 3);
        assert_eq!(index.aliases.len(), 2);
    }
}
