# GitHub Repository Setup Guide

This guide walks you through setting up the GitHub repository to enable automatic releases for the Stacks CLI.

## 1. Create GitHub Repository

1. **Create Repository:**
   - Go to [GitHub](https://github.com) and create a new repository
   - Repository name: `claude-code-stacks`
   - Description: "A CLI tool for managing Claude Code workflow stacks"
   - Choose Public (for open source) or Private
   - **Don't** initialize with README, .gitignore, or license (we already have these)

2. **Add Repository URL to Code:**
   - The `Cargo.toml` is already configured with the repository URL
   - The install script references `csaben/claude-code-stacks`
   - Update these if your username/org is different

## 2. Push Code to GitHub

```bash
# Add the GitHub remote (replace with your actual repository URL)
git remote add origin https://github.com/csaben/claude-code-stacks.git

# Push all code and tags
git push -u origin main

# If you have tags already, push them too
git push --tags
```

## 3. Configure GitHub Actions (Automatic)

The CI/CD workflows are already included in `.github/workflows/`:

- **`ci.yml`**: Runs on every push/PR (testing, linting, security audit)
- **`release.yml`**: Runs on tag pushes to create releases with binaries

### Workflow Features:

**CI Pipeline (`ci.yml`):**
- ✅ Tests on Ubuntu, macOS, Windows
- ✅ Tests with stable and beta Rust
- ✅ Code formatting checks (`cargo fmt`)
- ✅ Linting with Clippy
- ✅ Security audit with `cargo audit`
- ✅ Code coverage reporting

**Release Pipeline (`release.yml`):**
- ✅ Cross-compilation for 5 platforms
- ✅ Binary stripping and optimization
- ✅ SHA256 checksums
- ✅ Automatic changelog generation
- ✅ GitHub releases with assets

## 4. Create Your First Release

### Option A: Tag-based Release (Recommended)
```bash
# Create and push a version tag
git tag v0.1.0
git push origin v0.1.0
```

This will automatically:
1. Trigger the release workflow
2. Build binaries for all platforms
3. Run tests and security checks
4. Create a GitHub release
5. Upload all binaries and checksums

### Option B: Manual Release
1. Go to Actions tab in your GitHub repository
2. Click "Release" workflow
3. Click "Run workflow"
4. Choose your options and run

## 5. Verify Installation Works

Once the release is created:

```bash
# Test the installation script
curl -sSL https://raw.githubusercontent.com/csaben/claude-code-stacks/main/install.sh | bash

# Or test locally first
bash install.sh
```

## 6. Configure Optional Features

### Repository Settings:

**Branch Protection (Recommended):**
- Go to Settings → Branches
- Add rule for `main` branch
- Enable "Require status checks to pass"
- Enable "Require up-to-date branches"
- Select CI checks that must pass

**Secrets (Optional):**
- `CODECOV_TOKEN`: For code coverage reporting
- No other secrets needed (GitHub provides `GITHUB_TOKEN` automatically)

### Repository Topics:
Add these topics to help discovery:
- `cli`
- `claude-code`  
- `workflow-automation`
- `rust`
- `developer-tools`

## 7. Badges for README (Optional)

Add these to your README.md:

```markdown
[![CI](https://github.com/csaben/claude-code-stacks/workflows/CI/badge.svg)](https://github.com/csaben/claude-code-stacks/actions/workflows/ci.yml)
[![Release](https://github.com/csaben/claude-code-stacks/workflows/Release/badge.svg)](https://github.com/csaben/claude-code-stacks/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
```

## 8. Release Process Going Forward

### Semantic Versioning:
- `v0.1.0` → `v0.1.1` (patch: bug fixes)
- `v0.1.0` → `v0.2.0` (minor: new features)
- `v0.1.0` → `v1.0.0` (major: breaking changes)

### Release Steps:
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` (optional, auto-generated)
3. Commit changes: `git commit -m "chore: bump version to v0.1.1"`
4. Create tag: `git tag v0.1.1`
5. Push: `git push && git push --tags`
6. GitHub Actions handles the rest!

## 9. Troubleshooting

### Common Issues:

**Release workflow fails:**
- Check Actions tab for detailed logs
- Verify all target platforms can build
- Check for dependency issues

**Install script fails:**
- Verify release was created successfully
- Check binary naming matches expectations
- Ensure binaries are executable

**Tests fail:**
- CI requires `tmux` and `fzf` on runners
- Some tests may need adjustment for different platforms

## 10. Monitoring

**Check these regularly:**
- Actions tab: Monitor CI/CD pipeline health
- Security tab: Dependabot alerts
- Insights: Repository statistics and usage

## Summary

After setup, your workflow will be:

1. **Develop** → Push to main (CI runs automatically)
2. **Ready to release** → Create version tag (Release runs automatically)  
3. **Users install** → `curl | bash` downloads pre-built binary
4. **Profit** 🎉

The entire CI/CD pipeline is now configured and ready to go!