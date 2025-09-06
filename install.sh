#!/bin/bash

# Claude Code Stacks - One-liner Installation
# curl -LsSf https://raw.githubusercontent.com/csaben/claude-code-stacks/main/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="git@github.com:csaben/claude-code-stacks.git"
REPO_URL_HTTPS="https://github.com/csaben/claude-code-stacks.git"
INSTALL_DIR="$HOME/.local/bin"
CACHE_DIR="$HOME/.claude-stacks"
BINARY_NAME="stacks"

print_header() {
    echo -e "${BLUE}"
    cat << 'EOF'
   _____ _                 _        _____ _             _        
  / ____| |               | |      / ____| |           | |       
 | |    | | __ _ _   _  __| | ___ | (___ | |_ __ _  ___| | _____ 
 | |    | |/ _` | | | |/ _` |/ _ \ \___ \| __/ _` |/ __| |/ / __|
 | |____| | (_| | |_| | (_| |  __/ ____) | || (_| | (__|   <\__ \
  \_____|_|\__,_|\__,_|\__,_|\___||_____/ \__\__,_|\___|_|\_\___/
                                                                 
EOF
    echo -e "${NC}"
    echo -e "${GREEN}Claude Code Stacks - Global Workflow Automation${NC}"
    echo ""
}

detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

check_dependencies() {
    local missing_deps=()
    
    # Check for git
    if ! command -v git &> /dev/null; then
        missing_deps+=("git")
    fi
    
    # Check for fzf
    if ! command -v fzf &> /dev/null; then
        echo -e "${YELLOW}Warning: fzf not found. Installing fzf...${NC}"
        install_fzf
    fi
    
    # Check for tmux
    if ! command -v tmux &> /dev/null; then
        echo -e "${YELLOW}Warning: tmux not found. Installing tmux...${NC}"
        install_tmux
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        echo -e "${RED}Error: Missing required dependencies: ${missing_deps[*]}${NC}"
        echo "Please install the missing dependencies and try again."
        exit 1
    fi
}

install_fzf() {
    local os=$(detect_os)
    
    if [[ "$os" == "linux" ]]; then
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y fzf
        elif command -v yum &> /dev/null; then
            sudo yum install -y fzf
        elif command -v pacman &> /dev/null; then
            sudo pacman -S fzf
        else
            echo "Installing fzf via git..."
            git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
            ~/.fzf/install --no-update-rc --key-bindings --completion
        fi
    elif [[ "$os" == "macos" ]]; then
        if command -v brew &> /dev/null; then
            brew install fzf
        else
            echo "Installing fzf via git..."
            git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
            ~/.fzf/install --no-update-rc --key-bindings --completion
        fi
    fi
}

install_tmux() {
    local os=$(detect_os)
    
    if [[ "$os" == "linux" ]]; then
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y tmux
        elif command -v yum &> /dev/null; then
            sudo yum install -y tmux
        elif command -v pacman &> /dev/null; then
            sudo pacman -S tmux
        fi
    elif [[ "$os" == "macos" ]]; then
        if command -v brew &> /dev/null; then
            brew install tmux
        fi
    fi
}

setup_directories() {
    echo -e "${BLUE}Setting up directories...${NC}"
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Create cache directory
    mkdir -p "$CACHE_DIR"
    
    echo "  ✓ Created $INSTALL_DIR"
    echo "  ✓ Created $CACHE_DIR"
}

install_stacks_binary() {
    echo -e "${BLUE}Installing stacks binary...${NC}"
    
    cat > "$INSTALL_DIR/$BINARY_NAME" << 'EOF'
#!/bin/bash

# Claude Code Stacks - Global CLI
# Natural language interface to Claude Code workflow stacks

set -e

CACHE_DIR="$HOME/.claude-stacks"
REPO_URL="https://github.com/csaben/claude-code-stacks.git"
CURRENT_PROJECT_DIR="$(pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

show_help() {
    cat << EOF
Claude Code Stacks - Natural Language Workflow Automation

Usage: stacks [command]

Commands:
    (no args)         Interactive stack selection with fzf
    list              List available stacks  
    status            Show active stacks in current project
    update            Update stack repository cache
    contribute        Contribute local changes back to repository
    help              Show this help message

Natural Language Interface:
    Just describe what you want to do in plain English!
    
    Examples:
    "I want to add linting to this project"
    "Push my stack changes back to the repo"
    "Set up testing for my Docker configs"
    "Apply Clark's style guidelines"

The system understands your intent and executes the appropriate workflows.
EOF
}

update_cache() {
    echo -e "${BLUE}Updating stack repository cache...${NC}"
    
    if [[ -d "$CACHE_DIR/.git" ]]; then
        cd "$CACHE_DIR"
        git pull origin main
        echo "  ✓ Updated existing cache"
    else
        rm -rf "$CACHE_DIR"
        git clone "$REPO_URL" "$CACHE_DIR"
        echo "  ✓ Cloned fresh repository"
    fi
}

list_stacks() {
    if [[ ! -d "$CACHE_DIR/stacks" ]]; then
        echo "Initializing stack cache..."
        update_cache
    fi
    
    echo -e "${GREEN}Available stacks:${NC}"
    for stack_dir in "$CACHE_DIR/stacks"/stack-*; do
        if [[ -d "$stack_dir" ]]; then
            local stack_name=$(basename "$stack_dir")
            local description=$(grep -m1 "^# Description:" "$stack_dir/CLAUDE.md" 2>/dev/null | cut -d: -f2- | xargs || echo "No description")
            printf "  %-15s %s\n" "$stack_name" "$description"
        fi
    done
}

interactive_selection() {
    if [[ ! -d "$CACHE_DIR/stacks" ]]; then
        echo "Initializing stack cache..."
        update_cache
    fi
    
    # Create selection menu
    local options=()
    for stack_dir in "$CACHE_DIR/stacks"/stack-*; do
        if [[ -d "$stack_dir" ]]; then
            local stack_name=$(basename "$stack_dir")
            local description=$(grep -m1 "^# Description:" "$stack_dir/CLAUDE.md" 2>/dev/null | cut -d: -f2- | xargs || echo "No description")
            options+=("$stack_name: $description")
        fi
    done
    
    # Use fzf for selection
    local selected=$(printf '%s\n' "${options[@]}" | fzf --multi --prompt="Select stacks: " --header="Use TAB for multi-select, ENTER to confirm")
    
    if [[ -n "$selected" ]]; then
        echo -e "${GREEN}Selected stacks:${NC}"
        echo "$selected" | while IFS= read -r line; do
            local stack_name=$(echo "$line" | cut -d: -f1)
            echo "  ✓ $stack_name"
            setup_stack_in_project "$stack_name"
        done
        
        echo ""
        echo -e "${BLUE}Stacks are now available in your project!${NC}"
        echo -e "Try: ${YELLOW}claude 'help me set up linting for this project'${NC}"
    fi
}

setup_stack_in_project() {
    local stack_name="$1"
    local stack_source="$CACHE_DIR/stacks/$stack_name"
    local project_claude_dir="$CURRENT_PROJECT_DIR/.claude"
    
    if [[ ! -d "$stack_source" ]]; then
        echo -e "${RED}Error: Stack '$stack_name' not found${NC}"
        return 1
    fi
    
    # Create .claude directory structure
    mkdir -p "$project_claude_dir/stacks/$stack_name"
    mkdir -p "$project_claude_dir/commands"
    
    # Copy stack configuration
    cp -r "$stack_source"/* "$project_claude_dir/stacks/$stack_name/"
    
    # Check MCP requirements for this stack
    echo "  Checking MCP requirements for $stack_name..."
    check_mcp_requirements "$stack_name"
    
    # Create project-specific CLAUDE.md if it doesn't exist
    if [[ ! -f "$project_claude_dir/CLAUDE.md" ]]; then
        cat > "$project_claude_dir/CLAUDE.md" << 'CLAUDE_EOF'
# Claude Code Project Configuration

This project uses Claude Code Stacks for workflow automation.

## Active Stacks
<!-- This section is automatically maintained -->

## Natural Language Interface

You can interact with this project using natural language. Just describe what you want to do:

- "Help me fix the linting issues"
- "Set up testing for this project" 
- "Apply our style guidelines"
- "Deploy my changes"
- "Update the documentation"

The system will automatically understand your intent and execute the appropriate workflows.

## Available Commands

The following automated workflows are available based on your active stacks:
<!-- Commands are automatically generated from active stacks -->
CLAUDE_EOF
    fi
    
    # Update active stacks in CLAUDE.md
    update_active_stacks_list "$stack_name"
    
    # Create natural language command interface
    create_stack_commands "$stack_name"
}

update_active_stacks_list() {
    local new_stack="$1"
    local claude_md="$CURRENT_PROJECT_DIR/.claude/CLAUDE.md"
    
    # Add stack to active stacks section
    sed -i "/## Active Stacks/a\\- $new_stack: $(grep -m1 "^# Description:" "$CACHE_DIR/stacks/$new_stack/CLAUDE.md" 2>/dev/null | cut -d: -f2- | xargs)" "$claude_md"
}

create_stack_commands() {
    local stack_name="$1"
    local commands_dir="$CURRENT_PROJECT_DIR/.claude/commands"
    
    # Check MCP requirements for this stack
    if command -v mcp-checker.sh &> /dev/null; then
        echo "  Checking MCP requirements..."
        mcp-checker.sh check "$stack_name" 2>/dev/null || true
    fi
    
    # Create natural language interface command
    cat > "$commands_dir/${stack_name}-interface.md" << EOF
# $stack_name Natural Language Interface

This command enables natural language interaction with $stack_name.

## Usage
Users can simply describe what they want to do in natural language:

\`\`\`
claude "I want to use $stack_name to help with my project"
claude "Apply $stack_name workflow to fix issues"
\`\`\`

## Implementation
The system uses headless Claude Code to interpret natural language requests and execute appropriate $stack_name workflows.
EOF
}

check_mcp_requirements() {
    local stack_name="$1"
    
    # Define MCP requirements for each stack
    case "$stack_name" in
        stack-4|stack-5)
            check_single_mcp "github" "GitHub integration (optional for advanced features)"
            ;;
        stack-7)
            check_single_mcp "postgres" "PostgreSQL database integration"
            check_single_mcp "redis" "Redis cache integration"
            ;;
        *)
            echo "    No special MCP requirements"
            ;;
    esac
}

check_single_mcp() {
    local mcp_name="$1"
    local description="$2"
    
    if command -v claude &> /dev/null; then
        if claude mcp list 2>/dev/null | grep -q "$mcp_name"; then
            echo "    ✓ $mcp_name MCP is available"
        else
            echo -e "    ${YELLOW}⚠${NC}  $mcp_name MCP not found ($description)"
            show_mcp_setup_hint "$mcp_name"
        fi
    else
        echo -e "    ${YELLOW}⚠${NC}  Claude Code not found - cannot check MCP availability"
    fi
}

show_mcp_setup_hint() {
    local mcp_name="$1"
    
    case "$mcp_name" in
        postgres)
            echo "      Setup: claude mcp add postgres -- npx -y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/db"
            ;;
        redis)
            echo "      Setup: claude mcp add redis -- docker run -i --rm mcp/redis redis://host.docker.internal:6379"
            ;;
        github)
            echo "      Setup: See https://github.com/github/github-mcp-server/blob/main/docs/installation-guides/install-claude.md"
            ;;
    esac
}

show_project_status() {
    if [[ ! -d "$CURRENT_PROJECT_DIR/.claude/stacks" ]]; then
        echo -e "${YELLOW}No stacks configured in current project${NC}"
        echo "Run 'stacks' to select and configure stacks"
        return
    fi
    
    echo -e "${GREEN}Active stacks in current project:${NC}"
    for stack_dir in "$CURRENT_PROJECT_DIR/.claude/stacks"/stack-*; do
        if [[ -d "$stack_dir" ]]; then
            local stack_name=$(basename "$stack_dir")
            echo "  ✓ $stack_name"
        fi
    done
    
    echo ""
    echo -e "${BLUE}Checking MCP requirements:${NC}"
    for stack_dir in "$CURRENT_PROJECT_DIR/.claude/stacks"/stack-*; do
        if [[ -d "$stack_dir" ]]; then
            local stack_name=$(basename "$stack_dir")
            echo "  $stack_name:"
            check_mcp_requirements "$stack_name"
        fi
    done
    
    echo ""
    echo -e "${BLUE}Available via natural language:${NC}"
    echo "  claude 'help me with this project'"
    echo "  claude 'fix any issues you find'"
    echo "  claude 'apply best practices'"
}

contribute_changes() {
    echo -e "${BLUE}Checking for local stack modifications...${NC}"
    
    # This will be enhanced to detect and contribute changes back
    # For now, provide guidance
    echo -e "${YELLOW}Contribution workflow coming soon!${NC}"
    echo "For now, manually create PRs to: $REPO_URL"
}

# Main command processing
case "${1:-}" in
    list)
        list_stacks
        ;;
    status)
        show_project_status
        ;;
    update)
        update_cache
        ;;
    contribute)
        contribute_changes
        ;;
    help|--help|-h)
        show_help
        ;;
    "")
        interactive_selection
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        echo "Run 'stacks help' for usage information"
        exit 1
        ;;
esac
EOF

    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    echo "  ✓ Installed $BINARY_NAME to $INSTALL_DIR"
}

update_shell_profile() {
    local shell_profile=""
    
    # Detect shell and set appropriate profile file
    if [[ -n "$ZSH_VERSION" ]] || [[ "$SHELL" == *"zsh"* ]]; then
        shell_profile="$HOME/.zshrc"
    elif [[ -n "$BASH_VERSION" ]] || [[ "$SHELL" == *"bash"* ]]; then
        if [[ -f "$HOME/.bash_profile" ]]; then
            shell_profile="$HOME/.bash_profile"
        else
            shell_profile="$HOME/.bashrc"
        fi
    fi
    
    if [[ -n "$shell_profile" ]]; then
        # Check if PATH already includes our install directory
        if ! grep -q "$INSTALL_DIR" "$shell_profile" 2>/dev/null; then
            echo "" >> "$shell_profile"
            echo "# Claude Code Stacks" >> "$shell_profile"
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_profile"
            echo "  ✓ Added $INSTALL_DIR to PATH in $shell_profile"
            echo -e "${YELLOW}  Please run: source $shell_profile${NC}"
        else
            echo "  ✓ PATH already configured"
        fi
    fi
}

main() {
    print_header
    
    echo -e "${BLUE}Checking system requirements...${NC}"
    check_dependencies
    echo "  ✓ All dependencies satisfied"
    echo ""
    
    setup_directories
    echo ""
    
    install_stacks_binary
    echo ""
    
    echo -e "${BLUE}Updating shell configuration...${NC}"
    update_shell_profile
    echo ""
    
    echo -e "${GREEN}Installation complete! 🎉${NC}"
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Restart your terminal or run: export PATH=\"$INSTALL_DIR:\$PATH\""
    echo "  2. Navigate to any project directory"  
    echo "  3. Run: ${YELLOW}stacks${NC}"
    echo "  4. Select stacks and start using natural language with Claude Code!"
    echo ""
    echo -e "${BLUE}Examples:${NC}"
    echo "  ${YELLOW}stacks${NC}                    # Interactive stack selection"
    echo "  ${YELLOW}stacks list${NC}               # List available stacks"
    echo "  ${YELLOW}claude 'help with this project'${NC} # Natural language interface"
    echo ""
}

# Run main function
main "$@"