use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::commands::CommandContext;
use crate::config::YaatConfig;
use crate::git::GitRepo;
use crate::platform::{default_repo_path, ensure_dir};

pub fn execute(
    path: Option<PathBuf>,
    clone_url: Option<String>,
    _context: &mut CommandContext,
) -> Result<()> {
    let repo_path = match path {
        Some(p) => p,
        None => default_repo_path()?,
    };

    // Expand ~ to home directory if present
    let repo_path = if repo_path.starts_with("~") {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        home.join(repo_path.strip_prefix("~").unwrap())
    } else {
        repo_path
    };

    info!("Initializing YAAT repository at {}", repo_path.display());

    // Check if path already exists
    if repo_path.exists() {
        if crate::git::is_git_repo(&repo_path) {
            bail!("A git repository already exists at {}", repo_path.display());
        } else {
            bail!(
                "Directory already exists at {} but is not a git repository",
                repo_path.display()
            );
        }
    }

    // Create the directory
    ensure_dir(&repo_path)?;

    // Clone or initialize
    let was_cloned = clone_url.is_some();
    let repo = if let Some(url) = clone_url {
        GitRepo::clone(&url, &repo_path)?
    } else {
        GitRepo::init(&repo_path)?
    };

    // Create config file
    let config_path = repo_path.join("yaat.yaml");
    if !config_path.exists() {
        let config = YaatConfig::default();
        config.to_file(&config_path)?;
        debug!("Created config file at {}", config_path.display());
    }

    // Create directory structure
    let config_dir = repo_path.join("config");
    let home_dir = repo_path.join("home");
    let hosts_dir = repo_path.join("hosts");

    ensure_dir(&config_dir)?;
    ensure_dir(&home_dir)?;
    ensure_dir(&hosts_dir)?;

    // Add initial files and commit
    repo.add(&config_path)?;
    repo.add(&config_dir)?;
    repo.add(&home_dir)?;
    repo.add(&hosts_dir)?;

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
