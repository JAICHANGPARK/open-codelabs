use crate::api::dto::{
    AiRequest, CliRuntimeCapabilities, CliRuntimeInfo, CodeServerInfo, CreateCodeServerRequest,
    UpdateWorkspaceFilesRequest, WorkspaceFile,
};
use crate::cli::client::{ApiClient, BackupSummary, UpdateCheckSummary, UploadedMaterial};
use crate::cli::config::{
    default_config_path, default_profile_session_path, load_config, save_config, CliConfig,
    ConnectionProfile, RuntimePreference,
};
use crate::cli::session::{
    clear_session, default_session_path, load_session, save_session, SessionSnapshot, StoredSession,
};
use crate::domain::models::{
    AddAiMessagePayload, Codelab, CreateCodelab, CreateInlineCommentPayload, CreateMaterial,
    CreateQuiz, CreateStep, QuizSubmissionPayload, ReplyInlineCommentPayload,
    SaveAiConversationPayload, Step, UpdateStepsPayload,
};
use crate::infrastructure::db_models::AuditLog;
use crate::middleware::auth::now_epoch_seconds;
use crate::utils::crypto::encrypt_with_password;
use anyhow::{anyhow, bail, Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::time::Duration;

#[derive(Debug, Clone)]
struct GlobalOptions {
    base_url: Option<String>,
    session_file: Option<PathBuf>,
    config_file: PathBuf,
    profile: Option<String>,
    json: bool,
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self {
            base_url: env::var("OPEN_CODELABS_BASE_URL").ok(),
            session_file: env::var_os("OPEN_CODELABS_SESSION_FILE").map(PathBuf::from),
            config_file: default_config_path(),
            profile: env::var("OPEN_CODELABS_PROFILE").ok(),
            json: false,
        }
    }
}

#[derive(Debug)]
enum Command {
    Help,
    Admin(AdminCommand),
    Auth(AuthCommand),
    Connect(ConnectCommand),
    Login(LoginCommand),
    Logout,
    Session,
    Codelab(CodelabCommand),
    Backup(BackupCommand),
    Audit(AuditCommand),
    Workspace(WorkspaceCommand),
    Attendee(AttendeeCommand),
    HelpDesk(HelpDeskCommand),
    Feedback(FeedbackCommand),
    Materials(MaterialCommand),
    Quiz(QuizCommand),
    Submission(SubmissionCommand),
    Chat(ChatCommand),
    Upload(UploadCommand),
    Inline(InlineCommand),
    Ai(AiCommand),
}

#[derive(Debug)]
struct LoginCommand {
    admin_id: String,
    admin_pw: String,
}

#[derive(Debug)]
enum AuthCommand {
    Login(AuthLoginCommand),
    Logout,
    Status,
}

#[derive(Debug, Default)]
struct AuthLoginCommand {
    no_open: bool,
}

#[derive(Debug)]
enum ConnectCommand {
    Add {
        name: String,
        url: String,
        runtime: RuntimePreference,
        activate: bool,
    },
    Use {
        name: String,
    },
    List,
    Status,
}

#[derive(Debug)]
enum AdminCommand {
    Settings {
        gemini_api_key: String,
        admin_password: Option<String>,
    },
    Updates,
}

#[derive(Debug)]
enum CodelabCommand {
    List,
    Reference,
    Get {
        id: String,
    },
    Create(CreateCodelabCommand),
    Update {
        id: String,
        command: CreateCodelabCommand,
    },
    Delete {
        id: String,
    },
    Copy {
        id: String,
    },
    Export {
        id: String,
        output: Option<PathBuf>,
    },
    Import {
        file: PathBuf,
    },
    PushSteps {
        id: String,
        file: PathBuf,
    },
}

#[derive(Debug)]
struct CreateCodelabCommand {
    title: String,
    description: String,
    author: String,
    is_public: bool,
    quiz_enabled: bool,
    require_quiz: bool,
    require_feedback: bool,
    require_submission: bool,
    guide_file: Option<PathBuf>,
}

#[derive(Debug)]
enum BackupCommand {
    Export { output: Option<PathBuf> },
    Inspect { file: PathBuf },
    Restore { file: PathBuf },
}

#[derive(Debug)]
enum AuditCommand {
    Logs {
        limit: Option<usize>,
        offset: Option<usize>,
        action: Option<String>,
        codelab_id: Option<String>,
    },
}

#[derive(Debug)]
enum WorkspaceCommand {
    Create {
        codelab_id: String,
        structure_type: Option<String>,
        files_json: Option<PathBuf>,
    },
    Info {
        codelab_id: String,
    },
    Download {
        codelab_id: String,
        output: Option<PathBuf>,
    },
    Delete {
        codelab_id: String,
    },
    Branches {
        codelab_id: String,
    },
    BranchCreate {
        codelab_id: String,
        step_number: i32,
        branch_type: String,
    },
    BranchFiles {
        codelab_id: String,
        branch: String,
    },
    BranchRead {
        codelab_id: String,
        branch: String,
        file: String,
    },
    BranchUpdate {
        codelab_id: String,
        branch: String,
        files_json: PathBuf,
        delete_json: Option<PathBuf>,
        commit_message: Option<String>,
    },
    Folders {
        codelab_id: String,
    },
    FolderCreate {
        codelab_id: String,
        step_number: i32,
        folder_type: String,
        files_json: PathBuf,
    },
    FolderFiles {
        codelab_id: String,
        folder: String,
    },
    FolderRead {
        codelab_id: String,
        folder: String,
        file: String,
    },
    FolderUpdate {
        codelab_id: String,
        folder: String,
        files_json: PathBuf,
        delete_json: Option<PathBuf>,
    },
}

#[derive(Debug)]
enum AttendeeCommand {
    Join {
        codelab_id: String,
        name: String,
        code: String,
        email: Option<String>,
    },
    List {
        codelab_id: String,
    },
    Complete {
        codelab_id: String,
    },
    Certificate {
        attendee_id: Option<String>,
    },
}

#[derive(Debug)]
enum HelpDeskCommand {
    Request {
        codelab_id: String,
        step_number: i32,
    },
    List {
        codelab_id: String,
    },
    Resolve {
        codelab_id: String,
        help_id: String,
    },
}

#[derive(Debug)]
enum FeedbackCommand {
    Submit {
        codelab_id: String,
        difficulty: String,
        satisfaction: String,
        comment: Option<String>,
    },
    List {
        codelab_id: String,
    },
}

#[derive(Debug)]
enum MaterialCommand {
    List {
        codelab_id: String,
    },
    Upload {
        file: PathBuf,
    },
    Add {
        codelab_id: String,
        title: String,
        material_type: String,
        link_url: Option<String>,
        file_path: Option<String>,
    },
    Delete {
        codelab_id: String,
        material_id: String,
    },
}

#[derive(Debug)]
enum QuizCommand {
    List { codelab_id: String },
    Update { codelab_id: String, file: PathBuf },
    Submit { codelab_id: String, file: PathBuf },
    Submissions { codelab_id: String },
}

#[derive(Debug)]
enum SubmissionCommand {
    List {
        codelab_id: String,
    },
    File {
        codelab_id: String,
        attendee_id: Option<String>,
        file: PathBuf,
    },
    Link {
        codelab_id: String,
        attendee_id: Option<String>,
        url: String,
        title: Option<String>,
    },
    Delete {
        codelab_id: String,
        attendee_id: Option<String>,
        submission_id: String,
    },
}

#[derive(Debug)]
enum ChatCommand {
    History { codelab_id: String },
}

#[derive(Debug)]
enum UploadCommand {
    Image { file: PathBuf },
}

#[derive(Debug)]
enum InlineCommand {
    List {
        codelab_id: String,
        target_type: Option<String>,
        target_step_id: Option<String>,
    },
    Create {
        codelab_id: String,
        file: PathBuf,
    },
    Reply {
        codelab_id: String,
        thread_id: String,
        file: PathBuf,
    },
    Delete {
        codelab_id: String,
        thread_id: String,
        comment_id: String,
    },
}

#[derive(Debug)]
enum AiCommand {
    Conversations {
        codelab_id: String,
    },
    Stream {
        file: PathBuf,
    },
    Save {
        file: PathBuf,
    },
    Threads,
    ThreadCreate {
        title: String,
        codelab_id: Option<String>,
    },
    ThreadDelete {
        thread_id: String,
    },
    Messages {
        thread_id: String,
    },
    MessageAdd {
        thread_id: String,
        file: PathBuf,
    },
}

#[derive(Debug, Serialize)]
struct ConnectStatusOutput {
    profile: Option<String>,
    base_url: String,
    session_file: PathBuf,
    runtime_preference: String,
    runtime: String,
    version: Option<String>,
    reachable: bool,
    auth_methods: Vec<String>,
    capabilities: CliRuntimeCapabilities,
    probe_error: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthStatusOutput {
    authenticated: bool,
    profile: Option<String>,
    base_url: String,
    session_file: PathBuf,
    subject: Option<String>,
    role: Option<String>,
    codelab_id: Option<String>,
    expires_at: Option<usize>,
    error: Option<String>,
}

pub async fn entry() -> Result<()> {
    let (global, command) = parse_cli()?;
    if matches!(command, Command::Help) {
        print_help();
        return Ok(());
    }
    run(global, command).await
}

async fn run(global: GlobalOptions, command: Command) -> Result<()> {
    match command {
        Command::Help => {
            print_help();
        }
        Command::Admin(command) => {
            run_admin_command(&global, command).await?;
        }
        Command::Auth(command) => {
            run_auth_command(&global, command).await?;
        }
        Command::Connect(command) => {
            run_connect_command(&global, command).await?;
        }
        Command::Login(command) => {
            let config = load_config(&global.config_file)?;
            let active_profile = resolve_active_profile(&global, &config)?;
            let session_file = resolve_session_file(&global, active_profile.as_ref());
            let base_url = resolve_base_url(
                global.base_url.as_deref(),
                active_profile.as_ref().map(|(_, profile)| profile),
                None,
            );
            let client = ApiClient::new(base_url, None)?;
            let session = client
                .login_admin(&command.admin_id, &command.admin_pw)
                .await?;
            save_session(&session_file, &session)?;

            if global.json {
                print_json(&session)?;
            } else {
                println!("Saved admin session to {}", session_file.display());
                if let Some(sub) = session.sub.as_deref() {
                    println!("subject: {sub}");
                }
                if let Some(exp) = session.exp {
                    println!("expires_at: {exp}");
                }
                println!("base_url: {}", session.base_url);
            }
        }
        Command::Logout => {
            let config = load_config(&global.config_file)?;
            let active_profile = resolve_active_profile(&global, &config)?;
            let session_file = resolve_session_file(&global, active_profile.as_ref());
            let session = load_session(&session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(
                global.base_url.as_deref(),
                active_profile.as_ref().map(|(_, profile)| profile),
                Some(&session),
            );
            let client = ApiClient::new(base_url, Some(session))?;
            let logout_result = client.logout().await;
            clear_session(&session_file)?;
            logout_result?;

            if global.json {
                print_json(&serde_json::json!({ "status": "ok" }))?;
            } else {
                println!("Logged out and removed {}", session_file.display());
            }
        }
        Command::Session => {
            let config = load_config(&global.config_file)?;
            let active_profile = resolve_active_profile(&global, &config)?;
            let session_file = resolve_session_file(&global, active_profile.as_ref());
            let session = load_session(&session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(
                global.base_url.as_deref(),
                active_profile.as_ref().map(|(_, profile)| profile),
                Some(&session),
            );
            let client = ApiClient::new(base_url, Some(session.clone()))?;
            let snapshot = client.session().await?;

            let mut updated = session;
            updated.apply_snapshot(&snapshot);
            save_session(&session_file, &updated)?;

            if global.json {
                print_json(&snapshot)?;
            } else {
                print_session(&snapshot);
            }
        }
        Command::Codelab(command) => {
            let client = load_api_client(&global)?;
            run_codelab_command(&global, &client, command).await?;
        }
        Command::Backup(command) => {
            let config = load_config(&global.config_file)?;
            let active_profile = resolve_active_profile(&global, &config)?;
            let session_file = resolve_session_file(&global, active_profile.as_ref());
            let session = load_session(&session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(
                global.base_url.as_deref(),
                active_profile.as_ref().map(|(_, profile)| profile),
                Some(&session),
            );
            let client = ApiClient::new(base_url, Some(session))?;
            run_backup_command(&global, &client, command).await?;
        }
        Command::Audit(command) => {
            let config = load_config(&global.config_file)?;
            let active_profile = resolve_active_profile(&global, &config)?;
            let session_file = resolve_session_file(&global, active_profile.as_ref());
            let session = load_session(&session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(
                global.base_url.as_deref(),
                active_profile.as_ref().map(|(_, profile)| profile),
                Some(&session),
            );
            let client = ApiClient::new(base_url, Some(session))?;
            run_audit_command(&global, &client, command).await?;
        }
        Command::Workspace(command) => {
            let config = load_config(&global.config_file)?;
            let active_profile = resolve_active_profile(&global, &config)?;
            let session_file = resolve_session_file(&global, active_profile.as_ref());
            let session = load_session(&session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(
                global.base_url.as_deref(),
                active_profile.as_ref().map(|(_, profile)| profile),
                Some(&session),
            );
            let client = ApiClient::new(base_url, Some(session))?;
            run_workspace_command(&global, &client, command).await?;
        }
        Command::Attendee(command) => {
            let (client, session_file, session) = load_api_client_with_session(&global)?;
            run_attendee_command(&global, &client, &session_file, session.as_ref(), command)
                .await?;
        }
        Command::HelpDesk(command) => {
            let client = load_api_client(&global)?;
            run_helpdesk_command(&global, &client, command).await?;
        }
        Command::Feedback(command) => {
            let client = load_api_client(&global)?;
            run_feedback_command(&global, &client, command).await?;
        }
        Command::Materials(command) => {
            let client = load_api_client(&global)?;
            run_material_command(&global, &client, command).await?;
        }
        Command::Quiz(command) => {
            let client = load_api_client(&global)?;
            run_quiz_command(&global, &client, command).await?;
        }
        Command::Submission(command) => {
            let (client, _, session) = load_api_client_with_session(&global)?;
            run_submission_command(&global, &client, session.as_ref(), command).await?;
        }
        Command::Chat(command) => {
            let client = load_api_client(&global)?;
            run_chat_command(&global, &client, command).await?;
        }
        Command::Upload(command) => {
            let client = load_api_client(&global)?;
            run_upload_command(&global, &client, command).await?;
        }
        Command::Inline(command) => {
            let client = load_api_client(&global)?;
            run_inline_command(&global, &client, command).await?;
        }
        Command::Ai(command) => {
            let client = load_api_client(&global)?;
            run_ai_command(&global, &client, command).await?;
        }
    }

    Ok(())
}

async fn run_auth_command(global: &GlobalOptions, command: AuthCommand) -> Result<()> {
    match command {
        AuthCommand::Login(command) => {
            let config = load_config(&global.config_file)?;
            let active_profile = resolve_active_profile(global, &config)?;
            let session_file = resolve_session_file(global, active_profile.as_ref());
            let base_url = resolve_base_url(
                global.base_url.as_deref(),
                active_profile.as_ref().map(|(_, profile)| profile),
                None,
            );
            let client = ApiClient::new(base_url.clone(), None)?;
            let runtime = client.cli_runtime().await?;
            if !runtime.capabilities.browser_auth {
                bail!(
                    "Connected runtime `{}` does not support browser-based CLI authentication",
                    runtime.runtime
                );
            }

            let challenge = client.start_browser_auth().await?;
            let verification_url = format!(
                "{}/{}",
                base_url,
                challenge.verification_path.trim_start_matches('/')
            );

            if !command.no_open {
                if let Err(error) = open_browser(&verification_url) {
                    eprintln!("Failed to open browser automatically: {error}");
                    eprintln!("Open this URL manually: {verification_url}");
                }
            } else if !global.json {
                println!("Open this URL in your browser:");
                println!("{verification_url}");
            }

            let deadline = challenge.expires_at_epoch;
            loop {
                let poll = client
                    .poll_browser_auth(&challenge.request_id, &challenge.poll_token)
                    .await?;

                match poll.status.as_str() {
                    "approved" => {
                        let session = client
                            .exchange_browser_auth(&challenge.request_id, &challenge.poll_token)
                            .await?;
                        save_session(&session_file, &session)?;

                        if global.json {
                            print_json(&session)?;
                        } else {
                            println!(
                                "Authenticated and saved session to {}",
                                session_file.display()
                            );
                            println!("base_url: {}", session.base_url);
                            if let Some(sub) = session.sub.as_deref() {
                                println!("subject: {sub}");
                            }
                            if let Some(exp) = session.exp {
                                println!("expires_at: {exp}");
                            }
                        }
                        return Ok(());
                    }
                    "pending" => {
                        let now = now_epoch_seconds() as i64;
                        if now >= deadline {
                            bail!("CLI auth request expired before approval");
                        }
                        tokio::time::sleep(Duration::from_secs(
                            challenge.poll_interval_seconds.max(1),
                        ))
                        .await;
                    }
                    "expired" => bail!("CLI auth request expired"),
                    "consumed" => bail!("CLI auth request was already exchanged"),
                    other => bail!("Unexpected CLI auth state: {other}"),
                }
            }
        }
        AuthCommand::Logout => {
            let config = load_config(&global.config_file)?;
            let active_profile = resolve_active_profile(global, &config)?;
            let session_file = resolve_session_file(global, active_profile.as_ref());
            if !session_file.exists() {
                if global.json {
                    print_json(&serde_json::json!({ "status": "ok", "authenticated": false }))?;
                } else {
                    println!("No saved session at {}", session_file.display());
                }
                return Ok(());
            }

            let session = load_session(&session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` auth login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(
                global.base_url.as_deref(),
                active_profile.as_ref().map(|(_, profile)| profile),
                Some(&session),
            );
            let client = ApiClient::new(base_url, Some(session))?;
            let logout_result = client.logout().await;
            clear_session(&session_file)?;
            logout_result?;

            if global.json {
                print_json(&serde_json::json!({ "status": "ok" }))?;
            } else {
                println!("Logged out and removed {}", session_file.display());
            }
        }
        AuthCommand::Status => {
            let status = build_auth_status(global).await?;
            if global.json {
                print_json(&status)?;
            } else {
                print_auth_status(&status);
            }
        }
    }

    Ok(())
}

async fn run_admin_command(global: &GlobalOptions, command: AdminCommand) -> Result<()> {
    let config = load_config(&global.config_file)?;
    let active_profile = resolve_active_profile(global, &config)?;
    let session_file = resolve_session_file(global, active_profile.as_ref());
    let session = load_session(&session_file).with_context(|| {
        format!(
            "No saved session found. Run `{}` auth login first.",
            program_name()
        )
    })?;
    let base_url = resolve_base_url(
        global.base_url.as_deref(),
        active_profile.as_ref().map(|(_, profile)| profile),
        Some(&session),
    );
    let client = ApiClient::new(base_url, Some(session))?;

    match command {
        AdminCommand::Settings {
            gemini_api_key,
            admin_password,
        } => {
            let encrypted = if gemini_api_key.is_empty() {
                String::new()
            } else {
                let password = admin_password
                    .or_else(|| env::var("OPEN_CODELABS_ADMIN_PW").ok())
                    .ok_or_else(|| {
                        anyhow!(
                            "Missing admin password. Use --admin-password or OPEN_CODELABS_ADMIN_PW to encrypt the key."
                        )
                    })?;
                encrypt_with_password(&gemini_api_key, &password)
                    .map_err(|error| anyhow!("Failed to encrypt Gemini API key: {error}"))?
            };
            client.save_admin_settings(&encrypted).await?;

            if global.json {
                print_json(&serde_json::json!({ "status": "ok" }))?;
            } else if gemini_api_key.is_empty() {
                println!("Cleared Gemini API key");
            } else {
                println!("Updated Gemini API key");
            }
        }
        AdminCommand::Updates => {
            let updates = client.check_updates().await?;
            if global.json {
                print_json(&updates)?;
            } else {
                print_updates_summary(&updates);
            }
        }
    }

    Ok(())
}

async fn run_codelab_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: CodelabCommand,
) -> Result<()> {
    match command {
        CodelabCommand::List => {
            let codelabs = client.list_codelabs().await?;
            if global.json {
                print_json(&codelabs)?;
            } else {
                print_codelab_list(&codelabs);
            }
        }
        CodelabCommand::Reference => {
            let reference = client.reference_codelabs().await?;
            if global.json {
                print_json(&serde_json::json!({ "content": reference }))?;
            } else {
                println!("{reference}");
            }
        }
        CodelabCommand::Get { id } => {
            let (codelab, steps) = client.get_codelab(&id).await?;
            if global.json {
                print_json(&serde_json::json!({
                    "codelab": codelab,
                    "steps": steps,
                }))?;
            } else {
                print_codelab_detail(&codelab, &steps);
            }
        }
        CodelabCommand::Create(command) => {
            let payload = build_codelab_payload(command).await?;
            let codelab = client.create_codelab(&payload).await?;

            if global.json {
                print_json(&codelab)?;
            } else {
                println!("Created codelab {}", codelab.id);
                println!("title: {}", codelab.title);
                println!("author: {}", codelab.author);
            }
        }
        CodelabCommand::Update { id, command } => {
            let payload = build_codelab_payload(command).await?;
            let codelab = client.update_codelab(&id, &payload).await?;
            if global.json {
                print_json(&codelab)?;
            } else {
                println!("Updated codelab {}", codelab.id);
                println!("title: {}", codelab.title);
                println!("author: {}", codelab.author);
            }
        }
        CodelabCommand::Delete { id } => {
            client.delete_codelab(&id).await?;
            if global.json {
                print_json(&serde_json::json!({ "status": "ok", "codelab_id": id }))?;
            } else {
                println!("Deleted codelab {id}");
            }
        }
        CodelabCommand::Copy { id } => {
            let codelab = client.copy_codelab(&id).await?;
            if global.json {
                print_json(&codelab)?;
            } else {
                println!("Copied codelab {} to {}", id, codelab.id);
                println!("title: {}", codelab.title);
            }
        }
        CodelabCommand::Export { id, output } => {
            let archive = client.export_codelab(&id).await?;
            let output = output.unwrap_or_else(|| PathBuf::from(format!("codelab_{id}.zip")));
            tokio::fs::write(&output, archive)
                .await
                .with_context(|| format!("Failed to write {}", output.display()))?;

            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "output": output,
                }))?;
            } else {
                println!("Exported codelab {id} to {}", output.display());
            }
        }
        CodelabCommand::Import { file } => {
            let codelab = client.import_codelab(&file).await?;
            if global.json {
                print_json(&codelab)?;
            } else {
                println!("Imported codelab {}", codelab.id);
                println!("title: {}", codelab.title);
            }
        }
        CodelabCommand::PushSteps { id, file } => {
            let payload = load_steps_payload(&file).await?;
            client.push_steps(&id, &payload).await?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "codelab_id": id,
                    "steps": payload.steps.len(),
                }))?;
            } else {
                println!("Updated {} steps for codelab {id}", payload.steps.len());
            }
        }
    }
    Ok(())
}

async fn build_codelab_payload(command: CreateCodelabCommand) -> Result<CreateCodelab> {
    let guide_markdown = match command.guide_file {
        Some(path) => Some(
            tokio::fs::read_to_string(&path)
                .await
                .with_context(|| format!("Failed to read {}", path.display()))?,
        ),
        None => None,
    };

    Ok(CreateCodelab {
        title: command.title,
        description: command.description,
        author: command.author,
        is_public: Some(command.is_public),
        quiz_enabled: Some(command.quiz_enabled),
        require_quiz: Some(command.require_quiz),
        require_feedback: Some(command.require_feedback),
        require_submission: Some(command.require_submission),
        guide_markdown,
    })
}

async fn run_backup_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: BackupCommand,
) -> Result<()> {
    match command {
        BackupCommand::Export { output } => {
            let archive = client.export_backup().await?;
            let output = output.unwrap_or_else(|| PathBuf::from("backup_full.zip"));
            tokio::fs::write(&output, archive)
                .await
                .with_context(|| format!("Failed to write {}", output.display()))?;

            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "output": output,
                }))?;
            } else {
                println!("Exported backup to {}", output.display());
            }
        }
        BackupCommand::Inspect { file } => {
            let summary = client.inspect_backup(&file).await?;
            if global.json {
                print_json(&summary)?;
            } else {
                print_backup_summary(&summary);
            }
        }
        BackupCommand::Restore { file } => {
            client.restore_backup(&file).await?;
            if global.json {
                print_json(&serde_json::json!({ "status": "ok" }))?;
            } else {
                println!("Restored backup from {}", file.display());
            }
        }
    }
    Ok(())
}

async fn run_audit_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: AuditCommand,
) -> Result<()> {
    match command {
        AuditCommand::Logs {
            limit,
            offset,
            action,
            codelab_id,
        } => {
            let logs = client
                .audit_logs(limit, offset, action.as_deref(), codelab_id.as_deref())
                .await?;
            if global.json {
                print_json(&logs)?;
            } else {
                print_audit_logs(&logs);
            }
        }
    }
    Ok(())
}

async fn run_workspace_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: WorkspaceCommand,
) -> Result<()> {
    match command {
        WorkspaceCommand::Create {
            codelab_id,
            structure_type,
            files_json,
        } => {
            let workspace_files = match files_json {
                Some(path) => Some(load_workspace_files(&path).await?),
                None => None,
            };
            let info = client
                .create_workspace(&CreateCodeServerRequest {
                    codelab_id,
                    workspace_files,
                    structure_type,
                })
                .await?;
            if global.json {
                print_json(&info)?;
            } else {
                print_workspace_info(&info);
            }
        }
        WorkspaceCommand::Info { codelab_id } => {
            let info = client.workspace_info(&codelab_id).await?;
            if global.json {
                print_json(&info)?;
            } else {
                print_workspace_info(&info);
            }
        }
        WorkspaceCommand::Download { codelab_id, output } => {
            let archive = client.download_workspace(&codelab_id).await?;
            let output =
                output.unwrap_or_else(|| PathBuf::from(format!("workspace_{codelab_id}.tar.gz")));
            tokio::fs::write(&output, archive)
                .await
                .with_context(|| format!("Failed to write {}", output.display()))?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "output": output,
                }))?;
            } else {
                println!(
                    "Downloaded workspace for {codelab_id} to {}",
                    output.display()
                );
            }
        }
        WorkspaceCommand::Delete { codelab_id } => {
            client.delete_workspace(&codelab_id).await?;
            if global.json {
                print_json(&serde_json::json!({ "status": "ok" }))?;
            } else {
                println!("Deleted workspace for {codelab_id}");
            }
        }
        WorkspaceCommand::Branches { codelab_id } => {
            let branches = client.list_workspace_branches(&codelab_id).await?;
            if global.json {
                print_json(&branches)?;
            } else {
                println!("Branches for {codelab_id}:");
                for branch in branches {
                    println!("- {branch}");
                }
            }
        }
        WorkspaceCommand::BranchCreate {
            codelab_id,
            step_number,
            branch_type,
        } => {
            client
                .create_workspace_branch(&codelab_id, step_number, &branch_type)
                .await?;
            if global.json {
                print_json(&serde_json::json!({ "status": "ok" }))?;
            } else {
                println!(
                    "Created branch snapshot for codelab {} step {} ({})",
                    codelab_id, step_number, branch_type
                );
            }
        }
        WorkspaceCommand::BranchFiles { codelab_id, branch } => {
            let files = client.list_workspace_files(&codelab_id, &branch).await?;
            if global.json {
                print_json(&files)?;
            } else {
                println!("Branch files for {codelab_id} ({branch}):");
                for file in files {
                    println!("- {file}");
                }
            }
        }
        WorkspaceCommand::BranchRead {
            codelab_id,
            branch,
            file,
        } => {
            let content = client
                .read_workspace_file(&codelab_id, &branch, &file)
                .await?;
            if global.json {
                print_json(&serde_json::json!({
                    "codelab_id": codelab_id,
                    "branch": branch,
                    "file": file,
                    "content": content,
                }))?;
            } else {
                println!("{content}");
            }
        }
        WorkspaceCommand::BranchUpdate {
            codelab_id,
            branch,
            files_json,
            delete_json,
            commit_message,
        } => {
            let payload =
                load_workspace_update_request(&files_json, delete_json.as_deref(), commit_message)
                    .await?;
            client
                .update_workspace_branch_files(&codelab_id, &branch, &payload)
                .await?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "codelab_id": codelab_id,
                    "branch": branch,
                    "files_updated": payload.files.len(),
                    "files_deleted": payload.delete_files.as_ref().map(|items| items.len()).unwrap_or(0),
                }))?;
            } else {
                println!(
                    "Updated branch {} for {} ({} writes, {} deletes)",
                    branch,
                    codelab_id,
                    payload.files.len(),
                    payload
                        .delete_files
                        .as_ref()
                        .map(|items| items.len())
                        .unwrap_or(0)
                );
            }
        }
        WorkspaceCommand::Folders { codelab_id } => {
            let folders = client.list_workspace_folders(&codelab_id).await?;
            if global.json {
                print_json(&folders)?;
            } else {
                println!("Folders for {codelab_id}:");
                for folder in folders {
                    println!("- {folder}");
                }
            }
        }
        WorkspaceCommand::FolderCreate {
            codelab_id,
            step_number,
            folder_type,
            files_json,
        } => {
            let files = load_workspace_files(&files_json).await?;
            client
                .create_workspace_folder(&codelab_id, step_number, &folder_type, files)
                .await?;
            if global.json {
                print_json(&serde_json::json!({ "status": "ok" }))?;
            } else {
                println!(
                    "Created folder snapshot for codelab {} step {} ({})",
                    codelab_id, step_number, folder_type
                );
            }
        }
        WorkspaceCommand::FolderFiles { codelab_id, folder } => {
            let files = client
                .list_workspace_folder_files(&codelab_id, &folder)
                .await?;
            if global.json {
                print_json(&files)?;
            } else {
                println!("Folder files for {codelab_id} ({folder}):");
                for file in files {
                    println!("- {file}");
                }
            }
        }
        WorkspaceCommand::FolderRead {
            codelab_id,
            folder,
            file,
        } => {
            let content = client
                .read_workspace_folder_file(&codelab_id, &folder, &file)
                .await?;
            if global.json {
                print_json(&serde_json::json!({
                    "codelab_id": codelab_id,
                    "folder": folder,
                    "file": file,
                    "content": content,
                }))?;
            } else {
                println!("{content}");
            }
        }
        WorkspaceCommand::FolderUpdate {
            codelab_id,
            folder,
            files_json,
            delete_json,
        } => {
            let payload =
                load_workspace_update_request(&files_json, delete_json.as_deref(), None).await?;
            client
                .update_workspace_folder_files(&codelab_id, &folder, &payload)
                .await?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "codelab_id": codelab_id,
                    "folder": folder,
                    "files_updated": payload.files.len(),
                    "files_deleted": payload.delete_files.as_ref().map(|items| items.len()).unwrap_or(0),
                }))?;
            } else {
                println!(
                    "Updated folder {} for {} ({} writes, {} deletes)",
                    folder,
                    codelab_id,
                    payload.files.len(),
                    payload
                        .delete_files
                        .as_ref()
                        .map(|items| items.len())
                        .unwrap_or(0)
                );
            }
        }
    }
    Ok(())
}

async fn run_attendee_command(
    global: &GlobalOptions,
    client: &ApiClient,
    session_file: &Path,
    session: Option<&StoredSession>,
    command: AttendeeCommand,
) -> Result<()> {
    match command {
        AttendeeCommand::Join {
            codelab_id,
            name,
            code,
            email,
        } => {
            let (stored_session, attendee) = client
                .register_attendee(&codelab_id, &name, &code, email.as_deref())
                .await?;
            save_session(session_file, &stored_session)?;
            if global.json {
                print_json(&serde_json::json!({
                    "session_file": session_file,
                    "attendee": attendee,
                }))?;
            } else {
                println!("Joined codelab {}", attendee.codelab_id);
                println!("attendee_id: {}", attendee.id);
                println!("name: {}", attendee.name);
                println!("session_file: {}", session_file.display());
            }
        }
        AttendeeCommand::List { codelab_id } => {
            let attendees = client.get_attendees(&codelab_id).await?;
            if global.json {
                print_json(&attendees)?;
            } else {
                print_json(&attendees)?;
            }
        }
        AttendeeCommand::Complete { codelab_id } => {
            client.complete_codelab(&codelab_id).await?;
            if global.json {
                print_json(&serde_json::json!({ "status": "ok", "codelab_id": codelab_id }))?;
            } else {
                println!("Completed codelab {codelab_id}");
            }
        }
        AttendeeCommand::Certificate { attendee_id } => {
            let attendee_id = resolve_attendee_id(attendee_id, session, "attendee certificate")?;
            let certificate = client.get_certificate(&attendee_id).await?;
            if global.json {
                print_json(&certificate)?;
            } else {
                print_json(&certificate)?;
            }
        }
    }

    Ok(())
}

async fn run_helpdesk_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: HelpDeskCommand,
) -> Result<()> {
    match command {
        HelpDeskCommand::Request {
            codelab_id,
            step_number,
        } => {
            client.request_help(&codelab_id, step_number).await?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "codelab_id": codelab_id,
                    "step_number": step_number,
                }))?;
            } else {
                println!("Requested help for {codelab_id} step {step_number}");
            }
        }
        HelpDeskCommand::List { codelab_id } => {
            let requests = client.get_help_requests(&codelab_id).await?;
            if global.json {
                print_json(&requests)?;
            } else {
                print_json(&requests)?;
            }
        }
        HelpDeskCommand::Resolve {
            codelab_id,
            help_id,
        } => {
            client.resolve_help_request(&codelab_id, &help_id).await?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "codelab_id": codelab_id,
                    "help_id": help_id,
                }))?;
            } else {
                println!("Resolved help request {help_id}");
            }
        }
    }

    Ok(())
}

async fn run_feedback_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: FeedbackCommand,
) -> Result<()> {
    match command {
        FeedbackCommand::Submit {
            codelab_id,
            difficulty,
            satisfaction,
            comment,
        } => {
            client
                .submit_feedback(&codelab_id, &difficulty, &satisfaction, comment.as_deref())
                .await?;
            if global.json {
                print_json(&serde_json::json!({ "status": "ok", "codelab_id": codelab_id }))?;
            } else {
                println!("Submitted feedback for {codelab_id}");
            }
        }
        FeedbackCommand::List { codelab_id } => {
            let feedback = client.get_feedback(&codelab_id).await?;
            if global.json {
                print_json(&feedback)?;
            } else {
                print_json(&feedback)?;
            }
        }
    }

    Ok(())
}

async fn run_material_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: MaterialCommand,
) -> Result<()> {
    match command {
        MaterialCommand::List { codelab_id } => {
            let materials = client.get_materials(&codelab_id).await?;
            if global.json {
                print_json(&materials)?;
            } else {
                print_json(&materials)?;
            }
        }
        MaterialCommand::Upload { file } => {
            let uploaded = client.upload_material(&file).await?;
            print_uploaded_material(global, &uploaded)?;
        }
        MaterialCommand::Add {
            codelab_id,
            title,
            material_type,
            link_url,
            file_path,
        } => {
            let material = client
                .add_material(
                    &codelab_id,
                    &CreateMaterial {
                        title,
                        material_type,
                        link_url,
                        file_path,
                    },
                )
                .await?;
            if global.json {
                print_json(&material)?;
            } else {
                println!("Added material {}", material.id);
                println!("title: {}", material.title);
            }
        }
        MaterialCommand::Delete {
            codelab_id,
            material_id,
        } => {
            client.delete_material(&codelab_id, &material_id).await?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "material_id": material_id,
                }))?;
            } else {
                println!("Deleted material {material_id}");
            }
        }
    }

    Ok(())
}

async fn run_quiz_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: QuizCommand,
) -> Result<()> {
    match command {
        QuizCommand::List { codelab_id } => {
            let quizzes = client.get_quizzes(&codelab_id).await?;
            if global.json {
                print_json(&quizzes)?;
            } else {
                print_json(&quizzes)?;
            }
        }
        QuizCommand::Update { codelab_id, file } => {
            let quizzes = load_json_file::<Vec<CreateQuiz>>(&file).await?;
            client.update_quizzes(&codelab_id, &quizzes).await?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "codelab_id": codelab_id,
                    "quizzes": quizzes.len(),
                }))?;
            } else {
                println!("Updated {} quizzes for {}", quizzes.len(), codelab_id);
            }
        }
        QuizCommand::Submit { codelab_id, file } => {
            let payload = load_quiz_submission_payload(&file).await?;
            client.submit_quiz(&codelab_id, &payload).await?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "codelab_id": codelab_id,
                    "submissions": payload.submissions.len(),
                }))?;
            } else {
                println!(
                    "Submitted {} quiz answers for {}",
                    payload.submissions.len(),
                    codelab_id
                );
            }
        }
        QuizCommand::Submissions { codelab_id } => {
            let submissions = client.get_quiz_submissions(&codelab_id).await?;
            if global.json {
                print_json(&submissions)?;
            } else {
                print_json(&submissions)?;
            }
        }
    }

    Ok(())
}

async fn run_submission_command(
    global: &GlobalOptions,
    client: &ApiClient,
    session: Option<&StoredSession>,
    command: SubmissionCommand,
) -> Result<()> {
    match command {
        SubmissionCommand::List { codelab_id } => {
            let submissions = client.get_submissions(&codelab_id).await?;
            if global.json {
                print_json(&submissions)?;
            } else {
                print_json(&submissions)?;
            }
        }
        SubmissionCommand::File {
            codelab_id,
            attendee_id,
            file,
        } => {
            let attendee_id = resolve_attendee_id(attendee_id, session, "submission file")?;
            let submission = client
                .submit_submission_file(&codelab_id, &attendee_id, &file)
                .await?;
            if global.json {
                print_json(&submission)?;
            } else {
                println!("Uploaded submission {}", submission.id);
                println!("file_name: {}", submission.file_name);
            }
        }
        SubmissionCommand::Link {
            codelab_id,
            attendee_id,
            url,
            title,
        } => {
            let attendee_id = resolve_attendee_id(attendee_id, session, "submission link")?;
            let submission = client
                .submit_submission_link(&codelab_id, &attendee_id, &url, title.as_deref())
                .await?;
            if global.json {
                print_json(&submission)?;
            } else {
                println!("Created link submission {}", submission.id);
                println!(
                    "url: {}",
                    submission
                        .link_url
                        .as_deref()
                        .unwrap_or(&submission.file_path)
                );
            }
        }
        SubmissionCommand::Delete {
            codelab_id,
            attendee_id,
            submission_id,
        } => {
            let attendee_id = resolve_attendee_id(attendee_id, session, "submission delete")?;
            client
                .delete_submission(&codelab_id, &attendee_id, &submission_id)
                .await?;
            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "submission_id": submission_id,
                }))?;
            } else {
                println!("Deleted submission {submission_id}");
            }
        }
    }

    Ok(())
}

async fn run_chat_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: ChatCommand,
) -> Result<()> {
    match command {
        ChatCommand::History { codelab_id } => {
            let history = client.get_chat_history(&codelab_id).await?;
            if global.json {
                print_json(&history)?;
            } else {
                print_json(&history)?;
            }
        }
    }

    Ok(())
}

async fn run_upload_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: UploadCommand,
) -> Result<()> {
    match command {
        UploadCommand::Image { file } => {
            let uploaded = client.upload_image(&file).await?;
            if global.json {
                print_json(&uploaded)?;
            } else {
                println!("Uploaded image {}", uploaded.url);
            }
        }
    }

    Ok(())
}

async fn run_inline_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: InlineCommand,
) -> Result<()> {
    match command {
        InlineCommand::List {
            codelab_id,
            target_type,
            target_step_id,
        } => {
            let threads = client
                .get_inline_comments(
                    &codelab_id,
                    target_type.as_deref(),
                    target_step_id.as_deref(),
                )
                .await?;
            if global.json {
                print_json(&threads)?;
            } else {
                print_json(&threads)?;
            }
        }
        InlineCommand::Create { codelab_id, file } => {
            let payload = load_json_file::<CreateInlineCommentPayload>(&file).await?;
            let thread = client.create_inline_comment(&codelab_id, &payload).await?;
            if global.json {
                print_json(&thread)?;
            } else {
                print_json(&thread)?;
            }
        }
        InlineCommand::Reply {
            codelab_id,
            thread_id,
            file,
        } => {
            let payload = load_json_file::<ReplyInlineCommentPayload>(&file).await?;
            let thread = client
                .reply_inline_comment(&codelab_id, &thread_id, &payload)
                .await?;
            if global.json {
                print_json(&thread)?;
            } else {
                print_json(&thread)?;
            }
        }
        InlineCommand::Delete {
            codelab_id,
            thread_id,
            comment_id,
        } => {
            let result = client
                .delete_inline_comment(&codelab_id, &thread_id, &comment_id)
                .await?;
            if global.json {
                print_json(&result)?;
            } else {
                print_json(&result)?;
            }
        }
    }

    Ok(())
}

async fn run_ai_command(
    global: &GlobalOptions,
    client: &ApiClient,
    command: AiCommand,
) -> Result<()> {
    match command {
        AiCommand::Conversations { codelab_id } => {
            let conversations = client.get_ai_conversations(&codelab_id).await?;
            if global.json {
                print_json(&conversations)?;
            } else {
                print_json(&conversations)?;
            }
        }
        AiCommand::Stream { file } => {
            let payload = load_json_file::<AiRequest>(&file).await?;
            let stream = client.stream_ai(&payload).await?;
            if global.json {
                print_json(&serde_json::json!({ "stream": stream }))?;
            } else {
                println!("{stream}");
            }
        }
        AiCommand::Save { file } => {
            let payload = load_json_file::<SaveAiConversationPayload>(&file).await?;
            let result = client.save_ai_conversation(&payload).await?;
            if global.json {
                print_json(&result)?;
            } else {
                print_json(&result)?;
            }
        }
        AiCommand::Threads => {
            let threads = client.get_ai_threads().await?;
            if global.json {
                print_json(&threads)?;
            } else {
                print_json(&threads)?;
            }
        }
        AiCommand::ThreadCreate { title, codelab_id } => {
            let thread = client
                .create_ai_thread(&title, codelab_id.as_deref())
                .await?;
            if global.json {
                print_json(&thread)?;
            } else {
                println!("Created AI thread {}", thread.id);
                println!("title: {}", thread.title);
            }
        }
        AiCommand::ThreadDelete { thread_id } => {
            client.delete_ai_thread(&thread_id).await?;
            if global.json {
                print_json(&serde_json::json!({ "status": "ok", "thread_id": thread_id }))?;
            } else {
                println!("Deleted AI thread {thread_id}");
            }
        }
        AiCommand::Messages { thread_id } => {
            let messages = client.get_ai_messages(&thread_id).await?;
            if global.json {
                print_json(&messages)?;
            } else {
                print_json(&messages)?;
            }
        }
        AiCommand::MessageAdd { thread_id, file } => {
            let payload = load_json_file::<AddAiMessagePayload>(&file).await?;
            let message = client.add_ai_message(&thread_id, &payload).await?;
            if global.json {
                print_json(&message)?;
            } else {
                println!("Added AI message {}", message.id);
                println!("role: {}", message.role);
            }
        }
    }

    Ok(())
}

fn print_uploaded_material(global: &GlobalOptions, uploaded: &UploadedMaterial) -> Result<()> {
    if global.json {
        print_json(uploaded)
    } else {
        println!("url: {}", uploaded.url);
        println!("original_name: {}", uploaded.original_name);
        Ok(())
    }
}

async fn run_connect_command(global: &GlobalOptions, command: ConnectCommand) -> Result<()> {
    match command {
        ConnectCommand::Add {
            name,
            url,
            runtime,
            activate,
        } => {
            let mut config = load_config(&global.config_file)?;
            config.profiles.insert(
                name.clone(),
                ConnectionProfile {
                    base_url: url.trim_end_matches('/').to_string(),
                    runtime,
                },
            );
            let became_current = activate || config.current_profile.is_none();
            if became_current {
                config.current_profile = Some(name.clone());
            }
            save_config(&global.config_file, &config)?;

            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "profile": name,
                    "current": config.current_profile,
                    "config_file": global.config_file,
                }))?;
            } else {
                println!("Saved profile `{name}` to {}", global.config_file.display());
                if became_current {
                    println!("current_profile: {name}");
                }
            }
        }
        ConnectCommand::Use { name } => {
            let mut config = load_config(&global.config_file)?;
            if !config.profiles.contains_key(&name) {
                bail!("Unknown profile: {name}");
            }
            config.current_profile = Some(name.clone());
            save_config(&global.config_file, &config)?;

            if global.json {
                print_json(&serde_json::json!({
                    "status": "ok",
                    "profile": name,
                    "config_file": global.config_file,
                }))?;
            } else {
                println!("Current profile: {name}");
            }
        }
        ConnectCommand::List => {
            let config = load_config(&global.config_file)?;
            if global.json {
                let profiles = config
                    .profiles
                    .iter()
                    .map(|(name, profile)| {
                        serde_json::json!({
                            "name": name,
                            "base_url": profile.base_url,
                            "runtime": profile.runtime.as_str(),
                            "current": config.current_profile.as_deref() == Some(name.as_str()),
                        })
                    })
                    .collect::<Vec<_>>();
                print_json(&profiles)?;
            } else {
                print_connect_profiles(&config);
            }
        }
        ConnectCommand::Status => {
            let status = build_connect_status(global).await?;
            if global.json {
                print_json(&status)?;
            } else {
                print_connect_status(&status);
            }
        }
    }

    Ok(())
}

fn load_api_client(global: &GlobalOptions) -> Result<ApiClient> {
    let (client, _, _) = load_api_client_with_session(global)?;
    Ok(client)
}

fn load_api_client_with_session(
    global: &GlobalOptions,
) -> Result<(ApiClient, PathBuf, Option<StoredSession>)> {
    let config = load_config(&global.config_file)?;
    let active_profile = resolve_active_profile(global, &config)?;
    let session_file = resolve_session_file(global, active_profile.as_ref());
    let session =
        if session_file.exists() {
            Some(load_session(&session_file).with_context(|| {
                format!("Failed to load saved session {}", session_file.display())
            })?)
        } else {
            None
        };
    let base_url = resolve_base_url(
        global.base_url.as_deref(),
        active_profile.as_ref().map(|(_, profile)| profile),
        session.as_ref(),
    );
    let client = ApiClient::new(base_url, session.clone())?;
    Ok((client, session_file, session))
}

async fn build_auth_status(global: &GlobalOptions) -> Result<AuthStatusOutput> {
    let config = load_config(&global.config_file)?;
    let active_profile = resolve_active_profile(global, &config)?;
    let session_file = resolve_session_file(global, active_profile.as_ref());
    let base_url = resolve_base_url(
        global.base_url.as_deref(),
        active_profile.as_ref().map(|(_, profile)| profile),
        None,
    );

    if !session_file.exists() {
        return Ok(AuthStatusOutput {
            authenticated: false,
            profile: active_profile.map(|(name, _)| name),
            base_url,
            session_file,
            subject: None,
            role: None,
            codelab_id: None,
            expires_at: None,
            error: None,
        });
    }

    let session = load_session(&session_file)?;
    let base_url = resolve_base_url(
        global.base_url.as_deref(),
        active_profile.as_ref().map(|(_, profile)| profile),
        Some(&session),
    );
    let client = ApiClient::new(base_url.clone(), Some(session.clone()))?;
    match client.session().await {
        Ok(snapshot) => {
            let mut updated = session;
            updated.apply_snapshot(&snapshot);
            save_session(&session_file, &updated)?;
            Ok(AuthStatusOutput {
                authenticated: true,
                profile: active_profile.map(|(name, _)| name),
                base_url,
                session_file,
                subject: Some(snapshot.sub),
                role: Some(snapshot.role),
                codelab_id: snapshot.codelab_id,
                expires_at: Some(snapshot.exp),
                error: None,
            })
        }
        Err(error) => Ok(AuthStatusOutput {
            authenticated: false,
            profile: active_profile.map(|(name, _)| name),
            base_url,
            session_file,
            subject: session.sub,
            role: session.role,
            codelab_id: session.codelab_id,
            expires_at: session.exp,
            error: Some(error.to_string()),
        }),
    }
}

async fn build_connect_status(global: &GlobalOptions) -> Result<ConnectStatusOutput> {
    let config = load_config(&global.config_file)?;
    let active_profile = resolve_active_profile(global, &config)?;
    let session_file = resolve_session_file(global, active_profile.as_ref());
    let base_url = resolve_base_url(
        global.base_url.as_deref(),
        active_profile.as_ref().map(|(_, profile)| profile),
        None,
    );
    let runtime_preference = active_profile
        .as_ref()
        .map(|(_, profile)| profile.runtime)
        .unwrap_or(RuntimePreference::Auto);

    let client = ApiClient::new(base_url.clone(), None)?;
    let probe = if matches!(
        runtime_preference,
        RuntimePreference::Auto | RuntimePreference::Backend
    ) {
        match client.cli_runtime().await {
            Ok(info) => (Some(info), true, None),
            Err(error) => (
                static_runtime_info(runtime_preference),
                false,
                Some(error.to_string()),
            ),
        }
    } else {
        (
            static_runtime_info(runtime_preference),
            false,
            Some("This runtime does not expose the backend CLI probe endpoint.".to_string()),
        )
    };

    let (runtime_info, reachable, probe_error) = probe;
    Ok(ConnectStatusOutput {
        profile: active_profile.map(|(name, _)| name),
        base_url,
        session_file,
        runtime_preference: runtime_preference.as_str().to_string(),
        runtime: runtime_info
            .as_ref()
            .map(|info| info.runtime.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        version: runtime_info.as_ref().map(|info| info.version.clone()),
        reachable,
        auth_methods: runtime_info
            .as_ref()
            .map(|info| info.auth_methods.clone())
            .unwrap_or_default(),
        capabilities: runtime_info
            .map(|info| info.capabilities)
            .unwrap_or_else(empty_capabilities),
        probe_error,
    })
}

fn open_browser(url: &str) -> Result<()> {
    let status = if cfg!(target_os = "macos") {
        ProcessCommand::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        ProcessCommand::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .status()
    } else {
        ProcessCommand::new("xdg-open").arg(url).status()
    }
    .with_context(|| format!("Failed to launch browser for {url}"))?;

    if !status.success() {
        bail!("Browser launcher exited with status {status}");
    }
    Ok(())
}

async fn load_steps_payload(path: &Path) -> Result<UpdateStepsPayload> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;

    if value.get("steps").is_some() {
        return serde_json::from_value(value)
            .with_context(|| format!("Invalid steps payload in {}", path.display()));
    }

    let steps: Vec<CreateStep> = serde_json::from_value(value)
        .with_context(|| format!("Invalid steps array in {}", path.display()))?;
    Ok(UpdateStepsPayload { steps })
}

async fn load_json_file<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))
}

async fn load_quiz_submission_payload(path: &Path) -> Result<QuizSubmissionPayload> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;

    if value.get("submissions").is_some() {
        return serde_json::from_value(value)
            .with_context(|| format!("Invalid quiz submission payload in {}", path.display()));
    }

    let submissions = serde_json::from_value(value)
        .with_context(|| format!("Invalid quiz submission array in {}", path.display()))?;
    Ok(QuizSubmissionPayload { submissions })
}

async fn load_workspace_files(path: &Path) -> Result<Vec<WorkspaceFile>> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;

    if let Some(files) = value.get("files") {
        return serde_json::from_value(files.clone())
            .with_context(|| format!("Invalid workspace file list in {}", path.display()));
    }

    serde_json::from_value(value)
        .with_context(|| format!("Invalid workspace file array in {}", path.display()))
}

async fn load_workspace_update_request(
    files_path: &Path,
    delete_path: Option<&Path>,
    commit_message: Option<String>,
) -> Result<UpdateWorkspaceFilesRequest> {
    let raw = tokio::fs::read_to_string(files_path)
        .await
        .with_context(|| format!("Failed to read {}", files_path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse JSON from {}", files_path.display()))?;

    let mut payload = if value.get("files").is_some() {
        serde_json::from_value::<UpdateWorkspaceFilesRequest>(value).with_context(|| {
            format!(
                "Invalid workspace update payload in {}",
                files_path.display()
            )
        })?
    } else {
        UpdateWorkspaceFilesRequest {
            files: load_workspace_files(files_path).await?,
            delete_files: None,
            commit_message: None,
        }
    };

    if let Some(delete_path) = delete_path {
        payload.delete_files = Some(load_delete_files(delete_path).await?);
    }
    if let Some(commit_message) = commit_message {
        payload.commit_message = Some(commit_message);
    }

    Ok(payload)
}

async fn load_delete_files(path: &Path) -> Result<Vec<String>> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;

    if let Some(delete_files) = value.get("delete_files") {
        return serde_json::from_value(delete_files.clone())
            .with_context(|| format!("Invalid delete file list in {}", path.display()));
    }

    serde_json::from_value(value)
        .with_context(|| format!("Invalid delete file array in {}", path.display()))
}

fn resolve_attendee_id(
    attendee_id: Option<String>,
    session: Option<&StoredSession>,
    command_name: &str,
) -> Result<String> {
    if let Some(attendee_id) = attendee_id {
        return Ok(attendee_id);
    }
    if let Some(session) = session {
        if session.role.as_deref() == Some("attendee") {
            if let Some(subject) = session.sub.as_deref() {
                return Ok(subject.to_string());
            }
        }
    }
    bail!(
        "Missing attendee id for `{command_name}`. Pass --attendee-id or use an attendee session."
    );
}

fn resolve_active_profile(
    global: &GlobalOptions,
    config: &CliConfig,
) -> Result<Option<(String, ConnectionProfile)>> {
    let profile_name = global
        .profile
        .as_deref()
        .or(config.current_profile.as_deref());

    match profile_name {
        Some(profile_name) => {
            let profile = config
                .profiles
                .get(profile_name)
                .cloned()
                .ok_or_else(|| anyhow!("Unknown profile: {profile_name}"))?;
            Ok(Some((profile_name.to_string(), profile)))
        }
        None => Ok(None),
    }
}

fn resolve_session_file(
    global: &GlobalOptions,
    active_profile: Option<&(String, ConnectionProfile)>,
) -> PathBuf {
    if let Some(path) = &global.session_file {
        return path.clone();
    }
    if let Some((profile_name, _)) = active_profile {
        return default_profile_session_path(profile_name);
    }
    default_session_path()
}

fn resolve_base_url(
    explicit: Option<&str>,
    profile: Option<&ConnectionProfile>,
    session: Option<&StoredSession>,
) -> String {
    if let Some(explicit) = explicit {
        return explicit.trim_end_matches('/').to_string();
    }
    if let Some(profile) = profile {
        return profile.base_url.trim_end_matches('/').to_string();
    }
    if let Some(session) = session {
        return session.base_url.trim_end_matches('/').to_string();
    }
    env::var("OPEN_CODELABS_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn static_runtime_info(runtime: RuntimePreference) -> Option<CliRuntimeInfo> {
    match runtime {
        RuntimePreference::Auto => None,
        RuntimePreference::Backend => Some(CliRuntimeInfo {
            runtime: "backend".to_string(),
            version: "unknown".to_string(),
            auth_methods: vec!["browser".to_string(), "password".to_string()],
            capabilities: CliRuntimeCapabilities {
                admin_api: true,
                backup: true,
                workspace: true,
                audit: true,
                browser_auth: true,
            },
        }),
        RuntimePreference::Firebase => Some(CliRuntimeInfo {
            runtime: "firebase".to_string(),
            version: "frontend-managed".to_string(),
            auth_methods: vec!["google".to_string()],
            capabilities: empty_capabilities(),
        }),
        RuntimePreference::Supabase => Some(CliRuntimeInfo {
            runtime: "supabase".to_string(),
            version: "frontend-managed".to_string(),
            auth_methods: vec!["google".to_string()],
            capabilities: empty_capabilities(),
        }),
    }
}

fn empty_capabilities() -> CliRuntimeCapabilities {
    CliRuntimeCapabilities {
        admin_api: false,
        backup: false,
        workspace: false,
        audit: false,
        browser_auth: false,
    }
}

fn print_session(snapshot: &SessionSnapshot) {
    println!("subject: {}", snapshot.sub);
    println!("role: {}", snapshot.role);
    if let Some(codelab_id) = snapshot.codelab_id.as_deref() {
        println!("codelab_id: {codelab_id}");
    }
    println!("expires_at: {}", snapshot.exp);
}

fn print_codelab_list(codelabs: &[Codelab]) {
    println!("{:<38} {:<7} {:<20} {}", "id", "public", "author", "title");
    println!("{}", "-".repeat(96));
    for codelab in codelabs {
        println!(
            "{:<38} {:<7} {:<20} {}",
            codelab.id,
            if codelab.is_public != 0 { "yes" } else { "no" },
            truncate(&codelab.author, 20),
            codelab.title
        );
    }
}

fn print_codelab_detail(codelab: &Codelab, steps: &[Step]) {
    println!("id: {}", codelab.id);
    println!("title: {}", codelab.title);
    println!("author: {}", codelab.author);
    println!(
        "public: {}",
        if codelab.is_public != 0 { "yes" } else { "no" }
    );
    println!("steps: {}", steps.len());
    for step in steps {
        println!("  {:>2}. {}", step.step_number, step.title);
    }
}

fn print_backup_summary(summary: &BackupSummary) {
    println!("version: {}", summary.version);
    println!("created_at: {}", summary.created_at);
    println!("codelabs: {}", summary.codelabs);
    println!("steps: {}", summary.steps);
    println!("attendees: {}", summary.attendees);
    println!("materials: {}", summary.materials);
    println!("quizzes: {}", summary.quizzes);
    println!("submissions: {}", summary.submissions);
    println!("audit_logs: {}", summary.audit_logs);
    println!("uploads_files: {}", summary.uploads_files);
    println!("workspaces_files: {}", summary.workspaces_files);
}

fn print_workspace_info(info: &CodeServerInfo) {
    println!("path: {}", info.path);
    println!("structure_type: {}", info.structure_type);
}

fn print_updates_summary(summary: &UpdateCheckSummary) {
    println!(
        "{:<10} {:<12} {:<12} {:<8} {}",
        "component", "current", "latest", "update", "error"
    );
    println!("{}", "-".repeat(88));
    for (name, status) in [
        ("frontend", &summary.frontend),
        ("backend", &summary.backend),
    ] {
        println!(
            "{:<10} {:<12} {:<12} {:<8} {}",
            name,
            status.current.as_deref().unwrap_or("-"),
            status.latest.as_deref().unwrap_or("-"),
            if status.update_available { "yes" } else { "no" },
            status.error.as_deref().unwrap_or("-")
        );
    }
}

fn print_connect_profiles(config: &CliConfig) {
    if config.profiles.is_empty() {
        println!("No saved profiles.");
        return;
    }

    println!(
        "{:<9} {:<20} {:<10} {}",
        "current", "name", "runtime", "base_url"
    );
    println!("{}", "-".repeat(96));
    for (name, profile) in &config.profiles {
        let marker = if config.current_profile.as_deref() == Some(name.as_str()) {
            "*"
        } else {
            ""
        };
        println!(
            "{:<9} {:<20} {:<10} {}",
            marker,
            name,
            profile.runtime.as_str(),
            profile.base_url
        );
    }
}

fn print_connect_status(status: &ConnectStatusOutput) {
    println!(
        "profile: {}",
        status.profile.as_deref().unwrap_or("(implicit default)")
    );
    println!("base_url: {}", status.base_url);
    println!("session_file: {}", status.session_file.display());
    println!("runtime_preference: {}", status.runtime_preference);
    println!("runtime: {}", status.runtime);
    if let Some(version) = &status.version {
        println!("version: {version}");
    }
    println!("reachable: {}", if status.reachable { "yes" } else { "no" });
    if !status.auth_methods.is_empty() {
        println!("auth_methods: {}", status.auth_methods.join(", "));
    }
    println!(
        "capabilities: admin_api={}, backup={}, workspace={}, audit={}, browser_auth={}",
        status.capabilities.admin_api,
        status.capabilities.backup,
        status.capabilities.workspace,
        status.capabilities.audit,
        status.capabilities.browser_auth
    );
    if let Some(error) = &status.probe_error {
        println!("probe_error: {error}");
    }
}

fn print_auth_status(status: &AuthStatusOutput) {
    println!(
        "profile: {}",
        status.profile.as_deref().unwrap_or("(implicit default)")
    );
    println!("base_url: {}", status.base_url);
    println!("session_file: {}", status.session_file.display());
    println!(
        "authenticated: {}",
        if status.authenticated { "yes" } else { "no" }
    );
    if let Some(subject) = &status.subject {
        println!("subject: {subject}");
    }
    if let Some(role) = &status.role {
        println!("role: {role}");
    }
    if let Some(codelab_id) = &status.codelab_id {
        println!("codelab_id: {codelab_id}");
    }
    if let Some(expires_at) = status.expires_at {
        println!("expires_at: {expires_at}");
    }
    if let Some(error) = &status.error {
        println!("error: {error}");
    }
}

fn print_audit_logs(logs: &[AuditLog]) {
    println!(
        "{:<20} {:<24} {:<10} {:<38}",
        "created_at", "action", "actor", "codelab_id"
    );
    println!("{}", "-".repeat(108));
    for log in logs {
        println!(
            "{:<20} {:<24} {:<10} {:<38}",
            truncate(&log.created_at, 20),
            truncate(&log.action, 24),
            truncate(&log.actor_type, 10),
            log.codelab_id.as_deref().unwrap_or("-")
        );
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).context("Failed to serialize JSON output")?
    );
    Ok(())
}

fn truncate(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }
    let mut truncated = value.chars().take(max_len).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn parse_cli() -> Result<(GlobalOptions, Command)> {
    let mut global = GlobalOptions::default();
    let filtered_args = extract_global_options(env::args().skip(1).collect(), &mut global)?;
    let mut args = Args::new(filtered_args);

    let Some(command) = args.next() else {
        return Ok((global, Command::Help));
    };

    let command = match command.as_str() {
        "admin" => Command::Admin(parse_admin(&mut args)?),
        "auth" => Command::Auth(parse_auth(&mut args)?),
        "connect" => Command::Connect(parse_connect(&mut args)?),
        "login" => Command::Login(parse_login(&mut args)?),
        "logout" => Command::Logout,
        "session" => Command::Session,
        "codelab" => Command::Codelab(parse_codelab(&mut args)?),
        "backup" => Command::Backup(parse_backup(&mut args)?),
        "audit" => Command::Audit(parse_audit(&mut args)?),
        "workspace" => Command::Workspace(parse_workspace(&mut args)?),
        "attendee" => Command::Attendee(parse_attendee(&mut args)?),
        "help" => {
            if args.peek().is_some() {
                Command::HelpDesk(parse_helpdesk(&mut args)?)
            } else {
                Command::Help
            }
        }
        "feedback" => Command::Feedback(parse_feedback(&mut args)?),
        "materials" | "material" => Command::Materials(parse_material(&mut args)?),
        "quiz" | "quizzes" => Command::Quiz(parse_quiz(&mut args)?),
        "submission" | "submissions" => Command::Submission(parse_submission(&mut args)?),
        "chat" => Command::Chat(parse_chat(&mut args)?),
        "upload" => Command::Upload(parse_upload(&mut args)?),
        "inline" => Command::Inline(parse_inline(&mut args)?),
        "ai" => Command::Ai(parse_ai(&mut args)?),
        other => bail!("Unknown command: {other}"),
    };

    args.ensure_exhausted()?;
    Ok((global, command))
}

fn parse_admin(args: &mut Args) -> Result<AdminCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("admin"));
    };

    match subcommand.as_str() {
        "settings" => {
            let mut gemini_api_key = None;
            let mut admin_password = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--gemini-api-key" => {
                        gemini_api_key = Some(args.next_required("--gemini-api-key")?)
                    }
                    "--admin-password" => {
                        admin_password = Some(args.next_required("--admin-password")?)
                    }
                    "-h" | "--help" => return Err(help_error("admin settings")),
                    other => bail!("Unknown admin settings option: {other}"),
                }
            }

            Ok(AdminCommand::Settings {
                gemini_api_key: gemini_api_key.unwrap_or_default(),
                admin_password,
            })
        }
        "updates" => Ok(AdminCommand::Updates),
        _ => Err(help_error("admin")),
    }
}

fn parse_auth(args: &mut Args) -> Result<AuthCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("auth"));
    };

    match subcommand.as_str() {
        "login" => {
            let mut command = AuthLoginCommand::default();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--no-open" => command.no_open = true,
                    "-h" | "--help" => return Err(help_error("auth login")),
                    other => bail!("Unknown auth login option: {other}"),
                }
            }
            Ok(AuthCommand::Login(command))
        }
        "logout" => Ok(AuthCommand::Logout),
        "status" => Ok(AuthCommand::Status),
        _ => Err(help_error("auth")),
    }
}

fn extract_global_options(items: Vec<String>, global: &mut GlobalOptions) -> Result<Vec<String>> {
    let mut filtered = Vec::new();
    let mut index = 0;

    while index < items.len() {
        match items[index].as_str() {
            "--base-url" => {
                index += 1;
                let value = items
                    .get(index)
                    .cloned()
                    .ok_or_else(|| anyhow!("Missing value for --base-url"))?;
                global.base_url = Some(value);
            }
            "--session-file" => {
                index += 1;
                let value = items
                    .get(index)
                    .cloned()
                    .ok_or_else(|| anyhow!("Missing value for --session-file"))?;
                global.session_file = Some(PathBuf::from(value));
            }
            "--config-file" => {
                index += 1;
                let value = items
                    .get(index)
                    .cloned()
                    .ok_or_else(|| anyhow!("Missing value for --config-file"))?;
                global.config_file = PathBuf::from(value);
            }
            "--profile" => {
                index += 1;
                let value = items
                    .get(index)
                    .cloned()
                    .ok_or_else(|| anyhow!("Missing value for --profile"))?;
                global.profile = Some(value);
            }
            "--json" => {
                global.json = true;
            }
            "-h" | "--help" if filtered.is_empty() => {
                return Ok(Vec::new());
            }
            _ => filtered.push(items[index].clone()),
        }
        index += 1;
    }

    Ok(filtered)
}

fn parse_connect(args: &mut Args) -> Result<ConnectCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("connect"));
    };

    match subcommand.as_str() {
        "add" => {
            let mut name = None;
            let mut url = None;
            let mut runtime = RuntimePreference::Auto;
            let mut activate = false;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--name" => name = Some(args.next_required("--name")?),
                    "--url" => url = Some(args.next_required("--url")?),
                    "--runtime" => {
                        let value = args.next_required("--runtime")?;
                        runtime = RuntimePreference::parse(&value)
                            .ok_or_else(|| anyhow!("Invalid runtime: {value}"))?;
                    }
                    "--activate" => activate = true,
                    "-h" | "--help" => return Err(help_error("connect add")),
                    other => bail!("Unknown connect add option: {other}"),
                }
            }

            Ok(ConnectCommand::Add {
                name: name.ok_or_else(|| anyhow!("Missing --name"))?,
                url: url.ok_or_else(|| anyhow!("Missing --url"))?,
                runtime,
                activate,
            })
        }
        "use" => Ok(ConnectCommand::Use {
            name: parse_required_string_flag(args, "--name", "connect use")?,
        }),
        "list" => Ok(ConnectCommand::List),
        "status" => Ok(ConnectCommand::Status),
        _ => Err(help_error("connect")),
    }
}

fn parse_login(args: &mut Args) -> Result<LoginCommand> {
    let mut admin_id = env::var("OPEN_CODELABS_ADMIN_ID").ok();
    let mut admin_pw = env::var("OPEN_CODELABS_ADMIN_PW").ok();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--admin-id" => admin_id = Some(args.next_required("--admin-id")?),
            "--admin-pw" => admin_pw = Some(args.next_required("--admin-pw")?),
            "-h" | "--help" => return Err(help_error("login")),
            other => bail!("Unknown login option: {other}"),
        }
    }

    Ok(LoginCommand {
        admin_id: admin_id.ok_or_else(|| anyhow!("Missing --admin-id"))?,
        admin_pw: admin_pw.ok_or_else(|| anyhow!("Missing --admin-pw"))?,
    })
}

fn parse_codelab(args: &mut Args) -> Result<CodelabCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("codelab"));
    };

    match subcommand.as_str() {
        "list" => Ok(CodelabCommand::List),
        "reference" => Ok(CodelabCommand::Reference),
        "get" => Ok(CodelabCommand::Get {
            id: parse_required_string_flag(args, "--id", "codelab get")?,
        }),
        "create" => parse_create_codelab(args),
        "update" => parse_update_codelab(args),
        "delete" => Ok(CodelabCommand::Delete {
            id: parse_required_string_flag(args, "--id", "codelab delete")?,
        }),
        "copy" => Ok(CodelabCommand::Copy {
            id: parse_required_string_flag(args, "--id", "codelab copy")?,
        }),
        "export" => parse_codelab_export(args),
        "import" => Ok(CodelabCommand::Import {
            file: PathBuf::from(parse_required_string_flag(
                args,
                "--file",
                "codelab import",
            )?),
        }),
        "push-steps" => parse_push_steps(args),
        _ => Err(help_error("codelab")),
    }
}

fn parse_create_codelab(args: &mut Args) -> Result<CodelabCommand> {
    let mut title = None;
    let mut description = None;
    let mut author = None;
    let mut guide_file = None;
    let mut is_public = true;
    let mut quiz_enabled = false;
    let mut require_quiz = false;
    let mut require_feedback = false;
    let mut require_submission = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--title" => title = Some(args.next_required("--title")?),
            "--description" => description = Some(args.next_required("--description")?),
            "--author" => author = Some(args.next_required("--author")?),
            "--guide-file" => guide_file = Some(PathBuf::from(args.next_required("--guide-file")?)),
            "--private" => is_public = false,
            "--quiz-enabled" => quiz_enabled = true,
            "--require-quiz" => require_quiz = true,
            "--require-feedback" => require_feedback = true,
            "--require-submission" => require_submission = true,
            "-h" | "--help" => return Err(help_error("codelab create")),
            other => bail!("Unknown codelab create option: {other}"),
        }
    }

    Ok(CodelabCommand::Create(CreateCodelabCommand {
        title: title.ok_or_else(|| anyhow!("Missing --title"))?,
        description: description.ok_or_else(|| anyhow!("Missing --description"))?,
        author: author.ok_or_else(|| anyhow!("Missing --author"))?,
        is_public,
        quiz_enabled,
        require_quiz,
        require_feedback,
        require_submission,
        guide_file,
    }))
}

fn parse_update_codelab(args: &mut Args) -> Result<CodelabCommand> {
    let mut id = None;
    let mut title = None;
    let mut description = None;
    let mut author = None;
    let mut guide_file = None;
    let mut is_public = true;
    let mut quiz_enabled = false;
    let mut require_quiz = false;
    let mut require_feedback = false;
    let mut require_submission = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--id" => id = Some(args.next_required("--id")?),
            "--title" => title = Some(args.next_required("--title")?),
            "--description" => description = Some(args.next_required("--description")?),
            "--author" => author = Some(args.next_required("--author")?),
            "--guide-file" => guide_file = Some(PathBuf::from(args.next_required("--guide-file")?)),
            "--private" => is_public = false,
            "--quiz-enabled" => quiz_enabled = true,
            "--require-quiz" => require_quiz = true,
            "--require-feedback" => require_feedback = true,
            "--require-submission" => require_submission = true,
            "-h" | "--help" => return Err(help_error("codelab update")),
            other => bail!("Unknown codelab update option: {other}"),
        }
    }

    Ok(CodelabCommand::Update {
        id: id.ok_or_else(|| anyhow!("Missing --id"))?,
        command: CreateCodelabCommand {
            title: title.ok_or_else(|| anyhow!("Missing --title"))?,
            description: description.ok_or_else(|| anyhow!("Missing --description"))?,
            author: author.ok_or_else(|| anyhow!("Missing --author"))?,
            is_public,
            quiz_enabled,
            require_quiz,
            require_feedback,
            require_submission,
            guide_file,
        },
    })
}

fn parse_codelab_export(args: &mut Args) -> Result<CodelabCommand> {
    let mut id = None;
    let mut output = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--id" => id = Some(args.next_required("--id")?),
            "--output" => output = Some(PathBuf::from(args.next_required("--output")?)),
            "-h" | "--help" => return Err(help_error("codelab export")),
            other => bail!("Unknown codelab export option: {other}"),
        }
    }

    Ok(CodelabCommand::Export {
        id: id.ok_or_else(|| anyhow!("Missing --id"))?,
        output,
    })
}

fn parse_push_steps(args: &mut Args) -> Result<CodelabCommand> {
    let mut id = None;
    let mut file = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--id" => id = Some(args.next_required("--id")?),
            "--file" => file = Some(PathBuf::from(args.next_required("--file")?)),
            "-h" | "--help" => return Err(help_error("codelab push-steps")),
            other => bail!("Unknown codelab push-steps option: {other}"),
        }
    }

    Ok(CodelabCommand::PushSteps {
        id: id.ok_or_else(|| anyhow!("Missing --id"))?,
        file: file.ok_or_else(|| anyhow!("Missing --file"))?,
    })
}

fn parse_backup(args: &mut Args) -> Result<BackupCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("backup"));
    };

    match subcommand.as_str() {
        "export" => {
            let mut output = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--output" => output = Some(PathBuf::from(args.next_required("--output")?)),
                    "-h" | "--help" => return Err(help_error("backup export")),
                    other => bail!("Unknown backup export option: {other}"),
                }
            }
            Ok(BackupCommand::Export { output })
        }
        "inspect" => Ok(BackupCommand::Inspect {
            file: PathBuf::from(parse_required_string_flag(
                args,
                "--file",
                "backup inspect",
            )?),
        }),
        "restore" => Ok(BackupCommand::Restore {
            file: PathBuf::from(parse_required_string_flag(
                args,
                "--file",
                "backup restore",
            )?),
        }),
        _ => Err(help_error("backup")),
    }
}

fn parse_audit(args: &mut Args) -> Result<AuditCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("audit"));
    };
    match subcommand.as_str() {
        "logs" => {
            let mut limit = None;
            let mut offset = None;
            let mut action = None;
            let mut codelab_id = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--limit" => {
                        let value = args.next_required("--limit")?;
                        limit = Some(
                            value
                                .parse::<usize>()
                                .with_context(|| format!("Invalid value for --limit: {value}"))?,
                        );
                    }
                    "--offset" => {
                        let value = args.next_required("--offset")?;
                        offset = Some(
                            value
                                .parse::<usize>()
                                .with_context(|| format!("Invalid value for --offset: {value}"))?,
                        );
                    }
                    "--action" => action = Some(args.next_required("--action")?),
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "-h" | "--help" => return Err(help_error("audit logs")),
                    other => bail!("Unknown audit logs option: {other}"),
                }
            }

            Ok(AuditCommand::Logs {
                limit,
                offset,
                action,
                codelab_id,
            })
        }
        _ => Err(help_error("audit")),
    }
}

fn parse_workspace(args: &mut Args) -> Result<WorkspaceCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("workspace"));
    };

    match subcommand.as_str() {
        "create" => {
            let mut codelab_id = None;
            let mut structure_type = None;
            let mut files_json = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--structure-type" => {
                        structure_type = Some(args.next_required("--structure-type")?)
                    }
                    "--files-json" => {
                        files_json = Some(PathBuf::from(args.next_required("--files-json")?))
                    }
                    "-h" | "--help" => return Err(help_error("workspace create")),
                    other => bail!("Unknown workspace create option: {other}"),
                }
            }

            Ok(WorkspaceCommand::Create {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                structure_type,
                files_json,
            })
        }
        "info" => Ok(WorkspaceCommand::Info {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "workspace info")?,
        }),
        "download" => {
            let mut codelab_id = None;
            let mut output = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--output" => output = Some(PathBuf::from(args.next_required("--output")?)),
                    "-h" | "--help" => return Err(help_error("workspace download")),
                    other => bail!("Unknown workspace download option: {other}"),
                }
            }

            Ok(WorkspaceCommand::Download {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                output,
            })
        }
        "delete" => Ok(WorkspaceCommand::Delete {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "workspace delete")?,
        }),
        "branches" => Ok(WorkspaceCommand::Branches {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "workspace branches")?,
        }),
        "branch-create" => {
            let mut codelab_id = None;
            let mut step_number = None;
            let mut branch_type = "start".to_string();

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--step-number" => {
                        let value = args.next_required("--step-number")?;
                        step_number = Some(value.parse::<i32>().with_context(|| {
                            format!("Invalid value for --step-number: {value}")
                        })?);
                    }
                    "--branch-type" => branch_type = args.next_required("--branch-type")?,
                    "-h" | "--help" => return Err(help_error("workspace branch-create")),
                    other => bail!("Unknown workspace branch-create option: {other}"),
                }
            }

            Ok(WorkspaceCommand::BranchCreate {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                step_number: step_number.ok_or_else(|| anyhow!("Missing --step-number"))?,
                branch_type,
            })
        }
        "branch-files" => {
            let mut codelab_id = None;
            let mut branch = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--branch" => branch = Some(args.next_required("--branch")?),
                    "-h" | "--help" => return Err(help_error("workspace branch-files")),
                    other => bail!("Unknown workspace branch-files option: {other}"),
                }
            }

            Ok(WorkspaceCommand::BranchFiles {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                branch: branch.ok_or_else(|| anyhow!("Missing --branch"))?,
            })
        }
        "branch-read" => {
            let mut codelab_id = None;
            let mut branch = None;
            let mut file = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--branch" => branch = Some(args.next_required("--branch")?),
                    "--file" => file = Some(args.next_required("--file")?),
                    "-h" | "--help" => return Err(help_error("workspace branch-read")),
                    other => bail!("Unknown workspace branch-read option: {other}"),
                }
            }

            Ok(WorkspaceCommand::BranchRead {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                branch: branch.ok_or_else(|| anyhow!("Missing --branch"))?,
                file: file.ok_or_else(|| anyhow!("Missing --file"))?,
            })
        }
        "branch-update" => {
            let mut codelab_id = None;
            let mut branch = None;
            let mut files_json = None;
            let mut delete_json = None;
            let mut commit_message = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--branch" => branch = Some(args.next_required("--branch")?),
                    "--files-json" => {
                        files_json = Some(PathBuf::from(args.next_required("--files-json")?))
                    }
                    "--delete-json" => {
                        delete_json = Some(PathBuf::from(args.next_required("--delete-json")?))
                    }
                    "--commit-message" => {
                        commit_message = Some(args.next_required("--commit-message")?)
                    }
                    "-h" | "--help" => return Err(help_error("workspace branch-update")),
                    other => bail!("Unknown workspace branch-update option: {other}"),
                }
            }

            Ok(WorkspaceCommand::BranchUpdate {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                branch: branch.ok_or_else(|| anyhow!("Missing --branch"))?,
                files_json: files_json.ok_or_else(|| anyhow!("Missing --files-json"))?,
                delete_json,
                commit_message,
            })
        }
        "folders" => Ok(WorkspaceCommand::Folders {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "workspace folders")?,
        }),
        "folder-create" => {
            let mut codelab_id = None;
            let mut step_number = None;
            let mut folder_type = "start".to_string();
            let mut files_json = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--step-number" => {
                        let value = args.next_required("--step-number")?;
                        step_number = Some(value.parse::<i32>().with_context(|| {
                            format!("Invalid value for --step-number: {value}")
                        })?);
                    }
                    "--folder-type" => folder_type = args.next_required("--folder-type")?,
                    "--files-json" => {
                        files_json = Some(PathBuf::from(args.next_required("--files-json")?))
                    }
                    "-h" | "--help" => return Err(help_error("workspace folder-create")),
                    other => bail!("Unknown workspace folder-create option: {other}"),
                }
            }

            Ok(WorkspaceCommand::FolderCreate {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                step_number: step_number.ok_or_else(|| anyhow!("Missing --step-number"))?,
                folder_type,
                files_json: files_json.ok_or_else(|| anyhow!("Missing --files-json"))?,
            })
        }
        "folder-files" => {
            let mut codelab_id = None;
            let mut folder = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--folder" => folder = Some(args.next_required("--folder")?),
                    "-h" | "--help" => return Err(help_error("workspace folder-files")),
                    other => bail!("Unknown workspace folder-files option: {other}"),
                }
            }

            Ok(WorkspaceCommand::FolderFiles {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                folder: folder.ok_or_else(|| anyhow!("Missing --folder"))?,
            })
        }
        "folder-read" => {
            let mut codelab_id = None;
            let mut folder = None;
            let mut file = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--folder" => folder = Some(args.next_required("--folder")?),
                    "--file" => file = Some(args.next_required("--file")?),
                    "-h" | "--help" => return Err(help_error("workspace folder-read")),
                    other => bail!("Unknown workspace folder-read option: {other}"),
                }
            }

            Ok(WorkspaceCommand::FolderRead {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                folder: folder.ok_or_else(|| anyhow!("Missing --folder"))?,
                file: file.ok_or_else(|| anyhow!("Missing --file"))?,
            })
        }
        "folder-update" => {
            let mut codelab_id = None;
            let mut folder = None;
            let mut files_json = None;
            let mut delete_json = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--folder" => folder = Some(args.next_required("--folder")?),
                    "--files-json" => {
                        files_json = Some(PathBuf::from(args.next_required("--files-json")?))
                    }
                    "--delete-json" => {
                        delete_json = Some(PathBuf::from(args.next_required("--delete-json")?))
                    }
                    "-h" | "--help" => return Err(help_error("workspace folder-update")),
                    other => bail!("Unknown workspace folder-update option: {other}"),
                }
            }

            Ok(WorkspaceCommand::FolderUpdate {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                folder: folder.ok_or_else(|| anyhow!("Missing --folder"))?,
                files_json: files_json.ok_or_else(|| anyhow!("Missing --files-json"))?,
                delete_json,
            })
        }
        _ => Err(help_error("workspace")),
    }
}

fn parse_attendee(args: &mut Args) -> Result<AttendeeCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("attendee"));
    };

    match subcommand.as_str() {
        "join" | "register" => {
            let mut codelab_id = None;
            let mut name = None;
            let mut code = None;
            let mut email = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--name" => name = Some(args.next_required("--name")?),
                    "--code" => code = Some(args.next_required("--code")?),
                    "--email" => email = Some(args.next_required("--email")?),
                    "-h" | "--help" => return Err(help_error("attendee join")),
                    other => bail!("Unknown attendee join option: {other}"),
                }
            }

            Ok(AttendeeCommand::Join {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                name: name.ok_or_else(|| anyhow!("Missing --name"))?,
                code: code.ok_or_else(|| anyhow!("Missing --code"))?,
                email,
            })
        }
        "list" => Ok(AttendeeCommand::List {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "attendee list")?,
        }),
        "complete" => Ok(AttendeeCommand::Complete {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "attendee complete")?,
        }),
        "certificate" => {
            let mut attendee_id = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--attendee-id" => attendee_id = Some(args.next_required("--attendee-id")?),
                    "-h" | "--help" => return Err(help_error("attendee certificate")),
                    other => bail!("Unknown attendee certificate option: {other}"),
                }
            }
            Ok(AttendeeCommand::Certificate { attendee_id })
        }
        _ => Err(help_error("attendee")),
    }
}

fn parse_helpdesk(args: &mut Args) -> Result<HelpDeskCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("help"));
    };

    match subcommand.as_str() {
        "request" => {
            let mut codelab_id = None;
            let mut step_number = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--step-number" => {
                        let value = args.next_required("--step-number")?;
                        step_number = Some(value.parse::<i32>().with_context(|| {
                            format!("Invalid value for --step-number: {value}")
                        })?);
                    }
                    "-h" | "--help" => return Err(help_error("help request")),
                    other => bail!("Unknown help request option: {other}"),
                }
            }

            Ok(HelpDeskCommand::Request {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                step_number: step_number.ok_or_else(|| anyhow!("Missing --step-number"))?,
            })
        }
        "list" => Ok(HelpDeskCommand::List {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "help list")?,
        }),
        "resolve" => {
            let mut codelab_id = None;
            let mut help_id = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--help-id" => help_id = Some(args.next_required("--help-id")?),
                    "-h" | "--help" => return Err(help_error("help resolve")),
                    other => bail!("Unknown help resolve option: {other}"),
                }
            }

            Ok(HelpDeskCommand::Resolve {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                help_id: help_id.ok_or_else(|| anyhow!("Missing --help-id"))?,
            })
        }
        _ => Err(help_error("help")),
    }
}

fn parse_feedback(args: &mut Args) -> Result<FeedbackCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("feedback"));
    };

    match subcommand.as_str() {
        "submit" => {
            let mut codelab_id = None;
            let mut difficulty = None;
            let mut satisfaction = None;
            let mut comment = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--difficulty" => difficulty = Some(args.next_required("--difficulty")?),
                    "--satisfaction" => satisfaction = Some(args.next_required("--satisfaction")?),
                    "--comment" => comment = Some(args.next_required("--comment")?),
                    "-h" | "--help" => return Err(help_error("feedback submit")),
                    other => bail!("Unknown feedback submit option: {other}"),
                }
            }

            Ok(FeedbackCommand::Submit {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                difficulty: difficulty.ok_or_else(|| anyhow!("Missing --difficulty"))?,
                satisfaction: satisfaction.ok_or_else(|| anyhow!("Missing --satisfaction"))?,
                comment,
            })
        }
        "list" => Ok(FeedbackCommand::List {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "feedback list")?,
        }),
        _ => Err(help_error("feedback")),
    }
}

fn parse_material(args: &mut Args) -> Result<MaterialCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("materials"));
    };

    match subcommand.as_str() {
        "list" => Ok(MaterialCommand::List {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "materials list")?,
        }),
        "upload" => Ok(MaterialCommand::Upload {
            file: PathBuf::from(parse_required_string_flag(
                args,
                "--file",
                "materials upload",
            )?),
        }),
        "add" => {
            let mut codelab_id = None;
            let mut title = None;
            let mut material_type = None;
            let mut link_url = None;
            let mut file_path = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--title" => title = Some(args.next_required("--title")?),
                    "--type" => material_type = Some(args.next_required("--type")?),
                    "--url" => link_url = Some(args.next_required("--url")?),
                    "--file-path" => file_path = Some(args.next_required("--file-path")?),
                    "-h" | "--help" => return Err(help_error("materials add")),
                    other => bail!("Unknown materials add option: {other}"),
                }
            }

            Ok(MaterialCommand::Add {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                title: title.ok_or_else(|| anyhow!("Missing --title"))?,
                material_type: material_type.ok_or_else(|| anyhow!("Missing --type"))?,
                link_url,
                file_path,
            })
        }
        "delete" => {
            let mut codelab_id = None;
            let mut material_id = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--material-id" => material_id = Some(args.next_required("--material-id")?),
                    "-h" | "--help" => return Err(help_error("materials delete")),
                    other => bail!("Unknown materials delete option: {other}"),
                }
            }

            Ok(MaterialCommand::Delete {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                material_id: material_id.ok_or_else(|| anyhow!("Missing --material-id"))?,
            })
        }
        _ => Err(help_error("materials")),
    }
}

fn parse_quiz(args: &mut Args) -> Result<QuizCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("quiz"));
    };

    match subcommand.as_str() {
        "list" => Ok(QuizCommand::List {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "quiz list")?,
        }),
        "update" => {
            let mut codelab_id = None;
            let mut file = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--file" => file = Some(PathBuf::from(args.next_required("--file")?)),
                    "-h" | "--help" => return Err(help_error("quiz update")),
                    other => bail!("Unknown quiz update option: {other}"),
                }
            }

            Ok(QuizCommand::Update {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                file: file.ok_or_else(|| anyhow!("Missing --file"))?,
            })
        }
        "submit" => {
            let mut codelab_id = None;
            let mut file = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--file" => file = Some(PathBuf::from(args.next_required("--file")?)),
                    "-h" | "--help" => return Err(help_error("quiz submit")),
                    other => bail!("Unknown quiz submit option: {other}"),
                }
            }

            Ok(QuizCommand::Submit {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                file: file.ok_or_else(|| anyhow!("Missing --file"))?,
            })
        }
        "submissions" => Ok(QuizCommand::Submissions {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "quiz submissions")?,
        }),
        _ => Err(help_error("quiz")),
    }
}

fn parse_submission(args: &mut Args) -> Result<SubmissionCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("submission"));
    };

    match subcommand.as_str() {
        "list" => Ok(SubmissionCommand::List {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "submission list")?,
        }),
        "file" => {
            let mut codelab_id = None;
            let mut attendee_id = None;
            let mut file = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--attendee-id" => attendee_id = Some(args.next_required("--attendee-id")?),
                    "--file" => file = Some(PathBuf::from(args.next_required("--file")?)),
                    "-h" | "--help" => return Err(help_error("submission file")),
                    other => bail!("Unknown submission file option: {other}"),
                }
            }

            Ok(SubmissionCommand::File {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                attendee_id,
                file: file.ok_or_else(|| anyhow!("Missing --file"))?,
            })
        }
        "link" => {
            let mut codelab_id = None;
            let mut attendee_id = None;
            let mut url = None;
            let mut title = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--attendee-id" => attendee_id = Some(args.next_required("--attendee-id")?),
                    "--url" => url = Some(args.next_required("--url")?),
                    "--title" => title = Some(args.next_required("--title")?),
                    "-h" | "--help" => return Err(help_error("submission link")),
                    other => bail!("Unknown submission link option: {other}"),
                }
            }

            Ok(SubmissionCommand::Link {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                attendee_id,
                url: url.ok_or_else(|| anyhow!("Missing --url"))?,
                title,
            })
        }
        "delete" => {
            let mut codelab_id = None;
            let mut attendee_id = None;
            let mut submission_id = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--attendee-id" => attendee_id = Some(args.next_required("--attendee-id")?),
                    "--submission-id" => {
                        submission_id = Some(args.next_required("--submission-id")?)
                    }
                    "-h" | "--help" => return Err(help_error("submission delete")),
                    other => bail!("Unknown submission delete option: {other}"),
                }
            }

            Ok(SubmissionCommand::Delete {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                attendee_id,
                submission_id: submission_id.ok_or_else(|| anyhow!("Missing --submission-id"))?,
            })
        }
        _ => Err(help_error("submission")),
    }
}

fn parse_chat(args: &mut Args) -> Result<ChatCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("chat"));
    };

    match subcommand.as_str() {
        "history" => Ok(ChatCommand::History {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "chat history")?,
        }),
        _ => Err(help_error("chat")),
    }
}

fn parse_upload(args: &mut Args) -> Result<UploadCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("upload"));
    };

    match subcommand.as_str() {
        "image" => Ok(UploadCommand::Image {
            file: PathBuf::from(parse_required_string_flag(args, "--file", "upload image")?),
        }),
        _ => Err(help_error("upload")),
    }
}

fn parse_inline(args: &mut Args) -> Result<InlineCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("inline"));
    };

    match subcommand.as_str() {
        "list" => {
            let mut codelab_id = None;
            let mut target_type = None;
            let mut target_step_id = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--target-type" => target_type = Some(args.next_required("--target-type")?),
                    "--target-step-id" => {
                        target_step_id = Some(args.next_required("--target-step-id")?)
                    }
                    "-h" | "--help" => return Err(help_error("inline list")),
                    other => bail!("Unknown inline list option: {other}"),
                }
            }

            Ok(InlineCommand::List {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                target_type,
                target_step_id,
            })
        }
        "create" => {
            let mut codelab_id = None;
            let mut file = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--file" => file = Some(PathBuf::from(args.next_required("--file")?)),
                    "-h" | "--help" => return Err(help_error("inline create")),
                    other => bail!("Unknown inline create option: {other}"),
                }
            }

            Ok(InlineCommand::Create {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                file: file.ok_or_else(|| anyhow!("Missing --file"))?,
            })
        }
        "reply" => {
            let mut codelab_id = None;
            let mut thread_id = None;
            let mut file = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--thread-id" => thread_id = Some(args.next_required("--thread-id")?),
                    "--file" => file = Some(PathBuf::from(args.next_required("--file")?)),
                    "-h" | "--help" => return Err(help_error("inline reply")),
                    other => bail!("Unknown inline reply option: {other}"),
                }
            }

            Ok(InlineCommand::Reply {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                thread_id: thread_id.ok_or_else(|| anyhow!("Missing --thread-id"))?,
                file: file.ok_or_else(|| anyhow!("Missing --file"))?,
            })
        }
        "delete" => {
            let mut codelab_id = None;
            let mut thread_id = None;
            let mut comment_id = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "--thread-id" => thread_id = Some(args.next_required("--thread-id")?),
                    "--comment-id" => comment_id = Some(args.next_required("--comment-id")?),
                    "-h" | "--help" => return Err(help_error("inline delete")),
                    other => bail!("Unknown inline delete option: {other}"),
                }
            }

            Ok(InlineCommand::Delete {
                codelab_id: codelab_id.ok_or_else(|| anyhow!("Missing --codelab-id"))?,
                thread_id: thread_id.ok_or_else(|| anyhow!("Missing --thread-id"))?,
                comment_id: comment_id.ok_or_else(|| anyhow!("Missing --comment-id"))?,
            })
        }
        _ => Err(help_error("inline")),
    }
}

fn parse_ai(args: &mut Args) -> Result<AiCommand> {
    let Some(subcommand) = args.next() else {
        return Err(help_error("ai"));
    };

    match subcommand.as_str() {
        "conversations" => Ok(AiCommand::Conversations {
            codelab_id: parse_required_string_flag(args, "--codelab-id", "ai conversations")?,
        }),
        "stream" => Ok(AiCommand::Stream {
            file: PathBuf::from(parse_required_string_flag(args, "--file", "ai stream")?),
        }),
        "save" => Ok(AiCommand::Save {
            file: PathBuf::from(parse_required_string_flag(args, "--file", "ai save")?),
        }),
        "threads" => Ok(AiCommand::Threads),
        "thread-create" => {
            let mut title = None;
            let mut codelab_id = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--title" => title = Some(args.next_required("--title")?),
                    "--codelab-id" => codelab_id = Some(args.next_required("--codelab-id")?),
                    "-h" | "--help" => return Err(help_error("ai thread-create")),
                    other => bail!("Unknown ai thread-create option: {other}"),
                }
            }

            Ok(AiCommand::ThreadCreate {
                title: title.ok_or_else(|| anyhow!("Missing --title"))?,
                codelab_id,
            })
        }
        "thread-delete" => Ok(AiCommand::ThreadDelete {
            thread_id: parse_required_string_flag(args, "--thread-id", "ai thread-delete")?,
        }),
        "messages" => Ok(AiCommand::Messages {
            thread_id: parse_required_string_flag(args, "--thread-id", "ai messages")?,
        }),
        "message-add" => {
            let mut thread_id = None;
            let mut file = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--thread-id" => thread_id = Some(args.next_required("--thread-id")?),
                    "--file" => file = Some(PathBuf::from(args.next_required("--file")?)),
                    "-h" | "--help" => return Err(help_error("ai message-add")),
                    other => bail!("Unknown ai message-add option: {other}"),
                }
            }

            Ok(AiCommand::MessageAdd {
                thread_id: thread_id.ok_or_else(|| anyhow!("Missing --thread-id"))?,
                file: file.ok_or_else(|| anyhow!("Missing --file"))?,
            })
        }
        _ => Err(help_error("ai")),
    }
}

fn parse_required_string_flag(args: &mut Args, flag: &str, help_topic: &str) -> Result<String> {
    let mut value = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            current if current == flag => value = Some(args.next_required(flag)?),
            "-h" | "--help" => return Err(help_error(help_topic)),
            other => bail!("Unknown option for {help_topic}: {other}"),
        }
    }
    value.ok_or_else(|| anyhow!("Missing {flag}"))
}

fn help_error(topic: &str) -> anyhow::Error {
    anyhow!(
        "Usage help requested for `{topic}`. Run `{}` help for command overview.",
        program_name()
    )
}

fn print_help() {
    let program_name = program_name();
    println!(
        r#"Open Codelabs CLI

Usage:
  {program_name} [global options] <command>

Global options:
  --base-url <url>        Backend base URL (default: OPEN_CODELABS_BASE_URL or http://localhost:8080)
  --session-file <path>   Session file path override
  --config-file <path>    CLI config path (default: ~/.open-codelabs/config.json)
  --profile <name>        Saved connection profile to use
  --json                  Print JSON instead of table/text output
  -h, --help              Show this help

Commands:
  admin settings [--gemini-api-key <key>] [--admin-password <pw>]
  admin updates
  auth login [--no-open]
  auth logout
  auth status
  connect add --name <name> --url <url> [--runtime <auto|backend|firebase|supabase>] [--activate]
  connect use --name <name>
  connect list
  connect status
  login --admin-id <id> --admin-pw <pw>   Legacy direct login
  logout                                   Legacy alias for auth logout
  session                                  Legacy alias for auth status
  codelab list
  codelab reference
  codelab get --id <id>
  codelab create --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
  codelab update --id <id> --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
  codelab delete --id <id>
  codelab copy --id <id>
  codelab export --id <id> [--output <path>]
  codelab import --file <zip>
  codelab push-steps --id <id> --file <json>
  backup export [--output <path>]
  backup inspect --file <zip>
  backup restore --file <zip>
  audit logs [--limit <n>] [--offset <n>] [--action <name>] [--codelab-id <id>]
  workspace create --codelab-id <id> [--structure-type <branch|folder>] [--files-json <path>]
  workspace info --codelab-id <id>
  workspace download --codelab-id <id> [--output <path>]
  workspace delete --codelab-id <id>
  workspace branches --codelab-id <id>
  workspace branch-create --codelab-id <id> --step-number <n> [--branch-type <start|end>]
  workspace branch-files --codelab-id <id> --branch <name>
  workspace branch-read --codelab-id <id> --branch <name> --file <path>
  workspace branch-update --codelab-id <id> --branch <name> --files-json <path> [--delete-json <path>] [--commit-message <message>]
  workspace folders --codelab-id <id>
  workspace folder-create --codelab-id <id> --step-number <n> --files-json <path> [--folder-type <start|end>]
  workspace folder-files --codelab-id <id> --folder <name>
  workspace folder-read --codelab-id <id> --folder <name> --file <path>
  workspace folder-update --codelab-id <id> --folder <name> --files-json <path> [--delete-json <path>]
  attendee join --codelab-id <id> --name <name> --code <code> [--email <email>]
  attendee list --codelab-id <id>
  attendee complete --codelab-id <id>
  attendee certificate [--attendee-id <id>]
  help request --codelab-id <id> --step-number <n>
  help list --codelab-id <id>
  help resolve --codelab-id <id> --help-id <id>
  feedback submit --codelab-id <id> --difficulty <1-5> --satisfaction <1-5> [--comment <text>]
  feedback list --codelab-id <id>
  materials list --codelab-id <id>
  materials upload --file <path>
  materials add --codelab-id <id> --title <title> --type <link|file> [--url <url>] [--file-path <path>]
  materials delete --codelab-id <id> --material-id <id>
  quiz list --codelab-id <id>
  quiz update --codelab-id <id> --file <json>
  quiz submit --codelab-id <id> --file <json>
  quiz submissions --codelab-id <id>
  submission list --codelab-id <id>
  submission file --codelab-id <id> [--attendee-id <id>] --file <path>
  submission link --codelab-id <id> [--attendee-id <id>] --url <url> [--title <title>]
  submission delete --codelab-id <id> [--attendee-id <id>] --submission-id <id>
  chat history --codelab-id <id>
  upload image --file <path>
  inline list --codelab-id <id> [--target-type <guide|step>] [--target-step-id <id>]
  inline create --codelab-id <id> --file <json>
  inline reply --codelab-id <id> --thread-id <id> --file <json>
  inline delete --codelab-id <id> --thread-id <id> --comment-id <id>
  ai conversations --codelab-id <id>
  ai stream --file <json>
  ai save --file <json>
  ai threads
  ai thread-create --title <title> [--codelab-id <id>]
  ai thread-delete --thread-id <id>
  ai messages --thread-id <id>
  ai message-add --thread-id <id> --file <json>

Environment:
  OPEN_CODELABS_BASE_URL
  OPEN_CODELABS_ADMIN_ID
  OPEN_CODELABS_ADMIN_PW
  OPEN_CODELABS_CONFIG_FILE
  OPEN_CODELABS_PROFILE
  OPEN_CODELABS_SESSION_FILE
"#
    );
}

fn program_name() -> String {
    env::args()
        .next()
        .and_then(|value| {
            std::path::Path::new(&value)
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "oc".to_string())
}

#[derive(Debug)]
struct Args {
    items: Vec<String>,
    index: usize,
}

impl Args {
    fn new(items: Vec<String>) -> Self {
        Self { items, index: 0 }
    }

    fn next(&mut self) -> Option<String> {
        let item = self.items.get(self.index).cloned();
        if item.is_some() {
            self.index += 1;
        }
        item
    }

    fn peek(&self) -> Option<&str> {
        self.items.get(self.index).map(String::as_str)
    }

    fn next_required(&mut self, flag: &str) -> Result<String> {
        self.next()
            .ok_or_else(|| anyhow!("Missing value for {flag}"))
    }

    fn ensure_exhausted(&self) -> Result<()> {
        if let Some(unexpected) = self.peek() {
            bail!("Unexpected argument: {unexpected}");
        }
        Ok(())
    }
}
