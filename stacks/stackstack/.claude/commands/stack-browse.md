# Stack Browse Command

## Description
Interactive guide to browse and explore available stacks with detailed information and usage recommendations.

## Usage
```
/stack-browse [filter]
```

## Parameters
- `filter` (optional): Filter stacks by category (js, infra, testing, all)

## What This Command Does

Instead of just running `stacks` directly, this command provides:
1. **Detailed stack information** with explanations
2. **Usage recommendations** based on project context
3. **Interactive guidance** for stack selection
4. **Compatibility information** between stacks

## Available Stacks Guide

### 🔧 ts-lint-stack
**Category**: Code Quality / JavaScript/TypeScript  
**Maturity**: Stable  
**Dependencies**: None

**What it provides**:
- Comprehensive linting for TS/JS projects
- ESLint and Biome configuration  
- Auto-fixing capabilities
- Pre-commit hook integration
- Code style enforcement

**Best for**:
- Any TypeScript or JavaScript project
- Teams wanting consistent code quality
- Projects with multiple contributors
- CI/CD pipelines requiring lint checks

**Commands you'll get**:
- `/lint` - Run linting on current project
- `/fix-lint` - Auto-fix linting issues
- `/setup-eslint` - Configure ESLint rules
- `/setup-prettier` - Set up code formatting

**Works well with**: stackstack (for management), stack-2 (for full-stack projects)

---

### 📚 stackstack  
**Category**: Stack Management / Workflow Automation  
**Maturity**: Stable  
**Dependencies**: None

**What it provides**:
- Complete stacks usage documentation
- Automated git workflow management  
- Stack validation and health checking
- Comprehensive help system

**Best for**:
- **Every project** (recommended always)
- Users new to the stacks system
- Managing multiple stacks
- Ensuring proper git workflows

**Commands you'll get**:
- `/stack-help` - Complete stacks guide
- `/stack-commit` - Commit stack changes  
- `/stack-status` - Show all stack statuses
- `/stack-validate` - Validate configurations
- `/stack-browse` - This command!
- `/stack-recommend` - Get stack recommendations

**Works well with**: Any other stack (always recommended)

---

### 🧪 stack-2
**Category**: Infrastructure / Testing / DevOps  
**Maturity**: Stable  
**Dependencies**: Docker (optional)

**What it provides**:
- Docker and Docker Compose validation
- Nginx configuration testing
- Infrastructure example validation
- Container build and startup testing

**Best for**:
- Projects with Docker containers
- Web applications with nginx
- Infrastructure-as-code projects  
- DevOps and deployment workflows

**Commands you'll get**:
- `/test-docker` - Validate Docker configurations
- `/test-nginx` - Test nginx configurations
- `/validate-compose` - Check docker-compose files
- `/test-examples` - Run all infrastructure tests

**Works well with**: ts-lint-stack (for full-stack), stackstack (always recommended)

## Quick Selection Guide

### Tell me about your project:

#### "I'm building a TypeScript/React app"
**Recommended**: `ts-lint-stack` + `stackstack`
- ts-lint-stack: Code quality and consistency
- stackstack: Workflow management and guidance

#### "I'm working with Docker and containers"  
**Recommended**: `stack-2` + `stackstack`
- stack-2: Container validation and testing
- stackstack: Stack management

#### "Full-stack web application with frontend and backend"
**Recommended**: `ts-lint-stack` + `stack-2` + `stackstack`
- ts-lint-stack: Frontend code quality
- stack-2: Backend/infrastructure testing  
- stackstack: Overall management

#### "I'm new to this system"
**Start with**: `stackstack` only
- Learn the system first
- Add other stacks as needed
- Use `/stack-recommend` for suggestions

#### "I want everything / maximum capability"
**Power user combo**: `ts-lint-stack` + `stack-2` + `stackstack`
- Complete coverage for most project types
- All available tools and workflows

## Stack Compatibility Matrix

| Stack | ts-lint-stack | stack-2 | stackstack |
|-------|---------------|---------|-----------|
| ts-lint-stack | ➖ | ✅ Great | ✅ Essential |
| stack-2 | ✅ Great | ➖ | ✅ Essential |
| stackstack | ✅ Essential | ✅ Essential | ➖ |

- ✅ **Great**: Work excellently together
- ✅ **Essential**: Always recommended combination
- ➖ **N/A**: Same stack

## Getting Started

After browsing, here's how to proceed:

1. **Run the stacks command**:
   ```bash
   stacks
   ```

2. **Select your chosen stacks** using space bar

3. **Confirm and checkout** with Enter

4. **Commit the changes** (important!):
   ```bash
   /stack-commit
   ```

5. **Verify everything works**:
   ```bash
   /stack-validate
   ```

## Need Help Choosing?

Use these commands:
- `/stack-recommend` - Get personalized recommendations
- `/stack-help` - Complete stacks documentation
- `/stack-status` - See what's currently installed

## Pro Tips

- **Start small**: Begin with 1-2 stacks, add more later
- **Always include stackstack**: It helps manage everything else  
- **Match your project**: Choose stacks relevant to your tech stack
- **Commit immediately**: Use `/stack-commit` right after checkout
- **Validate regularly**: Use `/stack-validate` to catch issues

---
*Ready to browse stacks? Run `stacks` to start the selection process!*