#!/bin/bash

# Claude Code Stacks - Contribution Workflow
# Natural language interface for contributing stack changes back to repository

set -e

REPO_URL="git@github.com:csaben/claude-code-stacks.git"
CACHE_DIR="$HOME/.claude-stacks"
CURRENT_PROJECT="$(pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

show_help() {
    cat << EOF
Claude Code Stack Contribution Tool

Usage: stack-contribute [command]

Commands:
    detect              Detect local stack modifications
    validate           Validate changes before contribution
    contribute         Create PR with local changes
    sync               Sync with remote repository

Natural Language Interface:
    Just describe what you want to do:
    "I modified the linting stack and want to contribute it back"
    "Push my stack changes to the main repo"
    "Check if my changes are ready for contribution"

EOF
}

detect_modifications() {
    echo -e "${BLUE}Detecting local stack modifications...${NC}"
    
    local claude_dir="$CURRENT_PROJECT/.claude"
    local modified_stacks=()
    
    if [[ ! -d "$claude_dir/stacks" ]]; then
        echo -e "${YELLOW}No stacks found in current project${NC}"
        return 1
    fi
    
    # Compare with cache
    for stack_dir in "$claude_dir/stacks"/stack-*; do
        if [[ -d "$stack_dir" ]]; then
            local stack_name=$(basename "$stack_dir")
            local cache_stack="$CACHE_DIR/stacks/$stack_name"
            
            if [[ -d "$cache_stack" ]]; then
                # Check for differences
                if ! diff -r "$stack_dir" "$cache_stack" >/dev/null 2>&1; then
                    modified_stacks+=("$stack_name")
                    echo -e "  ${GREEN}✓${NC} Modified: $stack_name"
                    
                    # Show specific changes
                    echo "    Changes detected:"
                    diff -r "$cache_stack" "$stack_dir" --brief 2>/dev/null | while read line; do
                        echo "      $line"
                    done
                fi
            else
                modified_stacks+=("$stack_name")
                echo -e "  ${GREEN}✓${NC} New stack: $stack_name"
            fi
        fi
    done
    
    if [[ ${#modified_stacks[@]} -eq 0 ]]; then
        echo -e "${YELLOW}No modifications detected${NC}"
        return 1
    fi
    
    echo ""
    echo -e "${GREEN}Found ${#modified_stacks[@]} modified stack(s)${NC}"
    return 0
}

validate_changes() {
    echo -e "${BLUE}Validating stack changes...${NC}"
    
    # Use Claude Code in headless mode to validate changes
    local validation_prompt="
Validate the local stack modifications for contribution readiness.

Please check:
1. All required files are present (CLAUDE.md, .local-settings.json, etc.)
2. Documentation is complete and accurate
3. No sensitive information is included
4. Changes follow established patterns
5. Natural language interfaces are properly configured

Provide a detailed validation report.
"
    
    echo "  Running Claude Code validation..."
    if command -v claude &> /dev/null; then
        claude --mode=plan -p "$validation_prompt"
    else
        echo -e "${YELLOW}  Claude Code not available for validation${NC}"
        echo -e "${YELLOW}  Manual review recommended${NC}"
    fi
}

create_contribution_branch() {
    local stack_name="$1"
    local branch_name="contribute-${stack_name}-$(date +%s)"
    
    echo -e "${BLUE}Creating contribution branch: $branch_name${NC}"
    
    # Navigate to cache directory (our local copy of the repo)
    cd "$CACHE_DIR"
    
    # Create new branch
    git checkout -b "$branch_name"
    
    # Copy modified stack
    local source_stack="$CURRENT_PROJECT/.claude/stacks/$stack_name"
    local target_stack="$CACHE_DIR/stacks/$stack_name"
    
    if [[ -d "$source_stack" ]]; then
        echo "  Copying $stack_name to repository..."
        cp -r "$source_stack"/* "$target_stack/"
        
        # Add changes
        git add "stacks/$stack_name/"
        
        # Create commit message using Claude Code
        local commit_msg=$(generate_commit_message "$stack_name")
        git commit -m "$commit_msg"
        
        echo -e "  ${GREEN}✓${NC} Created commit for $stack_name"
        return 0
    else
        echo -e "${RED}Error: Stack $stack_name not found in project${NC}"
        return 1
    fi
}

generate_commit_message() {
    local stack_name="$1"
    
    # Use Claude Code to generate appropriate commit message
    local prompt="
Generate a concise, technical commit message for contributing changes to $stack_name.

The commit should:
1. Follow conventional commit format
2. Be descriptive but concise  
3. Focus on the technical changes made
4. Include the stack name

Example format: 'feat(stack-1): add TypeScript linting support'
"
    
    if command -v claude &> /dev/null; then
        local msg=$(claude --mode=plan -p "$prompt" 2>/dev/null | head -1)
        if [[ -n "$msg" ]]; then
            echo "$msg"
        else
            echo "feat($stack_name): update stack configuration"
        fi
    else
        echo "feat($stack_name): update stack configuration"
    fi
}

create_pull_request() {
    echo -e "${BLUE}Creating pull request...${NC}"
    
    # Push branch
    local current_branch=$(git branch --show-current)
    git push origin "$current_branch"
    
    # Create PR using gh CLI if available
    if command -v gh &> /dev/null; then
        local pr_title="Stack contribution: $(git log -1 --format='%s')"
        local pr_body="
## Stack Contribution

This PR contributes improvements to one or more Claude Code stacks.

### Changes Made
$(git log --oneline -5)

### Testing
- [ ] Validated stack configuration
- [ ] Tested natural language interface
- [ ] Verified integration with Claude Code

### Notes
Contributed via claude-code-stacks automated workflow.
"
        
        gh pr create --title "$pr_title" --body "$pr_body"
        echo -e "${GREEN}✓ Pull request created${NC}"
    else
        echo -e "${YELLOW}GitHub CLI not available${NC}"
        echo "Manual PR creation required:"
        echo "  Repository: $REPO_URL"
        echo "  Branch: $current_branch"
    fi
}

contribute_workflow() {
    echo -e "${BLUE}Starting contribution workflow...${NC}"
    
    # Detect modifications
    if ! detect_modifications; then
        echo "No changes to contribute"
        return 0
    fi
    
    # Validate changes
    validate_changes
    
    # Ask for confirmation
    echo ""
    echo -e "${YELLOW}Ready to contribute changes?${NC}"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Contribution cancelled"
        return 0
    fi
    
    # Get modified stacks
    local claude_dir="$CURRENT_PROJECT/.claude"
    for stack_dir in "$claude_dir/stacks"/stack-*; do
        if [[ -d "$stack_dir" ]]; then
            local stack_name=$(basename "$stack_dir")
            local cache_stack="$CACHE_DIR/stacks/$stack_name"
            
            if [[ -d "$cache_stack" ]] && ! diff -r "$stack_dir" "$cache_stack" >/dev/null 2>&1; then
                create_contribution_branch "$stack_name"
            fi
        fi
    done
    
    # Create pull request
    create_pull_request
    
    echo -e "${GREEN}Contribution workflow completed!${NC}"
}

sync_with_remote() {
    echo -e "${BLUE}Syncing with remote repository...${NC}"
    
    cd "$CACHE_DIR"
    git checkout main
    git pull origin main
    
    echo -e "${GREEN}✓ Synced with remote${NC}"
}

process_natural_language() {
    local input="$1"
    
    # Simple pattern matching for natural language
    if [[ "$input" =~ (contribute|push|submit).*(back|repo) ]] || [[ "$input" =~ (modified|changed).*(stack) ]]; then
        contribute_workflow
    elif [[ "$input" =~ (detect|check|find).*(change|modif) ]]; then
        detect_modifications
    elif [[ "$input" =~ (validate|verify|test).*(change) ]]; then
        validate_changes
    elif [[ "$input" =~ (sync|update).*(remote|repo) ]]; then
        sync_with_remote
    else
        echo -e "${YELLOW}I understand you want to work with stack contributions.${NC}"
        echo "Here are the available options:"
        echo "  - Detect modifications"
        echo "  - Validate changes"  
        echo "  - Contribute changes"
        echo "  - Sync with remote"
        echo ""
        echo "What would you like to do?"
    fi
}

# Main command processing
case "${1:-}" in
    detect)
        detect_modifications
        ;;
    validate)
        validate_changes
        ;;
    contribute)
        contribute_workflow
        ;;
    sync)
        sync_with_remote
        ;;
    help|--help|-h)
        show_help
        ;;
    "")
        echo "Interactive mode - describe what you want to do:"
        read -p "> " user_input
        process_natural_language "$user_input"
        ;;
    *)
        # Treat as natural language input
        process_natural_language "$*"
        ;;
esac