use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct YaatConfig {
    /// Repository path (relative to home or absolute)
    #[serde(default = "default_repo_path")]
    pub repo_path: String,

    /// Default hostname to use when not specified
    #[serde(default = "default_hostname")]
    pub default_host: String,

    /// Files/directories to exclude from sync
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Directories in ~/.config/ to include (backed up as complete folders, symlinked as directories)
    #[serde(default)]
    pub config_dirs: Vec<String>,

    /// Individual files in ~/ to include (backed up and symlinked individually)
    #[serde(default)]
    pub home_files: Vec<String>,

    /// Unified include list (e.g., config/hypr/, home/.gitconfig)
    /// If provided, config_dirs and home_files are derived from it.
    #[serde(default)]
    pub include: Vec<String>,

    /// Host-specific configurations
    #[serde(default)]
    pub hosts: HashMap<String, HostConfig>,

    /// Symlink settings
    #[serde(default)]
    pub symlink: SymlinkConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HostConfig {
    /// Files specific to this host
    #[serde(default)]
    pub files: Vec<String>,

    /// Additional exclude patterns for this host
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Environment variables for this host
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymlinkConfig {
    /// Create symlinks instead of copying files
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Backup existing files before creating symlinks
    #[serde(default = "default_true")]
    pub backup: bool,
}

impl Default for YaatConfig {
    fn default() -> Self {
        Self {
            repo_path: default_repo_path(),
            default_host: default_hostname(),
            exclude: vec![
                ".git".to_string(),
                ".gitignore".to_string(),
                "yaat.yaml".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".cache".to_string(),
            ],
            config_dirs: vec![], // Empty by default, will be populated during init
            home_files: vec![],  // Empty by default, will be populated during init
            include: vec![],
            hosts: HashMap::new(),
            symlink: SymlinkConfig::default(),
        }
    }
}

impl Default for SymlinkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backup: true,
        }
    }
}

fn default_repo_path() -> String {
    "~/.dotfiles".to_string()
}

fn default_hostname() -> String {
    crate::platform::hostname().unwrap_or_else(|_| "default".to_string())
}

fn default_true() -> bool {
    true
}

impl YaatConfig {
    /// Load configuration from a YAML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: YaatConfig = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Save configuration to a YAML file
    pub fn to_file(&self, path: &PathBuf) -> Result<()> {
        let contents = serde_yaml::to_string(self).context("Failed to serialize config")?;

        std::fs::write(path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Get the repository path, expanding ~ to home directory
    #[allow(dead_code)]
    pub fn expand_repo_path(&self) -> Result<PathBuf> {
        if self.repo_path.starts_with("~/") {
            let home = dirs::home_dir().context("Could not determine home directory")?;
            let rest = &self.repo_path[2..];
            Ok(home.join(rest))
        } else {
            Ok(PathBuf::from(&self.repo_path))
        }
    }

    /// Get host-specific config or default
    pub fn get_host_config(&self, hostname: Option<&str>) -> Option<&HostConfig> {
        let host = hostname.unwrap_or(&self.default_host);
        self.hosts.get(host)
    }

    /// Check if a file should be excluded
    pub fn is_excluded(&self, path: &str, hostname: Option<&str>) -> bool {
        // Check global excludes
        if self.exclude.iter().any(|e| path.contains(e)) {
            return true;
        }

        // Check host-specific excludes
        if let Some(host_config) = self.get_host_config(hostname) {
            if host_config.exclude.iter().any(|e| path.contains(e)) {
                return true;
            }
        }

        false
    }

    /// Check if a config directory is managed
    #[allow(dead_code)]
    pub fn is_config_dir_managed(&self, name: &str) -> bool {
        self.config_dirs
            .iter()
            .any(|d| d == name || d.trim_end_matches('/') == name)
    }

    /// Check if a home file is managed
    pub fn is_home_file_managed(&self, name: &str) -> bool {
        self.home_files
            .iter()
            .any(|f| f == name || f.trim_start_matches('.') == name.trim_start_matches('.'))
    }

    /// Get effective config dirs, deriving from `include` if present.
    pub fn effective_config_dirs(&self) -> Vec<String> {
        if !self.include.is_empty() {
            let mut dirs = Vec::new();
            for item in &self.include {
                let trimmed = item.trim_end_matches('/');
                if trimmed.starts_with("config/") {
                    let rest = &trimmed[7..]; // after "config/"
                    if !rest.contains('/') {
                        // Top-level config dir, e.g., config/hypr
                        dirs.push(rest.to_string());
                    }
                }
            }
            dirs
        } else {
            self.config_dirs.clone()
        }
    }

    /// Get effective home files, deriving from `include` if present.
    pub fn effective_home_files(&self) -> Vec<String> {
        if !self.include.is_empty() {
            let mut files = Vec::new();
            for item in &self.include {
                let trimmed = item.trim_end_matches('/');
                if trimmed.starts_with("home/") {
                    files.push(trimmed.strip_prefix("home/").unwrap().to_string());
                }
            }
            files
        } else {
            self.home_files.clone()
        }
    }
}
