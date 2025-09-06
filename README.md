# Claude Code Stacks

Git checkout and tmux orchestrator for Claude Code workflow setups. Let Claude decide which stacks to use, while stacks handles the git and tmux mechanics.

## One-Liner Installation

```bash
curl -LsSf https://raw.githubusercontent.com/csaben/claude-code-stacks/main/install.sh | bash
```

## Quick Start

```bash
# Navigate to any project
cd ~/my-awesome-project/

# Let Claude decide which stacks to checkout
stacks checkout "I need linting and style checking for this TypeScript project"

# Claude picks appropriate stacks, stacks does git checkout + tmux setup
# Then use Claude normally with the configured stacks
claude "help me set up this project properly"
```

## Key Features

- **Smart Orchestration**: Claude interprets your needs, stacks handles git checkout
- **Git-based**: Each stack is a proper git repository you can modify and contribute to
- **Tmux Integration**: Automatic multi-pane monitoring setup
- **Simple Interface**: Just describe what you need, no complex commands to remember

## System Architecture

```
Global Installation:
~/.local/bin/stacks           # Global CLI command
~/.claude-stacks/            # Cached repository

Per Project (Git-based):
my-project/
├── stack-1/                 # git checkout of linting stack
│   ├── .claude/
│   ├── CLAUDE.md
│   └── init.sh
├── stack-3/                 # git checkout of style stack
│   ├── .claude/
│   ├── CLAUDE.md
│   └── init.sh
├── src/
└── package.json
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

## Commands

```bash
# Core workflow
stacks checkout "description of what you need"  # Claude selects and checks out stacks
stacks list                                      # List available stacks  
stacks status                                    # Show active stacks in project
stacks tmux                                      # Start tmux monitoring session

# Maintenance  
stacks update                                    # Update stack repository cache
stacks contribute                                # Show modified stacks ready for contribution
```

## Example Usage

```bash
# Let Claude pick stacks based on your description
stacks checkout "React app that needs linting, testing, and Clark's style"
# → Claude selects stack-1 (linting), stack-2 (testing), stack-3 (style)
# → stacks does git checkout for each
# → Ready to use claude with those stack configurations

# Start tmux monitoring  
stacks tmux
# → Creates session with pane for each active stack
# → Jump between panes to monitor Claude operations

# Work normally with Claude
claude "set up this React project with proper linting"
# → Claude uses the checked-out stack configurations
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

## Contributing

The git-based approach makes contributing improvements simple:

1. **Modify locally**: Edit files directly in checked-out stack directories (e.g., `stack-1/`)
2. **Commit changes**: `cd stack-1 && git add . && git commit -m "improve linting config"`
3. **Push upstream**: `git push origin main` (or create feature branch)
4. **Natural request**: `stacks contribute` shows which stacks have modifications

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