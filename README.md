# AutoDev Clone

> Automated AI Development Platform - Rust Implementation

Delino AutoDev의 기능을 동일하게 구현한 오픈소스 자동화 개발 플랫폼입니다.

## 🚀 주요 기능

### 1. **자동화된 AI 개발**
- Claude Code 등의 AI 에이전트를 사용하여 코드 자동 생성
- GitHub Actions를 통한 완전 자동화된 워크플로우
- PR 자동 생성 및 관리

### 2. **CompositeTask - 지능형 작업 분해**
- 복잡한 작업을 자동으로 하위 작업으로 분해
- 의존성 그래프 분석을 통한 최적 병렬 실행
- 자동 승인 모드 지원

### 3. **자동 코드 리뷰 처리**
- PR 리뷰 코멘트에 자동 대응
- CI 실패 자동 수정
- 반복적인 피드백 처리

### 4. **확장성**
- REST API 제공
- CLI 인터페이스
- 데이터베이스 기반 영구 저장
- 메트릭 및 통계 수집

## 📋 시스템 아키텍처

```
┌─────────────────────────────────────────────────────────────┐
│                         AutoDev                              │
│                                                              │
│  ┌──────────────┐     ┌──────────────┐    ┌─────────────┐  │
│  │   CLI/API    │────▶│    Engine    │───▶│  AI Agent   │  │
│  └──────────────┘     └──────────────┘    └─────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ GitHub Actions   │
                    │   + AI Agent     │
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Pull Request    │
                    └──────────────────┘
```

### 구조 설명

1. **Engine**: 작업 생성, 실행, 관리의 핵심 로직
2. **TaskDecomposer**: 복잡한 작업을 여러 하위 작업으로 분해
3. **GitHubClient**: GitHub API 통합
4. **AIAgent**: Claude Code 등의 AI 에이전트 통합
5. **Database**: PostgreSQL 기반 영구 저장소
6. **API/CLI**: 사용자 인터페이스

## 🛠️ 설치 및 설정

### 필수 요구사항

- Rust 1.75 이상
- PostgreSQL 14 이상 (옵션)
- GitHub 계정 및 토큰
- Anthropic API 키 (Claude Code 사용 시)

### 빠른 시작

1. **저장소 클론**
```bash
git clone https://github.com/yourusername/autodev-clone
cd autodev-clone
```

2. **환경 변수 설정**
```bash
cp .env.example .env
# .env 파일을 편집하여 실제 값 입력
```

3. **데이터베이스 초기화 (옵션)**
```bash
# Docker Compose 사용
docker-compose up -d postgres

# 스키마 초기화
cargo run --bin autodev -- init-db
```

4. **빌드 및 실행**
```bash
# 개발 모드
cargo run --bin autodev -- serve --port 3000

# 릴리즈 빌드
cargo build --release
./target/release/autodev serve --port 3000
```

## 📖 사용 방법

### CLI 사용

#### 1. 단순 작업 생성 및 실행
```bash
autodev task \
  --owner myorg \
  --repo myproject \
  --title "Add user authentication" \
  --description "Implement JWT-based authentication" \
  --prompt "Add JWT authentication to the API. Include login and logout endpoints with proper error handling." \
  --execute
```

#### 2. 복합 작업 생성 (병렬 실행)
```bash
autodev composite \
  --owner myorg \
  --repo myproject \
  --title "Improve documentation translations" \
  --description "Review and fix translations for all documentation pages" \
  --prompt "Improve the translation quality for each page. Create one task per page. Each task should handle a single page. If a page's translation has no issues, do not create a PR. Review all translations manually - DO NOT use automated commands. Include all supported languages: English, Korean, Japanese, Chinese." \
  --execute
```

#### 3. 자동 승인 모드로 보안 감사
```bash
autodev composite \
  --owner myorg \
  --repo myproject \
  --title "Security audit for RPC methods" \
  --description "Review and fix security issues in all RPC methods" \
  --prompt "Review all RPC methods and fix any security issues. Create one task per RPC method. Each task should handle a single method. Fix all security issues, but do NOT create a PR if the method already has sufficient test coverage." \
  --auto-approve \
  --execute
```

#### 4. 작업 상태 확인
```bash
# 특정 작업 상태
autodev status task_abc123

# 모든 작업 나열
autodev list

# 완료된 작업만 보기
autodev list --status completed --limit 20

# 통계 보기
autodev stats
```

### API 사용

#### 서버 시작
```bash
autodev serve --port 3000
```

#### API 엔드포인트

**단순 작업 생성**
```bash
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "repository_owner": "myorg",
    "repository_name": "myproject",
    "title": "Add authentication",
    "description": "Implement JWT auth",
    "prompt": "Add JWT authentication to the API"
  }'
```

**복합 작업 생성**
```bash
curl -X POST http://localhost:3000/composite-tasks \
  -H "Content-Type: application/json" \
  -d '{
    "repository_owner": "myorg",
    "repository_name": "myproject",
    "title": "Security audit",
    "description": "Review all RPC methods",
    "composite_prompt": "Review and fix security issues in all RPC methods...",
    "auto_approve": true
  }'
```

**작업 실행**
```bash
curl -X POST http://localhost:3000/tasks/{task_id}/execute
```

**작업 상태 조회**
```bash
curl http://localhost:3000/tasks/{task_id}
```

**모든 작업 조회**
```bash
curl http://localhost:3000/tasks
```

## 🔧 대상 저장소에 AutoDev 설정

AutoDev를 사용하려는 저장소에 다음 설정을 추가하세요.

### 1. Workflow 파일 복사

```bash
# 대상 저장소에서
mkdir -p .github/workflows
cp /path/to/auto-dev/templates/autodev.yml .github/workflows/

# 또는 직접 다운로드
curl -o .github/workflows/autodev.yml \
  https://raw.githubusercontent.com/load28/auto-dev/main/templates/autodev.yml
```

### 2. GitHub Secrets 설정

대상 저장소의 Settings → Secrets and variables → Actions에서:

- **ANTHROPIC_API_KEY**: Claude API 키 추가
- **GITHUB_TOKEN**: 자동 제공됨 (추가 불필요)

### 3. 테스트

```bash
# Issue에 댓글로 테스트 (Webhook 설정 완료 시)
autodev: add a simple README file

# 또는 CLI로 테스트
./target/release/autodev task \
  --owner your-org \
  --repo your-repo \
  --title "Test AutoDev" \
  --prompt "Add README file" \
  --execute
```

상세한 설정 가이드는 [docs/SETUP.md](docs/SETUP.md)를 참조하세요.

## 📊 기능 상세

### CompositeTask - 작업 분해 및 병렬 실행

```rust
// 예시: 번역 작업을 페이지별로 자동 분해
let composite_task = engine
    .create_composite_task(
        &repo,
        "Improve translation quality".to_string(),
        "Review and fix translations for all pages".to_string(),
        "Improve the translation quality for each page...".to_string(),
        false, // 수동 승인
    )
    .await?;

// 의존성 그래프 분석 및 병렬 배치 생성
// Batch 1: [Task A, Task B, Task C] - 병렬 실행
// Batch 2: [Task D] - Batch 1 완료 후 실행
// Batch 3: [Task E, Task F] - Batch 2 완료 후 실행

engine.execute_composite_task(&composite_task, &repo).await?;
```

### 자동 코드 리뷰 처리

PR 리뷰 코멘트가 달리면 자동으로:
1. AI 에이전트가 코멘트 분석
2. 코드 수정 자동 적용
3. 변경사항 커밋 및 푸시
4. PR에 응답 코멘트 작성

### CI 실패 자동 수정

CI가 실패하면 자동으로:
1. CI 로그 수집 및 분석
2. AI 에이전트가 문제 파악 및 수정
3. 수정사항 커밋 및 푸시
4. CI 재실행

## 🗄️ 데이터베이스 스키마

### tasks 테이블
```sql
CREATE TABLE tasks (
    id VARCHAR(255) PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    prompt TEXT NOT NULL,
    task_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    dependencies TEXT[] NOT NULL DEFAULT '{}',
    repository_owner VARCHAR(255) NOT NULL,
    repository_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    pr_url TEXT,
    workflow_run_id VARCHAR(255),
    error TEXT,
    auto_approve BOOLEAN NOT NULL DEFAULT FALSE
);
```

### composite_tasks 테이블
```sql
CREATE TABLE composite_tasks (
    id VARCHAR(255) PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    auto_approve BOOLEAN NOT NULL DEFAULT FALSE,
    repository_owner VARCHAR(255) NOT NULL,
    repository_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ
);
```

### execution_logs 테이블
```sql
CREATE TABLE execution_logs (
    id SERIAL PRIMARY KEY,
    task_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    message TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);
```

### metrics 테이블
```sql
CREATE TABLE metrics (
    id SERIAL PRIMARY KEY,
    task_id VARCHAR(255) NOT NULL,
    execution_time_ms BIGINT NOT NULL,
    files_changed INTEGER NOT NULL DEFAULT 0,
    lines_added INTEGER NOT NULL DEFAULT 0,
    lines_removed INTEGER NOT NULL DEFAULT 0,
    ai_tokens_used INTEGER NOT NULL DEFAULT 0,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);
```

## 🧪 테스트

```bash
# 모든 테스트 실행
cargo test

# 특정 테스트 실행
cargo test test_task_decomposition

# 통합 테스트
cargo test --test integration

# 커버리지 확인
cargo tarpaulin --out Html
```

## 📈 모니터링 및 메트릭

### 수집되는 메트릭

- 작업 실행 시간
- 변경된 파일 수
- 추가/삭제된 코드 라인 수
- 사용된 AI 토큰 수
- 성공/실패율
- 평균 응답 시간

### 통계 조회

```bash
# CLI로 통계 보기
autodev stats

# API로 통계 조회
curl http://localhost:3000/stats
```

## 🔐 보안 고려사항

1. **GitHub Token**: 최소 권한 원칙 적용
   - `repo`: 저장소 접근
   - `workflow`: Actions 트리거
   - `write:discussion`: PR 코멘트 작성

2. **API Key**: 환경 변수로 관리, 절대 코드에 포함하지 않음

3. **Database**: SSL/TLS 연결 사용 권장

4. **API**: CORS 설정 및 인증 미들웨어 추가 권장

## 🐳 Docker 배포

```bash
# 이미지 빌드
docker build -t autodev:latest .

# Docker Compose로 실행
docker-compose up -d

# 로그 확인
docker-compose logs -f autodev
```

## 🤝 기여 방법

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 라이선스

MIT License - 자유롭게 사용, 수정, 배포 가능

## 🙏 감사의 말

이 프로젝트는 Delino AutoDev의 아키텍처를 참고하여 만들어졌습니다.

## 📞 지원 및 문의

- Issues: GitHub Issues 사용
- Discussions: GitHub Discussions 활용
- Email: support@autodev.example.com

---

## 📚 추가 문서

- [API 문서](docs/api.md)
- [아키텍처 가이드](docs/architecture.md)
- [개발 가이드](docs/development.md)
- [배포 가이드](docs/deployment.md)

## 🎯 로드맵

- [ ] v0.1.0: 기본 기능 구현
- [ ] v0.2.0: 웹 UI 추가
- [ ] v0.3.0: 다양한 AI 에이전트 지원
- [ ] v0.4.0: 플러그인 시스템
- [ ] v1.0.0: 프로덕션 준비 완료

## 💡 사용 팁

### 효과적인 Prompt 작성

```
좋은 예시:
"Add JWT authentication to the API. Include:
- Login endpoint with email/password
- Logout endpoint
- Token refresh mechanism
- Proper error handling for invalid credentials
- Unit tests for all endpoints"

나쁜 예시:
"Add authentication"
```

### CompositeTask 최적화

```
좋은 예시:
"Review all RPC methods and fix security issues.
Create one task per RPC method.
Each task should be independent."

나쁜 예시:
"Fix all security issues in the codebase"
```

## 🔍 트러블슈팅

### 문제: GitHub Actions가 트리거되지 않음
- GitHub Token 권한 확인
- Workflow 파일 문법 확인
- Repository Settings에서 Actions 활성화 확인

### 문제: AI 에이전트 실행 실패
- API 키 확인
- 네트워크 연결 확인
- AI 에이전트 CLI 설치 확인

### 문제: 데이터베이스 연결 오류
- DATABASE_URL 확인
- PostgreSQL 서버 실행 확인
- 방화벽 설정 확인