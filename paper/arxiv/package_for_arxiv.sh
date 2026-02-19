#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_NAME="${1:-open-codelabs-arxiv-source.tar.gz}"
TMP_DIR="$(mktemp -d)"
STAGE_DIR="$TMP_DIR/arxiv-src"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

mkdir -p "$STAGE_DIR"

rsync -a \
  --exclude=".DS_Store" \
  --exclude=".git" \
  --exclude=".idea" \
  --exclude=".gitignore" \
  --exclude="*.aux" \
  --exclude="*.blg" \
  --exclude="*.fdb_latexmk" \
  --exclude="*.fls" \
  --exclude="*.log" \
  --exclude="*.out" \
  --exclude="*.pdf" \
  --exclude="*.synctex.gz" \
  --exclude="*.toc" \
  --exclude="$OUTPUT_NAME" \
  --exclude="README.md" \
  --exclude="Makefile" \
  --exclude="package_for_arxiv.sh" \
  "$SCRIPT_DIR/" "$STAGE_DIR/"

# arXiv strips hidden files and directories; remove them in advance.
find "$STAGE_DIR" -mindepth 1 -name ".*" -exec rm -rf {} +

INVALID_NAMES="$(
  find "$STAGE_DIR" -type f \
    | sed "s|$STAGE_DIR/||" \
    | LC_ALL=C grep -Ev '^[A-Za-z0-9._+\-\/]+$' || true
)"

if [[ -n "$INVALID_NAMES" ]]; then
  echo "Found filenames with unsupported characters:"
  echo "$INVALID_NAMES"
  echo "Rename files before uploading to arXiv."
  exit 1
fi

tar -C "$STAGE_DIR" -czf "$SCRIPT_DIR/$OUTPUT_NAME" .
echo "Created $SCRIPT_DIR/$OUTPUT_NAME"
