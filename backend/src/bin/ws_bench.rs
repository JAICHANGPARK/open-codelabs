use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use reqwest::{header, Client};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct Config {
    base_url: String,
    admin_id: String,
    admin_pw: String,
    users: Vec<usize>,
    duration_secs: u64,
    chat_rate_per_sec: f64,
    step_interval_secs: u64,
    output: String,
}

impl Config {
    fn defaults() -> Self {
        Self {
            base_url: env::var("BENCH_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            admin_id: env::var("ADMIN_ID").unwrap_or_else(|_| "admin".to_string()),
            admin_pw: env::var("ADMIN_PW").unwrap_or_else(|_| "admin".to_string()),
            users: vec![50, 100, 200],
            duration_secs: env_u64("WS_BENCH_DURATION_SECS", 30),
            chat_rate_per_sec: env_f64("WS_BENCH_CHAT_RATE", 0.5),
            step_interval_secs: env_u64("WS_BENCH_STEP_INTERVAL_SECS", 7),
            output: env::var("WS_BENCH_OUTPUT").unwrap_or_else(|_| {
                format!(
                    "bench-results/ws-bench-{}.json",
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
                "--users" => {
                    let raw = next_arg_value(&mut args, "--users")?;
                    cfg.users = parse_users_csv(&raw)?;
                }
                "--duration-secs" => {
                    cfg.duration_secs = parse_u64_arg(
                        &next_arg_value(&mut args, "--duration-secs")?,
                        "--duration-secs",
                    )?
                }
                "--chat-rate" => {
                    cfg.chat_rate_per_sec =
                        parse_f64_arg(&next_arg_value(&mut args, "--chat-rate")?, "--chat-rate")?
                }
                "--step-interval-secs" => {
                    cfg.step_interval_secs = parse_u64_arg(
                        &next_arg_value(&mut args, "--step-interval-secs")?,
                        "--step-interval-secs",
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
        if cfg.users.is_empty() {
            return Err(anyhow!("--users cannot be empty"));
        }
        if cfg.duration_secs == 0 {
            return Err(anyhow!("--duration-secs must be >= 1"));
        }
        if cfg.chat_rate_per_sec < 0.0 {
            return Err(anyhow!("--chat-rate must be >= 0"));
        }
        if cfg.step_interval_secs == 0 {
            return Err(anyhow!("--step-interval-secs must be >= 1"));
        }

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
    name: String,
    token: String,
}

#[derive(Serialize)]
struct WsBenchReport {
    generated_at_utc: String,
    config: WsBenchPublicConfig,
    cases: Vec<WsCaseResult>,
    notes: Vec<String>,
}

#[derive(Serialize)]
struct WsBenchPublicConfig {
    base_url: String,
    users: Vec<usize>,
    duration_secs: u64,
    chat_rate_per_sec: f64,
    step_interval_secs: u64,
    output: String,
}

#[derive(Serialize)]
struct WsCaseResult {
    users: usize,
    codelab_id: String,
    duration_secs: u64,
    sent_chat: u64,
    sent_step_progress: u64,
    recv_chat: u64,
    ws_disconnects: u64,
    ws_errors: u64,
    ws_error_samples: Vec<String>,
    throughput: ThroughputSummary,
    e2e_latency_ms: PercentileSummary,
}

#[derive(Serialize)]
struct ThroughputSummary {
    sent_chat_per_sec: f64,
    recv_chat_per_sec: f64,
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

#[derive(Default)]
struct ClientMetrics {
    sent_chat: u64,
    sent_step: u64,
    recv_chat: u64,
    disconnects: u64,
    errors: Vec<String>,
    rtt_ms: Vec<f64>,
}

#[derive(Default)]
struct SharedClientState {
    metrics: ClientMetrics,
    pending: HashMap<String, Instant>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_args()?;

    let http_client = Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;

    let mut cases = Vec::new();
    for users in &cfg.users {
        println!(
            "Running WS case: users={} duration={}s chat_rate={}/s step_interval={}s",
            users, cfg.duration_secs, cfg.chat_rate_per_sec, cfg.step_interval_secs
        );
        let case = run_case(&http_client, &cfg, *users).await?;
        println!(
            "Case done users={} sent_chat={} p95={:.2}ms",
            case.users, case.sent_chat, case.e2e_latency_ms.p95
        );
        cases.push(case);
    }

    let notes = vec![
        "WS benchmark measures sender->echo roundtrip latency for chat messages.".to_string(),
        "If ws_disconnects/ws_errors are high, raise RATE_LIMIT_GENERAL_PER_MINUTE and inspect server logs.".to_string(),
    ];
    let report = WsBenchReport {
        generated_at_utc: Utc::now().to_rfc3339(),
        config: WsBenchPublicConfig {
            base_url: cfg.base_url.clone(),
            users: cfg.users.clone(),
            duration_secs: cfg.duration_secs,
            chat_rate_per_sec: cfg.chat_rate_per_sec,
            step_interval_secs: cfg.step_interval_secs,
            output: cfg.output.clone(),
        },
        cases,
        notes,
    };

    write_json(&cfg.output, &report)?;
    print_summary(&report);

    Ok(())
}

async fn run_case(http_client: &Client, cfg: &Config, users: usize) -> Result<WsCaseResult> {
    let admin = login_admin(http_client, cfg).await?;
    let codelab_id = create_codelab(http_client, cfg, &admin).await?;
    let attendees = register_attendees(http_client, &cfg.base_url, &codelab_id, users).await?;
    let ws_base = to_ws_base_url(&cfg.base_url)?;

    let duration = Duration::from_secs(cfg.duration_secs);
    let chat_interval = if cfg.chat_rate_per_sec <= 0.0 {
        None
    } else {
        Some(Duration::from_secs_f64(1.0 / cfg.chat_rate_per_sec))
    };
    let step_interval = Duration::from_secs(cfg.step_interval_secs);

    let mut handles = Vec::with_capacity(attendees.len());
    for attendee in attendees {
        let ws_url = format!(
            "{}/api/ws/{}?as=attendee&token={}",
            ws_base, codelab_id, attendee.token
        );
        handles.push(tokio::spawn(run_ws_client(
            attendee,
            ws_url,
            duration,
            chat_interval,
            step_interval,
        )));
    }

    let mut sent_chat = 0u64;
    let mut sent_step = 0u64;
    let mut recv_chat = 0u64;
    let mut disconnects = 0u64;
    let mut errors = Vec::new();
    let mut rtt_samples = Vec::new();

    for handle in handles {
        match handle.await {
            Ok(Ok(metrics)) => {
                sent_chat += metrics.sent_chat;
                sent_step += metrics.sent_step;
                recv_chat += metrics.recv_chat;
                disconnects += metrics.disconnects;
                errors.extend(metrics.errors);
                rtt_samples.extend(metrics.rtt_ms);
            }
            Ok(Err(err)) => {
                disconnects += 1;
                errors.push(err.to_string());
            }
            Err(join_err) => {
                disconnects += 1;
                errors.push(format!("Join error: {join_err}"));
            }
        }
    }

    let duration_secs = cfg.duration_secs as f64;
    let throughput = ThroughputSummary {
        sent_chat_per_sec: if duration_secs > 0.0 {
            sent_chat as f64 / duration_secs
        } else {
            0.0
        },
        recv_chat_per_sec: if duration_secs > 0.0 {
            recv_chat as f64 / duration_secs
        } else {
            0.0
        },
    };

    let e2e = summarize_percentiles(&mut rtt_samples);
    let ws_errors = errors.len() as u64;
    let ws_error_samples = errors.into_iter().take(8).collect::<Vec<_>>();

    Ok(WsCaseResult {
        users,
        codelab_id,
        duration_secs: cfg.duration_secs,
        sent_chat,
        sent_step_progress: sent_step,
        recv_chat,
        ws_disconnects: disconnects,
        ws_errors,
        ws_error_samples,
        throughput,
        e2e_latency_ms: e2e,
    })
}

async fn run_ws_client(
    attendee: AttendeeSession,
    ws_url: String,
    duration: Duration,
    chat_interval: Option<Duration>,
    step_interval: Duration,
) -> Result<ClientMetrics> {
    let url = Url::parse(&ws_url).context("Invalid ws url")?;
    let (socket, _) = connect_async(url.as_str())
        .await
        .with_context(|| format!("Failed to connect WS for {}", attendee.name))?;
    let (mut ws_write, mut ws_read) = socket.split();

    let shared = Arc::new(Mutex::new(SharedClientState::default()));
    let shared_reader = shared.clone();
    let my_name = attendee.name.clone();

    let reader_task = tokio::spawn(async move {
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let parsed = serde_json::from_str::<Value>(&text);
                    let Ok(v) = parsed else {
                        let mut state = shared_reader.lock().await;
                        state.metrics.errors.push("Invalid JSON frame".to_string());
                        continue;
                    };

                    let msg_type = v.get("type").and_then(Value::as_str).unwrap_or_default();
                    if msg_type != "chat" {
                        continue;
                    }

                    let sender = v
                        .get("sender")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let message = v
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();

                    let mut state = shared_reader.lock().await;
                    state.metrics.recv_chat += 1;
                    if sender == my_name && message.starts_with("BENCH_CHAT::") {
                        let maybe_id = message.strip_prefix("BENCH_CHAT::").unwrap_or_default();
                        if let Some(sent_at) = state.pending.remove(maybe_id) {
                            state
                                .metrics
                                .rtt_ms
                                .push(sent_at.elapsed().as_secs_f64() * 1000.0);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    let mut state = shared_reader.lock().await;
                    state.metrics.disconnects += 1;
                    break;
                }
                Ok(_) => {}
                Err(err) => {
                    let mut state = shared_reader.lock().await;
                    state.metrics.disconnects += 1;
                    state.metrics.errors.push(format!("WS read error: {err}"));
                    break;
                }
            }
        }
    });

    let start = Instant::now();
    let end = start + duration;
    let mut next_chat = chat_interval.map(|_| start);
    let mut next_step = start + step_interval;

    loop {
        let now = Instant::now();
        if now >= end {
            break;
        }

        let mut next_deadline = end;
        if let Some(nc) = next_chat {
            if nc < next_deadline {
                next_deadline = nc;
            }
        }
        if next_step < next_deadline {
            next_deadline = next_step;
        }

        if now < next_deadline {
            sleep(next_deadline - now).await;
            continue;
        }

        if let Some(nc) = next_chat {
            if now >= nc {
                let message_id = Uuid::new_v4().to_string();
                let text = format!("BENCH_CHAT::{message_id}");
                {
                    let mut state = shared.lock().await;
                    state.pending.insert(message_id.clone(), Instant::now());
                }
                let payload = json!({
                    "type": "chat",
                    "message": text
                })
                .to_string();
                if let Err(err) = ws_write.send(Message::Text(payload.into())).await {
                    let mut state = shared.lock().await;
                    state
                        .metrics
                        .errors
                        .push(format!("WS send chat error: {err}"));
                    state.metrics.disconnects += 1;
                    break;
                }
                {
                    let mut state = shared.lock().await;
                    state.metrics.sent_chat += 1;
                }
                if let Some(interval) = chat_interval {
                    next_chat = Some(nc + interval);
                }
                continue;
            }
        }

        if now >= next_step {
            let step_number = ((now.duration_since(start).as_secs() % 10) + 1) as i32;
            let payload = json!({
                "type": "step_progress",
                "step_number": step_number
            })
            .to_string();
            if let Err(err) = ws_write.send(Message::Text(payload.into())).await {
                let mut state = shared.lock().await;
                state
                    .metrics
                    .errors
                    .push(format!("WS send step error: {err}"));
                state.metrics.disconnects += 1;
                break;
            }
            {
                let mut state = shared.lock().await;
                state.metrics.sent_step += 1;
            }
            next_step += step_interval;
        }
    }

    sleep(Duration::from_millis(700)).await;
    let _ = ws_write.send(Message::Close(None)).await;
    let _ = timeout(Duration::from_secs(3), reader_task).await;

    let mut state = shared.lock().await;
    // Pending messages that never came back are counted as implicit WS errors.
    let pending = state.pending.len();
    if pending > 0 {
        state
            .metrics
            .errors
            .push(format!("{pending} pending echoes not received"));
        state.pending.clear();
    }

    Ok(std::mem::take(&mut state.metrics))
}

async fn register_attendees(
    client: &Client,
    base_url: &str,
    codelab_id: &str,
    count: usize,
) -> Result<Vec<AttendeeSession>> {
    let mut sessions = Vec::with_capacity(count);
    for i in 0..count {
        let name = format!("ws-user-{:04}", i + 1);
        let code = format!("ws-code-{:04}", i + 1);
        let response = client
            .post(format!("{}/api/codelabs/{}/register", base_url, codelab_id))
            .json(&json!({ "name": name, "code": code, "email": Value::Null }))
            .send()
            .await
            .with_context(|| format!("Failed to register attendee {}", i + 1))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!(
                "Register attendee failed HTTP {} body={}",
                status.as_u16(),
                truncate(&body, 180)
            ));
        }

        let parsed: Value =
            serde_json::from_str(&body).context("Invalid attendee registration JSON")?;
        let _id = parsed
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("Attendee response missing id"))?;
        let token = parsed
            .get("token")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("Attendee response missing token"))?;

        sessions.push(AttendeeSession {
            name,
            token: token.to_string(),
        });
    }
    Ok(sessions)
}

async fn login_admin(client: &Client, cfg: &Config) -> Result<SessionCookies> {
    let response = client
        .post(format!("{}/api/login", cfg.base_url))
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
            "title": format!("WS Bench {}", Utc::now().format("%Y-%m-%d %H:%M:%S")),
            "description": "Auto-generated websocket benchmark codelab",
            "author": "ws-bench",
            "is_public": true,
            "quiz_enabled": false,
            "require_quiz": false,
            "require_feedback": false,
            "require_submission": false,
            "guide_markdown": "# WS Benchmark"
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
            truncate(&body, 180)
        ));
    }
    let parsed: Value = serde_json::from_str(&body).context("Invalid codelab JSON")?;
    let id = parsed
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Create codelab response missing id"))?;
    Ok(id.to_string())
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
            truncate(&body, 180)
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

fn summarize_percentiles(samples: &mut Vec<f64>) -> PercentileSummary {
    if samples.is_empty() {
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
    samples.sort_by(|a, b| a.total_cmp(b));
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    PercentileSummary {
        samples: samples.len(),
        min: *samples.first().unwrap_or(&0.0),
        mean,
        p50: percentile(samples, 0.50),
        p95: percentile(samples, 0.95),
        p99: percentile(samples, 0.99),
        max: *samples.last().unwrap_or(&0.0),
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let p = p.clamp(0.0, 1.0);
    let max_idx = sorted.len() - 1;
    let rank = p * max_idx as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let w = rank - lower as f64;
        sorted[lower] * (1.0 - w) + sorted[upper] * w
    }
}

fn to_ws_base_url(http_base: &str) -> Result<String> {
    let ws = if let Some(rest) = http_base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else if let Some(rest) = http_base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if http_base.starts_with("ws://") || http_base.starts_with("wss://") {
        http_base.to_string()
    } else {
        return Err(anyhow!("Unsupported base URL scheme: {http_base}"));
    };
    Ok(ws.trim_end_matches('/').to_string())
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

fn write_json<T: Serialize>(path: &str, payload: &T) -> Result<()> {
    let output = PathBuf::from(path);
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory {}", parent.display())
            })?;
        }
    }
    let content = serde_json::to_string_pretty(payload).context("Failed to serialize JSON")?;
    fs::write(&output, content).with_context(|| format!("Failed to write {}", output.display()))?;
    Ok(())
}

fn print_summary(report: &WsBenchReport) {
    println!();
    println!("WS benchmark complete: {}", report.config.output);
    println!(
        "{:<8} {:>10} {:>10} {:>10} {:>9} {:>9}",
        "users", "sent/s", "recv/s", "p95(ms)", "disc", "errors"
    );
    println!("{}", "-".repeat(68));
    for case in &report.cases {
        println!(
            "{:<8} {:>10.1} {:>10.1} {:>10.2} {:>9} {:>9}",
            case.users,
            case.throughput.sent_chat_per_sec,
            case.throughput.recv_chat_per_sec,
            case.e2e_latency_ms.p95,
            case.ws_disconnects,
            case.ws_errors
        );
    }
}

fn parse_users_csv(raw: &str) -> Result<Vec<usize>> {
    let mut users = Vec::new();
    for part in raw.split(',') {
        let item = part.trim();
        if item.is_empty() {
            continue;
        }
        let value = item
            .parse::<usize>()
            .with_context(|| format!("Invalid users value: {item}"))?;
        if value == 0 {
            return Err(anyhow!("User count must be >= 1"));
        }
        users.push(value);
    }
    if users.is_empty() {
        return Err(anyhow!("No valid users parsed from --users"));
    }
    Ok(users)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_f64(key: &str, default: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(default)
}

fn parse_u64_arg(value: &str, flag: &str) -> Result<u64> {
    value
        .parse::<u64>()
        .with_context(|| format!("Invalid value for {flag}: {value}"))
}

fn parse_f64_arg(value: &str, flag: &str) -> Result<f64> {
    value
        .parse::<f64>()
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

fn print_help() {
    println!(
        r#"WebSocket benchmark runner for Open Codelabs.

Usage:
  cargo run --release --bin ws_bench -- [options]

Options:
  --base-url <url>               Backend URL (default: http://localhost:8080)
  --admin-id <id>                Admin ID (default: env ADMIN_ID or "admin")
  --admin-pw <pw>                Admin password (default: env ADMIN_PW or "admin")
  --users <csv>                  Concurrent users (default: 50,100,200)
  --duration-secs <n>            Case duration seconds (default: 30)
  --chat-rate <n>                Chat messages per user per sec (default: 0.5)
  --step-interval-secs <n>       step_progress interval seconds (default: 7)
  --output <path>                JSON output path
  -h, --help                     Show this help

Example:
  cargo run --release --bin ws_bench -- \
    --users 50,100,200 \
    --duration-secs 60 \
    --chat-rate 0.5 \
    --step-interval-secs 7 \
    --output bench-results/ws-paper-run.json
"#
    );
}
