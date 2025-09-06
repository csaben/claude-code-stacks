#!/bin/bash

# Claude Code Stacks - Tmux Integration Manager
# Multi-pane workflow monitoring and management

set -e

SESSION_NAME="claude-stacks"
PROJECT_ROOT="$(pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

show_help() {
    cat << EOF
Claude Code Stacks - Tmux Integration

Usage: tmux-stack-manager.sh [command]

Commands:
    start              Start tmux session with active stacks
    status             Show current session status
    monitor <stack>    Monitor specific stack in dedicated pane
    stop               Stop the tmux session
    attach             Attach to existing session

Natural Language Interface:
    "Start monitoring my stacks"
    "Show me what's happening with the linting stack"
    "Open tmux workspace for this project"

EOF
}

detect_active_stacks() {
    local claude_dir="$PROJECT_ROOT/.claude"
    local active_stacks=()
    
    if [[ -d "$claude_dir/stacks" ]]; then
        for stack_dir in "$claude_dir/stacks"/stack-*; do
            if [[ -d "$stack_dir" ]]; then
                active_stacks+=($(basename "$stack_dir"))
            fi
        done
    fi
    
    echo "${active_stacks[@]}"
}

create_tmux_session() {
    echo -e "${BLUE}Creating Claude Code Stacks tmux session...${NC}"
    
    # Kill existing session if it exists
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        echo "  Existing session found, killing it..."
        tmux kill-session -t "$SESSION_NAME"
    fi
    
    # Create new session
    tmux new-session -d -s "$SESSION_NAME" -c "$PROJECT_ROOT"
    tmux rename-window -t "$SESSION_NAME:0" "main"
    
    # Send welcome message to main window
    tmux send-keys -t "$SESSION_NAME:main" "echo 'Claude Code Stacks Session'" Enter
    tmux send-keys -t "$SESSION_NAME:main" "echo 'Project: $PROJECT_ROOT'" Enter
    tmux send-keys -t "$SESSION_NAME:main" "echo 'Active stacks will appear in separate panes'" Enter
    tmux send-keys -t "$SESSION_NAME:main" "echo ''" Enter
    
    echo -e "  ${GREEN}✓${NC} Created session: $SESSION_NAME"
}

setup_stack_pane() {
    local stack_name="$1"
    local stack_dir="$PROJECT_ROOT/.claude/stacks/$stack_name"
    
    if [[ ! -d "$stack_dir" ]]; then
        echo -e "${YELLOW}Warning: Stack $stack_name not found${NC}"
        return 1
    fi
    
    echo -e "${BLUE}Setting up pane for $stack_name...${NC}"
    
    # Create new window for this stack
    tmux new-window -t "$SESSION_NAME" -n "$stack_name" -c "$stack_dir"
    
    # Set up the pane with stack-specific monitoring
    case "$stack_name" in
        "stack-1")
            setup_linting_pane "$stack_name"
            ;;
        "stack-2")
            setup_testing_pane "$stack_name"
            ;;
        "stack-3")
            setup_style_pane "$stack_name"
            ;;
        "stack-4")
            setup_git_pane "$stack_name"
            ;;
        "stack-5")
            setup_cicd_pane "$stack_name"
            ;;
        "stack-6")
            setup_docs_pane "$stack_name"
            ;;
        "stack-7")
            setup_database_pane "$stack_name"
            ;;
        *)
            setup_generic_pane "$stack_name"
            ;;
    esac
    
    echo -e "  ${GREEN}✓${NC} $stack_name pane ready"
}

setup_linting_pane() {
    local stack_name="$1"
    
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '🔍 Linting Stack Monitor'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Monitoring: $PROJECT_ROOT'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Available commands:'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '  - Run linter: npm run lint OR ruff check OR cargo clippy'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '  - Auto-fix: npm run lint:fix OR ruff format'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '  - Natural language: claude \"fix linting issues\"'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    
    # Set up file watcher for automatic linting
    if command -v inotifywait &> /dev/null; then
        tmux send-keys -t "$SESSION_NAME:$stack_name" "# Starting file watcher for auto-linting..." Enter
        # This would set up background file watching in a real implementation
    fi
}

setup_testing_pane() {
    local stack_name="$1"
    
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '🧪 Testing Stack Monitor'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Monitoring: Docker configs, nginx, examples'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Available commands:'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '  - Test Docker: docker-compose config'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '  - Test nginx: nginx -t'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '  - Natural language: claude \"test my configurations\"'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
}

setup_style_pane() {
    local stack_name="$1"
    
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '✨ Clark Style Monitor'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Monitoring: Style compliance'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Style rules:'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '  ❌ No emojis in code/docs'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '  📝 Concise READMEs (<200 lines)'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '  📦 Use uv for Python, bun for JS'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Natural language: claude \"apply Clark style\"'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
}

setup_git_pane() {
    local stack_name="$1"
    
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '📦 Git Operations Monitor'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Repository: $(git remote get-url origin 2>/dev/null || echo \"Local repo\")'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "git status" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Natural language: claude \"commit these changes\"'" Enter
}

setup_cicd_pane() {
    local stack_name="$1"
    
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '🚀 CI/CD Monitor'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Monitoring: Pipelines and deployments'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    
    # Check for CI/CD configs
    if [[ -d "$PROJECT_ROOT/.github/workflows" ]]; then
        tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'GitHub Actions detected'" Enter
        tmux send-keys -t "$SESSION_NAME:$stack_name" "ls -la $PROJECT_ROOT/.github/workflows/" Enter
    elif [[ -f "$PROJECT_ROOT/.gitlab-ci.yml" ]]; then
        tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'GitLab CI detected'" Enter
    else
        tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'No CI/CD configuration found'" Enter
    fi
}

setup_docs_pane() {
    local stack_name="$1"
    
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '📚 Documentation Monitor'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Monitoring: docs/ directory and design documents'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    
    if [[ -d "$PROJECT_ROOT/docs" ]]; then
        tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Documentation directory found:'" Enter
        tmux send-keys -t "$SESSION_NAME:$stack_name" "find $PROJECT_ROOT/docs -name '*.md' | head -10" Enter
    else
        tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'No docs/ directory found'" Enter
    fi
}

setup_database_pane() {
    local stack_name="$1"
    
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '🗄️  Database Monitor'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Monitoring: Database configurations'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    
    # Check for database configs
    if [[ -f "$PROJECT_ROOT/docker-compose.yml" ]]; then
        tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Docker Compose configuration found:'" Enter
        tmux send-keys -t "$SESSION_NAME:$stack_name" "grep -i 'postgres\\|mongodb\\|redis\\|mysql' $PROJECT_ROOT/docker-compose.yml || echo 'No databases detected'" Enter
    else
        tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'No docker-compose.yml found'" Enter
    fi
}

setup_generic_pane() {
    local stack_name="$1"
    
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo '⚙️  $stack_name Monitor'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo 'Stack directory: $(pwd)'" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:$stack_name" "ls -la" Enter
}

start_monitoring_session() {
    echo -e "${BLUE}Starting Claude Code Stacks monitoring session...${NC}"
    
    # Create base session
    create_tmux_session
    
    # Get active stacks
    local active_stacks=($(detect_active_stacks))
    
    if [[ ${#active_stacks[@]} -eq 0 ]]; then
        echo -e "${YELLOW}No active stacks found in project${NC}"
        echo "Run 'stacks' first to configure your project"
        return 1
    fi
    
    echo -e "${BLUE}Setting up panes for ${#active_stacks[@]} stacks...${NC}"
    
    # Set up pane for each active stack
    for stack in "${active_stacks[@]}"; do
        setup_stack_pane "$stack"
    done
    
    # Create router/control pane
    tmux new-window -t "$SESSION_NAME" -n "control" -c "$PROJECT_ROOT"
    tmux send-keys -t "$SESSION_NAME:control" "echo '🎛️  Claude Code Stacks Control Panel'" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo 'Project: $PROJECT_ROOT'" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo 'Active stacks: ${active_stacks[*]}'" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo 'Commands:'" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo '  stacks                    - Manage stacks'" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo '  stack-contribute         - Contribute changes'" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo '  claude \"<request>\"       - Natural language interface'" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo 'Navigation:'" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo '  Ctrl+B, then arrow keys  - Switch between panes'" Enter
    tmux send-keys -t "$SESSION_NAME:control" "echo '  Ctrl+B, then w          - Window list'" Enter
    
    echo ""
    echo -e "${GREEN}Monitoring session started! 🎉${NC}"
    echo ""
    echo -e "${BLUE}To attach:${NC} tmux attach-session -t $SESSION_NAME"
    echo -e "${BLUE}Windows created:${NC}"
    tmux list-windows -t "$SESSION_NAME"
}

show_session_status() {
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        echo -e "${GREEN}Claude Code Stacks session is active${NC}"
        echo ""
        echo -e "${BLUE}Windows:${NC}"
        tmux list-windows -t "$SESSION_NAME"
        echo ""
        echo -e "${BLUE}To attach:${NC} tmux attach-session -t $SESSION_NAME"
    else
        echo -e "${YELLOW}No active monitoring session${NC}"
        echo "Run 'tmux-stack-manager.sh start' to create one"
    fi
}

stop_session() {
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        tmux kill-session -t "$SESSION_NAME"
        echo -e "${GREEN}Stopped monitoring session${NC}"
    else
        echo -e "${YELLOW}No active session to stop${NC}"
    fi
}

attach_session() {
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        tmux attach-session -t "$SESSION_NAME"
    else
        echo -e "${YELLOW}No active session found${NC}"
        echo "Run 'tmux-stack-manager.sh start' first"
    fi
}

# Main command processing
case "${1:-}" in
    start)
        start_monitoring_session
        ;;
    status)
        show_session_status
        ;;
    stop)
        stop_session
        ;;
    attach)
        attach_session
        ;;
    monitor)
        if [[ -n "$2" ]]; then
            setup_stack_pane "$2"
        else
            echo "Usage: tmux-stack-manager.sh monitor <stack-name>"
        fi
        ;;
    help|--help|-h)
        show_help
        ;;
    "")
        echo -e "${BLUE}Claude Code Stacks - Tmux Integration${NC}"
        echo ""
        echo "What would you like to do?"
        echo "  1. Start monitoring session"
        echo "  2. Check session status"  
        echo "  3. Attach to existing session"
        echo "  4. Stop session"
        echo ""
        read -p "Choose (1-4): " choice
        case $choice in
            1) start_monitoring_session ;;
            2) show_session_status ;;
            3) attach_session ;;
            4) stop_session ;;
            *) echo "Invalid choice" ;;
        esac
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        show_help
        exit 1
        ;;
esac