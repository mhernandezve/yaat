use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

use crate::commands::CommandContext;
use crate::platform::config_dir;

pub fn execute(host: Option<String>, dry_run: bool, context: &mut CommandContext) -> Result<()> {
    let hostname = host.unwrap_or_else(|| context.config.default_host.clone());
    info!("Syncing configurations for host: {}", hostname);

    if dry_run {
        info!("[DRY RUN] No changes will be made");
    }

    let mut synced_count = 0;
    let mut skipped_count = 0;
    let mut already_synced_count = 0;

    let effective_config_dirs = context.config.effective_config_dirs();
    let effective_home_files = context.config.effective_home_files();

    // Sync config directories (as directory symlinks)
    if !effective_config_dirs.is_empty() {
        info!("Syncing config directories...");
        let (synced, skipped, already_synced) =
            sync_config_dirs(context, &effective_config_dirs, dry_run)?;
        synced_count += synced;
        skipped_count += skipped;
        already_synced_count += already_synced;
    }

    // Sync home files (as individual file symlinks)
    if !effective_home_files.is_empty() {
        info!("Syncing home files...");
        let (synced, skipped, already_synced) =
            sync_home_files(context, &effective_home_files, dry_run)?;
        synced_count += synced;
        skipped_count += skipped;
        already_synced_count += already_synced;
    }

    // Sync host-specific overrides (file by file)
    let repo_hosts_dir = context.repo_path.join("hosts").join(&hostname);
    if repo_hosts_dir.exists() {
        info!("Syncing host-specific configuration for {}...", hostname);
        let (synced, skipped, already_synced) =
            sync_host_overrides(&repo_hosts_dir, context, dry_run)?;
        synced_count += synced;
        skipped_count += skipped;
        already_synced_count += already_synced;
    }

    if dry_run {
        info!(
            "[DRY RUN] Summary: {} already synced, {} would sync, {} would skip",
            already_synced_count, synced_count, skipped_count
        );
    } else {
        info!(
            "✓ Summary: {} already synced, {} newly synced, {} skipped",
            already_synced_count, synced_count, skipped_count
        );
    }

    Ok(())
}

fn sync_config_dirs(
    context: &CommandContext,
    config_dirs: &[String],
    dry_run: bool,
) -> Result<(usize, usize, usize)> {
    let mut synced = 0;
    let mut skipped = 0;
    let mut already_synced = 0;
    let config_dir = config_dir()?;

    for dir_name in config_dirs {
        // Skip comments
        if dir_name.starts_with('#') || dir_name.trim().is_empty() {
            continue;
        }

        let repo_dir = context.repo_path.join("config").join(dir_name);
        let system_dir = config_dir.join(dir_name);

        // Check if repo directory exists
        if !repo_dir.exists() {
            warn!("Repo directory does not exist: {}", repo_dir.display());
            skipped += 1;
            continue;
        }

        // Check symlink status using shared module
        match crate::symlink::check_symlink_status(&system_dir, &repo_dir)? {
            crate::symlink::SymlinkStatus::Correct => {
                info!(
                    "  Already synced: {} -> {}",
                    system_dir.display(),
                    repo_dir.display()
                );
                already_synced += 1;
                continue;
            }
            crate::symlink::SymlinkStatus::Divergent { actual } => {
                warn!(
                    "Diverged symlink detected: {} -> {} (expected: {})",
                    system_dir.display(),
                    actual.display(),
                    repo_dir.display()
                );
            }
            crate::symlink::SymlinkStatus::Broken => {
                warn!(
                    "Broken symlink detected: {} (expected: {})",
                    system_dir.display(),
                    repo_dir.display()
                );
            }
            crate::symlink::SymlinkStatus::Missing | crate::symlink::SymlinkStatus::NotASymlink => {
                // Will be created or replaced below
            }
        }

        // Handle existing directory/file
        if system_dir.exists() || system_dir.is_symlink() {
            if context.config.symlink.backup {
                let backup_path = format!("{}.backup", system_dir.display());
                if !dry_run {
                    fs::rename(&system_dir, &backup_path)
                        .with_context(|| format!("Failed to backup {}", system_dir.display()))?;
                    info!(
                        "Backed up existing: {} -> {}",
                        system_dir.display(),
                        backup_path
                    );
                } else {
                    info!(
                        "[DRY RUN] Would backup: {} -> {}",
                        system_dir.display(),
                        backup_path
                    );
                }
            } else {
                if !dry_run {
                    if system_dir.is_dir() {
                        fs::remove_dir_all(&system_dir).with_context(|| {
                            format!("Failed to remove {}", system_dir.display())
                        })?;
                    } else {
                        fs::remove_file(&system_dir).with_context(|| {
                            format!("Failed to remove {}", system_dir.display())
                        })?;
                    }
                }
            }
        }

        // Create directory symlink
        if context.config.symlink.enabled {
            if dry_run {
                info!(
                    "[DRY RUN] Would create directory symlink: {} -> {}",
                    system_dir.display(),
                    repo_dir.display()
                );
            } else {
                std::os::unix::fs::symlink(&repo_dir, &system_dir).with_context(|| {
                    format!(
                        "Failed to create directory symlink: {} -> {}",
                        system_dir.display(),
                        repo_dir.display()
                    )
                })?;
                info!(
                    "Created directory symlink: {} -> {}",
                    system_dir.display(),
                    repo_dir.display()
                );
            }
        } else {
            // Copy entire directory
            if dry_run {
                info!(
                    "[DRY RUN] Would copy directory: {} -> {}",
                    repo_dir.display(),
                    system_dir.display()
                );
            } else {
                copy_dir_all(&repo_dir, &system_dir)?;
                info!(
                    "Copied directory: {} -> {}",
                    repo_dir.display(),
                    system_dir.display()
                );
            }
        }

        synced += 1;
    }

    Ok((synced, skipped, already_synced))
}

fn sync_home_files(
    context: &CommandContext,
    home_files: &[String],
    dry_run: bool,
) -> Result<(usize, usize, usize)> {
    let mut synced = 0;
    let mut skipped = 0;
    let mut already_synced = 0;
    let home_dir = dirs::home_dir().context("Could not determine home directory")?;

    for file_name in home_files {
        // Skip comments
        if file_name.starts_with('#') || file_name.trim().is_empty() {
            continue;
        }

        let repo_file = context.repo_path.join("home").join(file_name);
        let system_file = home_dir.join(file_name);

        // Check if repo file exists
        if !repo_file.exists() {
            warn!("Repo file does not exist: {}", repo_file.display());
            skipped += 1;
            continue;
        }

        // Ensure parent directory exists
        if let Some(parent) = system_file.parent() {
            if !parent.exists() && !dry_run {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        // Check symlink status using shared module
        match crate::symlink::check_symlink_status(&system_file, &repo_file)? {
            crate::symlink::SymlinkStatus::Correct => {
                info!(
                    "  Already synced: {} -> {}",
                    system_file.display(),
                    repo_file.display()
                );
                already_synced += 1;
                continue;
            }
            crate::symlink::SymlinkStatus::Divergent { actual } => {
                warn!(
                    "Diverged symlink detected: {} -> {} (expected: {})",
                    system_file.display(),
                    actual.display(),
                    repo_file.display()
                );
            }
            crate::symlink::SymlinkStatus::Broken => {
                warn!(
                    "Broken symlink detected: {} (expected: {})",
                    system_file.display(),
                    repo_file.display()
                );
            }
            crate::symlink::SymlinkStatus::Missing | crate::symlink::SymlinkStatus::NotASymlink => {
                // Will be created or replaced below
            }
        }

        // Handle existing file
        if system_file.exists() || system_file.is_symlink() {
            if context.config.symlink.backup {
                let backup_path = format!("{}.backup", system_file.display());
                if !dry_run {
                    fs::rename(&system_file, &backup_path)
                        .with_context(|| format!("Failed to backup {}", system_file.display()))?;
                    info!(
                        "Backed up existing file: {} -> {}",
                        system_file.display(),
                        backup_path
                    );
                } else {
                    info!(
                        "[DRY RUN] Would backup: {} -> {}",
                        system_file.display(),
                        backup_path
                    );
                }
            } else {
                if !dry_run {
                    fs::remove_file(&system_file)
                        .with_context(|| format!("Failed to remove {}", system_file.display()))?;
                }
            }
        }

        // Create file symlink
        if context.config.symlink.enabled {
            if dry_run {
                info!(
                    "[DRY RUN] Would create symlink: {} -> {}",
                    system_file.display(),
                    repo_file.display()
                );
            } else {
                std::os::unix::fs::symlink(&repo_file, &system_file).with_context(|| {
                    format!(
                        "Failed to create symlink: {} -> {}",
                        system_file.display(),
                        repo_file.display()
                    )
                })?;
                info!(
                    "Created symlink: {} -> {}",
                    system_file.display(),
                    repo_file.display()
                );
            }
        } else {
            // Copy file
            if dry_run {
                info!(
                    "[DRY RUN] Would copy: {} -> {}",
                    repo_file.display(),
                    system_file.display()
                );
            } else {
                fs::copy(&repo_file, &system_file).with_context(|| {
                    format!(
                        "Failed to copy: {} -> {}",
                        repo_file.display(),
                        system_file.display()
                    )
                })?;
                info!(
                    "Copied: {} -> {}",
                    repo_file.display(),
                    system_file.display()
                );
            }
        }

        synced += 1;
    }

    Ok((synced, skipped, already_synced))
}

fn sync_host_overrides(
    repo_hosts_dir: &Path,
    context: &CommandContext,
    dry_run: bool,
) -> Result<(usize, usize, usize)> {
    let mut synced = 0;
    let mut skipped = 0;
    let mut already_synced = 0;
    let config_dir = config_dir()?;

    for entry in walkdir::WalkDir::new(repo_hosts_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let repo_file_path = entry.path();

        // Skip directories
        if repo_file_path.is_dir() {
            continue;
        }

        // Get relative path from hosts/<hostname>/
        let relative_path = repo_file_path
            .strip_prefix(repo_hosts_dir)
            .context("Failed to get relative path")?;

        // Check if excluded
        if context.config.is_excluded(
            &relative_path.to_string_lossy(),
            Some(&context.config.default_host),
        ) {
            warn!("Skipping excluded file: {}", relative_path.display());
            skipped += 1;
            continue;
        }

        // Target is in ~/.config/
        let target_path = config_dir.join(relative_path);

        // Ensure parent directory exists
        if let Some(parent) = target_path.parent() {
            if !parent.exists() && !dry_run {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        // Check symlink status using shared module
        match crate::symlink::check_symlink_status(&target_path, &repo_file_path)? {
            crate::symlink::SymlinkStatus::Correct => {
                info!(
                    "  Already synced: {} -> {}",
                    target_path.display(),
                    repo_file_path.display()
                );
                already_synced += 1;
                continue;
            }
            crate::symlink::SymlinkStatus::Divergent { actual } => {
                warn!(
                    "Diverged symlink detected: {} -> {} (expected: {})",
                    target_path.display(),
                    actual.display(),
                    repo_file_path.display()
                );
            }
            crate::symlink::SymlinkStatus::Broken => {
                warn!(
                    "Broken symlink detected: {} (expected: {})",
                    target_path.display(),
                    repo_file_path.display()
                );
            }
            crate::symlink::SymlinkStatus::Missing | crate::symlink::SymlinkStatus::NotASymlink => {
                // Will be created or replaced below
            }
        }

        // Handle existing file
        if target_path.exists() || target_path.is_symlink() {
            if context.config.symlink.backup {
                let backup_path = format!("{}.backup", target_path.display());
                if !dry_run {
                    fs::rename(&target_path, &backup_path)
                        .with_context(|| format!("Failed to backup {}", target_path.display()))?;
                    info!(
                        "Backed up existing file: {} -> {}",
                        target_path.display(),
                        backup_path
                    );
                } else {
                    info!(
                        "[DRY RUN] Would backup: {} -> {}",
                        target_path.display(),
                        backup_path
                    );
                }
            } else {
                if !dry_run {
                    fs::remove_file(&target_path)
                        .with_context(|| format!("Failed to remove {}", target_path.display()))?;
                }
            }
        }

        // Create symlink or copy
        if context.config.symlink.enabled {
            if dry_run {
                info!(
                    "[DRY RUN] Would create symlink: {} -> {}",
                    target_path.display(),
                    repo_file_path.display()
                );
            } else {
                std::os::unix::fs::symlink(repo_file_path, &target_path).with_context(|| {
                    format!(
                        "Failed to create symlink: {} -> {}",
                        target_path.display(),
                        repo_file_path.display()
                    )
                })?;
                info!(
                    "Created symlink: {} -> {}",
                    target_path.display(),
                    repo_file_path.display()
                );
            }
        } else {
            if dry_run {
                info!(
                    "[DRY RUN] Would copy: {} -> {}",
                    repo_file_path.display(),
                    target_path.display()
                );
            } else {
                fs::copy(repo_file_path, &target_path).with_context(|| {
                    format!(
                        "Failed to copy: {} -> {}",
                        repo_file_path.display(),
                        target_path.display()
                    )
                })?;
                info!(
                    "Copied: {} -> {}",
                    repo_file_path.display(),
                    target_path.display()
                );
            }
        }

        synced += 1;
    }

    Ok((synced, skipped, already_synced))
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.as_ref().join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
