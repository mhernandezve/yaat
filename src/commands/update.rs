use anyhow::{bail, Result};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::config::YaatConfig;
use crate::git::GitRepo;
use crate::known_configs::{KNOWN_CONFIGS, KNOWN_HOME_FILES};
use crate::platform::{ensure_dir, yaat_config_path};

pub fn execute(repo_path: Option<PathBuf>, ask_unknown: bool, dry_run: bool) -> Result<()> {
    // Resolve repo path
    let repo_path = match repo_path {
        Some(p) => p,
        None => {
            // Try YAAT_REPO env or default
            match std::env::var("YAAT_REPO") {
                Ok(env_path) => PathBuf::from(env_path),
                Err(_) => crate::platform::default_repo_path()?,
            }
        }
    };

    // Expand ~ if present
    let repo_path = if repo_path.starts_with("~") {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        home.join(repo_path.strip_prefix("~").unwrap())
    } else {
        repo_path
    };

    // Check if it's a YAAT repository
    if !is_yaat_repo(&repo_path) {
        bail!(
            "Not a YAAT repository: {}. Run 'yaat init' first.",
            repo_path.display()
        );
    }

    info!("Updating YAAT repository at {}", repo_path.display());

    if dry_run {
        info!("[DRY RUN] No changes will be made");
    }

    // Load existing config
    let config_path = yaat_config_path(&repo_path);
    let mut config = YaatConfig::from_file(&config_path)?;

    // Load ignore list
    let ignore_list = load_yaatignore(&repo_path);

    // Track changes
    let mut added_configs: Vec<String> = Vec::new();
    let mut added_home: Vec<String> = Vec::new();

    // 1. Update known configs (automatic, no prompt)
    let (new_known_configs, new_known_home) = detect_new_known_configs(&config);

    for name in new_known_configs {
        if !dry_run {
            config.config_dirs.push(name.clone());
        }
        added_configs.push(name);
    }

    for name in new_known_home {
        if !dry_run {
            config.home_files.push(name.clone());
        }
        added_home.push(name);
    }

    if !added_configs.is_empty() {
        info!(
            "Detected {} new known config directories",
            added_configs.len()
        );
        for name in &added_configs {
            info!("  + {}", name);
        }
    }

    if !added_home.is_empty() {
        info!("Detected {} new known home files", added_home.len());
        for name in &added_home {
            info!("  + {}", name);
        }
    }

    // 2. If ask_unknown, detect and prompt for unknown configs
    if ask_unknown {
        // Detect unknown config directories
        let unknown_configs = detect_unknown_configs(&config, &ignore_list)?;

        if !unknown_configs.is_empty() {
            println!("\nUnknown config directories in ~/.config/:");
            for (name, path) in unknown_configs {
                let file_count = count_files_direct(&path);
                match prompt_for_config(&name, file_count, "config_dirs")? {
                    PromptResult::Add => {
                        if !dry_run {
                            config.config_dirs.push(name.clone());
                        }
                        added_configs.push(name.clone());
                        println!("  + Added: {}", name);
                    }
                    PromptResult::Ignore => {
                        if !dry_run {
                            add_to_yaatignore(&repo_path, &name)?;
                        }
                        println!("  + Added to .yaatignore: {}", name);
                    }
                    PromptResult::Skip => {
                        println!("  + Skipped: {}", name);
                    }
                }
            }
        }

        // Detect unknown home files
        let unknown_home = detect_unknown_home_files(&config, &ignore_list)?;

        if !unknown_home.is_empty() {
            println!("\nUnknown files in ~/:");
            for (name, path) in unknown_home {
                let file_count = if path.is_file() { 1 } else { 0 };
                match prompt_for_config(&name, file_count, "home_files")? {
                    PromptResult::Add => {
                        if !dry_run {
                            config.home_files.push(name.clone());
                        }
                        added_home.push(name.clone());
                        println!("  + Added: {}", name);
                    }
                    PromptResult::Ignore => {
                        if !dry_run {
                            add_to_yaatignore(&repo_path, &name)?;
                        }
                        println!("  + Added to .yaatignore: {}", name);
                    }
                    PromptResult::Skip => {
                        println!("  + Skipped: {}", name);
                    }
                }
            }
        }
    }

    // Save config if changes were made
    if !dry_run && (!added_configs.is_empty() || !added_home.is_empty()) {
        config.to_file(&config_path)?;
        debug!("Updated yaat.yaml");
    }

    // Ensure directory structure exists
    if !dry_run {
        ensure_dir(&repo_path.join("config"))?;
        ensure_dir(&repo_path.join("home"))?;
        ensure_dir(&repo_path.join("hosts"))?;
    }

    // Commit changes if not dry_run
    if !dry_run && (!added_configs.is_empty() || !added_home.is_empty()) {
        if prompt_yes_no("\nCommit changes? [Y/n]")? {
            let repo = GitRepo::open(&repo_path)?;
            repo.add(&config_path)?;

            let added_items: Vec<String> = added_configs
                .iter()
                .chain(added_home.iter())
                .cloned()
                .collect();

            let commit_msg = if added_items.len() == 1 {
                format!("Update yaat.yaml - add {}", added_items[0])
            } else {
                format!("Update yaat.yaml - add {} configs", added_items.len())
            };

            repo.commit(&commit_msg)?;
            info!("Created commit: {}", commit_msg);
        }
    }

    if dry_run {
        info!(
            "[DRY RUN] Would add {} config dirs and {} home files",
            added_configs.len(),
            added_home.len()
        );
    } else if added_configs.is_empty() && added_home.is_empty() {
        info!("No new configurations to add.");
    } else {
        info!(
            "✓ Updated YAAT repository with {} new configurations",
            added_configs.len() + added_home.len()
        );
    }

    Ok(())
}

// Helper functions

fn is_yaat_repo(path: &Path) -> bool {
    path.join("yaat.yaml").exists() && path.join(".git").exists()
}

fn detect_new_known_configs(config: &YaatConfig) -> (Vec<String>, Vec<String>) {
    let mut new_configs = Vec::new();
    let mut new_home = Vec::new();

    // Detect known configs in ~/.config/ that aren't in config yet
    if let Some(config_dir) = dirs::config_dir() {
        for known in KNOWN_CONFIGS {
            if config_dir.join(known).exists() && !config.config_dirs.contains(&known.to_string()) {
                new_configs.push(known.to_string());
            }
        }
    }

    // Detect known files in ~/ that aren't in config yet
    if let Some(home_dir) = dirs::home_dir() {
        for known in KNOWN_HOME_FILES {
            if home_dir.join(known).exists() && !config.home_files.contains(&known.to_string()) {
                new_home.push(known.to_string());
            }
        }
    }

    (new_configs, new_home)
}

fn detect_unknown_configs(
    config: &YaatConfig,
    ignore_list: &HashSet<String>,
) -> Result<Vec<(String, PathBuf)>> {
    let mut unknown = Vec::new();

    if let Some(config_dir) = dirs::config_dir() {
        for entry in fs::read_dir(&config_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if !entry.file_type()?.is_dir() {
                continue;
            }
            if KNOWN_CONFIGS.contains(&name.as_str()) {
                continue;
            }
            if config.config_dirs.contains(&name) {
                continue;
            }
            if ignore_list.contains(&name) {
                continue;
            }

            unknown.push((name, entry.path()));
        }
    }

    unknown.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(unknown)
}

fn detect_unknown_home_files(
    config: &YaatConfig,
    ignore_list: &HashSet<String>,
) -> Result<Vec<(String, PathBuf)>> {
    let mut unknown = Vec::new();

    if let Some(home_dir) = dirs::home_dir() {
        for entry in fs::read_dir(&home_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if !name.starts_with('.') {
                continue;
            }
            if KNOWN_HOME_FILES.contains(&name.as_str()) {
                continue;
            }
            if config.home_files.contains(&name) {
                continue;
            }
            if ignore_list.contains(&name) {
                continue;
            }

            unknown.push((name, entry.path()));
        }
    }

    unknown.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(unknown)
}

fn count_files_direct(path: &Path) -> usize {
    match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .count(),
        Err(_) => 0,
    }
}

enum PromptResult {
    Add,
    Ignore,
    Skip,
}

fn prompt_for_config(name: &str, file_count: usize, config_type: &str) -> Result<PromptResult> {
    print!(
        "? {} ({} files) - Add to {}? [y/N/i] ",
        name, file_count, config_type
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => Ok(PromptResult::Add),
        "i" | "ignore" => Ok(PromptResult::Ignore),
        _ => Ok(PromptResult::Skip),
    }
}

fn prompt_yes_no(question: &str) -> Result<bool> {
    print!("{} ", question);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();
    Ok(trimmed.is_empty() || trimmed == "y" || trimmed == "yes")
}

fn load_yaatignore(repo_path: &Path) -> HashSet<String> {
    let ignore_path = repo_path.join(".yaatignore");

    if !ignore_path.exists() {
        return HashSet::new();
    }

    fs::read_to_string(&ignore_path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

fn add_to_yaatignore(repo_path: &Path, name: &str) -> Result<()> {
    let ignore_path = repo_path.join(".yaatignore");

    let mut content = if ignore_path.exists() {
        fs::read_to_string(&ignore_path)?
    } else {
        "# YAAT ignore list - configs that won't be prompted in --ask-unknown\n".to_string()
    };

    if !content.lines().any(|l| l.trim() == name) {
        content.push_str(name);
        content.push('\n');
        fs::write(&ignore_path, content)?;
    }

    Ok(())
}
