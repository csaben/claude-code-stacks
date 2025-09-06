`stacks` is a cli tool for 
- checking out common workflows i use with claude (checkout a stack and symlink .claude files and copy settings into the cwd settings for claude)
- worktree setup (since im a noob with worktrees)
- syncing of mcp server info to match cwd (postgres servers etc)
- cmds: stacks, stacks worktree, stacks sync


See @stacks/stack-2/CLAUDE.md for additional stack instructions.


See @stacks/stack-1/CLAUDE.md for additional stack instructions.

## motivation
claude code configurations vary from project type to project type (i.e. uv python fastapi server vs react typescript). the goal is to quickly be able to checkout one of these `stack`'s that is setup as one would to solve the entire project type in the current working directory of another project using claude. this way you can combine the current stuff with the one hyper optimized `stack`. another main goal is that every project should improve the `stacks` used. so if i make a change to a stack i checked out, it should be super easy to push the change back to the source repo so next time i use it in a project i pick up right where i left off. 


## things that stacks provide
- .claude/ contents (agents, commands, .local-settings.json)
- symlink the agents and commands s.t. claude see thems with slash commands
- add .local-settings.json to the cwd .claude/.local-settings.json
- check if any mcp allowances require mcp servers not already installed. install ones that don't require API keys automatically and note it but provide bash install command for ones that need a key.

samples
```
# postgres
claude mcp add postgres -- npx -y @modelcontextprotocol/server-postgres postgresql://localhost/your_database

# redis
claude mcp add redis -- docker run -i --rm mcp/redis redis://host.docker.internal:6379

# sentry
claude mcp add --transport http sentry https://mcp.sentry.dev/mcp

# jam
claude mcp add --transport http jam https://mcp.jam.dev/mcp

# github
# https://github.com/github/github-mcp-server/blob/main/docs/installation-guides/install-claude.md

# postgres example

# If your PostgreSQL is running in Docker
docker run --name postgres-db -e POSTGRES_PASSWORD=mypassword -p 5432:5432 -v postgres-data:/var/lib/postgresql/data postgres:15

# Connect Claude Code to it
claude mcp add postgres -- npx -y @modelcontextprotocol/server-postgres postgresql://postgres:mypassword@localhost:5432/postgres

```

## setup
using stacks should be similar to how `uv` by astral is installed. via a simple curl and bash. i.e.
- One-liner curl installation: `curl -sSL https://raw.githubusercontent.com/csaben/claude-code-stacks/main/install.sh | bash`
- Creates binary at `/home/user/.local/bin/stacks`

## dependencies
- tmux
- claude
- fzf

## reference for how to use claude and organize .claude/ and files within
- agents: https://docs.anthropic.com/en/docs/claude-code/sub-agents#subagent-configuration
- settings: https://docs.anthropic.com/en/docs/claude-code/settings#available-settings
- common workflow: https://docs.anthropic.com/en/docs/claude-code/common-workflows
- mcp: https://docs.anthropic.com/en/docs/claude-code/mcp#use-mcp-resources 
- quickstart: https://docs.anthropic.com/en/docs/claude-code/quickstart
- claude.md imports: https://docs.anthropic.com/en/docs/claude-code/memory#claude-md-imports
- slash commands: https://docs.anthropic.com/en/docs/claude-code/slash-commands 
- headless mode: https://docs.anthropic.com/en/docs/claude-code/sdk/sdk-headless


## stacks
-> open fzf for me to choose which stack(s) i want to checkout
-> runs checkout & sym links the commands+agents to the local project .claude/
-> edits the ./CLAUDE.md to have relative path that is set to "read @stack-1/CLAUDE.md" based on this 
CLAUDE.md files can import additional files using `@path/to/import` syntax. The following example imports 3 files:
Copy

```
See @README for project overview and @package.json for available npm commands for this project.

# Additional Instructions
- git workflow @docs/git-instructions.md
```

local settings should be combined into the cwd .claude/settings.local.json rather than symlink'd
**Project settings** are saved in your project directory:

- `.claude/settings.json` for settings that are checked into source control and shared with your team

additionally there should be a check for if mcp servers referenced in the final settings.json are present and if not strategy for how to add it should be provided to user. some require api keys which is why not to just auto install

additional tooling that is separate from the core of stacks

## stacks worktree
has the following behavior:

✓ Checking git repository...
✓ Checking dependencies (tmux, claude, fzf)...

Task name: feature-user-auth

Select branch strategy:
> Create new branch from current (main)
  Create new branch from main/master  
  Use existing branch
  Create new branch from remote

Select worktree location:
> ../myapp-feature-user-auth (recommended)
  ../worktrees/feature-user-auth
  ~/dev/worktrees/feature-user-auth
  Custom path...

Tmux session configuration:
> Create new session: myapp-feature-user-auth
  Attach to existing session
  Run without tmux

Review configuration:
  Task: feature-user-auth
  Branch: feature-user-auth (new from main)
  Location: ../myapp-feature-user-auth
  Session: myapp-feature-user-auth
  
> Proceed
  Cancel
  Modify settings

✓ Creating worktree...
✓ Creating branch 'feature-user-auth' from 'main'
✓ Setting up tmux session 'myapp-feature-user-auth'
✓ Starting Claude Code in right pane

Ready! Press Enter to attach to session or Escape to launch in headless mode..

## stacks sync
prompts claude to update the mcp url and passwords used in the settings.json based on the docker-compose and similar files

## style
- no emojis
- use cargo for rust
- use bun for typscript
- use uv for python (uv python subagent)
- commit messages should exclude "🤖 Generated with [Claude Code](https://claude.ai/code)Co-Authored-By: Claude <noreply@anthropic.com>"
- use conventional commit style guide https://www.conventionalcommits.org/en/v1.0.0/#summary