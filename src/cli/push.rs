use std::path::PathBuf;
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
        
        // Check if it's a git repository with changes
        if stack_path.join(".git").exists() && has_uncommitted_changes(&stack_path)? {
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

fn has_uncommitted_changes(stack_path: &std::path::PathBuf) -> Result<bool> {
    let status_output = Command::new("git")
        .current_dir(stack_path)
        .args(&["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;
    
    Ok(!status_output.stdout.is_empty())
}

async fn push_single_stack(stack_name: String, message: Option<String>) -> Result<()> {
    println!("🔄 Pushing changes for stack: {}", stack_name);
    
    let stack_path = std::env::current_dir()?.join("stacks").join(&stack_name);
    
    // Check if stack directory exists
    if !stack_path.exists() {
        bail!("Stack '{}' not found. Run 'stacks checkout {}' first.", stack_name, stack_name);
    }
    
    // Check if it's a git repository
    if !stack_path.join(".git").exists() {
        bail!("Stack '{}' is not a git repository. It may have been created manually or with an older version.", stack_name);
    }
    
    // Load stack metadata
    let metadata = load_stack_metadata(&stack_path)?;
    println!("  📋 Source: {}", metadata.source_repo);
    
    // Check for uncommitted changes
    let status_output = Command::new("git")
        .current_dir(&stack_path)
        .args(&["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;
    
    let has_changes = !status_output.stdout.is_empty();
    
    if !has_changes {
        println!("  ℹ️ No changes detected in stack '{}'", stack_name);
        return Ok(());
    }
    
    // Show the changes
    println!("  📝 Changes detected:");
    let status_output = Command::new("git")
        .current_dir(&stack_path)
        .args(&["status", "--short"])
        .output()
        .context("Failed to show git status")?;
    
    println!("{}", String::from_utf8_lossy(&status_output.stdout));
    
    // Get commit message
    let commit_message = if let Some(msg) = message {
        msg
    } else if std::io::stdin().is_terminal() {
        Input::<String>::new()
            .with_prompt("Enter commit message")
            .with_initial_text(&format!("feat: update {} stack", stack_name))
            .interact_text()?
    } else {
        format!("feat: update {} stack", stack_name)
    };
    
    // Confirm the push
    let should_proceed = if std::io::stdin().is_terminal() {
        Confirm::new()
            .with_prompt(&format!("Push changes to {}?", metadata.source_repo))
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
    
    // Stage all changes in the stack repository
    println!("  📋 Staging stack changes...");
    let add_output = Command::new("git")
        .current_dir(&stack_path)
        .args(&["add", "."])
        .output()
        .context("Failed to stage stack changes")?;
    
    if !add_output.status.success() {
        let error = String::from_utf8_lossy(&add_output.stderr);
        bail!("Failed to stage stack changes: {}", error);
    }
    
    // Commit the changes to the stack repository
    println!("  💾 Committing stack changes...");
    let commit_output = Command::new("git")
        .current_dir(&stack_path)
        .args(&["commit", "-m", &commit_message])
        .output()
        .context("Failed to commit stack changes")?;
    
    if !commit_output.status.success() {
        let error = String::from_utf8_lossy(&commit_output.stderr);
        bail!("Failed to commit stack changes: {}", error);
    }
    
    // Push directly to the stack's source repository
    println!("  🚀 Pushing to origin...");
    let push_output = Command::new("git")
        .current_dir(&stack_path)
        .args(&["push", "origin", &metadata.source_branch])
        .output()
        .context("Failed to push to origin")?;
    
    if !push_output.status.success() {
        let error = String::from_utf8_lossy(&push_output.stderr);
        bail!("Failed to push to origin: {}", error);
    }
    
    println!("  ✅ Successfully pushed changes!");
    println!("  📝 Changes pushed directly to {} repository via subtree", metadata.source_repo);
    
    Ok(())
}

fn load_stack_metadata(stack_path: &PathBuf) -> Result<StackMetadata> {
    let metadata_file = stack_path.join(".stack-metadata.json");
    
    if !metadata_file.exists() {
        bail!("Stack metadata not found. This stack may have been created with an older version or manually.");
    }
    
    let metadata_content = std::fs::read_to_string(metadata_file)
        .context("Failed to read stack metadata")?;
    
    let metadata: StackMetadata = serde_json::from_str(&metadata_content)
        .context("Failed to parse stack metadata")?;
    
    Ok(metadata)
}