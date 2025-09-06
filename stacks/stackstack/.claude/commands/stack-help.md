# Stack Help Command

## Description
Comprehensive guide to using the stacks system - your complete reference for working with stacks.

## Usage
```
/stack-help [topic]
```

## Parameters
- `topic` (optional): Specific help topic (basic, workflow, commands, troubleshooting)

## Complete Stacks Guide

### What Are Stacks?
Stacks are pre-configured Claude Code environments that provide:
- **🤖 Specialized agents** - AI assistants for specific tasks
- **⚡ Slash commands** - Quick actions for common workflows
- **⚙️ Optimized settings** - Configurations tailored to project types  
- **🔌 MCP integrations** - Enhanced capabilities through external services

### How to Get Started

#### 1. Browse Available Stacks
```bash
stacks
```
- Shows all available stacks with descriptions
- Use fuzzy search by typing to filter
- Navigate with arrow keys

#### 2. Select Stacks
- **Space** to select/deselect stacks
- **Enter** to proceed with selected stacks
- **Ctrl+C** to cancel

#### 3. Confirm Checkout
- Review selected stacks
- Type 'yes' or just press Enter to proceed
- Stack files are downloaded and integrated

#### 4. Commit Changes (Important!)
After checkout, run:
```bash
/stack-commit
```
This commits the new .claude files and stack integrations to git.

### Available Stacks & When to Use Them

#### 🔧 ts-lint-stack
- **Purpose**: TypeScript/JavaScript code quality
- **Best for**: Any TS/JS project
- **Provides**: ESLint, Biome, auto-fixing, pre-commit hooks
- **Commands**: `/lint`, `/fix-lint`, `/setup-prettier`

#### 📚 stackstack
- **Purpose**: Stack management and workflow automation  
- **Best for**: Every project (helps manage other stacks)
- **Provides**: Git automation, validation, usage guidance
- **Commands**: `/stack-commit`, `/stack-status`, `/stack-validate`

#### 🧪 stack-2  
- **Purpose**: Infrastructure testing and validation
- **Best for**: Projects with Docker, nginx, or infrastructure
- **Provides**: Container testing, config validation
- **Commands**: `/test-docker`, `/validate-nginx`

### Common Workflows

#### Starting a New TypeScript Project
```bash
# 1. Check out essential stacks
stacks
# Select: ts-lint-stack, stackstack

# 2. Commit the stack setup  
/stack-commit

# 3. Set up linting for your project
/lint
```

#### Full-Stack Development Setup
```bash
# 1. Get comprehensive stack coverage
stacks  
# Select: ts-lint-stack, stack-2, stackstack

# 2. Commit everything
/stack-commit

# 3. Validate the setup
/stack-validate

# 4. Check overall status
/stack-status
```

### Stack Management Commands

#### Essential Commands
- `/stack-help` - This help guide
- `/stack-status` - Show status of all stacks
- `/stack-commit` - Commit stack changes to git
- `/stack-validate` - Check stack health and configuration

#### When to Use Each Command
- **After checkout**: `/stack-commit` (always!)
- **Before pushing**: `/stack-status` (check for uncommitted changes)
- **Troubleshooting**: `/stack-validate` (find and fix issues)
- **Learning**: `/stack-help` (comprehensive guidance)

### Project Type Recommendations

Choose stacks based on your project:

- **🟨 TypeScript/JavaScript**: `ts-lint-stack` + `stackstack`
- **🐳 Infrastructure/DevOps**: `stack-2` + `stackstack`  
- **🌐 Full-Stack Web App**: `ts-lint-stack` + `stack-2` + `stackstack`
- **📦 Any Project**: Always include `stackstack` for management

### Troubleshooting

#### Stack Not Working?
```bash
/stack-validate stackname
```
This will check for issues and suggest fixes.

#### Commands Not Available?
- Ensure you committed after checkout: `/stack-commit`
- Restart Claude Code to refresh command discovery
- Check if symlinks are broken: `/stack-validate`

#### Git Issues?
- Check what needs committing: `/stack-status`
- Commit stack changes: `/stack-commit`
- Validate git state: `git status`

### Advanced Usage

#### Multiple Projects
Each project can have different stack combinations. The stacks integrate at the project level.

#### Stack Updates
```bash
stacks pull stackname  # Update specific stack
stacks push stackname  # Push your changes back
```

#### Custom Configurations
After checkout, you can customize:
- `.claude/.local-settings.json` - Add your preferred settings
- Individual agent/command files - Modify as needed
- Stack configurations - Adapt to your workflow

### Getting Help
- `/stack-help` - This comprehensive guide
- `/stack-help basic` - Just the essentials  
- `/stack-help workflow` - Step-by-step workflows
- `/stack-help troubleshooting` - Common issues and fixes

### Best Practices
1. **Always commit after checkout**: Use `/stack-commit`
2. **Start with stackstack**: It helps manage everything else
3. **Validate regularly**: Use `/stack-validate` to catch issues early
4. **Check status before pushing**: Use `/stack-status` 
5. **Choose stacks thoughtfully**: Match them to your project needs

---
*This guide covers the complete stacks system. For specific stack documentation, see individual CLAUDE.md files in each stack.*