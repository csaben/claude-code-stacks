# Claude Code Stacks - System Reference

## Architecture Overview

The Claude Code Stacks system evolved from a complex stack management system to a simplified prompt storage and symlink manager. This reference documents the significant components developed during the project.

## Key Components

### 1. Installation System (`install.sh`)

**Purpose**: Creates global `stacks` CLI command and manages stack cache.

**Key Features**:
- One-liner curl installation: `curl -sSL https://raw.githubusercontent.com/csaben/claude-code-stacks/main/install.sh | bash`
- Creates binary at `/home/user/.local/bin/stacks`
- Manages cache directory at `~/.claude-stacks/`
- Git-based stack repository cloning from https://github.com/csaben/claude-code-stacks.git

**Critical Functions**:
```bash
update_cache() # Clones/updates stack repository
claude_checkout() # Launches Claude with stack prompts (final evolution)
setup_stack_in_project() # Creates git worktrees and symlinks
```

### 2. Stack Configuration Format

**Location**: `/stacks/stack-{1-7}/CLAUDE.md`

**Required Headers**:
```markdown
# Description: Brief description of stack purpose
# Setup Prompt: "Detailed instructions for Claude to set up this stack..."
```

**Stack Definitions**:
- **stack-1**: Automatic linting across the project
- **stack-2**: Automatic testing of examples (nginx, docker compose, dockerfile) 
- **stack-3**: Clark idiomatic style formatting (no emojis, concise readmes, use uv, use bun)
- **stack-4**: Git commands with auto subtree services
- **stack-5**: CI/CD workflows for git
- **stack-6**: Design doc generation and update to Google Drive
- **stack-7**: Database setup with MCP configuration awareness

### 3. Agent Discovery System

**Problem**: Claude Code looks for agents in `.claude/agents/` directories only.

**Solution**: Symlink-based discovery system.
- Git worktrees created in `./stack-{n}/` directories
- Symlinks from `./stack-{n}/.claude/agents/` to `./.claude/agents/`
- Allows Claude Code to discover agents from multiple stacks simultaneously

**Example Structure**:
```
project/
├── .claude/
│   ├── agents/
│   │   ├── clark-style-enforcer.md -> ../../stack-3/.claude/agents/clark-style-enforcer.md
│   │   └── linting-agent.md -> ../../stack-1/.claude/agents/linting-agent.md
├── stack-1/ (git worktree)
├── stack-3/ (git worktree)
```

### 4. Claude Code Integration

**Final Architecture**: Stacks as prompt storage for Claude.

**Workflow**:
1. User runs `stacks checkout "description"`
2. Stacks loads all available stack prompts from cache
3. Stacks launches Claude with comprehensive prompt containing:
   - User request
   - All available stack descriptions and setup prompts
   - Instructions for Claude to handle git operations and symlinks

**Example Prompt Structure**:
```
User request: "I need linting for this project"

Available Claude Code stacks:
**stack-1**: Linting
Setup: "Create git worktree, install linters, configure..."

Based on the user's request, please:
1. Determine which stacks would be most helpful
2. Execute the setup prompts for those stacks
3. Create proper git worktrees
4. Set up symlinks for agent discovery
5. Apply immediate improvements
```

### 5. Command Interface

**Core Commands**:
- `stacks` - Show help/interactive selection
- `stacks checkout "description"` - Launch Claude with setup prompts
- `stacks list` - List available stacks
- `stacks status` - Show active stacks in current project

### 6. Git Integration Approaches

**Evolution**:
1. **Initial**: File copying to `.claude/stacks/`
2. **Intermediate**: Git sparse-checkout for specific stacks
3. **Final**: Git worktrees for parallel stack management

**Worktree Benefits**:
- Separate working directories for each stack
- Shared git history
- Independent branch tracking
- Cleaner symlink management

### 7. Agent File Format

**Required YAML Frontmatter**:
```yaml
---
name: agent-name
description: Agent description
tools: [list, of, tools]
---
```

**Example**: `clark-style-enforcer.md`
- Enforces no-emoji, concise documentation style
- Uses bun/uv for package management
- Applies idiomatic conventions

### 8. MCP Integration

**Purpose**: Model Context Protocol server management for specialized capabilities.

**Stack Requirements**:
- postgres: Database connectivity
- redis: Caching and session storage
- github: Repository management

**Configuration**: Checked during stack status, with setup hints provided.

### 9. Shell Integration

**PATH Configuration**: Automatic addition of `/home/user/.local/bin` to PATH.

**Tmux Integration**: Planned for multi-pane monitoring (not fully implemented).

### 10. Error Resolution History

**Key Issues Resolved**:
1. **Shell Compatibility**: Changed `curl | sh` to `curl | bash` for proper syntax support
2. **Heredoc Conflicts**: Fixed nested EOF markers in script generation
3. **Agent Discovery**: Implemented symlink-based discovery for Claude Code
4. **Claude CLI Options**: Fixed `--mode=plan` to `--permission-mode plan`
5. **Binary Updates**: Proper installation workflow for script changes

## Final State

The system successfully:
- Provides one-liner installation
- Stores stack prompts in git repository
- Launches Claude with comprehensive setup context
- Handles git worktree creation and symlink management
- Enables cumulative agent discovery across multiple stacks
- Applied Clark style successfully (emoji removal, concise documentation)

## Test Results

**Successful Test Case**:
```bash
cd audio-viewer-project
stacks checkout "I need Clark's idiomatic style checking"
# Result: Claude selected stack-3, created worktree, symlinked agents, applied style fixes to README.md
```

**Multi-Stack Test**:
```bash
stacks checkout "I need linting and Clark style"
# Result: Claude selected stack-1 and stack-3, both configured with symlinks
```

The system achieved the goal of natural language stack selection with Claude handling all complex setup operations.