use crate::cli::client::ApiClient;
use crate::domain::models::{
    CreateCodelab, CreateMaterial, CreateQuiz, CreateStep, UpdateStepsPayload,
};
use anyhow::{Context, Result};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        Annotated, ListResourcesResult, PaginatedRequestParams, RawResource,
        ReadResourceRequestParams, ReadResourceResult, Resource, ResourceContents,
        ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, Json, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Clone)]
pub struct McpServerState {
    pub client: ApiClient,
    pub profile_name: Option<String>,
    pub base_url: String,
    pub session_file: PathBuf,
    pub session_role: Option<String>,
    pub session_subject: Option<String>,
    pub runtime_preference: String,
}

#[derive(Clone)]
struct OpenCodelabsMcpServer {
    state: McpServerState,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CodelabIdParams {
    /// Stable codelab identifier.
    id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ScopedCodelabParams {
    /// Stable codelab identifier.
    codelab_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ResolveHelpRequestParams {
    /// Stable codelab identifier.
    codelab_id: String,
    /// Help request identifier to resolve.
    help_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateCodelabInput {
    /// Display title shown in the workshop UI.
    title: String,
    /// Short summary shown in codelab listings.
    description: String,
    /// Author name displayed on the codelab card.
    author: String,
    /// Whether the codelab can be joined without admin access.
    is_public: Option<bool>,
    /// Whether quizzes are enabled for the codelab.
    quiz_enabled: Option<bool>,
    /// Whether quiz completion is required before marking complete.
    require_quiz: Option<bool>,
    /// Whether learner feedback is required before completion.
    require_feedback: Option<bool>,
    /// Whether a submission is required before completion.
    require_submission: Option<bool>,
    /// Optional facilitator guide markdown shown beside the steps.
    guide_markdown: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateCodelabInput {
    /// Stable codelab identifier.
    id: String,
    /// Display title shown in the workshop UI.
    title: String,
    /// Short summary shown in codelab listings.
    description: String,
    /// Author name displayed on the codelab card.
    author: String,
    /// Whether the codelab can be joined without admin access.
    is_public: Option<bool>,
    /// Whether quizzes are enabled for the codelab.
    quiz_enabled: Option<bool>,
    /// Whether quiz completion is required before marking complete.
    require_quiz: Option<bool>,
    /// Whether learner feedback is required before completion.
    require_feedback: Option<bool>,
    /// Whether a submission is required before completion.
    require_submission: Option<bool>,
    /// Optional facilitator guide markdown shown beside the steps.
    guide_markdown: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ReplaceStepsInput {
    /// Stable codelab identifier.
    codelab_id: String,
    /// Ordered steps that should replace the existing step list.
    steps: Vec<StepInput>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct StepInput {
    /// Existing step identifier when updating in place.
    id: Option<String>,
    /// Step title shown in the workshop UI.
    title: String,
    /// Markdown content for the step body.
    content_markdown: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MaterialIdParams {
    /// Stable codelab identifier.
    codelab_id: String,
    /// Material identifier to delete.
    material_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateMaterialInput {
    /// Stable codelab identifier.
    codelab_id: String,
    /// Material title shown in the UI.
    title: String,
    /// Material kind, usually `link` or `file`.
    material_type: String,
    /// Link URL when `material_type` is `link`.
    link_url: Option<String>,
    /// Uploaded file path or asset URL when `material_type` is `file`.
    file_path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UploadMaterialAssetInput {
    /// Local filesystem path to upload.
    file_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateQuizzesInput {
    /// Stable codelab identifier.
    codelab_id: String,
    /// Replacement quiz list.
    quizzes: Vec<CreateQuizInput>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateQuizInput {
    /// Prompt shown to the learner.
    question: String,
    /// Quiz type such as `multiple_choice`.
    quiz_type: Option<String>,
    /// Ordered answer options.
    options: Vec<String>,
    /// Correct answer index for single-answer quizzes.
    correct_answer: i32,
    /// Correct answer indices for multi-answer quizzes.
    correct_answers: Option<Vec<i32>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct WorkspaceBranchParams {
    /// Stable codelab identifier.
    codelab_id: String,
    /// Branch snapshot name.
    branch: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct WorkspaceFolderParams {
    /// Stable codelab identifier.
    codelab_id: String,
    /// Folder snapshot name.
    folder: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct WorkspaceBranchFileParams {
    /// Stable codelab identifier.
    codelab_id: String,
    /// Branch snapshot name.
    branch: String,
    /// Relative file path inside the branch snapshot.
    file: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct WorkspaceFolderFileParams {
    /// Stable codelab identifier.
    codelab_id: String,
    /// Folder snapshot name.
    folder: String,
    /// Relative file path inside the folder snapshot.
    file: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceTarget<'a> {
    Connection,
    Session,
    Reference,
    CodelabIndex,
    CodelabDetail(&'a str),
    CodelabBundle(&'a str),
    CodelabGuide(&'a str),
    CodelabSteps(&'a str),
    CodelabAttendees(&'a str),
    CodelabHelpRequests(&'a str),
    CodelabMaterials(&'a str),
    CodelabQuizzes(&'a str),
    CodelabFeedback(&'a str),
    CodelabSubmissions(&'a str),
    CodelabQuizSubmissions(&'a str),
    CodelabChatHistory(&'a str),
    WorkspaceInfo(&'a str),
    WorkspaceBranches(&'a str),
    WorkspaceFolders(&'a str),
}

impl OpenCodelabsMcpServer {
    fn new(state: McpServerState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }

    fn is_admin_session(&self) -> bool {
        self.state.session_role.as_deref() == Some("admin")
    }

    fn require_admin_session(&self, action: &str) -> Result<(), McpError> {
        if self.is_admin_session() {
            Ok(())
        } else {
            Err(McpError::internal_error(
                format!(
                    "{action} requires an admin session. Run `oc auth login` before launching `oc mcp serve`."
                ),
                None,
            ))
        }
    }

    async fn build_connection_payload(&self) -> Value {
        let probe = if matches!(self.state.runtime_preference.as_str(), "auto" | "backend") {
            match self.state.client.cli_runtime().await {
                Ok(runtime) => json!({
                    "reachable": true,
                    "runtime": runtime,
                    "probe_error": null,
                }),
                Err(error) => json!({
                    "reachable": false,
                    "runtime": null,
                    "probe_error": error.to_string(),
                }),
            }
        } else {
            json!({
                "reachable": false,
                "runtime": null,
                "probe_error": format!(
                    "Runtime preference `{}` does not expose the backend CLI probe.",
                    self.state.runtime_preference
                ),
            })
        };

        json!({
            "profile": self.state.profile_name,
            "base_url": self.state.base_url,
            "runtime_preference": self.state.runtime_preference,
            "session_file": self.state.session_file,
            "authenticated": self.state.session_role.is_some(),
            "session": {
                "role": self.state.session_role,
                "subject": self.state.session_subject,
            },
            "probe": probe,
        })
    }

    fn build_session_payload(&self) -> Value {
        json!({
            "authenticated": self.state.session_role.is_some(),
            "role": self.state.session_role,
            "subject": self.state.session_subject,
            "session_file": self.state.session_file,
        })
    }

    async fn build_resource_index(&self) -> Vec<Resource> {
        let mut resources = vec![
            resource(
                "oc://connection",
                "connection",
                Some("Open Codelabs connection status"),
                Some("Current oc profile, base URL, runtime preference, and probe result."),
                Some("application/json"),
            ),
            resource(
                "oc://session",
                "session",
                Some("Open Codelabs session status"),
                Some("Current oc session subject, role, and session file path."),
                Some("application/json"),
            ),
            resource(
                "oc://reference",
                "reference",
                Some("Open Codelabs reference"),
                Some("Built-in codelab reference payload used by oc codelab reference."),
                Some("text/plain"),
            ),
            resource(
                "oc://codelabs",
                "codelabs",
                Some("Visible codelabs"),
                Some("List of codelabs visible to the current session."),
                Some("application/json"),
            ),
        ];

        if let Ok(codelabs) = self.state.client.list_codelabs().await {
            for codelab in codelabs {
                resources.push(resource(
                    format!("oc://codelabs/{}", codelab.id),
                    format!("codelab-{}", codelab.id),
                    Some(format!("Codelab: {}", codelab.title)),
                    Some("Codelab metadata and top-level settings."),
                    Some("application/json"),
                ));
                resources.push(resource(
                    format!("oc://codelabs/{}/guide", codelab.id),
                    format!("codelab-guide-{}", codelab.id),
                    Some(format!("Guide markdown: {}", codelab.title)),
                    Some("Facilitator guide markdown for the codelab."),
                    Some("text/markdown"),
                ));
                resources.push(resource(
                    format!("oc://codelabs/{}/steps", codelab.id),
                    format!("codelab-steps-{}", codelab.id),
                    Some(format!("Steps: {}", codelab.title)),
                    Some("Ordered step list for the codelab."),
                    Some("application/json"),
                ));

                if self.is_admin_session() {
                    resources.push(resource(
                        format!("oc://codelabs/{}/bundle", codelab.id),
                        format!("codelab-bundle-{}", codelab.id),
                        Some(format!("Bundle: {}", codelab.title)),
                        Some(
                            "Combined metadata, guide, steps, materials, and quizzes for the codelab. Requires an admin session.",
                        ),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/attendees", codelab.id),
                        format!("codelab-attendees-{}", codelab.id),
                        Some(format!("Attendees: {}", codelab.title)),
                        Some("Learner roster for the codelab. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/help", codelab.id),
                        format!("codelab-help-{}", codelab.id),
                        Some(format!("Help requests: {}", codelab.title)),
                        Some("Open help requests for the codelab. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/materials", codelab.id),
                        format!("codelab-materials-{}", codelab.id),
                        Some(format!("Materials: {}", codelab.title)),
                        Some("Attached materials for the codelab. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/quizzes", codelab.id),
                        format!("codelab-quizzes-{}", codelab.id),
                        Some(format!("Quizzes: {}", codelab.title)),
                        Some("Quiz definitions for the codelab. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/quiz-submissions", codelab.id),
                        format!("codelab-quiz-submissions-{}", codelab.id),
                        Some(format!("Quiz submissions: {}", codelab.title)),
                        Some("Quiz submissions with attendee metadata. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/feedback", codelab.id),
                        format!("codelab-feedback-{}", codelab.id),
                        Some(format!("Feedback: {}", codelab.title)),
                        Some("Feedback rows for the codelab. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/submissions", codelab.id),
                        format!("codelab-submissions-{}", codelab.id),
                        Some(format!("Submissions: {}", codelab.title)),
                        Some("Learner submissions for the codelab. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/chat", codelab.id),
                        format!("codelab-chat-{}", codelab.id),
                        Some(format!("Chat history: {}", codelab.title)),
                        Some("Stored chat history for the codelab. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/workspace", codelab.id),
                        format!("codelab-workspace-{}", codelab.id),
                        Some(format!("Workspace: {}", codelab.title)),
                        Some("Workspace metadata for the codelab. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/workspace/branches", codelab.id),
                        format!("codelab-workspace-branches-{}", codelab.id),
                        Some(format!("Workspace branches: {}", codelab.title)),
                        Some("Branch snapshots for the codelab workspace. Requires an admin session."),
                        Some("application/json"),
                    ));
                    resources.push(resource(
                        format!("oc://codelabs/{}/workspace/folders", codelab.id),
                        format!("codelab-workspace-folders-{}", codelab.id),
                        Some(format!("Workspace folders: {}", codelab.title)),
                        Some("Folder snapshots for the codelab workspace. Requires an admin session."),
                        Some("application/json"),
                    ));
                }
            }
        }

        resources
    }

    async fn codelab_detail_value(&self, id: &str) -> Result<Value, McpError> {
        let (codelab, steps) = self
            .state
            .client
            .get_codelab(id)
            .await
            .map_err(internal_error)?;
        Ok(json!({
            "codelab": codelab,
            "steps_count": steps.len(),
        }))
    }

    async fn codelab_guide_markdown(&self, id: &str) -> Result<String, McpError> {
        let (codelab, _) = self
            .state
            .client
            .get_codelab(id)
            .await
            .map_err(internal_error)?;
        Ok(codelab.guide_markdown.unwrap_or_default())
    }

    async fn codelab_steps_value(&self, id: &str) -> Result<Value, McpError> {
        let (_, steps) = self
            .state
            .client
            .get_codelab(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(steps))
    }

    async fn codelab_bundle_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Reading a codelab bundle")?;
        let ((codelab, steps), materials, quizzes) = tokio::try_join!(
            self.state.client.get_codelab(id),
            self.state.client.get_materials(id),
            self.state.client.get_quizzes(id),
        )
        .map_err(internal_error)?;
        Ok(json!({
            "codelab": codelab,
            "steps": steps,
            "materials": materials,
            "quizzes": quizzes,
        }))
    }

    async fn codelab_attendees_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing attendees")?;
        let attendees = self
            .state
            .client
            .get_attendees(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(attendees))
    }

    async fn codelab_help_requests_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing help requests")?;
        let help_requests = self
            .state
            .client
            .get_help_requests(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(help_requests))
    }

    async fn reference_value(&self) -> Result<String, McpError> {
        self.state
            .client
            .reference_codelabs()
            .await
            .map_err(internal_error)
    }

    async fn codelab_materials_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing materials")?;
        let materials = self
            .state
            .client
            .get_materials(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(materials))
    }

    async fn codelab_quizzes_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing quizzes")?;
        let quizzes = self
            .state
            .client
            .get_quizzes(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(quizzes))
    }

    async fn codelab_quiz_submissions_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing quiz submissions")?;
        let submissions = self
            .state
            .client
            .get_quiz_submissions(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(submissions))
    }

    async fn codelab_feedback_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing feedback")?;
        let feedback = self
            .state
            .client
            .get_feedback(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(feedback))
    }

    async fn codelab_submissions_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing submissions")?;
        let submissions = self
            .state
            .client
            .get_submissions(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(submissions))
    }

    async fn codelab_chat_history_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing chat history")?;
        let chat_history = self
            .state
            .client
            .get_chat_history(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(chat_history))
    }

    async fn workspace_info_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Reading workspace metadata")?;
        let workspace = self
            .state
            .client
            .workspace_info(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(workspace))
    }

    async fn workspace_branches_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing workspace branches")?;
        let branches = self
            .state
            .client
            .list_workspace_branches(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(branches))
    }

    async fn workspace_folders_value(&self, id: &str) -> Result<Value, McpError> {
        self.require_admin_session("Listing workspace folders")?;
        let folders = self
            .state
            .client
            .list_workspace_folders(id)
            .await
            .map_err(internal_error)?;
        Ok(json!(folders))
    }
}

#[tool_router(router = tool_router)]
impl OpenCodelabsMcpServer {
    #[tool(
        name = "get_connection",
        description = "Return the active Open Codelabs base URL, runtime probe, and session summary."
    )]
    async fn get_connection(&self) -> Json<Value> {
        Json(self.build_connection_payload().await)
    }

    #[tool(
        name = "list_codelabs",
        description = "List codelabs visible to the current Open Codelabs session."
    )]
    async fn list_codelabs(&self) -> Result<Json<Value>, McpError> {
        let codelabs = self
            .state
            .client
            .list_codelabs()
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(codelabs)))
    }

    #[tool(
        name = "get_codelab",
        description = "Fetch codelab metadata, guide markdown, and ordered steps for a specific codelab."
    )]
    async fn get_codelab(
        &self,
        params: Parameters<CodelabIdParams>,
    ) -> Result<Json<Value>, McpError> {
        let (codelab, steps) = self
            .state
            .client
            .get_codelab(&params.0.id)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!({
            "codelab": codelab,
            "steps": steps,
        })))
    }

    #[tool(
        name = "get_codelab_bundle",
        description = "Fetch codelab metadata, guide, steps, materials, and quizzes together. Requires an admin session."
    )]
    async fn get_codelab_bundle(
        &self,
        params: Parameters<CodelabIdParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self.codelab_bundle_value(&params.0.id).await?;
        Ok(Json(value))
    }

    #[tool(
        name = "create_codelab",
        description = "Create a new codelab. Requires an admin session."
    )]
    async fn create_codelab(
        &self,
        params: Parameters<CreateCodelabInput>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Creating a codelab")?;
        let codelab = self
            .state
            .client
            .create_codelab(&into_create_codelab(params.0))
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(codelab)))
    }

    #[tool(
        name = "update_codelab",
        description = "Update codelab metadata. Requires an admin session."
    )]
    async fn update_codelab(
        &self,
        params: Parameters<UpdateCodelabInput>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Updating a codelab")?;
        let input = params.0;
        let codelab = self
            .state
            .client
            .update_codelab(&input.id, &into_update_codelab(&input))
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(codelab)))
    }

    #[tool(
        name = "replace_codelab_steps",
        description = "Replace the full ordered step list for a codelab. Requires an admin session."
    )]
    async fn replace_codelab_steps(
        &self,
        params: Parameters<ReplaceStepsInput>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Replacing codelab steps")?;
        let codelab_id = params.0.codelab_id.clone();
        let payload = json!({
            "steps": params
                .0
                .steps
                .into_iter()
                .map(|step| CreateStep {
                    id: step.id,
                    title: step.title,
                    content_markdown: step.content_markdown,
                })
                .collect::<Vec<_>>(),
        });
        let payload: UpdateStepsPayload =
            serde_json::from_value(payload).map_err(internal_error)?;
        self.state
            .client
            .push_steps(&codelab_id, &payload)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!({
            "status": "ok",
            "codelab_id": codelab_id,
        })))
    }

    #[tool(
        name = "get_codelab_reference",
        description = "Return the built-in Open Codelabs reference payload."
    )]
    async fn get_codelab_reference(&self) -> Result<String, McpError> {
        self.reference_value().await
    }

    #[tool(
        name = "copy_codelab",
        description = "Copy a codelab including its steps. Requires an admin session."
    )]
    async fn copy_codelab(
        &self,
        params: Parameters<CodelabIdParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Copying a codelab")?;
        let codelab = self
            .state
            .client
            .copy_codelab(&params.0.id)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(codelab)))
    }

    #[tool(
        name = "delete_codelab",
        description = "Delete a codelab and its related data. Requires an admin session."
    )]
    async fn delete_codelab(
        &self,
        params: Parameters<CodelabIdParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Deleting a codelab")?;
        self.state
            .client
            .delete_codelab(&params.0.id)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!({
            "status": "deleted",
            "id": params.0.id,
        })))
    }

    #[tool(
        name = "list_materials",
        description = "List codelab materials. Requires an admin session."
    )]
    async fn list_materials(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self.codelab_materials_value(&params.0.codelab_id).await?;
        Ok(Json(value))
    }

    #[tool(
        name = "upload_material_asset",
        description = "Upload a local file and return the asset URL for a codelab material. Requires an admin session."
    )]
    async fn upload_material_asset(
        &self,
        params: Parameters<UploadMaterialAssetInput>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Uploading a material asset")?;
        let uploaded = self
            .state
            .client
            .upload_material(std::path::Path::new(&params.0.file_path))
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(uploaded)))
    }

    #[tool(
        name = "add_material",
        description = "Add a material record to a codelab. Requires an admin session."
    )]
    async fn add_material(
        &self,
        params: Parameters<CreateMaterialInput>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Adding a material")?;
        let input = params.0;
        let codelab_id = input.codelab_id.clone();
        let payload = CreateMaterial {
            title: input.title,
            material_type: input.material_type,
            link_url: input.link_url,
            file_path: input.file_path,
        };
        let material = self
            .state
            .client
            .add_material(&codelab_id, &payload)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(material)))
    }

    #[tool(
        name = "delete_material",
        description = "Delete a material record from a codelab. Requires an admin session."
    )]
    async fn delete_material(
        &self,
        params: Parameters<MaterialIdParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Deleting a material")?;
        self.state
            .client
            .delete_material(&params.0.codelab_id, &params.0.material_id)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!({
            "status": "deleted",
            "codelab_id": params.0.codelab_id,
            "material_id": params.0.material_id,
        })))
    }

    #[tool(
        name = "list_quizzes",
        description = "List quizzes for a codelab. Requires an admin session."
    )]
    async fn list_quizzes(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self.codelab_quizzes_value(&params.0.codelab_id).await?;
        Ok(Json(value))
    }

    #[tool(
        name = "update_quizzes",
        description = "Replace the full quiz set for a codelab. Requires an admin session."
    )]
    async fn update_quizzes(
        &self,
        params: Parameters<UpdateQuizzesInput>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Updating quizzes")?;
        let input = params.0;
        let codelab_id = input.codelab_id;
        let quizzes = input
            .quizzes
            .into_iter()
            .map(|quiz| CreateQuiz {
                question: quiz.question,
                quiz_type: quiz.quiz_type,
                options: quiz.options,
                correct_answer: quiz.correct_answer,
                correct_answers: quiz.correct_answers,
            })
            .collect::<Vec<_>>();
        self.state
            .client
            .update_quizzes(&codelab_id, &quizzes)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!({
            "status": "ok",
            "codelab_id": codelab_id,
            "quizzes_count": quizzes.len(),
        })))
    }

    #[tool(
        name = "list_feedback",
        description = "List attendee feedback for a codelab. Requires an admin session."
    )]
    async fn list_feedback(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self.codelab_feedback_value(&params.0.codelab_id).await?;
        Ok(Json(value))
    }

    #[tool(
        name = "list_submissions",
        description = "List learner submissions for a codelab. Requires an admin session."
    )]
    async fn list_submissions(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self.codelab_submissions_value(&params.0.codelab_id).await?;
        Ok(Json(value))
    }

    #[tool(
        name = "get_chat_history",
        description = "List stored chat history for a codelab. Requires an admin session."
    )]
    async fn get_chat_history(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self
            .codelab_chat_history_value(&params.0.codelab_id)
            .await?;
        Ok(Json(value))
    }

    #[tool(
        name = "list_quiz_submissions",
        description = "List quiz submissions for a codelab. Requires an admin session."
    )]
    async fn list_quiz_submissions(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self
            .codelab_quiz_submissions_value(&params.0.codelab_id)
            .await?;
        Ok(Json(value))
    }

    #[tool(
        name = "get_workspace_info",
        description = "Read workspace metadata for a codelab. Requires an admin session."
    )]
    async fn get_workspace_info(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self.workspace_info_value(&params.0.codelab_id).await?;
        Ok(Json(value))
    }

    #[tool(
        name = "list_workspace_branches",
        description = "List branch snapshots for a codelab workspace. Requires an admin session."
    )]
    async fn list_workspace_branches(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self.workspace_branches_value(&params.0.codelab_id).await?;
        Ok(Json(value))
    }

    #[tool(
        name = "list_workspace_folders",
        description = "List folder snapshots for a codelab workspace. Requires an admin session."
    )]
    async fn list_workspace_folders(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        let value = self.workspace_folders_value(&params.0.codelab_id).await?;
        Ok(Json(value))
    }

    #[tool(
        name = "list_workspace_branch_files",
        description = "List file paths inside a workspace branch snapshot. Requires an admin session."
    )]
    async fn list_workspace_branch_files(
        &self,
        params: Parameters<WorkspaceBranchParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Listing workspace branch files")?;
        let files = self
            .state
            .client
            .list_workspace_files(&params.0.codelab_id, &params.0.branch)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(files)))
    }

    #[tool(
        name = "read_workspace_branch_file",
        description = "Read one file from a workspace branch snapshot. Requires an admin session."
    )]
    async fn read_workspace_branch_file(
        &self,
        params: Parameters<WorkspaceBranchFileParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Reading a workspace branch file")?;
        let input = params.0;
        let content = self
            .state
            .client
            .read_workspace_file(&input.codelab_id, &input.branch, &input.file)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!({
            "codelab_id": input.codelab_id,
            "branch": input.branch,
            "file": input.file,
            "content": content,
        })))
    }

    #[tool(
        name = "list_workspace_folder_files",
        description = "List file paths inside a workspace folder snapshot. Requires an admin session."
    )]
    async fn list_workspace_folder_files(
        &self,
        params: Parameters<WorkspaceFolderParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Listing workspace folder files")?;
        let files = self
            .state
            .client
            .list_workspace_folder_files(&params.0.codelab_id, &params.0.folder)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(files)))
    }

    #[tool(
        name = "read_workspace_folder_file",
        description = "Read one file from a workspace folder snapshot. Requires an admin session."
    )]
    async fn read_workspace_folder_file(
        &self,
        params: Parameters<WorkspaceFolderFileParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Reading a workspace folder file")?;
        let input = params.0;
        let content = self
            .state
            .client
            .read_workspace_folder_file(&input.codelab_id, &input.folder, &input.file)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!({
            "codelab_id": input.codelab_id,
            "folder": input.folder,
            "file": input.file,
            "content": content,
        })))
    }

    #[tool(
        name = "list_attendees",
        description = "List attendee records for a codelab. Requires an admin session."
    )]
    async fn list_attendees(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Listing attendees")?;
        let attendees = self
            .state
            .client
            .get_attendees(&params.0.codelab_id)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(attendees)))
    }

    #[tool(
        name = "list_help_requests",
        description = "List active help requests for a codelab. Requires an admin session."
    )]
    async fn list_help_requests(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Listing help requests")?;
        let help_requests = self
            .state
            .client
            .get_help_requests(&params.0.codelab_id)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!(help_requests)))
    }

    #[tool(
        name = "resolve_help_request",
        description = "Resolve an active help request. Requires an admin session."
    )]
    async fn resolve_help_request(
        &self,
        params: Parameters<ResolveHelpRequestParams>,
    ) -> Result<Json<Value>, McpError> {
        self.require_admin_session("Resolving a help request")?;
        self.state
            .client
            .resolve_help_request(&params.0.codelab_id, &params.0.help_id)
            .await
            .map_err(internal_error)?;
        Ok(Json(json!({
            "status": "resolved",
            "codelab_id": params.0.codelab_id,
            "help_id": params.0.help_id,
        })))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for OpenCodelabsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Use the Open Codelabs MCP server to inspect connection state, read full codelab bundles, materials, quizzes, submissions, workspace metadata, and perform focused admin actions when the oc session is authenticated."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: rmcp::model::Implementation {
                name: "open-codelabs-mcp".to_string(),
                title: Some("Open Codelabs MCP".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: Some(
                    "MCP bridge for Open Codelabs, reusing oc connect/auth state.".to_string(),
                ),
                icons: None,
                website_url: Some("https://github.com/JAICHANGPARK/open-codelabs".to_string()),
            },
            ..Default::default()
        }
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            Ok(ListResourcesResult::with_all_items(
                self.build_resource_index().await,
            ))
        }
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            match parse_resource_target(&request.uri)? {
                ResourceTarget::Connection => {
                    json_resource(&request.uri, self.build_connection_payload().await)
                }
                ResourceTarget::Session => {
                    json_resource(&request.uri, self.build_session_payload())
                }
                ResourceTarget::Reference => {
                    let reference = self.reference_value().await?;
                    text_resource(&request.uri, "text/plain", reference)
                }
                ResourceTarget::CodelabIndex => {
                    let codelabs = self
                        .state
                        .client
                        .list_codelabs()
                        .await
                        .map_err(internal_error)?;
                    json_resource(&request.uri, json!(codelabs))
                }
                ResourceTarget::CodelabDetail(id) => {
                    let value = self.codelab_detail_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabBundle(id) => {
                    let value = self.codelab_bundle_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabGuide(id) => {
                    let markdown = self.codelab_guide_markdown(id).await?;
                    markdown_resource(&request.uri, markdown)
                }
                ResourceTarget::CodelabSteps(id) => {
                    let value = self.codelab_steps_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabAttendees(id) => {
                    let value = self.codelab_attendees_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabHelpRequests(id) => {
                    let value = self.codelab_help_requests_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabMaterials(id) => {
                    let value = self.codelab_materials_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabQuizzes(id) => {
                    let value = self.codelab_quizzes_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabFeedback(id) => {
                    let value = self.codelab_feedback_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabSubmissions(id) => {
                    let value = self.codelab_submissions_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabQuizSubmissions(id) => {
                    let value = self.codelab_quiz_submissions_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::CodelabChatHistory(id) => {
                    let value = self.codelab_chat_history_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::WorkspaceInfo(id) => {
                    let value = self.workspace_info_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::WorkspaceBranches(id) => {
                    let value = self.workspace_branches_value(id).await?;
                    json_resource(&request.uri, value)
                }
                ResourceTarget::WorkspaceFolders(id) => {
                    let value = self.workspace_folders_value(id).await?;
                    json_resource(&request.uri, value)
                }
            }
        }
    }
}

pub async fn serve_stdio(state: McpServerState) -> Result<()> {
    let service = OpenCodelabsMcpServer::new(state)
        .serve(stdio())
        .await
        .context("Failed to start Open Codelabs MCP server over stdio")?;
    service
        .waiting()
        .await
        .context("Open Codelabs MCP server stopped unexpectedly")?;
    Ok(())
}

fn into_create_codelab(input: CreateCodelabInput) -> CreateCodelab {
    CreateCodelab {
        title: input.title,
        description: input.description,
        author: input.author,
        is_public: input.is_public,
        quiz_enabled: input.quiz_enabled,
        require_quiz: input.require_quiz,
        require_feedback: input.require_feedback,
        require_submission: input.require_submission,
        guide_markdown: input.guide_markdown,
    }
}

fn into_update_codelab(input: &UpdateCodelabInput) -> CreateCodelab {
    CreateCodelab {
        title: input.title.clone(),
        description: input.description.clone(),
        author: input.author.clone(),
        is_public: input.is_public,
        quiz_enabled: input.quiz_enabled,
        require_quiz: input.require_quiz,
        require_feedback: input.require_feedback,
        require_submission: input.require_submission,
        guide_markdown: input.guide_markdown.clone(),
    }
}

fn parse_resource_target(uri: &str) -> Result<ResourceTarget<'_>, McpError> {
    if uri == "oc://connection" {
        return Ok(ResourceTarget::Connection);
    }
    if uri == "oc://session" {
        return Ok(ResourceTarget::Session);
    }
    if uri == "oc://reference" {
        return Ok(ResourceTarget::Reference);
    }
    if uri == "oc://codelabs" {
        return Ok(ResourceTarget::CodelabIndex);
    }
    if let Some(rest) = uri.strip_prefix("oc://codelabs/") {
        let parts = rest.split('/').collect::<Vec<_>>();
        return match parts.as_slice() {
            [id] if !id.is_empty() => Ok(ResourceTarget::CodelabDetail(id)),
            [id, "bundle"] if !id.is_empty() => Ok(ResourceTarget::CodelabBundle(id)),
            [id, "guide"] if !id.is_empty() => Ok(ResourceTarget::CodelabGuide(id)),
            [id, "steps"] if !id.is_empty() => Ok(ResourceTarget::CodelabSteps(id)),
            [id, "attendees"] if !id.is_empty() => Ok(ResourceTarget::CodelabAttendees(id)),
            [id, "help"] if !id.is_empty() => Ok(ResourceTarget::CodelabHelpRequests(id)),
            [id, "materials"] if !id.is_empty() => Ok(ResourceTarget::CodelabMaterials(id)),
            [id, "quizzes"] if !id.is_empty() => Ok(ResourceTarget::CodelabQuizzes(id)),
            [id, "quiz-submissions"] if !id.is_empty() => {
                Ok(ResourceTarget::CodelabQuizSubmissions(id))
            }
            [id, "feedback"] if !id.is_empty() => Ok(ResourceTarget::CodelabFeedback(id)),
            [id, "submissions"] if !id.is_empty() => Ok(ResourceTarget::CodelabSubmissions(id)),
            [id, "chat"] if !id.is_empty() => Ok(ResourceTarget::CodelabChatHistory(id)),
            [id, "workspace"] if !id.is_empty() => Ok(ResourceTarget::WorkspaceInfo(id)),
            [id, "workspace", "branches"] if !id.is_empty() => {
                Ok(ResourceTarget::WorkspaceBranches(id))
            }
            [id, "workspace", "folders"] if !id.is_empty() => {
                Ok(ResourceTarget::WorkspaceFolders(id))
            }
            _ => Err(McpError::resource_not_found(
                format!("Unknown Open Codelabs resource URI: {uri}"),
                None,
            )),
        };
    }

    Err(McpError::resource_not_found(
        format!("Unknown Open Codelabs resource URI: {uri}"),
        None,
    ))
}

fn resource<U, N, T, D>(
    uri: U,
    name: N,
    title: Option<T>,
    description: Option<D>,
    mime_type: Option<&str>,
) -> Resource
where
    U: Into<String>,
    N: Into<String>,
    T: Into<String>,
    D: Into<String>,
{
    let mut raw = RawResource::new(uri, name);
    raw.title = title.map(Into::into);
    raw.description = description.map(Into::into);
    raw.mime_type = mime_type.map(str::to_string);
    Annotated::new(raw, None)
}

fn json_resource(uri: &str, value: Value) -> Result<ReadResourceResult, McpError> {
    let text = serde_json::to_string_pretty(&value).map_err(internal_error)?;
    Ok(ReadResourceResult {
        contents: vec![ResourceContents::TextResourceContents {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            text,
            meta: None,
        }],
    })
}

fn markdown_resource(uri: &str, markdown: String) -> Result<ReadResourceResult, McpError> {
    text_resource(uri, "text/markdown", markdown)
}

fn text_resource(uri: &str, mime_type: &str, text: String) -> Result<ReadResourceResult, McpError> {
    Ok(ReadResourceResult {
        contents: vec![ResourceContents::TextResourceContents {
            uri: uri.to_string(),
            mime_type: Some(mime_type.to_string()),
            text,
            meta: None,
        }],
    })
}

fn internal_error(error: impl std::fmt::Display) -> McpError {
    McpError::internal_error(error.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::{parse_resource_target, ResourceTarget};

    #[test]
    fn parses_known_resource_uris() {
        assert_eq!(
            parse_resource_target("oc://connection").expect("connection"),
            ResourceTarget::Connection
        );
        assert_eq!(
            parse_resource_target("oc://session").expect("session"),
            ResourceTarget::Session
        );
        assert_eq!(
            parse_resource_target("oc://reference").expect("reference"),
            ResourceTarget::Reference
        );
        assert_eq!(
            parse_resource_target("oc://codelabs").expect("codelabs"),
            ResourceTarget::CodelabIndex
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1").expect("detail"),
            ResourceTarget::CodelabDetail("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/bundle").expect("bundle"),
            ResourceTarget::CodelabBundle("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/guide").expect("guide"),
            ResourceTarget::CodelabGuide("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/steps").expect("steps"),
            ResourceTarget::CodelabSteps("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/attendees").expect("attendees"),
            ResourceTarget::CodelabAttendees("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/help").expect("help"),
            ResourceTarget::CodelabHelpRequests("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/materials").expect("materials"),
            ResourceTarget::CodelabMaterials("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/quizzes").expect("quizzes"),
            ResourceTarget::CodelabQuizzes("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/quiz-submissions")
                .expect("quiz submissions"),
            ResourceTarget::CodelabQuizSubmissions("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/feedback").expect("feedback"),
            ResourceTarget::CodelabFeedback("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/submissions").expect("submissions"),
            ResourceTarget::CodelabSubmissions("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/chat").expect("chat"),
            ResourceTarget::CodelabChatHistory("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/workspace").expect("workspace"),
            ResourceTarget::WorkspaceInfo("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/workspace/branches")
                .expect("workspace branches"),
            ResourceTarget::WorkspaceBranches("lab-1")
        );
        assert_eq!(
            parse_resource_target("oc://codelabs/lab-1/workspace/folders")
                .expect("workspace folders"),
            ResourceTarget::WorkspaceFolders("lab-1")
        );
    }

    #[test]
    fn rejects_unknown_resource_uris() {
        assert!(parse_resource_target("oc://unknown").is_err());
        assert!(parse_resource_target("oc://codelabs/lab-1/unknown").is_err());
    }
}
