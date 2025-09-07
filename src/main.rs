use clap::{Parser, Subcommand};
use anyhow::Result;

mod cli;
mod core;
mod utils;
mod config;

use cli::{checkout, push, status, pull, worktree, sync, cleanup};
use config::{StacksConfig, TmuxStrategy, InTmuxBehavior};

#[derive(Parser)]
#[command(name = "stacks")]
#[command(about = "A CLI tool for managing Claude Code workflow stacks")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check out one or more stacks for use in the current project
    #[command(name = "checkout")]
    Checkout {
        /// GitHub URL or stack name to checkout directly
        #[arg(value_name = "STACK_URL_OR_NAME")]
        stack: Option<String>,
    },
    /// Push changes in stacks back to source repositories
    #[command(name = "push")]
    Push {
        /// Stack name to push changes for (optional - pushes all if not specified)
        #[arg(value_name = "STACK_NAME")]
        stack_name: Option<String>,
        /// Commit message for the changes
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Show git status of all checked-out stacks
    #[command(name = "status")]
    Status,
    /// Update stacks from source repositories
    #[command(name = "pull")]
    Pull {
        /// Stack name to update (optional - updates all if not specified)
        #[arg(value_name = "STACK_NAME")]
        stack_name: Option<String>,
    },
    /// Manage git worktrees with tmux integration
    Worktree,
    /// Sync MCP server configurations from docker-compose and other sources
    Sync,
    /// Clean up worktrees by pushing stacks, removing symlinks, and cleaning CLAUDE.md
    Cleanup,
    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set configuration value
    Set {
        /// Setting key (tmux-strategy, prompt-strategy, in-tmux-behavior)
        key: String,
        /// Setting value
        value: String,
    },
    /// Interactive configuration editor
    Edit,
    /// Reset configuration to defaults
    Reset,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Checkout { stack }) => {
            checkout::run_with_stack(stack).await
        }
        Some(Commands::Push { stack_name, message }) => {
            push::run(stack_name, message).await
        }
        Some(Commands::Status) => {
            status::run().await
        }
        Some(Commands::Pull { stack_name }) => {
            pull::run(stack_name).await
        }
        Some(Commands::Worktree) => worktree::run().await,
        Some(Commands::Sync) => sync::run().await,
        Some(Commands::Cleanup) => cleanup::run().await,
        Some(Commands::Config { command }) => handle_config_command(command).await,
        None => {
            // Default behavior - run checkout command
            checkout::run().await
        }
    }
}

async fn handle_config_command(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            show_config().await?;
        }
        ConfigCommands::Edit => {
            interactive_config_editor().await?;
        }
        ConfigCommands::Set { key, value } => {
            match key.as_str() {
                "tmux-strategy" => {
                    let strategy = TmuxStrategy::from_str(&value)?;
                    config::update_config(|config| {
                        config.tmux_strategy = strategy;
                    })?;
                    println!("Set tmux-strategy to: {}", value);
                }
                "prompt-strategy" => {
                    let prompt = value.parse::<bool>()
                        .map_err(|_| anyhow::anyhow!("Invalid boolean value: {}", value))?;
                    config::update_config(|config| {
                        config.prompt_for_strategy = prompt;
                    })?;
                    println!("Set prompt-strategy to: {}", prompt);
                }
                "in-tmux-behavior" => {
                    let behavior = InTmuxBehavior::from_str(&value)?;
                    config::update_config(|config| {
                        config.in_tmux_behavior = behavior;
                    })?;
                    println!("Set in-tmux-behavior to: {}", value);
                }
                _ => anyhow::bail!("Unknown config key: {}. Valid keys: tmux-strategy, prompt-strategy, in-tmux-behavior", key),
            }
        }
        ConfigCommands::Reset => {
            let default_config = StacksConfig::default();
            config::save_config(&default_config)?;
            println!("Configuration reset to defaults");
        }
    }
    Ok(())
}

async fn show_config() -> Result<()> {
    let config = config::load_config()?;
    let config_path = config::get_config_path()?;
    
    println!("📋 Current Configuration");
    println!("═══════════════════════");
    
    println!("\n🎯 Tmux Strategy: {} ({})", 
        config.tmux_strategy.as_str(), 
        config.tmux_strategy.description()
    );
    println!("   Options:");
    for strategy in [
        TmuxStrategy::SeparateSessions,
        TmuxStrategy::QuadSplit,
        TmuxStrategy::HorizontalSplit,
        TmuxStrategy::MultipleWindows,
    ] {
        let marker = if strategy.as_str() == config.tmux_strategy.as_str() { "→" } else { " " };
        println!("   {} {}: {}", marker, strategy.as_str(), strategy.description());
    }
    
    println!("\n🔄 Prompt Strategy: {}", if config.prompt_for_strategy { "enabled" } else { "disabled" });
    println!("   • enabled: Ask which tmux strategy to use each time");
    println!("   • disabled: Use default tmux strategy without asking");
    
    println!("\n🖥️  In-Tmux Behavior: {} ({})", 
        config.in_tmux_behavior.as_str(),
        config.in_tmux_behavior.description()
    );
    println!("   Options:");
    for behavior in [
        InTmuxBehavior::NewWindows,
        InTmuxBehavior::NewSession,
        InTmuxBehavior::Ask,
    ] {
        let marker = if behavior.as_str() == config.in_tmux_behavior.as_str() { "→" } else { " " };
        println!("   {} {}: {}", marker, behavior.as_str(), behavior.description());
    }
    
    println!("\n📁 Config file: {}", config_path.display());
    
    Ok(())
}

async fn interactive_config_editor() -> Result<()> {
    use dialoguer::{Select, Confirm};
    
    let mut config = config::load_config()?;
    let mut changes_made = false;
    
    loop {
        println!("\n📋 Interactive Configuration Editor");
        println!("══════════════════════════════════");
        
        let options = vec![
            format!("Tmux Strategy: {} ({})", config.tmux_strategy.as_str(), config.tmux_strategy.description()),
            format!("Prompt Strategy: {}", if config.prompt_for_strategy { "enabled" } else { "disabled" }),
            format!("In-Tmux Behavior: {} ({})", config.in_tmux_behavior.as_str(), config.in_tmux_behavior.description()),
            "💾 Save and Exit".to_string(),
            "❌ Exit without Saving".to_string(),
        ];
        
        let selection = Select::new()
            .with_prompt("Select setting to modify")
            .items(&options)
            .default(0)
            .interact()?;
            
        match selection {
            0 => {
                // Tmux Strategy
                let strategies = vec![
                    TmuxStrategy::SeparateSessions,
                    TmuxStrategy::QuadSplit,
                    TmuxStrategy::HorizontalSplit,
                    TmuxStrategy::MultipleWindows,
                ];
                
                let strategy_options: Vec<String> = strategies.iter()
                    .map(|s| format!("{}: {}", s.as_str(), s.description()))
                    .collect();
                
                let current_index = strategies.iter().position(|s| s.as_str() == config.tmux_strategy.as_str()).unwrap_or(0);
                
                let selected_strategy = Select::new()
                    .with_prompt("Choose tmux strategy")
                    .items(&strategy_options)
                    .default(current_index)
                    .interact()?;
                    
                if strategies[selected_strategy].as_str() != config.tmux_strategy.as_str() {
                    config.tmux_strategy = strategies[selected_strategy].clone();
                    changes_made = true;
                    println!("✅ Updated tmux strategy to: {}", config.tmux_strategy.as_str());
                }
            }
            1 => {
                // Prompt Strategy
                let prompt_options = vec![
                    "disabled: Use default strategy without asking",
                    "enabled: Ask which strategy to use each time",
                ];
                
                let current_index = if config.prompt_for_strategy { 1 } else { 0 };
                
                let selected = Select::new()
                    .with_prompt("Choose prompt behavior")
                    .items(&prompt_options)
                    .default(current_index)
                    .interact()?;
                    
                let new_value = selected == 1;
                if new_value != config.prompt_for_strategy {
                    config.prompt_for_strategy = new_value;
                    changes_made = true;
                    println!("✅ Updated prompt strategy to: {}", if new_value { "enabled" } else { "disabled" });
                }
            }
            2 => {
                // In-Tmux Behavior
                let behaviors = vec![
                    InTmuxBehavior::NewWindows,
                    InTmuxBehavior::NewSession,
                    InTmuxBehavior::Ask,
                ];
                
                let behavior_options: Vec<String> = behaviors.iter()
                    .map(|b| format!("{}: {}", b.as_str(), b.description()))
                    .collect();
                
                let current_index = behaviors.iter().position(|b| b.as_str() == config.in_tmux_behavior.as_str()).unwrap_or(0);
                
                let selected_behavior = Select::new()
                    .with_prompt("Choose in-tmux behavior")
                    .items(&behavior_options)
                    .default(current_index)
                    .interact()?;
                    
                if behaviors[selected_behavior].as_str() != config.in_tmux_behavior.as_str() {
                    config.in_tmux_behavior = behaviors[selected_behavior].clone();
                    changes_made = true;
                    println!("✅ Updated in-tmux behavior to: {}", config.in_tmux_behavior.as_str());
                }
            }
            3 => {
                // Save and Exit
                if changes_made {
                    config::save_config(&config)?;
                    println!("💾 Configuration saved successfully!");
                } else {
                    println!("No changes to save.");
                }
                break;
            }
            4 => {
                // Exit without Saving
                if changes_made {
                    let confirm_exit = Confirm::new()
                        .with_prompt("You have unsaved changes. Exit anyway?")
                        .default(false)
                        .interact()?;
                    if !confirm_exit {
                        continue;
                    }
                }
                println!("Configuration editor exited without saving.");
                break;
            }
            _ => unreachable!(),
        }
    }
    
    Ok(())
}
