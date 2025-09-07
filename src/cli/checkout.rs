use anyhow::{Result, Context};
use dialoguer::{Confirm, Input};
use skim::prelude::*;
use std::io::Cursor;
use std::process::Command;
use std::path::PathBuf;

use crate::core::stack_manager::Stack;
use crate::core::remote_stack_manager::RemoteStackManager;
use crate::core::symlink_manager::SymlinkManager;
use crate::core::settings_merger::SettingsMerger;
use crate::core::mcp_validator::McpValidator;
use crate::core::permission_generator::PermissionGenerator;
use crate::utils::claude_md_updater::ClaudeMdUpdater;
use crate::utils::dependency_check::check_dependencies;

pub async fn run() -> Result<()> {
    run_worktree_stack_session().await
}

/// Main function implementing the new worktree + tmux + stacks paradigm
async fn run_worktree_stack_session() -> Result<()> {
    println!("Setting up worktree-based stack session...");
    check_dependencies().context("Dependency check failed")?;
    
    // Get current directory name for tmux window naming
    let cwd = std::env::current_dir()?;
    let cwd_stem = cwd.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("project");
    let tmux_window_name = format!("{}-stacks", cwd_stem);
    
    // Check if we're in a git repository
    let git_status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    if !git_status.status.success() {
        anyhow::bail!("Not in a git repository. Please run this command from a git repository.");
    }

    // Create or attach to tmux session
    setup_tmux_window(&tmux_window_name).await?;
    
    // Main loop - keep adding worktrees until user is done
    loop {
        if !create_stack_worktree(&tmux_window_name).await? {
            break;
        }
        
        let add_another = Confirm::new()
            .with_prompt("Add another worktree pane?")
            .default(true)
            .interact()?;
            
        if !add_another {
            break;
        }
    }
    
    println!("\nStack session setup complete!");
    println!("Attach to tmux session: tmux attach -t {}", tmux_window_name);
    println!("Run 'stacks cleanup' when ready to clean up worktrees and merge back");
    
    Ok(())
}

/// Set up tmux window for the stack session
async fn setup_tmux_window(window_name: &str) -> Result<()> {
    // Check if tmux session already exists
    let session_exists = Command::new("tmux")
        .args(&["has-session", "-t", window_name])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !session_exists {
        // Create new tmux session
        println!("Creating tmux session: {}", window_name);
        Command::new("tmux")
            .args(&["new-session", "-d", "-s", window_name])
            .output()
            .context("Failed to create tmux session")?;
    } else {
        println!("Using existing tmux session: {}", window_name);
    }
    
    Ok(())
}

/// Create a single worktree with selected stacks and launch Claude
async fn create_stack_worktree(tmux_session: &str) -> Result<bool> {
    // Get feature/task name from user
    let feature_name: String = Input::new()
        .with_prompt("Feature/task name")
        .interact_text()?;

    if feature_name.trim().is_empty() {
        return Ok(false);
    }

    // Get Claude prompt (optional)
    let claude_prompt: String = Input::new()
        .with_prompt("Claude prompt (or press Enter for default 'claude')")
        .default("claude".to_string())
        .interact_text()?;

    // Select stacks using skim
    let selected_stacks = select_stacks_with_skim().await?;
    
    if selected_stacks.is_empty() {
        // Allow Claude to work without stacks in current directory
        println!("No stacks selected - Claude will work in current directory without stack configuration");
        
        // Create worktree anyway but without stacks
        let worktree_path = create_worktree_for_feature(&feature_name).await?;
        
        // Create new tmux pane and launch Claude with the prompt
        create_tmux_pane_with_claude(tmux_session, &worktree_path, &claude_prompt).await?;
        
        println!("Created worktree '{}' with no stacks (vanilla Claude)", feature_name);
        return Ok(true);
    }

    // Create worktree
    let worktree_path = create_worktree_for_feature(&feature_name).await?;
    
    // Add selected stacks to the worktree
    add_stacks_to_worktree(&worktree_path, &selected_stacks).await?;
    
    // Create new tmux pane and launch Claude with the prompt
    create_tmux_pane_with_claude(tmux_session, &worktree_path, &claude_prompt).await?;
    
    println!("Created worktree '{}' with {} stack(s)", feature_name, selected_stacks.len());
    
    Ok(true)
}

/// Use skim to let user select stacks from remote
async fn select_stacks_with_skim() -> Result<Vec<Stack>> {
    println!("Discovering remote stacks...");
    
    // Discover available stacks from remote
    let remote_manager = RemoteStackManager::new().context("Failed to initialize remote stack manager")?;
    let stacks = remote_manager.discover_remote_stacks().await.context("Failed to discover remote stacks")?;
    
    if stacks.is_empty() {
        anyhow::bail!("No stacks found in remote repository");
    }

    // Prepare items for skim, with option to continue without stacks
    let mut items: Vec<String> = vec!["[NONE] - Continue without any stacks (Claude will work in current directory)".to_string()];
    items.extend(stacks.iter().map(|stack| {
        format!("{} - {}", stack.name, stack.description.as_ref().unwrap_or(&"No description".to_string()))
    }));
    
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .prompt(Some("Select stacks (Tab for multi-select, or choose [NONE] to work without stacks): "))
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(items.join("\n")));

    if let Some(out) = Skim::run_with(&options, Some(items)) {
        if out.is_abort {
            return Ok(vec![]);
        }

        let selected_stacks: Vec<Stack> = out.selected_items
            .iter()
            .filter_map(|item| {
                let item_output = item.output();
                let item_text = item_output.as_ref();
                // Find the stack name (everything before the first " - ")
                let stack_name = item_text.split(" - ").next()?;
                
                // Skip the "[NONE]" option
                if stack_name == "[NONE]" {
                    return None;
                }
                
                stacks.iter().find(|s| s.name == stack_name).cloned()
            })
            .collect();

        Ok(selected_stacks)
    } else {
        Ok(vec![])
    }
}

/// Create git worktree for the feature
async fn create_worktree_for_feature(feature_name: &str) -> Result<PathBuf> {
    let branch_name = format!("feature-{}", feature_name);
    let worktree_path = PathBuf::from(format!("../{}-{}", 
        std::env::current_dir()?.file_stem().unwrap().to_str().unwrap(), 
        feature_name
    ));

    // Create branch and worktree
    println!("Creating worktree at {}", worktree_path.display());
    
    let output = Command::new("git")
        .args(&["worktree", "add", "-b", &branch_name, worktree_path.to_str().unwrap()])
        .output()
        .context("Failed to create git worktree")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create worktree: {}", error);
    }

    // Set up automatic permissions for the feature branch
    setup_feature_permissions(&worktree_path).await?;

    Ok(worktree_path)
}

/// Add selected stacks to the worktree using subtree operations
async fn add_stacks_to_worktree(worktree_path: &PathBuf, stacks: &[Stack]) -> Result<()> {
    // Store the original directory
    let original_dir = std::env::current_dir()?;
    
    // Change to worktree directory
    std::env::set_current_dir(worktree_path)?;
    
    let remote_manager = RemoteStackManager::new().context("Failed to initialize remote manager")?;
    
    for stack in stacks {
        println!("Adding stack: {}", stack.name);
        remote_manager.add_stack_subtree(&stack.name).await?;
        
        // Create a Stack object with the correct worktree-relative path
        let worktree_stack_path = PathBuf::from(format!("stacks/{}", stack.name));
        let worktree_stack = Stack::new(stack.name.clone(), worktree_stack_path);
        
        // Create symlinks and merge settings using the worktree-relative stack
        let symlink_manager = SymlinkManager::new();
        symlink_manager.create_symlinks_for_stack(&worktree_stack).await?;
        
        let settings_merger = SettingsMerger::new();
        settings_merger.merge_stack_settings(&worktree_stack).await?;
        
        // Add stack import to CLAUDE.md with demarcation
        let claude_updater = ClaudeMdUpdater::new();
        claude_updater.add_stack_import_with_demarcation(&stack.name).await?;
    }
    
    // Return to original directory
    std::env::set_current_dir(original_dir)?;
    
    Ok(())
}

/// Create tmux pane and launch Claude with the given prompt
async fn create_tmux_pane_with_claude(session: &str, worktree_path: &PathBuf, prompt: &str) -> Result<()> {
    let worktree_abs_path = worktree_path.canonicalize()?;
    
    // Create new pane in the session
    Command::new("tmux")
        .args(&[
            "split-window", "-t", session,
            "-c", worktree_abs_path.to_str().unwrap()
        ])
        .output()
        .context("Failed to create tmux pane")?;
    
    // Send the Claude command to the new pane
    let claude_cmd = if prompt == "claude" {
        "claude".to_string()
    } else {
        format!("claude \"{}\"", prompt)
    };
    
    Command::new("tmux")
        .args(&[
            "send-keys", "-t", session,
            &claude_cmd, "Enter"
        ])
        .output()
        .context("Failed to send Claude command to tmux pane")?;
    
    Ok(())
}

pub async fn run_with_stack(direct_stack: Option<String>) -> Result<()> {
    use is_terminal::IsTerminal;
    
    println!("🔍 Checking dependencies...");
    check_dependencies().context("Dependency check failed")?;
    
    println!("📦 Discovering available stacks...");
    
    // Discover available stacks from remote (GitHub)
    let remote_manager = RemoteStackManager::new().context("Failed to initialize remote stack manager")?;
    let stacks = remote_manager.discover_remote_stacks().await.context("Failed to discover remote stacks")?;
    
    println!("  🌐 Found {} remote stack(s) from GitHub", stacks.len());
    
    if stacks.is_empty() {
        println!("No stacks found in the stacks/ directory.");
        return Ok(());
    }

    let selected_stacks = if let Some(direct_stack_name) = direct_stack {
        // Direct stack specified - validate it exists
        if stacks.iter().any(|s| s.name == direct_stack_name) {
            println!("🎯 Direct checkout: {}", direct_stack_name);
            vec![direct_stack_name]
        } else {
            println!("❌ Stack '{}' not found. Available stacks:", direct_stack_name);
            for stack in &stacks {
                println!("  • {} - {}", stack.name, stack.description.as_ref().unwrap_or(&"No description".to_string()));
            }
            return Ok(());
        }
    } else {
        println!("🎯 Select stacks to checkout (use Tab for multi-select, or choose [NONE] to work without stacks):");
        let selected_stack_objects = select_stacks_with_skim().await?;
        selected_stack_objects.iter().map(|s| s.name.clone()).collect()
    };
    
    if selected_stacks.is_empty() {
        println!("No stacks selected - Claude will work in the current directory without stack configuration.");
        println!("💡 Claude Code is ready to use in this directory with default settings.");
        return Ok(());
    }

    // Find selected stack objects
    let selected_stack_objects: Vec<_> = stacks
        .iter()
        .filter(|stack| selected_stacks.contains(&stack.name))
        .collect();

    // Show what will be done
    println!("\n📋 Selected stacks:");
    for stack in &selected_stack_objects {
        println!("  • {} - {}", stack.name, stack.description.as_ref().unwrap_or(&"No description".to_string()));
    }

    let should_proceed = if std::io::stdin().is_terminal() {
        Confirm::new()
            .with_prompt("Proceed with checkout?")
            .default(true)
            .interact()?
    } else {
        println!("Auto-proceeding with checkout in non-interactive mode...");
        true
    };

    if !should_proceed {
        println!("Checkout cancelled.");
        return Ok(());
    }

    // Initialize remote manager for downloading  
    let remote_manager = RemoteStackManager::new().context("Failed to initialize remote stack manager for processing")?;

    // Process each selected stack
    for stack in selected_stack_objects {
        println!("\n🔧 Processing stack: {}", stack.name);
        
        // Add stack as subtree if not already present
        let stack_path = stack.path.clone();
        if !stack.path.exists() {
            // Add stack as git subtree
            remote_manager.add_stack_subtree(&stack.name).await
                .with_context(|| format!("Failed to add stack {} as subtree", stack.name))?;
        } else {
            println!("  📁 Stack already present: {}", stack.name);
        }

        // Update stack with the correct path
        let cached_stack = if stack_path != stack.path {
            crate::core::stack_manager::Stack::new(stack.name.clone(), stack_path)
        } else {
            stack.clone()
        };

        // Create symlinks for .claude files
        let symlink_manager = SymlinkManager::new();
        symlink_manager.create_symlinks_for_stack(&cached_stack).await
            .with_context(|| format!("Failed to create symlinks for stack {}", cached_stack.name))?;

        // Merge settings
        let settings_merger = SettingsMerger::new();
        settings_merger.merge_stack_settings(&cached_stack).await
            .with_context(|| format!("Failed to merge settings for stack {}", cached_stack.name))?;

        // Update CLAUDE.md
        let md_updater = ClaudeMdUpdater::new();
        md_updater.add_stack_import(&cached_stack.name).await
            .with_context(|| format!("Failed to update CLAUDE.md for stack {}", cached_stack.name))?;

        println!("  ✅ Stack {} checkout complete", cached_stack.name);
    }

    // Check for missing MCP servers
    println!("\n🔍 Checking MCP server requirements...");
    let mcp_validator = McpValidator::new();
    let missing_servers = mcp_validator.validate_mcp_servers().await
        .context("Failed to validate MCP servers")?;

    if !missing_servers.is_empty() {
        println!("\n⚠️ Missing MCP servers detected:");
        let install_commands = mcp_validator.generate_installation_commands(&missing_servers);
        
        for (server, command) in missing_servers.iter().zip(install_commands.iter()) {
            println!("  • {} ({})", server.name, server.transport);
            println!("    {}", command);
        }
        println!("\nRun the above commands to install missing MCP servers.");
    } else {
        println!("  ✅ All required MCP servers are available");
    }

    println!("\n🎉 All selected stacks have been checked out successfully!");
    println!("💡 You can now use the agents and commands from the selected stacks.");
    
    Ok(())
}

/// Set up automatic permissions that protect the main directory while allowing full access to the feature directory
async fn setup_feature_permissions(worktree_path: &PathBuf) -> Result<()> {
    println!("🛡️ Setting up automatic permissions for feature branch...");
    
    // Get the current working directory (main project directory)
    let current_dir = std::env::current_dir()
        .context("Failed to get current working directory")?;
    
    // Create permission generator
    let permission_generator = PermissionGenerator::new(current_dir.clone(), worktree_path.clone());
    
    // Apply permissions to the feature directory's .claude/settings.local.json
    let feature_settings_path = worktree_path.join(".claude").join("settings.local.json");
    
    // Ensure the .claude directory exists in the feature directory
    let claude_dir = worktree_path.join(".claude");
    if !claude_dir.exists() {
        tokio::fs::create_dir_all(&claude_dir).await
            .with_context(|| format!("Failed to create .claude directory in {}", worktree_path.display()))?;
    }
    
    permission_generator.apply_to_local_settings(&feature_settings_path).await
        .context("Failed to apply feature permissions")?;
    
    println!("  ✅ Permissions configured:");
    println!("    • Full access to: {}", worktree_path.display());
    println!("    • Read-only access to: {}", current_dir.display());
    println!("    • Settings saved to: {}", feature_settings_path.display());
    
    Ok(())
}

