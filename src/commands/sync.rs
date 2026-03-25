use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

use crate::commands::CommandContext;
use crate::platform::config_dir;

pub fn execute(host: Option<String>, dry_run: bool, context: &mut CommandContext) -> Result<()> {
    let hostname = host.unwrap_or_else(|| context.config.default_host.clone());
    info!("Syncing configurations for host: {}", hostname);

    if dry_run {
        info!("[DRY RUN] No changes will be made");
    }

    // Get the base config directory in the repo
    let repo_config_dir = context.repo_path.join("config");
    let repo_home_dir = context.repo_path.join("home");
    let repo_hosts_dir = context.repo_path.join("hosts").join(&hostname);

    let mut synced_count = 0;
    let mut skipped_count = 0;

    // Sync base config files
    if repo_config_dir.exists() {
        info!("Syncing config directory...");
        let (synced, skipped) = sync_directory(
            &repo_config_dir,
            &context.config,
            &hostname,
            dry_run,
            &context.repo_path,
        )?;
        synced_count += synced;
        skipped_count += skipped;
    }

    // Sync home files
    if repo_home_dir.exists() {
        info!("Syncing home directory...");
        let (synced, skipped) = sync_directory(
            &repo_home_dir,
            &context.config,
            &hostname,
            dry_run,
            &context.repo_path,
        )?;
        synced_count += synced;
        skipped_count += skipped;
    }

    // Sync host-specific overrides
    if repo_hosts_dir.exists() {
        info!("Syncing host-specific configuration for {}...", hostname);
        let (synced, skipped) = sync_directory(
            &repo_hosts_dir,
            &context.config,
            &hostname,
            dry_run,
            &context.repo_path,
        )?;
        synced_count += synced;
        skipped_count += skipped;
    }

    if dry_run {
        info!(
            "[DRY RUN] Would sync {} files, skip {}",
            synced_count, skipped_count
        );
    } else {
        info!(
            "✓ Successfully synced {} files, skipped {}",
            synced_count, skipped_count
        );
    }

    Ok(())
}

fn sync_directory(
    repo_dir: &Path,
    config: &crate::config::YaatConfig,
    hostname: &str,
    dry_run: bool,
    repo_path: &Path,
) -> Result<(usize, usize)> {
    let mut synced = 0;
    let mut skipped = 0;

    for entry in walkdir::WalkDir::new(repo_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let repo_file_path = entry.path();

        // Skip directories
        if repo_file_path.is_dir() {
            continue;
        }

        // Get relative path from repo root
        let relative_path = repo_file_path
            .strip_prefix(repo_path)
            .context("Failed to get relative path")?;

        // Check if excluded
        if config.is_excluded(&relative_path.to_string_lossy(), Some(hostname)) {
            debug!("Skipping excluded file: {}", relative_path.display());
            skipped += 1;
            continue;
        }

        // Determine target path
        let target_path = if relative_path.starts_with("config/") {
            let config = config_dir()?;
            let stripped = relative_path
                .strip_prefix("config/")
                .context("Invalid config path")?;
            config.join(stripped)
        } else if relative_path.starts_with("home/") {
            let home = dirs::home_dir().context("Could not determine home directory")?;
            let stripped = relative_path
                .strip_prefix("home/")
                .context("Invalid home path")?;
            home.join(stripped)
        } else if relative_path.starts_with(format!("hosts/{}/", hostname)) {
            let config = config_dir()?;
            let stripped = relative_path
                .strip_prefix(format!("hosts/{}/", hostname))
                .context("Invalid host path")?;
            config.join(stripped)
        } else {
            // Fallback: use as-is relative to home
            let home = dirs::home_dir().context("Could not determine home directory")?;
            home.join(relative_path)
        };

        // Ensure parent directory exists
        if let Some(parent) = target_path.parent() {
            if !parent.exists() && !dry_run {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
                debug!("Created directory: {}", parent.display());
            }
        }

        // Check if target exists
        if target_path.exists() {
            // Check if it's already a symlink to the right place
            if let Ok(existing_target) = fs::read_link(&target_path) {
                if existing_target == repo_file_path {
                    debug!("Already synced: {}", target_path.display());
                    continue;
                }
            }

            // Handle existing file
            if config.symlink.backup {
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
                    fs::remove_file(&target_path).with_context(|| {
                        format!("Failed to remove existing file: {}", target_path.display())
                    })?;
                }
            }
        }

        // Create symlink or copy
        if config.symlink.enabled {
            if dry_run {
                info!(
                    "[DRY RUN] Would create symlink: {} -> {}",
                    target_path.display(),
                    repo_file_path.display()
                );
            } else {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(repo_file_path, &target_path).with_context(
                        || {
                            format!(
                                "Failed to create symlink: {} -> {}",
                                target_path.display(),
                                repo_file_path.display()
                            )
                        },
                    )?;
                }
                #[cfg(windows)]
                {
                    if repo_file_path.is_dir() {
                        std::os::windows::fs::symlink_dir(repo_file_path, &target_path)
                            .with_context(|| {
                                format!(
                                    "Failed to create directory symlink: {} -> {}",
                                    target_path.display(),
                                    repo_file_path.display()
                                )
                            })?;
                    } else {
                        std::os::windows::fs::symlink_file(repo_file_path, &target_path)
                            .with_context(|| {
                                format!(
                                    "Failed to create file symlink: {} -> {}",
                                    target_path.display(),
                                    repo_file_path.display()
                                )
                            })?;
                    }
                }
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

    Ok((synced, skipped))
}
