use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::commands::CommandContext;
use crate::git::GitRepo;
use crate::platform::system_to_repo_path;

pub fn execute(file: PathBuf, host: Option<String>, context: &mut CommandContext) -> Result<()> {
    let file = if file.starts_with("~") {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        home.join(file.strip_prefix("~").unwrap())
    } else {
        file.canonicalize()
            .with_context(|| format!("File not found: {}", file.display()))?
    };

    if !file.exists() {
        bail!("File does not exist: {}", file.display());
    }

    info!("Adding {} to YAAT repository", file.display());

    // Determine where to place the file in the repo
    let repo_relative = system_to_repo_path(&file, &context.repo_path)?;
    let target_path = context.repo_path.join(&repo_relative);

    debug!("Target path in repo: {}", target_path.display());

    // Check if already in repo
    if target_path.exists() {
        warn!(
            "File already exists in repository at {}",
            target_path.display()
        );
        info!("Use 'yaat sync' to update instead");
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Copy the file to the repo
    if file.is_file() {
        std::fs::copy(&file, &target_path)
            .with_context(|| format!("Failed to copy file to {}", target_path.display()))?;
    } else if file.is_dir() {
        copy_dir_all(&file, &target_path)?;
    }

    info!("Copied {} to {}", file.display(), target_path.display());

    // Open repo and add to git
    let repo = GitRepo::open(&context.repo_path)?;
    repo.add(&target_path)?;

    // Create commit message
    let commit_msg = format!(
        "Add {} {}",
        if file.is_dir() { "directory" } else { "file" },
        repo_relative.display()
    );

    repo.commit(&commit_msg)?;

    info!(
        "✓ Successfully added {} to repository",
        repo_relative.display()
    );

    // Update config if host-specific
    if let Some(hostname) = host {
        info!("  (Host-specific: {})", hostname);
        // TODO: Update config to track host-specific files
    }

    Ok(())
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create directory: {}", dst.display()))?;

    for entry in std::fs::read_dir(src)
        .with_context(|| format!("Failed to read directory: {}", src.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().context("Invalid file name")?;
        let dest_path = dst.join(file_name);

        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)
                .with_context(|| format!("Failed to copy file: {}", path.display()))?;
        }
    }

    Ok(())
}
