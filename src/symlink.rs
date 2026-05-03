use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum SymlinkStatus {
    /// Symlink exists and points to the expected target
    Correct,
    /// Symlink exists but points to a different target
    Divergent { actual: PathBuf },
    /// Path does not exist
    Missing,
    /// Path exists but is not a symlink
    NotASymlink,
    /// Symlink exists but points to a non-existent target
    Broken,
}

/// Check the status of a system path against an expected symlink target.
///
/// Returns:
/// - `Correct` if `system_path` is a symlink pointing to `expected_target`
/// - `Divergent` if `system_path` is a symlink pointing elsewhere
/// - `Missing` if `system_path` does not exist
/// - `NotASymlink` if `system_path` exists but is not a symlink
/// - `Broken` if `system_path` is a symlink but its target does not exist
pub fn check_symlink_status(system_path: &Path, expected_target: &Path) -> Result<SymlinkStatus> {
    if !system_path.exists() {
        // It could be a broken symlink, which exists() returns false for
        if system_path.is_symlink() {
            return Ok(SymlinkStatus::Broken);
        }
        return Ok(SymlinkStatus::Missing);
    }

    if !system_path.is_symlink() {
        return Ok(SymlinkStatus::NotASymlink);
    }

    let actual_target = std::fs::read_link(system_path)?;

    // Normalize both paths for comparison (resolve relative paths, etc.)
    let expected_canonical = if expected_target.exists() {
        expected_target
            .canonicalize()
            .unwrap_or_else(|_| expected_target.to_path_buf())
    } else {
        expected_target.to_path_buf()
    };

    let actual_canonical = if actual_target.exists() {
        actual_target
            .canonicalize()
            .unwrap_or_else(|_| actual_target.clone())
    } else {
        actual_target.clone()
    };

    if actual_canonical == expected_canonical {
        Ok(SymlinkStatus::Correct)
    } else {
        Ok(SymlinkStatus::Divergent {
            actual: actual_target,
        })
    }
}

/// Check if a path is a symlink pointing to the expected target.
pub fn is_correct_symlink(system_path: &Path, expected_target: &Path) -> Result<bool> {
    Ok(matches!(
        check_symlink_status(system_path, expected_target)?,
        SymlinkStatus::Correct
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_missing() {
        let tmp = TempDir::new().unwrap();
        let system = tmp.path().join("missing");
        let target = tmp.path().join("target");

        assert_eq!(
            check_symlink_status(&system, &target).unwrap(),
            SymlinkStatus::Missing
        );
    }

    #[test]
    fn test_not_a_symlink() {
        let tmp = TempDir::new().unwrap();
        let system = tmp.path().join("file.txt");
        fs::write(&system, "content").unwrap();
        let target = tmp.path().join("target");

        assert_eq!(
            check_symlink_status(&system, &target).unwrap(),
            SymlinkStatus::NotASymlink
        );
    }

    #[test]
    fn test_correct_symlink() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target_dir");
        fs::create_dir(&target).unwrap();
        let system = tmp.path().join("link");
        std::os::unix::fs::symlink(&target, &system).unwrap();

        assert_eq!(
            check_symlink_status(&system, &target).unwrap(),
            SymlinkStatus::Correct
        );
    }

    #[test]
    fn test_divergent_symlink() {
        let tmp = TempDir::new().unwrap();
        let target_a = tmp.path().join("target_a");
        let target_b = tmp.path().join("target_b");
        fs::create_dir(&target_a).unwrap();
        fs::create_dir(&target_b).unwrap();
        let system = tmp.path().join("link");
        std::os::unix::fs::symlink(&target_a, &system).unwrap();

        let result = check_symlink_status(&system, &target_b).unwrap();
        assert!(matches!(result, SymlinkStatus::Divergent { actual } if actual == target_a));
    }

    #[test]
    fn test_broken_symlink() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("nonexistent");
        let system = tmp.path().join("link");
        std::os::unix::fs::symlink(&target, &system).unwrap();

        assert_eq!(
            check_symlink_status(&system, &target).unwrap(),
            SymlinkStatus::Broken
        );
    }
}
