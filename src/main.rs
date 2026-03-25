use git2::Repository;
use std::path::Path;
use std::os::unix::fs;
use std::process::Command;
use std::io;

fn clone_repo(repo_url: &str, repo_path: &str) -> Result<Repository, git2::Error> {
    Repository::clone(repo_url, repo_path)
}

fn open_repo(repo_path: &str) -> Result<Repository, git2::Error> {
    Repository::open(repo_path)
}

fn validate_path(path: &str) -> bool {
    Path::new(path).exists()
}

fn create_symlink(target: &str, link_path: &str) -> io::Result<()> {
    fs::symlink(target, link_path)
}

fn run_nvim_update() -> io::Result<()> {
    let output = Command::new("nvim")
        .arg("--headless")
        .arg("+Lazy! update")
        .arg("+qa")
        .output()?;

    if output.status.success() {
        println!("nvim update succeeded");
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("nvim update failed");
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_path = "/home/mh0s/.aaaa/";
    let repo_url = "https://github.com/mhernandezve/dotfiles.git";

    if validate_path(repo_path) {
        println!("Path already exists. Opening...");
        open_repo(repo_path)?;
        return Ok(());
    }

    clone_repo(repo_url, repo_path)?;
    println!("Repository cloned successfully!");

    match create_symlink(repo_path, "/home/mh0s/.config/nvim.1") {
        Ok(_) => println!("Symlink created successfully"),
        Err(e) => eprintln!("Failed to create symlink: {}", e),
    }

    run_nvim_update()?;

    Ok(())
}
