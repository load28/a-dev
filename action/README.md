# AutoDev Task Executor Action

GitHub Docker Action for executing coding tasks using Claude 4.5 Sonnet API.

## Usage

```yaml
- uses: load28/a-dev/action@main
  with:
    task_prompt: "Add user authentication with OAuth"
    task_id: "task-123"
    anthropic_api_key: ${{ secrets.ANTHROPIC_API_KEY }}
```

## Inputs

### `task_prompt` (required)

The task description/prompt for Claude to execute.

**Example**: `"Implement Google OAuth login system with client-side authentication"`

### `task_id` (required)

Unique identifier for the task.

**Example**: `"511e1652-19f3-4f6c-a3d2-7db96b2f7af8"`

### `anthropic_api_key` (required)

Anthropic API key for Claude API access. Store this as a repository secret.

**Example**: `${{ secrets.ANTHROPIC_API_KEY }}`

## Example Workflow

```yaml
name: 'AutoDev Task Execution'

on:
  workflow_dispatch:
    inputs:
      task_id:
        description: "Task ID"
        required: true
      task_title:
        description: "Task title"
        required: true
      prompt:
        description: "Task prompt"
        required: true

jobs:
  execute_task:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write

    steps:
      - uses: actions/checkout@v4

      # Checkout AutoDev Action
      - uses: actions/checkout@v4
        with:
          repository: load28/a-dev
          path: .autodev-action
          sparse-checkout: |
            action

      # Execute task with Claude
      - uses: ./.autodev-action/action
        with:
          task_prompt: ${{ inputs.prompt }}
          task_id: ${{ inputs.task_id }}
          anthropic_api_key: ${{ secrets.ANTHROPIC_API_KEY }}

      - name: Commit changes
        run: |
          git config user.name "AutoDev Bot"
          git config user.email "autodev@github-actions.bot"
          git add -A
          git commit -m "AutoDev: ${{ inputs.task_title }}" || echo "No changes"
          git push
```

## Features

- ✅ Uses Claude 4.5 Sonnet (`claude-sonnet-4-5-20250929`)
- ✅ Automatic repository structure analysis
- ✅ Intelligent task execution
- ✅ Saves Claude's response for debugging
- ✅ Creates task completion markers

## Output Files

After execution, the action creates:

- `claude_response.txt` - Full response from Claude API
- `.autodev-task-{task_id}.txt` - Task execution metadata

## Requirements

- Docker-enabled runner (default GitHub Actions runners support this)
- Anthropic API key with Claude access

## License

MIT
