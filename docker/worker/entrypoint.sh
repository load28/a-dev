#!/bin/bash
set -e
set -x  # Enable command tracing for debugging

# Redirect all output to both console and log file
exec > >(tee -a /output/execution.log)
exec 2>&1

echo "[$(date -Iseconds)] =========================================="
echo "[$(date -Iseconds)] AutoDev Worker Starting"
echo "[$(date -Iseconds)] =========================================="

# Error handler function
send_error_callback() {
  local error_msg="$1"

  echo "[$(date -Iseconds)] ERROR: ${error_msg}"

  if [ -n "$AUTODEV_SERVER_URL" ]; then
    echo "[$(date -Iseconds)] Notifying AutoDev server of error..."

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
echo "[$(date -Iseconds)] Setting up GitHub CLI authentication..."
export GH_TOKEN="${GITHUB_TOKEN}"

# Git 저장소 클론
echo "[$(date -Iseconds)] Cloning repository ${REPO_OWNER}/${REPO_NAME}..."
git clone "https://${GITHUB_TOKEN}@github.com/${REPO_OWNER}/${REPO_NAME}.git" repo
cd repo

# BASE_BRANCH를 부모 브랜치로 사용하고, 태스크 전용 브랜치 생성
# 언더스코어를 사용하여 Git ref 계층 구조 충돌 회피
TASK_BRANCH="${BASE_BRANCH}_${TASK_ID}"

echo "[$(date -Iseconds)] Fetching parent branch: ${BASE_BRANCH}"
git fetch origin "${BASE_BRANCH}"

echo "[$(date -Iseconds)] Creating task branch: ${TASK_BRANCH} from origin/${BASE_BRANCH}"
git checkout -b "${TASK_BRANCH}" "origin/${BASE_BRANCH}"

echo ""
echo "[$(date -Iseconds)] Executing Claude Code..."
echo "[$(date -Iseconds)] Task: ${TASK_TITLE}"
echo "[$(date -Iseconds)] Prompt: ${TASK_PROMPT}"
echo ""

# Claude Code 실행 (출력을 별도 로그 파일에도 저장)
claude \
  --dangerously-skip-permissions \
  --allowedTools "Bash,Read,Write,Edit,Glob,Grep" \
  --model sonnet \
  --max-turns 10 \
  --output-format text \
  --append-system-prompt "Make autonomous decisions and modify files directly without asking questions. Complete the task in minimal steps." \
  "${TASK_PROMPT}" 2>&1 | tee /output/claude.log

CLAUDE_EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "[$(date -Iseconds)] Claude Code execution completed with exit code: ${CLAUDE_EXIT_CODE}"
echo ""

if [ ${CLAUDE_EXIT_CODE} -ne 0 ]; then
  echo "[$(date -Iseconds)] ERROR: Claude Code failed with exit code ${CLAUDE_EXIT_CODE}"
  send_error_callback "Claude Code execution failed with exit code ${CLAUDE_EXIT_CODE}"
fi

# 변경사항 확인
echo "[$(date -Iseconds)] Checking for changes..."
git add -A
if git diff --staged --quiet; then
  echo "[$(date -Iseconds)] No changes to commit"
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

# 변경사항 표시
echo "[$(date -Iseconds)] Changes detected:"
git diff --staged --stat

# 변경사항 커밋
echo "[$(date -Iseconds)] Committing changes..."
git commit -m "AutoDev: ${TASK_TITLE}

Task ID: ${TASK_ID}

${TASK_PROMPT}

Generated with AutoDev
Co-Authored-By: Claude <noreply@anthropic.com>"

# 푸시 (태스크 브랜치를 푸시)
echo "[$(date -Iseconds)] Pushing task branch to origin: ${TASK_BRANCH}"
git push origin "${TASK_BRANCH}"

echo ""
echo "[$(date -Iseconds)] Creating Pull Request..."
echo ""

# PR 생성 (태스크 브랜치 → 부모 브랜치)
echo "[$(date -Iseconds)] Creating PR: ${TASK_BRANCH} → ${BASE_BRANCH}"
PR_URL=$(gh pr create \
  --base "${BASE_BRANCH}" \
  --head "${TASK_BRANCH}" \
  --title "AutoDev: ${TASK_TITLE}" \
  --body "Task: ${TASK_TITLE}

**Task ID:** \`${TASK_ID}\`
**Task Branch:** \`${TASK_BRANCH}\`
**Base Branch:** \`${BASE_BRANCH}\`

Description:
${TASK_PROMPT}

Changes:
This PR contains the automated changes for this task.

---
🤖 Generated with AutoDev
Powered by Claude 4.5 Sonnet" || echo "")

if [ -z "$PR_URL" ]; then
  echo "[$(date -Iseconds)] ERROR: Failed to create PR"
  send_error_callback "Failed to create PR"
fi

PR_NUMBER=$(echo "$PR_URL" | sed 's/.*\/pull\///')

echo "[$(date -Iseconds)] PR created: $PR_URL (#${PR_NUMBER})"

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
echo "[$(date -Iseconds)] ✓ Task completed successfully"

# Notify AutoDev server (if callback URL provided)
if [ -n "$AUTODEV_SERVER_URL" ]; then
  echo ""
  echo "[$(date -Iseconds)] Notifying AutoDev server..."

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
    || echo "[$(date -Iseconds)] Failed to notify server (non-fatal)"
fi

echo "[$(date -Iseconds)] =========================================="
echo "[$(date -Iseconds)] AutoDev Worker Completed"
echo "[$(date -Iseconds)] Log files saved to /output/"
echo "[$(date -Iseconds)] =========================================="
