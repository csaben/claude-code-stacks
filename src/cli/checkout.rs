use anyhow::{Result, Context};
use dialoguer::Confirm;
use skim::prelude::*;
use std::io::Cursor;

use crate::core::stack_manager::Stack;
use crate::core::remote_stack_manager::RemoteStackManager;
use crate::core::symlink_manager::SymlinkManager;
use crate::core::settings_merger::SettingsMerger;
use crate::core::mcp_validator::McpValidator;
use crate::utils::claude_md_updater::ClaudeMdUpdater;
use crate::utils::dependency_check::check_dependencies;

pub async fn run() -> Result<()> {
    run_with_stack(None).await
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
        println!("🎯 Select stacks to checkout (use Tab for multi-select):");
        select_stacks_with_skim(&stacks).await?
    };
    
    if selected_stacks.is_empty() {
        println!("No stacks selected.");
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

async fn select_stacks_with_skim(stacks: &[Stack]) -> Result<Vec<String>> {
    use is_terminal::IsTerminal;
    
    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() || std::env::var("TERM").is_err() {
        println!("Non-interactive environment detected. Available stacks:");
        for (i, stack) in stacks.iter().enumerate() {
            println!("  {}. {} - {}", 
                i + 1, 
                stack.name, 
                stack.description.as_ref().unwrap_or(&"No description".to_string())
            );
        }
        
        // For non-interactive, return all stacks
        println!("Selecting all available stacks for non-interactive mode.");
        return Ok(stacks.iter().map(|s| s.name.clone()).collect());
    }
    
    // Format stacks for display
    let formatted_items: Vec<String> = stacks
        .iter()
        .map(|stack| {
            if let Some(ref desc) = stack.description {
                format!("{} - {}", stack.name, desc)
            } else {
                stack.name.clone()
            }
        })
        .collect();
    
    let input = formatted_items.join("\n");
    
    let options = SkimOptionsBuilder::default()
        .height(Some("80%"))
        .multi(true)
        .prompt(Some("Select stacks: "))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build skim options: {}", e))?;

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));
    
    let selected_items = Skim::run_with(&options, Some(items))
        .ok_or_else(|| anyhow::anyhow!("Skim selection was cancelled"))?;
    
    if selected_items.is_abort {
        return Ok(Vec::new());
    }
    
    let selected_stacks: Vec<String> = selected_items
        .selected_items
        .iter()
        .map(|item| {
            let text = item.text();
            // Extract stack name (everything before first " - " if present)
            if let Some(pos) = text.find(" - ") {
                text[..pos].trim().to_string()
            } else {
                text.trim().to_string()
            }
        })
        .collect();
    
    Ok(selected_stacks)
}