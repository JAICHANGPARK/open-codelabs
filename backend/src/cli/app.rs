use crate::api::dto::{CodeServerInfo, CreateCodeServerRequest, WorkspaceFile};
use crate::cli::client::{ApiClient, BackupSummary};
use crate::cli::session::{
    clear_session, default_session_path, load_session, save_session, SessionSnapshot, StoredSession,
};
use crate::domain::models::{Codelab, CreateCodelab, CreateStep, Step, UpdateStepsPayload};
use crate::infrastructure::db_models::AuditLog;
use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct GlobalOptions {
    base_url: Option<String>,
    session_file: PathBuf,
    json: bool,
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self {
            base_url: env::var("OPEN_CODELABS_BASE_URL").ok(),
            session_file: default_session_path(),
            json: false,
        }
    }
}

#[derive(Debug)]
enum Command {
    Help,
    Login(LoginCommand),
    Logout,
    Session,
    Codelab(CodelabCommand),
    Backup(BackupCommand),
    Audit(AuditCommand),
    Workspace(WorkspaceCommand),
}

#[derive(Debug)]
struct LoginCommand {
    admin_id: String,
    admin_pw: String,
}

#[derive(Debug)]
enum CodelabCommand {
    List,
    Get { id: String },
    Create(CreateCodelabCommand),
    Export { id: String, output: Option<PathBuf> },
    Import { file: PathBuf },
    PushSteps { id: String, file: PathBuf },
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
    Folders {
        codelab_id: String,
    },
    FolderCreate {
        codelab_id: String,
        step_number: i32,
        folder_type: String,
        files_json: PathBuf,
    },
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
        Command::Login(command) => {
            let base_url = resolve_base_url(global.base_url.as_deref(), None);
            let client = ApiClient::new(base_url, None)?;
            let session = client
                .login_admin(&command.admin_id, &command.admin_pw)
                .await?;
            save_session(&global.session_file, &session)?;

            if global.json {
                print_json(&session)?;
            } else {
                println!("Saved admin session to {}", global.session_file.display());
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
            let session = load_session(&global.session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(global.base_url.as_deref(), Some(&session));
            let client = ApiClient::new(base_url, Some(session))?;
            let logout_result = client.logout().await;
            clear_session(&global.session_file)?;
            logout_result?;

            if global.json {
                print_json(&serde_json::json!({ "status": "ok" }))?;
            } else {
                println!("Logged out and removed {}", global.session_file.display());
            }
        }
        Command::Session => {
            let session = load_session(&global.session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(global.base_url.as_deref(), Some(&session));
            let client = ApiClient::new(base_url, Some(session.clone()))?;
            let snapshot = client.session().await?;

            let mut updated = session;
            updated.apply_snapshot(&snapshot);
            save_session(&global.session_file, &updated)?;

            if global.json {
                print_json(&snapshot)?;
            } else {
                print_session(&snapshot);
            }
        }
        Command::Codelab(command) => {
            let session = load_session(&global.session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(global.base_url.as_deref(), Some(&session));
            let client = ApiClient::new(base_url, Some(session))?;
            run_codelab_command(&global, &client, command).await?;
        }
        Command::Backup(command) => {
            let session = load_session(&global.session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(global.base_url.as_deref(), Some(&session));
            let client = ApiClient::new(base_url, Some(session))?;
            run_backup_command(&global, &client, command).await?;
        }
        Command::Audit(command) => {
            let session = load_session(&global.session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(global.base_url.as_deref(), Some(&session));
            let client = ApiClient::new(base_url, Some(session))?;
            run_audit_command(&global, &client, command).await?;
        }
        Command::Workspace(command) => {
            let session = load_session(&global.session_file).with_context(|| {
                format!(
                    "No saved session found. Run `{}` login first.",
                    program_name()
                )
            })?;
            let base_url = resolve_base_url(global.base_url.as_deref(), Some(&session));
            let client = ApiClient::new(base_url, Some(session))?;
            run_workspace_command(&global, &client, command).await?;
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
            let guide_markdown = match command.guide_file {
                Some(path) => Some(
                    tokio::fs::read_to_string(&path)
                        .await
                        .with_context(|| format!("Failed to read {}", path.display()))?,
                ),
                None => None,
            };

            let codelab = client
                .create_codelab(&CreateCodelab {
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
                .await?;

            if global.json {
                print_json(&codelab)?;
            } else {
                println!("Created codelab {}", codelab.id);
                println!("title: {}", codelab.title);
                println!("author: {}", codelab.author);
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

fn resolve_base_url(explicit: Option<&str>, session: Option<&StoredSession>) -> String {
    if let Some(explicit) = explicit {
        return explicit.trim_end_matches('/').to_string();
    }
    if let Some(session) = session {
        return session.base_url.trim_end_matches('/').to_string();
    }
    env::var("OPEN_CODELABS_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
        .trim_end_matches('/')
        .to_string()
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
    let mut args = Args::new(env::args().skip(1).collect());
    let mut global = GlobalOptions::default();

    loop {
        match args.peek() {
            Some("--base-url") => {
                args.next();
                global.base_url = Some(args.next_required("--base-url")?);
            }
            Some("--session-file") => {
                args.next();
                global.session_file = PathBuf::from(args.next_required("--session-file")?);
            }
            Some("--json") => {
                args.next();
                global.json = true;
            }
            Some("-h") | Some("--help") => {
                return Ok((global, Command::Help));
            }
            _ => break,
        }
    }

    let Some(command) = args.next() else {
        return Ok((global, Command::Help));
    };

    let command = match command.as_str() {
        "login" => Command::Login(parse_login(&mut args)?),
        "logout" => Command::Logout,
        "session" => Command::Session,
        "codelab" => Command::Codelab(parse_codelab(&mut args)?),
        "backup" => Command::Backup(parse_backup(&mut args)?),
        "audit" => Command::Audit(parse_audit(&mut args)?),
        "workspace" => Command::Workspace(parse_workspace(&mut args)?),
        "help" => Command::Help,
        other => bail!("Unknown command: {other}"),
    };

    args.ensure_exhausted()?;
    Ok((global, command))
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
        "get" => Ok(CodelabCommand::Get {
            id: parse_required_string_flag(args, "--id", "codelab get")?,
        }),
        "create" => parse_create_codelab(args),
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
        _ => Err(help_error("workspace")),
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
  --session-file <path>   Session file path (default: ~/.open-codelabs/session.json)
  --json                  Print JSON instead of table/text output
  -h, --help              Show this help

Commands:
  login --admin-id <id> --admin-pw <pw>
  logout
  session
  codelab list
  codelab get --id <id>
  codelab create --title <title> --description <desc> --author <author> [--private] [--guide-file <path>] [--quiz-enabled] [--require-quiz] [--require-feedback] [--require-submission]
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
  workspace folders --codelab-id <id>
  workspace folder-create --codelab-id <id> --step-number <n> --files-json <path> [--folder-type <start|end>]

Environment:
  OPEN_CODELABS_BASE_URL
  OPEN_CODELABS_ADMIN_ID
  OPEN_CODELABS_ADMIN_PW
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
