use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::config::YaatConfig;
use crate::git::GitRepo;
use crate::known_configs::{KNOWN_CONFIGS, KNOWN_HOME_FILES};
use crate::platform::ensure_dir;

pub fn execute(repo_path: PathBuf, clone_url: Option<String>) -> Result<()> {
    info!("Initializing YAAT repository at {}", repo_path.display());

    // Check if it's already a YAAT repository
    if is_yaat_repo(&repo_path) {
        bail!("YAAT repository already exists at {}", repo_path.display());
    }

    // Check if directory already exists and is not empty
    if repo_path.exists() {
        // Check if directory is empty (only allow if empty)
        let is_empty = repo_path.read_dir()?.next().is_none();
        if !is_empty {
            bail!(
                "Directory already exists and is not empty: {}. \
                 Remove it first or use a different path.",
                repo_path.display()
            );
        }
        info!("Using existing empty directory {}", repo_path.display());
    } else {
        // Create the directory
        ensure_dir(&repo_path)?;
        info!("Created directory {}", repo_path.display());
    }

    // Auto-detect known configurations
    let detected_configs = detect_known_configs();

    // Create config file (yaat.yaml) with detected configs
    let mut config = YaatConfig::default();
    config.include = detected_configs;
    let config_path = repo_path.join("yaat.yaml");
    config.to_file(&config_path)?;
    debug!("Created yaat.yaml");

    // Create directory structure
    let config_dir = repo_path.join("config");
    let home_dir = repo_path.join("home");
    let hosts_dir = repo_path.join("hosts");

    ensure_dir(&config_dir)?;
    ensure_dir(&home_dir)?;
    ensure_dir(&hosts_dir)?;
    info!("Created directory structure (config/, home/, hosts/)");

    // Initialize or clone git repository
    let was_cloned = clone_url.is_some();
    let repo = if let Some(url) = clone_url {
        GitRepo::clone(&url, &repo_path)?
    } else {
        GitRepo::init(&repo_path)?
    };

    // Add yaat.yaml and create initial commit
    repo.add(&config_path)?;

    let commit_msg = if was_cloned {
        "Initialize YAAT repository from remote"
    } else {
        "Initialize YAAT repository"
    };

    repo.commit(commit_msg)?;

    // Display detected configs
    let detected_count = config
        .include
        .iter()
        .filter(|s| !s.starts_with('#'))
        .count();

    info!(
        "✓ Successfully initialized YAAT repository at {}",
        repo_path.display()
    );

    if detected_count > 0 {
        info!("  Detected {} known configurations:", detected_count);
        for item in &config.include {
            if !item.starts_with('#') {
                info!("    - {}", item);
            }
        }
        info!(
            "  Edit {} to add/remove configurations",
            config_path.display()
        );
    } else {
        info!("  No known configurations detected.");
        info!(
            "  Edit {} to add your configurations manually",
            config_path.display()
        );
    }

    info!("  Config directory: {}", config_dir.display());
    info!("  Home files: {}", home_dir.display());
    info!("  Host-specific configs: {}", hosts_dir.display());

    Ok(())
}

/// Check if a directory is already a YAAT repository
fn is_yaat_repo(path: &PathBuf) -> bool {
    path.join("yaat.yaml").exists() && path.join(".git").exists()
}

/// Detect known configuration directories and files
fn detect_known_configs() -> Vec<String> {
    let mut include = Vec::new();

    // Detect known configs in ~/.config/
    if let Some(config_dir) = dirs::config_dir() {
        for config in KNOWN_CONFIGS {
            if config_dir.join(config).exists() {
                include.push(format!("config/{}/", config));
            }
        }
    }

    // Detect known files in ~/
    if let Some(home_dir) = dirs::home_dir() {
        for (file, repo_path) in KNOWN_HOME_FILES {
            if home_dir.join(file).exists() {
                include.push(repo_path.to_string());
            }
        }
    }

    // Add comment if nothing detected
    if include.is_empty() {
        include.push("# No known configurations detected".to_string());
        include.push("# Add your configurations below, e.g.:".to_string());
        include.push("# - config/hypr/".to_string());
        include.push("# - home/.bashrc".to_string());
    }

    // Add optional IDE configs as comments
    include.push("#".to_string());
    include.push("# Optional: Uncomment to include IDE settings".to_string());
    include.push("# - config/Code/          # VS Code".to_string());
    include.push("# - config/Cursor/        # Cursor IDE".to_string());

    include
}
