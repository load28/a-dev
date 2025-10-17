# AutoDev GitHub Action

[![GitHub Action](https://img.shields.io/badge/GitHub-Action-blue?logo=github-actions)](https://github.com/load28/auto-dev/tree/main/action)
[![Claude Code](https://img.shields.io/badge/Powered%20by-Claude%20Code-orange)](https://claude.com/code)

AI-powered code generation using **Claude Code** (Claude 4.5 Sonnet).

## 🚀 Features

- ✅ **Autonomous code generation** - Claude Code analyzes and implements features
- ✅ **Automatic testing** - Runs tests and fixes errors iteratively
- ✅ **Git integration** - Creates commits with meaningful messages
- ✅ **PR creation** - Automatically opens pull requests
- ✅ **Context-aware** - Understands your entire codebase
- ✅ **Enterprise-grade** - Built on Claude 4.5 Sonnet

## 📋 Quick Start

### 1. Add workflow to your repository

Create `.github/workflows/autodev.yml`:

```yaml
name: 'AutoDev'

on:
  workflow_dispatch:
    inputs:
      prompt:
        description: "What to build"
        type: string
        required: true

jobs:
  autodev:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write

    steps:
      - uses: actions/checkout@v4

      - uses: load28/auto-dev/action@main
        with:
          prompt: ${{ inputs.prompt }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
```

### 2. Set GitHub Secrets

1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Add **ANTHROPIC_API_KEY** with your Claude API key
   - Get your API key: https://console.anthropic.com

### 3. Run the workflow

1. Go to **Actions** tab
2. Click **AutoDev** workflow
3. Click **Run workflow**
4. Enter your prompt: `"Add user authentication with JWT"`
5. Click **Run workflow**
6. ✅ Wait ~2-5 minutes for PR creation!

## 💡 Example Prompts

### Simple Tasks
- `"Add a README.md file with project description"`
- `"Create a .gitignore for Rust project"`
- `"Add license file (MIT)"`

### Features
- `"Add Google OAuth authentication"`
- `"Create REST API for user management"`
- `"Add database migrations for PostgreSQL"`
- `"Implement JWT token generation and validation"`

### Testing & Quality
- `"Add unit tests for all API endpoints"`
- `"Add integration tests with test database"`
- `"Fix all Clippy warnings"`
- `"Add error handling with thiserror"`

### Refactoring
- `"Refactor authentication to use async/await"`
- `"Split monolithic file into modules"`
- `"Add comprehensive documentation"`
- `"Optimize database queries"`

## ⚙️ Configuration

### Inputs

| Name | Description | Required | Default |
|------|-------------|----------|---------|
| `prompt` | Instructions for what to build | Yes | - |
| `base_branch` | Branch to create PR from | No | `main` |
| `agent` | AI agent to use | No | `claude_code` |
| `task_title` | Title for the pull request | No | `AutoDev Task` |

### Environment Variables

| Name | Description | Required |
|------|-------------|----------|
| `ANTHROPIC_API_KEY` | Your Claude API key | Yes |
| `GITHUB_TOKEN` | Automatically provided | Yes |

### Outputs

| Name | Description |
|------|-------------|
| `pr_url` | URL of the created pull request |
| `pr_number` | Number of the created pull request |
| `branch_name` | Name of the created branch |

## 📖 How It Works

1. **Analysis** - Claude Code analyzes your codebase and requirements
2. **Planning** - Creates an implementation plan
3. **Coding** - Generates code changes across multiple files
4. **Testing** - Runs tests and fixes errors automatically
5. **Iteration** - Refines code until tests pass
6. **Commit** - Creates meaningful commit message
7. **PR** - Opens pull request with summary

## 🎯 Best Practices

### ✅ Do

- **Be specific** - Clear prompts get better results
- **Start small** - Test with simple tasks first
- **Review PRs** - Always review AI-generated code
- **Break down** - Split large features into smaller tasks
- **Iterate** - Use follow-up prompts to refine

### ❌ Don't

- **Vague prompts** - "Make it better" won't work well
- **Too complex** - Don't ask for entire applications at once
- **Skip review** - Always review security and logic
- **Expect perfection** - AI makes mistakes, review carefully

## 💰 Cost Estimation

Costs are based on Claude API usage (prompt + completion tokens):

- **Simple tasks** (add file, small changes): $0.50-1
- **Medium tasks** (API endpoint, auth): $2-5
- **Large tasks** (full feature, multiple files): $5-15
- **Complex features** (with iterations): $10-30

## 🔒 Security

- Your code runs in **your** GitHub Actions runners
- API keys are stored as **GitHub Secrets**
- Claude Code has **read-only** access to most operations
- All changes go through **pull requests** for review

## 🛠️ Troubleshooting

### "ANTHROPIC_API_KEY not set"
→ Add the secret in **Settings** → **Secrets and variables** → **Actions**

### "Permission denied"
→ Ensure workflow has permissions:
```yaml
permissions:
  contents: write
  pull-requests: write
```

### "Claude Code failed"
→ Check that your prompt is clear and achievable
→ Try breaking down into smaller steps

### "No changes made"
→ Prompt might be too vague
→ Try being more specific about implementation

### "Tests failing"
→ Claude Code will automatically try to fix test failures
→ May need manual intervention for complex issues

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md).

## 📄 License

MIT License - see [LICENSE](../LICENSE)

## 🔗 Links

- [GitHub Repository](https://github.com/load28/auto-dev)
- [Setup Guide](../docs/SETUP.md)
- [Claude Code Documentation](https://docs.claude.com/claude-code)
- [Anthropic Console](https://console.anthropic.com)

## 💬 Support

- [Issues](https://github.com/load28/auto-dev/issues)
- [Discussions](https://github.com/load28/auto-dev/discussions)

---

**Powered by [Claude Code](https://claude.com/code)** - The agentic coding assistant from Anthropic
