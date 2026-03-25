use anyhow::{Context, Result};
use git2::{Repository, Signature, StatusOptions};
use std::path::PathBuf;
use tracing::{debug, info, warn};

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    /// Initialize a new git repository
    pub fn init(path: &PathBuf) -> Result<Self> {
        let repo = Repository::init(path).with_context(|| {
            format!("Failed to initialize git repository at {}", path.display())
        })?;

        info!("Initialized git repository at {}", path.display());

        // Create initial .gitignore
        let gitignore_path = path.join(".gitignore");
        if !gitignore_path.exists() {
            let gitignore_content = r#"# YAAT - Yet Another Dotfiles Manager
# This file is managed by YAAT

# OS files
.DS_Store
Thumbs.db

# Editor files
*.swp
*.swo
*~
.idea/
.vscode/

# Temporary files
*.tmp
*.bak
"#;
            std::fs::write(&gitignore_path, gitignore_content)
                .context("Failed to create .gitignore")?;
            debug!("Created .gitignore at {}", gitignore_path.display());
        }

        Ok(Self { repo })
    }

    /// Open an existing git repository
    pub fn open(path: &PathBuf) -> Result<Self> {
        let repo = Repository::open(path)
            .with_context(|| format!("Failed to open git repository at {}", path.display()))?;

        Ok(Self { repo })
    }

    /// Clone a remote repository
    pub fn clone(url: &str, path: &PathBuf) -> Result<Self> {
        info!("Cloning repository from {} to {}", url, path.display());

        let repo = Repository::clone(url, path)
            .with_context(|| format!("Failed to clone repository from {}", url))?;

        info!("Successfully cloned repository");
        Ok(Self { repo })
    }

    /// Get the repository path
    pub fn path(&self) -> PathBuf {
        self.repo
            .workdir()
            .unwrap_or_else(|| self.repo.path())
            .into()
    }

    /// Add a file to the git index
    pub fn add(&self, path: &PathBuf) -> Result<()> {
        let mut index = self
            .repo
            .index()
            .context("Failed to get repository index")?;

        let repo_path = self.path();
        let relative_path = path.strip_prefix(&repo_path).with_context(|| {
            format!(
                "Path {} is not within repository {}",
                path.display(),
                repo_path.display()
            )
        })?;

        index
            .add_path(relative_path)
            .with_context(|| format!("Failed to add {} to index", path.display()))?;

        index.write().context("Failed to write index")?;

        debug!("Added {} to git index", relative_path.display());
        Ok(())
    }

    /// Commit changes with a message
    pub fn commit(&self, message: &str) -> Result<()> {
        let mut index = self
            .repo
            .index()
            .context("Failed to get repository index")?;

        let oid = index.write_tree().context("Failed to write tree")?;
        let tree = self.repo.find_tree(oid).context("Failed to find tree")?;

        let signature =
            Signature::now("YAAT", "yaat@localhost").context("Failed to create signature")?;

        let parent_commits: Vec<_> = match self.repo.head() {
            Ok(head) => {
                let commit = head.peel_to_commit().context("Failed to peel to commit")?;
                vec![commit]
            }
            Err(_) => vec![], // No commits yet
        };

        let parents: Vec<_> = parent_commits.iter().collect();

        self.repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &parents,
            )
            .context("Failed to create commit")?;

        info!("Created commit: {}", message);
        Ok(())
    }

    /// Get repository status
    pub fn status(&self) -> Result<Vec<(String, git2::Status)>> {
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);

        let statuses = self
            .repo
            .statuses(Some(&mut status_opts))
            .context("Failed to get repository status")?;

        let mut result = Vec::new();
        for status in statuses.iter() {
            if let Some(path) = status.path() {
                result.push((path.to_string(), status.status()));
            }
        }

        Ok(result)
    }

    /// Check if the working directory is clean
    pub fn is_clean(&self) -> Result<bool> {
        let statuses = self.status()?;
        Ok(statuses.is_empty())
    }

    /// Pull latest changes from remote
    pub fn pull(&self) -> Result<()> {
        // This is a simplified pull - in production you'd want proper merge handling
        warn!("Pull operation not fully implemented");
        Ok(())
    }

    /// Push changes to remote
    pub fn push(&self) -> Result<()> {
        warn!("Push operation not fully implemented");
        Ok(())
    }
}

/// Check if a directory is a git repository
pub fn is_git_repo(path: &PathBuf) -> bool {
    Repository::open(path).is_ok()
}
