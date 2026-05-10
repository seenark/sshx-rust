use std::path::PathBuf;

use crate::error::SshxError;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SshxConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ui: UIConfig,
    #[serde(default)]
    pub tunnel: TunnelConfig,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GeneralConfig {
    #[serde(default)]
    pub ssh_config_path: Option<PathBuf>,
    #[serde(default)]
    pub default_user: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UIConfig {
    #[serde(default = "default_fuzzy_threshold")]
    pub fuzzy_threshold: f32,
    #[serde(default = "default_host_weight")]
    pub host_weight: f32,
    #[serde(default = "default_hostname_weight")]
    pub hostname_weight: f32,
    #[serde(default = "default_true")]
    pub show_descriptions: bool,
    #[serde(default = "default_true")]
    pub show_hostnames: bool,
    #[serde(default)]
    pub group_sort: GroupSort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupSort {
    #[default]
    Alphabetical,
    ConfigOrder,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TunnelConfig {
    #[serde(default = "default_check_interval")]
    pub check_interval_ms: u64,
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_s: u64,
}

fn default_fuzzy_threshold() -> f32 { 0.4 }
fn default_host_weight() -> f32 { 0.7 }
fn default_hostname_weight() -> f32 { 0.3 }
fn default_true() -> bool { true }
fn default_check_interval() -> u64 { 500 }
fn default_connect_timeout() -> u64 { 10 }

impl Default for SshxConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ui: UIConfig::default(),
            tunnel: TunnelConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            ssh_config_path: None,
            default_user: None,
        }
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            fuzzy_threshold: default_fuzzy_threshold(),
            host_weight: default_host_weight(),
            hostname_weight: default_hostname_weight(),
            show_descriptions: default_true(),
            show_hostnames: default_true(),
            group_sort: GroupSort::Alphabetical,
        }
    }
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: default_check_interval(),
            connect_timeout_s: default_connect_timeout(),
        }
    }
}

impl SshxConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sshx")
            .join("config.toml")
    }

    pub fn load() -> Result<SshxConfig, SshxError> {
        Self::load_from(&Self::config_path())
    }

    pub fn load_from(path: &PathBuf) -> Result<SshxConfig, SshxError> {
        if !path.exists() {
            return Ok(SshxConfig::default());
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) if c.trim().is_empty() => return Ok(SshxConfig::default()),
            Ok(c) => c,
            Err(e) => {
                return Err(SshxError::SshxConfigParseFailed {
                    path: path.clone(),
                    reason: format!("failed to read file: {}", e),
                });
            }
        };

        match toml::from_str::<SshxConfig>(&content) {
            Ok(config) => Ok(config),
            Err(e) => Err(SshxError::SshxConfigParseFailed {
                path: path.clone(),
                reason: e.to_string(),
            }),
        }
    }

    pub fn ssh_config_path(&self) -> PathBuf {
        self.general
            .ssh_config_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(".ssh").join("config"))
    }

    pub fn generate_default_toml() -> String {
        let config = SshxConfig::default();
        toml::to_string_pretty(&config).expect("failed to serialize default config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_defaults() {
        let config = SshxConfig::default();
        assert_eq!(config.general.ssh_config_path, None);
        assert_eq!(config.general.default_user, None);
        assert_eq!(config.ui.fuzzy_threshold, 0.4);
        assert_eq!(config.ui.host_weight, 0.7);
        assert_eq!(config.ui.hostname_weight, 0.3);
        assert_eq!(config.ui.show_descriptions, true);
        assert_eq!(config.ui.show_hostnames, true);
        assert_eq!(config.ui.group_sort, GroupSort::Alphabetical);
        assert_eq!(config.tunnel.check_interval_ms, 500);
        assert_eq!(config.tunnel.connect_timeout_s, 10);
    }

    #[test]
    fn test_load_missing_file_returns_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let config = SshxConfig::load_from(&path).unwrap();
        assert_eq!(config, SshxConfig::default());
    }

    #[test]
    fn test_load_empty_file_returns_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.toml");
        std::fs::write(&path, "").unwrap();
        let config = SshxConfig::load_from(&path).unwrap();
        assert_eq!(config, SshxConfig::default());
    }

    #[test]
    fn test_load_valid_partial() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("partial.toml");
        std::fs::write(
            &path,
            r#"
[ui]
fuzzy_threshold = 0.6
show_descriptions = false
"#,
        )
        .unwrap();
        let config = SshxConfig::load_from(&path).unwrap();
        assert_eq!(config.ui.fuzzy_threshold, 0.6);
        assert_eq!(config.ui.show_descriptions, false);
        assert_eq!(config.ui.host_weight, 0.7);
        assert_eq!(config.tunnel.check_interval_ms, 500);
    }

    #[test]
    fn test_load_malformed_toml_is_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "invalid = toml [[[").unwrap();
        let result = SshxConfig::load_from(&path);
        assert!(matches!(result, Err(SshxError::SshxConfigParseFailed { .. })));
    }

    #[test]
    fn test_generate_default_toml_roundtrip() {
        let toml_str = SshxConfig::generate_default_toml();
        let config = toml::from_str::<SshxConfig>(&toml_str).unwrap();
        assert_eq!(config, SshxConfig::default());
    }

    #[test]
    fn test_ssh_config_path_default() {
        let config = SshxConfig::default();
        let ssh_path = config.ssh_config_path();
        assert!(ssh_path.components().any(|c| c.as_os_str() == ".ssh"));
        assert!(ssh_path.file_name().map(|n| n == "config").unwrap_or(false));
    }
}