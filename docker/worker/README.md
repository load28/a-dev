# AutoDev Worker Docker Image

이 Docker 이미지는 AutoDev 태스크를 실행하는 워커 컨테이너입니다.

## 이미지 빌드

```bash
cd docker/worker
docker build -t autodev-worker:latest .
```

## 이미지 확인

```bash
docker images | grep autodev-worker
```

## 환경 변수

워커 컨테이너는 다음 환경 변수가 필요합니다:

- `ANTHROPIC_API_KEY`: Claude API 키
- `GITHUB_TOKEN`: GitHub 토큰 (PR 생성용)
- `TASK_ID`: 태스크 ID
- `TASK_TITLE`: 태스크 제목
- `TASK_PROMPT`: 태스크 프롬프트
- `REPO_OWNER`: GitHub 저장소 소유자
- `REPO_NAME`: GitHub 저장소 이름
- `BASE_BRANCH`: 기준 브랜치
- `TARGET_BRANCH`: 타겟 브랜치 (PR 대상)
- `COMPOSITE_TASK_ID`: 복합 태스크 ID (standalone인 경우 "standalone")
- `AUTODEV_SERVER_URL`: (Optional) AutoDev 서버 콜백 URL

## 수동 실행 예제

```bash
docker run --rm \
  -e ANTHROPIC_API_KEY="your-api-key" \
  -e GITHUB_TOKEN="your-github-token" \
  -e TASK_ID="task-123" \
  -e TASK_TITLE="Add feature" \
  -e TASK_PROMPT="Add a new feature to the project" \
  -e REPO_OWNER="myorg" \
  -e REPO_NAME="myrepo" \
  -e BASE_BRANCH="main" \
  -e TARGET_BRANCH="main" \
  -e COMPOSITE_TASK_ID="standalone" \
  -v /tmp/output:/output \
  autodev-worker:latest
```

## 출력

워커는 `/output/result.json` 파일을 생성합니다:

```json
{
  "has_changes": true,
  "pr_number": 123,
  "pr_url": "https://github.com/owner/repo/pull/123",
  "success": true,
  "error": null
}
```

## 포함된 도구

- Node.js 20
- Git
- GitHub CLI (`gh`)
- Claude Code CLI (`claude`)
- curl
