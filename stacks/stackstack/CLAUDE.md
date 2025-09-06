# Description: Stack management, workflow automation, and comprehensive stacks usage guide

# Stack: stackstack - Complete Stacks System Guide & Automation

This stack provides comprehensive guidance on using the stacks ecosystem, plus automated workflow management for stack operations.

## What Are Stacks?

Stacks are pre-configured Claude Code environments that provide:
- **Specialized agents** for specific tasks (linting, testing, etc.)
- **Slash commands** for common workflows  
- **Settings and configurations** tailored to project types
- **MCP server integrations** for enhanced capabilities

Think of stacks as "workflow templates" that instantly set up Claude with the right tools for your project type.

## How to Use Stacks

### 1. Discover Available Stacks
```bash
stacks
```
This shows all available stacks with descriptions. Use fuzzy search to filter (type to search).

### 2. Select and Checkout Stacks
- Use arrow keys or type to filter stacks
- Select one or multiple stacks (space to select, enter to proceed)
- Confirm checkout when prompted

### 3. What Happens During Checkout
- Stack is added as git subtree to `stacks/stack-name/`
- Agents symlinked to `.claude/agents/stack-name_agent.md`
- Commands symlinked to `.claude/commands/stack-name_command.md`  
- Settings merged into `.claude/.local-settings.json`
- CLAUDE.md updated with `@stacks/stack-name/CLAUDE.md` import

### 4. After Checkout
- New slash commands become available immediately
- Specialized agents are ready to help
- Settings take effect for enhanced capabilities

## Available Stacks

### ts-lint-stack
**Purpose**: Comprehensive TypeScript/JavaScript linting
**When to use**: Any TS/JS project needing code quality enforcement
**Provides**: ESLint/Biome configuration, auto-fixing, pre-commit hooks

### stackstack  
**Purpose**: Stack management and workflow automation
**When to use**: Always (this stack helps manage other stacks)
**Provides**: Git workflow automation, stack validation, usage guidance

### stack-2
**Purpose**: Automatic testing for infrastructure (Docker, nginx)
**When to use**: Projects with containers, web servers, or infrastructure
**Provides**: Docker validation, nginx testing, configuration validation

## Common Workflows

### New TypeScript Project
```bash
# 1. Check out linting and stack management
stacks
# Select: ts-lint-stack, stackstack

# 2. Commit the stack setup (automated prompt)
/stack-commit

# 3. Start using linting commands
/lint  # or whatever commands the stack provides
```

### Full-Stack Development
```bash
# 1. Check out multiple stacks for comprehensive setup
stacks
# Select: ts-lint-stack, stack-2, stackstack

# 2. Commit setup
/stack-commit  

# 3. Validate everything is working
/stack-validate
```

## Project Type Recommendations

- **TypeScript/JavaScript**: ts-lint-stack + stackstack
- **Infrastructure/DevOps**: stack-2 + stackstack  
- **Full-Stack Web**: ts-lint-stack + stack-2 + stackstack
- **Any Project**: Always include stackstack for workflow management

## Capabilities
- Automated detection of stack additions requiring git commits
- Stack management commands and workflows
- Git workflow automation for .claude file changes
- Stack validation and health checking
- Documentation and guidance for stack operations

## Usage
This stack automatically detects when stacks have been added and guides through the proper git workflow to commit the changes.

## Commands
- `/stack-commit` - Commit stack additions and .claude file changes
- `/stack-status` - Show status of all stacks and pending changes
- `/stack-validate` - Validate stack configurations and setup
- `/stack-sync` - Sync stack changes and update documentation

## Triggers
- After `stacks` checkout command completes
- When .claude files are modified by stack operations
- Before pushing stack changes

## Agent Configuration
The Claude Code agent in this stack has access to:
- Git operations for staging and committing changes
- File system access for reading stack configurations
- Stack status checking and validation
- Automated workflow recommendations

## MCP Permissions
- Git repository access for staging/committing changes
- File system access for .claude configuration files
- Bash execution for stack commands
- Read access to stack directories and metadata

## Workflow Integration
This stack integrates with the stacks workflow to:
1. Detect when new stacks have been added via `stacks checkout`
2. Identify changes to .claude symlinks and settings
3. Provide git commit recommendations with proper messages
4. Ensure stack additions are properly tracked in git history