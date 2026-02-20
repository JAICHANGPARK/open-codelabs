#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
FIG_DIR="$ROOT/figures"

echo "[mermaid] rendering Mermaid diagrams to PNG..."

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig1-architecture.mmd" \
  -o "$FIG_DIR/fig1-architecture.pdf" \
  -b transparent -t neutral

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig1-architecture.mmd" \
  -o "$FIG_DIR/fig1-architecture.png" \
  -b transparent -t neutral \
  -w 2600 -H 1600 -s 3

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig2-live-signal.mmd" \
  -o "$FIG_DIR/fig2-live-signal.pdf" \
  -b transparent -t neutral

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig2-live-signal.mmd" \
  -o "$FIG_DIR/fig2-live-signal.png" \
  -b transparent -t neutral \
  -w 2600 -H 1700 -s 3

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig4-db-core.mmd" \
  -o "$FIG_DIR/fig4-db-core.pdf" \
  -b transparent -t neutral

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig4-db-core.mmd" \
  -o "$FIG_DIR/fig4-db-core.png" \
  -b transparent -t neutral \
  -w 2400 -H 1700 -s 3

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig4-db-learning.mmd" \
  -o "$FIG_DIR/fig4-db-learning.pdf" \
  -b transparent -t neutral

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig4-db-learning.mmd" \
  -o "$FIG_DIR/fig4-db-learning.png" \
  -b transparent -t neutral \
  -w 2200 -H 1500 -s 3

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig4-db-realtime.mmd" \
  -o "$FIG_DIR/fig4-db-realtime.pdf" \
  -b transparent -t neutral

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig4-db-realtime.mmd" \
  -o "$FIG_DIR/fig4-db-realtime.png" \
  -b transparent -t neutral \
  -w 2200 -H 1500 -s 3

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig4-db-assessment-ai.mmd" \
  -o "$FIG_DIR/fig4-db-assessment-ai.pdf" \
  -b transparent -t neutral

npx --cache /tmp/npm-cache-mermaid -y @mermaid-js/mermaid-cli \
  -i "$FIG_DIR/fig4-db-assessment-ai.mmd" \
  -o "$FIG_DIR/fig4-db-assessment-ai.png" \
  -b transparent -t neutral \
  -w 2200 -H 1500 -s 3

echo "[mermaid] done (pdf + png)."
