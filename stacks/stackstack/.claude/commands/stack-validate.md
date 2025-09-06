# Stack Validate Command

## Description
Validate stack configurations, symlinks, and integration health across all stacks.

## Usage
```
/stack-validate [stack-name]
```

## Parameters
- `stack-name` (optional): Validate specific stack. If omitted, validates all stacks.

## Validation Checks

### 1. Symlink Validation
- ✅ All agent files properly symlinked to `.claude/agents/`
- ✅ All command files properly symlinked to `.claude/commands/`
- ✅ Symlinks point to correct stack files
- ✅ No broken or missing symlinks

### 2. Configuration Validation
- ✅ `.claude/.local-settings.json` properly merged
- ✅ CLAUDE.md contains correct stack imports
- ✅ Stack CLAUDE.md files exist and are valid
- ✅ Required MCP servers available

### 3. Git Integration
- ✅ Stack directories exist as expected
- ✅ No uncommitted changes that should be committed
- ✅ Subtree integration working correctly

### 4. File Structure
- ✅ Stack directory structure is correct
- ✅ Required files (.claude/, CLAUDE.md) present
- ✅ Agent and command files are properly formatted

## Example Output
```
🔍 Validating Stack: ts-lint-stack
═════════════════════════════════

✅ Symlinks:
  • .claude/agents/ts-lint-stack_linting-agent.md → stacks/ts-lint-stack/.claude/agents/linting-agent.md
  • .claude/commands/ts-lint-stack_lint-interface.md → stacks/ts-lint-stack/.claude/commands/lint-interface.md

✅ Configuration:
  • Settings properly merged in .claude/.local-settings.json
  • Import added to CLAUDE.md: @stacks/ts-lint-stack/CLAUDE.md

✅ Files:
  • CLAUDE.md exists and valid
  • All required .claude files present

🎉 Stack ts-lint-stack is properly configured!
```

## Error Detection
- 🔴 **Missing Files**: Required stack files not found
- 🔴 **Broken Symlinks**: Symlinks pointing to non-existent files
- 🔴 **Configuration Issues**: Settings not properly merged
- 🔴 **Git Issues**: Uncommitted changes or integration problems

## Auto-Fix Suggestions
When issues are found, provides actionable commands to fix them:
```bash
# Fix broken symlinks
ln -sf ../stacks/ts-lint-stack/.claude/agents/linting-agent.md .claude/agents/ts-lint-stack_linting-agent.md

# Commit missing changes
git add .claude/ CLAUDE.md
git commit -m "fix(stacks): repair ts-lint-stack integration"
```