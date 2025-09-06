use std::process::Command;
use anyhow::{Result, Context};

/// Check if all required dependencies are available
pub fn check_dependencies() -> Result<()> {
    let deps = vec![
        ("tmux", "tmux is required for worktree management"),
        ("claude", "claude CLI is required for MCP operations"), 
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

