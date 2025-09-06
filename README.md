# Claude Code Stacks

Complete workflow automation system for Claude Code with tmux integration and intelligent stack management.

## Quick Start

```bash
# Initial setup
./claude-workflow-manager.sh setup

# Start workflow for your project  
./claude-workflow-manager.sh start /path/to/your/project

# Apply complex changes with automatic stack selection
./claude-workflow-manager.sh apply "update repo to work with fichub. ensure it works with 'purple days'"
```

## System Overview

This system provides 7 specialized stacks that work together to automate your development workflow:

- **stack-1**: Automatic linting across projects
- **stack-2**: Testing of examples (nginx, docker, compose)  
- **stack-3**: Clark idiomatic style formatting
- **stack-4**: Git operations with auto subtree services
- **stack-5**: CI/CD workflows
- **stack-6**: Design doc generation to Google Drive
- **stack-7**: Database setup with MCP configuration

## Architecture

```
claude-workflow-manager.sh    # Main orchestration
├── stack-router.sh          # Individual stack management  
└── stacks/                  # Stack configurations
    ├── stack-1/            # Linting
    ├── stack-2/            # Testing
    ├── stack-3/            # Style formatting
    ├── stack-4/            # Git operations
    ├── stack-5/            # CI/CD
    ├── stack-6/            # Documentation
    └── stack-7/            # Database setup
```

Each stack contains:
- `CLAUDE.md` - Stack description and capabilities
- `init.sh` - Initialization script
- `.local-settings.json` - Claude Code agent permissions

## Usage

### Basic Commands

```bash
# List available stacks
./stack-router.sh list

# Run specific stacks
./stack-router.sh run stack-1 stack-3

# Show workflow status  
./claude-workflow-manager.sh status

# Stop all workflows
./claude-workflow-manager.sh stop
```

### Advanced Workflow

```bash
# Resoul-specific workflow (example from CLAUDE.md)
./claude-workflow-manager.sh resoul "add fichub support with character encoding fixes"
```

This automatically:
1. Detects required stacks based on change description
2. Starts tmux session with multiple panes
3. Initializes Claude Code agents in each stack
4. Provides auto-approval for common operations

### Tmux Integration

The system creates a tmux session called `claude-workflow` with:
- `main` - Your project workspace
- `router` - Stack management interface  
- `stack-1` through `stack-7` - Individual stack workspaces

Attach with: `tmux attach-session -t claude-workflow`

## Stack Details

### stack-1: Linting
Auto-detects project type and configures appropriate linters (eslint, ruff, clippy).

### stack-2: Testing  
Validates Docker configs, Nginx setups, and project examples.

### stack-3: Style Formatting
Enforces Clark's style: no emojis, concise READMEs, use uv/bun.

### stack-4: Git Operations
Automated git workflows with subtree management.

### stack-5: CI/CD
GitHub Actions and GitLab CI pipeline management.

### stack-6: Documentation
Auto-generates design docs and syncs to Google Drive.

### stack-7: Database Setup
Reads docker-compose files and configures MCP database connections.

## Configuration

Global config at `~/.claude-workflow-config`:
```bash
WORKFLOW_ROOT="/path/to/claude-code-stacks"
AUTO_APPROVE_SAFE_COMMANDS=true
USE_TMUX_INTEGRATION=true
DEFAULT_STACKS="stack-1,stack-3"
```

## Dependencies

- tmux (for multi-pane management)
- git (for repository operations)
- docker (optional, for testing stacks)
- bun/uv (installed by stack-3 as needed)

Install with: `./claude-workflow-manager.sh install-deps`