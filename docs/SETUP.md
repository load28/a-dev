# AutoDev Setup Guide

Complete guide to set up AutoDev for your repository.

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Step 1: Get Claude API Key](#step-1-get-claude-api-key)
- [Step 2: Add Workflow to Your Repository](#step-2-add-workflow-to-your-repository)
- [Step 3: Configure GitHub Secrets](#step-3-configure-github-secrets)
- [Step 4: Test the Setup](#step-4-test-the-setup)
- [Step 5: Use for Real Work](#step-5-use-for-real-work)
- [Troubleshooting](#troubleshooting)
- [Advanced Configuration](#advanced-configuration)
- [Best Practices](#best-practices)

---

## Prerequisites

### Required

- **GitHub repository** - Any repository where you want to use AutoDev
- **GitHub Actions enabled** - Usually enabled by default
- **Claude API key** - Get from https://console.anthropic.com

### Costs

- **GitHub Actions**: Free for public repos, included in paid plans
- **Claude API**: Pay-as-you-go pricing
  - ~$0.50-$5 per task on average
  - See [Anthropic Pricing](https://www.anthropic.com/pricing)

---

## Step 1: Get Claude API Key

1. Visit https://console.anthropic.com
2. Sign up or log in
3. Navigate to **API Keys**
4. Click **Create Key**
5. Name it: `AutoDev - <Your Repo Name>`
6. Copy the key (starts with `sk-ant-`)
7. ⚠️ **Store it safely** - You won't be able to see it again!

---

## Step 2: Add Workflow to Your Repository

### Option A: Copy Template File

```bash
# In your repository root
mkdir -p .github/workflows
cp /path/to/auto-dev/templates/autodev.yml .github/workflows/

# Or download directly
curl -o .github/workflows/autodev.yml \
  https://raw.githubusercontent.com/load28/a-dev/main/templates/autodev.yml
```

### Option B: Create Manually

Create `.github/workflows/autodev.yml`:

```yaml
name: 'AutoDev'
run-name: 'AutoDev: ${{ inputs.task_title }}'

on:
  workflow_dispatch:
    inputs:
      prompt:
        description: "코딩 작업 설명 (예: 구글 OAuth 인증 추가)"
        type: string
        required: true
      base_branch:
        description: "기준 브랜치"
        type: string
        required: false
        default: 'main'
      task_title:
        description: "작업 제목"
        type: string
        required: false
        default: 'AutoDev Task'

jobs:
  autodev:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write
      issues: write
      actions: read

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: load28/autodev-action@main
        with:
          prompt: ${{ inputs.prompt }}
          base_branch: ${{ inputs.base_branch }}
          task_title: ${{ inputs.task_title }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
```

### Commit and Push

```bash
git add .github/workflows/autodev.yml
git commit -m "feat: add AutoDev workflow"
git push
```

---

## Step 3: Configure GitHub Secrets

1. Go to your repository on GitHub
2. Click **Settings** tab
3. In the left sidebar, click **Secrets and variables** → **Actions**
4. Click **New repository secret**
5. Add the secret:
   - **Name**: `ANTHROPIC_API_KEY`
   - **Value**: Your Claude API key (sk-ant-...)
6. Click **Add secret**

✅ Done! `GITHUB_TOKEN` is automatically provided by GitHub Actions.

---

## Step 4: Test the Setup

### Simple Test Task

1. Go to your repository
2. Click **Actions** tab
3. Click **AutoDev** workflow in the left sidebar
4. Click **Run workflow** button (right side)
5. Fill in the inputs:
   - **Prompt**: `"Add a README.md file with project description"`
   - **Base branch**: `main` (or your default branch)
   - **Task title**: `Add README`
6. Click **Run workflow**

### What Happens

1. Workflow starts (takes ~2-5 minutes)
2. Claude Code analyzes your repository
3. Creates a new branch (e.g., `autodev/20251017-abc123`)
4. Generates README.md
5. Commits the changes
6. Creates a pull request

### Verify Success

1. Go to **Pull requests** tab
2. You should see a new PR titled "Add README"
3. Review the generated README.md
4. ✅ If it looks good, merge the PR!

---

## Step 5: Use for Real Work

### Example: Add Authentication

```yaml
Prompt: "Add Google OAuth authentication with JWT tokens.
Include routes for /auth/google, /auth/google/callback,
and middleware for protected routes."
```

### Example: Create API Endpoint

```yaml
Prompt: "Create REST API for blog posts with CRUD operations.
Use PostgreSQL with sqlx. Include proper error handling
and input validation."
```

### Example: Add Tests

```yaml
Prompt: "Add comprehensive unit tests for all API endpoints.
Use mock database for testing. Aim for 80%+ code coverage."
```

### Example: Refactoring

```yaml
Prompt: "Refactor error handling to use thiserror.
Create custom error types for different modules
and proper error propagation."
```

---

## Troubleshooting

### ❌ "ANTHROPIC_API_KEY not set"

**Problem**: Secret not configured correctly

**Solution**:
1. Check Settings → Secrets and variables → Actions
2. Verify `ANTHROPIC_API_KEY` is listed
3. If not, add it again
4. Re-run the workflow

### ❌ "Permission denied" or "Resource not accessible"

**Problem**: Insufficient permissions

**Solution**: Verify workflow permissions in `.github/workflows/autodev.yml`:

```yaml
permissions:
  contents: write
  pull-requests: write
  issues: write
  actions: read
```

### ❌ "Claude Code failed" or "Execution error"

**Problem**: Prompt unclear or too complex

**Solutions**:
- **Be more specific**: Instead of "add auth", try "add Google OAuth with specific routes"
- **Break it down**: Split large features into smaller tasks
- **Check logs**: Click on the workflow run → expand steps → read error messages

### ❌ "No changes made"

**Problem**: Prompt too vague or already implemented

**Solutions**:
- Check if feature already exists
- Be more specific about what to implement
- Try a different prompt

### ❌ "Tests failing" in PR

**Problem**: Generated code has issues

**Solutions**:
- Claude Code tries to fix tests automatically
- Review the PR and fix manually if needed
- Use a follow-up prompt: "Fix the failing tests in PR #123"

### ❌ "PR creation failed"

**Problem**: Branch name conflict or PR already exists

**Solutions**:
- Check if a PR already exists for this branch
- Delete old branches and try again
- Check GitHub Actions logs for specific error

---

## Advanced Configuration

### Custom Base Branch

Use a development branch instead of main:

```yaml
with:
  base_branch: 'develop'
```

### Custom PR Title

```yaml
with:
  task_title: 'feat: Add OAuth Authentication'
```

### Multiple Workflows

Create separate workflows for different types of tasks:

- `.github/workflows/autodev-features.yml` - New features
- `.github/workflows/autodev-tests.yml` - Test generation
- `.github/workflows/autodev-fixes.yml` - Bug fixes

### Scheduled Tasks

Add a schedule trigger (use with caution):

```yaml
on:
  workflow_dispatch:
    # ... existing inputs
  schedule:
    - cron: '0 9 * * 1'  # Every Monday at 9 AM
```

---

## Best Practices

### ✅ Writing Good Prompts

**Good**:
```
"Add Google OAuth authentication with the following:
- Route: /auth/google
- Callback: /auth/google/callback
- JWT token generation
- Middleware for protected routes
- Environment variables for CLIENT_ID and CLIENT_SECRET"
```

**Bad**:
```
"add auth"
```

### ✅ Task Breakdown

**Instead of**:
```
"Build a complete e-commerce website"
```

**Do**:
1. "Add product model and database schema"
2. "Create product CRUD API"
3. "Add cart functionality"
4. "Implement checkout process"
5. "Add payment integration"

### ✅ Iterative Development

1. Run AutoDev for initial implementation
2. Review the PR
3. Request changes in PR comments
4. Run AutoDev again with: "Fix review comments in PR #123"

### ✅ Code Review

Always review AI-generated code for:
- **Security issues** - SQL injection, XSS, authentication
- **Business logic** - Ensure it matches requirements
- **Error handling** - Proper error propagation
- **Tests** - Verify test coverage and quality
- **Documentation** - Add comments if needed

### ✅ Cost Management

- **Start small** - Test with simple tasks first
- **Monitor usage** - Check Anthropic Console for costs
- **Set limits** - Configure billing alerts
- **Batch tasks** - Combine related changes in one prompt

---

## Example Workflows

### Workflow 1: Add Feature

```bash
# Day 1: Initial implementation
Prompt: "Add user authentication with JWT"

# Review PR → Request changes

# Day 2: Fix issues
Prompt: "Address review comments in PR #45.
Fix token expiration and add refresh token support"

# Merge PR ✅
```

### Workflow 2: Test-Driven Development

```bash
# Step 1: Tests first
Prompt: "Add unit tests for user service.
Test all CRUD operations with mock database"

# Step 2: Implement
Prompt: "Implement user service to pass all tests"

# Step 3: Integration tests
Prompt: "Add integration tests with real test database"
```

### Workflow 3: Refactoring

```bash
# Current: Messy error handling
Prompt: "Refactor error handling:
1. Create custom error types with thiserror
2. Add error variants for database, validation, and auth errors
3. Update all functions to use new error types
4. Ensure all errors have meaningful messages"
```

---

## Next Steps

1. ✅ **Test with simple tasks** - Get comfortable with the workflow
2. 📚 **Read Claude Code docs** - https://docs.claude.com/claude-code
3. 🤝 **Join discussions** - Share experiences and tips
4. 🔧 **Customize** - Adapt workflow to your needs
5. 🚀 **Scale up** - Use for real development work

---

## Support & Resources

- **Documentation**: [action/README.md](../action/README.md)
- **Issues**: https://github.com/load28/auto-dev/issues
- **Discussions**: https://github.com/load28/auto-dev/discussions
- **Claude Code Docs**: https://docs.claude.com/claude-code
- **Anthropic Console**: https://console.anthropic.com

---

**Happy coding with AutoDev! 🚀**
