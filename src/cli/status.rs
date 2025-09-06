use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};
use walkdir::WalkDir;

use crate::core::remote_stack_manager::StackMetadata;

pub async fn run() -> Result<()> {
    println!("📊 Stack Status Report");
    println!("═══════════════════════");
    
    let stacks_dir = std::env::current_dir()?.join("stacks");
    
    if !stacks_dir.exists() {
        println!("No stacks directory found. Run 'stacks checkout <stack-name>' to check out a stack.");
        return Ok(());
    }
    
    let mut found_stacks = false;
    
    // Find all stack directories
    for entry in WalkDir::new(&stacks_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        found_stacks = true;
        let stack_path = entry.path().to_path_buf();
        let stack_name = entry.file_name().to_string_lossy().to_string();
        
        println!("\n📦 Stack: {}", stack_name);
        
        // For subtrees, check if this is a valid stack directory
        println!("  📂 Type: Subtree (part of main repository)");
        
        // Check for subtree changes in main repository
        match check_subtree_status(&stack_name) {
            Ok(status_info) => {
                if status_info.has_changes {
                    println!("  📝 Status: {} changes in subtree", status_info.changes_count);
                    if !status_info.changes.is_empty() {
                        for change in status_info.changes.iter().take(5) {
                            // Remove the stacks/stack-name/ prefix for cleaner display
                            let clean_change = change.replace(&format!("stacks/{}/", stack_name), "");
                            println!("    {}", clean_change);
                        }
                        if status_info.changes.len() > 5 {
                            println!("    ... and {} more", status_info.changes.len() - 5);
                        }
                    }
                } else {
                    println!("  ✅ Status: Clean (no changes in subtree)");
                }
            }
            Err(e) => {
                println!("  ❌ Status: Failed to get subtree status: {}", e);
            }
        }
        
        // Show last commit info for the subtree
        if let Ok(commit_info) = get_subtree_last_commit(&stack_name) {
            println!("  🕒 Last subtree change: {}", commit_info);
        }
    }
    
    if !found_stacks {
        println!("No stacks found in the stacks directory.");
        println!("Run 'stacks checkout <stack-name>' to check out a stack.");
    }
    
    Ok(())
}

struct GitStatusInfo {
    has_changes: bool,
    changes_count: usize,
    changes: Vec<String>,
}

fn load_stack_metadata(stack_path: &Path) -> Result<StackMetadata> {
    let metadata_file = stack_path.join(".stack-metadata.json");
    
    let metadata_content = std::fs::read_to_string(metadata_file)
        .context("Failed to read stack metadata")?;
    
    let metadata: StackMetadata = serde_json::from_str(&metadata_content)
        .context("Failed to parse stack metadata")?;
    
    Ok(metadata)
}

fn get_current_branch(stack_path: &PathBuf) -> Result<String> {
    let output = Command::new("git")
        .current_dir(stack_path)
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

fn check_git_status(stack_path: &PathBuf) -> Result<GitStatusInfo> {
    let output = Command::new("git")
        .current_dir(stack_path)
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;
    
    let status_lines = String::from_utf8_lossy(&output.stdout);
    let changes: Vec<String> = status_lines
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect();
    
    Ok(GitStatusInfo {
        has_changes: !changes.is_empty(),
        changes_count: changes.len(),
        changes,
    })
}

fn get_remote_status(stack_path: &PathBuf) -> Result<String> {
    // Fetch from origin first (quietly)
    let _fetch_output = Command::new("git")
        .current_dir(stack_path)
        .args(["fetch", "origin", "--quiet"])
        .output();
    
    // Check if ahead/behind
    let output = Command::new("git")
        .current_dir(stack_path)
        .args(["status", "-b", "--porcelain"])
        .output()
        .context("Failed to check remote status")?;
    
    if output.status.success() {
        let status_output = String::from_utf8_lossy(&output.stdout);
        for line in status_output.lines() {
            if line.starts_with("##") {
                if line.contains("[ahead") || line.contains("[behind") {
                    // Extract the ahead/behind information
                    if let Some(bracket_start) = line.find('[') {
                        if let Some(bracket_end) = line.find(']') {
                            return Ok(line[bracket_start..=bracket_end].to_string());
                        }
                    }
                }
                break;
            }
        }
    }
    
    Ok(String::new())
}

fn get_last_commit_info(stack_path: &PathBuf) -> Result<String> {
    let output = Command::new("git")
        .current_dir(stack_path)
        .args(["log", "-1", "--format=%h - %s (%cr)"])
        .output()
        .context("Failed to get last commit info")?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Ok("No commits found".to_string())
    }
}

fn check_subtree_status(stack_name: &str) -> Result<GitStatusInfo> {
    let output = Command::new("git")
        .args(["status", "--porcelain", &format!("stacks/{}", stack_name)])
        .output()
        .context("Failed to check subtree git status")?;
    
    let status_lines = String::from_utf8_lossy(&output.stdout);
    let changes: Vec<String> = status_lines
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect();
    
    Ok(GitStatusInfo {
        has_changes: !changes.is_empty(),
        changes_count: changes.len(),
        changes,
    })
}

fn get_subtree_last_commit(stack_name: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%h - %s (%cr)", "--", &format!("stacks/{}", stack_name)])
        .output()
        .context("Failed to get last commit info for subtree")?;
    
    if output.status.success() {
        let commit_info = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if commit_info.is_empty() {
            Ok("No commits found for subtree".to_string())
        } else {
            Ok(commit_info)
        }
    } else {
        Ok("No commits found for subtree".to_string())
    }
}