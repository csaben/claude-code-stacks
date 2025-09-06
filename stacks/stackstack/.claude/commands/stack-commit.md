# Stack Commit Command

## Description
Automatically detect and commit stack-related changes to git, including .claude file modifications, symlinks, and CLAUDE.md updates.

## Usage
```
/stack-commit [message]
```

## Parameters
- `message` (optional): Custom commit message. If not provided, generates appropriate message based on detected changes.

## What it does
1. **Detects stack changes**: Identifies modifications to .claude directory, symlinks, and CLAUDE.md
2. **Stages changes**: Runs `git add` on relevant files
3. **Commits changes**: Creates commit with descriptive message
4. **Validates**: Ensures all stack-related changes are properly tracked

## Examples
```bash
# Auto-detect and commit stack changes
/stack-commit

# Commit with custom message
/stack-commit "Add linting and testing stacks for new project setup"
```

## Typical commit messages generated:
- `feat(stacks): add ts-lint-stack with linting agents and settings`
- `feat(stacks): add stackstack for workflow automation`
- `chore(stacks): update .claude settings and symlinks`

## Files typically included:
- `.claude/agents/*` (new symlinks)
- `.claude/commands/*` (new symlinks)  
- `.claude/.local-settings.json` (updated settings)
- `CLAUDE.md` (new stack imports)
- `stacks/*/` (new subtree directories)

## Integration
This command is automatically suggested by the stack-manager agent after:
- Running `stacks checkout`
- Detecting .claude directory changes
- Before `stacks push` operations