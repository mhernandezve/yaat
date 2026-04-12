use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

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

    // Backup config directories (complete folders)
    if !context.config.config_dirs.is_empty() {
        info!("Backing up config directories...");
        let (count, skip) = backup_config_dirs(&config_dir, context, dry_run)?;
        backed_up += count;
        skipped += skip;
    }

    // Backup home files (individual files)
    if !context.config.home_files.is_empty() {
        info!("Backing up home files...");
        let (count, skip) = backup_home_files(&home, context, dry_run)?;
        backed_up += count;
        skipped += skip;
    }

    // Also backup any tracked files that exist in the repo but might have been updated
    let tracked_home_files = get_tracked_home_files(context)?;
    if !tracked_home_files.is_empty() {
        info!("Checking tracked home files for updates...");
        for file in tracked_home_files {
            // Skip if already in home_files list (already processed)
            let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if context.config.is_home_file_managed(file_name) {
                continue;
            }

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
        repo.add_all()?;

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

fn backup_config_dirs(
    config_dir: &Path,
    context: &mut CommandContext,
    dry_run: bool,
) -> Result<(usize, usize)> {
    let mut backed_up = 0;
    let mut skipped = 0;

    for dir_name in &context.config.config_dirs {
        // Skip comments and empty lines
        if dir_name.starts_with('#') || dir_name.trim().is_empty() {
            continue;
        }

        let full_path = config_dir.join(dir_name);

        if !full_path.exists() {
            debug!("Config directory does not exist: {}", full_path.display());
            continue;
        }

        if !full_path.is_dir() {
            warn!("Not a directory: {}", full_path.display());
            continue;
        }

        // Backup entire directory
        info!("Backing up config directory: {}", dir_name);

        for entry in walkdir::WalkDir::new(&full_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let system_path = entry.path();
            if system_path.is_dir() {
                continue;
            }

            // Skip symlinks
            if system_path.is_symlink() {
                warn_symlink(system_path);
                skipped += 1;
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

            let target = context.repo_path.join(&repo_relative);
            if backup_file(system_path, &target, dry_run)? {
                backed_up += 1;
            } else {
                skipped += 1;
            }
        }
    }

    Ok((backed_up, skipped))
}

fn backup_home_files(
    home_dir: &Path,
    context: &mut CommandContext,
    dry_run: bool,
) -> Result<(usize, usize)> {
    let mut backed_up = 0;
    let mut skipped = 0;

    for file_name in &context.config.home_files {
        // Skip comments and empty lines
        if file_name.starts_with('#') || file_name.trim().is_empty() {
            continue;
        }

        let full_path = home_dir.join(file_name);

        if !full_path.exists() {
            debug!("Home file does not exist: {}", full_path.display());
            continue;
        }

        if full_path.is_dir() {
            warn!("Expected file but found directory: {}", full_path.display());
            continue;
        }

        // Skip symlinks
        if full_path.is_symlink() {
            warn_symlink(&full_path);
            skipped += 1;
            continue;
        }

        // Convert to relative path (home/ filename)
        let repo_relative = PathBuf::from("home").join(file_name);

        // Check if excluded
        if context
            .config
            .is_excluded(&repo_relative.to_string_lossy(), None)
        {
            skipped += 1;
            continue;
        }

        let target = context.repo_path.join(&repo_relative);
        if backup_file(&full_path, &target, dry_run)? {
            backed_up += 1;
        } else {
            skipped += 1;
        }
    }

    Ok((backed_up, skipped))
}

fn warn_symlink(path: &Path) {
    if let Ok(target) = fs::read_link(path) {
        warn!(
            "Skipping symlink: {} -> {}. \
             Ensure the target is backed up in your dotfiles repository, \
             or manually copy the content if needed.",
            path.display(),
            target.display()
        );
    } else {
        warn!(
            "Skipping broken symlink: {}. \
             The target no longer exists.",
            path.display()
        );
    }
}

fn backup_file(source: &Path, target: &Path, dry_run: bool) -> Result<bool> {
    // Ensure parent directory exists (for both new and updated files)
    if let Some(parent) = target.parent() {
        if !parent.exists() && !dry_run {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            debug!("Created directory: {}", parent.display());
        }
    }

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

        // Skip symlinks
        if repo_path.is_symlink() {
            debug!("Skipping symlink: {}", repo_path.display());
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
