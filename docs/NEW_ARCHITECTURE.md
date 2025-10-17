# AutoDev 개선된 아키텍처

## 개요

이 문서는 AutoDev의 개선된 아키텍처를 설명합니다. 새로운 아키텍처는 AutoDev 서버의 지능형 작업 분해 기능을 완전히 활용하고, Claude API를 통한 병렬 실행을 지원합니다.

## 기존 아키텍처의 문제점

1. CLI/GitHub App → 직접 대상 저장소 워크플로우 실행
2. AutoDev 서버의 작업 분해 기능 미활용
3. Claude Code 의존성
4. 병렬 실행 불가능
5. 하위 작업 간 체계적인 PR 관리 부재

## 새로운 아키텍처

```
┌─────────────────────────────────────────────────────────────┐
│                  CLI / GitHub App Event                      │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              AutoDev Server (작업 분해)                      │
│  - AI 기반 작업 분해 (TaskDecomposer)                        │
│  - 의존성 분석 (Dependency Graph)                            │
│  - 병렬 배치 생성 (Parallel Batches)                         │
│  - 파일 겹침 방지 최적화                                      │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│           각 하위 작업별 워크플로우 디스패치                  │
│  - Batch 1: [Task A, Task B, Task C] → 병렬 실행             │
│  - Batch 2: [Task D]         → Batch 1 완료 후 실행         │
│  - Batch 3: [Task E, Task F] → Batch 2 완료 후 실행         │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│        Docker + Claude API로 작업 수행                       │
│  - GitHub Actions 워크플로우 내 Docker 실행                  │
│  - Claude API (claude-sonnet-4-5-20250929) 호출             │
│  - 독립적인 파일/모듈만 수정 (겹침 없음)                      │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│         하위 작업 PR → 상위 태스크 브랜치                     │
│  - subtask-1 → autodev/{composite_task_id}                  │
│  - subtask-2 → autodev/{composite_task_id}                  │
│  - ...                                                       │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│      AutoDev Server 콜백 (workflow-complete)                 │
│  - 작업 상태 업데이트                                         │
│  - 의존 작업 자동 시작                                        │
│  - 모든 하위 작업 완료 시 final PR 생성                       │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│    모든 하위 작업 완료 → 최종 PR to main                      │
│  - autodev/{composite_task_id} → main                       │
└─────────────────────────────────────────────────────────────┘
```

## 주요 개선 사항

### 1. 지능형 작업 분해

AutoDev 서버의 `TaskDecomposer`가 AI를 사용하여 작업을 지능적으로 분해합니다:

- **도메인 감지**: 번역, 보안, 리팩토링, 테스팅 등 자동 감지
- **의존성 분석**: 작업 간 의존성을 자동으로 파악
- **병렬 배치 생성**: 독립적인 작업을 그룹화하여 병렬 실행
- **파일 겹침 방지**: 각 작업이 수정하는 파일이 겹치지 않도록 최적화

### 2. 새로운 API 엔드포인트

#### POST /tasks/decompose

작업을 AI 기반으로 분해합니다.

**요청**:
```json
{
  "repository_owner": "myorg",
  "repository_name": "myrepo",
  "title": "번역 품질 개선",
  "description": "모든 문서 페이지의 번역 품질을 개선합니다",
  "composite_prompt": "docs/ 폴더의 모든 마크다운 파일을 검토하고 번역 품질을 개선하세요. 각 페이지별로 독립적으로 작업하세요."
}
```

**응답**:
```json
{
  "composite_task_id": "abc-123",
  "subtasks": [
    {
      "id": "task_1",
      "title": "docs/intro.md 번역 개선",
      "status": "Pending",
      "created_at": "2025-10-17T10:00:00Z"
    },
    {
      "id": "task_2",
      "title": "docs/guide.md 번역 개선",
      "status": "Pending",
      "created_at": "2025-10-17T10:00:00Z"
    }
  ],
  "parallel_batches": [
    ["task_1", "task_2"]  // 모두 병렬 실행 가능
  ],
  "total_estimated_minutes": 30
}
```

#### POST /tasks/:composite_task_id/orchestrate

복합 작업의 실행을 오케스트레이션합니다.

**요청**:
```json
{
  "repository_owner": "myorg",
  "repository_name": "myrepo",
  "base_branch": "main"
}
```

**응답**:
```json
{
  "composite_task_id": "abc-123",
  "started_subtasks": ["task_1", "task_2"],
  "message": "Started 2 subtasks from the first parallel batch"
}
```

#### POST /callbacks/workflow-complete

워크플로우 완료 시 콜백을 받습니다.

**요청**:
```json
{
  "task_id": "task_1",
  "composite_task_id": "abc-123",
  "repository_owner": "myorg",
  "repository_name": "myrepo",
  "pr_number": 123,
  "pr_url": "https://github.com/myorg/myrepo/pull/123",
  "success": true,
  "error": null
}
```

**응답**:
```json
{
  "message": "Task task_1 processed successfully",
  "next_tasks_started": ["task_3", "task_4"]
}
```

### 3. 새로운 워크플로우 템플릿

`templates/autodev-subtask.yml`은 Claude API를 사용하여 Docker 환경에서 작업을 수행합니다:

- **독립 실행**: 각 하위 작업이 독립적으로 실행
- **Claude 4.5 Sonnet 사용**: claude-sonnet-4-5-20250929 모델 사용 (코딩 및 에이전트 작업 최적화)
- **자동 콜백**: 작업 완료 시 AutoDev 서버에 자동 알림
- **PR 자동 생성**: 작업 완료 시 자동으로 PR 생성

### 4. 프롬프트 최적화

`task_decomposition_system.txt`에 작업 겹침 방지 규칙 추가:

```
6. 작업 겹침 절대 금지 (CRITICAL):
   - 각 작업이 수정하는 파일/모듈이 절대 겹치지 않도록 분해하세요
   - 동일한 파일을 여러 작업에서 수정하면 병합 충돌이 발생합니다
   - 파일 단위, 모듈 단위, 컴포넌트 단위로 명확히 분리하세요
   - 각 작업의 description에 "수정 대상 파일"을 명시하세요
```

## 사용 방법

### 1. 서버 설정

`.env` 파일 설정:

```env
# GitHub Configuration
GITHUB_TOKEN=github_pat_xxxxxxxxxxxxx

# AI Agent Configuration
ANTHROPIC_API_KEY=sk-ant-xxxxxxxxxxxxx

# API Server Configuration
API_PORT=3000
API_HOST=0.0.0.0
```

### 2. 서버 실행

```bash
cargo run --bin autodev-api
```

### 3. 대상 저장소 설정

대상 저장소에 워크플로우 파일 복사:

```bash
# 대상 저장소에서
mkdir -p .github/workflows
cp /path/to/a-dev/templates/autodev-subtask.yml .github/workflows/
```

GitHub Secrets 설정:
- `ANTHROPIC_API_KEY`: Claude API 키

### 4. 작업 생성 및 실행

#### cURL로 직접 호출

```bash
# 1. 작업 분해
RESPONSE=$(curl -X POST http://localhost:3000/tasks/decompose \
  -H "Content-Type: application/json" \
  -d '{
    "repository_owner": "myorg",
    "repository_name": "myrepo",
    "title": "문서 번역 개선",
    "description": "모든 문서 번역 품질 개선",
    "composite_prompt": "docs/ 폴더의 모든 마크다운 파일을 검토하고 번역 품질을 개선하세요."
  }')

echo $RESPONSE | jq .

# 2. 복합 작업 ID 추출
COMPOSITE_TASK_ID=$(echo $RESPONSE | jq -r '.composite_task_id')

# 3. 실행
curl -X POST "http://localhost:3000/tasks/$COMPOSITE_TASK_ID/orchestrate" \
  -H "Content-Type: application/json" \
  -d '{
    "repository_owner": "myorg",
    "repository_name": "myrepo",
    "base_branch": "main"
  }'
```

#### Python 스크립트 예제

```python
import requests

# 1. 작업 분해
response = requests.post(
    "http://localhost:3000/tasks/decompose",
    json={
        "repository_owner": "myorg",
        "repository_name": "myrepo",
        "title": "보안 감사",
        "description": "모든 API 엔드포인트 보안 감사",
        "composite_prompt": "src/api/ 폴더의 모든 엔드포인트를 검토하고 보안 이슈를 수정하세요. 각 파일별로 독립적으로 작업하세요."
    }
)

data = response.json()
composite_task_id = data["composite_task_id"]

print(f"Created composite task: {composite_task_id}")
print(f"Subtasks: {len(data['subtasks'])}")
print(f"Parallel batches: {data['parallel_batches']}")

# 2. 실행
response = requests.post(
    f"http://localhost:3000/tasks/{composite_task_id}/orchestrate",
    json={
        "repository_owner": "myorg",
        "repository_name": "myrepo",
        "base_branch": "main"
    }
)

result = response.json()
print(f"Started subtasks: {result['started_subtasks']}")
```

## 브랜치 전략

```
main
 │
 └─ autodev/{composite_task_id}  (parent branch)
     │
     ├─ autodev/{composite_task_id}/subtask-{task_1}
     │  └─ PR → autodev/{composite_task_id}
     │
     ├─ autodev/{composite_task_id}/subtask-{task_2}
     │  └─ PR → autodev/{composite_task_id}
     │
     └─ autodev/{composite_task_id}/subtask-{task_3}
        └─ PR → autodev/{composite_task_id}

     모든 하위 PR 머지 완료 후:
     autodev/{composite_task_id} → main (Final PR)
```

## 장점

1. **병렬 실행**: 독립적인 작업을 동시에 실행하여 속도 향상
2. **충돌 없음**: 파일 겹침 방지로 병합 충돌 최소화
3. **지능형 분해**: AI 기반 작업 분해로 최적의 작업 단위 생성
4. **자동화**: 의존성 기반 자동 실행 및 PR 관리
5. **유연성**: Claude API 사용으로 다양한 모델 선택 가능
6. **확장성**: 서버 기반 아키텍처로 여러 저장소 동시 관리 가능

## 제한 사항

1. **API 비용**: Claude API 사용으로 인한 비용 발생
2. **네트워크 의존성**: AutoDev 서버와의 통신 필요
3. **복잡도**: 초기 설정이 기존보다 복잡

## 향후 개선 방향

1. **웹 UI 추가**: 작업 모니터링 및 관리를 위한 대시보드
2. **다양한 AI 모델 지원**: GPT-4, Gemini 등 추가
3. **자동 테스트 생성**: 작업 완료 시 자동으로 테스트 코드 생성
4. **메트릭 수집**: 작업 성공률, 실행 시간 등 통계
5. **Webhook 통합**: PR 머지 이벤트 자동 감지 및 처리
