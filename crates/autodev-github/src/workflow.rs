use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDispatch {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub inputs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowJob {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl WorkflowRun {
    pub fn is_completed(&self) -> bool {
        self.status == "completed"
    }

    pub fn is_successful(&self) -> bool {
        self.conclusion.as_ref().map_or(false, |c| c == "success")
    }

    pub fn is_failed(&self) -> bool {
        self.conclusion.as_ref().map_or(false, |c| c == "failure" || c == "cancelled")
    }
}