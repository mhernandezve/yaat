use anyhow::{bail, Result};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::config::YaatConfig;
use crate::git::GitRepo;
use crate::known_configs::{KNOWN_CONFIGS, KNOWN_HOME_FILES};
use crate::platform::ensure_dir;

pub fn execute(repo_path: PathBuf, clone_url: Option<String>, ask_unknown: bool) -> Result<()> {
    info!("Initializing YAAT repository at {}", repo_path.display());

    // Check if it's already a YAAT repository
    if is_yaat_repo(&repo_path) {
        if ask_unknown {
            // Ask if user wants to update
            if prompt_yes_no("YAAT repository already exists. Update configuration and detect new configs? [y/N]")? {
                return update_existing_repo(repo_path);
            }
            return Ok(());
        }
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

    // Initialize new repository
    init_new_repo(repo_path, clone_url, ask_unknown)
}

fn init_new_repo(repo_path: PathBuf, clone_url: Option<String>, ask_unknown: bool) -> Result<()> {
    // Auto-detect known configurations
    let (mut detected_configs, mut detected_home_files) = detect_known_configs();

    // If ask_unknown, detect and prompt for unknown configs
    if ask_unknown {
        let ignore_list = load_yaatignore(&repo_path);

        // Detect unknown config directories
        let unknown_configs = detect_unknown_configs(&detected_configs, &ignore_list)?;
        if !unknown_configs.is_empty() {
            println!("\nUnknown config directories in ~/.config/:");
            for (name, path) in unknown_configs {
                let file_count = count_files_direct(&path);
                match prompt_for_config(&name, file_count, "config_dirs")? {
                    PromptResult::Add => {
                        println!("    Added: {}", name);
                        detected_configs.push(name);
                    }
                    PromptResult::Ignore => {
                        add_to_yaatignore(&repo_path, &name)?;
                        println!("    Added to .yaatignore");
                    }
                    PromptResult::Skip => {
                        println!("    Skipped");
                    }
                }
            }
        }

        // Detect unknown home files
        let unknown_home = detect_unknown_home_files(&detected_home_files, &ignore_list)?;
        if !unknown_home.is_empty() {
            println!("\nUnknown files in ~/:");
            for (name, path) in unknown_home {
                let file_count = if path.is_file() { 1 } else { 0 };
                match prompt_for_config(&name, file_count, "home_files")? {
                    PromptResult::Add => {
                        println!("    Added: {}", name);
                        detected_home_files.push(name);
                    }
                    PromptResult::Ignore => {
                        add_to_yaatignore(&repo_path, &name)?;
                        println!("    Added to .yaatignore");
                    }
                    PromptResult::Skip => {
                        println!("    Skipped");
                    }
                }
            }
        }
    }

    // Create config file (yaat.yaml) with detected configs
    let mut config = YaatConfig::default();
    config.config_dirs = detected_configs;
    config.home_files = detected_home_files;
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

    // Ask for commit confirmation if ask_unknown
    let should_commit = if ask_unknown {
        prompt_yes_no("\nCommit changes? [Y/n]")?
    } else {
        true
    };

    if should_commit {
        let commit_msg = if was_cloned {
            "Initialize YAAT repository from remote"
        } else {
            "Initialize YAAT repository"
        };
        repo.commit(commit_msg)?;
        info!("Created commit: {}", commit_msg);
    }

    // Display detected configs
    display_summary(&config, &config_path, &config_dir, &home_dir, &hosts_dir);

    Ok(())
}

fn update_existing_repo(repo_path: PathBuf) -> Result<()> {
    // Load existing config
    let config_path = repo_path.join("yaat.yaml");
    let mut config = YaatConfig::from_file(&config_path)?;

    // Load ignore list
    let ignore_list = load_yaatignore(&repo_path);

    // Detect new unknown configs (not in config_dirs and not in ignore)
    let unknown_configs = detect_unknown_configs(&config.config_dirs, &ignore_list)?;
    let unknown_home = detect_unknown_home_files(&config.home_files, &ignore_list)?;

    let mut added_configs = Vec::new();
    let mut added_home = Vec::new();

    if !unknown_configs.is_empty() {
        println!("\nScanning for new config directories...");
        println!(
            "Found {} new config(s) not in yaat.yaml:",
            unknown_configs.len()
        );

        for (name, path) in unknown_configs {
            let file_count = count_files_direct(&path);
            match prompt_for_config(&name, file_count, "config_dirs")? {
                PromptResult::Add => {
                    println!("    Added: {}", name);
                    config.config_dirs.push(name.clone());
                    added_configs.push(name);
                }
                PromptResult::Ignore => {
                    add_to_yaatignore(&repo_path, &name)?;
                    println!("    Added to .yaatignore");
                }
                PromptResult::Skip => {
                    println!("    Skipped");
                }
            }
        }
    }

    if !unknown_home.is_empty() {
        println!(
            "\nFound {} new file(s) not in yaat.yaml:",
            unknown_home.len()
        );

        for (name, path) in unknown_home {
            let file_count = if path.is_file() { 1 } else { 0 };
            match prompt_for_config(&name, file_count, "home_files")? {
                PromptResult::Add => {
                    println!("    Added: {}", name);
                    config.home_files.push(name.clone());
                    added_home.push(name);
                }
                PromptResult::Ignore => {
                    add_to_yaatignore(&repo_path, &name)?;
                    println!("    Added to .yaatignore");
                }
                PromptResult::Skip => {
                    println!("    Skipped");
                }
            }
        }
    }

    if added_configs.is_empty() && added_home.is_empty() {
        println!("\nNo new configurations to add.");
        return Ok(());
    }

    // Save updated config
    config.to_file(&config_path)?;
    info!("Updated yaat.yaml");

    // Commit if confirmed
    if prompt_yes_no("\nCommit changes? [Y/n]")? {
        let repo = GitRepo::open(&repo_path)?;
        repo.add(&config_path)?;

        let added_items: Vec<String> = added_configs
            .iter()
            .chain(added_home.iter())
            .cloned()
            .collect();
        let commit_msg = format!("Update yaat.yaml - add {}", added_items.join(", "));
        repo.commit(&commit_msg)?;
        info!("Created commit: {}", commit_msg);
    }

    Ok(())
}

/// Check if a directory is already a YAAT repository
fn is_yaat_repo(path: &PathBuf) -> bool {
    path.join("yaat.yaml").exists() && path.join(".git").exists()
}

/// Detect known configuration directories and files
fn detect_known_configs() -> (Vec<String>, Vec<String>) {
    let mut config_dirs = Vec::new();
    let mut home_files = Vec::new();

    // Detect known configs in ~/.config/
    if let Some(config_dir) = dirs::config_dir() {
        for config in KNOWN_CONFIGS {
            if config_dir.join(config).exists() {
                config_dirs.push(config.to_string());
            }
        }
    }

    // Detect known files in ~/
    if let Some(home_dir) = dirs::home_dir() {
        for file in KNOWN_HOME_FILES {
            if home_dir.join(file).exists() {
                home_files.push(file.to_string());
            }
        }
    }

    (config_dirs, home_files)
}

/// Detect unknown config directories (not in known list, not in existing, not in ignore)
fn detect_unknown_configs(
    existing: &[String],
    ignore_list: &HashSet<String>,
) -> Result<Vec<(String, PathBuf)>> {
    let mut unknown = Vec::new();

    if let Some(config_dir) = dirs::config_dir() {
        for entry in fs::read_dir(&config_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip if not a directory
            if !entry.file_type()?.is_dir() {
                continue;
            }

            // Skip if in known configs
            if KNOWN_CONFIGS.contains(&name.as_str()) {
                continue;
            }

            // Skip if already in existing list
            if existing.contains(&name) {
                continue;
            }

            // Skip if in ignore list
            if ignore_list.contains(&name) {
                continue;
            }

            unknown.push((name, entry.path()));
        }
    }

    // Sort alphabetically for consistency
    unknown.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(unknown)
}

/// Detect unknown home files (not in known list, not in existing, not in ignore)
fn detect_unknown_home_files(
    existing: &[String],
    ignore_list: &HashSet<String>,
) -> Result<Vec<(String, PathBuf)>> {
    let mut unknown = Vec::new();

    if let Some(home_dir) = dirs::home_dir() {
        for entry in fs::read_dir(&home_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Only hidden files
            if !name.starts_with('.') {
                continue;
            }

            // Skip if in known home files
            if KNOWN_HOME_FILES.contains(&name.as_str()) {
                continue;
            }

            // Skip if already in existing list
            if existing.contains(&name) {
                continue;
            }

            // Skip if in ignore list
            if ignore_list.contains(&name) {
                continue;
            }

            unknown.push((name, entry.path()));
        }
    }

    // Sort alphabetically for consistency
    unknown.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(unknown)
}

/// Count files directly in a directory (not recursive)
fn count_files_direct(path: &Path) -> usize {
    match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
            .count(),
        Err(_) => 0,
    }
}

/// Prompt result enum
enum PromptResult {
    Add,
    Ignore,
    Skip,
}

/// Prompt user for a config
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

/// Prompt user with yes/no question
fn prompt_yes_no(question: &str) -> Result<bool> {
    print!("{} ", question);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();
    Ok(trimmed == "y" || trimmed == "yes")
}

/// Load .yaatignore file
fn load_yaatignore(repo_path: &Path) -> HashSet<String> {
    let ignore_path = repo_path.join(".yaatignore");

    if !ignore_path.exists() {
        return HashSet::new();
    }

    fs::read_to_string(&ignore_path)
        .unwrap_or_default()
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// Add entry to .yaatignore
fn add_to_yaatignore(repo_path: &Path, name: &str) -> Result<()> {
    let ignore_path = repo_path.join(".yaatignore");

    let mut content = if ignore_path.exists() {
        fs::read_to_string(&ignore_path)?
    } else {
        "# YAAT ignore list - configs that won't be prompted in --ask-unknown\n".to_string()
    };

    // Add entry if not already present
    if !content.lines().any(|line| line.trim() == name) {
        content.push_str(name);
        content.push('\n');
        fs::write(&ignore_path, content)?;
    }

    Ok(())
}

/// Display summary of initialization
fn display_summary(
    config: &YaatConfig,
    config_path: &Path,
    config_dir: &Path,
    home_dir: &Path,
    hosts_dir: &Path,
) {
    let config_count = config.config_dirs.len();
    let home_count = config.home_files.len();

    info!("✓ Successfully initialized YAAT repository");

    if config_count > 0 {
        info!("  Detected {} config directories:", config_count);
        for item in &config.config_dirs {
            if !item.starts_with('#') {
                info!("    - {}", item);
            }
        }
    }

    if home_count > 0 {
        info!("  Detected {} home files:", home_count);
        for item in &config.home_files {
            if !item.starts_with('#') {
                info!("    - {}", item);
            }
        }
    }

    if config_count == 0 && home_count == 0 {
        info!("  No configurations detected.");
    }

    info!(
        "  Edit {} to add/remove configurations",
        config_path.display()
    );

    info!("  Config directory: {}", config_dir.display());
    info!("  Home files: {}", home_dir.display());
    info!("  Host-specific configs: {}", hosts_dir.display());
}
