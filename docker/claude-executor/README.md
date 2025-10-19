# AutoDev Claude Executor Docker Image

이 Docker 이미지는 AutoDev의 로컬 실행 환경을 제공합니다.

## 포함된 도구

- **Node.js 20**: JavaScript 런타임
- **Claude Code CLI**: Anthropic의 Claude Code 커맨드라인 도구
- **Git**: 버전 관리
- **GitHub CLI (gh)**: GitHub API 인터랙션

## 빌드

```bash
docker build -t autodev-claude-executor:latest .
```

## 사용 방법

### 직접 실행
```bash
docker run --rm \
  -v $(pwd):/workspace \
  -e ANTHROPIC_API_KEY=your-api-key \
  -e GITHUB_TOKEN=your-github-token \
  autodev-claude-executor:latest \
  claude "Add a README file"
```

### AutoDev에서 자동 사용
AutoDev 서버가 `AUTODEV_LOCAL_EXECUTOR=true`로 설정되어 있으면 자동으로 이 이미지를 사용합니다.

## 환경 변수

- `ANTHROPIC_API_KEY`: Claude API 키 (필수)
- `GITHUB_TOKEN` / `GH_TOKEN`: GitHub 인증 토큰 (PR 생성 시 필요)

## 주의사항

- 이 이미지는 로컬 개발 및 테스트 목적으로 설계되었습니다
- 프로덕션 환경에서는 적절한 보안 설정을 적용하세요
