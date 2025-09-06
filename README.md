# Claude Code Stacks

Complete natural language workflow automation system for Claude Code with global CLI, fzf integration, and tmux monitoring.

## One-Liner Installation

```bash
curl -LsSf https://raw.githubusercontent.com/csaben/claude-code-stacks/main/install.sh | bash
```

## 🎯 Quick Start

```bash
# Navigate to any project
cd ~/my-awesome-project/

# Interactive stack selection with fzf
stacks

# Natural language interface - just describe what you want!
claude "help me set up linting for this project"
claude "fix any style issues and apply Clark's guidelines"
claude "I modified the linting stack, contribute it back to the repo"
```

## ✨ Key Features

- **🌐 Global CLI**: Works from any directory with `stacks` command
- **🔍 fzf Integration**: Beautiful interactive stack selection
- **🗣️ Natural Language**: Just describe what you want in plain English
- **🔄 Auto-Contribution**: Easy workflow to contribute improvements back
- **📺 Tmux Monitoring**: Multi-pane monitoring of all active stacks
- **🤖 Headless Automation**: Uses Claude Code's headless mode for automation

## 🏗️ System Architecture

```
Global Installation:
~/.local/bin/stacks           # Global CLI command
~/.claude-stacks/            # Cached repository

Per Project:
my-project/
├── .claude/
│   ├── stacks/              # Active stack configurations
│   ├── commands/            # Natural language interfaces
│   └── CLAUDE.md            # Project-specific configuration
```

## 📋 Available Stacks

| Stack | Description | Natural Language Examples |
|-------|-------------|---------------------------|
| **stack-1** | Automatic linting (eslint, ruff, clippy) | "fix linting issues", "check code style" |
| **stack-2** | Testing (Docker, nginx, examples) | "test my configurations", "validate examples" |  
| **stack-3** | Clark idiomatic style | "apply Clark style", "remove emojis", "use uv/bun" |
| **stack-4** | Git operations & subtree management | "commit these changes", "manage git workflow" |
| **stack-5** | CI/CD workflows | "set up GitHub Actions", "deploy pipeline" |
| **stack-6** | Design doc generation → Google Drive | "update documentation", "sync design docs" |
| **stack-7** | Database setup with MCP config | "configure database", "set up postgres" |

## 🎛️ Commands

### Global Commands (available anywhere)
```bash
stacks                    # Interactive stack selection with fzf
stacks list              # List available stacks
stacks status           # Show active stacks in current project  
stacks update           # Update stack repository cache
stack-contribute        # Contribute local changes back to repo
```

### Natural Language Interface
Just talk to Claude Code naturally:
```bash
claude "I want to add linting to this TypeScript project"
claude "Test my Docker configurations"
claude "Apply our style guidelines to the entire codebase"  
claude "Push my stack improvements to the main repository"
```

### Tmux Monitoring
```bash
tmux-stack-manager.sh start    # Start multi-pane monitoring
tmux-stack-manager.sh status   # Check monitoring status
tmux-stack-manager.sh attach   # Attach to session
```

## 🚦 Complete User Journey

### First Time Setup (Any New Machine)
```bash
# One command installation
curl -LsSf https://raw.githubusercontent.com/csaben/claude-code-stacks/main/install.sh | bash

# Restart terminal or source profile
source ~/.zshrc  # or ~/.bashrc
```

### Daily Workflow
```bash
# Navigate to your project
cd ~/development/my-project/

# Select stacks interactively  
stacks
# ┌─ stack-1: Automatic linting ──────────────────┐
# │ stack-2: Testing infrastructure               │ 
# │ stack-3: Clark idiomatic style               │
# │ stack-4: Git operations                      │
# └───────────────────────────────────────────────┘

# Start monitoring (optional)
tmux-stack-manager.sh start

# Work naturally with Claude Code
claude "help me improve this codebase"
claude "fix any issues you find"
claude "prepare this for deployment"
```

### Contributing Improvements
```bash
# After modifying a stack locally
claude "I improved the linting configuration, can you contribute it back?"
# System automatically:
# 1. Detects your changes
# 2. Validates modifications  
# 3. Creates contribution branch
# 4. Opens PR to csaben/claude-code-stacks
```

## 🔧 Advanced Features

### Multi-Stack Operations
The system intelligently combines stacks based on your request:
```bash
claude "prepare this project for production"
# Automatically activates:
# - stack-1: Linting
# - stack-2: Testing  
# - stack-3: Style formatting
# - stack-4: Git operations
# - stack-5: CI/CD setup
```

### Tmux Workspace  
Creates dedicated monitoring panes for each active stack:
- **Main**: Your primary workspace
- **Control**: Stack management interface
- **stack-1**: Linting status and controls
- **stack-2**: Testing results and logs
- **stack-3**: Style compliance monitoring
- **And so on...**

### Headless Automation
Leverages Claude Code's headless mode for automated operations:
```bash
# Background validation
claude --mode=plan -p "validate stack configuration"

# Automated fixes  
claude --mode=auto-accept -p "fix all linting violations"
```

## 🎨 The Clark Style (stack-3)

Enforces opinionated style guidelines:
- ❌ **No emojis** in code, comments, or documentation
- 📝 **Concise READMEs** - essential information only (<200 lines)
- 📦 **Use uv** for Python (not pip/poetry)
- 📦 **Use bun** for JavaScript/TypeScript (not npm/yarn)
- 🔧 **Minimal dependencies** - prefer standard library
- 📖 **Clear, technical language** without marketing fluff

## 🤝 Contributing

The system makes it trivial to contribute improvements:

1. **Modify locally**: Edit any stack in `.claude/stacks/`
2. **Natural request**: `claude "contribute my changes"`
3. **Automatic workflow**: System handles validation, branching, and PR creation

## 📁 Repository Structure

```
claude-code-stacks/
├── install.sh                    # One-liner installation
├── stacks/                       # Stack definitions
│   ├── stack-1/                  # Linting stack
│   │   ├── CLAUDE.md
│   │   ├── .local-settings.json
│   │   ├── .claude/
│   │   │   ├── commands/         # Natural language interfaces
│   │   │   └── agents/           # Specialized agents
│   │   └── init.sh
│   └── stack-{2-7}/             # Other stacks
├── natural-language-processor.py # Intent analysis
├── stack-contribute.sh           # Contribution workflow
├── tmux-stack-manager.sh         # Multi-pane monitoring
└── README.md                     # This file
```

## 🔗 Integration Points

- **Claude Code**: Headless mode, natural language processing
- **Git**: Automatic contribution workflows
- **Tmux**: Multi-pane monitoring and management
- **fzf**: Beautiful interactive selection
- **Docker**: Configuration testing and validation
- **Package Managers**: uv, bun integration

## 🎯 Perfect for Resoul Workflow

Exactly matches your described workflow:
```bash
cd ~/resoul-project/
claude "update repo to work with fichub. ensure it works with 'purple days' on fichub"

# System automatically:
# ✅ Applies linting (stack-1)
# ✅ Tests examples (stack-2)  
# ✅ Enforces Clark style (stack-3)
# ✅ Manages git operations (stack-4)
# ✅ Sets up CI/CD (stack-5)
# ✅ Updates design docs → Google Drive (stack-6)
# ✅ Configures databases if needed (stack-7)
```

No commands to remember. No syntax to learn. Just natural language describing what you want to accomplish.