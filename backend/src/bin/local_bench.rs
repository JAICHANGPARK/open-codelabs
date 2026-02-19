use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use futures_util::{future::BoxFuture, stream, StreamExt};
use reqwest::{header, Client, Method};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct SessionCookies {
    cookie_header: String,
    csrf_token: Option<String>,
}

#[derive(Clone, Debug)]
struct AttendeeSession {
    attendee_id: String,
    cookies: SessionCookies,
}

#[derive(Clone, Debug)]
struct Config {
    base_url: String,
    admin_id: String,
    admin_pw: String,
    attendees: usize,
    read_requests: usize,
    write_requests: usize,
    read_concurrency: usize,
    write_concurrency: usize,
    output: String,
}

impl Config {
    fn defaults() -> Self {
        Self {
            base_url: env::var("BENCH_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            admin_id: env::var("ADMIN_ID").unwrap_or_else(|_| "admin".to_string()),
            admin_pw: env::var("ADMIN_PW").unwrap_or_else(|_| "admin".to_string()),
            attendees: env_usize("BENCH_ATTENDEES", 20),
            read_requests: env_usize("BENCH_READ_REQUESTS", 200),
            write_requests: env_usize("BENCH_WRITE_REQUESTS", 100),
            read_concurrency: env_usize("BENCH_READ_CONCURRENCY", 20),
            write_concurrency: env_usize("BENCH_WRITE_CONCURRENCY", 10),
            output: env::var("BENCH_OUTPUT").unwrap_or_else(|_| {
                format!(
                    "bench-results/local-bench-{}.json",
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
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                "--base-url" => cfg.base_url = next_arg_value(&mut args, "--base-url")?,
                "--admin-id" => cfg.admin_id = next_arg_value(&mut args, "--admin-id")?,
                "--admin-pw" => cfg.admin_pw = next_arg_value(&mut args, "--admin-pw")?,
                "--attendees" => {
                    cfg.attendees =
                        parse_usize_arg(&next_arg_value(&mut args, "--attendees")?, "--attendees")?
                }
                "--read-requests" => {
                    cfg.read_requests = parse_usize_arg(
                        &next_arg_value(&mut args, "--read-requests")?,
                        "--read-requests",
                    )?
                }
                "--write-requests" => {
                    cfg.write_requests = parse_usize_arg(
                        &next_arg_value(&mut args, "--write-requests")?,
                        "--write-requests",
                    )?
                }
                "--read-concurrency" => {
                    cfg.read_concurrency = parse_usize_arg(
                        &next_arg_value(&mut args, "--read-concurrency")?,
                        "--read-concurrency",
                    )?
                }
                "--write-concurrency" => {
                    cfg.write_concurrency = parse_usize_arg(
                        &next_arg_value(&mut args, "--write-concurrency")?,
                        "--write-concurrency",
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
        if cfg.attendees == 0 {
            return Err(anyhow!("--attendees must be >= 1"));
        }
        if cfg.read_concurrency == 0 || cfg.write_concurrency == 0 {
            return Err(anyhow!("Concurrency values must be >= 1"));
        }

        Ok(cfg)
    }
}

#[derive(Debug)]
struct SetupData {
    codelab_id: String,
    admin_session: SessionCookies,
    attendees: Vec<AttendeeSession>,
}

#[derive(Clone, Debug)]
struct RequestOutcome {
    status: Option<u16>,
    error: Option<String>,
}

impl RequestOutcome {
    fn with_status(status: u16) -> Self {
        Self {
            status: Some(status),
            error: None,
        }
    }

    fn with_error(error: String) -> Self {
        Self {
            status: None,
            error: Some(error),
        }
    }
}

#[derive(Debug)]
struct RequestSample {
    latency_ms: f64,
    outcome: RequestOutcome,
}

type RequestFn = Arc<dyn Fn(usize) -> BoxFuture<'static, RequestOutcome> + Send + Sync>;

#[derive(Serialize)]
struct BenchmarkReport {
    generated_at_utc: String,
    local_environment: LocalEnvironment,
    config: PublicConfig,
    setup: SetupSummary,
    scenarios: Vec<ScenarioResult>,
    notes: Vec<String>,
}

#[derive(Serialize)]
struct LocalEnvironment {
    os: String,
    arch: String,
    rust_log: Option<String>,
}

#[derive(Serialize)]
struct PublicConfig {
    base_url: String,
    admin_id: String,
    attendees: usize,
    read_requests: usize,
    write_requests: usize,
    read_concurrency: usize,
    write_concurrency: usize,
    output: String,
}

#[derive(Serialize)]
struct SetupSummary {
    codelab_id: String,
    attendee_count: usize,
}

#[derive(Serialize)]
struct ScenarioResult {
    name: String,
    total_requests: usize,
    completed_requests: usize,
    success_2xx: usize,
    http_errors: usize,
    transport_errors: usize,
    error_rate: f64,
    duration_ms: f64,
    requests_per_second: f64,
    latency_ms: LatencySummary,
    status_counts: HashMap<String, usize>,
    top_transport_errors: Vec<ErrorCount>,
}

#[derive(Serialize)]
struct LatencySummary {
    min: f64,
    mean: f64,
    p50: f64,
    p95: f64,
    p99: f64,
    max: f64,
}

#[derive(Serialize)]
struct ErrorCount {
    error: String,
    count: usize,
}

impl ScenarioResult {
    fn from_samples(
        name: &str,
        total_requests: usize,
        duration: Duration,
        samples: &[RequestSample],
    ) -> Self {
        let mut latencies = Vec::with_capacity(samples.len());
        let mut status_counts: HashMap<String, usize> = HashMap::new();
        let mut transport_errors: HashMap<String, usize> = HashMap::new();

        let mut success_2xx = 0usize;
        let mut http_errors = 0usize;
        let mut transport_error_count = 0usize;

        for sample in samples {
            latencies.push(sample.latency_ms);
            match sample.outcome.status {
                Some(status) => {
                    *status_counts.entry(status.to_string()).or_insert(0) += 1;
                    if (200..300).contains(&status) {
                        success_2xx += 1;
                    } else {
                        http_errors += 1;
                    }
                }
                None => {
                    transport_error_count += 1;
                    *status_counts
                        .entry("transport_error".to_string())
                        .or_insert(0) += 1;
                    let key = sample
                        .outcome
                        .error
                        .as_deref()
                        .unwrap_or("unknown transport error")
                        .to_string();
                    *transport_errors.entry(key).or_insert(0) += 1;
                }
            }
        }

        latencies.sort_by(|a, b| a.total_cmp(b));
        let mean = if latencies.is_empty() {
            0.0
        } else {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        };
        let latency_ms = LatencySummary {
            min: latencies.first().copied().unwrap_or(0.0),
            mean,
            p50: percentile(&latencies, 0.50),
            p95: percentile(&latencies, 0.95),
            p99: percentile(&latencies, 0.99),
            max: latencies.last().copied().unwrap_or(0.0),
        };

        let duration_ms = duration.as_secs_f64() * 1000.0;
        let requests_per_second = if duration_ms > 0.0 {
            samples.len() as f64 / (duration_ms / 1000.0)
        } else {
            0.0
        };
        let total_errors = http_errors + transport_error_count;
        let error_rate = if total_requests > 0 {
            total_errors as f64 / total_requests as f64
        } else {
            0.0
        };

        let mut top_transport_errors = transport_errors
            .into_iter()
            .map(|(error, count)| ErrorCount { error, count })
            .collect::<Vec<_>>();
        top_transport_errors.sort_by(|a, b| b.count.cmp(&a.count));
        top_transport_errors.truncate(5);

        Self {
            name: name.to_string(),
            total_requests,
            completed_requests: samples.len(),
            success_2xx,
            http_errors,
            transport_errors: transport_error_count,
            error_rate,
            duration_ms,
            requests_per_second,
            latency_ms,
            status_counts,
            top_transport_errors,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_args()?;
    println!(
        "Preparing local benchmark (base_url={}, attendees={})",
        cfg.base_url, cfg.attendees
    );

    let client = Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;

    let setup = setup_data(&client, &cfg).await?;
    println!("Setup complete. codelab_id={}", setup.codelab_id);

    let scenarios = vec![
        run_admin_attendees_read(&client, &cfg, &setup).await,
        run_admin_chat_history_read(&client, &cfg, &setup).await,
        run_attendee_help_write(&client, &cfg, &setup).await,
        run_attendee_submission_link_write(&client, &cfg, &setup).await,
    ];

    let too_many_requests = scenarios
        .iter()
        .map(|s| s.status_counts.get("429").copied().unwrap_or(0))
        .sum::<usize>();

    let mut notes = vec![
        "Local-only run. Use this for baseline/trend comparison, not absolute production capacity.".to_string(),
        "For paper tables, repeat each run multiple times and report median/IQR with confidence intervals.".to_string(),
    ];
    if too_many_requests > 0 {
        notes.push(
            "Detected HTTP 429 responses. Increase RATE_LIMIT_* values for higher-load experiments."
                .to_string(),
        );
    }

    let report = BenchmarkReport {
        generated_at_utc: Utc::now().to_rfc3339(),
        local_environment: LocalEnvironment {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_log: env::var("RUST_LOG").ok(),
        },
        config: PublicConfig {
            base_url: cfg.base_url.clone(),
            admin_id: cfg.admin_id.clone(),
            attendees: cfg.attendees,
            read_requests: cfg.read_requests,
            write_requests: cfg.write_requests,
            read_concurrency: cfg.read_concurrency,
            write_concurrency: cfg.write_concurrency,
            output: cfg.output.clone(),
        },
        setup: SetupSummary {
            codelab_id: setup.codelab_id.clone(),
            attendee_count: setup.attendees.len(),
        },
        scenarios,
        notes,
    };

    write_report(&cfg.output, &report)?;
    print_summary(&report);

    Ok(())
}

async fn run_admin_attendees_read(
    client: &Client,
    cfg: &Config,
    setup: &SetupData,
) -> ScenarioResult {
    let client = client.clone();
    let base_url = cfg.base_url.clone();
    let codelab_id = setup.codelab_id.clone();
    let admin_cookie = setup.admin_session.cookie_header.clone();

    let request_fn: RequestFn = Arc::new(move |_idx| {
        let client = client.clone();
        let base_url = base_url.clone();
        let codelab_id = codelab_id.clone();
        let admin_cookie = admin_cookie.clone();
        Box::pin(async move {
            let url = format!("{}/api/codelabs/{}/attendees", base_url, codelab_id);
            match client
                .get(url)
                .header(header::COOKIE, admin_cookie)
                .send()
                .await
            {
                Ok(res) => RequestOutcome::with_status(res.status().as_u16()),
                Err(err) => RequestOutcome::with_error(err.to_string()),
            }
        })
    });

    run_scenario(
        "admin_get_attendees",
        cfg.read_requests,
        cfg.read_concurrency,
        request_fn,
    )
    .await
}

async fn run_admin_chat_history_read(
    client: &Client,
    cfg: &Config,
    setup: &SetupData,
) -> ScenarioResult {
    let client = client.clone();
    let base_url = cfg.base_url.clone();
    let codelab_id = setup.codelab_id.clone();
    let admin_cookie = setup.admin_session.cookie_header.clone();

    let request_fn: RequestFn = Arc::new(move |_idx| {
        let client = client.clone();
        let base_url = base_url.clone();
        let codelab_id = codelab_id.clone();
        let admin_cookie = admin_cookie.clone();
        Box::pin(async move {
            let url = format!("{}/api/codelabs/{}/chat", base_url, codelab_id);
            match client
                .get(url)
                .header(header::COOKIE, admin_cookie)
                .send()
                .await
            {
                Ok(res) => RequestOutcome::with_status(res.status().as_u16()),
                Err(err) => RequestOutcome::with_error(err.to_string()),
            }
        })
    });

    run_scenario(
        "admin_get_chat_history",
        cfg.read_requests,
        cfg.read_concurrency,
        request_fn,
    )
    .await
}

async fn run_attendee_help_write(
    client: &Client,
    cfg: &Config,
    setup: &SetupData,
) -> ScenarioResult {
    let client = client.clone();
    let base_url = cfg.base_url.clone();
    let codelab_id = setup.codelab_id.clone();
    let sessions = Arc::new(setup.attendees.clone());

    let request_fn: RequestFn = Arc::new(move |idx| {
        let client = client.clone();
        let codelab_id = codelab_id.clone();
        let base_url = base_url.clone();
        let sessions = sessions.clone();
        Box::pin(async move {
            let session = sessions[idx % sessions.len()].clone();
            let url = format!("{}/api/codelabs/{}/help", base_url, codelab_id);
            let mut req = client
                .request(Method::POST, url)
                .header(header::COOKIE, session.cookies.cookie_header);
            if let Some(csrf) = session.cookies.csrf_token {
                req = req.header("x-csrf-token", csrf);
            }
            req = req.json(&json!({
                "step_number": ((idx % 8) + 1) as i32
            }));

            match req.send().await {
                Ok(res) => RequestOutcome::with_status(res.status().as_u16()),
                Err(err) => RequestOutcome::with_error(err.to_string()),
            }
        })
    });

    run_scenario(
        "attendee_post_help",
        cfg.write_requests,
        cfg.write_concurrency,
        request_fn,
    )
    .await
}

async fn run_attendee_submission_link_write(
    client: &Client,
    cfg: &Config,
    setup: &SetupData,
) -> ScenarioResult {
    let client = client.clone();
    let base_url = cfg.base_url.clone();
    let codelab_id = setup.codelab_id.clone();
    let sessions = Arc::new(setup.attendees.clone());

    let request_fn: RequestFn = Arc::new(move |idx| {
        let client = client.clone();
        let codelab_id = codelab_id.clone();
        let base_url = base_url.clone();
        let sessions = sessions.clone();
        Box::pin(async move {
            let session = sessions[idx % sessions.len()].clone();
            let url = format!(
                "{}/api/codelabs/{}/attendees/{}/submissions/link",
                base_url, codelab_id, session.attendee_id
            );

            let mut req = client
                .request(Method::POST, url)
                .header(header::COOKIE, session.cookies.cookie_header);
            if let Some(csrf) = session.cookies.csrf_token {
                req = req.header("x-csrf-token", csrf);
            }
            req = req.json(&json!({
                "url": format!("https://example.com/bench-submission/{}", idx),
                "title": format!("Bench Submission {}", idx)
            }));

            match req.send().await {
                Ok(res) => RequestOutcome::with_status(res.status().as_u16()),
                Err(err) => RequestOutcome::with_error(err.to_string()),
            }
        })
    });

    run_scenario(
        "attendee_post_submission_link",
        cfg.write_requests,
        cfg.write_concurrency,
        request_fn,
    )
    .await
}

async fn run_scenario(
    name: &str,
    total_requests: usize,
    concurrency: usize,
    request_fn: RequestFn,
) -> ScenarioResult {
    let started = Instant::now();

    let samples = stream::iter(0..total_requests)
        .map(|idx| {
            let request_fn = request_fn.clone();
            async move {
                let request_started = Instant::now();
                let outcome = (request_fn)(idx).await;
                RequestSample {
                    latency_ms: request_started.elapsed().as_secs_f64() * 1000.0,
                    outcome,
                }
            }
        })
        .buffer_unordered(concurrency.max(1))
        .collect::<Vec<_>>()
        .await;

    ScenarioResult::from_samples(name, total_requests, started.elapsed(), &samples)
}

async fn setup_data(client: &Client, cfg: &Config) -> Result<SetupData> {
    let admin_session = login_admin(client, cfg).await?;
    let codelab_id = create_codelab(client, cfg, &admin_session).await?;

    let mut attendees = Vec::with_capacity(cfg.attendees);
    for idx in 0..cfg.attendees {
        let name = format!("bench-user-{:04}", idx + 1);
        let code = format!("bench-code-{:04}", idx + 1);
        attendees.push(register_attendee(client, &cfg.base_url, &codelab_id, &name, &code).await?);
    }

    Ok(SetupData {
        codelab_id,
        admin_session,
        attendees,
    })
}

async fn login_admin(client: &Client, cfg: &Config) -> Result<SessionCookies> {
    let url = format!("{}/api/login", cfg.base_url);
    let response = client
        .post(url)
        .json(&json!({
            "admin_id": cfg.admin_id,
            "admin_pw": cfg.admin_pw,
        }))
        .send()
        .await
        .context("Failed to call /api/login")?;

    build_session_from_response(response, "/api/login").await
}

async fn create_codelab(client: &Client, cfg: &Config, admin: &SessionCookies) -> Result<String> {
    let mut req = client
        .post(format!("{}/api/codelabs", cfg.base_url))
        .header(header::COOKIE, admin.cookie_header.clone());
    if let Some(csrf) = &admin.csrf_token {
        req = req.header("x-csrf-token", csrf);
    }

    let response = req
        .json(&json!({
            "title": format!("Local Bench {}", Utc::now().format("%Y-%m-%d %H:%M:%S")),
            "description": "Auto-generated benchmark codelab",
            "author": "local-bench",
            "is_public": true,
            "quiz_enabled": false,
            "require_quiz": false,
            "require_feedback": false,
            "require_submission": false,
            "guide_markdown": "# Local Benchmark"
        }))
        .send()
        .await
        .context("Failed to call /api/codelabs")?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(
            "Failed to create codelab: HTTP {} body={}",
            status.as_u16(),
            truncate(&body, 200)
        ));
    }

    let parsed: Value =
        serde_json::from_str(&body).context("Failed to parse create codelab response")?;
    let id = parsed
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Create codelab response missing id"))?;
    Ok(id.to_string())
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
        .json(&json!({
            "name": name,
            "code": code,
            "email": Value::Null
        }))
        .send()
        .await
        .with_context(|| format!("Failed to register attendee {name}"))?;

    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(
            "Failed to register attendee {name}: HTTP {} body={}",
            status.as_u16(),
            truncate(&body, 200)
        ));
    }

    let cookies = parse_set_cookie_headers(&headers);
    if cookies.is_empty() {
        return Err(anyhow!(
            "No Set-Cookie header for attendee registration ({name})"
        ));
    }
    let csrf_token = find_csrf_token(&cookies);
    let cookie_header = build_cookie_header(&cookies);

    let parsed: Value = serde_json::from_str(&body).context("Failed to parse attendee response")?;
    let attendee_id = parsed
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Attendee registration response missing id"))?;

    Ok(AttendeeSession {
        attendee_id: attendee_id.to_string(),
        cookies: SessionCookies {
            cookie_header,
            csrf_token,
        },
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
            truncate(&body, 200)
        ));
    }

    let cookie_map = parse_set_cookie_headers(&headers);
    if cookie_map.is_empty() {
        return Err(anyhow!("No Set-Cookie headers from {context_path}"));
    }
    let cookie_header = build_cookie_header(&cookie_map);
    let csrf_token = find_csrf_token(&cookie_map);
    Ok(SessionCookies {
        cookie_header,
        csrf_token,
    })
}

fn parse_set_cookie_headers(headers: &header::HeaderMap) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for value in headers.get_all(header::SET_COOKIE).iter() {
        let Ok(raw) = value.to_str() else {
            continue;
        };
        let Some(first_pair) = raw.split(';').next() else {
            continue;
        };
        let Some((name, value)) = first_pair.split_once('=') else {
            continue;
        };
        let name = name.trim();
        let value = value.trim();
        if !name.is_empty() {
            map.insert(name.to_string(), value.to_string());
        }
    }
    map
}

fn build_cookie_header(cookies: &HashMap<String, String>) -> String {
    let mut pairs = cookies
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>();
    pairs.sort();
    pairs.join("; ")
}

fn find_csrf_token(cookies: &HashMap<String, String>) -> Option<String> {
    cookies
        .iter()
        .find(|(name, _)| name.ends_with("oc_csrf"))
        .map(|(_, value)| value.clone())
}

fn write_report(path: &str, report: &BenchmarkReport) -> Result<()> {
    let output_path = PathBuf::from(path);
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory {}", parent.display())
            })?;
        }
    }
    let serialized = serde_json::to_string_pretty(report).context("Failed to serialize report")?;
    fs::write(&output_path, serialized)
        .with_context(|| format!("Failed to write report to {}", output_path.display()))?;
    Ok(())
}

fn print_summary(report: &BenchmarkReport) {
    println!();
    println!("Benchmark complete: {}", report.config.output);
    println!("codelab_id: {}", report.setup.codelab_id);
    println!();
    println!(
        "{:<32} {:>9} {:>9} {:>9} {:>9} {:>9}",
        "scenario", "p50(ms)", "p95(ms)", "p99(ms)", "error%", "rps"
    );
    println!("{}", "-".repeat(84));
    for scenario in &report.scenarios {
        println!(
            "{:<32} {:>9.1} {:>9.1} {:>9.1} {:>9.2} {:>9.1}",
            scenario.name,
            scenario.latency_ms.p50,
            scenario.latency_ms.p95,
            scenario.latency_ms.p99,
            scenario.error_rate * 100.0,
            scenario.requests_per_second,
        );
    }

    if !report.notes.is_empty() {
        println!();
        println!("Notes:");
        for note in &report.notes {
            println!("- {note}");
        }
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let p = p.clamp(0.0, 1.0);
    let max_index = sorted_values.len() - 1;
    let rank = p * max_index as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted_values[lower]
    } else {
        let weight = rank - lower as f64;
        sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
    }
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
    let mut truncated = value.chars().take(max_len).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn print_help() {
    println!(
        r#"Local benchmark runner for Open Codelabs backend.

Usage:
  cargo run --release --bin local_bench -- [options]

Options:
  --base-url <url>               Backend base URL (default: http://localhost:8080)
  --admin-id <id>                Admin login ID (default: env ADMIN_ID or "admin")
  --admin-pw <pw>                Admin login password (default: env ADMIN_PW or "admin")
  --attendees <n>                Number of attendees to auto-register (default: 20)
  --read-requests <n>            Requests per read scenario (default: 200)
  --write-requests <n>           Requests per write scenario (default: 100)
  --read-concurrency <n>         Concurrency for read scenarios (default: 20)
  --write-concurrency <n>        Concurrency for write scenarios (default: 10)
  --output <path>                JSON output path
  -h, --help                     Show this help

Environment overrides:
  BENCH_BASE_URL, BENCH_ATTENDEES, BENCH_READ_REQUESTS, BENCH_WRITE_REQUESTS,
  BENCH_READ_CONCURRENCY, BENCH_WRITE_CONCURRENCY, BENCH_OUTPUT

Scenarios executed:
  1) GET  /api/codelabs/{{id}}/attendees                (admin read)
  2) GET  /api/codelabs/{{id}}/chat                     (admin read)
  3) POST /api/codelabs/{{id}}/help                     (attendee write)
  4) POST /api/codelabs/{{id}}/attendees/{{aid}}/submissions/link

Note:
  If you see many HTTP 429 responses, raise RATE_LIMIT_* in backend env for load testing.
"#
    );
}
