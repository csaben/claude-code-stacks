# Claude Code Stacks CLI

A powerful CLI tool for managing Claude Code workflow configurations, git worktrees, and MCP server synchronization.

## 🚀 Quick Start

### Installation

Install with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/csaben/claude-code-stacks/main/install.sh | bash
```

This will:
- Download the latest binary for your platform
- Install it to `~/.local/bin/stacks`
- Update your PATH if needed

### Basic Usage

```bash
# Discover and checkout stacks
stacks

# Create git worktree with tmux session
stacks worktree

# Sync MCP servers from docker-compose
stacks sync
```

## 📖 Overview

Claude Code Stacks helps you manage reusable Claude Code configurations across projects. Each "stack" contains:

- **Agents**: Specialized Claude agents for specific tasks
- **Commands**: Custom slash commands
- **Settings**: Claude Code configuration
- **Documentation**: Stack-specific instructions

## 🎯 Commands

### `stacks` (default command)

Discover and check out one or more stacks from your `stacks/` directory.

**Features:**
- Interactive fuzzy search with `fzf`
- Multi-stack selection
- Automatic symlink creation for agents and commands
- Settings merging
- CLAUDE.md import updates
- MCP server validation

**Example:**
```bash
stacks
# Opens fzf interface to select stacks
# Processes selected stacks automatically
```

### `stacks worktree`

Create git worktrees with integrated tmux sessions for isolated development.

**Features:**
- Interactive branch strategy selection
- Configurable worktree locations
- Automatic tmux session creation
- Claude Code integration in split pane

**Example:**
```bash
stacks worktree
# Guided workflow:
# 1. Enter task name
# 2. Choose branch strategy
# 3. Select worktree location
# 4. Configure tmux session
```

### `stacks sync`

Synchronize MCP server configurations from docker-compose files.

**Features:**
- Auto-discovery of compose files
- Service type detection (Postgres, Redis, etc.)
- MCP command generation
- Automatic server configuration

**Example:**
```bash
stacks sync
# Scans for docker-compose.yml files
# Generates claude mcp add commands
# Applies configurations
```

## 📁 Stack Structure

A typical stack directory looks like:

```
stacks/
├── linting/
│   ├── .claude/
│   │   ├── agents/
│   │   │   └── linting-agent.md
│   │   ├── commands/
│   │   │   └── lint-interface.md
│   │   └── .local-settings.json
│   └── CLAUDE.md
└── testing/
    ├── .claude/
    │   ├── agents/
    │   │   └── testing-specialist.md
    │   └── .local-settings.json
    └── CLAUDE.md
```

### Stack Components

#### Agents (`stacks/*/. claude/agents/*.md`)
Claude agents with YAML frontmatter:
```markdown
---
name: linting-specialist
description: Specialized agent for code linting
---

Agent instructions and capabilities...
```

#### Settings (`stacks/*/.claude/.local-settings.json`)
Claude Code configuration:
```json
{
  "permissions": {
    "allow": ["npm run lint", "ruff check"]
  },
  "auto_approve": ["cargo clippy"]
}
```

#### Documentation (`stacks/*/CLAUDE.md`)
Stack-specific instructions:
```markdown
# Description: Automatic linting across the project
# Linting Stack

Provides comprehensive linting capabilities...
```

## 🔧 Configuration

### Dependencies

Required:
- `git` - Version control
- `tmux` - Terminal multiplexer (for worktree command)
- `fzf` - Fuzzy finder (for stack selection)

Optional:
- `claude` - Claude CLI (for MCP functionality)

### Environment

The CLI respects standard environment variables:
- `HOME` - User home directory
- `PATH` - Executable search path

## 🏗️ Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/csaben/claude-code-stacks.git
cd claude-code-stacks

# Build for current platform
./build.sh local

# Cross-compile for all platforms
./build.sh all

# Run tests
./build.sh test
```

### Project Structure

```
src/
├── main.rs                 # CLI entry point
├── cli/
│   ├── checkout.rs         # Stack checkout logic
│   ├── worktree.rs         # Git worktree management
│   └── sync.rs             # MCP synchronization
├── core/
│   ├── stack_manager.rs    # Stack discovery/validation
│   ├── symlink_manager.rs  # Symlink operations
│   ├── settings_merger.rs  # JSON settings merging
│   └── mcp_validator.rs    # MCP server validation
└── utils/
    ├── dependency_check.rs # System dependency validation
    └── claude_md_updater.rs # CLAUDE.md management
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run `./build.sh test` to verify
6. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🔗 Links

- [Claude Code Documentation](https://docs.anthropic.com/en/docs/claude-code)
- [GitHub Repository](https://github.com/csaben/claude-code-stacks)
- [Issue Tracker](https://github.com/csaben/claude-code-stacks/issues)