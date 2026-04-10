use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::config::YaatConfig;
use crate::git::GitRepo;
use crate::platform::ensure_dir;

pub fn execute(repo_path: PathBuf, clone_url: Option<String>) -> Result<()> {
    info!("Initializing YAAT repository at {}", repo_path.display());

    // Check if it's already a YAAT repository
    if is_yaat_repo(&repo_path) {
        bail!("YAAT repository already exists at {}", repo_path.display());
    }

    // Check if directory already exists
    if repo_path.exists() {
        bail!(
            "Directory already exists: {}. \
             Remove it first or use a different path.",
            repo_path.display()
        );
    }

    // Create the directory
    ensure_dir(&repo_path)?;
    info!("Created directory {}", repo_path.display());

    // Create config file (yaat.yaml) FIRST
    let config = YaatConfig::default();
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

    info!(
        "✓ Successfully initialized YAAT repository at {}",
        repo_path.display()
    );
    info!("  Config directory: {}", config_dir.display());
    info!("  Home files: {}", home_dir.display());
    info!("  Host-specific configs: {}", hosts_dir.display());

    Ok(())
}

/// Check if a directory is already a YAAT repository
fn is_yaat_repo(path: &PathBuf) -> bool {
    path.join("yaat.yaml").exists() && path.join(".git").exists()
}
