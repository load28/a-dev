use crate::agent::AIAgent;
use crate::schema::{DomainDetectionResponse, ExampleRankingResponse, TaskDomain, TaskDecompositionResponse};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Few-shot 학습을 위한 예제
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FewShotExample {
    pub domain: TaskDomain,
    pub user_prompt: String,
    pub assistant_response: TaskDecompositionResponse,
}

/// Few-shot 예제 데이터베이스
pub struct ExampleDatabase {
    examples: Vec<FewShotExample>,
    domain_index: HashMap<TaskDomain, Vec<usize>>,
    agent: Option<Arc<dyn AIAgent>>,
}

impl ExampleDatabase {
    /// 내장된 예제로 데이터베이스 초기화
    pub fn new() -> Self {
        let examples = Self::load_builtin_examples();
        let domain_index = Self::build_domain_index(&examples);

        Self {
            examples,
            domain_index,
            agent: None,
        }
    }

    /// AI agent와 함께 데이터베이스 초기화
    pub fn with_agent(agent: Arc<dyn AIAgent>) -> Self {
        let examples = Self::load_builtin_examples();
        let domain_index = Self::build_domain_index(&examples);

        Self {
            examples,
            domain_index,
            agent: Some(agent),
        }
    }

    /// 내장 예제 로드
    fn load_builtin_examples() -> Vec<FewShotExample> {
        let json_data = include_str!("../prompts/few_shot_examples.json");
        serde_json::from_str(json_data).expect("Failed to parse few_shot_examples.json")
    }

    /// 도메인별 인덱스 구축
    fn build_domain_index(examples: &[FewShotExample]) -> HashMap<TaskDomain, Vec<usize>> {
        let mut index: HashMap<TaskDomain, Vec<usize>> = HashMap::new();

        for (i, example) in examples.iter().enumerate() {
            index
                .entry(example.domain.clone())
                .or_insert_with(Vec::new)
                .push(i);
        }

        index
    }

    /// 도메인에 맞는 예제 검색
    pub fn find_by_domain(&self, domain: &TaskDomain) -> Vec<&FewShotExample> {
        self.domain_index
            .get(domain)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.examples.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// AI 기반 의미론적 유사도로 예제 선택
    pub async fn find_relevant_examples_with_ai(&self, user_prompt: &str, limit: usize) -> Result<Vec<&FewShotExample>> {
        let agent = self.agent.as_ref().ok_or_else(|| {
            crate::Error::ConfigError("ExampleDatabase initialized without AI agent".to_string())
        })?;

        let system_prompt = include_str!("../prompts/example_selection_system.txt").to_string();

        // 예제 목록 포맷팅
        let examples_list = self
            .examples
            .iter()
            .enumerate()
            .map(|(i, ex)| {
                format!(
                    "{}. [도메인: {:?}] {}",
                    i, ex.domain, ex.user_prompt
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let user_message = format!(
            "사용자 요청: {}\n\n제공된 예제:\n{}\n\n가장 유사한 {}개의 예제를 선택하세요.",
            user_prompt, examples_list, limit
        );

        tracing::debug!("AI 예제 선택 시작");

        let json_response = agent.chat_json(&system_prompt, &user_message).await?;

        let ranking: ExampleRankingResponse = serde_json::from_str(&json_response)
            .map_err(|e| crate::Error::ParseError(format!("Failed to parse example ranking response: {}", e)))?;

        // 인덱스 유효성 검증
        let selected_examples: Vec<&FewShotExample> = ranking
            .selected_indices
            .iter()
            .filter_map(|&idx| self.examples.get(idx))
            .collect();

        tracing::info!(
            "AI 예제 선택 완료: {}개 선택 (점수: {:?})",
            selected_examples.len(),
            ranking.scores
        );

        for (i, reason) in ranking.reasoning.iter().enumerate() {
            tracing::debug!("  선택 근거 {}: {}", i + 1, reason);
        }

        Ok(selected_examples)
    }

    /// 키워드 기반으로 가장 관련성 높은 예제 찾기 (fallback)
    pub fn find_relevant_examples_fallback(&self, user_prompt: &str, limit: usize) -> Vec<&FewShotExample> {
        let mut scored_examples: Vec<(usize, &FewShotExample)> = self
            .examples
            .iter()
            .map(|example| {
                let score = self.calculate_relevance_score(user_prompt, example);
                (score, example)
            })
            .collect();

        // 점수 내림차순 정렬
        scored_examples.sort_by(|a, b| b.0.cmp(&a.0));

        scored_examples
            .into_iter()
            .take(limit)
            .map(|(_, example)| example)
            .collect()
    }

    /// 예제 선택 (AI 우선, 실패 시 fallback)
    pub async fn find_relevant_examples(&self, user_prompt: &str, limit: usize) -> Vec<&FewShotExample> {
        match self.find_relevant_examples_with_ai(user_prompt, limit).await {
            Ok(examples) => examples,
            Err(e) => {
                tracing::warn!("AI 예제 선택 실패, fallback 사용: {}", e);
                self.find_relevant_examples_fallback(user_prompt, limit)
            }
        }
    }

    /// 간단한 키워드 매칭으로 관련성 점수 계산
    fn calculate_relevance_score(&self, user_prompt: &str, example: &FewShotExample) -> usize {
        let user_lower = user_prompt.to_lowercase();
        let example_lower = example.user_prompt.to_lowercase();

        // 단어 토큰화
        let user_words: Vec<&str> = user_lower.split_whitespace().collect();
        let example_words: Vec<&str> = example_lower.split_whitespace().collect();

        // 공통 단어 개수 계산
        let mut score = 0;
        for user_word in &user_words {
            if example_words.contains(&user_word) {
                score += 1;
            }
        }

        score
    }

    /// AI 기반 도메인 감지 (한글/영어 모두 지원)
    pub async fn detect_domain_with_ai(&self, user_prompt: &str) -> Result<TaskDomain> {
        let agent = self.agent.as_ref().ok_or_else(|| {
            crate::Error::ConfigError("ExampleDatabase initialized without AI agent".to_string())
        })?;

        let system_prompt = include_str!("../prompts/domain_detection_system.txt").to_string();
        let user_message = format!("사용자 요청:\n{}", user_prompt);

        tracing::debug!("AI 도메인 감지 시작: {}", user_prompt);

        let json_response = agent.chat_json(&system_prompt, &user_message).await?;

        let detection: DomainDetectionResponse = serde_json::from_str(&json_response)
            .map_err(|e| crate::Error::ParseError(format!("Failed to parse domain detection response: {}", e)))?;

        tracing::info!(
            "도메인 감지 완료: {:?} (신뢰도: {:.2}, 근거: {})",
            detection.domain,
            detection.confidence,
            detection.reasoning
        );

        Ok(detection.domain)
    }

    /// 도메인 감지 (키워드 기반 fallback)
    pub fn detect_domain_fallback(&self, user_prompt: &str) -> TaskDomain {
        let prompt_lower = user_prompt.to_lowercase();

        // 한글 + 영어 키워드 매칭
        if prompt_lower.contains("translate") || prompt_lower.contains("translation")
            || prompt_lower.contains("번역") || prompt_lower.contains("다국어") {
            TaskDomain::Translation
        } else if prompt_lower.contains("security") || prompt_lower.contains("audit")
            || prompt_lower.contains("vulnerability") || prompt_lower.contains("보안")
            || prompt_lower.contains("취약점") || prompt_lower.contains("인증") {
            TaskDomain::Security
        } else if prompt_lower.contains("refactor") || prompt_lower.contains("refactoring")
            || prompt_lower.contains("리팩토링") || prompt_lower.contains("재구성") {
            TaskDomain::Refactoring
        } else if prompt_lower.contains("test") || prompt_lower.contains("testing")
            || prompt_lower.contains("coverage") || prompt_lower.contains("테스트")
            || prompt_lower.contains("커버리지") {
            TaskDomain::Testing
        } else if prompt_lower.contains("document") || prompt_lower.contains("documentation")
            || prompt_lower.contains("readme") || prompt_lower.contains("문서") {
            TaskDomain::Documentation
        } else if prompt_lower.contains("fix") || prompt_lower.contains("bug")
            || prompt_lower.contains("crash") || prompt_lower.contains("leak")
            || prompt_lower.contains("수정") || prompt_lower.contains("버그") {
            TaskDomain::Bugfix
        } else if prompt_lower.contains("feature") || prompt_lower.contains("implement")
            || prompt_lower.contains("add") || prompt_lower.contains("기능")
            || prompt_lower.contains("추가") || prompt_lower.contains("구현") {
            TaskDomain::Feature
        } else {
            TaskDomain::Generic
        }
    }

    /// 도메인 감지 (AI 우선, 실패 시 fallback)
    pub async fn detect_domain(&self, user_prompt: &str) -> TaskDomain {
        match self.detect_domain_with_ai(user_prompt).await {
            Ok(domain) => domain,
            Err(e) => {
                tracing::warn!("AI 도메인 감지 실패, fallback 사용: {}", e);
                self.detect_domain_fallback(user_prompt)
            }
        }
    }

    /// 모든 예제 가져오기
    pub fn all_examples(&self) -> &[FewShotExample] {
        &self.examples
    }
}

impl Default for ExampleDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_examples() {
        let db = ExampleDatabase::new();
        assert!(!db.all_examples().is_empty());
    }

    #[test]
    fn test_find_by_domain() {
        let db = ExampleDatabase::new();
        let translation_examples = db.find_by_domain(&TaskDomain::Translation);

        assert!(!translation_examples.is_empty());
        for example in translation_examples {
            assert_eq!(example.domain, TaskDomain::Translation);
        }
    }

    #[test]
    fn test_detect_domain() {
        let db = ExampleDatabase::new();

        assert_eq!(
            db.detect_domain("Translate all pages to Korean"),
            TaskDomain::Translation
        );

        assert_eq!(
            db.detect_domain("Perform security audit on API"),
            TaskDomain::Security
        );

        assert_eq!(
            db.detect_domain("Refactor authentication service"),
            TaskDomain::Refactoring
        );

        assert_eq!(
            db.detect_domain("Increase test coverage to 90%"),
            TaskDomain::Testing
        );

        assert_eq!(
            db.detect_domain("Fix memory leak in WebSocket handler"),
            TaskDomain::Bugfix
        );
    }

    #[test]
    fn test_find_relevant_examples() {
        let db = ExampleDatabase::new();
        let examples = db.find_relevant_examples("Translate documentation to multiple languages", 3);

        assert!(!examples.is_empty());
        assert!(examples.len() <= 3);
    }
}
