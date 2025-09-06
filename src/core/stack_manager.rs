use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub claude_dir: PathBuf,
}

impl Stack {
    pub fn new(name: String, path: PathBuf) -> Self {
        let claude_dir = path.join(".claude");
        Self {
            name,
            path,
            description: None,
            claude_dir,
        }
    }

    /// Check if this stack has a valid .claude directory structure
    pub fn is_valid(&self) -> bool {
        self.claude_dir.exists() && 
        self.claude_dir.is_dir() &&
        (self.has_agents() || self.has_commands() || self.has_settings())
    }

    pub fn has_agents(&self) -> bool {
        self.claude_dir.join("agents").exists()
    }

    pub fn has_commands(&self) -> bool {
        self.claude_dir.join("commands").exists()
    }

    pub fn has_settings(&self) -> bool {
        self.claude_dir.join(".local-settings.json").exists()
    }

    /// Get the CLAUDE.md path if it exists
    pub fn claude_md_path(&self) -> Option<PathBuf> {
        let path = self.path.join("CLAUDE.md");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Read and parse the stack's CLAUDE.md file for description
    pub async fn load_description(&mut self) -> Result<()> {
        if let Some(claude_md) = self.claude_md_path() {
            let content = tokio::fs::read_to_string(&claude_md)
                .await
                .with_context(|| format!("Failed to read {}", claude_md.display()))?;
            
            // Extract first line starting with # Description: if present
            for line in content.lines() {
                if line.starts_with("# Description:") {
                    self.description = Some(line.trim_start_matches("# Description:").trim().to_string());
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Discover all available stacks in the stacks directory
pub async fn discover_stacks() -> Result<Vec<Stack>> {
    let stacks_dir = Path::new("stacks");
    
    if !stacks_dir.exists() {
        anyhow::bail!("No stacks directory found. Create a 'stacks' directory with your stack configurations.");
    }

    let mut stacks = Vec::new();
    
    for entry in WalkDir::new(stacks_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let stack_name = entry.file_name().to_string_lossy().to_string();
        let mut stack = Stack::new(stack_name, entry.path().to_path_buf());
        
        if stack.is_valid() {
            stack.load_description().await?;
            stacks.push(stack);
        }
    }

    if stacks.is_empty() {
        anyhow::bail!("No valid stacks found in the stacks directory. Each stack should have a .claude directory with agents, commands, or settings.");
    }

    stacks.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(stacks)
}

