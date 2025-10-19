# AutoDev 로컬 Docker Executor 가이드

## 개요

AutoDev의 로컬 Docker Executor는 GitHub Actions 클라우드 환경 대신 **로컬 Docker 컨테이너**에서 태스크를 실행하여 **약 3배 빠른 실행 속도** (3분+ → 1분)를 제공합니다.

## 주요 장점

1. **속도 향상**: GitHub Actions 클라우드 대신 로컬에서 실행하여 약 3배 빠름
2. **비용 절감**: GitHub Actions 실행 시간 감소
3. **디버깅 용이**: 로컬 환경에서 즉시 확인 및 디버깅 가능
4. **기존 기능 100% 유지**:
   - Callback 메커니즘 동일
   - Auto-approve 자동 머지 동일
   - 다음 배치 자동 시작 동일
   - 복합 태스크 PR 생성 동일

## 아키텍처 비교

### 기존 (GitHub Actions)
```
Dashboard/CLI
  → AutoDev Server
  → GitHub Actions (클라우드, 느림 3분+)
  → Claude Code
  → PR 생성
  → Callback
  → Auto-merge (if enabled)
```

### 개선 (Local Docker)
```
Dashboard/CLI
  → AutoDev Server
  → Local Docker (로컬, 빠름 1분)
  → Claude Code
  → PR 생성
  → Callback
  → Auto-merge (if enabled)
```

## 설치 및 설정

### 1단계: Docker 이미지 빌드

```bash
cd /path/to/a-dev

# Claude Executor 이미지 빌드
docker build -t autodev-claude-executor:latest ./docker/claude-executor/

# 빌드 확인
docker images | grep autodev-claude-executor
```

### 2단계: 환경 변수 설정

`.env` 파일에 다음 설정 추가:

```bash
# 로컬 실행 모드 활성화
AUTODEV_LOCAL_EXECUTOR=true

# AutoDev 서버 URL (콜백용)
AUTODEV_SERVER_URL=http://localhost:3000

# 작업 공간 디렉토리
AUTODEV_WORKSPACE_DIR=/tmp/autodev-workspace

# 기존 설정도 필요
GITHUB_TOKEN=github_pat_xxxxxxxxxxxxx
ANTHROPIC_API_KEY=sk-ant-xxxxxxxxxxxxx
```

### 3단계: 작업 공간 디렉토리 생성

```bash
mkdir -p /tmp/autodev-workspace
chmod 777 /tmp/autodev-workspace
```

### 4단계: AutoDev 서버 재시작

```bash
# Docker Compose 사용 시
docker-compose down
docker-compose up -d

# 직접 실행 시
cargo run --bin autodev-api
```

## 사용 방법

### 로컬 모드 활성화 확인

서버 시작 시 로그에서 다음 메시지 확인:

```
INFO autodev_worker: Using LOCAL EXECUTOR mode
```

### 단순 태스크 실행

```bash
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "repository_owner": "myorg",
    "repository_name": "myrepo",
    "title": "Add README",
    "description": "Create a basic README file",
    "prompt": "Create a README.md file with project description"
  }'
```

### 복합 태스크 실행 (병렬 배치)

```bash
curl -X POST http://localhost:3000/tasks/decompose \
  -H "Content-Type: application/json" \
  -d '{
    "repository_owner": "myorg",
    "repository_name": "myrepo",
    "title": "문서 번역 개선",
    "description": "모든 문서 번역 품질 개선",
    "composite_prompt": "docs/ 폴더의 모든 마크다운 파일을 검토하고 번역 품질을 개선하세요."
  }'

# 복합 태스크 ID 가져오기
COMPOSITE_TASK_ID="abc-123"  # 응답에서 가져온 ID

# 실행
curl -X POST "http://localhost:3000/tasks/$COMPOSITE_TASK_ID/orchestrate" \
  -H "Content-Type: application/json" \
  -d '{
    "repository_owner": "myorg",
    "repository_name": "myrepo",
    "base_branch": "main"
  }'
```

## 동작 방식

### 1. 태스크 수신
AutoDev 서버가 Dashboard 또는 CLI로부터 태스크를 받습니다.

### 2. 로컬 Docker 실행
```
1. 저장소를 로컬 임시 디렉토리에 클론
2. Docker 컨테이너 시작 (저장소 마운트)
3. Claude Code CLI 실행
4. 변경사항 커밋 & 푸시
5. GitHub API로 PR 생성
```

### 3. Callback 전송
로컬에서 실행 완료 후, AutoDev 서버에 콜백 전송:

```json
{
  "task_id": "task_123",
  "composite_task_id": "composite_456",
  "repository_owner": "myorg",
  "repository_name": "myrepo",
  "pr_number": 789,
  "pr_url": "https://github.com/myorg/myrepo/pull/789",
  "success": true,
  "error": null
}
```

### 4. Auto-merge 및 다음 배치
AutoDev 서버가 콜백을 받아서:
- `auto_approve=true`면 PR 자동 머지
- 의존성이 충족된 다음 배치 태스크 자동 시작
- 모든 서브태스크 완료 시 최종 PR 생성

## 트러블슈팅

### Docker 이미지가 없다는 오류

```bash
# 이미지 빌드
docker build -t autodev-claude-executor:latest ./docker/claude-executor/
```

### Git 인증 실패

`.env` 파일에 `GITHUB_TOKEN`이 올바르게 설정되었는지 확인:

```bash
# 토큰 테스트
curl -H "Authorization: token YOUR_GITHUB_TOKEN" https://api.github.com/user
```

### Docker 권한 오류

Docker 소켓 접근 권한 확인:

```bash
# 현재 사용자를 docker 그룹에 추가 (Linux)
sudo usermod -aG docker $USER
newgrp docker

# macOS는 Docker Desktop 설치 시 자동 설정됨
```

### 작업 공간 권한 오류

```bash
# 작업 공간 디렉토리 권한 설정
sudo chmod 777 /tmp/autodev-workspace
```

### Claude Code 실행 실패

`ANTHROPIC_API_KEY`가 올바르게 설정되었는지 확인:

```bash
# API 키 테스트
claude --version
```

## 성능 비교

| 환경 | 평균 실행 시간 | 특징 |
|------|---------------|------|
| GitHub Actions (클라우드) | 3분+ | 안정적이지만 느림 |
| Local Docker | 1분 | **3배 빠름**, 즉시 디버깅 가능 |

## GitHub Actions로 다시 전환

로컬 모드를 비활성화하려면 `.env` 파일에서:

```bash
AUTODEV_LOCAL_EXECUTOR=false
```

그리고 서버 재시작:

```bash
docker-compose restart autodev
```

## 주의사항

1. **Docker 리소스**: 병렬 실행 시 충분한 메모리/CPU 할당 필요
2. **임시 파일**: 작업 완료 후 자동으로 정리되지만, 디스크 공간 확인 권장
3. **동시 실행 제한**: 너무 많은 태스크 동시 실행 시 리소스 부족 가능

## 추가 정보

- 로컬 Executor 크레이트: `crates/autodev-local-executor/`
- Docker 이미지: `docker/claude-executor/Dockerfile`
- Worker 통합: `crates/autodev-worker/src/executor.rs`

## 질문 및 지원

- GitHub Issues: https://github.com/load28/a-dev/issues
- 문서: [README.md](README.md)
