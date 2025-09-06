use std::path::{Path, PathBuf};
use std::fs;
use std::os::unix::fs as unix_fs;
use anyhow::{Result, Context};
use walkdir::WalkDir;

use super::stack_manager::Stack;

pub struct SymlinkManager {
    claude_dir: PathBuf,
}

impl SymlinkManager {
    pub fn new() -> Self {
        Self {
            claude_dir: PathBuf::from(".claude"),
        }
    }

    /// Create symlinks for all relevant files in a stack
    pub async fn create_symlinks_for_stack(&self, stack: &Stack) -> Result<()> {
        // Ensure .claude directory exists
        self.ensure_claude_dir_exists()?;

        // Create symlinks for agents
        if stack.has_agents() {
            self.create_symlinks_for_subdir(stack, "agents").await?;
        }

        // Create symlinks for commands
        if stack.has_commands() {
            self.create_symlinks_for_subdir(stack, "commands").await?;
        }

        Ok(())
    }

    /// Create symlinks for a subdirectory (agents or commands)
    async fn create_symlinks_for_subdir(&self, stack: &Stack, subdir: &str) -> Result<()> {
        let source_dir = stack.claude_dir.join(subdir);
        let target_dir = self.claude_dir.join(subdir);

        if !source_dir.exists() {
            return Ok(());
        }

        // Ensure target directory exists
        fs::create_dir_all(&target_dir)
            .with_context(|| format!("Failed to create directory {}", target_dir.display()))?;

        // Walk through source directory and create symlinks
        for entry in WalkDir::new(&source_dir)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let source_file = entry.path();
            let relative_path = source_file.strip_prefix(&source_dir)?;
            let target_file = target_dir.join(relative_path);

            // Create parent directories if needed
            if let Some(parent) = target_file.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create parent directory for {}", target_file.display()))?;
            }

            self.create_symlink_with_prefix(source_file, &target_file, &stack.name).await?;
        }

        Ok(())
    }

    /// Create a symlink with stack name prefix to avoid conflicts
    async fn create_symlink_with_prefix(&self, source: &Path, target: &Path, stack_name: &str) -> Result<()> {
        // Generate target path with stack prefix
        let filename = target.file_name()
            .and_then(|name| name.to_str())
            .context("Invalid filename")?;
        
        let prefixed_filename = format!("{}_{}", stack_name, filename);
        let prefixed_target = target.with_file_name(prefixed_filename);

        // Check if symlink already exists
        if prefixed_target.exists() {
            if prefixed_target.is_symlink() {
                // Check if it points to the same source
                let existing_target = fs::read_link(&prefixed_target)?;
                let canonical_source = fs::canonicalize(source)?;
                let canonical_existing = fs::canonicalize(&existing_target).unwrap_or(existing_target);
                
                if canonical_source == canonical_existing {
                    // Already correctly linked
                    return Ok(());
                }
                
                // Remove existing symlink
                fs::remove_file(&prefixed_target)
                    .with_context(|| format!("Failed to remove existing symlink {}", prefixed_target.display()))?;
            } else {
                anyhow::bail!("Target file {} already exists and is not a symlink", prefixed_target.display());
            }
        }

        // Create the symlink
        let absolute_source = fs::canonicalize(source)
            .with_context(|| format!("Failed to canonicalize source path {}", source.display()))?;
            
        unix_fs::symlink(&absolute_source, &prefixed_target)
            .with_context(|| format!("Failed to create symlink from {} to {}", 
                absolute_source.display(), prefixed_target.display()))?;

        println!("  📎 Created symlink: {}", prefixed_target.display());
        Ok(())
    }

    /// Ensure the .claude directory exists
    fn ensure_claude_dir_exists(&self) -> Result<()> {
        if !self.claude_dir.exists() {
            fs::create_dir_all(&self.claude_dir)
                .with_context(|| format!("Failed to create .claude directory at {}", self.claude_dir.display()))?;
        }
        Ok(())
    }

    /// Remove symlinks for a specific stack
    pub async fn remove_stack_symlinks(&self, stack_name: &str) -> Result<()> {
        let dirs_to_check = ["agents", "commands"];
        
        for dir in &dirs_to_check {
            let search_dir = self.claude_dir.join(dir);
            if !search_dir.exists() {
                continue;
            }

            // Find and remove symlinks with the stack prefix
            for entry in WalkDir::new(&search_dir)
                .min_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file() && e.path().is_symlink())
            {
                let filename = entry.file_name().to_string_lossy();
                let prefix = format!("{}_", stack_name);
                
                if filename.starts_with(&prefix) {
                    fs::remove_file(entry.path())
                        .with_context(|| format!("Failed to remove symlink {}", entry.path().display()))?;
                    println!("  🗑️ Removed symlink: {}", entry.path().display());
                }
            }
        }

        Ok(())
    }
}