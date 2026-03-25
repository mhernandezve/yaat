use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::commands::CommandContext;
use crate::git::GitRepo;
use crate::platform::{config_dir, system_to_repo_path};

pub fn execute(host: Option<String>, dry_run: bool, context: &mut CommandContext) -> Result<()> {
    let hostname = host.unwrap_or_else(|| context.config.default_host.clone());
    info!("Backing up configurations for host: {}", hostname);

    if dry_run {
        info!("[DRY RUN] No changes will be made");
    }

    let config_dir = config_dir()?;
    let home = dirs::home_dir().context("Could not determine home directory")?;

    let mut backed_up = 0;
    let mut skipped = 0;

    // Backup config files
    info!("Backing up config directory: {}", config_dir.display());
    let (count, skip) = backup_config_files(&config_dir, &home, context, dry_run)?;
    backed_up += count;
    skipped += skip;

    // Backup home files (if tracked)
    let tracked_home_files = get_tracked_home_files(context)?;
    if !tracked_home_files.is_empty() {
        info!("Backing up tracked home files...");
        for file in tracked_home_files {
            if !file.exists() {
                debug!("File does not exist, skipping: {}", file.display());
                skipped += 1;
                continue;
            }

            let repo_path = system_to_repo_path(&file, &context.repo_path)?;
            let target = context.repo_path.join(&repo_path);

            if backup_file(&file, &target, dry_run)? {
                backed_up += 1;
            } else {
                skipped += 1;
            }
        }
    }

    // Commit changes if not dry run
    if !dry_run && backed_up > 0 {
        let repo = GitRepo::open(&context.repo_path)?;

        // Stage all changes
        repo.add(&context.repo_path)?;

        // Commit
        let commit_msg = format!("Backup configurations for {}", hostname);
        repo.commit(&commit_msg)?;

        info!("✓ Created commit: {}", commit_msg);
    }

    if dry_run {
        info!(
            "[DRY RUN] Would backup {} files, skip {}",
            backed_up, skipped
        );
    } else {
        info!(
            "✓ Successfully backed up {} files, skipped {}",
            backed_up, skipped
        );
    }

    Ok(())
}

fn backup_config_files(
    config_dir: &Path,
    _home_dir: &Path,
    context: &mut CommandContext,
    dry_run: bool,
) -> Result<(usize, usize)> {
    let mut backed_up = 0;
    let mut skipped = 0;

    // Walk the config directory
    for entry in walkdir::WalkDir::new(config_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let system_path = entry.path();

        // Skip directories
        if system_path.is_dir() {
            continue;
        }

        // Convert to relative path
        let repo_relative =
            match system_to_repo_path(&system_path.to_path_buf(), &context.repo_path) {
                Ok(p) => p,
                Err(e) => {
                    debug!("Could not convert path: {} - {}", system_path.display(), e);
                    skipped += 1;
                    continue;
                }
            };

        // Check if excluded
        if context
            .config
            .is_excluded(&repo_relative.to_string_lossy(), None)
        {
            debug!("Skipping excluded file: {}", system_path.display());
            skipped += 1;
            continue;
        }

        // Determine target path in repo
        let target = context.repo_path.join(&repo_relative);

        if backup_file(system_path, &target, dry_run)? {
            backed_up += 1;
        } else {
            skipped += 1;
        }
    }

    Ok((backed_up, skipped))
}

fn backup_file(source: &Path, target: &Path, dry_run: bool) -> Result<bool> {
    // Check if file has changed
    if target.exists() {
        let source_modified = fs::metadata(source).and_then(|m| m.modified()).ok();
        let target_modified = fs::metadata(target).and_then(|m| m.modified()).ok();

        if source_modified == target_modified {
            // Check content hash (simplified: compare files)
            if files_equal(source, target)? {
                debug!("File unchanged, skipping: {}", source.display());
                return Ok(false);
            }
        }

        if dry_run {
            info!(
                "[DRY RUN] Would update: {} -> {}",
                source.display(),
                target.display()
            );
        } else {
            // Copy updated file
            fs::copy(source, target).with_context(|| {
                format!(
                    "Failed to copy: {} -> {}",
                    source.display(),
                    target.display()
                )
            })?;
            info!("Updated: {} -> {}", source.display(), target.display());
        }
    } else {
        // Ensure parent directory exists
        if let Some(parent) = target.parent() {
            if !parent.exists() && !dry_run {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
                debug!("Created directory: {}", parent.display());
            }
        }

        if dry_run {
            info!(
                "[DRY RUN] Would backup: {} -> {}",
                source.display(),
                target.display()
            );
        } else {
            fs::copy(source, target).with_context(|| {
                format!(
                    "Failed to copy: {} -> {}",
                    source.display(),
                    target.display()
                )
            })?;
            info!("Backed up: {} -> {}", source.display(), target.display());
        }
    }

    Ok(true)
}

fn files_equal(a: &Path, b: &Path) -> Result<bool> {
    let a_content = fs::read(a)?;
    let b_content = fs::read(b)?;
    Ok(a_content == b_content)
}

fn get_tracked_home_files(context: &CommandContext) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let home_dir = context.repo_path.join("home");
    if !home_dir.exists() {
        return Ok(files);
    }

    let home_dir_clone = home_dir.clone();

    for entry in walkdir::WalkDir::new(home_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let repo_path = entry.path();

        if repo_path.is_dir() {
            continue;
        }

        // Convert repo path to system path
        let relative = repo_path
            .strip_prefix(&home_dir_clone)
            .context("Invalid home path in repo")?;
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let system_path = home.join(relative);

        files.push(system_path);
    }

    Ok(files)
}
