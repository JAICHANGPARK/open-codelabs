use anyhow::{anyhow, Result};
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::process::Command;

pub struct CodeServerManager {
    workspace_base: PathBuf,
}

impl CodeServerManager {
    fn is_writable_directory(path: &Path) -> bool {
        if std::fs::create_dir_all(path).is_err() {
            return false;
        }

        let probe = path.join(format!(
            ".oc-write-probe-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let write_ok = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&probe)
            .and_then(|mut file| file.write_all(b"probe"))
            .is_ok();
        let _ = std::fs::remove_file(&probe);
        write_ok
    }

    fn resolve_default_workspace_base() -> PathBuf {
        if let Ok(path) = std::env::var("WORKSPACE_BASE") {
            return PathBuf::from(path);
        }

        // Keep container compatibility when /app/workspaces exists,
        // but fall back to a writable local project path.
        let container_default = std::env::var("WORKSPACE_CONTAINER_BASE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/app/workspaces"));
        if Self::is_writable_directory(&container_default) {
            return container_default;
        }

        let local_default = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("data/workspaces");
        if Self::is_writable_directory(&local_default) {
            return local_default;
        }

        std::env::temp_dir().join("open-codelabs-workspaces")
    }

    pub fn default_workspace_base() -> PathBuf {
        Self::resolve_default_workspace_base()
    }

    pub fn workspace_path(&self, codelab_id: &str) -> PathBuf {
        self.workspace_base.join(codelab_id)
    }

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

    async fn run_command_output(mut cmd: Command, context: &str) -> Result<String> {
        let output = cmd.output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if !stderr.trim().is_empty() {
                stderr.trim()
            } else if !stdout.trim().is_empty() {
                stdout.trim()
            } else {
                "unknown error"
            };
            return Err(anyhow!("{}: {}", context, detail));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn new(workspace_base: PathBuf) -> Self {
        Self { workspace_base }
    }

    pub fn from_env() -> Result<Self> {
        let workspace_base = Self::resolve_default_workspace_base();
        Ok(Self { workspace_base })
    }

    /// Create workspace directory for a codelab
    pub async fn create_workspace(&self, codelab_id: &str) -> Result<PathBuf> {
        let workspace_path = self.workspace_path(codelab_id);
        fs::create_dir_all(&workspace_path).await?;
        Ok(workspace_path)
    }

    /// Write a file to the workspace
    pub async fn write_file(&self, codelab_id: &str, file_path: &str, content: &str) -> Result<()> {
        let full_path = self.workspace_base.join(codelab_id).join(file_path);

        // Create parent directory
        let parent = full_path
            .parent()
            .ok_or_else(|| anyhow!("invalid workspace file path: {}", file_path))?;
        fs::create_dir_all(parent).await?;

        // Write file content
        fs::write(&full_path, content).await?;

        Ok(())
    }

    /// Remove a file or directory from the workspace
    pub async fn remove_path(&self, codelab_id: &str, file_path: &str) -> Result<()> {
        let full_path = self.workspace_base.join(codelab_id).join(file_path);
        let metadata = match fs::metadata(&full_path).await {
            Ok(metadata) => metadata,
            Err(_) => return Ok(()),
        };

        if metadata.is_dir() {
            fs::remove_dir_all(&full_path).await?;
        } else {
            fs::remove_file(&full_path).await?;
        }

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

    pub async fn current_branch(&self, codelab_id: &str) -> Result<String> {
        let workspace_path = self.workspace_base.join(codelab_id);
        Self::run_command_output(
            {
                let mut cmd = Command::new("git");
                cmd.arg("rev-parse")
                    .arg("--abbrev-ref")
                    .arg("HEAD")
                    .current_dir(&workspace_path);
                cmd
            },
            "Failed to get current git branch",
        )
        .await
    }

    pub async fn checkout_branch(&self, codelab_id: &str, branch: &str) -> Result<()> {
        let workspace_path = self.workspace_base.join(codelab_id);
        Self::run_command(
            {
                let mut cmd = Command::new("git");
                cmd.arg("checkout").arg(branch).current_dir(&workspace_path);
                cmd
            },
            "Failed to checkout git branch",
        )
        .await
    }

    pub async fn commit_changes(&self, codelab_id: &str, message: &str) -> Result<()> {
        let workspace_path = self.workspace_base.join(codelab_id);
        Self::run_command(
            {
                let mut cmd = Command::new("git");
                cmd.arg("add").arg(".").current_dir(&workspace_path);
                cmd
            },
            "Failed to stage workspace changes",
        )
        .await?;

        Self::run_command(
            {
                let mut cmd = Command::new("git");
                cmd.arg("commit")
                    .arg("-m")
                    .arg(message)
                    .arg("--allow-empty")
                    .current_dir(&workspace_path);
                cmd
            },
            "Failed to commit workspace changes",
        )
        .await
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
        folder_type: &str,          // "start" or "end"
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
            let parent = full_path
                .parent()
                .ok_or_else(|| anyhow!("invalid step file path: {}", file_path))?;
            fs::create_dir_all(parent).await?;
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
                        files.push(rel_path.to_string_lossy().to_string());
                    }
                }
                if path.is_dir() {
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
    pub async fn read_file(
        &self,
        codelab_id: &str,
        branch: &str,
        file_path: &str,
    ) -> Result<String> {
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{LazyLock, Mutex};
    use tempfile::tempdir;

    static FS_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct EnvVarRestore {
        key: &'static str,
        value: Option<String>,
    }

    impl EnvVarRestore {
        fn new(key: &'static str) -> Self {
            Self {
                key,
                value: std::env::var(key).ok(),
            }
        }
    }

    impl Drop for EnvVarRestore {
        fn drop(&mut self) {
            if let Some(value) = &self.value {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    struct CurrentDirRestore {
        previous: PathBuf,
    }

    impl CurrentDirRestore {
        fn new() -> Self {
            Self {
                previous: std::env::current_dir().expect("current dir"),
            }
        }
    }

    impl Drop for CurrentDirRestore {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.previous);
        }
    }

    #[tokio::test]
    async fn test_workspace_operations() -> Result<()> {
        let dir = tempdir()?;
        let manager = CodeServerManager::new(dir.path().to_path_buf());
        let codelab_id = "test-codelab";

        // Test create_workspace
        let ws_path = manager.create_workspace(codelab_id).await?;
        assert!(ws_path.exists());

        // Test write_file
        manager.write_file(codelab_id, "hello.txt", "world").await?;
        let content = fs::read_to_string(ws_path.join("hello.txt")).await?;
        assert_eq!(content, "world");
        manager
            .write_file(codelab_id, "nested/deep.txt", "deep")
            .await?;
        assert_eq!(
            fs::read_to_string(ws_path.join("nested/deep.txt")).await?,
            "deep"
        );

        // Test list_folders
        manager
            .create_step_folder(codelab_id, 1, "start", &[])
            .await?;
        let folders = manager.list_folders(codelab_id).await?;
        assert!(folders.contains(&"step-1-start".to_string()));

        // Test remove_path
        manager.remove_path(codelab_id, "hello.txt").await?;
        assert!(!ws_path.join("hello.txt").exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_git_operations() -> Result<()> {
        let dir = tempdir()?;
        let manager = CodeServerManager::new(dir.path().to_path_buf());
        let codelab_id = "git-test";
        manager.create_workspace(codelab_id).await?;

        // Test init_git_repo
        manager.init_git_repo(codelab_id).await?;
        let ws_path = dir.path().join(codelab_id);
        assert!(ws_path.join(".git").exists());

        // Test commit_changes
        manager
            .write_file(codelab_id, "README.md", "# Test")
            .await?;
        manager.commit_changes(codelab_id, "initial commit").await?;

        // Test current_branch (usually 'master' or 'main' depending on git version, but we just want to know it doesn't fail)
        let branch = manager.current_branch(codelab_id).await?;
        assert!(!branch.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_folder_file_operations() -> Result<()> {
        let dir = tempdir()?;
        let manager = CodeServerManager::new(dir.path().to_path_buf());
        let codelab_id = "folder-test";
        manager.create_workspace(codelab_id).await?;

        manager
            .create_step_folder(
                codelab_id,
                2,
                "end",
                &[
                    ("main.rs".to_string(), "fn main() {}".to_string()),
                    ("nested/lib.rs".to_string(), "pub fn run() {}".to_string()),
                    (
                        "nested/deep/lib2.rs".to_string(),
                        "pub fn run2() {}".to_string(),
                    ),
                ],
            )
            .await?;

        let files = manager.list_folder_files(codelab_id, "step-2-end").await?;
        assert_eq!(
            files,
            vec![
                "main.rs".to_string(),
                "nested/deep/lib2.rs".to_string(),
                "nested/lib.rs".to_string()
            ]
        );

        let content = manager
            .read_folder_file(codelab_id, "step-2-end", "nested/lib.rs")
            .await?;
        assert_eq!(content, "pub fn run() {}");

        Ok(())
    }

    #[test]
    fn test_is_writable_directory_and_resolve_base_paths() -> Result<()> {
        let _lock = FS_ENV_LOCK.lock().expect("fs env lock");
        let _env_restore = EnvVarRestore::new("WORKSPACE_BASE");
        let _container_restore = EnvVarRestore::new("WORKSPACE_CONTAINER_BASE");
        std::env::remove_var("WORKSPACE_BASE");
        std::env::remove_var("WORKSPACE_CONTAINER_BASE");

        let dir = tempdir()?;
        assert!(CodeServerManager::is_writable_directory(dir.path()));

        std::env::set_var("WORKSPACE_BASE", dir.path().to_string_lossy().to_string());
        assert_eq!(
            CodeServerManager::resolve_default_workspace_base(),
            PathBuf::from(dir.path())
        );

        std::env::remove_var("WORKSPACE_BASE");
        let resolved = CodeServerManager::resolve_default_workspace_base();
        assert!(!resolved.as_os_str().is_empty());

        Ok(())
    }

    #[test]
    fn test_resolve_default_workspace_base_prefers_container_override_when_writable() -> Result<()>
    {
        let _lock = FS_ENV_LOCK.lock().expect("fs env lock");
        let _env_restore = EnvVarRestore::new("WORKSPACE_BASE");
        let _container_restore = EnvVarRestore::new("WORKSPACE_CONTAINER_BASE");
        std::env::remove_var("WORKSPACE_BASE");

        let dir = tempdir()?;
        std::env::set_var(
            "WORKSPACE_CONTAINER_BASE",
            dir.path().to_string_lossy().to_string(),
        );

        assert_eq!(
            CodeServerManager::resolve_default_workspace_base(),
            dir.path().to_path_buf()
        );
        Ok(())
    }

    #[test]
    fn test_resolve_default_workspace_base_uses_local_default_when_container_unwritable(
    ) -> Result<()> {
        let _lock = FS_ENV_LOCK.lock().expect("fs env lock");
        let _env_restore = EnvVarRestore::new("WORKSPACE_BASE");
        let _container_restore = EnvVarRestore::new("WORKSPACE_CONTAINER_BASE");
        let _cwd_restore = CurrentDirRestore::new();
        std::env::remove_var("WORKSPACE_BASE");
        std::env::set_var("WORKSPACE_CONTAINER_BASE", "/root/oc-unwritable-workspaces");

        let dir = tempdir()?;
        std::env::set_current_dir(dir.path())?;

        let resolved = CodeServerManager::resolve_default_workspace_base();
        let resolved_canonical = std::fs::canonicalize(&resolved)?;
        let expected_canonical = std::fs::canonicalize(dir.path().join("data/workspaces"))?;
        assert_eq!(resolved_canonical, expected_canonical);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_resolve_default_workspace_base_falls_back_to_temp_when_local_unwritable() -> Result<()>
    {
        use std::os::unix::fs::PermissionsExt;

        let _lock = FS_ENV_LOCK.lock().expect("fs env lock");
        let _env_restore = EnvVarRestore::new("WORKSPACE_BASE");
        let _container_restore = EnvVarRestore::new("WORKSPACE_CONTAINER_BASE");
        let _cwd_restore = CurrentDirRestore::new();
        std::env::remove_var("WORKSPACE_BASE");
        std::env::set_var("WORKSPACE_CONTAINER_BASE", "/root/oc-unwritable-workspaces");

        let dir = tempdir()?;
        let no_write = dir.path().join("no-write");
        std::fs::create_dir_all(&no_write)?;
        let mut perms = std::fs::metadata(&no_write)?.permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(&no_write, perms)?;
        std::env::set_current_dir(&no_write)?;

        let resolved = CodeServerManager::resolve_default_workspace_base();
        assert_eq!(
            resolved,
            std::env::temp_dir().join("open-codelabs-workspaces")
        );
        Ok(())
    }

    #[test]
    fn test_env_var_restore_restores_existing_value() {
        let _lock = FS_ENV_LOCK.lock().expect("fs env lock");
        let _env_restore = EnvVarRestore::new("WORKSPACE_BASE");
        std::env::set_var("WORKSPACE_BASE", "before");
        {
            let _restore = EnvVarRestore::new("WORKSPACE_BASE");
            std::env::set_var("WORKSPACE_BASE", "after");
        }
        assert_eq!(std::env::var("WORKSPACE_BASE").as_deref(), Ok("before"));
        std::env::remove_var("WORKSPACE_BASE");
    }

    #[tokio::test]
    async fn test_branch_and_file_listing() -> Result<()> {
        let dir = tempdir()?;
        let manager = CodeServerManager::new(dir.path().to_path_buf());
        let codelab_id = "branch-test";
        manager.create_workspace(codelab_id).await?;
        manager.init_git_repo(codelab_id).await?;
        manager
            .write_file(codelab_id, "README.md", "# hello")
            .await?;
        manager.commit_changes(codelab_id, "init").await?;
        manager.create_step_branch(codelab_id, 1, "start").await?;

        let current = manager.current_branch(codelab_id).await?;
        let branches = manager.list_branches(codelab_id).await?;
        assert!(branches.contains(&current));
        assert!(branches.contains(&"step-1-start".to_string()));

        let files = manager.list_files(codelab_id, &current).await?;
        assert!(files.contains(&"README.md".to_string()));

        let readme = manager.read_file(codelab_id, &current, "README.md").await?;
        assert_eq!(readme.trim(), "# hello");

        manager.checkout_branch(codelab_id, "step-1-start").await?;
        assert_eq!(manager.current_branch(codelab_id).await?, "step-1-start");

        Ok(())
    }

    #[tokio::test]
    async fn test_archive_and_remove_workspace() -> Result<()> {
        let dir = tempdir()?;
        let manager = CodeServerManager::new(dir.path().to_path_buf());
        let codelab_id = "archive-test";
        manager.create_workspace(codelab_id).await?;
        manager.write_file(codelab_id, "a.txt", "b").await?;

        let archive = manager.archive_workspace(codelab_id).await?;
        assert!(!archive.is_empty());

        manager.remove_workspace(codelab_id).await?;
        assert!(!dir.path().join(codelab_id).exists());
        assert!(manager.archive_workspace(codelab_id).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_archive_workspace_returns_tar_failure() -> Result<()> {
        let dir = tempdir()?;
        let manager = CodeServerManager::new(dir.path().to_path_buf());
        let codelab_id = "not-a-directory";
        fs::write(dir.path().join(codelab_id), "x").await?;

        let err = manager
            .archive_workspace(codelab_id)
            .await
            .expect_err("must fail");
        assert!(err.to_string().contains("Failed to create archive"));
        Ok(())
    }

    #[tokio::test]
    async fn test_non_git_operations_return_errors() -> Result<()> {
        let dir = tempdir()?;
        let manager = CodeServerManager::new(dir.path().to_path_buf());
        let codelab_id = "nogit-test";
        manager.create_workspace(codelab_id).await?;

        assert!(manager.current_branch(codelab_id).await.is_err());
        assert!(manager.checkout_branch(codelab_id, "nope").await.is_err());
        assert!(manager.commit_changes(codelab_id, "msg").await.is_err());
        assert!(manager.list_branches(codelab_id).await.is_err());
        assert!(manager.list_files(codelab_id, "main").await.is_err());
        assert!(manager
            .read_file(codelab_id, "main", "README.md")
            .await
            .is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_path_for_directory_and_missing_path() -> Result<()> {
        let dir = tempdir()?;
        let manager = CodeServerManager::new(dir.path().to_path_buf());
        let codelab_id = "remove-test";
        let ws = manager.create_workspace(codelab_id).await?;
        manager
            .write_file(codelab_id, "nested/hello.txt", "world")
            .await?;
        assert!(ws.join("nested").exists());

        manager.remove_path(codelab_id, "nested").await?;
        assert!(!ws.join("nested").exists());

        // Should be no-op when the path does not exist.
        manager.remove_path(codelab_id, "does-not-exist").await?;

        Ok(())
    }

    #[test]
    fn test_from_env_uses_workspace_base() -> Result<()> {
        let _lock = FS_ENV_LOCK.lock().expect("fs env lock");
        let _env_restore = EnvVarRestore::new("WORKSPACE_BASE");
        std::env::set_var("WORKSPACE_BASE", "/tmp/open-codelabs-workspace");
        let manager = CodeServerManager::from_env()?;
        assert_eq!(
            CodeServerManager::default_workspace_base(),
            PathBuf::from("/tmp/open-codelabs-workspace")
        );
        assert_eq!(
            manager.workspace_path("codelab"),
            PathBuf::from("/tmp/open-codelabs-workspace/codelab")
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_internal_command_error_detail_selection() -> Result<()> {
        let stdout_err = CodeServerManager::run_command(
            {
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg("echo command-output && exit 1");
                cmd
            },
            "ctx",
        )
        .await
        .expect_err("stdout error expected");
        assert!(stdout_err.to_string().contains("command-output"));

        let unknown_err = CodeServerManager::run_command(
            {
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg("exit 1");
                cmd
            },
            "ctx",
        )
        .await
        .expect_err("unknown error expected");
        assert!(unknown_err.to_string().contains("unknown error"));

        let stdout_output_err = CodeServerManager::run_command_output(
            {
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg("echo output-only && exit 1");
                cmd
            },
            "ctx",
        )
        .await
        .expect_err("stdout error expected");
        assert!(stdout_output_err.to_string().contains("output-only"));

        let unknown_output_err = CodeServerManager::run_command_output(
            {
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg("exit 1");
                cmd
            },
            "ctx",
        )
        .await
        .expect_err("unknown error expected");
        assert!(unknown_output_err.to_string().contains("unknown error"));

        Ok(())
    }
}
