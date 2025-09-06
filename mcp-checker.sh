#!/bin/bash

# Claude Code Stacks - MCP Requirements Checker
# Validates MCP availability and provides setup instructions

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

show_help() {
    cat << EOF
Claude Code MCP Requirements Checker

Usage: mcp-checker.sh [command] [stack-name]

Commands:
    check <stack>     Check MCP requirements for specific stack
    check-all         Check requirements for all active stacks
    install <mcp>     Show installation instructions for specific MCP
    list              List all available MCP integrations

Examples:
    mcp-checker.sh check stack-7
    mcp-checker.sh install postgres
    mcp-checker.sh check-all

EOF
}

# MCP definitions with setup instructions
declare -A MCP_SETUPS
MCP_SETUPS[postgres]="claude mcp add postgres -- npx -y @modelcontextprotocol/server-postgres postgresql://postgres:password@localhost:5432/postgres"
MCP_SETUPS[redis]="claude mcp add redis -- docker run -i --rm mcp/redis redis://host.docker.internal:6379"
MCP_SETUPS[sentry]="claude mcp add --transport http sentry https://mcp.sentry.dev/mcp"
MCP_SETUPS[jam]="claude mcp add --transport http jam https://mcp.jam.dev/mcp"
MCP_SETUPS[github]="# See: https://github.com/github/github-mcp-server/blob/main/docs/installation-guides/install-claude.md"

# Stack MCP requirements
declare -A STACK_MCPS
STACK_MCPS[stack-1]=""  # No special MCPs needed
STACK_MCPS[stack-2]=""  # Uses Docker directly
STACK_MCPS[stack-3]=""  # No special MCPs needed
STACK_MCPS[stack-4]="github"  # Optional for advanced git operations
STACK_MCPS[stack-5]="github"  # For CI/CD integration
STACK_MCPS[stack-6]=""  # Google Drive integration (custom)
STACK_MCPS[stack-7]="postgres redis"  # Common database MCPs

check_claude_available() {
    if ! command -v claude &> /dev/null; then
        echo -e "${RED}Error: Claude Code not found${NC}"
        echo "Please install Claude Code first: https://claude.ai/code"
        return 1
    fi
    
    echo -e "${GREEN}✓ Claude Code is available${NC}"
    return 0
}

get_installed_mcps() {
    if ! claude mcp list &> /dev/null; then
        echo ""
        return
    fi
    
    claude mcp list 2>/dev/null | grep -E "^\s*[a-z]" | awk '{print $1}' || echo ""
}

check_mcp_installed() {
    local mcp_name="$1"
    local installed_mcps="$(get_installed_mcps)"
    
    if [[ "$installed_mcps" =~ $mcp_name ]]; then
        return 0
    else
        return 1
    fi
}

show_mcp_setup() {
    local mcp_name="$1"
    
    echo -e "${BLUE}Setup instructions for $mcp_name MCP:${NC}"
    echo ""
    
    case "$mcp_name" in
        postgres)
            echo -e "${YELLOW}PostgreSQL MCP Setup:${NC}"
            echo "1. Start PostgreSQL (if not running):"
            echo "   docker run --name postgres-db -e POSTGRES_PASSWORD=mypassword -p 5432:5432 -d postgres:15"
            echo ""
            echo "2. Add MCP integration:"
            echo "   ${MCP_SETUPS[postgres]}"
            echo ""
            echo -e "${YELLOW}Note:${NC} Replace connection string with your actual database credentials"
            ;;
        redis)
            echo -e "${YELLOW}Redis MCP Setup:${NC}"
            echo "1. Start Redis (if not running):"
            echo "   docker run --name redis-cache -p 6379:6379 -d redis:7-alpine"
            echo ""
            echo "2. Add MCP integration:"
            echo "   ${MCP_SETUPS[redis]}"
            ;;
        github)
            echo -e "${YELLOW}GitHub MCP Setup:${NC}"
            echo "1. Create GitHub Personal Access Token:"
            echo "   - Go to GitHub Settings > Developer Settings > Personal Access Tokens"
            echo "   - Generate token with repo, issues, and pull requests permissions"
            echo ""
            echo "2. Install GitHub MCP:"
            echo "   ${MCP_SETUPS[github]}"
            echo ""
            echo -e "${YELLOW}Note:${NC} Requires GITHUB_TOKEN environment variable"
            ;;
        sentry)
            echo -e "${YELLOW}Sentry MCP Setup:${NC}"
            echo "1. Get Sentry Auth Token from your Sentry account"
            echo "2. Add MCP integration:"
            echo "   ${MCP_SETUPS[sentry]}"
            echo ""
            echo -e "${YELLOW}Note:${NC} Requires SENTRY_AUTH_TOKEN environment variable"
            ;;
        jam)
            echo -e "${YELLOW}Jam MCP Setup:${NC}"
            echo "1. Sign up for Jam account if needed"
            echo "2. Add MCP integration:"
            echo "   ${MCP_SETUPS[jam]}"
            ;;
        *)
            echo -e "${RED}Unknown MCP: $mcp_name${NC}"
            ;;
    esac
    echo ""
}

check_stack_mcps() {
    local stack_name="$1"
    
    if [[ -z "${STACK_MCPS[$stack_name]}" ]]; then
        echo -e "${GREEN}✓ $stack_name: No special MCP requirements${NC}"
        return 0
    fi
    
    local required_mcps="${STACK_MCPS[$stack_name]}"
    local missing_mcps=()
    local available_mcps=()
    
    echo -e "${BLUE}Checking MCP requirements for $stack_name...${NC}"
    
    for mcp in $required_mcps; do
        if check_mcp_installed "$mcp"; then
            available_mcps+=("$mcp")
            echo -e "  ${GREEN}✓${NC} $mcp MCP is available"
        else
            missing_mcps+=("$mcp")
            echo -e "  ${YELLOW}⚠${NC}  $mcp MCP is missing"
        fi
    done
    
    if [[ ${#missing_mcps[@]} -gt 0 ]]; then
        echo ""
        echo -e "${YELLOW}Missing MCPs for $stack_name:${NC}"
        for mcp in "${missing_mcps[@]}"; do
            echo -e "  - $mcp (run: ${BLUE}mcp-checker.sh install $mcp${NC})"
        done
        echo ""
        echo -e "${YELLOW}$stack_name will work with limited functionality without these MCPs.${NC}"
        return 1
    else
        echo -e "${GREEN}✓ All MCP requirements satisfied for $stack_name${NC}"
        return 0
    fi
}

check_all_stacks() {
    local project_dir="$(pwd)"
    local claude_dir="$project_dir/.claude"
    
    if [[ ! -d "$claude_dir/stacks" ]]; then
        echo -e "${YELLOW}No stacks configured in current project${NC}"
        echo "Run 'stacks' to configure your project first"
        return 1
    fi
    
    echo -e "${BLUE}Checking MCP requirements for all active stacks...${NC}"
    echo ""
    
    local total_issues=0
    
    for stack_dir in "$claude_dir/stacks"/stack-*; do
        if [[ -d "$stack_dir" ]]; then
            local stack_name=$(basename "$stack_dir")
            check_stack_mcps "$stack_name"
            if [[ $? -ne 0 ]]; then
                ((total_issues++))
            fi
            echo ""
        fi
    done
    
    if [[ $total_issues -eq 0 ]]; then
        echo -e "${GREEN}🎉 All MCP requirements satisfied!${NC}"
    else
        echo -e "${YELLOW}Found $total_issues stacks with missing MCPs${NC}"
        echo "Run setup commands above to enable full functionality"
    fi
}

list_available_mcps() {
    echo -e "${BLUE}Available MCP Integrations:${NC}"
    echo ""
    echo -e "${GREEN}Database MCPs:${NC}"
    echo "  - postgres    : PostgreSQL database integration"
    echo "  - redis       : Redis cache and pub/sub"
    echo ""
    echo -e "${GREEN}Development MCPs:${NC}"  
    echo "  - github      : GitHub repository integration"
    echo "  - sentry      : Error monitoring and tracking"
    echo "  - jam         : Screenshot and bug reporting"
    echo ""
    echo -e "${BLUE}Usage:${NC}"
    echo "  mcp-checker.sh install <mcp-name>    # Show setup instructions"
    echo "  mcp-checker.sh check <stack-name>    # Check stack requirements"
}

# Main command processing
case "${1:-}" in
    check)
        check_claude_available || exit 1
        if [[ -n "$2" ]]; then
            check_stack_mcps "$2"
        else
            echo "Usage: mcp-checker.sh check <stack-name>"
            exit 1
        fi
        ;;
    check-all)
        check_claude_available || exit 1
        check_all_stacks
        ;;
    install)
        if [[ -n "$2" ]]; then
            show_mcp_setup "$2"
        else
            echo "Usage: mcp-checker.sh install <mcp-name>"
            list_available_mcps
            exit 1
        fi
        ;;
    list)
        list_available_mcps
        ;;
    help|--help|-h)
        show_help
        ;;
    "")
        echo -e "${BLUE}Claude Code MCP Requirements Checker${NC}"
        echo ""
        echo "What would you like to do?"
        echo "  1. Check all active stacks"
        echo "  2. Check specific stack"
        echo "  3. List available MCPs"
        echo "  4. Show setup instructions"
        echo ""
        read -p "Choose (1-4): " choice
        case $choice in
            1) check_claude_available && check_all_stacks ;;
            2) 
                read -p "Enter stack name (e.g., stack-7): " stack_name
                check_claude_available && check_stack_mcps "$stack_name"
                ;;
            3) list_available_mcps ;;
            4) 
                read -p "Enter MCP name (e.g., postgres): " mcp_name
                show_mcp_setup "$mcp_name"
                ;;
            *) echo "Invalid choice" ;;
        esac
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        show_help
        exit 1
        ;;
esac