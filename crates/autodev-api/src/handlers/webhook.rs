use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde_json::Value;

use crate::state::ApiState;

pub async fn handle_github_webhook(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    // Get event type from headers
    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Get signature for verification
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    tracing::info!("Received GitHub webhook event: {}", event_type);

    // Verify signature (if webhook secret is configured)
    if let Ok(webhook_secret) = std::env::var("GITHUB_WEBHOOK_SECRET") {
        let payload_bytes = serde_json::to_vec(&payload).unwrap_or_default();

        if !autodev_github::WebhookHandler::verify_signature(
            &payload_bytes,
            signature,
            &webhook_secret,
        ) {
            tracing::warn!("Invalid webhook signature");
            return StatusCode::UNAUTHORIZED;
        }
    }

    // Parse event
    match autodev_github::WebhookHandler::parse_event(event_type, payload) {
        Ok(event) => {
            use autodev_github::WebhookEvent;

            match event {
                WebhookEvent::PullRequestOpened { pull_request, repository } => {
                    tracing::info!(
                        "PR opened: #{} - {}",
                        pull_request.number,
                        pull_request.title
                    );

                    // Handle new PR
                    handle_pr_opened(state, pull_request, repository).await;
                }
                WebhookEvent::PullRequestReviewSubmitted { review, pull_request, repository } => {
                    tracing::info!(
                        "PR review submitted: #{} - {}",
                        pull_request.number,
                        review.state
                    );

                    // Handle PR review
                    handle_pr_review(state, review, pull_request, repository).await;
                }
                WebhookEvent::WorkflowRun { workflow_run, repository } => {
                    tracing::info!(
                        "Workflow run: {} - {}",
                        workflow_run.name,
                        workflow_run.status
                    );

                    // Handle workflow completion
                    if workflow_run.status == "completed" {
                        handle_workflow_completion(state, workflow_run, repository).await;
                    }
                }
                WebhookEvent::IssueCommentCreated { comment, issue, repository } => {
                    tracing::info!(
                        "Issue comment created: #{} - {}",
                        issue.number,
                        comment.body.chars().take(50).collect::<String>()
                    );

                    // Check if comment starts with "autodev:"
                    if comment.body.trim().starts_with("autodev:") {
                        handle_issue_comment(state, comment, issue, repository).await;
                    }
                }
                _ => {
                    tracing::debug!("Unhandled webhook event type");
                }
            }

            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("Failed to parse webhook event: {}", e);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn handle_pr_opened(
    state: ApiState,
    pr: autodev_github::webhook::PullRequestPayload,
    repo: autodev_github::webhook::RepositoryPayload,
) {
    tracing::info!("Handling PR opened: #{} in {}", pr.number, repo.full_name);

    // Check if this is an AutoDev PR
    if pr.title.contains("[AutoDev]") || pr.body.as_ref().map_or(false, |b| b.contains("autodev")) {
        // Add a comment
        let github_repo = autodev_github::Repository::new(
            repo.owner.login.clone(),
            repo.name.clone(),
        );

        if let Err(e) = state.github_client
            .create_pr_comment(
                &github_repo,
                pr.number,
                "🤖 AutoDev is monitoring this PR and will handle reviews automatically.",
            )
            .await
        {
            tracing::error!("Failed to comment on PR: {}", e);
        }
    }
}

async fn handle_pr_review(
    state: ApiState,
    review: autodev_github::webhook::ReviewPayload,
    pr: autodev_github::webhook::PullRequestPayload,
    repo: autodev_github::webhook::RepositoryPayload,
) {
    tracing::info!("Handling PR review: #{} - {}", pr.number, review.state);

    // If review requests changes, handle with AI
    if review.state == "changes_requested" {
        if let Some(review_body) = review.body {
            let github_repo = autodev_github::Repository::new(
                repo.owner.login.clone(),
                repo.name.clone(),
            );

            // Get PR diff (simplified - in real implementation, fetch from GitHub)
            let pr_diff = ""; // Would fetch actual diff

            // Use AI to address review comments
            match state.ai_agent
                .review_code_changes(pr_diff, &[review_body])
                .await
            {
                Ok(result) => {
                    let comment = format!(
                        "📝 Addressing review feedback:\n\n{}\n\n✅ Changes made:\n{}",
                        result.comments.join("\n"),
                        result.changes_made.iter()
                            .map(|c| format!("- {}", c))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );

                    if let Err(e) = state.github_client
                        .create_pr_comment(&github_repo, pr.number, &comment)
                        .await
                    {
                        tracing::error!("Failed to respond to review: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to process review with AI: {}", e);
                }
            }
        }
    }
}

async fn handle_workflow_completion(
    state: ApiState,
    workflow: autodev_github::webhook::WorkflowRunPayload,
    _repo: autodev_github::webhook::RepositoryPayload,
) {
    tracing::info!(
        "Handling workflow completion: {} - {:?}",
        workflow.name,
        workflow.conclusion
    );

    // If workflow failed, try to fix with AI
    if workflow.conclusion == Some("failure".to_string()) {
        // In real implementation, fetch workflow logs
        let ci_logs = "Build failed: syntax error in main.rs";

        match state.ai_agent.fix_ci_failures(ci_logs).await {
            Ok(result) => {
                tracing::info!(
                    "AI suggested fixes for CI failure: {:?}",
                    result.changes_made
                );

                // Would create a new commit with fixes
                // This is simplified - real implementation would:
                // 1. Clone the repo
                // 2. Apply fixes
                // 3. Commit and push
                // 4. Update the PR
            }
            Err(e) => {
                tracing::error!("Failed to fix CI with AI: {}", e);
            }
        }
    }

    // Update task status in database
    if let Some(ref db) = state.db {
        // Extract task ID from workflow name or inputs
        // This is simplified - real implementation would parse properly
        if let Some(task_id) = extract_task_id(&workflow.name) {
            let status = if workflow.conclusion == Some("success".to_string()) {
                autodev_core::TaskStatus::Completed
            } else {
                autodev_core::TaskStatus::Failed
            };

            if let Err(e) = db.update_task_status(&task_id, status, None).await {
                tracing::error!("Failed to update task status: {}", e);
            }
        }
    }
}

fn extract_task_id(workflow_name: &str) -> Option<String> {
    // Extract task ID from workflow name
    // Format: "AutoDev - Task {task_id}"
    if workflow_name.starts_with("AutoDev - Task ") {
        Some(workflow_name.replace("AutoDev - Task ", ""))
    } else {
        None
    }
}

async fn handle_issue_comment(
    state: ApiState,
    comment: autodev_github::webhook::CommentPayload,
    issue: autodev_github::webhook::IssuePayload,
    repo: autodev_github::webhook::RepositoryPayload,
) {
    tracing::info!("Handling issue comment with autodev command: #{}", issue.number);

    // Parse "autodev:" prompt
    let prompt = match comment.body.trim().strip_prefix("autodev:") {
        Some(p) => p.trim(),
        None => {
            tracing::warn!("Comment does not start with 'autodev:' prefix");
            return;
        }
    };

    if prompt.is_empty() {
        tracing::warn!("Empty prompt after 'autodev:' prefix");

        // Send error comment to issue
        let github_repo = autodev_github::Repository::new(
            repo.owner.login.clone(),
            repo.name.clone(),
        );

        let error_msg = "❌ AutoDev 오류: 프롬프트가 비어있습니다.\n\n사용 예시:\n```\nautodev: Add Google OAuth authentication\n```";

        if let Err(e) = state.github_client
            .create_issue_comment(&github_repo, issue.number, error_msg)
            .await
        {
            tracing::error!("Failed to post error comment: {}", e);
        }

        return;
    }

    tracing::info!("Parsed prompt: {}", prompt);

    let github_repo = autodev_github::Repository::new(
        repo.owner.login.clone(),
        repo.name.clone(),
    );

    // Post acknowledgment comment
    let ack_msg = format!(
        "🤖 AutoDev 작업이 시작되었습니다.\n\n**작업 내용:** {}\n\n워크플로우 실행 상태는 [Actions 탭](https://github.com/{}/actions)에서 확인하실 수 있습니다.",
        prompt,
        repo.full_name
    );

    if let Err(e) = state.github_client
        .create_issue_comment(&github_repo, issue.number, &ack_msg)
        .await
    {
        tracing::error!("Failed to post acknowledgment comment: {}", e);
    }

    // Trigger workflow via GitHub Actions
    let mut inputs = std::collections::HashMap::new();
    inputs.insert("prompt".to_string(), prompt.to_string());
    inputs.insert("task_title".to_string(), format!("AutoDev: {}", prompt));
    inputs.insert("base_branch".to_string(), "main".to_string()); // TODO: Make configurable

    match state.github_client
        .trigger_workflow(&github_repo, "autodev.yml", inputs)
        .await
    {
        Ok(workflow_run_id) => {
            tracing::info!("Workflow triggered successfully: {}", workflow_run_id);

            // Optionally: Store task in database if available
            if let Some(ref db) = state.db {
                let task = autodev_core::Task::new(
                    format!("AutoDev: {}", prompt),
                    format!("Triggered from Issue #{}", issue.number),
                    prompt.to_string(),
                );

                if let Err(e) = db.save_task(&task, &repo.owner.login, &repo.name).await {
                    tracing::error!("Failed to store task in database: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to trigger workflow: {}", e);

            // Post error comment
            let error_msg = format!(
                "❌ AutoDev 워크플로우 트리거 실패\n\n**오류:** {}\n\n다음을 확인해주세요:\n\n- `.github/workflows/autodev.yml` 파일이 존재하는지\n- `ANTHROPIC_API_KEY` secret이 설정되어 있는지\n- GitHub Actions가 활성화되어 있는지",
                e
            );

            if let Err(e) = state.github_client
                .create_issue_comment(&github_repo, issue.number, &error_msg)
                .await
            {
                tracing::error!("Failed to post error comment: {}", e);
            }
        }
    }
}