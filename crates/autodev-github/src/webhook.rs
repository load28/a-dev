use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum WebhookEvent {
    #[serde(rename = "opened")]
    PullRequestOpened {
        pull_request: PullRequestPayload,
        repository: RepositoryPayload,
    },
    #[serde(rename = "synchronize")]
    PullRequestSynchronize {
        pull_request: PullRequestPayload,
        repository: RepositoryPayload,
    },
    #[serde(rename = "closed")]
    PullRequestClosed {
        pull_request: PullRequestPayload,
        repository: RepositoryPayload,
    },
    #[serde(rename = "submitted")]
    PullRequestReviewSubmitted {
        review: ReviewPayload,
        pull_request: PullRequestPayload,
        repository: RepositoryPayload,
    },
    #[serde(rename = "created")]
    IssueCommentCreated {
        comment: CommentPayload,
        issue: IssuePayload,
        repository: RepositoryPayload,
    },
    WorkflowRun {
        workflow_run: WorkflowRunPayload,
        repository: RepositoryPayload,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestPayload {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub head: BranchInfo,
    pub base: BranchInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPayload {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: OwnerPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPayload {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewPayload {
    pub id: u64,
    pub body: Option<String>,
    pub state: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentPayload {
    pub id: u64,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuePayload {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunPayload {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub workflow_id: u64,
}

pub struct WebhookHandler;

impl WebhookHandler {
    /// Parse webhook payload
    pub fn parse_event(event_type: &str, payload: Value) -> Result<WebhookEvent> {
        match event_type {
            "pull_request" => {
                let action = payload["action"].as_str().unwrap_or("");
                match action {
                    "opened" => Ok(serde_json::from_value(payload)?),
                    "synchronize" => Ok(serde_json::from_value(payload)?),
                    "closed" => Ok(serde_json::from_value(payload)?),
                    _ => Err(crate::Error::UnsupportedEvent(action.to_string())),
                }
            }
            "pull_request_review" => Ok(serde_json::from_value(payload)?),
            "issue_comment" => Ok(serde_json::from_value(payload)?),
            "workflow_run" => Ok(serde_json::from_value(payload)?),
            _ => Err(crate::Error::UnsupportedEvent(event_type.to_string())),
        }
    }

    /// Verify GitHub webhook signature
    pub fn verify_signature(payload: &[u8], signature: &str, secret: &str) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("Invalid secret");
        mac.update(payload);

        let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        expected == signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_signature() {
        let payload = b"test payload";
        let secret = "my_secret";

        // This would be the actual signature from GitHub
        // For testing, we calculate it ourselves
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        assert!(WebhookHandler::verify_signature(payload, &signature, secret));
        assert!(!WebhookHandler::verify_signature(payload, "wrong_sig", secret));
    }
}