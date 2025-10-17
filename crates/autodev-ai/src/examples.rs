use crate::schema::{TaskDomain, TaskDecompositionResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

impl ExampleDatabase {
    /// 내장된 예제로 데이터베이스 초기화
    pub fn new() -> Self {
        let examples = Self::load_builtin_examples();
        let domain_index = Self::build_domain_index(&examples);

        Self {
            examples,
            domain_index,
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

    /// 키워드 기반으로 가장 관련성 높은 예제 찾기
    pub fn find_relevant_examples(&self, user_prompt: &str, limit: usize) -> Vec<&FewShotExample> {
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

    /// 도메인 감지 (키워드 기반)
    pub fn detect_domain(&self, user_prompt: &str) -> TaskDomain {
        let prompt_lower = user_prompt.to_lowercase();

        // 도메인별 키워드 매칭
        if prompt_lower.contains("translate") || prompt_lower.contains("translation") {
            TaskDomain::Translation
        } else if prompt_lower.contains("security") || prompt_lower.contains("audit") || prompt_lower.contains("vulnerability") {
            TaskDomain::Security
        } else if prompt_lower.contains("refactor") || prompt_lower.contains("refactoring") {
            TaskDomain::Refactoring
        } else if prompt_lower.contains("test") || prompt_lower.contains("testing") || prompt_lower.contains("coverage") {
            TaskDomain::Testing
        } else if prompt_lower.contains("document") || prompt_lower.contains("documentation") || prompt_lower.contains("readme") {
            TaskDomain::Documentation
        } else if prompt_lower.contains("fix") || prompt_lower.contains("bug") || prompt_lower.contains("crash") || prompt_lower.contains("leak") {
            TaskDomain::Bugfix
        } else if prompt_lower.contains("feature") || prompt_lower.contains("implement") || prompt_lower.contains("add") {
            TaskDomain::Feature
        } else {
            TaskDomain::Generic
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
