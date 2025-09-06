use clap::{Parser, Subcommand};
use anyhow::Result;

mod cli;
mod core;
mod utils;

use cli::{checkout, push, status, pull, worktree, sync};

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
        None => {
            // Default behavior - run checkout command
            checkout::run().await
        }
    }
}
