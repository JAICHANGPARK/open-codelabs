use anyhow::{anyhow, Result};
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

pub struct CodeServerManager {
    workspace_base: PathBuf,
}

impl CodeServerManager {
    async fn run_command(mut cmd: Command, context: &str) -> Result<()> {
        let output = cmd.output().await?;
        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if !stderr.trim().is_empty() {
            stderr.trim()
        } else if !stdout.trim().is_empty() {
            stdout.trim()
        } else {
            "unknown error"
        };

        Err(anyhow!("{}: {}", context, detail))
    }

    pub fn new() -> Result<Self> {
        let workspace_base = PathBuf::from("/app/workspaces");
        Ok(Self { workspace_base })
    }

    /// Create workspace directory for a codelab
    pub async fn create_workspace(&self, codelab_id: &str) -> Result<PathBuf> {
        let workspace_path = self.workspace_base.join(codelab_id);
        fs::create_dir_all(&workspace_path).await?;
        Ok(workspace_path)
    }

    /// Write a file to the workspace
    pub async fn write_file(
        &self,
        codelab_id: &str,
        file_path: &str,
        content: &str,
    ) -> Result<()> {
        let full_path = self.workspace_base.join(codelab_id).join(file_path);

        // Create parent directory
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write file content
        fs::write(&full_path, content).await?;

        Ok(())
    }

    /// Initialize git repository in workspace
    pub async fn init_git_repo(&self, codelab_id: &str) -> Result<()> {
        let workspace_path = self.workspace_base.join(codelab_id);

        Self::run_command(
            {
                let mut cmd = Command::new("git");
                cmd.arg("init").current_dir(&workspace_path);
                cmd
            },
            "Failed to initialize git repository",
        )
        .await?;

        Self::run_command(
            {
                let mut cmd = Command::new("git");
                cmd.arg("config")
                    .arg("user.name")
                    .arg("Codelab")
                    .current_dir(&workspace_path);
                cmd
            },
            "Failed to set git user.name",
        )
        .await?;

        Self::run_command(
            {
                let mut cmd = Command::new("git");
                cmd.arg("config")
                    .arg("user.email")
                    .arg("codelab@example.com")
                    .current_dir(&workspace_path);
                cmd
            },
            "Failed to set git user.email",
        )
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
        let workspace_path = self.workspace_base.join(codelab_id);
        let branch_name = format!("step-{}-{}", step_number, branch_type);

        // Commit current state
        Self::run_command(
            {
                let mut cmd = Command::new("git");
                cmd.arg("add").arg(".").current_dir(&workspace_path);
                cmd
            },
            "Failed to stage workspace files",
        )
        .await?;

        Self::run_command(
            {
                let mut cmd = Command::new("git");
                cmd.arg("commit")
                    .arg("-m")
                    .arg(format!("Step {} {}", step_number, branch_type))
                    .arg("--allow-empty")
                    .current_dir(&workspace_path);
                cmd
            },
            "Failed to create workspace commit",
        )
        .await?;

        // Create branch
        Self::run_command(
            {
                let mut cmd = Command::new("git");
                cmd.arg("branch")
                    .arg(&branch_name)
                    .current_dir(&workspace_path);
                cmd
            },
            "Failed to create workspace branch",
        )
        .await?;

        Ok(())
    }

    /// Create folder structure for a step (alternative to branches)
    pub async fn create_step_folder(
        &self,
        codelab_id: &str,
        step_number: i32,
        folder_type: &str, // "start" or "end"
        files: &[(String, String)], // (path, content)
    ) -> Result<()> {
        let workspace_path = self.workspace_base.join(codelab_id);
        let folder_name = format!("step-{}-{}", step_number, folder_type);
        let folder_path = workspace_path.join(&folder_name);

        // Create folder
        fs::create_dir_all(&folder_path).await?;

        // Write files to folder
        for (file_path, content) in files {
            let full_path = folder_path.join(file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&full_path, content).await?;
        }

        Ok(())
    }

    /// List folders in workspace (for folder-based structure)
    pub async fn list_folders(&self, codelab_id: &str) -> Result<Vec<String>> {
        let workspace_path = self.workspace_base.join(codelab_id);
        let mut folders = Vec::new();

        let mut entries = fs::read_dir(&workspace_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    folders.push(name.to_string());
                }
            }
        }

        folders.sort();
        Ok(folders)
    }

    /// List files in a folder (for folder-based structure)
    pub async fn list_folder_files(&self, codelab_id: &str, folder: &str) -> Result<Vec<String>> {
        let folder_path = self.workspace_base.join(codelab_id).join(folder);
        let mut files = Vec::new();

        fn collect_files(
            dir: &std::path::Path,
            base: &std::path::Path,
            files: &mut Vec<String>,
        ) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Ok(rel_path) = path.strip_prefix(base) {
                        if let Some(path_str) = rel_path.to_str() {
                            files.push(path_str.to_string());
                        }
                    }
                } else if path.is_dir() {
                    collect_files(&path, base, files)?;
                }
            }
            Ok(())
        }

        collect_files(&folder_path, &folder_path, &mut files)
            .map_err(|e| anyhow!("Failed to list files: {}", e))?;

        files.sort();
        Ok(files)
    }

    /// Read file from folder (for folder-based structure)
    pub async fn read_folder_file(
        &self,
        codelab_id: &str,
        folder: &str,
        file_path: &str,
    ) -> Result<String> {
        let full_path = self
            .workspace_base
            .join(codelab_id)
            .join(folder)
            .join(file_path);

        let content = fs::read_to_string(&full_path).await?;
        Ok(content)
    }

    /// Create tar archive of workspace
    pub async fn archive_workspace(&self, codelab_id: &str) -> Result<Vec<u8>> {
        let workspace_path = self.workspace_base.join(codelab_id);

        // Check if workspace directory exists
        if !workspace_path.exists() {
            return Err(anyhow!(
                "Workspace directory does not exist. The workspace may not have been created yet."
            ));
        }

        let output = Command::new("tar")
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
        let workspace_path = self.workspace_base.join(codelab_id);
        fs::remove_dir_all(&workspace_path).await?;
        Ok(())
    }

    /// List all branches in the workspace
    pub async fn list_branches(&self, codelab_id: &str) -> Result<Vec<String>> {
        let workspace_path = self.workspace_base.join(codelab_id);

        let output = Command::new("git")
            .arg("branch")
            .arg("--list")
            .current_dir(&workspace_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to list branches: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.trim().trim_start_matches("* ").to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(branches)
    }

    /// List files in a specific branch
    pub async fn list_files(&self, codelab_id: &str, branch: &str) -> Result<Vec<String>> {
        let workspace_path = self.workspace_base.join(codelab_id);

        let output = Command::new("git")
            .arg("ls-tree")
            .arg("-r")
            .arg("--name-only")
            .arg(branch)
            .current_dir(&workspace_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to list files: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(files)
    }

    /// Read file content from a specific branch
    pub async fn read_file(&self, codelab_id: &str, branch: &str, file_path: &str) -> Result<String> {
        let workspace_path = self.workspace_base.join(codelab_id);

        let output = Command::new("git")
            .arg("show")
            .arg(format!("{}:{}", branch, file_path))
            .current_dir(&workspace_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to read file: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
