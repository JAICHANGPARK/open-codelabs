#!/usr/bin/env python3
"""
Aggregate benchmark artifacts into paper-ready statistics.

Outputs:
- JSON summary
- CSV flat table

Statistics:
- median, IQR (Q1/Q3), mean
- bootstrap 95% CI for median
- optional Mann-Whitney U p-value vs baseline
- optional Cliff's delta vs baseline
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import random
import statistics
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple


@dataclass(frozen=True)
class MetricKey:
    suite: str
    scenario: str
    metric: str


def percentile(values: List[float], q: float) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return values[0]
    xs = sorted(values)
    idx = (len(xs) - 1) * q
    lo = int(math.floor(idx))
    hi = int(math.ceil(idx))
    if lo == hi:
        return xs[lo]
    frac = idx - lo
    return xs[lo] * (1.0 - frac) + xs[hi] * frac


def bootstrap_ci_median(values: List[float], confidence: float = 0.95, rounds: int = 5000, seed: int = 42) -> Tuple[float, float]:
    if not values:
        return (0.0, 0.0)
    if len(values) == 1:
        return (values[0], values[0])
    rng = random.Random(seed)
    n = len(values)
    medians = []
    for _ in range(rounds):
        sample = [values[rng.randrange(n)] for _ in range(n)]
        medians.append(statistics.median(sample))
    medians.sort()
    alpha = 1.0 - confidence
    lo = percentile(medians, alpha / 2.0)
    hi = percentile(medians, 1.0 - alpha / 2.0)
    return lo, hi


def rankdata_with_ties(values: List[float]) -> List[float]:
    # average ranks for ties, 1-based ranks
    indexed = sorted(enumerate(values), key=lambda x: x[1])
    ranks = [0.0] * len(values)
    i = 0
    while i < len(indexed):
        j = i + 1
        while j < len(indexed) and indexed[j][1] == indexed[i][1]:
            j += 1
        avg_rank = (i + 1 + j) / 2.0
        for k in range(i, j):
            original_index = indexed[k][0]
            ranks[original_index] = avg_rank
        i = j
    return ranks


def mann_whitney_u_test(x: List[float], y: List[float]) -> Dict[str, float]:
    # Two-sided p-value using normal approximation with tie correction.
    if not x or not y:
        return {"u": 0.0, "p_value": 1.0}
    n1 = len(x)
    n2 = len(y)
    combined = x + y
    ranks = rankdata_with_ties(combined)
    r1 = sum(ranks[:n1])
    u1 = r1 - n1 * (n1 + 1) / 2.0
    u2 = n1 * n2 - u1
    u = min(u1, u2)

    n = n1 + n2
    tie_counts: Dict[float, int] = defaultdict(int)
    for v in combined:
        tie_counts[v] += 1
    tie_term = sum(c**3 - c for c in tie_counts.values())
    denom = n * (n - 1)
    if denom <= 0:
        return {"u": u, "p_value": 1.0}
    sigma_sq = (n1 * n2 / 12.0) * ((n + 1) - (tie_term / denom))
    if sigma_sq <= 0:
        return {"u": u, "p_value": 1.0}
    sigma = math.sqrt(sigma_sq)
    mu = n1 * n2 / 2.0
    # continuity correction
    if u > mu:
        z = (u - mu - 0.5) / sigma
    else:
        z = (u - mu + 0.5) / sigma
    p = 2.0 * (1.0 - 0.5 * (1.0 + math.erf(abs(z) / math.sqrt(2))))
    return {"u": u, "p_value": max(0.0, min(1.0, p))}


def cliffs_delta(x: List[float], y: List[float]) -> float:
    if not x or not y:
        return 0.0
    greater = 0
    less = 0
    for xv in x:
        for yv in y:
            if xv > yv:
                greater += 1
            elif xv < yv:
                less += 1
    total = len(x) * len(y)
    if total == 0:
        return 0.0
    return (greater - less) / total


def cliffs_effect_size(delta: float) -> str:
    ad = abs(delta)
    if ad < 0.147:
        return "negligible"
    if ad < 0.33:
        return "small"
    if ad < 0.474:
        return "medium"
    return "large"


def as_float(value: Any) -> Optional[float]:
    if isinstance(value, (int, float)):
        return float(value)
    return None


def add_metric(
    bag: Dict[MetricKey, List[float]],
    suite: str,
    scenario: str,
    metric: str,
    value: Any,
) -> None:
    v = as_float(value)
    if v is None:
        return
    bag[MetricKey(suite, scenario, metric)].append(v)


def extract_rest_metrics(path: Path, bag: Dict[MetricKey, List[float]]) -> None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return
    if "scenarios" not in data or "config" not in data:
        return
    cfg = data.get("config", {})
    suffix = (
        f"a{cfg.get('attendees','na')}_rr{cfg.get('read_requests','na')}"
        f"_wr{cfg.get('write_requests','na')}_rc{cfg.get('read_concurrency','na')}"
        f"_wc{cfg.get('write_concurrency','na')}"
    )
    for scenario in data.get("scenarios", []):
        name = scenario.get("name", "unknown")
        sc = f"{name}|{suffix}"
        lat = scenario.get("latency_ms", {})
        add_metric(bag, "rest", sc, "latency_p50_ms", lat.get("p50"))
        add_metric(bag, "rest", sc, "latency_p95_ms", lat.get("p95"))
        add_metric(bag, "rest", sc, "latency_p99_ms", lat.get("p99"))
        add_metric(bag, "rest", sc, "rps", scenario.get("requests_per_second"))
        add_metric(bag, "rest", sc, "error_rate", scenario.get("error_rate"))


def extract_ws_metrics(path: Path, bag: Dict[MetricKey, List[float]]) -> None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return
    cases = data.get("cases")
    if not isinstance(cases, list):
        return
    for case in cases:
        users = case.get("users", "na")
        sc = f"users_{users}"
        lat = case.get("e2e_latency_ms", {})
        th = case.get("throughput", {})
        add_metric(bag, "ws", sc, "e2e_p50_ms", lat.get("p50"))
        add_metric(bag, "ws", sc, "e2e_p95_ms", lat.get("p95"))
        add_metric(bag, "ws", sc, "e2e_p99_ms", lat.get("p99"))
        add_metric(bag, "ws", sc, "sent_chat_per_sec", th.get("sent_chat_per_sec"))
        add_metric(bag, "ws", sc, "recv_chat_per_sec", th.get("recv_chat_per_sec"))
        add_metric(bag, "ws", sc, "ws_disconnects", case.get("ws_disconnects"))
        add_metric(bag, "ws", sc, "ws_errors", case.get("ws_errors"))


def extract_ops_metrics(path: Path, bag: Dict[MetricKey, List[float]]) -> None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return
    upload = data.get("upload")
    if isinstance(upload, dict):
        for case in upload.get("cases", []):
            label = case.get("label", "upload")
            conc = case.get("concurrency", "na")
            sc = f"{label}|c{conc}"
            lat = case.get("latency_ms", {})
            add_metric(bag, "ops.upload", sc, "latency_p50_ms", lat.get("p50"))
            add_metric(bag, "ops.upload", sc, "latency_p95_ms", lat.get("p95"))
            add_metric(bag, "ops.upload", sc, "latency_p99_ms", lat.get("p99"))
            reqs = as_float(case.get("requests"))
            ok = as_float(case.get("success_2xx"))
            if reqs and ok is not None and reqs > 0:
                add_metric(bag, "ops.upload", sc, "success_rate", ok / reqs)

    backup = data.get("backup")
    if isinstance(backup, dict):
        for ds in backup.get("datasets", []):
            sc = str(ds.get("dataset", "dataset"))
            add_metric(bag, "ops.backup", sc, "export_ms", ds.get("export_ms"))
            add_metric(bag, "ops.backup", sc, "inspect_ms", ds.get("inspect_ms"))
            add_metric(bag, "ops.backup", sc, "restore_ms", ds.get("restore_ms"))
            size_b = as_float(ds.get("backup_size_bytes"))
            if size_b is not None:
                add_metric(bag, "ops.backup", sc, "backup_size_mb", size_b / (1024 * 1024))

    cs = data.get("codeserver")
    if isinstance(cs, dict):
        sc = "codeserver"
        add_metric(bag, "ops.codeserver", sc, "create_codeserver_ms", cs.get("create_codeserver_ms"))
        add_metric(bag, "ops.codeserver", sc, "download_workspace_ms", cs.get("download_workspace_ms"))
        size_b = as_float(cs.get("download_size_bytes"))
        if size_b is not None:
            add_metric(bag, "ops.codeserver", sc, "download_size_mb", size_b / (1024 * 1024))
        for key in ["create_branch_ms", "update_branch_ms", "create_folder_ms", "update_folder_ms"]:
            lat = cs.get(key, {})
            if isinstance(lat, dict):
                add_metric(bag, "ops.codeserver", sc, f"{key}_p95", lat.get("p95"))


def extract_frontend_metrics(path: Path, bag: Dict[MetricKey, List[float]]) -> None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return
    cases = data.get("cases")
    if not isinstance(cases, list):
        return
    for case in cases:
        route = case.get("route", "/")
        conc = case.get("concurrency", "na")
        sc = f"{route}|c{conc}"
        lat = case.get("latency_ms", {})
        add_metric(bag, "frontend", sc, "latency_p50_ms", lat.get("p50"))
        add_metric(bag, "frontend", sc, "latency_p95_ms", lat.get("p95"))
        add_metric(bag, "frontend", sc, "latency_p99_ms", lat.get("p99"))
        add_metric(bag, "frontend", sc, "rps", case.get("requests_per_second"))
        add_metric(bag, "frontend", sc, "error_rate", case.get("error_rate"))


def extract_soak_metrics(path: Path, bag: Dict[MetricKey, List[float]]) -> None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return
    summary = data.get("summary")
    if not isinstance(summary, dict):
        return
    sc = "soak"
    add_metric(bag, "stability", sc, "cycle_success_rate", summary.get("cycle_success_rate"))
    dur = summary.get("cycle_duration_ms", {})
    if isinstance(dur, dict):
        add_metric(bag, "stability", sc, "cycle_p95_ms", dur.get("p95"))
        add_metric(bag, "stability", sc, "cycle_p99_ms", dur.get("p99"))
    restart = summary.get("restart_event")
    if isinstance(restart, dict):
        ready = restart.get("ready_result")
        if isinstance(ready, dict):
            add_metric(bag, "stability", sc, "restart_recovery_ms", ready.get("recovery_ms"))


def load_metrics(results_root: Path) -> Dict[MetricKey, List[float]]:
    bag: Dict[MetricKey, List[float]] = defaultdict(list)
    if not results_root.exists():
        return bag
    for path in results_root.rglob("*.json"):
        name = path.name.lower()
        if name in {"run-metadata.json"}:
            continue
        # loader dispatch by shape + parent naming
        parent = path.parent.name.lower()
        if parent == "rest":
            extract_rest_metrics(path, bag)
            continue
        if parent == "ws":
            extract_ws_metrics(path, bag)
            continue
        if parent == "ops":
            extract_ops_metrics(path, bag)
            continue
        if parent == "frontend":
            extract_frontend_metrics(path, bag)
            continue
        if parent == "soak":
            extract_soak_metrics(path, bag)
            continue

        # Fallback shape-based extraction for non-standard layout
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            continue
        if isinstance(data, dict):
            if "scenarios" in data and "local_environment" in data:
                extract_rest_metrics(path, bag)
            elif "cases" in data and "notes" in data and data.get("config", {}).get("users") is not None:
                extract_ws_metrics(path, bag)
            elif any(k in data for k in ("upload", "backup", "codeserver")):
                extract_ops_metrics(path, bag)
            elif data.get("config", {}).get("routes") is not None and "cases" in data:
                extract_frontend_metrics(path, bag)
            elif "cycles" in data and "summary" in data:
                extract_soak_metrics(path, bag)
    return bag


def summarize_metrics(
    current: Dict[MetricKey, List[float]],
    baseline: Optional[Dict[MetricKey, List[float]]] = None,
) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    keys = sorted(current.keys(), key=lambda k: (k.suite, k.scenario, k.metric))
    for key in keys:
        values = current[key]
        if not values:
            continue
        xs = sorted(values)
        mean_v = statistics.fmean(xs)
        median_v = statistics.median(xs)
        q1 = percentile(xs, 0.25)
        q3 = percentile(xs, 0.75)
        ci_lo, ci_hi = bootstrap_ci_median(xs)
        row: Dict[str, Any] = {
            "suite": key.suite,
            "scenario": key.scenario,
            "metric": key.metric,
            "n": len(xs),
            "mean": mean_v,
            "median": median_v,
            "q1": q1,
            "q3": q3,
            "iqr": q3 - q1,
            "ci95_low": ci_lo,
            "ci95_high": ci_hi,
        }
        if baseline is not None:
            base_xs = baseline.get(key, [])
            row["baseline_n"] = len(base_xs)
            if len(base_xs) > 0:
                mw = mann_whitney_u_test(xs, base_xs)
                delta = cliffs_delta(xs, base_xs)
                row["mann_whitney_u"] = mw["u"]
                row["p_value"] = mw["p_value"]
                row["cliffs_delta"] = delta
                row["effect_size"] = cliffs_effect_size(delta)
            else:
                row["mann_whitney_u"] = None
                row["p_value"] = None
                row["cliffs_delta"] = None
                row["effect_size"] = None
        rows.append(row)
    return rows


def write_csv(path: Path, rows: Iterable[Dict[str, Any]]) -> None:
    rows = list(rows)
    path.parent.mkdir(parents=True, exist_ok=True)
    columns = [
        "suite",
        "scenario",
        "metric",
        "n",
        "mean",
        "median",
        "q1",
        "q3",
        "iqr",
        "ci95_low",
        "ci95_high",
        "baseline_n",
        "mann_whitney_u",
        "p_value",
        "cliffs_delta",
        "effect_size",
    ]
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=columns)
        writer.writeheader()
        for row in rows:
            writer.writerow(row)


def main() -> None:
    parser = argparse.ArgumentParser(description="Aggregate benchmark JSONs into paper-ready stats")
    parser.add_argument("--results-root", required=True, help="Directory containing benchmark outputs")
    parser.add_argument("--baseline-root", default="", help="Optional baseline directory for significance testing")
    parser.add_argument("--output", default="bench-results/paper-stats.json", help="Output JSON path")
    parser.add_argument("--csv", default="bench-results/paper-stats.csv", help="Output CSV path")
    args = parser.parse_args()

    results_root = Path(args.results_root).resolve()
    baseline_root = Path(args.baseline_root).resolve() if args.baseline_root else None
    out_json = Path(args.output).resolve()
    out_csv = Path(args.csv).resolve()

    current = load_metrics(results_root)
    baseline = load_metrics(baseline_root) if baseline_root else None
    rows = summarize_metrics(current, baseline)

    report = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "results_root": str(results_root),
        "baseline_root": str(baseline_root) if baseline_root else None,
        "row_count": len(rows),
        "rows": rows,
        "notes": [
            "Reported statistics: median, IQR, bootstrap 95% CI for the median.",
            "If baseline_root is provided: Mann-Whitney U p-value and Cliff's delta are included.",
        ],
    }

    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    write_csv(out_csv, rows)
    print(f"Stats JSON: {out_json}")
    print(f"Stats CSV:  {out_csv}")


if __name__ == "__main__":
    main()
