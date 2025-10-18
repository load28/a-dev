#!/bin/bash
set -e

# Error handler function
send_error_callback() {
  local error_msg="$1"

  if [ -n "$AUTODEV_SERVER_URL" ]; then
    echo "Notifying AutoDev server of error..."

    PAYLOAD=$(cat <<EOF
{
  "task_id": "${TASK_ID:-unknown}",
  "composite_task_id": "${COMPOSITE_TASK_ID:-unknown}",
  "repository_owner": "${REPO_OWNER:-unknown}",
  "repository_name": "${REPO_NAME:-unknown}",
  "pr_number": null,
  "pr_url": null,
  "success": false,
  "error": "${error_msg}"
}
EOF
)

    curl -X POST \
      -H "Content-Type: application/json" \
      -d "$PAYLOAD" \
      "${AUTODEV_SERVER_URL}/callbacks/workflow-complete" \
      || true
  fi

  # Write error to output file
  cat > /output/result.json <<EOF
{
  "has_changes": false,
  "pr_number": null,
  "pr_url": null,
  "success": false,
  "error": "${error_msg}"
}
EOF

  exit 1
}

# Trap errors
trap 'send_error_callback "Script failed at line $LINENO"' ERR

# 환경 변수 검증
: "${ANTHROPIC_API_KEY:?ANTHROPIC_API_KEY is required}"
: "${GITHUB_TOKEN:?GITHUB_TOKEN is required}"
: "${TASK_ID:?TASK_ID is required}"
: "${TASK_TITLE:?TASK_TITLE is required}"
: "${TASK_PROMPT:?TASK_PROMPT is required}"
: "${REPO_OWNER:?REPO_OWNER is required}"
: "${REPO_NAME:?REPO_NAME is required}"
: "${BASE_BRANCH:?BASE_BRANCH is required}"
: "${TARGET_BRANCH:?TARGET_BRANCH is required}"
: "${COMPOSITE_TASK_ID:?COMPOSITE_TASK_ID is required}"

# Optional: AutoDev server callback URL
AUTODEV_SERVER_URL="${AUTODEV_SERVER_URL:-}"

echo "============================================================"
echo "AutoDev Docker Worker"
echo "Task ID: ${TASK_ID}"
echo "Task: ${TASK_TITLE}"
echo "Repository: ${REPO_OWNER}/${REPO_NAME}"
echo "Base Branch: ${BASE_BRANCH}"
echo "Target Branch: ${TARGET_BRANCH}"
echo "============================================================"
echo ""

# GitHub CLI 인증 설정
export GH_TOKEN="${GITHUB_TOKEN}"

# Git 저장소 클론
echo "Cloning repository..."
git clone "https://${GITHUB_TOKEN}@github.com/${REPO_OWNER}/${REPO_NAME}.git" repo
cd repo

# 브랜치 체크아웃
echo "Checking out branch: ${BASE_BRANCH}"
git fetch origin "${BASE_BRANCH}"
git checkout "${BASE_BRANCH}"

echo ""
echo "Executing Claude Code..."
echo ""

# Claude Code 실행
claude \
  --dangerously-skip-permissions \
  --allowedTools "Bash,Read,Write,Edit,Glob,Grep" \
  --model sonnet \
  --max-turns 10 \
  --output-format text \
  --append-system-prompt "Make autonomous decisions and modify files directly without asking questions. Complete the task in minimal steps." \
  "${TASK_PROMPT}"

echo ""
echo "Claude Code execution completed"
echo ""

# 변경사항 확인
git add -A
if git diff --staged --quiet; then
  echo "No changes to commit"
  cat > /output/result.json <<EOF
{
  "has_changes": false,
  "pr_number": null,
  "pr_url": null,
  "success": true,
  "error": null
}
EOF
  exit 0
fi

# 변경사항 커밋
echo "Committing changes..."
git commit -m "AutoDev: ${TASK_TITLE}

Task ID: ${TASK_ID}

${TASK_PROMPT}

Generated with AutoDev
Co-Authored-By: Claude <noreply@anthropic.com>"

# 푸시
echo "Pushing to origin..."
git push origin "${BASE_BRANCH}"

echo ""
echo "Creating Pull Request..."
echo ""

# PR 생성
PR_URL=$(gh pr create \
  --base "${TARGET_BRANCH}" \
  --head "${BASE_BRANCH}" \
  --title "AutoDev: ${TASK_TITLE}" \
  --body "Task: ${TASK_TITLE}

**Task ID:** \`${TASK_ID}\`

Description:
${TASK_PROMPT}

Changes:
This PR contains the automated changes for this task.

---
🤖 Generated with AutoDev
Powered by Claude 4.5 Sonnet" || echo "")

if [ -z "$PR_URL" ]; then
  echo "Failed to create PR"
  send_error_callback "Failed to create PR"
fi

PR_NUMBER=$(echo "$PR_URL" | sed 's/.*\/pull\///')

echo "PR created: $PR_URL (#{PR_NUMBER})"

# 결과 출력
cat > /output/result.json <<EOF
{
  "has_changes": true,
  "pr_number": ${PR_NUMBER},
  "pr_url": "${PR_URL}",
  "success": true,
  "error": null
}
EOF

echo ""
echo "✓ Task completed successfully"

# Notify AutoDev server (if callback URL provided)
if [ -n "$AUTODEV_SERVER_URL" ]; then
  echo ""
  echo "Notifying AutoDev server..."

  PAYLOAD=$(cat <<EOF
{
  "task_id": "${TASK_ID}",
  "composite_task_id": "${COMPOSITE_TASK_ID}",
  "repository_owner": "${REPO_OWNER}",
  "repository_name": "${REPO_NAME}",
  "pr_number": ${PR_NUMBER},
  "pr_url": "${PR_URL}",
  "success": true,
  "error": null
}
EOF
)

  curl -X POST \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD" \
    "${AUTODEV_SERVER_URL}/callbacks/workflow-complete" \
    || echo "Failed to notify server (non-fatal)"
fi
