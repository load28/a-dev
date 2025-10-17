#!/usr/bin/env python3
"""
AutoDev Task Execution Script
Executes tasks using Claude 4.5 Sonnet API
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

    for path in workspace.rglob("*"):
        if path.is_file() and not any(part.startswith(".") for part in path.parts):
            rel_path = path.relative_to(workspace)
            if path.stat().st_size < 100000:  # Only include files < 100KB
                repo_structure.append(str(rel_path))

    return repo_structure[:max_files]


def create_system_prompt(repo_structure, task_prompt):
    """Create system prompt for Claude"""
    files_list = "\n".join(repo_structure)

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
    # Get environment variables
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        print("ERROR: ANTHROPIC_API_KEY is required", file=sys.stderr)
        sys.exit(1)

    task_prompt = os.environ.get("TASK_PROMPT", "")
    task_id = os.environ.get("TASK_ID", "unknown")
    workspace = os.environ.get("WORKSPACE", "/workspace")

    if not task_prompt:
        print("ERROR: TASK_PROMPT is required", file=sys.stderr)
        sys.exit(1)

    print(f"Starting task execution: {task_id}")
    print(f"Workspace: {workspace}")

    # Get repository structure
    print("Analyzing repository structure...")
    repo_structure = get_repository_structure(workspace)
    print(f"Found {len(repo_structure)} files")

    # Create system prompt
    system_prompt = create_system_prompt(repo_structure, task_prompt)

    # Initialize Claude client
    print("Initializing Claude API (claude-sonnet-4-5-20250929)...")
    client = anthropic.Anthropic(api_key=api_key)

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
        print(f"Claude response received ({len(response_text)} characters)")

        # Save response for debugging
        response_file = os.path.join(workspace, "claude_response.txt")
        with open(response_file, "w") as f:
            f.write(response_text)
        print(f"Response saved to: {response_file}")

        # Create task completion marker
        marker_file = os.path.join(workspace, f".autodev-task-{task_id}.txt")
        with open(marker_file, "w") as f:
            f.write(f"Task {task_id} executed\n")
            f.write(f"Prompt: {task_prompt}\n")
            f.write(f"Response length: {len(response_text)} characters\n")

        print("Task analysis completed by Claude API")
        print("Note: This is a simplified workflow. Actual file modifications")
        print("would require parsing Claude's response and applying changes.")

        return 0

    except Exception as e:
        print(f"ERROR: Claude API call failed: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
