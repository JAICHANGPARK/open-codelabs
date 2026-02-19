# Local Benchmark Guide (Full Suite)

This guide runs a **local-only full benchmark suite** for Open Codelabs.
It is designed for paper artifacts (NeurIPS/SCI style): latency percentiles, throughput, stability, SWE quality, and reproducibility assets.

## 1) Start services locally

Backend:

```bash
cd backend
cp .env.sample .env
# set ADMIN_ID / ADMIN_PW / DATABASE_URL
cargo run --bin backend
```

Optional frontend (for render/load benchmark):

```bash
cd frontend
bun install
bun run dev --host 0.0.0.0 --port 5173
```

Default URLs:

- backend: `http://localhost:8080`
- frontend: `http://localhost:5173`

## 2) Recommended rate-limit override during benchmarks

Without this, load tests can hit many `429` responses:

```bash
export RATE_LIMIT_GENERAL_PER_MINUTE=100000
export RATE_LIMIT_LOGIN_PER_5_MIN=100000
export RATE_LIMIT_AI_PER_MINUTE=100000
export RATE_LIMIT_UPLOAD_PER_MINUTE=100000
```

Restart backend after env changes.

## 3) Run everything (all benchmarks)

```bash
cd backend
chmod +x bench/run_all_local.sh bench/run_matrix.sh
./bench/run_all_local.sh \
  --base-url http://localhost:8080 \
  --admin-id admin \
  --admin-pw admin \
  --rest-profile paper \
  --rest-repeats 10 \
  --ws-users 50,100,200 \
  --ws-duration-secs 60 \
  --ws-chat-rate 0.5 \
  --ws-step-interval-secs 7 \
  --ws-repeats 10 \
  --ops-repeats 5 \
  --soak-duration-secs 7200 \
  --frontend-base-url http://localhost:5173
```

Fast smoke run:

```bash
./bench/run_all_local.sh \
  --base-url http://localhost:8080 \
  --admin-id admin \
  --admin-pw admin \
  --rest-matrix-file "$(pwd)/bench/matrix.smoke.csv" \
  --rest-repeats 1 \
  --ws-users 5 \
  --ws-duration-secs 8 \
  --ws-repeats 1 \
  --ops-repeats 1 \
  --ops-profile quick \
  --ops-upload-requests 3 \
  --ops-codeserver-iterations 1 \
  --soak-duration-secs 30 \
  --soak-cycle-interval-secs 30 \
  --soak-local-attendees 3 \
  --soak-local-read-requests 12 \
  --soak-local-write-requests 8 \
  --soak-local-read-concurrency 3 \
  --soak-local-write-concurrency 2 \
  --skip-frontend
```

This orchestrates:

1. REST read/write matrix benchmark (`local_bench`)
2. WebSocket real-time load benchmark (`ws_bench`)
3. Upload/backup/codeserver operations benchmark (`ops_bench`)
4. Frontend route render/load benchmark (`frontend_render_bench.py`)
5. 2h soak test (`soak_bench.py`)
6. SWE metrics (`swe_metrics.py`)
7. Statistical aggregation (`analyze_paper_stats.py`)
8. Plot generation (`plot_paper.py`)

## 4) Individual benchmark commands

REST split load:

```bash
cd backend
cargo run --release --bin local_bench -- \
  --base-url http://localhost:8080 \
  --admin-id admin \
  --admin-pw admin \
  --attendees 100 \
  --read-requests 3000 \
  --write-requests 1500 \
  --read-concurrency 120 \
  --write-concurrency 80 \
  --output bench-results/rest-single.json
```

WS real-time load:

```bash
cd backend
cargo run --release --bin ws_bench -- \
  --base-url http://localhost:8080 \
  --admin-id admin \
  --admin-pw admin \
  --users 50,100,200 \
  --duration-secs 60 \
  --chat-rate 0.5 \
  --step-interval-secs 7 \
  --output bench-results/ws-single.json
```

Upload / backup / code-server:

```bash
cd backend
cargo run --release --bin ops_bench -- \
  --base-url http://localhost:8080 \
  --admin-id admin \
  --admin-pw admin \
  --profile paper \
  --scenarios upload,backup,codeserver \
  --upload-requests 60 \
  --codeserver-iterations 8 \
  --output bench-results/ops-single.json
```

Frontend route render/load:

```bash
cd backend
python3 bench/frontend_render_bench.py \
  --base-url http://localhost:5173 \
  --routes /,/admin,/codelabs \
  --concurrency 10,30,60 \
  --requests 300 \
  --repeats 3 \
  --output bench-results/frontend-single.json
```

Soak test:

```bash
cd backend
python3 bench/soak_bench.py \
  --base-url http://localhost:8080 \
  --admin-id admin \
  --admin-pw admin \
  --duration-secs 7200 \
  --cycle-interval-secs 300 \
  --backend-pid <backend_pid> \
  --output bench-results/soak.json
```

SWE metrics:

```bash
cd backend
python3 bench/swe_metrics.py \
  --backend-dir "$(pwd)" \
  --frontend-dir ../frontend \
  --output bench-results/swe-metrics.json
```

## 5) Statistics and plots

Aggregate repeated runs:

```bash
cd backend
python3 bench/analyze_paper_stats.py \
  --results-root bench-results/all-YYYYMMDD-HHMMSS \
  --output bench-results/all-YYYYMMDD-HHMMSS/stats/paper-stats-YYYYMMDD-HHMMSS.json \
  --csv bench-results/all-YYYYMMDD-HHMMSS/stats/paper-stats-YYYYMMDD-HHMMSS.csv
```

With baseline comparison:

```bash
python3 bench/analyze_paper_stats.py \
  --results-root bench-results/all-new \
  --baseline-root bench-results/all-old \
  --output bench-results/all-new/stats/paper-stats-YYYYMMDD-HHMMSS.json \
  --csv bench-results/all-new/stats/paper-stats-YYYYMMDD-HHMMSS.csv
```

Generate SVG charts:

```bash
python3 bench/plot_paper.py \
  --csv bench-results/all-YYYYMMDD-HHMMSS/stats/paper-stats-YYYYMMDD-HHMMSS.csv \
  --out-dir bench-results/all-YYYYMMDD-HHMMSS/plots-YYYYMMDD-HHMMSS
```

## 6) Paper protocol recommendations

- Warmup then measure: e.g. warmup 2 minutes + measure 8 minutes.
- Repeat each condition at least 10 times.
- Report median + IQR + bootstrap 95% CI.
- For A/B: report Mann-Whitney U p-value + Cliff’s delta.
- Keep fixed hardware/OS, commit hash, env, matrix.

## 7) Reproducibility artifacts produced

`run_all_local.sh` writes:

- raw benchmark JSON logs
- run summary CSVs (예: `run-summary-YYYYMMDD-HHMMSS.csv`)
- run metadata (예: `run-metadata-YYYYMMDD-HHMMSS.json`, UTC timestamp 포함)
- stage outcome table (예: `stage-status-YYYYMMDD-HHMMSS.csv`, started_at_utc / ended_at_utc / duration_sec 포함)
- SWE raw command logs (`swe-logs/*.log`)
- aggregated stats JSON/CSV (예: `paper-stats-YYYYMMDD-HHMMSS.{json,csv}`)
- SVG plots + HTML index (예: `plots-YYYYMMDD-HHMMSS/index.html`)

## 8) Notes and limitations

- Frontend script measures route response latency (SSR/route path), not browser visual paint metrics. For LCP/INP/CLS, run Lighthouse/Playwright separately.
- Some SWE metrics are best-effort and depend on installed local tools (`cargo-llvm-cov`, `cargo-audit`, `cargo mutants`).
- DB comparison (SQLite vs PostgreSQL) can be done by repeating the same suite with different `DATABASE_URL` and comparing via `--baseline-root`.
- `run_all_local.sh` continues even if one stage fails and returns non-zero at the end; check `stage-status-YYYYMMDD-HHMMSS.csv` for failed stages.
