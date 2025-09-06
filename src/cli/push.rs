use std::path::Path;
use std::process::Command;
use anyhow::{Result, Context, bail};
use dialoguer::{Confirm, Input};
use is_terminal::IsTerminal;
use walkdir::WalkDir;

use crate::core::remote_stack_manager::StackMetadata;

pub async fn run(stack_name: Option<String>, message: Option<String>) -> Result<()> {
    match stack_name {
        Some(name) => {
            // Push specific stack
            push_single_stack(name, message.clone()).await
        }
        None => {
            // Push all stacks with changes
            push_all_stacks(message).await
        }
    }
}

async fn push_all_stacks(message: Option<String>) -> Result<()> {
    println!("🔄 Pushing changes for all stacks with modifications...");
    
    let stacks_dir = std::env::current_dir()?.join("stacks");
    
    if !stacks_dir.exists() {
        println!("No stacks directory found. Run 'stacks checkout <stack-name>' to check out a stack.");
        return Ok(());
    }
    
    let mut stacks_with_changes = Vec::new();
    
    // Find all stack directories with changes
    for entry in WalkDir::new(&stacks_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let stack_name = entry.file_name().to_string_lossy().to_string();
        let stack_path = entry.path().to_path_buf();
        
        // Check if this subtree has changes in the main repository
        if has_subtree_changes(&stack_name)? {
            stacks_with_changes.push(stack_name);
        }
    }
    
    if stacks_with_changes.is_empty() {
        println!("  ✅ No stacks have uncommitted changes.");
        return Ok(());
    }
    
    println!("  📝 Found {} stack(s) with changes:", stacks_with_changes.len());
    for name in &stacks_with_changes {
        println!("    • {}", name);
    }
    
    // Confirm push all
    let should_proceed = if std::io::stdin().is_terminal() {
        Confirm::new()
            .with_prompt("Push changes for all these stacks?")
            .default(true)
            .interact()?
    } else {
        println!("Auto-proceeding with push in non-interactive mode...");
        true
    };
    
    if !should_proceed {
        println!("Push cancelled.");
        return Ok(());
    }
    
    // Push each stack
    for stack_name in stacks_with_changes {
        println!("\n{}", "=".repeat(50));
        match push_single_stack(stack_name.clone(), message.clone()).await {
            Ok(_) => println!("  ✅ Successfully pushed {}", stack_name),
            Err(e) => println!("  ❌ Failed to push {}: {}", stack_name, e),
        }
    }
    
    println!("\n🎉 Finished pushing all stacks!");
    Ok(())
}

fn has_uncommitted_changes(stack_path: &Path) -> Result<bool> {
    let status_output = Command::new("git")
        .current_dir(stack_path)
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;
    
    Ok(!status_output.stdout.is_empty())
}

fn has_subtree_changes(stack_name: &str) -> Result<bool> {
    let status_output = Command::new("git")
        .args(["status", "--porcelain", &format!("stacks/{}", stack_name)])
        .output()
        .context("Failed to check subtree git status")?;
    
    Ok(!status_output.stdout.is_empty())
}

async fn push_single_stack(stack_name: String, message: Option<String>) -> Result<()> {
    println!("🔄 Pushing changes for stack: {}", stack_name);
    
    let stack_path = std::env::current_dir()?.join("stacks").join(&stack_name);
    
    // Check if stack directory exists
    if !stack_path.exists() {
        bail!("Stack '{}' not found. Run 'stacks checkout {}' first.", stack_name, stack_name);
    }
    
    // For subtrees, determine the repository URL based on stack name
    let repo_url = if stack_name == "ts-lint-stack" {
        "git@github.com:csaben/ts-lint-stack.git".to_string()
    } else {
        format!("git@github.com:csaben/{}.git", stack_name)
    };
    println!("  📋 Target: {}", repo_url);
    
    // Check for changes in the subtree
    let has_changes = has_subtree_changes(&stack_name)?;
    
    if !has_changes {
        println!("  ℹ️ No changes detected in stack '{}'", stack_name);
        return Ok(());
    }
    
    // Show the changes in the subtree
    println!("  📝 Changes detected in subtree:");
    let status_output = Command::new("git")
        .args(["status", "--short", &format!("stacks/{}", stack_name)])
        .output()
        .context("Failed to show subtree git status")?;
    
    let output_str = String::from_utf8_lossy(&status_output.stdout);
    // Clean up the output to remove the stacks/stack-name/ prefix for better readability
    for line in output_str.lines() {
        if !line.trim().is_empty() {
            let clean_line = line.replace(&format!("stacks/{}/", stack_name), "");
            println!("    {}", clean_line);
        }
    }
    
    // Get commit message
    let commit_message = if let Some(msg) = message {
        msg
    } else if std::io::stdin().is_terminal() {
        Input::<String>::new()
            .with_prompt("Enter commit message")
            .with_initial_text(format!("feat: update {} stack", stack_name))
            .interact_text()?
    } else {
        format!("feat: update {} stack", stack_name)
    };
    
    // Confirm the push
    let should_proceed = if std::io::stdin().is_terminal() {
        Confirm::new()
            .with_prompt(format!("Push subtree changes to {}?", repo_url))
            .default(true)
            .interact()?
    } else {
        println!("Auto-proceeding with push in non-interactive mode...");
        true
    };
    
    if !should_proceed {
        println!("Push cancelled.");
        return Ok(());
    }
    
    // Stage changes in main repository (subtree changes)
    println!("  📋 Staging subtree changes...");
    let add_output = Command::new("git")
        .args(["add", &format!("stacks/{}", stack_name)])
        .output()
        .context("Failed to stage subtree changes")?;
    
    if !add_output.status.success() {
        let error = String::from_utf8_lossy(&add_output.stderr);
        bail!("Failed to stage subtree changes: {}", error);
    }
    
    // Commit the changes in main repository
    println!("  💾 Committing subtree changes...");
    let commit_output = Command::new("git")
        .args(["commit", "-m", &format!("feat({}): {}", stack_name, commit_message)])
        .output()
        .context("Failed to commit subtree changes")?;
    
    if !commit_output.status.success() {
        let error = String::from_utf8_lossy(&commit_output.stderr);
        bail!("Failed to commit subtree changes: {}", error);
    }
    
    // Push subtree changes back to the stack's repository
    println!("  🚀 Pushing subtree to {}...", repo_url);
    let push_output = Command::new("git")
        .args([
            "subtree", "push",
            "--prefix", &format!("stacks/{}", stack_name),
            &repo_url,
            "main"
        ])
        .output()
        .context("Failed to push subtree")?;
    
    if !push_output.status.success() {
        let error = String::from_utf8_lossy(&push_output.stderr);
        bail!("Failed to push subtree: {}", error);
    }
    
    println!("  ✅ Successfully pushed subtree changes!");
    println!("  📝 Changes pushed to {} via git subtree", repo_url);
    
    Ok(())
}

// Metadata loading no longer needed for subtree-based stacks
