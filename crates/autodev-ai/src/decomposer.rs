use crate::{agent::AIAgent, examples::ExampleDatabase, schema::TaskDecompositionResponse, Result};
use autodev_core::Task;
use std::sync::Arc;

pub struct TaskDecomposer {
    agent: Arc<dyn AIAgent>,
    example_db: ExampleDatabase,
    system_prompt: String,
}

impl TaskDecomposer {
    pub fn new(agent: Arc<dyn AIAgent>) -> Self {
        let system_prompt = include_str!("../prompts/task_decomposition_system.txt").to_string();
        let example_db = ExampleDatabase::new();

        Self {
            agent,
            example_db,
            system_prompt,
        }
    }

    /// AI 기반 작업 분해 (완전히 새로운 구현)
    pub async fn decompose(&self, composite_prompt: &str) -> Result<Vec<Task>> {
        tracing::info!("AI-based task decomposition started");

        // 1. 도메인 감지
        let detected_domain = self.example_db.detect_domain(composite_prompt);
        tracing::debug!("Detected domain: {:?}", detected_domain);

        // 2. 관련 예제 검색 (Few-shot learning)
        let relevant_examples = self.example_db.find_relevant_examples(composite_prompt, 2);

        // 3. Few-shot 프롬프트 구성
        let few_shot_prompt = self.build_few_shot_prompt(&relevant_examples);

        // 4. 최종 사용자 프롬프트 구성
        let full_user_prompt = format!(
            "{}\n\n---\n\n사용자 요청:\n{}",
            few_shot_prompt, composite_prompt
        );

        // 5. AI 호출 (JSON mode)
        let json_response = self
            .agent
            .chat_json(&self.system_prompt, &full_user_prompt)
            .await?;

        tracing::debug!("AI JSON response: {}", json_response);

        // 6. JSON 파싱
        let decomposition: TaskDecompositionResponse = serde_json::from_str(&json_response)
            .map_err(|e| {
                crate::Error::ParseError(format!("Failed to parse AI response: {}. Response: {}", e, json_response))
            })?;

        // 7. 검증
        decomposition.validate().map_err(|e| {
            crate::Error::ValidationError(format!("Task decomposition validation failed: {}", e))
        })?;

        tracing::info!(
            "Successfully decomposed into {} tasks across {} batches",
            decomposition.tasks.len(),
            decomposition.parallel_batches.len()
        );

        // 8. TaskSchema → Task 변환
        let tasks = self.convert_to_tasks(decomposition.tasks);

        Ok(tasks)
    }

    /// Few-shot 프롬프트 구성
    fn build_few_shot_prompt(
        &self,
        examples: &[&crate::examples::FewShotExample],
    ) -> String {
        if examples.is_empty() {
            return String::new();
        }

        let mut prompt = String::from("다음은 작업 분해의 예시입니다:\n\n");

        for (i, example) in examples.iter().enumerate() {
            prompt.push_str(&format!("## 예시 {}:\n\n", i + 1));
            prompt.push_str(&format!("사용자 요청:\n{}\n\n", example.user_prompt));
            prompt.push_str("AI 응답:\n");

            // JSON 직렬화
            let json_response = serde_json::to_string_pretty(&example.assistant_response)
                .unwrap_or_else(|_| "{}".to_string());

            prompt.push_str(&format!("```json\n{}\n```\n\n", json_response));
        }

        prompt
    }

    /// TaskSchema를 Task로 변환
    fn convert_to_tasks(&self, schemas: Vec<crate::schema::TaskSchema>) -> Vec<Task> {
        schemas
            .into_iter()
            .map(|schema| {
                let mut task = Task::new(
                    schema.title,
                    schema.description.clone(),
                    schema.description,
                );

                // ID 덮어쓰기 (AI가 생성한 ID 사용)
                task.id = schema.id;

                // 의존성 설정
                task.dependencies = schema.dependencies;

                task
            })
            .collect()
    }

    /// 레거시 메서드: 하위 호환성 유지 (내부적으로 AI 분해 사용)
    #[deprecated(note = "Use decompose() instead. This method now uses AI-based decomposition internally.")]
    pub async fn decompose_translation(&self, prompt: &str) -> Result<Vec<Task>> {
        self.decompose(prompt).await
    }

    #[deprecated(note = "Use decompose() instead")]
    pub async fn decompose_security_audit(&self, prompt: &str) -> Result<Vec<Task>> {
        self.decompose(prompt).await
    }

    #[deprecated(note = "Use decompose() instead")]
    pub async fn decompose_refactoring(&self, prompt: &str) -> Result<Vec<Task>> {
        self.decompose(prompt).await
    }

    #[deprecated(note = "Use decompose() instead")]
    pub async fn decompose_testing(&self, prompt: &str) -> Result<Vec<Task>> {
        self.decompose(prompt).await
    }

    #[deprecated(note = "Use decompose() instead")]
    pub async fn decompose_generic(&self, prompt: &str) -> Result<Vec<Task>> {
        self.decompose(prompt).await
    }

    /// 레거시 메서드: 이제 AI가 의존성까지 분석하므로 불필요
    #[deprecated(note = "Dependencies are now analyzed by AI during decomposition")]
    pub fn analyze_dependencies(&self, _tasks: &mut Vec<Task>) {
        // AI가 이미 의존성을 분석했으므로 아무것도 하지 않음
        tracing::warn!("analyze_dependencies() is deprecated. AI now handles dependency analysis.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claude::ClaudeAgent;

    #[tokio::test]
    #[ignore] // API 키 필요
    async fn test_ai_decomposition() {
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
        let agent = Arc::new(ClaudeAgent::new(api_key));
        let decomposer = TaskDecomposer::new(agent);

        let result = decomposer
            .decompose("Translate documentation pages to Korean and Japanese")
            .await;

        assert!(result.is_ok());
        let tasks = result.unwrap();
        assert!(!tasks.is_empty());

        println!("Generated {} tasks:", tasks.len());
        for task in &tasks {
            println!("  - {} (deps: {:?})", task.title, task.dependencies);
        }
    }
}
