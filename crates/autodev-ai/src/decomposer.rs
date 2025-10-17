use crate::{agent::AIAgent, Result};
use autodev_core::Task;
use std::sync::Arc;

pub struct TaskDecomposer {
    agent: Arc<dyn AIAgent>,
}

impl TaskDecomposer {
    pub fn new(agent: Arc<dyn AIAgent>) -> Self {
        Self { agent }
    }

    /// Decompose a composite task into subtasks
    pub async fn decompose(&self, composite_prompt: &str) -> Result<Vec<Task>> {
        tracing::info!("Decomposing composite task");

        // Analyze the prompt to determine task type
        let prompt_lower = composite_prompt.to_lowercase();

        if prompt_lower.contains("translation") || prompt_lower.contains("translate") {
            self.decompose_translation(composite_prompt).await
        } else if prompt_lower.contains("security") || prompt_lower.contains("audit") {
            self.decompose_security_audit(composite_prompt).await
        } else if prompt_lower.contains("refactor") {
            self.decompose_refactoring(composite_prompt).await
        } else if prompt_lower.contains("test") || prompt_lower.contains("testing") {
            self.decompose_testing(composite_prompt).await
        } else {
            self.decompose_generic(composite_prompt).await
        }
    }

    async fn decompose_translation(&self, prompt: &str) -> Result<Vec<Task>> {
        tracing::debug!("Decomposing translation task");

        let pages = vec!["intro", "features", "api", "guide", "faq"];
        let languages = vec!["ko", "ja", "zh", "es"];

        let mut tasks = Vec::new();

        for page in &pages {
            for lang in &languages {
                tasks.push(Task::new(
                    format!("Translate {} page to {}", page, lang),
                    format!("Improve translation quality for {} page in {}", page, lang),
                    format!(
                        "Review and fix translations for {} page in {}. \
                         Ensure cultural appropriateness and technical accuracy. \
                         Do not use automated translation tools.",
                        page, lang
                    ),
                ));
            }
        }

        Ok(tasks)
    }

    async fn decompose_security_audit(&self, prompt: &str) -> Result<Vec<Task>> {
        tracing::debug!("Decomposing security audit task");

        // Extract RPC methods or endpoints from the prompt
        let methods = vec![
            "getUserData",
            "updateProfile",
            "deleteAccount",
            "processPayment",
            "resetPassword",
        ];

        let tasks: Vec<Task> = methods
            .iter()
            .map(|method| {
                Task::new(
                    format!("Security audit for {}", method),
                    format!("Review and fix security issues in {}", method),
                    format!(
                        "Analyze {} for security vulnerabilities including: \
                         - SQL injection \
                         - XSS attacks \
                         - Authentication bypass \
                         - Data exposure \
                         - Rate limiting \
                         Fix any issues found and add appropriate validation.",
                        method
                    ),
                )
            })
            .collect();

        Ok(tasks)
    }

    async fn decompose_refactoring(&self, prompt: &str) -> Result<Vec<Task>> {
        tracing::debug!("Decomposing refactoring task");

        let components = vec![
            ("database", "Database access layer"),
            ("api", "API endpoints"),
            ("auth", "Authentication system"),
            ("utils", "Utility functions"),
        ];

        let tasks: Vec<Task> = components
            .iter()
            .map(|(name, desc)| {
                Task::new(
                    format!("Refactor {}", name),
                    format!("Improve {} code quality", desc),
                    format!(
                        "Refactor {} to: \
                         - Improve code organization \
                         - Reduce complexity \
                         - Add proper error handling \
                         - Update to modern patterns \
                         - Improve performance",
                        desc
                    ),
                )
            })
            .collect();

        Ok(tasks)
    }

    async fn decompose_testing(&self, prompt: &str) -> Result<Vec<Task>> {
        tracing::debug!("Decomposing testing task");

        let test_types = vec![
            ("unit", "Unit tests for core functions"),
            ("integration", "Integration tests for API"),
            ("e2e", "End-to-end tests for critical flows"),
            ("performance", "Performance tests for bottlenecks"),
        ];

        let tasks: Vec<Task> = test_types
            .iter()
            .map(|(test_type, desc)| {
                Task::new(
                    format!("Add {} tests", test_type),
                    desc.to_string(),
                    format!(
                        "Create comprehensive {} with: \
                         - High code coverage \
                         - Edge case handling \
                         - Clear test descriptions \
                         - Proper assertions",
                        desc
                    ),
                )
            })
            .collect();

        Ok(tasks)
    }

    async fn decompose_generic(&self, _prompt: &str) -> Result<Vec<Task>> {
        tracing::debug!("Using generic decomposition");

        // For generic tasks, create a simple breakdown
        Ok(vec![
            Task::new(
                "Analyze requirements".to_string(),
                "Understand and document requirements".to_string(),
                "Analyze the requirements and create a detailed plan".to_string(),
            ),
            Task::new(
                "Implement core functionality".to_string(),
                "Build the main features".to_string(),
                "Implement the core functionality following best practices".to_string(),
            ),
            Task::new(
                "Add tests".to_string(),
                "Create comprehensive tests".to_string(),
                "Add unit and integration tests for the implementation".to_string(),
            ),
            Task::new(
                "Documentation".to_string(),
                "Update documentation".to_string(),
                "Create or update documentation for the new functionality".to_string(),
            ),
        ])
    }

    /// Analyze dependencies between tasks
    pub fn analyze_dependencies(&self, tasks: &mut Vec<Task>) {
        // For now, most tasks are independent
        // In a real implementation, this would use AI to determine dependencies

        // Example: Make documentation depend on implementation
        if let Some(doc_task) = tasks.iter_mut().find(|t| t.title.contains("Documentation")) {
            if let Some(impl_task) = tasks.iter().find(|t| t.title.contains("Implement")) {
                doc_task.dependencies = vec![impl_task.id.clone()];
            }
        }
    }
}