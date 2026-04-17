use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the default dotfiles repository path
/// On Unix: ~/.dotfiles
/// On Windows: %USERPROFILE%\.dotfiles
pub fn default_repo_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".dotfiles"))
}

/// Get the configuration directory path
/// On Unix: ~/.config
/// On Windows: %APPDATA%
pub fn config_dir() -> Result<PathBuf> {
    dirs::config_dir().context("Could not determine config directory")
}

/// Get the current hostname
pub fn hostname() -> Result<String> {
    hostname::get()
        .context("Could not determine hostname")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("Hostname contains invalid UTF-8"))
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Get the YAAT configuration file path inside the repo
pub fn yaat_config_path(repo_path: &PathBuf) -> PathBuf {
    repo_path.join("yaat.yaml")
}

/// Convert a system config path to a relative path within the repo
/// e.g., ~/.config/nvim/init.lua -> config/nvim/init.lua
pub fn system_to_repo_path(system_path: &PathBuf, repo_path: &PathBuf) -> Result<PathBuf> {
    let config = config_dir()?;
    let home = dirs::home_dir().context("Could not determine home directory")?;

    // Check if it's in ~/.config
    if let Ok(relative) = system_path.strip_prefix(&config) {
        return Ok(PathBuf::from("config").join(relative));
    }

    // Check if it's directly in home
    if let Ok(relative) = system_path.strip_prefix(&home) {
        return Ok(PathBuf::from("home").join(relative));
    }

    // Otherwise, use absolute path relative to repo
    let relative = system_path
        .strip_prefix(repo_path)
        .map_err(|_| anyhow::anyhow!("Path is not within home, config, or repo directories"))?;
    Ok(relative.to_path_buf())
}

/// Convert a repo-relative path to the corresponding system path
#[allow(dead_code)]
pub fn repo_to_system_path(repo_relative: &PathBuf, repo_path: &PathBuf) -> Result<PathBuf> {
    if repo_relative.starts_with("config/") {
        let config = config_dir()?;
        let stripped = repo_relative
            .strip_prefix("config/")
            .map_err(|_| anyhow::anyhow!("Invalid config path"))?;
        return Ok(config.join(stripped));
    }

    if repo_relative.starts_with("home/") {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let stripped = repo_relative
            .strip_prefix("home/")
            .map_err(|_| anyhow::anyhow!("Invalid home path"))?;
        return Ok(home.join(stripped));
    }

    Ok(repo_path.join(repo_relative))
}
