use std::process::Command;
use std::path::PathBuf;
use anyhow::{Result, Context};
use dialoguer::{Input, Select, Confirm};

use crate::utils::dependency_check::check_dependencies;

#[derive(Debug)]
pub struct WorktreeConfig {
    pub task_name: String,
    pub branch_strategy: BranchStrategy,
    pub location: PathBuf,
    pub tmux_session: String,
}

#[derive(Debug)]
pub enum BranchStrategy {
    NewFromCurrent,
    NewFromMain,
    ExistingBranch(String),
    NewFromRemote(String),
}

pub async fn run() -> Result<()> {
    println!("🔍 Checking dependencies...");
    check_dependencies().context("Dependency check failed")?;
    
    // Check if we're in a git repository
    let git_status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    if !git_status.status.success() {
        anyhow::bail!("Not in a git repository. Please run this command from a git repository.");
    }

    // Get current branch and repo info
    let current_branch = get_current_branch()?;
    let repo_name = get_repo_name()?;
    
    println!("✅ Git repository detected (current branch: {})", current_branch);

    // Interactive configuration
    let config = gather_worktree_config(&current_branch, &repo_name).await?;
    
    // Show configuration summary
    println!("\n📋 Configuration Summary:");
    println!("  Task: {}", config.task_name);
    println!("  Branch Strategy: {:?}", config.branch_strategy);
    println!("  Location: {}", config.location.display());
    println!("  Tmux Session: {}", config.tmux_session);

    let should_proceed = Confirm::new()
        .with_prompt("Proceed with worktree creation?")
        .default(true)
        .interact()?;

    if !should_proceed {
        println!("Worktree creation cancelled.");
        return Ok(());
    }

    // Execute the worktree creation process
    execute_worktree_creation(&config).await?;

    println!("\n🎉 Worktree setup complete!");
    println!("💡 Attach to the tmux session with: tmux attach -t {}", config.tmux_session);

    Ok(())
}

async fn gather_worktree_config(current_branch: &str, repo_name: &str) -> Result<WorktreeConfig> {
    // Get task name
    let task_name: String = Input::new()
        .with_prompt("Task name")
        .interact_text()?;

    // Branch strategy selection
    let branch_strategies = vec![
        format!("Create new branch from current ({})", current_branch),
        "Create new branch from main/master".to_string(),
        "Use existing branch".to_string(),
        "Create new branch from remote".to_string(),
    ];

    let branch_selection = Select::new()
        .with_prompt("Select branch strategy")
        .items(&branch_strategies)
        .default(0)
        .interact()?;

    let branch_strategy = match branch_selection {
        0 => BranchStrategy::NewFromCurrent,
        1 => BranchStrategy::NewFromMain,
        2 => {
            let branch: String = Input::new()
                .with_prompt("Existing branch name")
                .interact_text()?;
            BranchStrategy::ExistingBranch(branch)
        }
        3 => {
            let remote_branch: String = Input::new()
                .with_prompt("Remote branch name (e.g., origin/feature-branch)")
                .interact_text()?;
            BranchStrategy::NewFromRemote(remote_branch)
        }
        _ => unreachable!(),
    };

    // Worktree location suggestions
    let default_location = format!("../{}-{}", repo_name, task_name);
    let location_options = vec![
        format!("{} (recommended)", default_location),
        format!("../worktrees/{}", task_name),
        format!("~/dev/worktrees/{}", task_name),
        "Custom path...".to_string(),
    ];

    let location_selection = Select::new()
        .with_prompt("Select worktree location")
        .items(&location_options)
        .default(0)
        .interact()?;

    let location = match location_selection {
        0 => PathBuf::from(&default_location),
        1 => PathBuf::from(format!("../worktrees/{}", task_name)),
        2 => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
            PathBuf::from(format!("{}/dev/worktrees/{}", home, task_name))
        }
        3 => {
            let custom_path: String = Input::new()
                .with_prompt("Custom worktree path")
                .interact_text()?;
            PathBuf::from(custom_path)
        }
        _ => unreachable!(),
    };

    // Tmux session configuration
    let default_session = format!("{}-{}", repo_name, task_name);
    let tmux_session: String = Input::new()
        .with_prompt("Tmux session name")
        .default(default_session)
        .interact_text()?;

    Ok(WorktreeConfig {
        task_name,
        branch_strategy,
        location,
        tmux_session,
    })
}

async fn execute_worktree_creation(config: &WorktreeConfig) -> Result<()> {
    // Create the branch if needed
    let branch_name = match &config.branch_strategy {
        BranchStrategy::NewFromCurrent => {
            let branch_name = format!("feature-{}", config.task_name);
            println!("🌱 Creating branch {} from current branch...", branch_name);
            
            let output = Command::new("git")
                .args(&["checkout", "-b", &branch_name])
                .output()
                .context("Failed to create new branch")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to create branch: {}", error);
            }

            branch_name
        }
        BranchStrategy::NewFromMain => {
            let branch_name = format!("feature-{}", config.task_name);
            println!("🌱 Creating branch {} from main/master...", branch_name);
            
            // First, fetch latest changes
            Command::new("git")
                .args(&["fetch", "origin"])
                .output()
                .context("Failed to fetch from origin")?;

            // Try main first, then master
            let main_branch = if branch_exists("origin/main") {
                "origin/main"
            } else if branch_exists("origin/master") {
                "origin/master"
            } else {
                anyhow::bail!("Neither origin/main nor origin/master found");
            };

            let output = Command::new("git")
                .args(&["checkout", "-b", &branch_name, main_branch])
                .output()
                .context("Failed to create branch from main")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to create branch: {}", error);
            }

            branch_name
        }
        BranchStrategy::ExistingBranch(branch) => {
            println!("🔄 Using existing branch {}...", branch);
            branch.clone()
        }
        BranchStrategy::NewFromRemote(remote_branch) => {
            let branch_name = format!("feature-{}", config.task_name);
            println!("🌱 Creating branch {} from {}...", branch_name, remote_branch);
            
            let output = Command::new("git")
                .args(&["checkout", "-b", &branch_name, remote_branch])
                .output()
                .context("Failed to create branch from remote")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to create branch: {}", error);
            }

            branch_name
        }
    };

    // Create the worktree
    println!("🏗️ Creating worktree at {}...", config.location.display());
    let output = Command::new("git")
        .args(&["worktree", "add", config.location.to_str().unwrap(), &branch_name])
        .output()
        .context("Failed to create git worktree")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create worktree: {}", error);
    }

    // Set up tmux session
    println!("🖥️ Setting up tmux session {}...", config.tmux_session);
    setup_tmux_session(config).await?;

    Ok(())
}

async fn setup_tmux_session(config: &WorktreeConfig) -> Result<()> {
    // Check if session already exists
    let session_exists = Command::new("tmux")
        .args(&["has-session", "-t", &config.tmux_session])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if session_exists {
        let should_kill = Confirm::new()
            .with_prompt(format!("Tmux session '{}' already exists. Kill and recreate?", config.tmux_session))
            .default(false)
            .interact()?;

        if should_kill {
            Command::new("tmux")
                .args(&["kill-session", "-t", &config.tmux_session])
                .output()
                .context("Failed to kill existing tmux session")?;
        } else {
            println!("Using existing tmux session.");
            return Ok(());
        }
    }

    // Create new tmux session
    let worktree_path = config.location.canonicalize()
        .context("Failed to resolve worktree path")?;

    // Create session with first window in the worktree directory
    Command::new("tmux")
        .args(&[
            "new-session", "-d", "-s", &config.tmux_session,
            "-c", worktree_path.to_str().unwrap()
        ])
        .output()
        .context("Failed to create tmux session")?;

    // Split the window vertically and start Claude Code in the right pane
    Command::new("tmux")
        .args(&[
            "split-window", "-h", "-t", &format!("{}:0", config.tmux_session),
            "-c", worktree_path.to_str().unwrap(),
            "claude"
        ])
        .output()
        .context("Failed to split tmux window and start Claude Code")?;

    // Select the left pane (development pane)
    Command::new("tmux")
        .args(&["select-pane", "-t", &format!("{}:0.0", config.tmux_session)])
        .output()
        .context("Failed to select tmux pane")?;

    println!("  ✅ Tmux session '{}' created with Claude Code in right pane", config.tmux_session);

    Ok(())
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;

    if !output.status.success() {
        anyhow::bail!("Failed to determine current branch");
    }

    let branch = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in branch name")?
        .trim()
        .to_string();

    Ok(branch)
}

fn get_repo_name() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to get repository root")?;

    if !output.status.success() {
        anyhow::bail!("Failed to determine repository root");
    }

    let repo_path_string = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in repository path")?;
    let repo_path = repo_path_string.trim();

    let repo_name = std::path::Path::new(repo_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repo")
        .to_string();

    Ok(repo_name)
}

fn branch_exists(branch_name: &str) -> bool {
    Command::new("git")
        .args(&["rev-parse", "--verify", branch_name])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}