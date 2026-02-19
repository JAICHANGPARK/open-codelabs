#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${BACKEND_DIR}/.." && pwd)"
FRONTEND_DIR="${REPO_ROOT}/frontend"

BASE_URL="${BENCH_BASE_URL:-http://localhost:8080}"
ADMIN_ID="${ADMIN_ID:-admin}"
ADMIN_PW="${ADMIN_PW:-admin}"
FRONTEND_BASE_URL="${FRONT_BENCH_BASE_URL:-http://localhost:5173}"

REST_PROFILE="paper"
REST_REPEATS=10
REST_MATRIX_FILE=""

WS_USERS="50,100,200"
WS_DURATION_SECS=60
WS_CHAT_RATE="0.5"
WS_STEP_INTERVAL_SECS=7
WS_REPEATS=10

OPS_SCENARIOS="upload,backup,codeserver"
OPS_PROFILE="paper"
OPS_UPLOAD_REQUESTS=60
OPS_CODESERVER_ITERATIONS=8
OPS_REPEATS=5

SOAK_DURATION_SECS=7200
SOAK_CYCLE_INTERVAL_SECS=300
SOAK_BACKEND_PID=""
SOAK_ENABLE=1
SOAK_LOCAL_ATTENDEES=30
SOAK_LOCAL_READ_REQUESTS=300
SOAK_LOCAL_WRITE_REQUESTS=150
SOAK_LOCAL_READ_CONCURRENCY=30
SOAK_LOCAL_WRITE_CONCURRENCY=20
SOAK_WS_USERS=""

FRONTEND_ROUTES="/,/admin,/codelabs"
FRONTEND_CONCURRENCY="10,30,60"
FRONTEND_REQUESTS=300
FRONTEND_REPEATS=3
FRONTEND_ENABLE=1

SWE_ENABLE=1
SWE_TIMEOUT_SECS=3600
SWE_RUN_MUTATION=0
SWE_MUTATION_SCORE=""

DRY_RUN=0
SKIP_REST=0
SKIP_WS=0
SKIP_OPS=0
SKIP_SOAK=0
SKIP_FRONTEND=0
SKIP_SWE=0
SKIP_STATS=0
SKIP_PLOT=0

BASELINE_ROOT=""
OUTPUT_ROOT=""
RUN_TAG=""
RUN_METADATA_FILE=""
STAGE_STATUS_FILE=""
STATS_JSON_FILE=""
STATS_CSV_FILE=""
PLOTS_DIR=""
STAGE_FAILURES=0
STAGE_RESULTS=()

usage() {
  cat <<'EOF'
Run full local benchmark suite for paper-ready artifacts.

Usage:
  ./bench/run_all_local.sh [options]

Core options:
  --base-url <url>                     Backend URL (default: http://localhost:8080)
  --admin-id <id>
  --admin-pw <pw>
  --output-root <path>                 Output root (default: bench-results/all-<timestamp>)
  --baseline-root <path>               Optional baseline directory for significance testing
  --dry-run                            Print commands only

REST benchmark:
  --rest-profile <quick|paper>         Matrix profile (default: paper)
  --rest-matrix-file <path>            Custom matrix CSV (overrides --rest-profile)
  --rest-repeats <n>                   Repeats per matrix row (default: 10)
  --skip-rest

WS benchmark:
  --ws-users <csv>                     e.g. 50,100,200
  --ws-duration-secs <n>
  --ws-chat-rate <float>               msg/s per user
  --ws-step-interval-secs <n>
  --ws-repeats <n>
  --skip-ws

Ops benchmark (upload/backup/codeserver):
  --ops-scenarios <csv>                upload,backup,codeserver
  --ops-profile <paper|quick>          default: paper
  --ops-upload-requests <n>
  --ops-codeserver-iterations <n>
  --ops-repeats <n>
  --skip-ops

Soak benchmark:
  --soak-duration-secs <n>             default 7200 (2h)
  --soak-cycle-interval-secs <n>       default 300
  --soak-backend-pid <pid>             optional resource sampling
  --soak-local-attendees <n>
  --soak-local-read-requests <n>
  --soak-local-write-requests <n>
  --soak-local-read-concurrency <n>
  --soak-local-write-concurrency <n>
  --soak-ws-users <csv>                default: first value from --ws-users
  --skip-soak

Frontend render/load benchmark:
  --frontend-base-url <url>            default http://localhost:5173
  --frontend-routes <csv>
  --frontend-concurrency <csv>
  --frontend-requests <n>
  --frontend-repeats <n>
  --skip-frontend

SWE metrics:
  --swe-timeout-secs <n>               default 3600
  --swe-run-mutation                   attempt cargo mutants
  --swe-mutation-score <float>         manual override
  --skip-swe

Post-processing:
  --skip-stats
  --skip-plot
  --skip-all-quality                   == --skip-soak --skip-frontend --skip-swe
  -h, --help
EOF
}

is_int() {
  [[ "$1" =~ ^[0-9]+$ ]]
}

require_int() {
  local value="$1"
  local name="$2"
  if ! is_int "${value}"; then
    echo "Invalid integer for ${name}: ${value}" >&2
    exit 1
  fi
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --base-url) BASE_URL="${2:?}"; shift 2 ;;
      --admin-id) ADMIN_ID="${2:?}"; shift 2 ;;
      --admin-pw) ADMIN_PW="${2:?}"; shift 2 ;;
      --output-root) OUTPUT_ROOT="${2:?}"; shift 2 ;;
      --baseline-root) BASELINE_ROOT="${2:?}"; shift 2 ;;
      --dry-run) DRY_RUN=1; shift ;;

      --rest-profile) REST_PROFILE="${2:?}"; shift 2 ;;
      --rest-matrix-file) REST_MATRIX_FILE="${2:?}"; shift 2 ;;
      --rest-repeats) REST_REPEATS="${2:?}"; shift 2 ;;
      --skip-rest) SKIP_REST=1; shift ;;

      --ws-users) WS_USERS="${2:?}"; shift 2 ;;
      --ws-duration-secs) WS_DURATION_SECS="${2:?}"; shift 2 ;;
      --ws-chat-rate) WS_CHAT_RATE="${2:?}"; shift 2 ;;
      --ws-step-interval-secs) WS_STEP_INTERVAL_SECS="${2:?}"; shift 2 ;;
      --ws-repeats) WS_REPEATS="${2:?}"; shift 2 ;;
      --skip-ws) SKIP_WS=1; shift ;;

      --ops-scenarios) OPS_SCENARIOS="${2:?}"; shift 2 ;;
      --ops-profile) OPS_PROFILE="${2:?}"; shift 2 ;;
      --ops-upload-requests) OPS_UPLOAD_REQUESTS="${2:?}"; shift 2 ;;
      --ops-codeserver-iterations) OPS_CODESERVER_ITERATIONS="${2:?}"; shift 2 ;;
      --ops-repeats) OPS_REPEATS="${2:?}"; shift 2 ;;
      --skip-ops) SKIP_OPS=1; shift ;;

      --soak-duration-secs) SOAK_DURATION_SECS="${2:?}"; shift 2 ;;
      --soak-cycle-interval-secs) SOAK_CYCLE_INTERVAL_SECS="${2:?}"; shift 2 ;;
      --soak-backend-pid) SOAK_BACKEND_PID="${2:?}"; shift 2 ;;
      --soak-local-attendees) SOAK_LOCAL_ATTENDEES="${2:?}"; shift 2 ;;
      --soak-local-read-requests) SOAK_LOCAL_READ_REQUESTS="${2:?}"; shift 2 ;;
      --soak-local-write-requests) SOAK_LOCAL_WRITE_REQUESTS="${2:?}"; shift 2 ;;
      --soak-local-read-concurrency) SOAK_LOCAL_READ_CONCURRENCY="${2:?}"; shift 2 ;;
      --soak-local-write-concurrency) SOAK_LOCAL_WRITE_CONCURRENCY="${2:?}"; shift 2 ;;
      --soak-ws-users) SOAK_WS_USERS="${2:?}"; shift 2 ;;
      --skip-soak) SKIP_SOAK=1; shift ;;

      --frontend-base-url) FRONTEND_BASE_URL="${2:?}"; shift 2 ;;
      --frontend-routes) FRONTEND_ROUTES="${2:?}"; shift 2 ;;
      --frontend-concurrency) FRONTEND_CONCURRENCY="${2:?}"; shift 2 ;;
      --frontend-requests) FRONTEND_REQUESTS="${2:?}"; shift 2 ;;
      --frontend-repeats) FRONTEND_REPEATS="${2:?}"; shift 2 ;;
      --skip-frontend) SKIP_FRONTEND=1; shift ;;

      --swe-timeout-secs) SWE_TIMEOUT_SECS="${2:?}"; shift 2 ;;
      --swe-run-mutation) SWE_RUN_MUTATION=1; shift ;;
      --swe-mutation-score) SWE_MUTATION_SCORE="${2:?}"; shift 2 ;;
      --skip-swe) SKIP_SWE=1; shift ;;

      --skip-stats) SKIP_STATS=1; shift ;;
      --skip-plot) SKIP_PLOT=1; shift ;;
      --skip-all-quality)
        SKIP_SOAK=1
        SKIP_FRONTEND=1
        SKIP_SWE=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        echo "Unknown argument: $1" >&2
        usage
        exit 1
        ;;
    esac
  done
}

print_cmd() {
  local -a cmd=("$@")
  printf '  $'
  for token in "${cmd[@]}"; do
    printf ' %q' "${token}"
  done
  printf '\n'
}

run_cmd() {
  local -a cmd=("$@")
  print_cmd "${cmd[@]}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    return 0
  fi
  set +e
  "${cmd[@]}"
  local exit_code=$?
  set -e
  return "${exit_code}"
}

run_stage() {
  local label="$1"
  shift
  local started_at
  started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  local started_epoch
  started_epoch="$(date +%s)"

  if run_cmd "$@"; then
    local ended_at
    ended_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    local ended_epoch
    ended_epoch="$(date +%s)"
    local duration_sec
    duration_sec=$((ended_epoch - started_epoch))
    STAGE_RESULTS+=("${started_at},${ended_at},OK,${label},0,${duration_sec}")
  else
    local exit_code=$?
    local ended_at
    ended_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    local ended_epoch
    ended_epoch="$(date +%s)"
    local duration_sec
    duration_sec=$((ended_epoch - started_epoch))
    STAGE_FAILURES=$((STAGE_FAILURES + 1))
    STAGE_RESULTS+=("${started_at},${ended_at},FAIL,${label},${exit_code},${duration_sec}")
    echo "[WARN] stage failed: ${label} (exit=${exit_code})" >&2
  fi
}

flush_stage_results() {
  local out="${STAGE_STATUS_FILE}"
  {
    echo "started_at_utc,ended_at_utc,status,label,exit_code,duration_sec"
    for row in "${STAGE_RESULTS[@]}"; do
      echo "${row}"
    done
  } > "${out}"
}

write_metadata() {
  local git_commit
  git_commit="$(git -C "${REPO_ROOT}" rev-parse --short HEAD 2>/dev/null || echo "unknown")"
  local git_branch
  git_branch="$(git -C "${REPO_ROOT}" branch --show-current 2>/dev/null || echo "unknown")"
  local uname_s
  uname_s="$(uname -s)"
  local uname_r
  uname_r="$(uname -r)"
  local uname_m
  uname_m="$(uname -m)"
  local rustc_ver
  rustc_ver="$(rustc --version 2>/dev/null || true)"
  local cargo_ver
  cargo_ver="$(cargo --version 2>/dev/null || true)"
  local bun_ver
  bun_ver="$(bun --version 2>/dev/null || true)"

  cat > "${RUN_METADATA_FILE}" <<EOF
{
  "generated_at_utc": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "run_tag": "${RUN_TAG}",
  "repo_root": "${REPO_ROOT}",
  "backend_dir": "${BACKEND_DIR}",
  "frontend_dir": "${FRONTEND_DIR}",
  "git_branch": "${git_branch}",
  "git_commit": "${git_commit}",
  "platform": {
    "os": "${uname_s}",
    "release": "${uname_r}",
    "arch": "${uname_m}"
  },
  "tool_versions": {
    "rustc": "${rustc_ver}",
    "cargo": "${cargo_ver}",
    "bun": "${bun_ver}"
  },
  "config": {
    "base_url": "${BASE_URL}",
    "admin_id": "${ADMIN_ID}",
    "frontend_base_url": "${FRONTEND_BASE_URL}",
    "rest_profile": "${REST_PROFILE}",
    "rest_matrix_file": "${REST_MATRIX_FILE}",
    "rest_repeats": ${REST_REPEATS},
    "ws_users": "${WS_USERS}",
    "ws_duration_secs": ${WS_DURATION_SECS},
    "ws_chat_rate": ${WS_CHAT_RATE},
    "ws_step_interval_secs": ${WS_STEP_INTERVAL_SECS},
    "ws_repeats": ${WS_REPEATS},
    "ops_scenarios": "${OPS_SCENARIOS}",
    "ops_profile": "${OPS_PROFILE}",
    "ops_upload_requests": ${OPS_UPLOAD_REQUESTS},
    "ops_codeserver_iterations": ${OPS_CODESERVER_ITERATIONS},
    "ops_repeats": ${OPS_REPEATS},
    "soak_duration_secs": ${SOAK_DURATION_SECS},
    "soak_cycle_interval_secs": ${SOAK_CYCLE_INTERVAL_SECS},
    "soak_backend_pid": "${SOAK_BACKEND_PID}",
    "soak_local_attendees": ${SOAK_LOCAL_ATTENDEES},
    "soak_local_read_requests": ${SOAK_LOCAL_READ_REQUESTS},
    "soak_local_write_requests": ${SOAK_LOCAL_WRITE_REQUESTS},
    "soak_local_read_concurrency": ${SOAK_LOCAL_READ_CONCURRENCY},
    "soak_local_write_concurrency": ${SOAK_LOCAL_WRITE_CONCURRENCY},
    "soak_ws_users": "${SOAK_WS_USERS}",
    "frontend_routes": "${FRONTEND_ROUTES}",
    "frontend_concurrency": "${FRONTEND_CONCURRENCY}",
    "frontend_requests": ${FRONTEND_REQUESTS},
    "frontend_repeats": ${FRONTEND_REPEATS},
    "swe_timeout_secs": ${SWE_TIMEOUT_SECS},
    "swe_run_mutation": ${SWE_RUN_MUTATION},
    "dry_run": ${DRY_RUN}
  }
}
EOF
}

main() {
  parse_args "$@"
  RUN_TAG="$(date +"%Y%m%d-%H%M%S")"

  require_int "${REST_REPEATS}" "--rest-repeats"
  require_int "${WS_DURATION_SECS}" "--ws-duration-secs"
  require_int "${WS_STEP_INTERVAL_SECS}" "--ws-step-interval-secs"
  require_int "${WS_REPEATS}" "--ws-repeats"
  require_int "${OPS_UPLOAD_REQUESTS}" "--ops-upload-requests"
  require_int "${OPS_CODESERVER_ITERATIONS}" "--ops-codeserver-iterations"
  require_int "${OPS_REPEATS}" "--ops-repeats"
  require_int "${SOAK_DURATION_SECS}" "--soak-duration-secs"
  require_int "${SOAK_CYCLE_INTERVAL_SECS}" "--soak-cycle-interval-secs"
  require_int "${SOAK_LOCAL_ATTENDEES}" "--soak-local-attendees"
  require_int "${SOAK_LOCAL_READ_REQUESTS}" "--soak-local-read-requests"
  require_int "${SOAK_LOCAL_WRITE_REQUESTS}" "--soak-local-write-requests"
  require_int "${SOAK_LOCAL_READ_CONCURRENCY}" "--soak-local-read-concurrency"
  require_int "${SOAK_LOCAL_WRITE_CONCURRENCY}" "--soak-local-write-concurrency"
  require_int "${FRONTEND_REQUESTS}" "--frontend-requests"
  require_int "${FRONTEND_REPEATS}" "--frontend-repeats"
  require_int "${SWE_TIMEOUT_SECS}" "--swe-timeout-secs"
  if [[ -n "${SOAK_BACKEND_PID}" ]]; then
    require_int "${SOAK_BACKEND_PID}" "--soak-backend-pid"
  fi

  if [[ -z "${OUTPUT_ROOT}" ]]; then
    OUTPUT_ROOT="${BACKEND_DIR}/bench-results/all-${RUN_TAG}"
  fi
  RUN_METADATA_FILE="${OUTPUT_ROOT}/run-metadata-${RUN_TAG}.json"
  STAGE_STATUS_FILE="${OUTPUT_ROOT}/stage-status-${RUN_TAG}.csv"
  STATS_JSON_FILE="${OUTPUT_ROOT}/stats/paper-stats-${RUN_TAG}.json"
  STATS_CSV_FILE="${OUTPUT_ROOT}/stats/paper-stats-${RUN_TAG}.csv"
  PLOTS_DIR="${OUTPUT_ROOT}/plots-${RUN_TAG}"

  mkdir -p "${OUTPUT_ROOT}"
  mkdir -p "${OUTPUT_ROOT}/rest" "${OUTPUT_ROOT}/ws" "${OUTPUT_ROOT}/ops" "${OUTPUT_ROOT}/soak" "${OUTPUT_ROOT}/frontend" "${OUTPUT_ROOT}/swe" "${OUTPUT_ROOT}/stats" "${PLOTS_DIR}"
  write_metadata

  echo "Benchmark output root: ${OUTPUT_ROOT}"

  cd "${BACKEND_DIR}"

  if [[ "${SKIP_REST}" -eq 0 ]]; then
    echo "[1/8] REST matrix benchmark"
    local -a rest_cmd=(
      ./bench/run_matrix.sh
      --base-url "${BASE_URL}"
      --admin-id "${ADMIN_ID}"
      --admin-pw "${ADMIN_PW}"
      --repeats "${REST_REPEATS}"
      --output-dir "${OUTPUT_ROOT}/rest"
    )
    if [[ -n "${REST_MATRIX_FILE}" ]]; then
      rest_cmd+=(--matrix-file "${REST_MATRIX_FILE}")
    else
      rest_cmd+=(--profile "${REST_PROFILE}")
    fi
    run_stage "rest_matrix" "${rest_cmd[@]}"
  else
    echo "[1/8] REST matrix benchmark skipped"
  fi

  if [[ "${SKIP_WS}" -eq 0 ]]; then
    echo "[2/8] WebSocket benchmark"
    for repeat in $(seq 1 "${WS_REPEATS}"); do
      repeat_tag="$(printf "r%02d" "${repeat}")"
      run_stage "ws_repeat_${repeat_tag}" \
        cargo run --release --bin ws_bench -- \
        --base-url "${BASE_URL}" \
        --admin-id "${ADMIN_ID}" \
        --admin-pw "${ADMIN_PW}" \
        --users "${WS_USERS}" \
        --duration-secs "${WS_DURATION_SECS}" \
        --chat-rate "${WS_CHAT_RATE}" \
        --step-interval-secs "${WS_STEP_INTERVAL_SECS}" \
        --output "${OUTPUT_ROOT}/ws/ws-${repeat_tag}-${RUN_TAG}.json"
    done
  else
    echo "[2/8] WebSocket benchmark skipped"
  fi

  if [[ "${SKIP_OPS}" -eq 0 ]]; then
    echo "[3/8] Ops benchmark (upload/backup/codeserver)"
    for repeat in $(seq 1 "${OPS_REPEATS}"); do
      repeat_tag="$(printf "r%02d" "${repeat}")"
      run_stage "ops_repeat_${repeat_tag}" \
        cargo run --release --bin ops_bench -- \
        --base-url "${BASE_URL}" \
        --admin-id "${ADMIN_ID}" \
        --admin-pw "${ADMIN_PW}" \
        --profile "${OPS_PROFILE}" \
        --scenarios "${OPS_SCENARIOS}" \
        --upload-requests "${OPS_UPLOAD_REQUESTS}" \
        --codeserver-iterations "${OPS_CODESERVER_ITERATIONS}" \
        --output "${OUTPUT_ROOT}/ops/ops-${repeat_tag}-${RUN_TAG}.json"
    done
  else
    echo "[3/8] Ops benchmark skipped"
  fi

  if [[ "${SKIP_FRONTEND}" -eq 0 ]]; then
    echo "[4/8] Frontend render/load benchmark"
    run_stage "frontend_render" \
      python3 ./bench/frontend_render_bench.py \
      --base-url "${FRONTEND_BASE_URL}" \
      --routes "${FRONTEND_ROUTES}" \
      --concurrency "${FRONTEND_CONCURRENCY}" \
      --requests "${FRONTEND_REQUESTS}" \
      --repeats "${FRONTEND_REPEATS}" \
      --output "${OUTPUT_ROOT}/frontend/frontend-bench-${RUN_TAG}.json"
  else
    echo "[4/8] Frontend benchmark skipped"
  fi

  if [[ "${SKIP_SOAK}" -eq 0 ]]; then
    echo "[5/8] Soak benchmark"
    local -a soak_cmd=(
      python3 ./bench/soak_bench.py
      --base-url "${BASE_URL}"
      --admin-id "${ADMIN_ID}"
      --admin-pw "${ADMIN_PW}"
      --duration-secs "${SOAK_DURATION_SECS}"
      --cycle-interval-secs "${SOAK_CYCLE_INTERVAL_SECS}"
      --local-attendees "${SOAK_LOCAL_ATTENDEES}"
      --local-read-requests "${SOAK_LOCAL_READ_REQUESTS}"
      --local-write-requests "${SOAK_LOCAL_WRITE_REQUESTS}"
      --local-read-concurrency "${SOAK_LOCAL_READ_CONCURRENCY}"
      --local-write-concurrency "${SOAK_LOCAL_WRITE_CONCURRENCY}"
      --ws-users "${SOAK_WS_USERS:-$(echo "${WS_USERS}" | cut -d',' -f1)}"
      --ws-duration-secs "$(( WS_DURATION_SECS > 45 ? 45 : WS_DURATION_SECS ))"
      --ws-chat-rate "${WS_CHAT_RATE}"
      --ws-step-interval-secs "${WS_STEP_INTERVAL_SECS}"
      --output "${OUTPUT_ROOT}/soak/soak-${RUN_TAG}.json"
    )
    if [[ -n "${SOAK_BACKEND_PID}" ]]; then
      soak_cmd+=(--backend-pid "${SOAK_BACKEND_PID}")
    fi
    run_stage "soak" "${soak_cmd[@]}"
  else
    echo "[5/8] Soak benchmark skipped"
  fi

  if [[ "${SKIP_SWE}" -eq 0 ]]; then
    echo "[6/8] SWE metrics"
    local -a swe_cmd=(
      python3 ./bench/swe_metrics.py
      --backend-dir "${BACKEND_DIR}"
      --frontend-dir "${FRONTEND_DIR}"
      --timeout-secs "${SWE_TIMEOUT_SECS}"
      --output "${OUTPUT_ROOT}/swe/swe-metrics-${RUN_TAG}.json"
    )
    if [[ "${SWE_RUN_MUTATION}" -eq 1 ]]; then
      swe_cmd+=(--run-mutation)
    fi
    if [[ -n "${SWE_MUTATION_SCORE}" ]]; then
      swe_cmd+=(--mutation-score "${SWE_MUTATION_SCORE}")
    fi
    run_stage "swe_metrics" "${swe_cmd[@]}"
  else
    echo "[6/8] SWE metrics skipped"
  fi

  if [[ "${SKIP_STATS}" -eq 0 ]]; then
    echo "[7/8] Statistical aggregation"
    local -a stats_cmd=(
      python3 ./bench/analyze_paper_stats.py
      --results-root "${OUTPUT_ROOT}"
      --output "${STATS_JSON_FILE}"
      --csv "${STATS_CSV_FILE}"
    )
    if [[ -n "${BASELINE_ROOT}" ]]; then
      stats_cmd+=(--baseline-root "${BASELINE_ROOT}")
    fi
    run_stage "stats" "${stats_cmd[@]}"
  else
    echo "[7/8] Statistical aggregation skipped"
  fi

  if [[ "${SKIP_PLOT}" -eq 0 && "${SKIP_STATS}" -eq 0 ]]; then
    echo "[8/8] Plot generation"
    run_stage "plots" \
      python3 ./bench/plot_paper.py \
      --csv "${STATS_CSV_FILE}" \
      --out-dir "${PLOTS_DIR}"
  else
    echo "[8/8] Plot generation skipped"
  fi

  echo ""
  echo "All done."
  echo "Artifacts:"
  echo "  - ${RUN_METADATA_FILE}"
  echo "  - ${OUTPUT_ROOT}/rest"
  echo "  - ${OUTPUT_ROOT}/ws"
  echo "  - ${OUTPUT_ROOT}/ops"
  echo "  - ${OUTPUT_ROOT}/frontend"
  echo "  - ${OUTPUT_ROOT}/soak"
  echo "  - ${OUTPUT_ROOT}/swe"
  echo "  - ${STATS_JSON_FILE}"
  echo "  - ${STATS_CSV_FILE}"
  echo "  - ${PLOTS_DIR}/index.html"
  flush_stage_results
  echo "  - ${STAGE_STATUS_FILE}"

  if [[ "${STAGE_FAILURES}" -gt 0 ]]; then
    echo ""
    echo "Completed with stage failures: ${STAGE_FAILURES}" >&2
    exit 2
  fi
}

main "$@"
