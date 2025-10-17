use crate::{Repository, Result};
use octocrab::params::repos::Reference;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Clone)]
pub struct GitHubClient {
    client: Octocrab,
}

impl GitHubClient {
    pub fn new(token: String) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()?;

        Ok(Self { client })
    }

    /// Trigger a GitHub Actions workflow
    pub async fn trigger_workflow(
        &self,
        repo: &Repository,
        workflow_file: &str,
        inputs: HashMap<String, String>,
    ) -> Result<String> {
        tracing::info!(
            "Triggering workflow {} for {}/{}",
            workflow_file,
            repo.owner,
            repo.name
        );

        // Using octocrab for workflow dispatch (octocrab 0.32 API)
        // Convert HashMap to serde_json::Value
        let inputs_json = json!(inputs);

        self.client
            .actions()
            .create_workflow_dispatch(&repo.owner, &repo.name, workflow_file, &repo.branch)
            .inputs(inputs_json)
            .send()
            .await?;

        // Generate a workflow run ID
        let workflow_run_id = format!("run_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));

        tracing::info!("Workflow triggered: {}", workflow_run_id);

        Ok(workflow_run_id)
    }

    /// Check workflow status
    pub async fn check_workflow_status(
        &self,
        repo: &Repository,
        run_id: &str,
    ) -> Result<WorkflowStatus> {
        // Get workflow run status
        let runs = self
            .client
            .workflows(&repo.owner, &repo.name)
            .list_runs(run_id)
            .send()
            .await?;

        if let Some(run) = runs.items.first() {
            Ok(WorkflowStatus {
                status: run.status.to_string(),
                conclusion: run.conclusion.as_ref().map(|c| c.to_string()),
            })
        } else {
            Ok(WorkflowStatus {
                status: "unknown".to_string(),
                conclusion: None,
            })
        }
    }

    /// Create a pull request
    pub async fn create_pull_request(
        &self,
        repo: &Repository,
        title: String,
        body: String,
        head: String,
        base: String,
        draft: bool,
    ) -> Result<PullRequest> {
        tracing::info!("Creating PR: {} ({} -> {}) [draft: {}]", title, head, base, draft);

        let pr = self
            .client
            .pulls(&repo.owner, &repo.name)
            .create(title, head, base)
            .body(body)
            .draft(draft)
            .send()
            .await?;

        Ok(PullRequest {
            number: pr.number,
            url: pr.html_url.map(|u| u.to_string()),
            title: pr.title.unwrap_or_default(),
        })
    }

    /// Add comment to PR
    pub async fn create_pr_comment(
        &self,
        repo: &Repository,
        pr_number: u32,
        comment: &str,
    ) -> Result<()> {
        tracing::info!("Adding comment to PR #{}", pr_number);

        self.client
            .issues(&repo.owner, &repo.name)
            .create_comment(pr_number as u64, comment)
            .await?;

        Ok(())
    }

    /// Add comment to Issue
    ///
    /// GitHub API에서 PR과 Issue는 동일한 엔드포인트를 사용하지만
    /// 명확성을 위해 별도 메서드 제공
    pub async fn create_issue_comment(
        &self,
        repo: &Repository,
        issue_number: u32,
        comment: &str,
    ) -> Result<()> {
        tracing::info!("Adding comment to Issue #{}", issue_number);

        self.client
            .issues(&repo.owner, &repo.name)
            .create_comment(issue_number as u64, comment)
            .await?;

        Ok(())
    }

    /// Get pull request
    pub async fn get_pull_request(
        &self,
        repo: &Repository,
        pr_number: u32,
    ) -> Result<PullRequest> {
        let pr = self
            .client
            .pulls(&repo.owner, &repo.name)
            .get(pr_number as u64)
            .await?;

        Ok(PullRequest {
            number: pr.number,
            url: pr.html_url.map(|u| u.to_string()),
            title: pr.title.unwrap_or_default(),
        })
    }

    /// Merge a pull request
    pub async fn merge_pull_request(
        &self,
        repo: &Repository,
        pr_number: u64,
    ) -> Result<()> {
        tracing::info!("Merging PR #{} in {}/{}", pr_number, repo.owner, repo.name);

        self.client
            .pulls(&repo.owner, &repo.name)
            .merge(pr_number)
            .send()
            .await?;

        tracing::info!("✓ PR #{} merged successfully", pr_number);

        Ok(())
    }

    /// List repository workflows
    pub async fn list_workflows(&self, repo: &Repository) -> Result<Vec<String>> {
        let workflows = self
            .client
            .workflows(&repo.owner, &repo.name)
            .list()
            .send()
            .await?;

        Ok(workflows
            .items
            .iter()
            .map(|w| w.name.clone())
            .collect())
    }

    /// Create a branch
    pub async fn create_branch(
        &self,
        repo: &Repository,
        branch_name: &str,
        from_branch: &str,
    ) -> Result<()> {
        tracing::info!(
            "Creating branch {} from {} in {}/{}",
            branch_name,
            from_branch,
            repo.owner,
            repo.name
        );

        // Get the ref of the source branch (octocrab 0.32 uses Reference enum)
        let source_ref = self
            .client
            .repos(&repo.owner, &repo.name)
            .get_ref(&Reference::Branch(from_branch.to_string()))
            .await?;

        // Extract SHA from the Object enum using pattern matching (octocrab 0.32)
        // Object is marked as non-exhaustive, so we need a wildcard pattern
        use octocrab::models::repos::Object;
        let sha = match &source_ref.object {
            Object::Commit { sha, .. } | Object::Tag { sha, .. } => sha.clone(),
            _ => return Err(anyhow::anyhow!("Unexpected object type in ref").into()),
        };

        // Create new branch
        self.client
            .repos(&repo.owner, &repo.name)
            .create_ref(
                &Reference::Branch(branch_name.to_string()),
                sha,
            )
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub status: String,
    pub conclusion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub url: Option<String>,
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let result = GitHubClient::new("test_token".to_string());
        assert!(result.is_ok());
    }
}