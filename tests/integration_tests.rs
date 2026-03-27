use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_yaat_bin() -> PathBuf {
    // First, try to find the binary in the expected locations
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // Try multiple possible locations
    let possible_paths = [
        PathBuf::from(&manifest_dir)
            .join("target")
            .join("debug")
            .join("yaat"),
        PathBuf::from(&manifest_dir)
            .join("target")
            .join("release")
            .join("yaat"),
        // For CI environment, check CARGO_BIN_EXE_YAAT
        std::env::var("CARGO_BIN_EXE_YAAT")
            .map(PathBuf::from)
            .unwrap_or_default(),
    ];

    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }

    // Fallback: return the debug path even if it doesn't exist
    // This will give a clearer error message
    possible_paths[0].clone()
}

fn run_yaat(args: &[&str]) -> (bool, String, String) {
    let bin = get_yaat_bin();
    let output = Command::new(&bin)
        .args(args)
        .output()
        .expect("Failed to execute yaat");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    (success, stdout, stderr)
}

#[test]
fn test_yaat_help() {
    let (success, stdout, _) = run_yaat(&["--help"]);
    assert!(success);
    assert!(stdout.contains("Yet Another Dotfiles Manager"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("add"));
    assert!(stdout.contains("sync"));
    assert!(stdout.contains("backup"));
    assert!(stdout.contains("status"));
}

#[test]
fn test_yaat_version() {
    let (success, stdout, _) = run_yaat(&["--version"]);
    assert!(success);
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_init_creates_repo() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("dotfiles");

    let (success, stdout, stderr) = run_yaat(&["init", repo_path.to_str().unwrap()]);

    assert!(success, "Failed with stderr: {}", stderr);
    assert!(stdout.contains("Successfully initialized YAAT repository"));

    // Check directory structure
    assert!(repo_path.exists());
    assert!(repo_path.join("config").exists());
    assert!(repo_path.join("home").exists());
    assert!(repo_path.join("hosts").exists());
    assert!(repo_path.join("yaat.yaml").exists());
    assert!(repo_path.join(".git").exists());
    assert!(repo_path.join(".gitignore").exists());
}

#[test]
fn test_add_copies_file() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("dotfiles");
    let config_dir = temp_dir.path().join(".config");
    fs::create_dir_all(&config_dir).unwrap();

    // Create a test config file
    let test_file = config_dir.join("test.conf");
    fs::write(&test_file, "test content").unwrap();

    // Initialize repo
    run_yaat(&["init", repo_path.to_str().unwrap()]);

    // Change to repo directory and add file
    // Need to set HOME so YAAT recognizes the config path
    let bin = get_yaat_bin();
    let output = Command::new(&bin)
        .args(&["add", test_file.to_str().unwrap()])
        .current_dir(&repo_path)
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", config_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute yaat add");

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        success,
        "Failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Successfully added"));

    // Check file was copied
    let copied_file = repo_path.join("config").join("test.conf");
    assert!(copied_file.exists());
    assert_eq!(fs::read_to_string(&copied_file).unwrap(), "test content");
}

#[test]
fn test_status_shows_info() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("dotfiles");

    // Initialize repo
    run_yaat(&["init", repo_path.to_str().unwrap()]);

    // Run status
    let bin = get_yaat_bin();
    let output = Command::new(&bin)
        .args(&["status"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to execute yaat status");

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(success);
    assert!(stdout.contains("YAAT Status"));
    assert!(stdout.contains("Repository:"));
    assert!(stdout.contains("Configuration:"));
    assert!(stdout.contains("Symlinks:"));
}

#[test]
fn test_sync_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("dotfiles");
    let config_dir = temp_dir.path().join(".config");
    fs::create_dir_all(&config_dir).unwrap();

    // Initialize repo
    run_yaat(&["init", repo_path.to_str().unwrap()]);

    // Create a file in the repo
    let repo_config_dir = repo_path.join("config");
    fs::create_dir_all(&repo_config_dir).unwrap();
    fs::write(repo_config_dir.join("test.conf"), "repo content").unwrap();

    // Run sync with dry-run
    let bin = get_yaat_bin();
    let output = Command::new(&bin)
        .args(&["sync", "--dry-run"])
        .current_dir(&repo_path)
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", config_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute yaat sync");

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(success);
    assert!(stdout.contains("[DRY RUN]"));
}

#[test]
fn test_backup_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("dotfiles");
    let config_dir = temp_dir.path().join(".config");
    fs::create_dir_all(&config_dir).unwrap();

    // Create a config file
    fs::write(config_dir.join("test.conf"), "test content").unwrap();

    // Initialize repo
    run_yaat(&["init", repo_path.to_str().unwrap()]);

    // Run backup with dry-run
    let bin = get_yaat_bin();
    let output = Command::new(&bin)
        .args(&["backup", "--dry-run"])
        .current_dir(&repo_path)
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", config_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute yaat backup");

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(success);
    assert!(stdout.contains("[DRY RUN]"));
}

#[test]
fn test_init_with_clone() {
    // This test would require a git remote, skip for now
    // In real CI, we'd mock this or use a local bare repo
}
