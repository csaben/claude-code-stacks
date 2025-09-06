# Stack Recommend Command

## Description
Get personalized stack recommendations based on your project type, existing files, and development needs.

## Usage
```
/stack-recommend [project-type]
```

## Parameters
- `project-type` (optional): Specify your project type (js, ts, react, node, docker, fullstack, infra)

## How It Works

This command analyzes your project and provides tailored stack recommendations by examining:
1. **File structure** - What files and directories exist
2. **Package.json** - Dependencies and project type indicators  
3. **Docker files** - Container usage patterns
4. **Existing stacks** - What's already installed
5. **Project context** - Based on your description

## Project Type Detection & Recommendations

### 🟨 JavaScript/TypeScript Projects

**Auto-detected by**:
- `package.json` presence
- `.ts`, `.tsx`, `.js`, `.jsx` files
- `tsconfig.json` or similar configs

**Recommended stacks**:
- ✅ **ts-lint-stack** - Essential for code quality
- ✅ **stackstack** - For workflow management

**Reasoning**: Code quality is crucial for JS/TS projects. Consistent linting prevents bugs and improves maintainability.

---

### ⚛️ React/Frontend Projects

**Auto-detected by**:
- React dependencies in package.json
- `src/` directory with components
- Frontend-specific tooling (Vite, Webpack, etc.)

**Recommended stacks**:
- ✅ **ts-lint-stack** - Code quality and consistency  
- ✅ **stackstack** - Workflow management
- 🔄 **Consider stack-2** - If using Docker for deployment

**Reasoning**: Frontend projects benefit greatly from linting. Infrastructure testing helps if containerized.

---

### 🐳 Docker/Infrastructure Projects

**Auto-detected by**:
- `Dockerfile` present
- `docker-compose.yml` files
- Infrastructure config files

**Recommended stacks**:
- ✅ **stack-2** - Container testing and validation
- ✅ **stackstack** - Management and workflows
- 🔄 **Consider ts-lint-stack** - If also writing TypeScript

**Reasoning**: Infrastructure projects need validation and testing. Docker configs can be complex and error-prone.

---

### 🌐 Full-Stack Applications

**Auto-detected by**:
- Both frontend and backend code present
- Multiple package.json files
- Database configurations
- API and client directories

**Recommended stacks**:
- ✅ **ts-lint-stack** - Frontend/backend code quality
- ✅ **stack-2** - Infrastructure and deployment testing  
- ✅ **stackstack** - Essential for managing complexity

**Reasoning**: Full-stack projects have the most complexity and benefit from comprehensive tooling.

---

### 🔧 DevOps/Infrastructure Projects

**Auto-detected by**:
- Multiple Docker configurations
- CI/CD files (.github/workflows, etc.)
- Infrastructure-as-code files
- Deployment scripts

**Recommended stacks**:
- ✅ **stack-2** - Infrastructure validation priority
- ✅ **stackstack** - Essential for workflow management

**Reasoning**: Focus on infrastructure reliability and testing. Code quality less critical for config files.

---

### 🆕 New/Empty Projects

**Auto-detected by**:
- Minimal or no files
- No clear project type indicators

**Recommended approach**:
1. ✅ **Start with stackstack** - Learn the system first
2. 🔄 **Add others later** - As project develops
3. 💡 **Use `/stack-browse`** - To explore options

**Reasoning**: Better to start simple and add complexity as needed.

## Contextual Recommendations

### If you already have stacks:

#### Currently have: ts-lint-stack only
**Recommendation**: Add `stackstack` for better management
**Why**: You're missing workflow automation and validation tools

#### Currently have: stack-2 only  
**Recommendation**: Add `stackstack` + consider `ts-lint-stack`
**Why**: Need workflow management; code quality if writing TypeScript

#### Currently have: stackstack only
**Recommendation**: Perfect starting point! Add others based on project needs
**Why**: You have management tools, now add functionality stacks

## Smart Combinations

### 🎯 Recommended Combinations

**Minimal Setup** (good for learning):
- `stackstack` only

**TypeScript Focus**:
- `ts-lint-stack` + `stackstack`

**Infrastructure Focus**:  
- `stack-2` + `stackstack`

**Maximum Coverage** (full-stack projects):
- `ts-lint-stack` + `stack-2` + `stackstack`

## Step-by-Step Recommendation Process

### 1. Run Analysis
The command examines your project structure and provides a detailed report.

### 2. Get Personalized Recommendations  
Based on analysis, you'll get specific stack suggestions with reasoning.

### 3. Implementation Guidance
Clear next steps for adding the recommended stacks.

### 4. Follow-up Actions
What to do after installing the recommended stacks.

## Example Recommendations

### For a React + TypeScript project:
```
🔍 Project Analysis Complete!

Detected:
- TypeScript React application  
- Using Vite for building
- No Docker configuration
- No existing stacks

💡 Recommendations:
✅ ts-lint-stack - Essential for TS/React code quality
✅ stackstack - Workflow management and guidance

🎯 Priority: Start with these 2 stacks
📝 Next: Run `stacks` and select both recommended stacks
```

### For a Docker-based project:
```
🔍 Project Analysis Complete!

Detected:
- Docker Compose configuration
- Multiple services defined
- nginx configuration files
- No existing stacks

💡 Recommendations:  
✅ stack-2 - Docker and nginx testing/validation
✅ stackstack - Essential for managing stack workflows

🎯 Priority: Infrastructure testing is crucial
📝 Next: Focus on validating your Docker setup
```

## Using the Recommendations

After getting recommendations:

1. **Review the reasoning** - Understand why stacks were suggested
2. **Run `stacks`** - Start the selection process  
3. **Select recommended stacks** - Follow the guidance
4. **Commit changes** - Use `/stack-commit` after checkout
5. **Validate setup** - Run `/stack-validate` to ensure everything works

## Advanced Usage

### Force specific recommendations:
```bash
/stack-recommend fullstack  # Full-stack project guidance
/stack-recommend js         # JavaScript-focused recommendations  
/stack-recommend infra      # Infrastructure-focused suggestions
```

### Re-analyze after changes:
```bash
/stack-recommend
```
Run this again after adding files or changing project structure.

## Getting More Help

- `/stack-help` - Complete stacks documentation
- `/stack-browse` - Detailed information about each stack
- `/stack-status` - See what's currently installed
- `/stack-validate` - Check if current setup is healthy

---
*Smart recommendations help you choose the right tools for your project. Let the analysis guide your stack selection!*