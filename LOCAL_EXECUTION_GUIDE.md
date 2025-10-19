# AutoDev 로컬 실행 가이드

이 가이드는 AutoDev를 로컬 Docker 환경에서 실행하는 방법을 설명합니다.

## 아키텍처

```
┌─────────────────────────────────────────────────────┐
│  로컬 호스트 머신 (macOS/Linux)                       │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │  AutoDev API 서버 (Rust 프로세스)             │  │
│  │  - cargo run -p autodev-api                 │  │
│  │  - 포트 3000                                 │  │
│  └───────────────┬─────────────────────────────┘  │
│                  │                                 │
│                  │ Docker API 호출                  │
│                  ▼                                 │
│  ┌─────────────────────────────────────────────┐  │
│  │  Docker Engine                              │  │
│  │  ┌────────────┐  ┌────────────┐             │  │
│  │  │  Worker 1  │  │  Worker 2  │  ...        │  │
│  │  │  Container │  │  Container │             │  │
│  │  └────────────┘  └────────────┘             │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │  PostgreSQL (Docker Container)              │  │
│  │  - docker-compose up postgres               │  │
│  └─────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## 사전 준비

### 1. Docker 설치 및 실행
```bash
# Docker Desktop이 실행 중인지 확인
docker ps
```

### 2. Worker 이미지 빌드
```bash
cd docker/worker
docker build -t autodev-worker:latest .
cd ../..
```

### 3. 환경 변수 설정
`.env` 파일을 확인하고 필요한 값을 설정합니다:

```bash
# 필수 설정
AUTODEV_LOCAL_EXECUTOR=true
AUTODEV_WORKSPACE_DIR=/tmp/autodev-workspace
AUTODEV_SERVER_URL=http://localhost:3000

# API 키
ANTHROPIC_API_KEY=your-api-key
GITHUB_TOKEN=your-github-token

# 데이터베이스
DATABASE_URL=postgresql://autodev:password@localhost:5432/autodev
```

## 실행 순서

### 1단계: PostgreSQL 시작

```bash
docker-compose up -d postgres
```

PostgreSQL이 준비될 때까지 기다립니다:
```bash
docker-compose logs -f postgres
# "database system is ready to accept connections" 메시지 확인
```

### 2단계: AutoDev API 서버 시작 (호스트에서)

**새 터미널 창에서:**
```bash
# 빌드
cargo build --release

# 실행
cargo run -p autodev-api
```

또는 release 빌드로:
```bash
./target/release/autodev-api
```

서버가 시작되면 다음 메시지가 표시됩니다:
```
✓ Docker executor initialized for local execution
🚀 AutoDev API Server running on http://0.0.0.0:3000
```

### 3단계: 테스트

간단한 태스크로 테스트합니다:

```bash
curl -X POST http://localhost:3000/tasks/composite \
  -H "Content-Type: application/json" \
  -d '{
    "repository_owner": "your-org",
    "repository_name": "your-repo",
    "title": "Test Task",
    "description": "Test local execution",
    "composite_prompt": "Create a simple hello world function in README.md",
    "auto_approve": false
  }'
```

## 로그 확인

### AutoDev 서버 로그
서버를 실행한 터미널에서 실시간으로 확인됩니다.

### Worker 컨테이너 로그
```bash
# 실행 중인 워커 확인
docker ps | grep autodev-task

# 특정 워커 로그 확인
docker logs -f autodev-task-{task-id}
```

### PostgreSQL 로그
```bash
docker-compose logs -f postgres
```

## 디버깅

### 워크스페이스 확인
```bash
ls -la /tmp/autodev-workspace/
```

### Worker 이미지 확인
```bash
docker images | grep autodev-worker
```

### Docker 연결 테스트
```bash
docker ps
```

## 중지 방법

### 1. AutoDev 서버 중지
서버 터미널에서 `Ctrl+C`

### 2. PostgreSQL 중지
```bash
docker-compose down
```

### 3. 워커 컨테이너 정리 (필요시)
```bash
docker ps -a | grep autodev-task | awk '{print $1}' | xargs docker rm -f
```

## 문제 해결

### Docker 연결 오류
```
Failed to initialize Docker executor: Cannot connect to the Docker daemon
```

**해결책:**
1. Docker Desktop 실행 확인
2. Docker 소켓 권한 확인: `ls -l /var/run/docker.sock`

### Worker 이미지 없음
```
Error response from daemon: No such image: autodev-worker:latest
```

**해결책:**
```bash
cd docker/worker
docker build -t autodev-worker:latest .
```

### 경로 오류
```
bind source path does not exist
```

**해결책:**
1. `.env`에 `AUTODEV_WORKSPACE_DIR` 설정 확인
2. 디렉토리 생성: `mkdir -p /tmp/autodev-workspace`

### 포트 충돌
```
Address already in use (os error 48)
```

**해결책:**
1. 다른 포트 사용: `.env`에서 `API_PORT=3001` 설정
2. 기존 프로세스 종료: `lsof -ti:3000 | xargs kill -9`

## GitHub Actions와 비교

| 항목 | GitHub Actions | 로컬 Docker |
|------|---------------|-------------|
| 실행 위치 | GitHub 클라우드 | 로컬 머신 |
| 실행 시간 | 3+ 분 | ~1 분 |
| 비용 | 무료 (제한 있음) | 무료 |
| 네트워크 | 필요 | 로컬 |
| 디버깅 | 어려움 | 쉬움 |

## 다음 단계

로컬 실행이 정상 작동하면:
1. 실제 프로젝트로 테스트
2. Composite Task로 복잡한 작업 테스트
3. Auto-approve 모드로 자동화 테스트
