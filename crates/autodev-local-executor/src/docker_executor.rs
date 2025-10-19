use anyhow::{anyhow, Result};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, LogsOptions, StartContainerOptions, WaitContainerOptions};
use bollard::models::{HostConfig, Mount, MountTypeEnum};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use std::path::PathBuf;

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
    workspace_dir: PathBuf,
}

impl DockerExecutor {
    pub async fn new(
        anthropic_api_key: String,
        github_token: String,
        autodev_server_url: Option<String>,
        workspace_dir: PathBuf,
    ) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;

        // Verify docker connection
        docker.ping().await?;

        // Create workspace directory if it doesn't exist
        fs::create_dir_all(&workspace_dir).await?;

        Ok(Self {
            docker,
            anthropic_api_key,
            github_token,
            autodev_server_url,
            workspace_dir,
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

        // Create output directory on HOST filesystem
        let output_dir = self.workspace_dir.join(format!("output-{}", task.id));
        fs::create_dir_all(&output_dir).await?;

        tracing::debug!("Created output directory: {:?}", output_dir);

        // Build environment variables
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

        // Create container configuration with HOST path bind mount
        let output_dir_str = output_dir
            .to_str()
            .ok_or_else(|| anyhow!("Invalid output directory path"))?
            .to_string();

        let host_config = HostConfig {
            mounts: Some(vec![Mount {
                target: Some("/output".to_string()),
                source: Some(output_dir_str.clone()),
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

        tracing::debug!("Creating container with bind mount: {} -> /output", output_dir_str);

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

        // Create log file path
        let log_file_path = self.workspace_dir.join(format!("logs-{}.txt", task.id));
        // Create log file to ensure it exists
        let _ = fs::File::create(&log_file_path).await?;

        tracing::info!("Collecting container logs to: {:?}", log_file_path);

        // Collect container logs in the background
        let docker_clone = self.docker.clone();
        let container_id_clone = container.id.clone();
        let log_file_path_clone = log_file_path.clone();

        tokio::spawn(async move {
            let log_options = LogsOptions::<String> {
                follow: true,
                stdout: true,
                stderr: true,
                timestamps: true,
                ..Default::default()
            };

            let mut log_stream = docker_clone.logs(&container_id_clone, Some(log_options));

            if let Ok(mut file) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path_clone)
                .await
            {
                while let Some(log_result) = log_stream.next().await {
                    if let Ok(log_output) = log_result {
                        let log_str = log_output.to_string();
                        let _ = file.write_all(log_str.as_bytes()).await;
                        let _ = file.flush().await;
                    }
                }
            }
        });

        // Wait for container to finish
        let wait_options = WaitContainerOptions {
            condition: "not-running",
        };

        let mut wait_stream = self.docker.wait_container(&container.id, Some(wait_options));

        let exit_code = if let Some(wait_result) = wait_stream.next().await {
            wait_result?.status_code
        } else {
            // Read last 50 lines of log for error context
            let log_tail = Self::read_log_tail(&log_file_path, 50).await;
            return Err(anyhow!(
                "Container wait stream ended unexpectedly.\nLog file: {:?}\n\nLast 50 lines:\n{}",
                log_file_path,
                log_tail
            ));
        };

        tracing::info!("Container exited with code: {}", exit_code);

        // If container failed, include log tail in error
        if exit_code != 0 {
            let log_tail = Self::read_log_tail(&log_file_path, 50).await;
            return Err(anyhow!(
                "Container exited with code {}.\nLog file: {:?}\n\nLast 50 lines:\n{}",
                exit_code,
                log_file_path,
                log_tail
            ));
        }

        // Read result file
        let result_file = output_dir.join("result.json");
        let log_file_path_for_error = log_file_path.clone();
        let result_content = fs::read_to_string(&result_file).await.map_err(|e| {
            anyhow!(
                "Failed to read result file: {}. Container may have failed.\nCheck log file at: {:?}",
                e,
                log_file_path_for_error
            )
        })?;

        let result: TaskResult = serde_json::from_str(&result_content)?;

        // Cleanup output directory
        fs::remove_dir_all(&output_dir).await.ok();

        // Container is auto-removed due to auto_remove flag
        tracing::info!("Task execution completed: {:?}", result);
        tracing::info!("Container logs saved to: {:?}", log_file_path);

        // Keep log file for debugging (don't delete it)
        if !result.success {
            tracing::error!("Task failed. Check logs at: {:?}", log_file_path);
        }

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

    /// Read last N lines from log file
    async fn read_log_tail(log_file_path: &PathBuf, lines: usize) -> String {
        match fs::read_to_string(log_file_path).await {
            Ok(content) => {
                let all_lines: Vec<&str> = content.lines().collect();
                let start = all_lines.len().saturating_sub(lines);
                all_lines[start..].join("\n")
            }
            Err(e) => format!("Failed to read log file: {}", e),
        }
    }
}
