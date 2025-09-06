use std::process::{Command, Stdio};
use std::io::Write;
use anyhow::{Result, Context};
use dialoguer::Confirm;

use crate::core::stack_manager::{discover_stacks, format_stacks_for_fzf, parse_fzf_selection};
use crate::core::symlink_manager::SymlinkManager;
use crate::core::settings_merger::SettingsMerger;
use crate::core::mcp_validator::McpValidator;
use crate::utils::claude_md_updater::ClaudeMdUpdater;
use crate::utils::dependency_check::{check_dependencies, check_fzf_available};

pub async fn run() -> Result<()> {
    println!("🔍 Checking dependencies...");
    check_dependencies().context("Dependency check failed")?;
    
    println!("📦 Discovering available stacks...");
    let stacks = discover_stacks().await.context("Failed to discover stacks")?;
    
    if stacks.is_empty() {
        println!("No stacks found in the stacks/ directory.");
        return Ok(());
    }

    println!("🎯 Select stacks to checkout (use Tab for multi-select):");
    let selected_stacks = select_stacks_with_fzf(&stacks).await?;
    
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

    let should_proceed = Confirm::new()
        .with_prompt("Proceed with checkout?")
        .default(true)
        .interact()?;

    if !should_proceed {
        println!("Checkout cancelled.");
        return Ok(());
    }

    // Process each selected stack
    for stack in selected_stack_objects {
        println!("\n🔧 Processing stack: {}", stack.name);
        
        // Create symlinks for .claude files
        let symlink_manager = SymlinkManager::new();
        symlink_manager.create_symlinks_for_stack(stack).await
            .with_context(|| format!("Failed to create symlinks for stack {}", stack.name))?;

        // Merge settings
        let settings_merger = SettingsMerger::new();
        settings_merger.merge_stack_settings(stack).await
            .with_context(|| format!("Failed to merge settings for stack {}", stack.name))?;

        // Update CLAUDE.md
        let md_updater = ClaudeMdUpdater::new();
        md_updater.add_stack_import(&stack.name).await
            .with_context(|| format!("Failed to update CLAUDE.md for stack {}", stack.name))?;

        println!("  ✅ Stack {} checkout complete", stack.name);
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

async fn select_stacks_with_fzf(stacks: &[crate::core::stack_manager::Stack]) -> Result<Vec<String>> {
    check_fzf_available()?;
    
    let formatted_stacks = format_stacks_for_fzf(stacks);
    
    let mut fzf = Command::new("fzf")
        .args(&[
            "--multi",
            "--prompt=Select stacks: ",
            "--preview=echo 'Stack: {}' | cut -d' ' -f1 | xargs -I{} find stacks/{} -name '*.md' | head -5 | xargs cat 2>/dev/null || echo 'No preview available'",
            "--preview-window=right:50%:wrap",
            "--height=80%",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn fzf process")?;

    if let Some(stdin) = fzf.stdin.as_mut() {
        stdin.write_all(formatted_stacks.as_bytes())
            .context("Failed to write to fzf stdin")?;
    }

    let output = fzf.wait_with_output()
        .context("Failed to wait for fzf process")?;

    if !output.status.success() {
        if output.status.code() == Some(130) {
            // User cancelled with Ctrl+C
            return Ok(Vec::new());
        }
        anyhow::bail!("fzf failed with status: {}", output.status);
    }

    let fzf_output = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 output from fzf")?;

    Ok(parse_fzf_selection(&fzf_output))
}