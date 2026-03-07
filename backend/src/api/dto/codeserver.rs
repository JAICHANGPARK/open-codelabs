use serde::{Deserialize, Serialize};

/// Payload used to provision a workspace for a codelab.
#[derive(Deserialize, Serialize)]
pub struct CreateCodeServerRequest {
    /// Target codelab identifier.
    pub codelab_id: String,
    /// Optional seed files written into the workspace on creation.
    pub workspace_files: Option<Vec<WorkspaceFile>>,
    /// Workspace structure mode, typically `branch` or `folder`.
    pub structure_type: Option<String>,
}

/// Path/content pair used when writing files into a workspace.
#[derive(Deserialize, Serialize)]
pub struct WorkspaceFile {
    /// Relative file path inside the workspace.
    pub path: String,
    /// Full file contents to write.
    pub content: String,
}

/// Response returned for an existing provisioned workspace.
#[derive(Serialize, Deserialize)]
pub struct CodeServerInfo {
    /// Filesystem path of the workspace root.
    pub path: String,
    /// Workspace structure mode, typically `branch` or `folder`.
    pub structure_type: String,
}

/// Payload used to create a git branch snapshot for a step.
#[derive(Deserialize, Serialize)]
pub struct CreateBranchRequest {
    /// Step number used to derive the branch name.
    pub step_number: i32,
    /// Branch variant, typically `start` or `end`.
    pub branch_type: String,
}

/// Payload used to create a folder snapshot for a step.
#[derive(Deserialize, Serialize)]
pub struct CreateFolderRequest {
    /// Step number used to derive the folder name.
    pub step_number: i32,
    /// Folder variant, typically `start` or `end`.
    pub folder_type: String,
    /// Files that should exist inside the created folder snapshot.
    pub files: Vec<WorkspaceFile>,
}

/// Payload used to update a branch or folder workspace in place.
#[derive(Deserialize, Serialize)]
pub struct UpdateWorkspaceFilesRequest {
    /// Files to create or overwrite.
    pub files: Vec<WorkspaceFile>,
    /// Relative file paths to delete.
    pub delete_files: Option<Vec<String>>,
    /// Optional git commit message when the workspace is branch-based.
    pub commit_message: Option<String>,
}

/// Query parameter used when reading a single file from a workspace.
#[derive(Deserialize, Serialize)]
pub struct ReadFileQuery {
    /// Relative file path inside the workspace.
    pub file: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codeserver_info_serializes() {
        let info = CodeServerInfo {
            path: "/app/workspaces/123".to_string(),
            structure_type: "branch".to_string(),
        };
        let json = serde_json::to_value(&info).expect("serialize");
        assert_eq!(json["path"], "/app/workspaces/123");
        assert_eq!(json["structure_type"], "branch");
    }

    #[test]
    fn test_create_codeserver_request_deserializes() {
        let raw = r#"{
            "codelab_id": "c1",
            "workspace_files": [{"path":"main.rs","content":"fn main(){}"}],
            "structure_type": "folder"
        }"#;
        let req: CreateCodeServerRequest = serde_json::from_str(raw).expect("deserialize");
        assert_eq!(req.codelab_id, "c1");
        assert_eq!(req.structure_type.as_deref(), Some("folder"));
        let files = req.workspace_files.expect("files");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "main.rs");
    }
}
