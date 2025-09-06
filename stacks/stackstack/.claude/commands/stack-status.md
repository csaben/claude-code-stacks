# Stack Status Command

## Description
Show comprehensive status of all stacks including git status, pending changes, and integration health.

## Usage
```
/stack-status
```

## What it shows
1. **Stack Overview**: List all available stacks and their status
2. **Git Status**: Uncommitted changes in stack directories
3. **Integration Status**: .claude symlinks and settings integration
4. **Pending Actions**: Recommendations for commits, pushes, or fixes

## Example Output
```
📊 Stack Status Report
═══════════════════════

🌐 Available Stacks: 3
  ✅ ts-lint-stack (integrated, clean)
  ✅ stackstack (integrated, clean)  
  ✅ stack-2 (integrated, clean)

📝 Git Status:
  • No uncommitted stack changes
  
🔗 Integration Health:
  ✅ All agents symlinked correctly
  ✅ All commands available
  ✅ Settings merged properly
  ✅ CLAUDE.md imports up to date

💡 Recommendations:
  • All stacks are properly integrated
```

## Status Indicators
- ✅ **Clean**: No pending changes, properly integrated
- 📝 **Modified**: Has uncommitted changes  
- ⚠️ **Issues**: Configuration problems or missing files
- 🔄 **Behind**: Remote updates available

## Integration with Git
- Shows `git status` for stack-related files
- Identifies uncommitted .claude changes
- Suggests git operations when needed
- Validates symlink integrity

## When to Use
- After running `stacks checkout`
- Before `stacks push` operations
- When troubleshooting stack issues
- For regular health checks