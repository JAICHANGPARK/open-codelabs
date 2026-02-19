#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

BASE_URL="${BENCH_BASE_URL:-http://localhost:8080}"
ADMIN_ID="${ADMIN_ID:-admin}"
ADMIN_PW="${ADMIN_PW:-admin}"
REPEATS="${BENCH_REPEATS:-10}"
PROFILE="paper"
MATRIX_FILE=""
OUTPUT_DIR=""
DRY_RUN=0
RUN_TAG=""

usage() {
  cat <<'EOF'
Run local benchmark matrix with repeated trials.

Usage:
  ./bench/run_matrix.sh [options]

Options:
  --base-url <url>              Backend URL (default: http://localhost:8080)
  --admin-id <id>               Admin ID (default: env ADMIN_ID or "admin")
  --admin-pw <pw>               Admin password (default: env ADMIN_PW or "admin")
  --repeats <n>                 Repeats per matrix row (default: 10)
  --profile <quick|paper>       Built-in matrix profile (default: paper)
  --matrix-file <path>          Custom CSV matrix file (overrides --profile)
  --output-dir <path>           Output directory (default: auto timestamped)
  --dry-run                     Print commands without executing
  -h, --help                    Show this help

CSV format:
  scenario,attendees,read_requests,write_requests,read_concurrency,write_concurrency
EOF
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --base-url)
        BASE_URL="${2:?Missing value for --base-url}"
        shift 2
        ;;
      --admin-id)
        ADMIN_ID="${2:?Missing value for --admin-id}"
        shift 2
        ;;
      --admin-pw)
        ADMIN_PW="${2:?Missing value for --admin-pw}"
        shift 2
        ;;
      --repeats)
        REPEATS="${2:?Missing value for --repeats}"
        shift 2
        ;;
      --profile)
        PROFILE="${2:?Missing value for --profile}"
        shift 2
        ;;
      --matrix-file)
        MATRIX_FILE="${2:?Missing value for --matrix-file}"
        shift 2
        ;;
      --output-dir)
        OUTPUT_DIR="${2:?Missing value for --output-dir}"
        shift 2
        ;;
      --dry-run)
        DRY_RUN=1
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

require_integer() {
  local value="$1"
  local label="$2"
  if ! [[ "$value" =~ ^[0-9]+$ ]]; then
    echo "Invalid integer for ${label}: ${value}" >&2
    exit 1
  fi
}

trim() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

resolve_matrix_file() {
  if [[ -n "${MATRIX_FILE}" ]]; then
    printf '%s' "${MATRIX_FILE}"
    return 0
  fi

  case "${PROFILE}" in
    quick)
      printf '%s' "${SCRIPT_DIR}/matrix.quick.csv"
      ;;
    paper)
      printf '%s' "${SCRIPT_DIR}/matrix.paper.csv"
      ;;
    *)
      echo "Unknown profile: ${PROFILE}" >&2
      exit 1
      ;;
  esac
}

write_metadata() {
  local matrix_file="$1"
  local metadata_file="$2"

  cat > "${metadata_file}" <<EOF
{
  "generated_at_utc": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "run_tag": "${RUN_TAG}",
  "base_url": "${BASE_URL}",
  "admin_id": "${ADMIN_ID}",
  "repeats": ${REPEATS},
  "profile": "${PROFILE}",
  "matrix_file": "${matrix_file}",
  "dry_run": ${DRY_RUN}
}
EOF
}

main() {
  parse_args "$@"
  RUN_TAG="$(date +"%Y%m%d-%H%M%S")"
  require_integer "${REPEATS}" "--repeats"
  if [[ "${REPEATS}" -le 0 ]]; then
    echo "--repeats must be >= 1" >&2
    exit 1
  fi

  local matrix_file
  matrix_file="$(resolve_matrix_file)"
  if [[ ! -f "${matrix_file}" ]]; then
    echo "Matrix file not found: ${matrix_file}" >&2
    exit 1
  fi

  if [[ -z "${OUTPUT_DIR}" ]]; then
    OUTPUT_DIR="${BACKEND_DIR}/bench-results/matrix-${RUN_TAG}"
  fi

  mkdir -p "${OUTPUT_DIR}"
  local summary_file="${OUTPUT_DIR}/run-summary-${RUN_TAG}.csv"
  local metadata_file="${OUTPUT_DIR}/run-metadata-${RUN_TAG}.json"
  local matrix_copy="${OUTPUT_DIR}/matrix-${RUN_TAG}.csv"
  cp "${matrix_file}" "${matrix_copy}"
  write_metadata "${matrix_file}" "${metadata_file}"

  echo "timestamp_utc,scenario,repeat,status,exit_code,output_file,attendees,read_requests,write_requests,read_concurrency,write_concurrency" > "${summary_file}"

  local total_runs=0
  local success_runs=0
  local failed_runs=0

  cd "${BACKEND_DIR}"

  while IFS=',' read -r scenario attendees read_requests write_requests read_concurrency write_concurrency; do
    scenario="$(trim "${scenario}")"
    attendees="$(trim "${attendees}")"
    read_requests="$(trim "${read_requests}")"
    write_requests="$(trim "${write_requests}")"
    read_concurrency="$(trim "${read_concurrency}")"
    write_concurrency="$(trim "${write_concurrency}")"

    if [[ -z "${scenario}" || "${scenario}" == "scenario" ]]; then
      continue
    fi
    if [[ "${scenario}" == \#* ]]; then
      continue
    fi

    require_integer "${attendees}" "attendees(${scenario})"
    require_integer "${read_requests}" "read_requests(${scenario})"
    require_integer "${write_requests}" "write_requests(${scenario})"
    require_integer "${read_concurrency}" "read_concurrency(${scenario})"
    require_integer "${write_concurrency}" "write_concurrency(${scenario})"

    for repeat in $(seq 1 "${REPEATS}"); do
      total_runs=$((total_runs + 1))
      local repeat_tag
      repeat_tag="$(printf "r%02d" "${repeat}")"
      local output_file="${OUTPUT_DIR}/${scenario}-${repeat_tag}-${RUN_TAG}.json"

      local cmd=(
        cargo run --release --bin local_bench -- 
        --base-url "${BASE_URL}"
        --admin-id "${ADMIN_ID}"
        --admin-pw "${ADMIN_PW}"
        --attendees "${attendees}"
        --read-requests "${read_requests}"
        --write-requests "${write_requests}"
        --read-concurrency "${read_concurrency}"
        --write-concurrency "${write_concurrency}"
        --output "${output_file}"
      )

      local timestamp
      timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

      echo ""
      echo "[${total_runs}] scenario=${scenario} repeat=${repeat_tag}"
      if [[ "${DRY_RUN}" -eq 1 ]]; then
        printf 'DRY RUN:'
        for token in "${cmd[@]}"; do
          printf ' %q' "${token}"
        done
        printf '\n'
        success_runs=$((success_runs + 1))
        echo "${timestamp},${scenario},${repeat},DRY_RUN,0,${output_file},${attendees},${read_requests},${write_requests},${read_concurrency},${write_concurrency}" >> "${summary_file}"
        continue
      fi

      set +e
      "${cmd[@]}"
      local exit_code=$?
      set -e

      if [[ "${exit_code}" -eq 0 ]]; then
        success_runs=$((success_runs + 1))
        echo "${timestamp},${scenario},${repeat},OK,0,${output_file},${attendees},${read_requests},${write_requests},${read_concurrency},${write_concurrency}" >> "${summary_file}"
      else
        failed_runs=$((failed_runs + 1))
        echo "${timestamp},${scenario},${repeat},FAIL,${exit_code},${output_file},${attendees},${read_requests},${write_requests},${read_concurrency},${write_concurrency}" >> "${summary_file}"
        echo "Run failed (scenario=${scenario}, repeat=${repeat_tag}, exit_code=${exit_code})" >&2
      fi
    done
  done < "${matrix_file}"

  echo ""
  echo "Matrix run complete."
  echo "- output_dir: ${OUTPUT_DIR}"
  echo "- summary:    ${summary_file}"
  echo "- metadata:   ${metadata_file}"
  echo "- total:      ${total_runs}"
  echo "- success:    ${success_runs}"
  echo "- failed:     ${failed_runs}"

  if [[ "${failed_runs}" -gt 0 ]]; then
    exit 2
  fi
}

main "$@"
