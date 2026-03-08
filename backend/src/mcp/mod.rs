use crate::cli::client::ApiClient;
use crate::domain::models::{
    CreateCodelab, CreateMaterial, CreateQuiz, CreateStep, UpdateStepsPayload,
};
use anyhow::{Context, Result};
use rmcp::{
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::{
        Annotated, GetPromptRequestParams, GetPromptResult, ListPromptsResult, ListResourcesResult,
        PaginatedRequestParams, PromptMessage, PromptMessageRole, RawResource,
        ReadResourceRequestParams, ReadResourceResult, Resource, ResourceContents,
        ServerCapabilities, ServerInfo,
    },
    prompt, prompt_router,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, Json, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
    prompt_router: PromptRouter<Self>,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct McpToolPayload {
    /// Structured result payload returned by the tool.
    data: Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FacilitatorBriefPromptInput {
    /// Stable codelab identifier to brief.
    codelab_id: String,
    /// Optional focus area such as pacing, quizzes, troubleshooting, or demos.
    focus: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AuthoringChangePromptInput {
    /// Stable codelab identifier to edit.
    codelab_id: String,
    /// Requested change or authoring intent.
    request: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct HelpQueuePromptInput {
    /// Stable codelab identifier to triage.
    codelab_id: String,
    /// Optional triage policy or operating instruction.
    focus: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct LearnerOpsPromptInput {
    /// Stable codelab identifier to review.
    codelab_id: String,
    /// Optional operator question to answer while reviewing learner progress.
    focus: Option<String>,
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
            prompt_router: Self::prompt_router(),
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

    fn prompt_focus_suffix(focus: Option<&str>) -> String {
        match focus.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => format!(" Focus on: {value}."),
            None => String::new(),
        }
    }

    fn codelab_prompt_link(&self, id: &str, suffix: &str) -> PromptMessage {
        let (uri, name, title, description, mime_type) = match suffix {
            "bundle" => (
                format!("oc://codelabs/{id}/bundle"),
                format!("codelab-bundle-{id}"),
                format!("Bundle: {id}"),
                "Combined metadata, guide, steps, materials, and quizzes.",
                Some("application/json"),
            ),
            "guide" => (
                format!("oc://codelabs/{id}/guide"),
                format!("codelab-guide-{id}"),
                format!("Guide markdown: {id}"),
                "Facilitator guide markdown.",
                Some("text/markdown"),
            ),
            "steps" => (
                format!("oc://codelabs/{id}/steps"),
                format!("codelab-steps-{id}"),
                format!("Steps: {id}"),
                "Ordered codelab steps.",
                Some("application/json"),
            ),
            "attendees" => (
                format!("oc://codelabs/{id}/attendees"),
                format!("codelab-attendees-{id}"),
                format!("Attendees: {id}"),
                "Learner roster for the codelab.",
                Some("application/json"),
            ),
            "help" => (
                format!("oc://codelabs/{id}/help"),
                format!("codelab-help-{id}"),
                format!("Help requests: {id}"),
                "Active help requests for the codelab.",
                Some("application/json"),
            ),
            "materials" => (
                format!("oc://codelabs/{id}/materials"),
                format!("codelab-materials-{id}"),
                format!("Materials: {id}"),
                "Attached codelab materials.",
                Some("application/json"),
            ),
            "quizzes" => (
                format!("oc://codelabs/{id}/quizzes"),
                format!("codelab-quizzes-{id}"),
                format!("Quizzes: {id}"),
                "Quiz definitions for the codelab.",
                Some("application/json"),
            ),
            "quiz-submissions" => (
                format!("oc://codelabs/{id}/quiz-submissions"),
                format!("codelab-quiz-submissions-{id}"),
                format!("Quiz submissions: {id}"),
                "Quiz submissions with learner metadata.",
                Some("application/json"),
            ),
            "feedback" => (
                format!("oc://codelabs/{id}/feedback"),
                format!("codelab-feedback-{id}"),
                format!("Feedback: {id}"),
                "Learner feedback rows.",
                Some("application/json"),
            ),
            "submissions" => (
                format!("oc://codelabs/{id}/submissions"),
                format!("codelab-submissions-{id}"),
                format!("Submissions: {id}"),
                "Learner submissions for the codelab.",
                Some("application/json"),
            ),
            "chat" => (
                format!("oc://codelabs/{id}/chat"),
                format!("codelab-chat-{id}"),
                format!("Chat history: {id}"),
                "Stored chat history for the codelab.",
                Some("application/json"),
            ),
            _ => (
                format!("oc://codelabs/{id}"),
                format!("codelab-{id}"),
                format!("Codelab: {id}"),
                "Codelab metadata and top-level settings.",
                Some("application/json"),
            ),
        };

        PromptMessage::new_resource_link(
            PromptMessageRole::User,
            resource(uri, name, Some(title), Some(description), mime_type),
        )
    }
}

#[prompt_router]
impl OpenCodelabsMcpServer {
    #[prompt(
        name = "facilitator-brief",
        description = "Prepare a facilitator briefing for one codelab using guide, steps, and optional admin-only operational context."
    )]
    async fn facilitator_brief_prompt(
        &self,
        params: Parameters<FacilitatorBriefPromptInput>,
    ) -> GetPromptResult {
        let input = params.0;
        let mut messages = vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "Prepare a facilitator briefing for codelab `{}`. Summarize the learning goals, recommended pacing, likely sticking points, and what the facilitator should watch for during the session.{}",
                    input.codelab_id,
                    Self::prompt_focus_suffix(input.focus.as_deref())
                ),
            ),
            self.codelab_prompt_link(&input.codelab_id, "detail"),
            self.codelab_prompt_link(&input.codelab_id, "guide"),
            self.codelab_prompt_link(&input.codelab_id, "steps"),
        ];

        if self.is_admin_session() {
            messages.push(self.codelab_prompt_link(&input.codelab_id, "materials"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "quizzes"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "help"));
        }

        GetPromptResult {
            description: Some(
                "Guide the model to create a facilitator-oriented briefing from the codelab context."
                    .to_string(),
            ),
            messages,
        }
    }

    #[prompt(
        name = "authoring-change-plan",
        description = "Review a codelab and turn a change request into a concrete authoring plan, using write tools when admin access is available."
    )]
    async fn authoring_change_plan_prompt(
        &self,
        params: Parameters<AuthoringChangePromptInput>,
    ) -> GetPromptResult {
        let input = params.0;
        let mut messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "Review codelab `{}` and turn this change request into a concrete update plan: {}. Read the current content first, call out which metadata, steps, materials, or quizzes should change, and only apply write tools after you can explain the intended edits clearly.",
                input.codelab_id, input.request
            ),
        )];

        if self.is_admin_session() {
            messages.push(self.codelab_prompt_link(&input.codelab_id, "bundle"));
        } else {
            messages.push(self.codelab_prompt_link(&input.codelab_id, "detail"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "guide"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "steps"));
            messages.push(PromptMessage::new_text(
                PromptMessageRole::User,
                "The current MCP session is not an admin session, so write tools may fail until `oc auth login` is refreshed.".to_string(),
            ));
        }

        GetPromptResult {
            description: Some(
                "Guide the model through a safe codelab authoring workflow grounded in the current content."
                    .to_string(),
            ),
            messages,
        }
    }

    #[prompt(
        name = "help-queue-triage",
        description = "Triage facilitator help requests for one codelab using learner, chat, and content context."
    )]
    async fn help_queue_triage_prompt(
        &self,
        params: Parameters<HelpQueuePromptInput>,
    ) -> GetPromptResult {
        let input = params.0;
        let mut messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "Triage the active help queue for codelab `{}`. Group similar issues, identify which requests are blocked or urgent, and recommend the next facilitator actions.{}",
                input.codelab_id,
                Self::prompt_focus_suffix(input.focus.as_deref())
            ),
        )];

        if self.is_admin_session() {
            messages.push(self.codelab_prompt_link(&input.codelab_id, "help"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "attendees"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "chat"));
        } else {
            messages.push(PromptMessage::new_text(
                PromptMessageRole::User,
                "The current MCP session does not have admin access, so help-queue resources are unavailable until the operator refreshes the session with `oc auth login`.".to_string(),
            ));
        }

        messages.push(self.codelab_prompt_link(&input.codelab_id, "guide"));
        messages.push(self.codelab_prompt_link(&input.codelab_id, "steps"));

        GetPromptResult {
            description: Some(
                "Use help requests, learner activity, and codelab context to produce a facilitator triage summary."
                    .to_string(),
            ),
            messages,
        }
    }

    #[prompt(
        name = "learner-ops-review",
        description = "Review learner progress for one codelab across attendees, submissions, quizzes, and feedback."
    )]
    async fn learner_ops_review_prompt(
        &self,
        params: Parameters<LearnerOpsPromptInput>,
    ) -> GetPromptResult {
        let input = params.0;
        let mut messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "Review learner progress for codelab `{}`. Explain who is blocked, who looks complete, and which follow-ups the facilitator should take next.{}",
                input.codelab_id,
                Self::prompt_focus_suffix(input.focus.as_deref())
            ),
        )];

        if self.is_admin_session() {
            messages.push(self.codelab_prompt_link(&input.codelab_id, "attendees"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "submissions"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "quiz-submissions"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "feedback"));
            messages.push(self.codelab_prompt_link(&input.codelab_id, "quizzes"));
        } else {
            messages.push(PromptMessage::new_text(
                PromptMessageRole::User,
                "The current MCP session does not have admin access, so learner operations data is unavailable until the operator refreshes the session with `oc auth login`.".to_string(),
            ));
        }

        messages.push(self.codelab_prompt_link(&input.codelab_id, "steps"));

        GetPromptResult {
            description: Some(
                "Help the model review learner progress and facilitator follow-up actions using attendee and submission context."
                    .to_string(),
            ),
            messages,
        }
    }
}

#[tool_router(router = tool_router)]
impl OpenCodelabsMcpServer {
    #[tool(
        name = "get_connection",
        description = "Return the active Open Codelabs base URL, runtime probe, and session summary."
    )]
    async fn get_connection(&self) -> Json<McpToolPayload> {
        tool_payload(self.build_connection_payload().await)
    }

    #[tool(
        name = "list_codelabs",
        description = "List codelabs visible to the current Open Codelabs session."
    )]
    async fn list_codelabs(&self) -> Result<Json<McpToolPayload>, McpError> {
        let codelabs = self
            .state
            .client
            .list_codelabs()
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!(codelabs)))
    }

    #[tool(
        name = "get_codelab",
        description = "Fetch codelab metadata, guide markdown, and ordered steps for a specific codelab."
    )]
    async fn get_codelab(
        &self,
        params: Parameters<CodelabIdParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        let (codelab, steps) = self
            .state
            .client
            .get_codelab(&params.0.id)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!({
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
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self.codelab_bundle_value(&params.0.id).await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "create_codelab",
        description = "Create a new codelab. Requires an admin session."
    )]
    async fn create_codelab(
        &self,
        params: Parameters<CreateCodelabInput>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Creating a codelab")?;
        let codelab = self
            .state
            .client
            .create_codelab(&into_create_codelab(params.0))
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!(codelab)))
    }

    #[tool(
        name = "update_codelab",
        description = "Update codelab metadata. Requires an admin session."
    )]
    async fn update_codelab(
        &self,
        params: Parameters<UpdateCodelabInput>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Updating a codelab")?;
        let input = params.0;
        let codelab = self
            .state
            .client
            .update_codelab(&input.id, &into_update_codelab(&input))
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!(codelab)))
    }

    #[tool(
        name = "replace_codelab_steps",
        description = "Replace the full ordered step list for a codelab. Requires an admin session."
    )]
    async fn replace_codelab_steps(
        &self,
        params: Parameters<ReplaceStepsInput>,
    ) -> Result<Json<McpToolPayload>, McpError> {
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
        Ok(tool_payload(json!({
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
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Copying a codelab")?;
        let codelab = self
            .state
            .client
            .copy_codelab(&params.0.id)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!(codelab)))
    }

    #[tool(
        name = "delete_codelab",
        description = "Delete a codelab and its related data. Requires an admin session."
    )]
    async fn delete_codelab(
        &self,
        params: Parameters<CodelabIdParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Deleting a codelab")?;
        self.state
            .client
            .delete_codelab(&params.0.id)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!({
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
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self.codelab_materials_value(&params.0.codelab_id).await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "upload_material_asset",
        description = "Upload a local file and return the asset URL for a codelab material. Requires an admin session."
    )]
    async fn upload_material_asset(
        &self,
        params: Parameters<UploadMaterialAssetInput>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Uploading a material asset")?;
        let uploaded = self
            .state
            .client
            .upload_material(std::path::Path::new(&params.0.file_path))
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!(uploaded)))
    }

    #[tool(
        name = "add_material",
        description = "Add a material record to a codelab. Requires an admin session."
    )]
    async fn add_material(
        &self,
        params: Parameters<CreateMaterialInput>,
    ) -> Result<Json<McpToolPayload>, McpError> {
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
        Ok(tool_payload(json!(material)))
    }

    #[tool(
        name = "delete_material",
        description = "Delete a material record from a codelab. Requires an admin session."
    )]
    async fn delete_material(
        &self,
        params: Parameters<MaterialIdParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Deleting a material")?;
        self.state
            .client
            .delete_material(&params.0.codelab_id, &params.0.material_id)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!({
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
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self.codelab_quizzes_value(&params.0.codelab_id).await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "update_quizzes",
        description = "Replace the full quiz set for a codelab. Requires an admin session."
    )]
    async fn update_quizzes(
        &self,
        params: Parameters<UpdateQuizzesInput>,
    ) -> Result<Json<McpToolPayload>, McpError> {
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
        Ok(tool_payload(json!({
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
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self.codelab_feedback_value(&params.0.codelab_id).await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "list_submissions",
        description = "List learner submissions for a codelab. Requires an admin session."
    )]
    async fn list_submissions(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self.codelab_submissions_value(&params.0.codelab_id).await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "get_chat_history",
        description = "List stored chat history for a codelab. Requires an admin session."
    )]
    async fn get_chat_history(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self
            .codelab_chat_history_value(&params.0.codelab_id)
            .await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "list_quiz_submissions",
        description = "List quiz submissions for a codelab. Requires an admin session."
    )]
    async fn list_quiz_submissions(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self
            .codelab_quiz_submissions_value(&params.0.codelab_id)
            .await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "get_workspace_info",
        description = "Read workspace metadata for a codelab. Requires an admin session."
    )]
    async fn get_workspace_info(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self.workspace_info_value(&params.0.codelab_id).await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "list_workspace_branches",
        description = "List branch snapshots for a codelab workspace. Requires an admin session."
    )]
    async fn list_workspace_branches(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self.workspace_branches_value(&params.0.codelab_id).await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "list_workspace_folders",
        description = "List folder snapshots for a codelab workspace. Requires an admin session."
    )]
    async fn list_workspace_folders(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        let value = self.workspace_folders_value(&params.0.codelab_id).await?;
        Ok(tool_payload(value))
    }

    #[tool(
        name = "list_workspace_branch_files",
        description = "List file paths inside a workspace branch snapshot. Requires an admin session."
    )]
    async fn list_workspace_branch_files(
        &self,
        params: Parameters<WorkspaceBranchParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Listing workspace branch files")?;
        let files = self
            .state
            .client
            .list_workspace_files(&params.0.codelab_id, &params.0.branch)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!(files)))
    }

    #[tool(
        name = "read_workspace_branch_file",
        description = "Read one file from a workspace branch snapshot. Requires an admin session."
    )]
    async fn read_workspace_branch_file(
        &self,
        params: Parameters<WorkspaceBranchFileParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Reading a workspace branch file")?;
        let input = params.0;
        let content = self
            .state
            .client
            .read_workspace_file(&input.codelab_id, &input.branch, &input.file)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!({
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
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Listing workspace folder files")?;
        let files = self
            .state
            .client
            .list_workspace_folder_files(&params.0.codelab_id, &params.0.folder)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!(files)))
    }

    #[tool(
        name = "read_workspace_folder_file",
        description = "Read one file from a workspace folder snapshot. Requires an admin session."
    )]
    async fn read_workspace_folder_file(
        &self,
        params: Parameters<WorkspaceFolderFileParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Reading a workspace folder file")?;
        let input = params.0;
        let content = self
            .state
            .client
            .read_workspace_folder_file(&input.codelab_id, &input.folder, &input.file)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!({
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
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Listing attendees")?;
        let attendees = self
            .state
            .client
            .get_attendees(&params.0.codelab_id)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!(attendees)))
    }

    #[tool(
        name = "list_help_requests",
        description = "List active help requests for a codelab. Requires an admin session."
    )]
    async fn list_help_requests(
        &self,
        params: Parameters<ScopedCodelabParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Listing help requests")?;
        let help_requests = self
            .state
            .client
            .get_help_requests(&params.0.codelab_id)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!(help_requests)))
    }

    #[tool(
        name = "resolve_help_request",
        description = "Resolve an active help request. Requires an admin session."
    )]
    async fn resolve_help_request(
        &self,
        params: Parameters<ResolveHelpRequestParams>,
    ) -> Result<Json<McpToolPayload>, McpError> {
        self.require_admin_session("Resolving a help request")?;
        self.state
            .client
            .resolve_help_request(&params.0.codelab_id, &params.0.help_id)
            .await
            .map_err(internal_error)?;
        Ok(tool_payload(json!({
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
                "Use the Open Codelabs MCP server to inspect connection state, read full codelab bundles, materials, quizzes, submissions, workspace metadata, reuse prompt templates for facilitator and authoring workflows, and perform focused admin actions when the oc session is authenticated."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
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

    fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<GetPromptResult, McpError>> + Send + '_ {
        async move {
            let prompt_context = rmcp::handler::server::prompt::PromptContext::new(
                self,
                request.name,
                request.arguments,
                context,
            );
            self.prompt_router.get_prompt(prompt_context).await
        }
    }

    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListPromptsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListPromptsResult {
            prompts: self.prompt_router.list_all(),
            next_cursor: None,
            meta: None,
        }))
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

fn tool_payload(value: Value) -> Json<McpToolPayload> {
    Json(McpToolPayload { data: value })
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
    use super::{
        parse_resource_target, FacilitatorBriefPromptInput, McpServerState, OpenCodelabsMcpServer,
        ResourceTarget,
    };
    use crate::cli::client::ApiClient;
    use rmcp::handler::server::wrapper::Parameters;
    use rmcp::model::PromptMessageContent;
    use std::path::PathBuf;

    fn test_server(is_admin: bool) -> OpenCodelabsMcpServer {
        OpenCodelabsMcpServer::new(McpServerState {
            client: ApiClient::new("http://localhost:8080", None).expect("api client"),
            profile_name: Some("test".to_string()),
            base_url: "http://localhost:8080".to_string(),
            session_file: PathBuf::from("/tmp/open-codelabs-test-session.json"),
            session_role: is_admin.then(|| "admin".to_string()),
            session_subject: Some("tester".to_string()),
            runtime_preference: "backend".to_string(),
        })
    }

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

    #[test]
    fn lists_expected_prompt_templates() {
        let server = test_server(false);
        let prompts = server.prompt_router.list_all();
        let prompt_names = prompts
            .iter()
            .map(|prompt| prompt.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            prompt_names,
            vec![
                "authoring-change-plan",
                "facilitator-brief",
                "help-queue-triage",
                "learner-ops-review",
            ]
        );
    }

    #[tokio::test]
    async fn facilitator_prompt_includes_admin_links_when_available() {
        let server = test_server(true);
        let prompt = server
            .facilitator_brief_prompt(Parameters(FacilitatorBriefPromptInput {
                codelab_id: "lab-1".to_string(),
                focus: Some("quizzes".to_string()),
            }))
            .await;

        let linked_uris = prompt
            .messages
            .iter()
            .filter_map(|message| match &message.content {
                PromptMessageContent::ResourceLink { link } => Some(link.uri.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(linked_uris.contains(&"oc://codelabs/lab-1/guide"));
        assert!(linked_uris.contains(&"oc://codelabs/lab-1/steps"));
        assert!(linked_uris.contains(&"oc://codelabs/lab-1/materials"));
        assert!(linked_uris.contains(&"oc://codelabs/lab-1/quizzes"));
        assert!(linked_uris.contains(&"oc://codelabs/lab-1/help"));
    }
}
