use crate::error::Result;
use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
    WaitContainerOptions, LogsOptions, LogOutput,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, debug, error};

#[derive(Clone)]
pub struct DockerManager {
    client: Docker,
}

impl DockerManager {
    pub fn new() -> Result<Self> {
        let client = Docker::connect_with_local_defaults()?;
        Ok(Self { client })
    }

    /// Ensure the Claude executor image exists
    pub async fn ensure_image(&self, image_name: &str) -> Result<()> {
        debug!("Checking if image exists: {}", image_name);

        // Check if image exists
        match self.client.inspect_image(image_name).await {
            Ok(_) => {
                debug!("Image {} already exists", image_name);
                return Ok(());
            }
            Err(_) => {
                info!("Image {} not found, pulling...", image_name);
            }
        }

        // Pull the image
        let options = Some(CreateImageOptions {
            from_image: image_name,
            ..Default::default()
        });

        let mut stream = self.client.create_image(options, None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(status) = info.status {
                        debug!("Pull status: {}", status);
                    }
                }
                Err(e) => {
                    error!("Error pulling image: {}", e);
                    return Err(e.into());
                }
            }
        }

        info!("Image {} pulled successfully", image_name);

        Ok(())
    }

    /// Run a command in a Docker container
    pub async fn run_command(
        &self,
        image: &str,
        command: Vec<String>,
        workspace_path: &Path,
        env_vars: HashMap<String, String>,
    ) -> Result<(String, String, i64)> {
        info!("Running command in Docker: {:?}", command);

        // Ensure image exists
        self.ensure_image(image).await?;

        // Convert workspace path to absolute path
        let workspace_abs = workspace_path.canonicalize()?;
        let workspace_mount = format!("{}:/workspace", workspace_abs.display());

        // Prepare environment variables
        let env: Vec<String> = env_vars
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Create container configuration
        let config = Config {
            image: Some(image.to_string()),
            cmd: Some(command.clone()),
            working_dir: Some("/workspace".to_string()),
            env: Some(env),
            host_config: Some(bollard::models::HostConfig {
                binds: Some(vec![workspace_mount]),
                ..Default::default()
            }),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        // Create container
        let container_name = format!("autodev-executor-{}", uuid::Uuid::new_v4());
        let options = CreateContainerOptions {
            name: container_name.clone(),
            ..Default::default()
        };

        let container = self.client.create_container(Some(options), config).await?;
        let container_id = container.id;

        debug!("Container created: {}", container_id);

        // Start container
        self.client
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await?;

        debug!("Container started: {}", container_id);

        // Wait for container to finish
        let wait_options = Some(WaitContainerOptions {
            condition: "not-running",
        });

        let mut wait_stream = self.client.wait_container(&container_id, wait_options);

        let exit_code = if let Some(result) = wait_stream.next().await {
            match result {
                Ok(response) => response.status_code,
                Err(e) => {
                    error!("Error waiting for container: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            -1
        };

        debug!("Container exited with code: {}", exit_code);

        // Collect logs
        let log_options = Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            follow: false,
            ..Default::default()
        });

        let mut log_stream = self.client.logs(&container_id, log_options);

        let mut stdout = String::new();
        let mut stderr = String::new();

        while let Some(result) = log_stream.next().await {
            match result {
                Ok(log) => match log {
                    LogOutput::StdOut { message } => {
                        stdout.push_str(&String::from_utf8_lossy(&message));
                    }
                    LogOutput::StdErr { message } => {
                        stderr.push_str(&String::from_utf8_lossy(&message));
                    }
                    _ => {}
                },
                Err(e) => {
                    error!("Error reading logs: {}", e);
                }
            }
        }

        // Remove container
        let remove_options = Some(RemoveContainerOptions {
            force: true,
            ..Default::default()
        });

        self.client
            .remove_container(&container_id, remove_options)
            .await?;

        debug!("Container removed: {}", container_id);

        info!("Command completed with exit code: {}", exit_code);

        Ok((stdout, stderr, exit_code))
    }
}

impl Default for DockerManager {
    fn default() -> Self {
        Self::new().expect("Failed to create Docker client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_manager_creation() {
        let result = DockerManager::new();
        assert!(result.is_ok());
    }
}
