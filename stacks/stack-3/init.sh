#!/bin/bash

# Stack-3 Initialization: Clark Idiomatic Style Formatting

echo "Initializing Stack-3: Clark Idiomatic Style Formatting"

# Create style enforcement scripts
create_style_scripts() {
    cat > apply-clark-style.sh << 'EOF'
#!/bin/bash
# Apply Clark's idiomatic style guidelines

echo "Applying Clark idiomatic style..."

# Remove emojis from all text files
echo "  Removing emojis from documentation and code..."
find . -type f \( -name "*.md" -o -name "*.txt" -o -name "*.py" -o -name "*.js" -o -name "*.ts" -o -name "*.rs" -o -name "*.go" \) \
  -not -path "./node_modules/*" -not -path "./.git/*" -not -path "./target/*" \
  -exec sed -i 's/[😀-🙏]//g' {} \;

# Check for package manager standardization needs
if [[ -f "package.json" ]] && [[ ! -f "bun.lockb" ]]; then
    echo "  Project uses npm/yarn - consider migrating to bun"
    echo "    Run: bun install (after installing bun)"
fi

if [[ -f "requirements.txt" ]] || [[ -f "setup.py" ]] && [[ ! -f "pyproject.toml" ]]; then
    echo "  Project uses old Python packaging - consider migrating to uv"
    echo "    Run: uv init && uv add $(cat requirements.txt | tr '\n' ' ')"
fi

echo "Style application complete"
EOF

    cat > check-style.sh << 'EOF'
#!/bin/bash
# Check for Clark style violations

echo "Checking for style violations..."

violations=0

# Check for emojis
emoji_files=$(find . -type f \( -name "*.md" -o -name "*.txt" -o -name "*.py" -o -name "*.js" -o -name "*.ts" \) \
  -not -path "./node_modules/*" -not -path "./.git/*" \
  -exec grep -l '[😀-🙏]' {} \; 2>/dev/null)

if [[ -n "$emoji_files" ]]; then
    echo "  ❌ Found emojis in files:"
    echo "$emoji_files" | sed 's/^/    /'
    ((violations++))
fi

# Check README length
for readme in $(find . -maxdepth 2 -name "README*" -type f); do
    lines=$(wc -l < "$readme")
    if [[ $lines -gt 200 ]]; then
        echo "  ❌ $readme is too long ($lines lines, should be <200)"
        ((violations++))
    fi
done

# Check package managers
if [[ -f "package.json" ]] && [[ -f "package-lock.json" ]] && [[ ! -f "bun.lockb" ]]; then
    echo "  ⚠️  Using npm instead of bun"
    ((violations++))
fi

if [[ -f "yarn.lock" ]] && [[ ! -f "bun.lockb" ]]; then
    echo "  ⚠️  Using yarn instead of bun"
    ((violations++))
fi

if [[ -f "requirements.txt" ]] && [[ ! -f "pyproject.toml" ]]; then
    echo "  ⚠️  Using pip instead of uv"
    ((violations++))
fi

if [[ $violations -eq 0 ]]; then
    echo "  ✅ No style violations found"
else
    echo "  Found $violations style violations"
fi

exit $violations
EOF

    cat > optimize-readme.sh << 'EOF'
#!/bin/bash
# Optimize README files for conciseness

echo "Optimizing README files..."

for readme in $(find . -maxdepth 2 -name "README*" -type f); do
    echo "  Processing $readme"
    
    # Create backup
    cp "$readme" "$readme.backup"
    
    # Remove excessive whitespace
    sed -i '/^$/N;/^\n$/d' "$readme"
    
    # Flag potential optimizations
    if grep -q "## Features" "$readme"; then
        echo "    Suggestion: Consider condensing Features section"
    fi
    
    if grep -q "## Installation" "$readme" && [[ $(grep -A 20 "## Installation" "$readme" | wc -l) -gt 10 ]]; then
        echo "    Suggestion: Simplify Installation section"
    fi
    
    lines_after=$(wc -l < "$readme")
    echo "    Processed: $lines_after lines"
done

echo "README optimization complete"
EOF

    cat > standardize-deps.sh << 'EOF'
#!/bin/bash
# Standardize to Clark's preferred package managers

echo "Standardizing package managers..."

# Convert to bun if Node.js project
if [[ -f "package.json" ]]; then
    if command -v bun &> /dev/null; then
        if [[ ! -f "bun.lockb" ]]; then
            echo "  Converting to bun..."
            rm -f package-lock.json yarn.lock
            bun install
        else
            echo "  Already using bun"
        fi
    else
        echo "  Install bun to standardize JavaScript package management"
        echo "    curl -fsSL https://bun.sh/install | bash"
    fi
fi

# Convert to uv if Python project
if [[ -f "requirements.txt" ]] || [[ -f "setup.py" ]]; then
    if command -v uv &> /dev/null; then
        if [[ ! -f "pyproject.toml" ]]; then
            echo "  Converting to uv..."
            uv init --no-readme
            if [[ -f "requirements.txt" ]]; then
                while IFS= read -r package; do
                    [[ -n "$package" ]] && [[ ! "$package" =~ ^# ]] && uv add "$package"
                done < requirements.txt
            fi
        else
            echo "  Already using modern Python packaging"
        fi
    else
        echo "  Install uv to standardize Python package management"
        echo "    curl -LsSf https://astral.sh/uv/install.sh | sh"
    fi
fi

echo "Package manager standardization complete"
EOF

    chmod +x apply-clark-style.sh check-style.sh optimize-readme.sh standardize-deps.sh
    echo "  Created style scripts: apply-clark-style.sh, check-style.sh, optimize-readme.sh, standardize-deps.sh"
}

# Main execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    TARGET_DIR="${1:-$(pwd)}"
    
    create_style_scripts
    
    echo ""
    echo "Stack-3 (Clark Style) initialized and ready for Claude Code"
    echo "Available commands:"
    echo "  ./apply-clark-style.sh - Apply all style guidelines"
    echo "  ./check-style.sh       - Check for violations"
    echo "  ./optimize-readme.sh   - Optimize README files"
    echo "  ./standardize-deps.sh  - Standardize package managers"
fi