#!/bin/bash

# Stack-2 Initialization: Automatic Testing Setup

echo "Initializing Stack-2: Automatic Testing"

# Check for Docker availability
check_docker() {
    if ! command -v docker &> /dev/null; then
        echo "  Warning: Docker not found. Install Docker to enable container testing."
        return 1
    fi
    
    if ! docker info &> /dev/null; then
        echo "  Warning: Docker daemon not running. Start Docker to enable testing."
        return 1
    fi
    
    echo "  Docker is available and running"
    return 0
}

# Check for Docker Compose
check_docker_compose() {
    if command -v docker-compose &> /dev/null; then
        echo "  Docker Compose (standalone) is available"
        return 0
    elif docker compose version &> /dev/null 2>&1; then
        echo "  Docker Compose (plugin) is available"
        return 0
    else
        echo "  Warning: Docker Compose not found"
        return 1
    fi
}

# Discover test targets
discover_test_targets() {
    local project_dir="${1:-$(pwd)}"
    
    echo "Discovering test targets in: $project_dir"
    
    # Find Docker Compose files
    local compose_files=($(find "$project_dir" -name "docker-compose*.yml" -o -name "docker-compose*.yaml" 2>/dev/null))
    if [[ ${#compose_files[@]} -gt 0 ]]; then
        echo "  Found Docker Compose files:"
        printf "    %s\n" "${compose_files[@]}"
    fi
    
    # Find Dockerfiles
    local dockerfiles=($(find "$project_dir" -name "Dockerfile*" 2>/dev/null))
    if [[ ${#dockerfiles[@]} -gt 0 ]]; then
        echo "  Found Dockerfiles:"
        printf "    %s\n" "${dockerfiles[@]}"
    fi
    
    # Find Nginx configs
    local nginx_configs=($(find "$project_dir" -name "nginx.conf" -o -name "*.nginx" -o -path "*/nginx/*" -name "*.conf" 2>/dev/null))
    if [[ ${#nginx_configs[@]} -gt 0 ]]; then
        echo "  Found Nginx configurations:"
        printf "    %s\n" "${nginx_configs[@]}"
    fi
    
    # Find example directories
    local example_dirs=($(find "$project_dir" -type d -name "example*" -o -name "demo*" 2>/dev/null))
    if [[ ${#example_dirs[@]} -gt 0 ]]; then
        echo "  Found example directories:"
        printf "    %s\n" "${example_dirs[@]}"
    fi
}

# Create testing utilities
create_test_scripts() {
    cat > test-docker.sh << 'EOF'
#!/bin/bash
# Test Docker configurations

echo "Testing Docker configurations..."

# Test Dockerfile syntax
for dockerfile in $(find . -name "Dockerfile*" 2>/dev/null); do
    echo "  Testing $dockerfile"
    if command -v hadolint &> /dev/null; then
        hadolint "$dockerfile"
    else
        echo "    Syntax check: docker build --no-cache -f $dockerfile ."
    fi
done

# Test Docker Compose files
for compose in $(find . -name "docker-compose*.yml" -o -name "docker-compose*.yaml" 2>/dev/null); do
    echo "  Testing $compose"
    docker-compose -f "$compose" config -q
done

echo "Docker configuration testing complete"
EOF

    cat > test-nginx.sh << 'EOF'
#!/bin/bash
# Test Nginx configurations

echo "Testing Nginx configurations..."

for config in $(find . -name "nginx.conf" -o -name "*.nginx" -o -path "*/nginx/*" -name "*.conf" 2>/dev/null); do
    echo "  Testing $config"
    # Use nginx container to test config syntax
    docker run --rm -v "$(realpath "$config")":/etc/nginx/nginx.conf:ro nginx:alpine nginx -t
done

echo "Nginx configuration testing complete"
EOF

    cat > test-examples.sh << 'EOF'
#!/bin/bash
# Test all examples

echo "Testing project examples..."

for example_dir in $(find . -type d -name "example*" -o -name "demo*" 2>/dev/null); do
    echo "  Testing example: $example_dir"
    
    if [[ -f "$example_dir/docker-compose.yml" ]]; then
        cd "$example_dir"
        docker-compose config -q && echo "    Docker Compose: OK"
        cd - > /dev/null
    fi
    
    if [[ -f "$example_dir/Dockerfile" ]]; then
        echo "    Found Dockerfile in $example_dir"
    fi
done

echo "Example testing complete"
EOF

    chmod +x test-docker.sh test-nginx.sh test-examples.sh
    echo "  Created testing scripts: test-docker.sh, test-nginx.sh, test-examples.sh"
}

# Main execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    TARGET_DIR="${1:-$(pwd)}"
    
    check_docker
    check_docker_compose
    discover_test_targets "$TARGET_DIR"
    create_test_scripts
    
    echo ""
    echo "Stack-2 (Testing) initialized and ready for Claude Code"
    echo "Available commands:"
    echo "  ./test-docker.sh   - Test Docker configurations"
    echo "  ./test-nginx.sh    - Test Nginx configurations"  
    echo "  ./test-examples.sh - Test all examples"
fi