#!/usr/bin/env python3
"""
Frontend render/load benchmark (local-only, dependency-free).

What this measures:
- Route HTTP latency distribution (p50/p95/p99)
- Throughput (req/s)
- Error rate and status distribution

This approximates frontend rendering load by exercising SSR/route responses
under concurrent requests.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import math
import os
import statistics
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Tuple


DEFAULT_BASE_URL = os.environ.get("FRONT_BENCH_BASE_URL", "http://localhost:5173")


@dataclass
class Sample:
    latency_ms: float
    status: int | None
    error: str | None


def parse_csv_ints(raw: str) -> List[int]:
    values = []
    for part in raw.split(","):
        part = part.strip()
        if not part:
            continue
        value = int(part)
        if value <= 0:
            raise ValueError(f"Expected positive integer, got {value}")
        values.append(value)
    if not values:
        raise ValueError("Expected at least one integer")
    return values


def parse_csv_routes(raw: str) -> List[str]:
    routes = []
    for part in raw.split(","):
        route = part.strip()
        if not route:
            continue
        if not route.startswith("/"):
            route = "/" + route
        routes.append(route)
    if not routes:
        raise ValueError("Expected at least one route")
    return routes


def percentile(values: List[float], p: float) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return values[0]
    sorted_values = sorted(values)
    idx = (len(sorted_values) - 1) * p
    lo = int(math.floor(idx))
    hi = int(math.ceil(idx))
    if lo == hi:
        return sorted_values[lo]
    frac = idx - lo
    return sorted_values[lo] * (1.0 - frac) + sorted_values[hi] * frac


def summarize(samples: List[Sample], duration_ms: float) -> Dict[str, Any]:
    latencies = [s.latency_ms for s in samples]
    statuses = Counter()
    transport_errors = Counter()
    success_2xx = 0
    http_errors = 0

    for sample in samples:
        if sample.status is not None:
            statuses[str(sample.status)] += 1
            if 200 <= sample.status < 300:
                success_2xx += 1
            else:
                http_errors += 1
        else:
            transport_errors[sample.error or "transport_error"] += 1

    transport_count = sum(transport_errors.values())
    completed = len(samples)
    error_rate = 0.0
    if completed > 0:
        error_rate = (http_errors + transport_count) / completed

    latency_ms = {
        "min": min(latencies) if latencies else 0.0,
        "mean": statistics.fmean(latencies) if latencies else 0.0,
        "p50": percentile(latencies, 0.50),
        "p95": percentile(latencies, 0.95),
        "p99": percentile(latencies, 0.99),
        "max": max(latencies) if latencies else 0.0,
    }

    rps = 0.0
    if duration_ms > 0:
        rps = completed / (duration_ms / 1000.0)

    top_transport_errors = [
        {"error": msg, "count": count}
        for msg, count in transport_errors.most_common(8)
    ]

    return {
        "total_requests": completed,
        "completed_requests": completed,
        "success_2xx": success_2xx,
        "http_errors": http_errors,
        "transport_errors": transport_count,
        "error_rate": error_rate,
        "duration_ms": duration_ms,
        "requests_per_second": rps,
        "latency_ms": latency_ms,
        "status_counts": dict(statuses),
        "top_transport_errors": top_transport_errors,
    }


def one_request(full_url: str, timeout_secs: float) -> Sample:
    req = urllib.request.Request(full_url, method="GET")
    start = time.perf_counter()
    try:
        with urllib.request.urlopen(req, timeout=timeout_secs) as res:
            _ = res.read()
            latency_ms = (time.perf_counter() - start) * 1000.0
            return Sample(latency_ms=latency_ms, status=getattr(res, "status", 200), error=None)
    except urllib.error.HTTPError as err:
        latency_ms = (time.perf_counter() - start) * 1000.0
        return Sample(latency_ms=latency_ms, status=err.code, error=None)
    except Exception as err:  # noqa: BLE001
        latency_ms = (time.perf_counter() - start) * 1000.0
        return Sample(latency_ms=latency_ms, status=None, error=str(err))


def run_case(full_url: str, requests: int, concurrency: int, timeout_secs: float) -> Tuple[List[Sample], float]:
    start = time.perf_counter()
    samples: List[Sample] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as pool:
        futures = [pool.submit(one_request, full_url, timeout_secs) for _ in range(requests)]
        for fut in concurrent.futures.as_completed(futures):
            samples.append(fut.result())
    duration_ms = (time.perf_counter() - start) * 1000.0
    return samples, duration_ms


def main() -> None:
    parser = argparse.ArgumentParser(description="Frontend render/load benchmark (local-only)")
    parser.add_argument("--base-url", default=DEFAULT_BASE_URL, help=f"Frontend URL (default: {DEFAULT_BASE_URL})")
    parser.add_argument("--routes", default="/,/admin,/codelabs", help="Comma-separated routes")
    parser.add_argument("--concurrency", default="10,30,60", help="Comma-separated concurrency levels")
    parser.add_argument("--requests", type=int, default=300, help="Requests per case")
    parser.add_argument("--repeats", type=int, default=3, help="Repeats per case")
    parser.add_argument("--timeout-secs", type=float, default=10.0, help="Per-request timeout")
    parser.add_argument("--warmup-requests", type=int, default=20, help="Warmup requests per route")
    parser.add_argument("--output", default=f"bench-results/frontend-bench-{datetime.now().strftime('%Y%m%d-%H%M%S')}.json")
    args = parser.parse_args()

    if args.requests <= 0 or args.repeats <= 0 or args.timeout_secs <= 0 or args.warmup_requests < 0:
        raise SystemExit("Invalid numeric arguments. requests/repeats/timeout must be positive.")

    base_url = args.base_url.rstrip("/")
    routes = parse_csv_routes(args.routes)
    conc_levels = parse_csv_ints(args.concurrency)

    cases: List[Dict[str, Any]] = []
    for route in routes:
        full_url = urllib.parse.urljoin(base_url + "/", route.lstrip("/"))
        if args.warmup_requests > 0:
            _warmup_samples, _ = run_case(
                full_url=full_url,
                requests=args.warmup_requests,
                concurrency=min(8, max(1, args.warmup_requests)),
                timeout_secs=args.timeout_secs,
            )
        for conc in conc_levels:
            for repeat in range(1, args.repeats + 1):
                print(f"[frontend] route={route} conc={conc} repeat={repeat}/{args.repeats}")
                samples, duration_ms = run_case(
                    full_url=full_url,
                    requests=args.requests,
                    concurrency=conc,
                    timeout_secs=args.timeout_secs,
                )
                summary = summarize(samples, duration_ms)
                summary.update(
                    {
                        "route": route,
                        "full_url": full_url,
                        "concurrency": conc,
                        "repeat": repeat,
                    }
                )
                cases.append(summary)

    report = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "base_url": base_url,
            "routes": routes,
            "concurrency": conc_levels,
            "requests_per_case": args.requests,
            "repeats": args.repeats,
            "timeout_secs": args.timeout_secs,
            "warmup_requests": args.warmup_requests,
            "output": args.output,
        },
        "cases": cases,
        "notes": [
            "Measures route response latency under concurrent load (SSR/route handling path).",
            "For browser paint metrics (LCP/INP/CLS), run Lighthouse/Playwright separately and merge artifacts.",
        ],
    }

    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"Frontend benchmark complete: {out_path}")


if __name__ == "__main__":
    main()
