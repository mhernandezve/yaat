use anyhow::Result;
use std::path::Path;
use tracing::info;

use crate::commands::CommandContext;
use crate::git::GitRepo;
use crate::platform::config_dir;

pub fn execute(verbose: bool, context: &mut CommandContext) -> Result<()> {
    info!("YAAT Status");
    info!("============");
    info!("");

    // Repository info
    info!("Repository: {}", context.repo_path.display());
    info!("Default host: {}", context.config.default_host);
    info!("");

    // Git status
    info!("Git Status:");
    let repo = GitRepo::open(&context.repo_path)?;
    let statuses = repo.status()?;

    if statuses.is_empty() {
        info!("  ✓ Working directory clean");
    } else {
        info!("  {} modified/untracked files:", statuses.len());
        for (path, status) in &statuses {
            let status_str = format_status(*status);
            info!("    [{}] {}", status_str, path);
        }
    }
    info!("");

    // Configuration summary
    info!("Configuration:");
    info!("  Repository path: {}", context.config.repo_path);
    info!(
        "  Symlinks: {}",
        if context.config.symlink.enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    info!(
        "  Backup existing: {}",
        if context.config.symlink.backup {
            "yes"
        } else {
            "no"
        }
    );
    if !context.config.exclude.is_empty() {
        info!("  Global excludes: {}", context.config.exclude.len());
        if verbose {
            for pattern in &context.config.exclude {
                info!("    - {}", pattern);
            }
        }
    }
    info!("");

    // Host-specific configurations
    if !context.config.hosts.is_empty() {
        info!("Configured hosts:");
        for (host, config) in &context.config.hosts {
            info!("  {}:", host);
            if !config.files.is_empty() {
                info!("    Files: {}", config.files.len());
            }
            if !config.exclude.is_empty() {
                info!("    Excludes: {}", config.exclude.len());
            }
            if !config.env.is_empty() {
                info!("    Environment variables: {}", config.env.len());
            }
        }
        info!("");
    }

    // Tracked files summary
    if verbose {
        info!("Tracked files:");
        let tracked = get_tracked_files(&context.repo_path)?;
        if tracked.is_empty() {
            info!("  No files tracked yet");
        } else {
            info!("  {} files tracked:", tracked.len());
            for file in &tracked {
                info!("    {}", file);
            }
        }
        info!("");

        // Check sync status
        info!("Sync Status:");
        check_sync_status(context)?;
    }

    Ok(())
}

fn format_status(status: git2::Status) -> &'static str {
    use git2::Status;

    if status.contains(Status::WT_NEW) {
        "??"
    } else if status.contains(Status::WT_MODIFIED) {
        " M"
    } else if status.contains(Status::WT_DELETED) {
        " D"
    } else if status.contains(Status::INDEX_NEW) {
        "A "
    } else if status.contains(Status::INDEX_MODIFIED) {
        "M "
    } else if status.contains(Status::INDEX_DELETED) {
        "D "
    } else {
        "??"
    }
}

fn get_tracked_files(repo_path: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    let config_dir = repo_path.join("config");
    let home_dir = repo_path.join("home");
    let hosts_dir = repo_path.join("hosts");

    // Config files
    if config_dir.exists() {
        for entry in walkdir::WalkDir::new(&config_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Ok(relative) = path.strip_prefix(repo_path) {
                    files.push(relative.to_string_lossy().to_string());
                }
            }
        }
    }

    // Home files
    if home_dir.exists() {
        for entry in walkdir::WalkDir::new(&home_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Ok(relative) = path.strip_prefix(repo_path) {
                    files.push(relative.to_string_lossy().to_string());
                }
            }
        }
    }

    // Host-specific files
    if hosts_dir.exists() {
        for entry in walkdir::WalkDir::new(&hosts_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Ok(relative) = path.strip_prefix(repo_path) {
                    files.push(relative.to_string_lossy().to_string());
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

fn check_sync_status(context: &CommandContext) -> Result<()> {
    let repo_config_dir = context.repo_path.join("config");
    let system_config = config_dir()?;

    if !repo_config_dir.exists() {
        info!("  No config files in repository");
        return Ok(());
    }

    let mut synced = 0;
    let mut pending = 0;
    let mut diverged = 0;

    for entry in walkdir::WalkDir::new(&repo_config_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let repo_file = entry.path();
        if repo_file.is_dir() {
            continue;
        }

        // Get relative path
        let relative = match repo_file.strip_prefix(&repo_config_dir) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let system_file = system_config.join(relative);

        if !system_file.exists() {
            pending += 1;
            if context.config.symlink.enabled {
                info!("  [PENDING] Symlink needed: {}", relative.display());
            } else {
                info!("  [PENDING] Copy needed: {}", relative.display());
            }
        } else if system_file.is_symlink() {
            // Check if symlink points to the right place
            match std::fs::read_link(&system_file) {
                Ok(target) if target == repo_file => {
                    synced += 1;
                }
                _ => {
                    diverged += 1;
                    info!("  [DIVERGED] Symlink mismatch: {}", relative.display());
                }
            }
        } else {
            // File exists but is not a symlink - check if content matches
            match files_equal(&repo_file, &system_file) {
                Ok(true) => {
                    synced += 1;
                }
                Ok(false) => {
                    diverged += 1;
                    info!("  [DIVERGED] Content differs: {}", relative.display());
                }
                Err(_) => {
                    diverged += 1;
                    info!("  [DIVERGED] Cannot compare: {}", relative.display());
                }
            }
        }
    }

    info!("");
    info!(
        "  Summary: {} synced, {} pending, {} diverged",
        synced, pending, diverged
    );

    Ok(())
}

fn files_equal(a: &Path, b: &Path) -> Result<bool> {
    let a_content = std::fs::read(a)?;
    let b_content = std::fs::read(b)?;
    Ok(a_content == b_content)
}
