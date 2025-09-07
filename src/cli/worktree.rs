use std::process::Command;
use std::path::PathBuf;
use anyhow::{Result, Context};
use dialoguer::{Input, Select, Confirm};

use crate::utils::dependency_check::check_dependencies;
use crate::config::{load_config, TmuxStrategy, InTmuxBehavior};

#[derive(Debug, Clone)]
pub struct WorktreeConfig {
    pub task_name: String,
    pub branch_strategy: BranchStrategy,
    pub location: PathBuf,
    pub tmux_session: String,
    pub tmux_strategy: TmuxStrategy,
    pub navigation_command: Option<String>,
}

#[derive(Debug, Clone)]
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
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    if !git_status.status.success() {
        anyhow::bail!("Not in a git repository. Please run this command from a git repository.");
    }

    // Get current branch and repo info
    let current_branch = get_current_branch()?;
    let repo_name = get_repo_name()?;
    
    println!("✅ Git repository detected (current branch: {})", current_branch);

    // Load config and interactive configuration
    let app_config = load_config()?;
    let config = gather_worktree_config(&current_branch, &repo_name, &app_config).await?;
    
    // Show configuration summary
    println!("\n📋 Configuration Summary:");
    println!("  Task: {}", config.task_name);
    println!("  Branch Strategy: {:?}", config.branch_strategy);
    println!("  Location: {}", config.location.display());
    println!("  Tmux Session: {}", config.tmux_session);
    println!("  Tmux Strategy: {}", config.tmux_strategy.description());

    let should_proceed = Confirm::new()
        .with_prompt("Proceed with worktree creation?")
        .default(true)
        .interact()?;

    if !should_proceed {
        println!("Worktree creation cancelled.");
        return Ok(());
    }

    // Execute the worktree creation process
    let final_config = execute_worktree_creation(&config, &current_branch).await?;

    println!("\n🎉 Worktree setup complete!");
    
    // Show navigation command or provide options
    if let Some(nav_command) = &final_config.navigation_command {
        println!("💡 Navigation: {}", nav_command);
    } else {
        show_navigation_options(&final_config).await?;
    }

    Ok(())
}

async fn gather_worktree_config(current_branch: &str, repo_name: &str, app_config: &crate::config::StacksConfig) -> Result<WorktreeConfig> {
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

    // Tmux strategy selection (if prompt_for_strategy is enabled)
    let tmux_strategy = if app_config.prompt_for_strategy {
        let strategies = vec![
            TmuxStrategy::SeparateSessions,
            TmuxStrategy::QuadSplit,
            TmuxStrategy::HorizontalSplit,
            TmuxStrategy::MultipleWindows,
        ];
        
        let strategy_descriptions: Vec<String> = strategies.iter()
            .map(|s| s.description().to_string())
            .collect();

        let selection = Select::new()
            .with_prompt("Select tmux layout strategy")
            .items(&strategy_descriptions)
            .default(0)
            .interact()?;

        strategies[selection].clone()
    } else {
        app_config.tmux_strategy.clone()
    };

    Ok(WorktreeConfig {
        task_name,
        branch_strategy,
        location,
        tmux_session,
        tmux_strategy,
        navigation_command: None,
    })
}

async fn execute_worktree_creation(config: &WorktreeConfig, current_branch: &str) -> Result<WorktreeConfig> {
    // Create the branch if needed
    let branch_name = match &config.branch_strategy {
        BranchStrategy::NewFromCurrent => {
            let target_branch_name = format!("feature-{}", config.task_name);
            
            // Check if we're already on the target branch
            if current_branch == target_branch_name {
                println!("📍 Already on target branch {}, using it for worktree...", target_branch_name);
                target_branch_name
            } else {
                println!("🌱 Creating branch {} from current branch...", target_branch_name);
                
                let output = Command::new("git")
                    .args(["checkout", "-b", &target_branch_name])
                    .output()
                    .context("Failed to create new branch")?;

                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    anyhow::bail!("Failed to create branch: {}", error);
                }

                target_branch_name
            }
        }
        BranchStrategy::NewFromMain => {
            let branch_name = format!("feature-{}", config.task_name);
            println!("🌱 Creating branch {} from main/master...", branch_name);
            
            // First, fetch latest changes
            Command::new("git")
                .args(["fetch", "origin"])
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
                .args(["checkout", "-b", &branch_name, main_branch])
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
                .args(["checkout", "-b", &branch_name, remote_branch])
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
        .args(["worktree", "add", config.location.to_str().unwrap(), &branch_name])
        .output()
        .context("Failed to create git worktree")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create worktree: {}", error);
    }

    // Check if user is already in tmux
    let in_tmux = is_in_tmux()?;
    
    // Set up tmux session with selected strategy
    println!("🖥️ Setting up tmux session {} with {} strategy{}...", 
        config.tmux_session, 
        config.tmux_strategy.as_str(),
        if in_tmux { " (already in tmux)" } else { "" }
    );
    
    let nav_command = setup_tmux_session(config, in_tmux).await?;
    
    let mut result_config = config.clone();
    result_config.navigation_command = nav_command;
    
    Ok(result_config)
}

fn is_in_tmux() -> Result<bool> {
    Ok(std::env::var("TMUX").is_ok())
}

fn get_current_tmux_session() -> Result<Option<String>> {
    if !is_in_tmux()? {
        return Ok(None);
    }
    
    let output = Command::new("tmux")
        .args(["display-message", "-p", "#S"])
        .output()
        .context("Failed to get current tmux session")?;
        
    if output.status.success() {
        let session = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in session name")?
            .trim()
            .to_string();
        Ok(Some(session))
    } else {
        Ok(None)
    }
}

async fn setup_tmux_session(config: &WorktreeConfig, in_tmux: bool) -> Result<Option<String>> {
    let worktree_path = config.location.canonicalize()
        .context("Failed to resolve worktree path")?;
    
    if in_tmux {
        // User is already in tmux - handle intelligently
        let current_session = get_current_tmux_session()?.unwrap_or_default();
        let app_config = load_config()?;
        
        match app_config.in_tmux_behavior {
            InTmuxBehavior::NewWindows => {
                // Create new windows in current session
                return setup_in_existing_session(config, &worktree_path, &current_session).await;
            }
            InTmuxBehavior::NewSession => {
                // Create new session as normal
            }
            InTmuxBehavior::Ask => {
                // Ask user what to do
                let options = vec![
                    format!("Create new windows in current session ({})", current_session),
                    "Create new session".to_string(),
                ];
                
                let selection = Select::new()
                    .with_prompt("You're already in tmux. What would you like to do?")
                    .items(&options)
                    .default(0)
                    .interact()?;
                    
                if selection == 0 {
                    return setup_in_existing_session(config, &worktree_path, &current_session).await;
                }
            }
        }
    }

    // Check if target session already exists
    let session_exists = Command::new("tmux")
        .args(["has-session", "-t", &config.tmux_session])
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
                .args(["kill-session", "-t", &config.tmux_session])
                .output()
                .context("Failed to kill existing tmux session")?;
        } else {
            println!("Using existing tmux session.");
            let nav_cmd = if in_tmux {
                format!("tmux switch-client -t {}", config.tmux_session)
            } else {
                format!("tmux attach -t {}", config.tmux_session)
            };
            return Ok(Some(nav_cmd));
        }
    }

    // Set up based on tmux strategy
    let nav_cmd = match config.tmux_strategy {
        TmuxStrategy::SeparateSessions => {
            setup_separate_sessions(config, &worktree_path).await?;
            if in_tmux {
                format!("tmux switch-client -t {}", config.tmux_session)
            } else {
                format!("tmux attach -t {}", config.tmux_session)
            }
        }
        TmuxStrategy::QuadSplit => {
            setup_quad_split(config, &worktree_path).await?;
            if in_tmux {
                format!("tmux switch-client -t {}", config.tmux_session)
            } else {
                format!("tmux attach -t {}", config.tmux_session)
            }
        }
        TmuxStrategy::HorizontalSplit => {
            setup_horizontal_split(config, &worktree_path).await?;
            if in_tmux {
                format!("tmux switch-client -t {}", config.tmux_session)
            } else {
                format!("tmux attach -t {}", config.tmux_session)
            }
        }
        TmuxStrategy::MultipleWindows => {
            setup_multiple_windows(config, &worktree_path).await?;
            if in_tmux {
                format!("tmux switch-client -t {}", config.tmux_session)
            } else {
                format!("tmux attach -t {}", config.tmux_session)
            }
        }
    };

    Ok(Some(nav_cmd))
}

async fn setup_in_existing_session(config: &WorktreeConfig, worktree_path: &PathBuf, current_session: &str) -> Result<Option<String>> {
    // Find next available window number
    let output = Command::new("tmux")
        .args(["list-windows", "-t", current_session, "-F", "#{window_index}"])
        .output()
        .context("Failed to list tmux windows")?;
    
    let existing_windows: Vec<u32> = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in window list")?
        .lines()
        .filter_map(|line| line.parse().ok())
        .collect();
    
    let start_window = existing_windows.iter().max().unwrap_or(&0) + 1;
    
    match config.tmux_strategy {
        TmuxStrategy::SeparateSessions | TmuxStrategy::MultipleWindows => {
            // Create multiple windows (up to 4)
            for i in 0..4 {
                let window_num = start_window + i;
                let window_name = format!("{}-{}", config.task_name, i + 1);
                let target = format!("{}:{}", current_session, window_num);
                let context_msg = format!("Failed to create window {}", window_num);
                
                Command::new("tmux")
                    .args([
                        "new-window", "-t", &target,
                        "-n", &window_name,
                        "-c", worktree_path.to_str().unwrap(),
                        "claude", "--permission-mode", "acceptEdits"
                    ])
                    .output()
                    .context(context_msg)?;
            }
            
            println!("  ✅ Created 4 new windows in current session '{}'", current_session);
            Ok(Some(format!("tmux select-window -t {}:{}", current_session, start_window)))
        }
        TmuxStrategy::QuadSplit => {
            // Create one window with 2x2 split
            let window_num = start_window;
            let window_name = format!("{}-quad", config.task_name);
            
            // Create window with first pane
            Command::new("tmux")
                .args([
                    "new-window", "-t", &format!("{}:{}", current_session, window_num),
                    "-n", &window_name,
                    "-c", worktree_path.to_str().unwrap(),
                    "claude", "--permission-mode", "acceptEdits"
                ])
                .output()
                .context("Failed to create quad split window")?;
                
            let window_target = format!("{}:{}", current_session, window_num);
            
            // Split vertically first (left/right)
            Command::new("tmux")
                .args([
                    "split-window", "-h", "-t", &window_target,
                    "-c", worktree_path.to_str().unwrap(),
                    "claude", "--permission-mode", "acceptEdits"
                ])
                .output()
                .context("Failed to split window vertically")?;

            // Split left pane horizontally (top/bottom)
            Command::new("tmux")
                .args([
                    "split-window", "-v", "-t", &format!("{}.0", window_target),
                    "-c", worktree_path.to_str().unwrap(),
                    "claude", "--permission-mode", "acceptEdits"
                ])
                .output()
                .context("Failed to split left pane horizontally")?;

            // Split right pane horizontally (top/bottom)  
            Command::new("tmux")
                .args([
                    "split-window", "-v", "-t", &format!("{}.1", window_target),
                    "-c", worktree_path.to_str().unwrap(),
                    "claude", "--permission-mode", "acceptEdits"
                ])
                .output()
                .context("Failed to split right pane horizontally")?;
                
            println!("  ✅ Created quad split window in current session '{}'", current_session);
            Ok(Some(format!("tmux select-window -t {}", window_target)))
        }
        TmuxStrategy::HorizontalSplit => {
            // Create one window with 4 horizontal panes
            let window_num = start_window;
            let window_name = format!("{}-horizontal", config.task_name);
            
            // Create window with first pane
            Command::new("tmux")
                .args([
                    "new-window", "-t", &format!("{}:{}", current_session, window_num),
                    "-n", &window_name,
                    "-c", worktree_path.to_str().unwrap(),
                    "claude", "--permission-mode", "acceptEdits"
                ])
                .output()
                .context("Failed to create horizontal split window")?;
                
            let window_target = format!("{}:{}", current_session, window_num);
            
            // Create 3 more horizontal panes (4 total)
            for i in 1..4 {
                let context_msg = format!("Failed to create pane {}", i);
                Command::new("tmux")
                    .args([
                        "split-window", "-v", "-t", &window_target,
                        "-c", worktree_path.to_str().unwrap(),
                        "claude", "--permission-mode", "acceptEdits"
                    ])
                    .output()
                    .context(context_msg)?;
            }
            
            println!("  ✅ Created horizontal split window in current session '{}'", current_session);
            Ok(Some(format!("tmux select-window -t {}", window_target)))
        }
    }
}

async fn show_navigation_options(config: &WorktreeConfig) -> Result<()> {
    // Get list of all sessions and windows
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
        .context("Failed to list tmux sessions")?;
    
    let sessions: Vec<String> = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in session list")?
        .lines()
        .map(|s| s.to_string())
        .collect();
        
    if sessions.is_empty() {
        println!("💡 No tmux sessions available. Start with: tmux attach -t {}", config.tmux_session);
        return Ok(());
    }
    
    // Get all windows for all sessions
    let mut navigation_options = Vec::new();
    
    for session in &sessions {
        let output = Command::new("tmux")
            .args(["list-windows", "-t", session, "-F", "#{session_name}:#{window_index} #{window_name}"])
            .output()
            .context("Failed to list windows")?;
            
        let windows: Vec<String> = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in window list")?
            .lines()
            .map(|s| s.to_string())
            .collect();
            
        navigation_options.extend(windows);
    }
    
    if navigation_options.is_empty() {
        println!("💡 Navigation: tmux attach -t {}", config.tmux_session);
        return Ok(());
    }
    
    // Use skim for fuzzy selection
    use skim::prelude::*;
    use std::io::Cursor;
    
    let options = navigation_options.join("\n");
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(options));
    
    let skim_options = SkimOptionsBuilder::default()
        .height(Some("40%"))
        .prompt(Some("Navigate to tmux window: "))
        .build()
        .unwrap();
        
    let selected_items = Skim::run_with(&skim_options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(Vec::new);
    
    if let Some(item) = selected_items.first() {
        let selected = item.output().to_string();
        let target = selected.split(' ').next().unwrap_or(&selected);
        
        let nav_command = if is_in_tmux()? {
            if target.contains(':') {
                format!("tmux select-window -t {}", target)
            } else {
                format!("tmux switch-client -t {}", target)
            }
        } else {
            if target.contains(':') {
                let session = target.split(':').next().unwrap();
                format!("tmux attach -t {} \\; select-window -t {}", session, target)
            } else {
                format!("tmux attach -t {}", target)
            }
        };
        
        println!("💡 Navigation: {}", nav_command);
        
        // Optionally execute the command
        let should_navigate = Confirm::new()
            .with_prompt("Execute navigation command now?")
            .default(true)
            .interact()?;
            
        if should_navigate {
            let parts: Vec<&str> = nav_command.split(' ').collect();
            if parts.len() >= 2 {
                Command::new(parts[0])
                    .args(&parts[1..])
                    .status()
                    .context("Failed to execute navigation command")?;
            }
        }
    } else {
        println!("💡 Navigation: tmux attach -t {}", config.tmux_session);
    }
    
    Ok(())
}

async fn setup_separate_sessions(config: &WorktreeConfig, worktree_path: &PathBuf) -> Result<()> {
    // Create session with first window in the worktree directory
    Command::new("tmux")
        .args([
            "new-session", "-d", "-s", &config.tmux_session,
            "-c", worktree_path.to_str().unwrap()
        ])
        .output()
        .context("Failed to create tmux session")?;

    // Split the window vertically and start Claude Code in the right pane
    Command::new("tmux")
        .args([
            "split-window", "-h", "-t", &format!("{}:0", config.tmux_session),
            "-c", worktree_path.to_str().unwrap(),
            "claude", "--permission-mode", "acceptEdits"
        ])
        .output()
        .context("Failed to split tmux window and start Claude Code")?;

    // Select the left pane (development pane)
    Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:0.0", config.tmux_session)])
        .output()
        .context("Failed to select tmux pane")?;

    println!("  ✅ Tmux session '{}' created with separate sessions layout", config.tmux_session);
    Ok(())
}

async fn setup_quad_split(config: &WorktreeConfig, worktree_path: &PathBuf) -> Result<()> {
    // Create session with first window in the worktree directory
    Command::new("tmux")
        .args([
            "new-session", "-d", "-s", &config.tmux_session,
            "-c", worktree_path.to_str().unwrap()
        ])
        .output()
        .context("Failed to create tmux session")?;

    // Split vertically first (left/right)
    Command::new("tmux")
        .args([
            "split-window", "-h", "-t", &format!("{}:0", config.tmux_session),
            "-c", worktree_path.to_str().unwrap(),
            "claude", "--permission-mode", "acceptEdits"
        ])
        .output()
        .context("Failed to split window vertically")?;

    // Split left pane horizontally (top/bottom)
    Command::new("tmux")
        .args([
            "split-window", "-v", "-t", &format!("{}:0.0", config.tmux_session),
            "-c", worktree_path.to_str().unwrap(),
            "claude", "--permission-mode", "acceptEdits"
        ])
        .output()
        .context("Failed to split left pane horizontally")?;

    // Split right pane horizontally (top/bottom)
    Command::new("tmux")
        .args([
            "split-window", "-v", "-t", &format!("{}:0.1", config.tmux_session),
            "-c", worktree_path.to_str().unwrap(),
            "claude", "--permission-mode", "acceptEdits"
        ])
        .output()
        .context("Failed to split right pane horizontally")?;

    // Select the first pane (top-left)
    Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:0.0", config.tmux_session)])
        .output()
        .context("Failed to select tmux pane")?;

    println!("  ✅ Tmux session '{}' created with 2x2 quad split layout", config.tmux_session);
    Ok(())
}

async fn setup_horizontal_split(config: &WorktreeConfig, worktree_path: &PathBuf) -> Result<()> {
    // Create session with first window in the worktree directory
    Command::new("tmux")
        .args([
            "new-session", "-d", "-s", &config.tmux_session,
            "-c", worktree_path.to_str().unwrap()
        ])
        .output()
        .context("Failed to create tmux session")?;

    // Create 3 more horizontal panes (4 total)
    for i in 1..4 {
        let target = format!("{}:0", config.tmux_session);
        let context_msg = format!("Failed to create pane {}", i);
        Command::new("tmux")
            .args([
                "split-window", "-v", "-t", &target,
                "-c", worktree_path.to_str().unwrap(),
                "claude", "--permission-mode", "acceptEdits"
            ])
            .output()
            .context(context_msg)?;
    }

    // Select the first pane (top)
    Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:0.0", config.tmux_session)])
        .output()
        .context("Failed to select tmux pane")?;

    println!("  ✅ Tmux session '{}' created with 4 horizontal panes layout", config.tmux_session);
    Ok(())
}

async fn setup_multiple_windows(config: &WorktreeConfig, worktree_path: &PathBuf) -> Result<()> {
    // Create session with first window
    Command::new("tmux")
        .args([
            "new-session", "-d", "-s", &config.tmux_session,
            "-c", worktree_path.to_str().unwrap(),
            "claude", "--permission-mode", "acceptEdits"
        ])
        .output()
        .context("Failed to create tmux session")?;

    // Create 3 more windows (4 total)
    for i in 1..4 {
        let target = format!("{}:{}", config.tmux_session, i);
        let context_msg = format!("Failed to create window {}", i);
        Command::new("tmux")
            .args([
                "new-window", "-t", &target,
                "-c", worktree_path.to_str().unwrap(),
                "claude", "--permission-mode", "acceptEdits"
            ])
            .output()
            .context(context_msg)?;
    }

    // Select the first window
    Command::new("tmux")
        .args(["select-window", "-t", &format!("{}:0", config.tmux_session)])
        .output()
        .context("Failed to select tmux window")?;

    println!("  ✅ Tmux session '{}' created with 4 windows layout", config.tmux_session);
    Ok(())
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
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
        .args(["rev-parse", "--show-toplevel"])
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
        .args(["rev-parse", "--verify", branch_name])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}