use std::process::Command;
use anyhow::{Result, Context};

/// Check if all required dependencies are available
pub fn check_dependencies() -> Result<()> {
    let deps = vec![
        ("tmux", "tmux is required for worktree management"),
        ("claude", "claude CLI is required for MCP operations"), 
        ("fzf", "fzf is required for interactive stack selection"),
        ("git", "git is required for worktree operations"),
    ];

    for (cmd, description) in deps {
        check_command_exists(cmd)
            .with_context(|| format!("{}: {}", description, cmd))?;
    }

    Ok(())
}

/// Check if a specific command exists in PATH
pub fn check_command_exists(command: &str) -> Result<()> {
    let output = Command::new("which")
        .arg(command)
        .output()
        .with_context(|| format!("Failed to check for {}", command))?;

    if !output.status.success() {
        anyhow::bail!("{} not found in PATH", command);
    }

    Ok(())
}

/// Check if fzf is available and working
pub fn check_fzf_available() -> Result<()> {
    let output = Command::new("fzf")
        .arg("--version")
        .output()
        .with_context(|| "Failed to execute fzf --version")?;

    if !output.status.success() {
        anyhow::bail!("fzf is not working properly");
    }

    Ok(())
}