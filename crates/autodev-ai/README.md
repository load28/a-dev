# AutoDev AI - Claude 기반 AI 개발 자동화 시스템

AutoDev의 AI 모듈로, Anthropic Claude API를 사용하여 소프트웨어 개발의 다양한 측면을 자동화합니다.

## 🌐 한글 전용 AI 시스템

**이 프로젝트의 모든 AI 프롬프트와 응답은 한글로만 작동합니다.**
- 작업 분해부터 코드 리뷰, CI 수정, 보안 분석까지 모든 AI 기능이 한글 중심
- 전문 용어는 영어 유지 (JWT, API, CI/CD 등)
- **Claude 4.5 Sonnet 모델** 사용 (최신, 코딩 및 에이전트 작업 최적화)

## 주요 기능

### 🤖 AI 기반 작업 분해
- **100% AI 기반**: 패턴 매칭 없이 Claude API가 동적으로 작업 분해
- **의존성 자동 분석**: AI가 작업 간 의존 관계 추론
- **병렬 실행 최적화**: DAG 분석으로 최적 실행 계획 생성
- **한글 프롬프트**: 모든 지시와 응답이 한글

### 📝 코드 리뷰 자동화
- 시니어 개발자 수준의 코드 품질 검토
- 가독성, 성능, 보안, 테스트 커버리지 분석
- 구체적이고 실행 가능한 개선 방안 제시

### 🔧 CI/CD 자동 수정
- CI 파이프라인 실패 로그 분석
- 근본 원인 파악 및 수정 방안 제시
- 빌드, 테스트, 린트 실패 자동 해결

### 💬 커밋 메시지 자동 생성
- Conventional Commits 형식 준수
- 코드 변경사항 분석 후 의미있는 메시지 생성
- 한글 제목 + 영어 타입 (feat, fix, docs 등)

### 🔒 보안 분석
- 화이트햇 해커 수준의 보안 취약점 스캔
- OWASP Top 10, CWE 기준 체크리스트
- 인증, 권한, 입력 검증, 데이터 보호 분석
- 구체적인 수정 코드 예제 제공

### 📚 Few-shot Learning
- 도메인별 예제 데이터베이스 (번역, 보안, 리팩토링, 테스팅, 기능 개발, 버그 수정)
- 동적 예제 선택으로 맥락 인식 강화
- 모든 예제가 한글로 작성됨

### 🎯 프롬프트 엔지니어링
- **Chain-of-Thought**: 단계별 사고 과정 (분석 → 분해 → 의존성 → 최적화 → 검증)
- **Role-based**: 전문가 역할 부여 (프로젝트 매니저, 시니어 개발자, 보안 전문가 등)
- **Structured Output**: JSON Schema로 구조화된 응답 보장
- **한글 시스템 프롬프트**: 5개의 전문화된 한글 프롬프트 파일

### ✅ 안전성 보장
- 순환 의존성 자동 감지
- 존재하지 않는 의존성 검증
- 타입 안전한 스키마 기반 파싱

## 설치

```toml
[dependencies]
autodev-ai = { path = "../autodev-ai" }
autodev-core = { path = "../autodev-core" }
```

## 사용 방법

### 작업 분해

```rust
use autodev_ai::{ClaudeAgent, TaskDecomposer};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Claude Agent 생성
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let agent = Arc::new(ClaudeAgent::new(api_key));

    // 2. TaskDecomposer 생성
    let decomposer = TaskDecomposer::new(agent.clone());

    // 3. 작업 분해 실행 (한글 프롬프트)
    let tasks = decomposer
        .decompose("모든 문서 페이지를 한국어와 일본어로 번역해주세요")
        .await?;

    // 4. 결과 확인
    println!("생성된 작업: {}개", tasks.len());
    for task in &tasks {
        println!("  - {} (의존성: {:?})", task.title, task.dependencies);
    }

    Ok(())
}
```

### 코드 리뷰

```rust
use autodev_ai::{AIAgent, ClaudeAgent};

let agent = ClaudeAgent::new(api_key);

// PR diff와 리뷰 코멘트 분석
let pr_diff = "diff --git a/src/auth.rs...";
let comments = vec!["타입 힌트 추가 필요".to_string()];

let review_result = agent
    .review_code_changes(pr_diff, &comments)
    .await?;

println!("변경사항: {:?}", review_result.changes_made);
// 출력: ["타입 힌트 추가됨", "에러 핸들링 개선됨"]
```

### CI 실패 자동 수정

```rust
use autodev_ai::{AIAgent, ClaudeAgent};

let agent = ClaudeAgent::new(api_key);

// CI 로그 분석 및 수정 방안 제시
let ci_logs = "error[E0382]: borrow of moved value: `deps`...";

let fix_result = agent
    .fix_ci_failures(ci_logs)
    .await?;

println!("수정 내용: {:?}", fix_result.changes_made);
// 출력: ["borrow checker 오류 수정", "테스트 업데이트"]
```

### 커밋 메시지 생성

```rust
use autodev_ai::{AIAgent, ClaudeAgent};

let agent = ClaudeAgent::new(api_key);

let changes = "Added JWT refresh token logic...";

let commit_msg = agent
    .generate_commit_message(changes)
    .await?;

println!("{}", commit_msg);
// 출력:
// feat(auth): JWT 토큰 갱신 기능 추가
//
// 기존에는 토큰 만료 시 사용자가 다시 로그인해야 했으나,
// 리프레시 토큰을 통해 자동으로 액세스 토큰을 갱신하도록 개선.
```

### 보안 분석

```rust
use autodev_ai::{AIAgent, ClaudeAgent};

let agent = ClaudeAgent::new(api_key);

let code = r#"
fn login(password: &str) -> bool {
    password == "admin123"  // 하드코딩된 비밀번호!
}
"#;

let issues = agent
    .analyze_security(code, "rust")
    .await?;

for issue in issues {
    println!("[{}] {}", issue.severity, issue.title);
    println!("  → {}", issue.recommendation);
}
```

### 특정 Claude 모델 선택

```rust
// Claude 4.5 Sonnet (최신, 가장 강력, 코딩 최적화, 기본값)
let agent = Arc::new(ClaudeAgent::with_model(api_key, "4.5"));

// Claude 3.5 Sonnet (이전 버전, 안정적)
let agent = Arc::new(ClaudeAgent::with_model(api_key, "3.5"));

// Claude 3 Opus (가장 강력한 3세대 모델)
let agent = Arc::new(ClaudeAgent::with_model(api_key, "opus"));

// Claude 4.5 Haiku (빠르고 저렴)
let agent = Arc::new(ClaudeAgent::with_model(api_key, "haiku-4.5"));

// Claude 3 Haiku (레거시, 가장 저렴)
let agent = Arc::new(ClaudeAgent::with_model(api_key, "haiku"));
```

### 도메인별 예제

#### 번역 작업
```rust
let tasks = decomposer
    .decompose("home, about, pricing 페이지를 한국어, 일본어, 중국어로 번역")
    .await?;
// → 9개의 독립적인 병렬 작업 생성 (3개 페이지 × 3개 언어)
```

#### 보안 감사
```rust
let tasks = decomposer
    .decompose("인증, 권한 부여, 입력 검증을 포함한 포괄적인 보안 감사 수행")
    .await?;
// → 각 보안 영역별 독립 작업 + 최종 리포트 작업 (의존성 포함)
```

#### 기능 개발
```rust
let tasks = decomposer
    .decompose("아바타 업로드와 개인정보 설정을 포함한 사용자 프로필 관리 기능 추가")
    .await?;
// → DB 스키마 → API 구현 → UI 구현 → 테스트 (순차적 의존성)
```

## 작업 분해 응답 스키마

Claude가 반환하는 JSON 구조:

```json
{
  "analysis": "작업 분석 요약",
  "domain": "translation|security|refactoring|testing|documentation|feature|bugfix|generic",
  "estimated_complexity": "low|medium|high",
  "tasks": [
    {
      "id": "task_1",
      "title": "번역: Home 페이지 → 한국어",
      "description": "Home 페이지의 모든 텍스트 콘텐츠를 한국어로 번역",
      "dependencies": [],
      "estimated_duration_minutes": 30,
      "tags": ["translation", "korean"]
    }
  ],
  "parallel_batches": [
    ["task_1", "task_2", "task_3"]
  ],
  "critical_path": ["task_1", "task_4"],
  "total_estimated_minutes": 120
}
```

## 프롬프트 구조

### 시스템 프롬프트 파일 (모두 한글)

1. **[`task_decomposition_system.txt`](prompts/task_decomposition_system.txt)** - 작업 분해
   - 역할: 프로젝트 매니저 + 소프트웨어 아키텍트
   - Chain-of-Thought: 분석 → 분해 → 의존성 → 최적화 → 검증
   - 8개 도메인별 가이드라인

2. **[`code_review_system.txt`](prompts/code_review_system.txt)** - 코드 리뷰
   - 역할: 10년 경력 시니어 개발자
   - 체크리스트: 코드 품질, 성능, 보안, 테스트, 문서화
   - Conventional Comments 형식

3. **[`ci_fix_system.txt`](prompts/ci_fix_system.txt)** - CI 실패 수정
   - 역할: CI/CD 파이프라인 전문가 + DevOps 엔지니어
   - 실패 분류: 빌드, 테스트, 린트, 환경, 타임아웃, 플랫폼
   - 근본 원인 분석 및 재발 방지 방안

4. **[`commit_message_system.txt`](prompts/commit_message_system.txt)** - 커밋 메시지
   - 역할: Git 커밋 메시지 작성 전문가
   - Conventional Commits 형식 (feat, fix, docs 등)
   - 한글 본문 + 영어 타입 혼용

5. **[`security_analysis_system.txt`](prompts/security_analysis_system.txt)** - 보안 분석
   - 역할: 애플리케이션 보안 전문가 + 화이트햇 해커
   - 체크리스트: 인증, 권한, 입력 검증, 데이터 보호, 비즈니스 로직, 의존성
   - OWASP, CWE 기준 분석

### Few-shot Examples
[`prompts/few_shot_examples.json`](prompts/few_shot_examples.json)에 6개 도메인 예제 포함 (모두 한글):
- **Translation**: 20개 병렬 작업 (5개 페이지 × 4개 언어)
- **Security**: 5개 독립 작업 + 통합 리포트
- **Feature**: DB → API → UI → 테스트 (순차 의존성)
- **Refactoring**: 마이크로서비스 분리 (복잡한 의존성)
- **Bugfix**: 재현 → 분석 → 수정 → 검증
- **Testing**: 단위 → 통합 → E2E 테스트 커버리지 향상

## 아키텍처

```
autodev-ai/
├── prompts/                              # 한글 프롬프트 파일 (외부화)
│   ├── task_decomposition_system.txt    # 작업 분해 시스템 프롬프트
│   ├── code_review_system.txt           # 코드 리뷰 시스템 프롬프트
│   ├── ci_fix_system.txt                # CI 수정 시스템 프롬프트
│   ├── commit_message_system.txt        # 커밋 메시지 시스템 프롬프트
│   ├── security_analysis_system.txt     # 보안 분석 시스템 프롬프트
│   ├── few_shot_examples.json           # Few-shot 예제 DB (한글)
│   └── task_decomposition_schema.json   # JSON Schema 정의
│
├── src/
│   ├── agent.rs                         # AIAgent trait + BaseAgent
│   │   ├── build_task_prompt()         # 한글 프롬프트 조합
│   │   ├── build_review_prompt()       # 한글 프롬프트 조합
│   │   └── build_ci_fix_prompt()       # 한글 프롬프트 조합
│   │
│   ├── claude.rs                        # ClaudeAgent 구현
│   │   ├── call_api()                  # Claude API 호출
│   │   ├── chat_json()                 # JSON mode (system prompt 지원)
│   │   ├── generate_commit_message()   # 한글 커밋 메시지 생성
│   │   └── analyze_security()          # 한글 보안 리포트 생성
│   │
│   ├── decomposer.rs                    # TaskDecomposer (AI 기반)
│   │   ├── ExampleDatabase             # Few-shot 예제 관리
│   │   │   ├── load()                  # JSON에서 예제 로드
│   │   │   ├── detect_domain()         # 키워드 기반 도메인 감지
│   │   │   └── find_relevant()         # 도메인별 예제 검색
│   │   └── decompose()                 # 한글 프롬프트 → Claude → JSON 파싱
│   │
│   └── schema.rs                        # 타입 안전 스키마
│       ├── TaskDecompositionResponse
│       │   ├── validate_dependencies_exist()
│       │   └── validate_no_cycles()
│       └── TaskDomain (enum)
│
└── Cargo.toml
    └── dependencies: reqwest, serde_json, async-trait
```

**데이터 흐름 (작업 분해 예시)**:
```
사용자 입력 (한글)
  ↓
TaskDecomposer::decompose()
  ↓
ExampleDatabase::detect_domain() - 도메인 감지 (키워드 매칭)
  ↓
ExampleDatabase::find_relevant() - Few-shot 예제 2개 선택
  ↓
System Prompt 로드 (include_str!)
  ↓
User Prompt 조합 (System + Examples + User Input)
  ↓
ClaudeAgent::chat_json() - Claude API 호출
  ↓
JSON 파싱 + 마크다운 제거
  ↓
TaskDecompositionResponse::validate() - 스키마 검증
  ↓
Vec<Task> 반환
```

## 환경 변수

```bash
# Anthropic API 키 (필수)
export ANTHROPIC_API_KEY=sk-ant-...

# 로그 레벨 (선택)
export RUST_LOG=debug
```

## 테스트

```bash
# 단위 테스트
cargo test --package autodev-ai

# 통합 테스트 (API 키 필요)
ANTHROPIC_API_KEY=sk-ant-... cargo test --package autodev-ai -- --ignored
```

## 성능

| 도메인 | 평균 소요 시간 | 생성 작업 수 | 비용 (예상) |
|--------|--------------|-------------|-----------|
| Translation | 3-5초 | 10-20 | $0.02-0.05 |
| Security | 5-8초 | 5-10 | $0.03-0.06 |
| Feature | 8-12초 | 6-15 | $0.05-0.10 |

*Claude 4.5 Sonnet 기준 ($3/M input, $15/M output)

### Claude 4.5 Sonnet 특징
- **1M 토큰 컨텍스트**: 대규모 코드베이스 처리 가능
- **64K 출력 토큰**: 긴 코드 생성 및 상세 분석 가능
- **코딩 최적화**: 복잡한 리팩토링 및 멀티 스텝 작업 처리 능력 향상
- **Prompt Caching**: 최대 90% 비용 절감 가능 (반복 프롬프트)

## 제한 사항

- **Claude 전용**: OpenAI 등 다른 LLM 미지원 (설계 결정)
- **API 키 필수**: Anthropic API 키 없이 작동 불가
- **한글 전용**: 모든 AI 프롬프트가 한글로 설계됨 (영어 입력은 가능하나 응답은 한글)
- **JSON 파싱 실패 가능**: AI가 잘못된 JSON 생성 시 재시도 로직 없음
- **네트워크 의존**: Claude API 호출 실패 시 graceful degradation 없음

## 로드맵

### v0.2.0 (안정성 개선)
- [ ] JSON 파싱 실패 시 자동 재시도 (최대 3회)
- [ ] API 호출 실패 시 exponential backoff
- [ ] 프롬프트 캐싱으로 비용 절감 (Anthropic Prompt Caching 활용)

### v0.3.0 (기능 확장)
- [ ] 커스텀 시스템 프롬프트 주입 API
- [ ] 사용자 정의 Few-shot 예제 추가 API
- [ ] 작업 분해 결과 캐싱 (동일 요청 재사용)

### v0.4.0 (고급 기능)
- [ ] 벡터 DB 기반 예제 검색 (RAG로 Few-shot 예제 자동 선택)
- [ ] 멀티 에이전트 협업 (작업 분해 → 리뷰 → 수정 파이프라인)
- [ ] 스트리밍 응답 지원 (실시간 진행 상황 표시)

## 라이선스

MIT License

## 기여

Pull Request 환영합니다! 특히 다음 영역:
- **프롬프트 개선**: 한글 시스템 프롬프트 품질 향상
- **Few-shot 예제 추가**: 새로운 도메인 예제 (DevOps, 데이터 분석 등)
- **성능 최적화**: API 호출 횟수 감소, 캐싱 전략
- **에러 핸들링**: JSON 파싱 실패, 네트워크 오류 복구
- **테스트 커버리지**: 통합 테스트, E2E 시나리오

### 프롬프트 수정 가이드

프롬프트 파일은 `prompts/` 디렉토리에 `.txt` 파일로 외부화되어 있습니다:

1. 파일 수정 후 바로 테스트 가능 (재컴파일 불필요)
2. 한글 유지 (전문 용어는 영어 허용)
3. 마크다운 체크리스트, 코드 블록 등 자유롭게 활용
4. 변경 후 실제 Claude API로 검증 필수

**예시**:
```bash
# prompts/code_review_system.txt 수정
vim crates/autodev-ai/prompts/code_review_system.txt

# 테스트 실행 (API 키 필요)
ANTHROPIC_API_KEY=sk-ant-... cargo test --package autodev-ai review_test
```
