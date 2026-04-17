use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "yaat")]
#[command(about = "Yet Another Assets Tool")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new dotfiles repository
    Init {
        /// Path to initialize the repository
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Clone from existing remote repository
        #[arg(short, long, value_name = "URL")]
        clone: Option<String>,
    },

    /// Update existing dotfiles repository (detect new configs)
    Update {
        /// Specific repository path to update
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Ask about unknown config directories and home files
        #[arg(short = 'a', long = "ask-unknown")]
        ask_unknown: bool,

        /// Dry run - show what would be done without making changes
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Add a file to the dotfiles repository
    Add {
        /// File path to add
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Host-specific configuration
        #[arg(short = 'H', long, value_name = "HOST")]
        host: Option<String>,
    },

    /// Sync configurations from repository to system
    Sync {
        /// Specific host configuration to sync
        #[arg(short = 'H', long, value_name = "HOST")]
        host: Option<String>,

        /// Dry run - show what would be done without making changes
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Backup current system configurations to repository
    Backup {
        /// Specific host to backup for
        #[arg(short = 'H', long, value_name = "HOST")]
        host: Option<String>,

        /// Dry run - show what would be done without making changes
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Show repository status
    Status {
        /// Show detailed status
        #[arg(short, long)]
        verbose: bool,
    },
}
