use anyhow::{anyhow, Result};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, WaitContainerOptions};
use bollard::models::{HostConfig, Mount, MountTypeEnum};
use serde::{Deserialize, Serialize};
use tokio::fs;
use futures_util::StreamExt;

use autodev_core::Task;
use autodev_github::Repository;

const WORKER_IMAGE: &str = "autodev-worker:latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub has_changes: bool,
    pub pr_number: Option<u64>,
    pub pr_url: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

pub struct DockerExecutor {
    docker: Docker,
    anthropic_api_key: String,
    github_token: String,
    autodev_server_url: Option<String>,
}

impl DockerExecutor {
    pub async fn new(
        anthropic_api_key: String,
        github_token: String,
        autodev_server_url: Option<String>,
    ) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;

        // Verify docker connection
        docker.ping().await?;

        Ok(Self {
            docker,
            anthropic_api_key,
            github_token,
            autodev_server_url,
        })
    }

    pub async fn execute_task(
        &self,
        task: &Task,
        repository: &Repository,
        base_branch: &str,
        target_branch: &str,
        composite_task_id: Option<&str>,
    ) -> Result<TaskResult> {
        tracing::info!(
            "Executing task {} in Docker container for {}/{}",
            task.id,
            repository.owner,
            repository.name
        );

        // Create temporary output directory
        let output_dir = format!("/tmp/autodev-output-{}", task.id);
        fs::create_dir_all(&output_dir).await?;

        // Build environment variables (as &str for bollard API)
        let env_strings = vec![
            format!("ANTHROPIC_API_KEY={}", self.anthropic_api_key),
            format!("GITHUB_TOKEN={}", self.github_token),
            format!("TASK_ID={}", task.id),
            format!("TASK_TITLE={}", task.title),
            format!("TASK_PROMPT={}", task.prompt),
            format!("REPO_OWNER={}", repository.owner),
            format!("REPO_NAME={}", repository.name),
            format!("BASE_BRANCH={}", base_branch),
            format!("TARGET_BRANCH={}", target_branch),
            format!("COMPOSITE_TASK_ID={}", composite_task_id.unwrap_or("standalone")),
            self.autodev_server_url.as_ref()
                .map(|url| format!("AUTODEV_SERVER_URL={}", url))
                .unwrap_or_else(|| "".to_string()),
        ];

        let env: Vec<&str> = env_strings.iter().map(|s| s.as_str()).collect();

        // Create container configuration
        let host_config = HostConfig {
            mounts: Some(vec![Mount {
                target: Some("/output".to_string()),
                source: Some(output_dir.clone()),
                typ: Some(MountTypeEnum::BIND),
                ..Default::default()
            }]),
            auto_remove: Some(true),
            ..Default::default()
        };

        let config = Config {
            image: Some(WORKER_IMAGE),
            env: Some(env),
            host_config: Some(host_config),
            ..Default::default()
        };

        // Create container
        let container_name = format!("autodev-task-{}", task.id);
        let create_options = CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        };

        let container = self
            .docker
            .create_container(Some(create_options), config)
            .await?;

        tracing::info!("Created container: {}", container.id);

        // Start container
        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await?;

        tracing::info!("Started container: {}", container.id);

        // Wait for container to finish
        let wait_options = WaitContainerOptions {
            condition: "not-running",
        };

        let mut wait_stream = self.docker.wait_container(&container.id, Some(wait_options));

        let exit_code = if let Some(wait_result) = wait_stream.next().await {
            wait_result?.status_code
        } else {
            return Err(anyhow!("Container wait stream ended unexpectedly"));
        };

        tracing::info!("Container exited with code: {}", exit_code);

        // Read result file
        let result_file = format!("{}/result.json", output_dir);
        let result_content = fs::read_to_string(&result_file).await.map_err(|e| {
            anyhow!("Failed to read result file: {}. Container may have failed.", e)
        })?;

        let result: TaskResult = serde_json::from_str(&result_content)?;

        // Cleanup output directory
        fs::remove_dir_all(&output_dir).await.ok();

        // Container is auto-removed due to auto_remove flag
        tracing::info!("Task execution completed: {:?}", result);

        Ok(result)
    }

    pub async fn check_worker_image_exists(&self) -> Result<bool> {
        let images = self.docker.list_images::<String>(None).await?;

        for image in images {
            let repo_tags = &image.repo_tags;
            if repo_tags.contains(&WORKER_IMAGE.to_string()) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn build_worker_image(&self, dockerfile_path: &str) -> Result<()> {
        tracing::info!("Building worker image from: {}", dockerfile_path);

        // This is a simplified version - in production, you'd want to use
        // bollard's build_image method with proper tar stream

        Err(anyhow!(
            "Worker image build not implemented. Please build manually with: \
            cd docker/worker && docker build -t {} .",
            WORKER_IMAGE
        ))
    }
}
