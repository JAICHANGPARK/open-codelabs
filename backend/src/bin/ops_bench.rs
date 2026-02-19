use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use futures_util::{stream, StreamExt};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use rand::RngCore;
use reqwest::{header, multipart, Client, Method, StatusCode};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct Config {
    base_url: String,
    admin_id: String,
    admin_pw: String,
    profile: String,
    scenarios: Vec<String>,
    upload_requests_per_case: usize,
    codeserver_iterations: usize,
    output: String,
}

impl Config {
    fn defaults() -> Self {
        Self {
            base_url: env::var("BENCH_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            admin_id: env::var("ADMIN_ID").unwrap_or_else(|_| "admin".to_string()),
            admin_pw: env::var("ADMIN_PW").unwrap_or_else(|_| "admin".to_string()),
            profile: env::var("OPS_BENCH_PROFILE").unwrap_or_else(|_| "paper".to_string()),
            scenarios: vec![
                "upload".to_string(),
                "backup".to_string(),
                "codeserver".to_string(),
            ],
            upload_requests_per_case: env_usize("OPS_BENCH_UPLOAD_REQUESTS", 60),
            codeserver_iterations: env_usize("OPS_BENCH_CODESERVER_ITERATIONS", 8),
            output: env::var("OPS_BENCH_OUTPUT").unwrap_or_else(|_| {
                format!(
                    "bench-results/ops-bench-{}.json",
                    Utc::now().format("%Y%m%d-%H%M%S")
                )
            }),
        }
    }

    fn from_args() -> Result<Self> {
        let mut cfg = Self::defaults();
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "--base-url" => cfg.base_url = next_arg_value(&mut args, "--base-url")?,
                "--admin-id" => cfg.admin_id = next_arg_value(&mut args, "--admin-id")?,
                "--admin-pw" => cfg.admin_pw = next_arg_value(&mut args, "--admin-pw")?,
                "--profile" => cfg.profile = next_arg_value(&mut args, "--profile")?,
                "--scenarios" => {
                    cfg.scenarios = parse_csv(next_arg_value(&mut args, "--scenarios")?)
                }
                "--upload-requests" => {
                    cfg.upload_requests_per_case = parse_usize_arg(
                        &next_arg_value(&mut args, "--upload-requests")?,
                        "--upload-requests",
                    )?
                }
                "--codeserver-iterations" => {
                    cfg.codeserver_iterations = parse_usize_arg(
                        &next_arg_value(&mut args, "--codeserver-iterations")?,
                        "--codeserver-iterations",
                    )?
                }
                "--output" => cfg.output = next_arg_value(&mut args, "--output")?,
                _ => return Err(anyhow!("Unknown argument: {arg}")),
            }
        }

        cfg.base_url = cfg.base_url.trim_end_matches('/').to_string();
        if cfg.base_url.is_empty() {
            return Err(anyhow!("--base-url cannot be empty"));
        }
        if cfg.scenarios.is_empty() {
            return Err(anyhow!("--scenarios cannot be empty"));
        }
        let profile = cfg.profile.to_lowercase();
        if profile != "paper" && profile != "quick" {
            return Err(anyhow!("--profile must be one of: paper, quick"));
        }
        cfg.profile = profile;
        Ok(cfg)
    }
}

#[derive(Clone, Debug)]
struct SessionCookies {
    cookie_header: String,
    csrf_token: Option<String>,
}

#[derive(Clone, Debug)]
struct AttendeeSession {
    id: String,
    cookie_header: String,
    csrf_token: Option<String>,
}

#[derive(Debug)]
struct RequestSample {
    latency_ms: f64,
    status: Option<u16>,
    error: Option<String>,
}

#[derive(Serialize)]
struct OpsBenchReport {
    generated_at_utc: String,
    config: OpsBenchPublicConfig,
    upload: Option<UploadReport>,
    backup: Option<BackupReport>,
    codeserver: Option<CodeserverReport>,
    notes: Vec<String>,
}

#[derive(Serialize)]
struct OpsBenchPublicConfig {
    base_url: String,
    profile: String,
    scenarios: Vec<String>,
    upload_requests_per_case: usize,
    codeserver_iterations: usize,
    output: String,
}

#[derive(Serialize)]
struct UploadReport {
    cases: Vec<UploadCaseResult>,
}

#[derive(Serialize)]
struct UploadCaseResult {
    label: String,
    generated_image_bytes: usize,
    requests: usize,
    concurrency: usize,
    success_2xx: usize,
    http_errors: usize,
    transport_errors: usize,
    latency_ms: PercentileSummary,
    status_counts: HashMap<String, usize>,
}

#[derive(Serialize)]
struct BackupReport {
    datasets: Vec<BackupDatasetResult>,
}

#[derive(Serialize)]
struct BackupDatasetResult {
    dataset: String,
    codelab_id: String,
    attendees_seeded: usize,
    steps_seeded: usize,
    quizzes_seeded: usize,
    export_ms: f64,
    inspect_ms: f64,
    restore_ms: f64,
    backup_size_bytes: usize,
    export_status: u16,
    inspect_status: u16,
    restore_status: u16,
}

#[derive(Serialize)]
struct CodeserverReport {
    codelab_id: String,
    iterations: usize,
    create_codeserver_ms: f64,
    download_workspace_ms: f64,
    download_size_bytes: usize,
    create_branch_ms: PercentileSummary,
    update_branch_ms: PercentileSummary,
    create_folder_ms: PercentileSummary,
    update_folder_ms: PercentileSummary,
}

#[derive(Serialize)]
struct PercentileSummary {
    samples: usize,
    min: f64,
    mean: f64,
    p50: f64,
    p95: f64,
    p99: f64,
    max: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_args()?;
    let client = Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        .build()
        .context("Failed to build reqwest client")?;

    let mut upload = None;
    let mut backup = None;
    let mut codeserver = None;

    if has_scenario(&cfg.scenarios, "upload") {
        println!("Running upload benchmark...");
        upload = Some(run_upload_benchmark(&client, &cfg).await?);
    }
    if has_scenario(&cfg.scenarios, "backup") {
        println!("Running backup benchmark...");
        backup = Some(run_backup_benchmark(&client, &cfg).await?);
    }
    if has_scenario(&cfg.scenarios, "codeserver") {
        println!("Running code-server benchmark...");
        codeserver = Some(run_codeserver_benchmark(&client, &cfg).await?);
    }

    let report = OpsBenchReport {
        generated_at_utc: Utc::now().to_rfc3339(),
        config: OpsBenchPublicConfig {
            base_url: cfg.base_url.clone(),
            profile: cfg.profile.clone(),
            scenarios: cfg.scenarios.clone(),
            upload_requests_per_case: cfg.upload_requests_per_case,
            codeserver_iterations: cfg.codeserver_iterations,
            output: cfg.output.clone(),
        },
        upload,
        backup,
        codeserver,
        notes: vec![
            "Upload benchmark uses PNG noise images (server converts to WebP).".to_string(),
            "Backup benchmark seeds synthetic workshop data before export/inspect/restore."
                .to_string(),
            "Code-server benchmark measures branch/folder update path plus workspace download."
                .to_string(),
        ],
    };

    write_json(&cfg.output, &report)?;
    println!("Ops benchmark complete: {}", cfg.output);
    Ok(())
}

async fn run_upload_benchmark(client: &Client, cfg: &Config) -> Result<UploadReport> {
    let admin = login_admin(client, cfg).await?;
    let codelab_id = create_codelab(client, cfg, &admin, "Upload Bench").await?;
    let attendee = register_attendee(
        client,
        &cfg.base_url,
        &codelab_id,
        "upload-user",
        "upload-code",
    )
    .await?;

    let image_plans = if cfg.profile == "quick" {
        vec![
            UploadPlan {
                label: "img_1mb".to_string(),
                width: 700,
                height: 700,
                concurrency: vec![2, 4],
            },
            UploadPlan {
                label: "img_3mb".to_string(),
                width: 1200,
                height: 900,
                concurrency: vec![2, 4],
            },
        ]
    } else {
        vec![
            UploadPlan {
                label: "img_1mb".to_string(),
                width: 700,
                height: 700,
                concurrency: vec![5, 10, 20],
            },
            UploadPlan {
                label: "img_3mb".to_string(),
                width: 1200,
                height: 900,
                concurrency: vec![5, 10, 20],
            },
            UploadPlan {
                label: "img_5mb".to_string(),
                width: 1500,
                height: 1100,
                concurrency: vec![5, 10, 20],
            },
        ]
    };

    let mut cases = Vec::new();
    for plan in image_plans {
        let image_data = generate_png_noise(plan.width, plan.height, 4_900_000)?;
        for concurrency in plan.concurrency {
            let requests = cfg.upload_requests_per_case.max(concurrency * 3);
            let samples = run_upload_case(
                client,
                &cfg.base_url,
                &attendee,
                image_data.clone(),
                requests,
                concurrency,
            )
            .await;
            cases.push(summarize_upload_case(
                &plan.label,
                image_data.len(),
                requests,
                concurrency,
                &samples,
            ));
        }
    }

    Ok(UploadReport { cases })
}

async fn run_backup_benchmark(client: &Client, cfg: &Config) -> Result<BackupReport> {
    let admin = login_admin(client, cfg).await?;
    let datasets = if cfg.profile == "quick" {
        vec![BackupSeedPlan::tiny(), BackupSeedPlan::small()]
    } else {
        vec![
            BackupSeedPlan::small(),
            BackupSeedPlan::medium(),
            BackupSeedPlan::large(),
        ]
    };

    let mut results = Vec::new();
    for plan in datasets {
        let seeded = seed_backup_dataset(client, cfg, &admin, &plan).await?;
        let export_res = timed_request(|| async {
            send_authed(
                client,
                Method::GET,
                &format!("{}/api/admin/backup/export", cfg.base_url),
                &admin,
                None,
            )
            .await
        })
        .await?;
        let export_status = export_res.1.status().as_u16();
        let backup_bytes = export_res.1.bytes().await.unwrap_or_default().to_vec();

        let inspect_res = timed_request(|| async {
            post_backup_file(
                client,
                &cfg.base_url,
                "/api/admin/backup/inspect",
                &admin,
                backup_bytes.clone(),
            )
            .await
        })
        .await?;
        let inspect_status = inspect_res.1.status().as_u16();

        let restore_res = timed_request(|| async {
            post_backup_file(
                client,
                &cfg.base_url,
                "/api/admin/backup/restore",
                &admin,
                backup_bytes.clone(),
            )
            .await
        })
        .await?;
        let restore_status = restore_res.1.status().as_u16();

        results.push(BackupDatasetResult {
            dataset: plan.name.clone(),
            codelab_id: seeded.codelab_id,
            attendees_seeded: plan.attendees,
            steps_seeded: plan.steps,
            quizzes_seeded: plan.quiz_count,
            export_ms: export_res.0,
            inspect_ms: inspect_res.0,
            restore_ms: restore_res.0,
            backup_size_bytes: backup_bytes.len(),
            export_status,
            inspect_status,
            restore_status,
        });
    }

    Ok(BackupReport { datasets: results })
}

async fn run_codeserver_benchmark(client: &Client, cfg: &Config) -> Result<CodeserverReport> {
    let admin = login_admin(client, cfg).await?;
    let codelab_id = create_codelab(client, cfg, &admin, "CodeServer Bench").await?;
    let seed_files = vec![
        json!({"path":"README.md","content":"# Bench"}),
        json!({"path":"src/main.rs","content":"fn main(){println!(\"bench\");}"}),
    ];

    let create_codeserver = timed_request(|| async {
        send_authed(
            client,
            Method::POST,
            &format!("{}/api/codeserver", cfg.base_url),
            &admin,
            Some(json!({
                "codelab_id": codelab_id,
                "workspace_files": seed_files,
                "structure_type": "branch"
            })),
        )
        .await
    })
    .await?;
    ensure_success(create_codeserver.1.status(), "create codeserver")?;

    let mut create_branch_samples = Vec::new();
    let mut update_branch_samples = Vec::new();
    let mut create_folder_samples = Vec::new();
    let mut update_folder_samples = Vec::new();

    for i in 1..=cfg.codeserver_iterations {
        let branch_name = format!("step-{}-start", i);
        let folder_name = format!("step-{}-start", i);

        let create_branch = timed_request(|| async {
            send_authed(
                client,
                Method::POST,
                &format!("{}/api/codeserver/{}/branch", cfg.base_url, codelab_id),
                &admin,
                Some(json!({ "step_number": i as i32, "branch_type": "start" })),
            )
            .await
        })
        .await?;
        create_branch_samples.push(create_branch.0);
        ensure_success(create_branch.1.status(), "create branch")?;

        let update_branch = timed_request(|| async {
            send_authed(
                client,
                Method::POST,
                &format!(
                    "{}/api/codeserver/{}/branches/{}/files",
                    cfg.base_url, codelab_id, branch_name
                ),
                &admin,
                Some(json!({
                    "files":[{"path":"src/iter.rs","content":format!("pub const I: i32 = {};", i)}],
                    "commit_message": format!("bench iter {}", i)
                })),
            )
            .await
        })
        .await?;
        update_branch_samples.push(update_branch.0);
        ensure_success(update_branch.1.status(), "update branch files")?;

        let create_folder = timed_request(|| async {
            send_authed(
                client,
                Method::POST,
                &format!("{}/api/codeserver/{}/folder", cfg.base_url, codelab_id),
                &admin,
                Some(json!({
                    "step_number": i as i32,
                    "folder_type": "start",
                    "files": [{"path":"index.txt","content":format!("folder {}", i)}]
                })),
            )
            .await
        })
        .await?;
        create_folder_samples.push(create_folder.0);
        ensure_success(create_folder.1.status(), "create folder")?;

        let update_folder = timed_request(|| async {
            send_authed(
                client,
                Method::POST,
                &format!(
                    "{}/api/codeserver/{}/folders/{}/files",
                    cfg.base_url, codelab_id, folder_name
                ),
                &admin,
                Some(json!({
                    "files":[{"path":"updated.txt","content":format!("updated {}", i)}]
                })),
            )
            .await
        })
        .await?;
        update_folder_samples.push(update_folder.0);
        ensure_success(update_folder.1.status(), "update folder files")?;
    }

    let download_workspace = timed_request(|| async {
        send_authed(
            client,
            Method::GET,
            &format!("{}/api/codeserver/{}/download", cfg.base_url, codelab_id),
            &admin,
            None,
        )
        .await
    })
    .await?;
    ensure_success(download_workspace.1.status(), "download workspace")?;
    let archive = download_workspace
        .1
        .bytes()
        .await
        .unwrap_or_default()
        .to_vec();

    Ok(CodeserverReport {
        codelab_id,
        iterations: cfg.codeserver_iterations,
        create_codeserver_ms: create_codeserver.0,
        download_workspace_ms: download_workspace.0,
        download_size_bytes: archive.len(),
        create_branch_ms: summarize_f64_samples(create_branch_samples),
        update_branch_ms: summarize_f64_samples(update_branch_samples),
        create_folder_ms: summarize_f64_samples(create_folder_samples),
        update_folder_ms: summarize_f64_samples(update_folder_samples),
    })
}

async fn run_upload_case(
    client: &Client,
    base_url: &str,
    attendee: &AttendeeSession,
    image_data: Vec<u8>,
    requests: usize,
    concurrency: usize,
) -> Vec<RequestSample> {
    let client = client.clone();
    let base_url = base_url.to_string();
    let attendee = attendee.clone();
    let image_data = Arc::new(image_data);

    stream::iter(0..requests)
        .map(move |idx| {
            let client = client.clone();
            let base_url = base_url.clone();
            let attendee = attendee.clone();
            let image_data = image_data.clone();
            async move {
                let started = Instant::now();
                let part = multipart::Part::bytes(image_data.as_ref().clone())
                    .file_name(format!("upload-{idx}.png"))
                    .mime_str("image/png")
                    .unwrap_or_else(|_| multipart::Part::bytes(image_data.as_ref().clone()));
                let form = multipart::Form::new().part("file", part);

                let mut req = client
                    .post(format!("{}/api/upload/image", base_url))
                    .header(header::COOKIE, attendee.cookie_header.clone());
                if let Some(csrf) = &attendee.csrf_token {
                    req = req.header("x-csrf-token", csrf);
                }

                match req.multipart(form).send().await {
                    Ok(res) => RequestSample {
                        latency_ms: started.elapsed().as_secs_f64() * 1000.0,
                        status: Some(res.status().as_u16()),
                        error: None,
                    },
                    Err(err) => RequestSample {
                        latency_ms: started.elapsed().as_secs_f64() * 1000.0,
                        status: None,
                        error: Some(err.to_string()),
                    },
                }
            }
        })
        .buffer_unordered(concurrency.max(1))
        .collect::<Vec<_>>()
        .await
}

fn summarize_upload_case(
    label: &str,
    generated_image_bytes: usize,
    requests: usize,
    concurrency: usize,
    samples: &[RequestSample],
) -> UploadCaseResult {
    let mut status_counts = HashMap::new();
    let mut success_2xx = 0usize;
    let mut http_errors = 0usize;
    let mut transport_errors = 0usize;
    let mut latencies = Vec::with_capacity(samples.len());

    for sample in samples {
        latencies.push(sample.latency_ms);
        if let Some(status) = sample.status {
            *status_counts.entry(status.to_string()).or_insert(0) += 1;
            if (200..300).contains(&status) {
                success_2xx += 1;
            } else {
                http_errors += 1;
            }
        } else {
            transport_errors += 1;
            *status_counts
                .entry("transport_error".to_string())
                .or_insert(0) += 1;
            if let Some(err) = &sample.error {
                *status_counts.entry(format!("err:{err}")).or_insert(0) += 1;
            }
        }
    }

    UploadCaseResult {
        label: label.to_string(),
        generated_image_bytes,
        requests,
        concurrency,
        success_2xx,
        http_errors,
        transport_errors,
        latency_ms: summarize_f64_samples(latencies),
        status_counts,
    }
}

async fn seed_backup_dataset(
    client: &Client,
    cfg: &Config,
    admin: &SessionCookies,
    plan: &BackupSeedPlan,
) -> Result<SeedResult> {
    let codelab_id = create_codelab(client, cfg, admin, &format!("Backup {}", plan.name)).await?;

    let steps = (1..=plan.steps)
        .map(|n| {
            json!({
                "id": Value::Null,
                "title": format!("Step {n}"),
                "content_markdown": format!("# Step {n}\n\nseed {}", plan.name)
            })
        })
        .collect::<Vec<_>>();
    let steps_res = send_authed(
        client,
        Method::PUT,
        &format!("{}/api/codelabs/{}/steps", cfg.base_url, codelab_id),
        admin,
        Some(json!({ "steps": steps })),
    )
    .await?;
    ensure_success(steps_res.status(), "seed steps")?;

    let quizzes = (0..plan.quiz_count)
        .map(|i| {
            json!({
                "question": format!("Q{}?", i + 1),
                "quiz_type": "multiple_choice",
                "options": ["A","B","C","D"],
                "correct_answer": 1,
                "correct_answers": [1]
            })
        })
        .collect::<Vec<_>>();
    let quizzes_res = send_authed(
        client,
        Method::PUT,
        &format!("{}/api/codelabs/{}/quizzes", cfg.base_url, codelab_id),
        admin,
        Some(Value::Array(quizzes)),
    )
    .await?;
    ensure_success(quizzes_res.status(), "seed quizzes")?;

    let quiz_list_res = send_authed(
        client,
        Method::GET,
        &format!("{}/api/codelabs/{}/quizzes", cfg.base_url, codelab_id),
        admin,
        None,
    )
    .await?;
    ensure_success(quiz_list_res.status(), "get quizzes")?;
    let quiz_body = quiz_list_res.text().await.unwrap_or_default();
    let quiz_ids = serde_json::from_str::<Vec<Value>>(&quiz_body)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|q| q.get("id").and_then(Value::as_str).map(|s| s.to_string()))
        .collect::<Vec<_>>();

    for i in 0..plan.attendees {
        let attendee = register_attendee(
            client,
            &cfg.base_url,
            &codelab_id,
            &format!("backup-user-{:04}", i + 1),
            &format!("backup-code-{:04}", i + 1),
        )
        .await?;

        for j in 0..plan.helps_per_attendee {
            let help_res = send_attendee_authed(
                client,
                Method::POST,
                &format!("{}/api/codelabs/{}/help", cfg.base_url, codelab_id),
                &attendee,
                Some(json!({ "step_number": ((j % plan.steps) + 1) as i32 })),
            )
            .await?;
            ensure_success(help_res.status(), "seed help request")?;
        }

        for j in 0..plan.submissions_per_attendee {
            let link_res = send_attendee_authed(
                client,
                Method::POST,
                &format!(
                    "{}/api/codelabs/{}/attendees/{}/submissions/link",
                    cfg.base_url, codelab_id, attendee.id
                ),
                &attendee,
                Some(json!({
                    "url": format!("https://example.com/backup/{}/{}", i + 1, j + 1),
                    "title": format!("Backup Link {}-{}", i + 1, j + 1)
                })),
            )
            .await?;
            ensure_success(link_res.status(), "seed submission link")?;
        }

        if i < plan.feedback_attendees {
            let feedback_res = send_attendee_authed(
                client,
                Method::POST,
                &format!("{}/api/codelabs/{}/feedback", cfg.base_url, codelab_id),
                &attendee,
                Some(json!({
                    "difficulty": "3",
                    "satisfaction": "5",
                    "comment": "seed"
                })),
            )
            .await?;
            if feedback_res.status() != StatusCode::OK
                && feedback_res.status() != StatusCode::CONFLICT
            {
                return Err(anyhow!("seed feedback failed: {}", feedback_res.status()));
            }
        }

        if !quiz_ids.is_empty() {
            let submissions = quiz_ids
                .iter()
                .map(|id| {
                    json!({
                        "quiz_id": id,
                        "answer": "1",
                        "is_correct": true
                    })
                })
                .collect::<Vec<_>>();
            let quiz_submit = send_attendee_authed(
                client,
                Method::POST,
                &format!(
                    "{}/api/codelabs/{}/quizzes/submit",
                    cfg.base_url, codelab_id
                ),
                &attendee,
                Some(json!({ "submissions": submissions })),
            )
            .await?;
            ensure_success(quiz_submit.status(), "seed quiz submission")?;
        }
    }

    Ok(SeedResult { codelab_id })
}

async fn post_backup_file(
    client: &Client,
    base_url: &str,
    path: &str,
    admin: &SessionCookies,
    data: Vec<u8>,
) -> Result<reqwest::Response> {
    let part = multipart::Part::bytes(data)
        .file_name("backup_full.zip")
        .mime_str("application/zip")
        .unwrap_or_else(|_| multipart::Part::bytes(Vec::new()).file_name("backup_full.zip"));
    let form = multipart::Form::new().part("file", part);

    let mut req = client
        .post(format!("{base_url}{path}"))
        .header(header::COOKIE, admin.cookie_header.clone());
    if let Some(csrf) = &admin.csrf_token {
        req = req.header("x-csrf-token", csrf);
    }
    let response = req
        .multipart(form)
        .send()
        .await
        .with_context(|| format!("Failed calling {path}"))?;
    Ok(response)
}

async fn timed_request<F, Fut>(f: F) -> Result<(f64, reqwest::Response)>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<reqwest::Response>>,
{
    let started = Instant::now();
    let res = f().await?;
    Ok((started.elapsed().as_secs_f64() * 1000.0, res))
}

async fn send_authed(
    client: &Client,
    method: Method,
    url: &str,
    admin: &SessionCookies,
    body: Option<Value>,
) -> Result<reqwest::Response> {
    let mut req = client
        .request(method.clone(), url)
        .header(header::COOKIE, admin.cookie_header.clone());
    if method != Method::GET {
        if let Some(csrf) = &admin.csrf_token {
            req = req.header("x-csrf-token", csrf);
        }
    }
    if let Some(body) = body {
        req = req
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body);
    }
    let response = req
        .send()
        .await
        .with_context(|| format!("Request failed: {url}"))?;
    Ok(response)
}

async fn send_attendee_authed(
    client: &Client,
    method: Method,
    url: &str,
    attendee: &AttendeeSession,
    body: Option<Value>,
) -> Result<reqwest::Response> {
    let mut req = client
        .request(method.clone(), url)
        .header(header::COOKIE, attendee.cookie_header.clone());
    if method != Method::GET {
        if let Some(csrf) = &attendee.csrf_token {
            req = req.header("x-csrf-token", csrf);
        }
    }
    if let Some(body) = body {
        req = req
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body);
    }
    let response = req
        .send()
        .await
        .with_context(|| format!("Request failed: {url}"))?;
    Ok(response)
}

async fn login_admin(client: &Client, cfg: &Config) -> Result<SessionCookies> {
    let response = client
        .post(format!("{}/api/login", cfg.base_url))
        .json(&json!({
            "admin_id": cfg.admin_id,
            "admin_pw": cfg.admin_pw
        }))
        .send()
        .await
        .context("Failed to call /api/login")?;
    build_session_from_response(response, "/api/login").await
}

async fn create_codelab(
    client: &Client,
    cfg: &Config,
    admin: &SessionCookies,
    title_prefix: &str,
) -> Result<String> {
    let mut req = client
        .post(format!("{}/api/codelabs", cfg.base_url))
        .header(header::COOKIE, admin.cookie_header.clone());
    if let Some(csrf) = &admin.csrf_token {
        req = req.header("x-csrf-token", csrf);
    }
    let response = req
        .json(&json!({
            "title": format!("{title_prefix} {}", Utc::now().format("%Y-%m-%d %H:%M:%S")),
            "description": "auto-generated benchmark codelab",
            "author": "ops-bench",
            "is_public": true,
            "quiz_enabled": true,
            "require_quiz": false,
            "require_feedback": false,
            "require_submission": false,
            "guide_markdown": "# Bench"
        }))
        .send()
        .await
        .context("Failed to call /api/codelabs")?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    ensure_success(status, "create codelab")?;
    let id = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|v| v.get("id").and_then(Value::as_str).map(|s| s.to_string()))
        .ok_or_else(|| anyhow!("Create codelab response missing id"))?;
    Ok(id)
}

async fn register_attendee(
    client: &Client,
    base_url: &str,
    codelab_id: &str,
    name: &str,
    code: &str,
) -> Result<AttendeeSession> {
    let response = client
        .post(format!("{}/api/codelabs/{}/register", base_url, codelab_id))
        .json(&json!({ "name": name, "code": code, "email": Value::Null }))
        .send()
        .await
        .with_context(|| format!("Failed to register attendee {name}"))?;
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await.unwrap_or_default();
    ensure_success(status, "register attendee")?;
    let id = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|v| v.get("id").and_then(Value::as_str).map(|s| s.to_string()))
        .ok_or_else(|| anyhow!("register attendee response missing id"))?;
    let cookie_map = parse_set_cookie_headers(&headers);
    if cookie_map.is_empty() {
        return Err(anyhow!("register attendee did not return cookies"));
    }
    Ok(AttendeeSession {
        id,
        cookie_header: build_cookie_header(&cookie_map),
        csrf_token: find_csrf_token(&cookie_map),
    })
}

async fn build_session_from_response(
    response: reqwest::Response,
    context_path: &str,
) -> Result<SessionCookies> {
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(
            "{context_path} failed: HTTP {} body={}",
            status.as_u16(),
            truncate(&body, 160)
        ));
    }
    let cookie_map = parse_set_cookie_headers(&headers);
    if cookie_map.is_empty() {
        return Err(anyhow!("No Set-Cookie headers from {context_path}"));
    }
    Ok(SessionCookies {
        cookie_header: build_cookie_header(&cookie_map),
        csrf_token: find_csrf_token(&cookie_map),
    })
}

fn parse_set_cookie_headers(headers: &header::HeaderMap) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for value in headers.get_all(header::SET_COOKIE).iter() {
        let Ok(raw) = value.to_str() else { continue };
        let Some(first) = raw.split(';').next() else {
            continue;
        };
        let Some((name, value)) = first.split_once('=') else {
            continue;
        };
        map.insert(name.trim().to_string(), value.trim().to_string());
    }
    map
}

fn build_cookie_header(map: &HashMap<String, String>) -> String {
    let mut pairs = map
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>();
    pairs.sort();
    pairs.join("; ")
}

fn find_csrf_token(cookies: &HashMap<String, String>) -> Option<String> {
    cookies
        .iter()
        .find(|(k, _)| k.ends_with("oc_csrf"))
        .map(|(_, v)| v.clone())
}

fn summarize_f64_samples(mut values: Vec<f64>) -> PercentileSummary {
    if values.is_empty() {
        return PercentileSummary {
            samples: 0,
            min: 0.0,
            mean: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
            max: 0.0,
        };
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    PercentileSummary {
        samples: values.len(),
        min: *values.first().unwrap_or(&0.0),
        mean,
        p50: percentile(&values, 0.50),
        p95: percentile(&values, 0.95),
        p99: percentile(&values, 0.99),
        max: *values.last().unwrap_or(&0.0),
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let p = p.clamp(0.0, 1.0);
    let max_idx = sorted.len() - 1;
    let rank = p * max_idx as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let w = rank - lo as f64;
        sorted[lo] * (1.0 - w) + sorted[hi] * w
    }
}

fn generate_png_noise(mut width: usize, mut height: usize, max_bytes: usize) -> Result<Vec<u8>> {
    loop {
        let mut pixels = vec![0u8; width * height * 3];
        rand::thread_rng().fill_bytes(&mut pixels);
        let mut out = Vec::new();
        {
            let encoder = PngEncoder::new(&mut out);
            encoder
                .write_image(
                    &pixels,
                    width as u32,
                    height as u32,
                    ExtendedColorType::Rgb8,
                )
                .context("PNG encode failed")?;
        }
        if out.len() <= max_bytes {
            return Ok(out);
        }
        width = ((width as f64) * 0.92).max(64.0) as usize;
        height = ((height as f64) * 0.92).max(64.0) as usize;
        if width <= 64 || height <= 64 {
            return Ok(out);
        }
    }
}

fn has_scenario(scenarios: &[String], name: &str) -> bool {
    scenarios
        .iter()
        .any(|s| s.eq_ignore_ascii_case(name) || s.eq_ignore_ascii_case("all"))
}

fn parse_csv(raw: String) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
}

fn ensure_success(status: StatusCode, label: &str) -> Result<()> {
    if status.is_success() {
        Ok(())
    } else {
        Err(anyhow!("{label} failed with HTTP {}", status.as_u16()))
    }
}

fn write_json<T: Serialize>(path: &str, payload: &T) -> Result<()> {
    let output = PathBuf::from(path);
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed creating output directory {}", parent.display())
            })?;
        }
    }
    let content = serde_json::to_string_pretty(payload).context("Failed serializing JSON")?;
    fs::write(&output, content).with_context(|| format!("Failed writing {}", output.display()))?;
    Ok(())
}

fn env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_usize_arg(value: &str, flag: &str) -> Result<usize> {
    value
        .parse::<usize>()
        .with_context(|| format!("Invalid value for {flag}: {value}"))
}

fn next_arg_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("Missing value for {flag}"))
}

fn truncate(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }
    let mut out = value.chars().take(max_len).collect::<String>();
    out.push_str("...");
    out
}

#[derive(Clone)]
struct UploadPlan {
    label: String,
    width: usize,
    height: usize,
    concurrency: Vec<usize>,
}

#[derive(Clone)]
struct BackupSeedPlan {
    name: String,
    steps: usize,
    attendees: usize,
    helps_per_attendee: usize,
    submissions_per_attendee: usize,
    feedback_attendees: usize,
    quiz_count: usize,
}

impl BackupSeedPlan {
    fn tiny() -> Self {
        Self {
            name: "tiny".to_string(),
            steps: 4,
            attendees: 4,
            helps_per_attendee: 1,
            submissions_per_attendee: 1,
            feedback_attendees: 2,
            quiz_count: 2,
        }
    }

    fn small() -> Self {
        Self {
            name: "small".to_string(),
            steps: 10,
            attendees: 20,
            helps_per_attendee: 1,
            submissions_per_attendee: 1,
            feedback_attendees: 10,
            quiz_count: 3,
        }
    }

    fn medium() -> Self {
        Self {
            name: "medium".to_string(),
            steps: 30,
            attendees: 80,
            helps_per_attendee: 2,
            submissions_per_attendee: 2,
            feedback_attendees: 40,
            quiz_count: 6,
        }
    }

    fn large() -> Self {
        Self {
            name: "large".to_string(),
            steps: 60,
            attendees: 160,
            helps_per_attendee: 3,
            submissions_per_attendee: 2,
            feedback_attendees: 80,
            quiz_count: 8,
        }
    }
}

struct SeedResult {
    codelab_id: String,
}

fn print_help() {
    println!(
        r#"Operations benchmark (upload/backup/code-server).

Usage:
  cargo run --release --bin ops_bench -- [options]

Options:
  --base-url <url>               Backend URL (default: http://localhost:8080)
  --admin-id <id>                Admin ID (default: env ADMIN_ID or "admin")
  --admin-pw <pw>                Admin password (default: env ADMIN_PW or "admin")
  --profile <paper|quick>        Workload profile (default: paper)
  --scenarios <csv>              upload,backup,codeserver,all (default: upload,backup,codeserver)
  --upload-requests <n>          Requests per upload case (default: 60)
  --codeserver-iterations <n>    Iterations per branch/folder loop (default: 8)
  --output <path>                JSON output path
  -h, --help                     Show this help

Examples:
  cargo run --release --bin ops_bench -- --scenarios upload,backup --output bench-results/ops.json
  cargo run --release --bin ops_bench -- --scenarios codeserver --codeserver-iterations 20
"#
    );
}
