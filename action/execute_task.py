#!/usr/bin/env python3
"""
AutoDev Task Execution Script
Executes tasks using Claude 4.5 Sonnet API

This script is designed to run as a GitHub Action (Docker-based).
GitHub Actions provides inputs as environment variables with INPUT_ prefix.
"""
import os
import sys
import json
import anthropic
from pathlib import Path


def get_repository_structure(workspace_path, max_files=50):
    """Get repository file structure"""
    repo_structure = []
    workspace = Path(workspace_path)

    if not workspace.exists():
        print(f"Warning: Workspace path does not exist: {workspace_path}", file=sys.stderr)
        return []

    for path in workspace.rglob("*"):
        # Skip hidden files and directories
        if path.is_file() and not any(part.startswith(".") for part in path.parts):
            try:
                rel_path = path.relative_to(workspace)
                # Only include files < 100KB
                if path.stat().st_size < 100000:
                    repo_structure.append(str(rel_path))
            except Exception as e:
                print(f"Warning: Failed to process {path}: {e}", file=sys.stderr)
                continue

    return repo_structure[:max_files]


def create_system_prompt(repo_structure, task_prompt):
    """Create system prompt for Claude"""
    files_list = "\n".join(repo_structure) if repo_structure else "(No files found)"

    return f"""You are an expert software engineer working on a specific task.

Repository structure:
{files_list}

Your task:
{task_prompt}

IMPORTANT:
1. Make focused changes related ONLY to this specific task
2. Do not modify files outside the scope of this task
3. Create clear, atomic commits
4. Write comprehensive commit messages
5. Ensure code quality and add tests if needed

Please analyze the codebase, make the necessary changes, and provide:
1. List of files to create/modify
2. Exact content for each file
3. Commit message
"""


def main():
    print("=" * 60)
    print("AutoDev Task Executor (Claude 4.5 Sonnet)")
    print("=" * 60)

    # Get environment variables
    # GitHub Actions provides inputs as INPUT_<NAME> (uppercase, hyphens become underscores)
    api_key = os.environ.get("INPUT_ANTHROPIC_API_KEY") or os.environ.get("ANTHROPIC_API_KEY")
    task_prompt = os.environ.get("INPUT_TASK_PROMPT") or os.environ.get("TASK_PROMPT", "")
    task_id = os.environ.get("INPUT_TASK_ID") or os.environ.get("TASK_ID", "unknown")
    workspace = os.environ.get("GITHUB_WORKSPACE") or os.environ.get("WORKSPACE", "/github/workspace")

    # Validate inputs
    if not api_key:
        print("ERROR: Anthropic API key is required", file=sys.stderr)
        print("Set INPUT_ANTHROPIC_API_KEY or ANTHROPIC_API_KEY environment variable", file=sys.stderr)
        sys.exit(1)

    if not task_prompt:
        print("ERROR: Task prompt is required", file=sys.stderr)
        print("Set INPUT_TASK_PROMPT or TASK_PROMPT environment variable", file=sys.stderr)
        sys.exit(1)

    print(f"Task ID: {task_id}")
    print(f"Workspace: {workspace}")
    print(f"Task Prompt: {task_prompt[:100]}..." if len(task_prompt) > 100 else f"Task Prompt: {task_prompt}")
    print()

    # Get repository structure
    print("Analyzing repository structure...")
    repo_structure = get_repository_structure(workspace)
    print(f"Found {len(repo_structure)} files")
    print()

    # Create system prompt
    system_prompt = create_system_prompt(repo_structure, task_prompt)

    # Initialize Claude client
    print("Initializing Claude API (claude-sonnet-4-5-20250929)...")
    client = anthropic.Anthropic(api_key=api_key)
    print()

    # Call Claude API
    print("Calling Claude API...")
    try:
        message = client.messages.create(
            model="claude-sonnet-4-5-20250929",
            max_tokens=8192,
            temperature=0.2,
            system=system_prompt,
            messages=[
                {
                    "role": "user",
                    "content": f"Please complete the task: {task_prompt}"
                }
            ]
        )

        response_text = message.content[0].text
        print(f"✓ Claude response received ({len(response_text)} characters)")
        print()

        # Save response for debugging
        response_file = os.path.join(workspace, "claude_response.txt")
        with open(response_file, "w", encoding="utf-8") as f:
            f.write(response_text)
        print(f"✓ Response saved to: {response_file}")

        # Create task completion marker
        marker_file = os.path.join(workspace, f".autodev-task-{task_id}.txt")
        with open(marker_file, "w", encoding="utf-8") as f:
            f.write(f"Task {task_id} executed\n")
            f.write(f"Prompt: {task_prompt}\n")
            f.write(f"Response length: {len(response_text)} characters\n")
            f.write(f"Model: claude-sonnet-4-5-20250929\n")
        print(f"✓ Task marker created: {marker_file}")
        print()

        print("=" * 60)
        print("Task analysis completed by Claude API")
        print("=" * 60)
        print()
        print("Note: This is a simplified workflow. Actual file modifications")
        print("would require parsing Claude's response and applying changes.")
        print()

        return 0

    except Exception as e:
        print()
        print("=" * 60)
        print("ERROR: Claude API call failed")
        print("=" * 60)
        print(f"{e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
