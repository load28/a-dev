use crate::{
    agent::{AIAgent, AgentResult, AgentType, ReviewResult},
    Result,
};
use async_trait::async_trait;
use autodev_core::Task;
use bollard::container::{Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions, WaitContainerOptions};
use bollard::Docker;
use futures_util::StreamExt;
use serde::Deserialize;

/// 마크다운 코드 블록 제거 헬퍼 함수
/// ```json\n{...}\n``` 또는 ```\n{...}\n``` 패턴을 순수 JSON으로 변환
fn strip_markdown_code_block(text: &str) -> &str {
    let trimmed = text.trim();

    // 마크다운 코드 블록으로 시작하는지 확인
    if trimmed.starts_with("```") {
        // 첫 번째 개행 찾기 (```json\n 또는 ```\n)
        if let Some(start) = trimmed.find('\n') {
            // 마지막 ``` 찾기
            if let Some(end) = trimmed.rfind("```") {
                if end > start {
                    return trimmed[start+1..end].trim();
                }
            }
        }
    }

    trimmed
}

/// Docker 컨테이너 기반 AI Executor
/// Claude Code CLI를 Docker 컨테이너에서 실행하여 OAuth 토큰으로 인증
pub struct DockerAIExecutor {
    docker: Docker,
    oauth_token: String,
    image: String,
}

impl DockerAIExecutor {
    pub fn new(oauth_token: String) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| crate::Error::ConfigError(format!("Failed to connect to Docker: {}", e)))?;

        Ok(Self {
            docker,
            oauth_token,
            image: "autodev-claude-executor:latest".to_string(),
        })
    }

    /// Docker 컨테이너에서 Claude Code 실행
    async fn execute_in_container(
        &self,
        system: &str,
        user: &str,
        json_mode: bool,
    ) -> Result<String> {
        // 1. 프롬프트 구성
        let full_prompt = if system.is_empty() {
            user.to_string()
        } else {
            format!("{}\\n\\n{}", system, user)
        };

        // 2. 명령어 구성
        let mut cmd = vec![
            "claude".to_string(),
            "--print".to_string(),
        ];

        if json_mode {
            cmd.push("--output-format".to_string());
            cmd.push("json".to_string()); // CLI 래퍼를 JSON으로

            // Worker 방식: append-system-prompt로 순수 JSON 응답 강제
            cmd.push("--append-system-prompt".to_string());
            cmd.push(
                "CRITICAL: You must respond with ONLY a valid JSON object. \
                No explanations, no markdown code blocks (```json), no additional text before or after. \
                Output the raw JSON object directly starting with { and ending with }.".to_string()
            );
        }

        cmd.push(full_prompt);

        // 3. 컨테이너 설정
        let container_name = format!("autodev-ai-{}", uuid::Uuid::new_v4());

        let config = Config {
            image: Some(self.image.clone()),
            cmd: Some(cmd),
            env: Some(vec![
                format!("CLAUDE_CODE_OAUTH_TOKEN={}", self.oauth_token),
            ]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(false),
            ..Default::default()
        };

        tracing::debug!("Creating Docker container for AI task: {}", container_name);

        // 4. 컨테이너 생성
        let container = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: container_name.clone(),
                    ..Default::default()
                }),
                config,
            )
            .await
            .map_err(|e| crate::Error::ApiError(format!("Failed to create container: {}", e)))?;

        // 5. 컨테이너 시작
        self.docker
            .start_container::<String>(&container.id, None)
            .await
            .map_err(|e| crate::Error::ApiError(format!("Failed to start container: {}", e)))?;

        tracing::debug!("Container started: {}", container.id);

        // 6. 로그 수집
        let mut output = String::new();
        let mut logs_stream = self.docker.logs(
            &container.id,
            Some(LogsOptions::<String> {
                stdout: true,
                stderr: true,
                follow: true,
                ..Default::default()
            }),
        );

        while let Some(log_result) = logs_stream.next().await {
            match log_result {
                Ok(log) => {
                    output.push_str(&log.to_string());
                }
                Err(e) => {
                    tracing::warn!("Error reading container logs: {}", e);
                    break;
                }
            }
        }

        // 7. 컨테이너 대기
        let wait_result = self
            .docker
            .wait_container(&container.id, None::<WaitContainerOptions<String>>);

        futures_util::pin_mut!(wait_result);

        while let Some(wait) = wait_result.next().await {
            match wait {
                Ok(wait_response) => {
                    tracing::debug!("Container exit code: {:?}", wait_response.status_code);
                }
                Err(e) => {
                    tracing::warn!("Error waiting for container: {}", e);
                }
            }
        }

        // 8. 컨테이너 삭제
        self.docker
            .remove_container(
                &container.id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
            .map_err(|e| {
                tracing::warn!("Failed to remove container: {}", e);
                crate::Error::ApiError(format!("Failed to remove container: {}", e))
            })?;

        tracing::debug!("Container removed: {}", container.id);

        // JSON 모드일 때 Claude CLI 래퍼 JSON에서 실제 응답 추출
        if json_mode {
            #[derive(Deserialize)]
            struct ClaudeCliResponse {
                result: String,
            }

            let parsed: ClaudeCliResponse = serde_json::from_str(output.trim())
                .map_err(|e| {
                    tracing::error!("Failed to parse Claude CLI JSON wrapper: {}\nRaw output: {}", e, output);
                    crate::Error::ParseError(format!("Failed to parse Claude CLI response: {}", e))
                })?;

            tracing::debug!("Extracted result from Claude CLI wrapper: {} chars", parsed.result.len());

            // 마크다운 코드 블록 제거 (방어적)
            let clean_result = strip_markdown_code_block(&parsed.result);

            if clean_result != parsed.result.trim() {
                tracing::debug!("Stripped markdown code block from result");
            }

            // JSON 유효성 검증
            serde_json::from_str::<serde_json::Value>(clean_result)
                .map_err(|e| {
                    tracing::error!("❌ Invalid JSON in result field: {}", e);
                    tracing::error!("Raw result (first 500 chars): {}",
                        &parsed.result.chars().take(500).collect::<String>());
                    tracing::error!("Cleaned result: {}", clean_result);
                    crate::Error::ParseError(format!(
                        "Claude returned invalid JSON despite prompt: {}",
                        e
                    ))
                })?;

            tracing::debug!("✓ Valid JSON extracted: {} chars", clean_result.len());
            Ok(clean_result.to_string())
        } else {
            Ok(output.trim().to_string())
        }
    }
}

#[async_trait]
impl AIAgent for DockerAIExecutor {
    fn agent_type(&self) -> AgentType {
        AgentType::Claude
    }

    async fn chat_json(&self, system: &str, user: &str) -> Result<String> {
        self.execute_in_container(system, user, true).await
    }

    async fn generate_commit_message(&self, _changes: &str) -> Result<String> {
        // Docker executor는 commit message 생성을 지원하지 않음
        Err(crate::Error::ConfigError(
            "Commit message generation not supported in Docker executor".to_string(),
        ))
    }

    async fn analyze_security(&self, _code: &str, _language: &str) -> Result<Vec<crate::agent::SecurityIssue>> {
        // Docker executor는 security analysis를 지원하지 않음
        Err(crate::Error::ConfigError(
            "Security analysis not supported in Docker executor".to_string(),
        ))
    }

    async fn execute_task(&self, _task: &Task, _repo_path: &str) -> Result<AgentResult> {
        // Docker executor는 task 실행을 지원하지 않음 (별도 Docker executor 사용)
        Err(crate::Error::ConfigError(
            "Task execution not supported in Docker AI executor".to_string(),
        ))
    }

    async fn review_code_changes(&self, _pr_diff: &str, _review_comments: &[String]) -> Result<ReviewResult> {
        // Docker executor는 code review를 지원하지 않음
        Err(crate::Error::ConfigError(
            "Code review changes not supported in Docker executor".to_string(),
        ))
    }

    async fn fix_ci_failures(&self, _ci_logs: &str) -> Result<ReviewResult> {
        // Docker executor는 CI failure 수정을 지원하지 않음
        Err(crate::Error::ConfigError(
            "CI failure fixing not supported in Docker executor".to_string(),
        ))
    }
}
