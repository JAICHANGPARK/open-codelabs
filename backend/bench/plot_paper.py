#!/usr/bin/env python3
"""
Generate simple SVG bar charts from paper-stats CSV.

Input columns expected:
- suite, scenario, metric, median, ci95_low, ci95_high
"""

from __future__ import annotations

import argparse
import csv
import html
import math
import re
from collections import defaultdict
from pathlib import Path
from typing import Any, Dict, List, Tuple


def sanitize(name: str) -> str:
    name = name.strip().lower()
    name = re.sub(r"[^a-z0-9._-]+", "-", name)
    name = re.sub(r"-{2,}", "-", name).strip("-")
    return name or "chart"


def f(value: Any, default: float = 0.0) -> float:
    try:
        return float(value)
    except Exception:
        return default


def load_rows(csv_path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    with csv_path.open("r", encoding="utf-8", newline="") as f_csv:
        reader = csv.DictReader(f_csv)
        for row in reader:
            rows.append(row)
    return rows


def draw_svg(
    title: str,
    rows: List[Dict[str, Any]],
    width: int = 1280,
    height: int = 720,
) -> str:
    pad_left = 120
    pad_right = 40
    pad_top = 70
    pad_bottom = 160
    plot_w = width - pad_left - pad_right
    plot_h = height - pad_top - pad_bottom
    if plot_w <= 0 or plot_h <= 0:
        raise ValueError("Invalid chart size")

    max_value = 0.0
    for row in rows:
        max_value = max(max_value, f(row.get("ci95_high")), f(row.get("median")))
    if max_value <= 0:
        max_value = 1.0
    max_value *= 1.1

    n = len(rows)
    slot = plot_w / max(1, n)
    bar_w = min(64.0, slot * 0.58)

    def y_of(v: float) -> float:
        return pad_top + plot_h - (v / max_value) * plot_h

    lines: List[str] = []
    lines.append(f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">')
    lines.append('<rect x="0" y="0" width="100%" height="100%" fill="white"/>')
    lines.append(f'<text x="{pad_left}" y="36" font-size="24" font-family="Menlo, Consolas, monospace" fill="#111">{html.escape(title)}</text>')

    # Grid + y ticks
    for i in range(6):
        v = (max_value / 5.0) * i
        y = y_of(v)
        lines.append(f'<line x1="{pad_left}" y1="{y:.2f}" x2="{pad_left + plot_w}" y2="{y:.2f}" stroke="#e5e7eb" stroke-width="1"/>')
        lines.append(
            f'<text x="{pad_left - 12}" y="{y + 4:.2f}" text-anchor="end" '
            f'font-size="12" font-family="Menlo, Consolas, monospace" fill="#374151">{v:.2f}</text>'
        )

    # Axes
    lines.append(f'<line x1="{pad_left}" y1="{pad_top}" x2="{pad_left}" y2="{pad_top + plot_h}" stroke="#111827" stroke-width="2"/>')
    lines.append(
        f'<line x1="{pad_left}" y1="{pad_top + plot_h}" x2="{pad_left + plot_w}" y2="{pad_top + plot_h}" '
        f'stroke="#111827" stroke-width="2"/>'
    )

    # Bars + labels + CI whiskers
    for i, row in enumerate(rows):
        median = f(row.get("median"))
        lo = f(row.get("ci95_low"), median)
        hi = f(row.get("ci95_high"), median)
        scenario = str(row.get("scenario", f"s{i+1}"))
        x_center = pad_left + slot * i + slot / 2.0
        x = x_center - bar_w / 2.0
        y = y_of(median)
        h = (pad_top + plot_h) - y
        color = "#1d4ed8" if i % 2 == 0 else "#0f766e"
        lines.append(f'<rect x="{x:.2f}" y="{y:.2f}" width="{bar_w:.2f}" height="{h:.2f}" fill="{color}" opacity="0.88"/>')

        # CI whiskers
        y_lo = y_of(lo)
        y_hi = y_of(hi)
        lines.append(f'<line x1="{x_center:.2f}" y1="{y_lo:.2f}" x2="{x_center:.2f}" y2="{y_hi:.2f}" stroke="#111827" stroke-width="2"/>')
        lines.append(f'<line x1="{x_center - 7:.2f}" y1="{y_lo:.2f}" x2="{x_center + 7:.2f}" y2="{y_lo:.2f}" stroke="#111827" stroke-width="2"/>')
        lines.append(f'<line x1="{x_center - 7:.2f}" y1="{y_hi:.2f}" x2="{x_center + 7:.2f}" y2="{y_hi:.2f}" stroke="#111827" stroke-width="2"/>')

        lines.append(
            f'<text x="{x_center:.2f}" y="{y - 8:.2f}" text-anchor="middle" '
            f'font-size="11" font-family="Menlo, Consolas, monospace" fill="#111827">{median:.2f}</text>'
        )

        # Rotated scenario labels
        safe_label = html.escape(scenario[:64])
        tx = x_center
        ty = pad_top + plot_h + 12
        lines.append(
            f'<g transform="translate({tx:.2f},{ty:.2f}) rotate(40)">'
            f'<text x="0" y="0" font-size="11" font-family="Menlo, Consolas, monospace" fill="#374151">{safe_label}</text>'
            f"</g>"
        )

    lines.append(
        '<text x="24" y="26" font-size="12" font-family="Menlo, Consolas, monospace" fill="#374151">'
        "Bars: median, whiskers: bootstrap 95% CI"
        "</text>"
    )
    lines.append("</svg>")
    return "\n".join(lines)


def write_index(out_dir: Path, charts: List[Tuple[str, str]]) -> None:
    items = "\n".join(
        [f'<li><a href="{html.escape(file_name)}">{html.escape(title)}</a></li>' for title, file_name in charts]
    )
    content = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Benchmark Charts</title>
  <style>
    body {{
      margin: 24px;
      font-family: Menlo, Consolas, monospace;
      color: #111827;
      background: #f8fafc;
    }}
    a {{ color: #1d4ed8; text-decoration: none; }}
    a:hover {{ text-decoration: underline; }}
    li {{ margin: 10px 0; }}
  </style>
</head>
<body>
  <h1>Benchmark Charts</h1>
  <ul>
    {items}
  </ul>
</body>
</html>
"""
    (out_dir / "index.html").write_text(content, encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate SVG charts from paper-stats CSV")
    parser.add_argument("--csv", required=True, help="Input CSV path")
    parser.add_argument("--out-dir", required=True, help="Output directory for SVG charts")
    args = parser.parse_args()

    csv_path = Path(args.csv).resolve()
    out_dir = Path(args.out_dir).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    rows = load_rows(csv_path)
    if not rows:
        raise SystemExit(f"No rows found in {csv_path}")

    groups: Dict[Tuple[str, str], List[Dict[str, Any]]] = defaultdict(list)
    for row in rows:
        key = (row.get("suite", "suite"), row.get("metric", "metric"))
        groups[key].append(row)

    charts: List[Tuple[str, str]] = []
    for (suite, metric), g_rows in sorted(groups.items()):
        # limit dense charts for readability
        g_rows = sorted(g_rows, key=lambda r: r.get("scenario", ""))[:60]
        title = f"{suite} :: {metric}"
        svg = draw_svg(title, g_rows)
        file_name = sanitize(f"{suite}-{metric}.svg")
        (out_dir / file_name).write_text(svg, encoding="utf-8")
        charts.append((title, file_name))

    write_index(out_dir, charts)
    print(f"Charts written to: {out_dir}")


if __name__ == "__main__":
    main()
