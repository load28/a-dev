# Workflow 업데이트 가이드

## 문제
기존 `autodev.yml` workflow가 구식 input 구조를 사용해서 최신 CLI와 호환되지 않습니다.

### 에러 메시지
```
Error: Octocrab error: GitHub: Unexpected inputs provided: ["branch", "commit_message", "task_id"]
```

## 해결 방법

대상 저장소(`load28/a-dev-test` 등)의 workflow 파일을 최신 템플릿으로 교체해야 합니다.

---

## 단계별 가이드

### 1. 대상 저장소 클론 또는 이동

```bash
# 이미 클론되어 있으면 해당 디렉토리로 이동
cd ~/path/to/a-dev-test

# 또는 새로 클론
git clone https://github.com/load28/a-dev-test.git
cd a-dev-test
```

### 2. Workflow 파일 업데이트

#### 옵션 A: 로컬에서 복사 (auto-dev 프로젝트가 로컬에 있는 경우)

```bash
# auto-dev 프로젝트의 템플릿 복사
cp ~/path/to/auto-dev/templates/autodev.yml .github/workflows/
```

#### 옵션 B: 직접 다운로드

```bash
# GitHub에서 직접 다운로드 (auto-dev를 push한 경우)
mkdir -p .github/workflows
curl -o .github/workflows/autodev.yml \
  https://raw.githubusercontent.com/load28/a-dev/main/templates/autodev.yml
```

#### 옵션 C: 수동 복사

`.github/workflows/autodev.yml` 파일을 다음 내용으로 교체:

```yaml
# AutoDev Workflow Template
# Copy this file to your repository: .github/workflows/autodev.yml

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
        description: "작업 제목 (PR 제목으로 사용됨)"
        type: string
        required: false
        default: 'AutoDev Task'
      agent:
        description: "AI 에이전트"
        type: choice
        default: 'claude_code'
        options:
          - claude_code

jobs:
  autodev:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write
      issues: write
      actions: read

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history needed for Claude Code

      # Single step - Claude Code handles everything!
      - name: Run AutoDev with Claude Code
        uses: load28/auto-dev/action@main
        with:
          prompt: ${{ inputs.prompt }}
          base_branch: ${{ inputs.base_branch }}
          agent: ${{ inputs.agent }}
          task_title: ${{ inputs.task_title }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
```

### 3. GitHub Secrets 설정

1. GitHub에서 저장소로 이동: `https://github.com/load28/a-dev-test`
2. **Settings** → **Secrets and variables** → **Actions** 클릭
3. **New repository secret** 클릭
4. 다음 secret 추가:

   **Name**: `ANTHROPIC_API_KEY`
   **Value**: `sk-ant-api03-xxxxxxxxx` (실제 Claude API 키)

5. **Add secret** 클릭

> ⚠️ `GITHUB_TOKEN`은 자동으로 제공되므로 따로 추가할 필요 없습니다.

### 4. Commit 및 Push

```bash
git add .github/workflows/autodev.yml
git commit -m "chore: update AutoDev workflow to use Claude Code"
git push origin main
```

---

## 검증

### 테스트 1: CLI에서 실행

```bash
cd ~/path/to/auto-dev

./target/release/autodev task \
  --owner load28 \
  --repo a-dev-test \
  --title "Add README" \
  --description "테스트" \
  --prompt "Add a simple README file with project description" \
  --execute
```

**예상 출력**:
```
✓ Workflow triggered: run_20251017_071900

🤖 Claude Code is now running in GitHub Actions.
   Check progress at: https://github.com/load28/a-dev-test/actions

💡 The workflow will:
   1. Checkout the repository
   2. Run Claude Code CLI with your prompt
   3. Automatically commit changes
   4. Create a pull request
```

### 테스트 2: GitHub Actions UI에서 확인

1. `https://github.com/load28/a-dev-test/actions` 방문
2. 최근 실행된 "AutoDev: Add README" workflow 확인
3. 실행 로그에서 Claude Code 실행 확인
4. PR이 생성되었는지 확인

### 테스트 3: Issue Comment (Webhook 설정 완료 시)

Issue에 댓글 작성:
```
autodev: add a simple README file
```

→ 자동으로 workflow가 트리거되어야 함

---

## 트러블슈팅

### ❌ "Unexpected inputs" 에러 재발

**원인**: workflow 파일이 제대로 업데이트되지 않음

**해결**:
```bash
# workflow 파일 확인
cat .github/workflows/autodev.yml | grep "inputs:" -A 10

# prompt, base_branch, task_title이 있어야 함
# task_id, branch, commit_message가 있으면 안됨
```

### ❌ "ANTHROPIC_API_KEY not set" 에러

**원인**: GitHub Secrets에 API 키가 설정되지 않음

**해결**:
1. https://github.com/load28/a-dev-test/settings/secrets/actions
2. `ANTHROPIC_API_KEY` 확인 및 추가

### ❌ "Resource not accessible by integration" 에러

**원인**: Workflow 권한 부족

**해결**:
1. Settings → Actions → General
2. "Workflow permissions" 섹션에서
3. ✅ "Read and write permissions" 선택
4. ✅ "Allow GitHub Actions to create and approve pull requests" 체크

### ❌ Claude Code 실행 실패

**원인**: Claude Code CLI 설치 문제

**해결**: 현재 `action/Dockerfile`에서 자동 설치되므로 대부분 문제 없음.
만약 실패하면 `action/Dockerfile` 확인:
```dockerfile
RUN npm install -g @anthropic-ai/claude-code
```

---

## 주요 변경 사항 요약

### Before (구식)
```yaml
inputs:
  task_id:      # ❌ 제거됨
    required: true
  branch:       # ❌ 제거됨
    required: true
  commit_message:  # ❌ 제거됨
    required: true
```

### After (신식)
```yaml
inputs:
  prompt:       # ✅ 새로 추가
    required: true
  task_title:   # ✅ 새로 추가
    required: false
  base_branch:  # ✅ 새로 추가
    required: false
```

### 아키텍처 변경

**Before**:
```
CLI → AI 로컬 실행 → Workflow → PR 생성
```

**After**:
```
CLI → Workflow → Claude Code (GitHub Actions) → PR 생성
```

---

## 다음 단계

1. ✅ Workflow 파일 업데이트 완료
2. ✅ Secrets 설정 완료
3. ✅ 테스트 실행 성공
4. 📋 다른 저장소에도 동일하게 적용

---

**문의사항**: https://github.com/load28/auto-dev/issues
