use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateCodeServerRequest {
    pub codelab_id: String,
    pub workspace_files: Option<Vec<WorkspaceFile>>,
    pub structure_type: Option<String>, // "branch" or "folder"
}

#[derive(Deserialize, Serialize)]
pub struct WorkspaceFile {
    pub path: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct CodeServerInfo {
    pub path: String,
    pub structure_type: String,
}

#[derive(Deserialize)]
pub struct CreateBranchRequest {
    pub step_number: i32,
    pub branch_type: String, // "start" or "end"
}

#[derive(Deserialize)]
pub struct CreateFolderRequest {
    pub step_number: i32,
    pub folder_type: String, // "start" or "end"
    pub files: Vec<WorkspaceFile>,
}

#[derive(Deserialize)]
pub struct UpdateWorkspaceFilesRequest {
    pub files: Vec<WorkspaceFile>,
    pub delete_files: Option<Vec<String>>,
    pub commit_message: Option<String>,
}

#[derive(Deserialize)]
pub struct ReadFileQuery {
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
