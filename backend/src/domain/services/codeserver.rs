use anyhow::{anyhow, Result};
use std::path::PathBuf;
use tokio::process::Command;

pub struct CodeServerManager {
    container_name: String,
}

impl CodeServerManager {
    pub fn new() -> Result<Self> {
        let container_name = std::env::var("CODESERVER_CONTAINER_NAME")
            .unwrap_or_else(|_| "open-codelabs-codeserver-1".to_string());
        Ok(Self { container_name })
    }

    /// Create workspace directory for a codelab
    pub async fn create_workspace(&self, codelab_id: &str) -> Result<PathBuf> {
        // Create directory in the codeserver container
        self.exec_command(&[
            "mkdir",
            "-p",
            &format!("/home/coder/workspace/{}", codelab_id),
        ])
        .await?;

        Ok(PathBuf::from(format!("/home/coder/workspace/{}", codelab_id)))
    }

    /// Write a file to the workspace
    pub async fn write_file(
        &self,
        codelab_id: &str,
        file_path: &str,
        content: &str,
    ) -> Result<()> {
        let dir_path = format!("/home/coder/workspace/{}/{}", codelab_id, file_path);

        // Create parent directory
        if let Some(parent) = std::path::Path::new(&dir_path).parent() {
            self.exec_command(&[
                "mkdir",
                "-p",
                parent.to_str().ok_or_else(|| anyhow!("Invalid path"))?,
            ])
            .await?;
        }

        // Write file content
        let escaped_content = content.replace("'", "'\\''");
        self.exec_command(&[
            "sh",
            "-c",
            &format!("printf '%s' '{}' > {}", escaped_content, dir_path),
        ])
        .await?;

        Ok(())
    }

    /// Execute a command inside the container
    pub async fn exec_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("docker")
            .arg("exec")
            .arg(&self.container_name)
            .args(args)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Initialize git repository in workspace
    pub async fn init_git_repo(&self, codelab_id: &str) -> Result<()> {
        let workspace_path = format!("/home/coder/workspace/{}", codelab_id);

        self.exec_command(&["sh", "-c", &format!("cd {} && git init", workspace_path)])
            .await?;
        self.exec_command(&[
            "sh",
            "-c",
            &format!("cd {} && git config user.name 'Codelab'", workspace_path),
        ])
        .await?;
        self.exec_command(&[
            "sh",
            "-c",
            &format!(
                "cd {} && git config user.email 'codelab@example.com'",
                workspace_path
            ),
        ])
        .await?;

        Ok(())
    }

    /// Create a git branch for a step
    pub async fn create_step_branch(
        &self,
        codelab_id: &str,
        step_number: i32,
        branch_type: &str, // "start" or "end"
    ) -> Result<()> {
        let workspace_path = format!("/home/coder/workspace/{}", codelab_id);
        let branch_name = format!("step-{}-{}", step_number, branch_type);

        // Commit current state
        self.exec_command(&[
            "sh",
            "-c",
            &format!("cd {} && git add .", workspace_path),
        ])
        .await?;

        self.exec_command(&[
            "sh",
            "-c",
            &format!(
                "cd {} && git commit -m 'Step {} {}' --allow-empty",
                workspace_path, step_number, branch_type
            ),
        ])
        .await?;

        // Create branch
        self.exec_command(&[
            "sh",
            "-c",
            &format!("cd {} && git branch {}", workspace_path, branch_name),
        ])
        .await?;

        Ok(())
    }

    /// Get the code server URL
    pub fn get_url(&self) -> Result<String> {
        let host = std::env::var("CODESERVER_HOST").unwrap_or_else(|_| "codeserver".to_string());
        let port = std::env::var("CODESERVER_PORT").unwrap_or_else(|_| "8443".to_string());
        Ok(format!("http://{}:{}", host, port))
    }

    /// Create tar archive of workspace
    pub async fn archive_workspace(&self, codelab_id: &str) -> Result<Vec<u8>> {
        let workspace_path = format!("/home/coder/workspace/{}", codelab_id);

        // Create tar archive using docker exec
        let output = Command::new("docker")
            .arg("exec")
            .arg(&self.container_name)
            .arg("tar")
            .arg("-czf")
            .arg("-")
            .arg("-C")
            .arg(&workspace_path)
            .arg(".")
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to create archive: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(output.stdout)
    }

    /// Remove workspace directory
    pub async fn remove_workspace(&self, codelab_id: &str) -> Result<()> {
        let workspace_path = format!("/home/coder/workspace/{}", codelab_id);
        self.exec_command(&["rm", "-rf", &workspace_path]).await?;
        Ok(())
    }
}
