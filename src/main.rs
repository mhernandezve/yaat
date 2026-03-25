use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

mod cli;
mod commands;
mod config;
mod git;
mod platform;

use cli::{Cli, Commands};
use commands::CommandContext;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let _subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    // Find repository
    let repo_path = find_repo()?;

    // Load configuration
    let config_path = platform::yaat_config_path(&repo_path);
    let config = config::YaatConfig::from_file(&config_path)?;

    let mut context = CommandContext::new(config, repo_path);

    match cli.command {
        Commands::Init { path, clone } => {
            commands::init::execute(path, clone, &mut context)?;
        }
        Commands::Add { file, host } => {
            commands::add::execute(file, host, &mut context)?;
        }
        Commands::Sync { host, dry_run } => {
            commands::sync::execute(host, dry_run, &mut context)?;
        }
        Commands::Backup { host, dry_run } => {
            commands::backup::execute(host, dry_run, &mut context)?;
        }
        Commands::Status { verbose } => {
            commands::status::execute(verbose, &mut context)?;
        }
    }

    Ok(())
}

fn find_repo() -> Result<PathBuf> {
    // First, check environment variable
    if let Ok(repo_path) = std::env::var("YAAT_REPO") {
        let path = PathBuf::from(repo_path);
        if git::is_git_repo(&path) {
            return Ok(path);
        }
        info!("YAAT_REPO points to non-git directory, searching elsewhere...");
    }

    // Check default location
    let default = platform::default_repo_path()?;
    if git::is_git_repo(&default) {
        return Ok(default);
    }

    // Search up from current directory
    let mut current = std::env::current_dir()?;
    loop {
        if git::is_git_repo(&current) {
            // Check if this is a YAAT repo (has yaat.yaml)
            let yaat_config = current.join("yaat.yaml");
            if yaat_config.exists() {
                return Ok(current);
            }
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    Err(anyhow::anyhow!(
        "Could not find YAAT repository.\n\
        Run 'yaat init' to create one, or set YAAT_REPO environment variable."
    ))
}
