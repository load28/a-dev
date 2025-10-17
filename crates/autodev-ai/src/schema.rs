use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 작업 도메인 타입
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TaskDomain {
    Translation,
    Security,
    Refactoring,
    Testing,
    Documentation,
    Feature,
    Bugfix,
    Generic,
}

/// 복잡도 추정
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityEstimate {
    Low,
    Medium,
    High,
}

/// AI가 반환하는 작업 분해 결과 스키마
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDecompositionResponse {
    /// 작업 분석 요약
    pub analysis: String,

    /// 작업 도메인
    pub domain: TaskDomain,

    /// 예상 복잡도
    pub estimated_complexity: ComplexityEstimate,

    /// 세부 작업 목록
    pub tasks: Vec<TaskSchema>,

    /// 병렬 실행 가능한 배치 (각 배치는 동시 실행 가능한 작업 ID 목록)
    pub parallel_batches: Vec<Vec<String>>,

    /// Critical path (가장 긴 의존성 체인)
    pub critical_path: Vec<String>,

    /// 전체 예상 소요 시간 (분)
    pub total_estimated_minutes: u32,
}

/// AI가 생성하는 개별 작업 스키마
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSchema {
    /// 작업 ID (예: "task_1", "translate_home_ko")
    pub id: String,

    /// 작업 제목 (동사로 시작)
    pub title: String,

    /// 작업 상세 설명
    pub description: String,

    /// 의존하는 작업 ID 목록
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// 예상 소요 시간 (분)
    #[serde(default)]
    pub estimated_duration_minutes: u32,

    /// 작업 태그 (카테고리, 기술 스택 등)
    #[serde(default)]
    pub tags: Vec<String>,
}

impl TaskDecompositionResponse {
    /// 순환 의존성 검증
    pub fn validate_no_circular_dependencies(&self) -> Result<(), String> {
        use std::collections::{HashSet, HashMap};

        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        for task in &self.tasks {
            graph.insert(task.id.clone(), task.dependencies.clone());
        }

        for task in &self.tasks {
            let mut visited = HashSet::new();
            let mut rec_stack = HashSet::new();

            if self.has_cycle(&task.id, &graph, &mut visited, &mut rec_stack) {
                return Err(format!("순환 의존성 감지: {}", task.id));
            }
        }

        Ok(())
    }

    fn has_cycle(
        &self,
        task_id: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(task_id.to_string());
        rec_stack.insert(task_id.to_string());

        if let Some(dependencies) = graph.get(task_id) {
            for dep_id in dependencies {
                if !visited.contains(dep_id) {
                    if self.has_cycle(dep_id, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep_id) {
                    return true;
                }
            }
        }

        rec_stack.remove(task_id);
        false
    }

    /// 모든 의존성이 존재하는 작업인지 검증
    pub fn validate_dependencies_exist(&self) -> Result<(), String> {
        use std::collections::HashSet;

        let task_ids: HashSet<String> = self.tasks.iter().map(|t| t.id.clone()).collect();

        for task in &self.tasks {
            for dep_id in &task.dependencies {
                if !task_ids.contains(dep_id) {
                    return Err(format!(
                        "작업 '{}'의 의존성 '{}'가 존재하지 않습니다",
                        task.id, dep_id
                    ));
                }
            }
        }

        Ok(())
    }

    /// 모든 검증 수행
    pub fn validate(&self) -> Result<(), String> {
        self.validate_dependencies_exist()?;
        self.validate_no_circular_dependencies()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_circular_dependencies() {
        let response = TaskDecompositionResponse {
            analysis: "Test".to_string(),
            domain: TaskDomain::Generic,
            estimated_complexity: ComplexityEstimate::Low,
            tasks: vec![
                TaskSchema {
                    id: "task_1".to_string(),
                    title: "Task 1".to_string(),
                    description: "Description".to_string(),
                    dependencies: vec![],
                    estimated_duration_minutes: 30,
                    tags: vec![],
                },
                TaskSchema {
                    id: "task_2".to_string(),
                    title: "Task 2".to_string(),
                    description: "Description".to_string(),
                    dependencies: vec!["task_1".to_string()],
                    estimated_duration_minutes: 30,
                    tags: vec![],
                },
            ],
            parallel_batches: vec![],
            critical_path: vec![],
            total_estimated_minutes: 60,
        };

        assert!(response.validate().is_ok());
    }

    #[test]
    fn test_circular_dependency_detected() {
        let response = TaskDecompositionResponse {
            analysis: "Test".to_string(),
            domain: TaskDomain::Generic,
            estimated_complexity: ComplexityEstimate::Low,
            tasks: vec![
                TaskSchema {
                    id: "task_1".to_string(),
                    title: "Task 1".to_string(),
                    description: "Description".to_string(),
                    dependencies: vec!["task_2".to_string()],
                    estimated_duration_minutes: 30,
                    tags: vec![],
                },
                TaskSchema {
                    id: "task_2".to_string(),
                    title: "Task 2".to_string(),
                    description: "Description".to_string(),
                    dependencies: vec!["task_1".to_string()],
                    estimated_duration_minutes: 30,
                    tags: vec![],
                },
            ],
            parallel_batches: vec![],
            critical_path: vec![],
            total_estimated_minutes: 60,
        };

        assert!(response.validate().is_err());
    }

    #[test]
    fn test_missing_dependency_detected() {
        let response = TaskDecompositionResponse {
            analysis: "Test".to_string(),
            domain: TaskDomain::Generic,
            estimated_complexity: ComplexityEstimate::Low,
            tasks: vec![
                TaskSchema {
                    id: "task_1".to_string(),
                    title: "Task 1".to_string(),
                    description: "Description".to_string(),
                    dependencies: vec!["task_99".to_string()], // 존재하지 않음
                    estimated_duration_minutes: 30,
                    tags: vec![],
                },
            ],
            parallel_batches: vec![],
            critical_path: vec![],
            total_estimated_minutes: 30,
        };

        assert!(response.validate().is_err());
    }
}
