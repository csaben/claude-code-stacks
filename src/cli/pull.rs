use std::path::Path;
use std::process::Command;
use anyhow::{Result, Context, bail};
use dialoguer::Confirm;
use is_terminal::IsTerminal;
use walkdir::WalkDir;

use crate::core::remote_stack_manager::StackMetadata;

pub async fn run(stack_name: Option<String>) -> Result<()> {
    match stack_name {
        Some(name) => {
            // Pull specific stack
            pull_single_stack(name).await
        }
        None => {
            // Pull all stacks
            pull_all_stacks().await
        }
    }
}

async fn pull_all_stacks() -> Result<()> {
    println!("🔄 Pulling updates for all stacks...");
    
    let stacks_dir = std::env::current_dir()?.join("stacks");
    
    if !stacks_dir.exists() {
        println!("No stacks directory found. Run 'stacks checkout <stack-name>' to check out a stack.");
        return Ok(());
    }
    
    let mut found_stacks = Vec::new();
    
    // Find all stack directories with metadata
    for entry in WalkDir::new(&stacks_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let stack_name = entry.file_name().to_string_lossy().to_string();
        let stack_path = entry.path().to_path_buf();
        
        // Check if it has metadata (indicating it's a managed stack)
        if stack_path.join(".stack-metadata.json").exists() {
            found_stacks.push(stack_name);
        }
    }
    
    if found_stacks.is_empty() {
        println!("  ℹ️ No managed stacks found to update.");
        return Ok(());
    }
    
    println!("  📝 Found {} managed stack(s):", found_stacks.len());
    for name in &found_stacks {
        println!("    • {}", name);
    }
    
    // Confirm pull all
    let should_proceed = if std::io::stdin().is_terminal() {
        Confirm::new()
            .with_prompt("Pull updates for all these stacks?")
            .default(true)
            .interact()?
    } else {
        println!("Auto-proceeding with pull in non-interactive mode...");
        true
    };
    
    if !should_proceed {
        println!("Pull cancelled.");
        return Ok(());
    }
    
    // Pull each stack
    for stack_name in found_stacks {
        println!("\n{}", "=".repeat(50));
        match pull_single_stack(stack_name.clone()).await {
            Ok(_) => println!("  ✅ Successfully updated {}", stack_name),
            Err(e) => println!("  ❌ Failed to update {}: {}", stack_name, e),
        }
    }
    
    println!("\n🎉 Finished updating all stacks!");
    Ok(())
}

async fn pull_single_stack(stack_name: String) -> Result<()> {
    println!("🔄 Pulling updates for stack: {}", stack_name);
    
    let stack_path = std::env::current_dir()?.join("stacks").join(&stack_name);
    
    // Check if stack directory exists
    if !stack_path.exists() {
        bail!("Stack '{}' not found. Run 'stacks checkout {}' first.", stack_name, stack_name);
    }
    
    // Load stack metadata
    let metadata = load_stack_metadata(&stack_path)?;
    println!("  📋 Source: {}", metadata.source_repo);
    
    // Check for uncommitted changes in the stack directory
    let status_output = Command::new("git")
        .current_dir(&stack_path)
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;
    
    let has_changes = !status_output.stdout.is_empty();
    
    if has_changes {
        println!("  ⚠️ Warning: Stack has uncommitted changes:");
        let status_output = Command::new("git")
            .current_dir(&stack_path)
            .args(["status", "--short"])
            .output()
            .context("Failed to show git status")?;
        
        println!("{}", String::from_utf8_lossy(&status_output.stdout));
        
        let should_continue = if std::io::stdin().is_terminal() {
            Confirm::new()
                .with_prompt("Continue with pull? (commit changes first)")
                .default(false)
                .interact()?
        } else {
            println!("Auto-proceeding with pull in non-interactive mode...");
            true
        };
        
        if !should_continue {
            println!("Pull cancelled.");
            return Ok(());
        }
        
        println!("  💡 Tip: Run 'stacks push {}' to commit and push your changes first", stack_name);
    }
    
    // Pull updates using git subtree
    println!("  📡 Pulling subtree updates from {}...", metadata.source_repo);
    let pull_output = Command::new("git")
        .args([
            "subtree", "pull",
            "--prefix", &format!("stacks/{}", stack_name),
            &metadata.source_repo,
            "main",
            "--squash"
        ])
        .output()
        .context("Failed to pull subtree updates")?;
    
    if !pull_output.status.success() {
        let error = String::from_utf8_lossy(&pull_output.stderr);
        
        // Check if it's already up to date
        if error.contains("Already up to date") || error.contains("up-to-date") {
            println!("  ✅ Subtree is already up to date!");
            return Ok(());
        }
        
        bail!("Failed to pull subtree updates: {}", error);
    }
    
    let output_str = String::from_utf8_lossy(&pull_output.stdout);
    if output_str.contains("Already up to date") {
        println!("  ✅ Subtree is already up to date!");
        return Ok(());
    }
    
    println!("  ✅ Successfully updated stack!");
    
    // Show recent changes
    let log_output = Command::new("git")
        .current_dir(&stack_path)
        .args(["log", "--oneline", "-5", "HEAD~5..HEAD"])
        .output()
        .context("Failed to show recent changes")?;
    
    if log_output.status.success() && !log_output.stdout.is_empty() {
        println!("  📝 Recent changes:");
        for line in String::from_utf8_lossy(&log_output.stdout).lines().take(3) {
            if !line.trim().is_empty() {
                println!("    {}", line);
            }
        }
    }
    
    println!("  🎉 Stack '{}' updated successfully!", stack_name);
    
    Ok(())
}

fn load_stack_metadata(stack_path: &Path) -> Result<StackMetadata> {
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

