use anyhow::{Result, Context};
use dialoguer::Confirm;
use std::process::Command;
use walkdir::WalkDir;
use std::path::PathBuf;

use crate::core::symlink_manager::SymlinkManager;
use crate::utils::claude_md_updater::ClaudeMdUpdater;

/// Main cleanup command - push stacks, remove symlinks, clean CLAUDE.md
pub async fn run() -> Result<()> {
    println!("Starting stacks cleanup process...");
    
    // Check if we're in a git repository
    let git_status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    if !git_status.status.success() {
        anyhow::bail!("Not in a git repository. Please run this command from a git repository.");
    }

    // Find all worktrees that might contain stacks
    let worktrees = find_project_worktrees().await?;
    
    if worktrees.is_empty() {
        println!("No project worktrees found to clean up.");
        return Ok(());
    }

    println!("Found {} worktree(s) to process:", worktrees.len());
    for worktree in &worktrees {
        println!("  - {}", worktree.display());
    }

    let should_proceed = Confirm::new()
        .with_prompt("Proceed with cleanup? This will push stack changes, remove symlinks, and clean CLAUDE.md")
        .default(false)
        .interact()?;

    if !should_proceed {
        println!("Cleanup cancelled.");
        return Ok(());
    }

    // Process each worktree
    for worktree_path in worktrees {
        cleanup_worktree(&worktree_path).await?;
    }

    println!("Cleanup complete! Worktrees are ready for merging back to main.");
    
    Ok(())
}

/// Find worktrees that belong to this project
async fn find_project_worktrees() -> Result<Vec<PathBuf>> {
    let mut worktrees = Vec::new();
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("project");

    // Look for worktrees in parent directory with pattern: {project-name}-{feature}
    let parent_dir = current_dir.parent().unwrap_or(&current_dir);
    
    for entry in WalkDir::new(parent_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let dir_name = entry.file_name().to_string_lossy();
        
        // Skip the current directory
        if entry.path() == current_dir {
            continue;
        }
        
        // Check if this looks like a project worktree
        if dir_name.starts_with(&format!("{}-", project_name)) && entry.path().join(".git").exists() {
            // Verify it's actually a worktree by checking if it has stacks
            if entry.path().join("stacks").exists() {
                worktrees.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(worktrees)
}

/// Clean up a specific worktree
async fn cleanup_worktree(worktree_path: &PathBuf) -> Result<()> {
    println!("\nProcessing worktree: {}", worktree_path.display());
    
    // Change to worktree directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(worktree_path)?;
    
    // Find all stacks in this worktree
    let stacks_dir = PathBuf::from("stacks");
    if !stacks_dir.exists() {
        println!("  No stacks directory found, skipping");
        std::env::set_current_dir(original_dir)?;
        return Ok(());
    }

    let stack_names = find_stack_names(&stacks_dir)?;
    
    if stack_names.is_empty() {
        println!("  No stacks found, skipping");
        std::env::set_current_dir(original_dir)?;
        return Ok(());
    }

    println!("  Found {} stack(s): {}", stack_names.len(), stack_names.join(", "));

    // Push any changes in stacks back to their repositories
    push_stack_changes(&stack_names).await?;
    
    // Remove symlinks
    remove_stack_symlinks(&stack_names).await?;
    
    // Remove stacks directories 
    remove_stacks_directories(&stack_names).await?;
    
    // Clean CLAUDE.md below demarcation line
    clean_claude_md().await?;
    
    // Return to original directory
    std::env::set_current_dir(original_dir)?;
    
    println!("  ✅ Cleaned up worktree: {}", worktree_path.display());
    
    Ok(())
}

/// Find all stack names in the stacks directory
fn find_stack_names(stacks_dir: &PathBuf) -> Result<Vec<String>> {
    let mut stack_names = Vec::new();
    
    for entry in WalkDir::new(stacks_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let dir_name = entry.file_name().to_string_lossy();
        
        // Skip the stacks directory itself
        if entry.path() == stacks_dir {
            continue;
        }
        
        // Verify this looks like a stack (has .claude directory)
        if entry.path().join(".claude").exists() {
            stack_names.push(dir_name.to_string());
        }
    }
    
    Ok(stack_names)
}

/// Push any uncommitted changes in stacks back to their repositories
async fn push_stack_changes(stack_names: &[String]) -> Result<()> {
    println!("  📤 Pushing stack changes...");
    
    for stack_name in stack_names {
        // Check if there are changes in this stack
        let stack_path = format!("stacks/{}", stack_name);
        let status_output = Command::new("git")
            .args(&["status", "--porcelain", &stack_path])
            .output()
            .context("Failed to check git status for stack")?;

        if !String::from_utf8_lossy(&status_output.stdout).trim().is_empty() {
            println!("    Pushing changes for stack: {}", stack_name);
            
            // Stage and commit stack changes
            Command::new("git")
                .args(&["add", &stack_path])
                .output()
                .context("Failed to stage stack changes")?;
            
            let commit_message = format!("feat({}): update stack from worktree", stack_name);
            Command::new("git")
                .args(&["commit", "-m", &commit_message])
                .output()
                .context("Failed to commit stack changes")?;
            
            // Push using subtree
            let repo_url = get_stack_repo_url(stack_name);
            let push_output = Command::new("git")
                .args([
                    "subtree", "push",
                    "--prefix", &stack_path,
                    &repo_url,
                    "main"
                ])
                .output()
                .context("Failed to push subtree")?;
            
            if !push_output.status.success() {
                let error = String::from_utf8_lossy(&push_output.stderr);
                println!("    Warning: Failed to push {}: {}", stack_name, error);
            } else {
                println!("    ✅ Pushed stack: {}", stack_name);
            }
        }
    }
    
    Ok(())
}

/// Get the repository URL for a stack
fn get_stack_repo_url(stack_name: &str) -> String {
    if stack_name == "ts-lint-stack" {
        "git@github.com:csaben/ts-lint-stack.git".to_string()
    } else {
        // Default pattern - assume separate repo per stack
        format!("git@github.com:csaben/{}.git", stack_name)
    }
}

/// Remove symlinks created for stacks
async fn remove_stack_symlinks(stack_names: &[String]) -> Result<()> {
    println!("  🔗 Removing symlinks...");
    
    let symlink_manager = SymlinkManager::new();
    
    for stack_name in stack_names {
        // Remove symlinks for this stack
        if let Err(e) = symlink_manager.remove_stack_symlinks(stack_name).await {
            println!("    Warning: Failed to remove symlinks for {}: {}", stack_name, e);
        } else {
            println!("    ✅ Removed symlinks for: {}", stack_name);
        }
    }
    
    Ok(())
}

/// Remove stacks directories
async fn remove_stacks_directories(stack_names: &[String]) -> Result<()> {
    println!("  📁 Removing stack directories...");
    
    for stack_name in stack_names {
        let stack_path = PathBuf::from(format!("stacks/{}", stack_name));
        
        if stack_path.exists() {
            if let Err(e) = tokio::fs::remove_dir_all(&stack_path).await {
                println!("    Warning: Failed to remove {}: {}", stack_path.display(), e);
            } else {
                println!("    ✅ Removed directory: {}", stack_path.display());
            }
        }
    }
    
    // Remove stacks directory if it's empty
    let stacks_dir = PathBuf::from("stacks");
    if stacks_dir.exists() {
        if let Ok(entries) = tokio::fs::read_dir(&stacks_dir).await {
            let mut count = 0;
            let mut entries = entries;
            while entries.next_entry().await?.is_some() {
                count += 1;
            }
            
            if count == 0 {
                if let Err(e) = tokio::fs::remove_dir(&stacks_dir).await {
                    println!("    Warning: Failed to remove empty stacks directory: {}", e);
                } else {
                    println!("    ✅ Removed empty stacks directory");
                }
            }
        }
    }
    
    Ok(())
}

/// Clean CLAUDE.md by removing everything below the demarcation line
async fn clean_claude_md() -> Result<()> {
    println!("  📝 Cleaning CLAUDE.md...");
    
    let claude_updater = ClaudeMdUpdater::new();
    claude_updater.cleanup_demarcated_imports().await?;
    
    println!("    ✅ Cleaned CLAUDE.md");
    
    Ok(())
}