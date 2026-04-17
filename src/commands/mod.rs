pub mod add;
pub mod backup;
pub mod init;
pub mod status;
pub mod sync;
pub mod update;

use crate::config::YaatConfig;

/// Common context shared between commands
pub struct CommandContext {
    pub config: YaatConfig,
    pub repo_path: std::path::PathBuf,
}

impl CommandContext {
    pub fn new(config: YaatConfig, repo_path: std::path::PathBuf) -> Self {
        Self { config, repo_path }
    }
}
