#!/usr/bin/env python3
"""
Long-run soak benchmark orchestrator.

Runs mixed local benchmarks in cycles:
- local_bench (HTTP read/write)
- ws_bench (real-time path)

Collects:
- cycle success/failure
- cycle durations
- optional backend process CPU/RSS samples
- optional restart-recovery timing (if restart command is provided)
"""

from __future__ import annotations

import argparse
import json
import os
import shlex
import statistics
import subprocess
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


def run_cmd(
    cmd: List[str],
    cwd: Path,
    timeout_secs: int,
    stdout_path: Path,
    stderr_path: Path,
) -> Dict[str, Any]:
    start = time.perf_counter()
    result = subprocess.run(
        cmd,
        cwd=str(cwd),
        text=True,
        capture_output=True,
        timeout=timeout_secs,
        check=False,
    )
    duration_ms = (time.perf_counter() - start) * 1000.0
    stdout_path.write_text(result.stdout, encoding="utf-8", errors="replace")
    stderr_path.write_text(result.stderr, encoding="utf-8", errors="replace")
    return {
        "cmd": cmd,
        "exit_code": result.returncode,
        "duration_ms": duration_ms,
        "stdout_log": str(stdout_path),
        "stderr_log": str(stderr_path),
    }


def sample_backend_resources(pid: Optional[int]) -> Optional[Dict[str, Any]]:
    if pid is None:
        return None
    try:
        result = subprocess.run(
            ["ps", "-p", str(pid), "-o", "%cpu=", "-o", "rss="],
            text=True,
            capture_output=True,
            check=False,
        )
        if result.returncode != 0:
            return {"pid": pid, "alive": False}
        line = result.stdout.strip()
        if not line:
            return {"pid": pid, "alive": False}
        parts = line.split()
        if len(parts) < 2:
            return {"pid": pid, "alive": True, "raw": line}
        cpu = float(parts[0])
        rss_kb = int(parts[1])
        return {
            "pid": pid,
            "alive": True,
            "cpu_percent": cpu,
            "rss_kb": rss_kb,
            "rss_mb": rss_kb / 1024.0,
        }
    except Exception as err:  # noqa: BLE001
        return {"pid": pid, "alive": False, "error": str(err)}


def wait_until_ready(url: str, timeout_secs: int, interval_secs: float = 0.5) -> Dict[str, Any]:
    start = time.perf_counter()
    deadline = start + timeout_secs
    attempts = 0
    while time.perf_counter() < deadline:
        attempts += 1
        req = urllib.request.Request(url, method="GET")
        try:
            with urllib.request.urlopen(req, timeout=3) as res:
                if 200 <= getattr(res, "status", 200) < 500:
                    elapsed_ms = (time.perf_counter() - start) * 1000.0
                    return {"ready": True, "recovery_ms": elapsed_ms, "attempts": attempts}
        except urllib.error.HTTPError as err:
            # Even if auth fails (401/403), server is up.
            if 400 <= err.code < 500:
                elapsed_ms = (time.perf_counter() - start) * 1000.0
                return {"ready": True, "recovery_ms": elapsed_ms, "attempts": attempts, "http_code": err.code}
        except Exception:
            pass
        time.sleep(interval_secs)
    elapsed_ms = (time.perf_counter() - start) * 1000.0
    return {"ready": False, "recovery_ms": elapsed_ms, "attempts": attempts}


def p(values: List[float], q: float) -> float:
    if not values:
        return 0.0
    values = sorted(values)
    if len(values) == 1:
        return values[0]
    idx = (len(values) - 1) * q
    lo = int(idx)
    hi = min(lo + 1, len(values) - 1)
    frac = idx - lo
    return values[lo] * (1.0 - frac) + values[hi] * frac


def main() -> None:
    parser = argparse.ArgumentParser(description="Long-run mixed soak benchmark")
    parser.add_argument("--base-url", default=os.environ.get("BENCH_BASE_URL", "http://localhost:8080"))
    parser.add_argument("--admin-id", default=os.environ.get("ADMIN_ID", "admin"))
    parser.add_argument("--admin-pw", default=os.environ.get("ADMIN_PW", "admin"))
    parser.add_argument("--duration-secs", type=int, default=7200, help="Total soak duration")
    parser.add_argument("--cycle-interval-secs", type=int, default=300, help="Target cycle interval")
    parser.add_argument("--backend-pid", type=int, default=None, help="Backend process pid for resource sampling")
    parser.add_argument("--local-attendees", type=int, default=30)
    parser.add_argument("--local-read-requests", type=int, default=300)
    parser.add_argument("--local-write-requests", type=int, default=150)
    parser.add_argument("--local-read-concurrency", type=int, default=30)
    parser.add_argument("--local-write-concurrency", type=int, default=20)
    parser.add_argument("--ws-users", default="30", help="CSV users for ws_bench (ex: 30 or 50,100)")
    parser.add_argument("--ws-duration-secs", type=int, default=45)
    parser.add_argument("--ws-chat-rate", type=float, default=0.5)
    parser.add_argument("--ws-step-interval-secs", type=int, default=7)
    parser.add_argument("--command-timeout-secs", type=int, default=3600)
    parser.add_argument("--restart-cmd", default="", help="Optional shell command to restart backend once mid-run")
    parser.add_argument("--restart-at-secs", type=int, default=-1, help="When to execute restart command (default: midpoint)")
    parser.add_argument("--ready-check-url", default="", help="URL to poll after restart (default: <base>/api/login)")
    parser.add_argument("--ready-timeout-secs", type=int, default=180)
    parser.add_argument(
        "--output",
        default=f"bench-results/soak-bench-{datetime.now().strftime('%Y%m%d-%H%M%S')}.json",
    )
    args = parser.parse_args()

    if args.duration_secs <= 0 or args.cycle_interval_secs <= 0:
        raise SystemExit("duration-secs and cycle-interval-secs must be >= 1")

    backend_dir = Path(__file__).resolve().parents[1]
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    logs_dir = output_path.parent / "soak-logs"
    logs_dir.mkdir(parents=True, exist_ok=True)

    start_wall = datetime.now(timezone.utc)
    start_monotonic = time.perf_counter()
    deadline = start_monotonic + args.duration_secs

    restart_trigger = args.restart_at_secs
    if restart_trigger < 0:
        restart_trigger = args.duration_secs // 2
    ready_check_url = args.ready_check_url.strip() or f"{args.base_url.rstrip('/')}/api/login"
    restart_event: Optional[Dict[str, Any]] = None
    restart_done = False

    cycles: List[Dict[str, Any]] = []
    cycle_index = 0
    while time.perf_counter() < deadline:
        cycle_index += 1
        elapsed_secs = int(time.perf_counter() - start_monotonic)

        if args.restart_cmd.strip() and not restart_done and elapsed_secs >= restart_trigger:
            restart_start = time.perf_counter()
            restart_proc = subprocess.run(
                args.restart_cmd,
                shell=True,
                text=True,
                capture_output=True,
                check=False,
            )
            restart_exec_ms = (time.perf_counter() - restart_start) * 1000.0
            readiness = wait_until_ready(
                url=ready_check_url,
                timeout_secs=args.ready_timeout_secs,
            )
            restart_event = {
                "at_elapsed_secs": elapsed_secs,
                "cmd": args.restart_cmd,
                "cmd_exit_code": restart_proc.returncode,
                "cmd_stdout": restart_proc.stdout[-4000:],
                "cmd_stderr": restart_proc.stderr[-4000:],
                "restart_command_ms": restart_exec_ms,
                "ready_check_url": ready_check_url,
                "ready_result": readiness,
            }
            restart_done = True

        cycle_start = time.perf_counter()
        cycle_id = f"cycle-{cycle_index:04d}"

        local_output = output_path.parent / f"{cycle_id}-local.json"
        local_stdout = logs_dir / f"{cycle_id}-local.stdout.log"
        local_stderr = logs_dir / f"{cycle_id}-local.stderr.log"
        ws_output = output_path.parent / f"{cycle_id}-ws.json"
        ws_stdout = logs_dir / f"{cycle_id}-ws.stdout.log"
        ws_stderr = logs_dir / f"{cycle_id}-ws.stderr.log"

        local_cmd = [
            "cargo",
            "run",
            "--release",
            "--bin",
            "local_bench",
            "--",
            "--base-url",
            args.base_url,
            "--admin-id",
            args.admin_id,
            "--admin-pw",
            args.admin_pw,
            "--attendees",
            str(args.local_attendees),
            "--read-requests",
            str(args.local_read_requests),
            "--write-requests",
            str(args.local_write_requests),
            "--read-concurrency",
            str(args.local_read_concurrency),
            "--write-concurrency",
            str(args.local_write_concurrency),
            "--output",
            str(local_output),
        ]
        ws_cmd = [
            "cargo",
            "run",
            "--release",
            "--bin",
            "ws_bench",
            "--",
            "--base-url",
            args.base_url,
            "--admin-id",
            args.admin_id,
            "--admin-pw",
            args.admin_pw,
            "--users",
            args.ws_users,
            "--duration-secs",
            str(args.ws_duration_secs),
            "--chat-rate",
            str(args.ws_chat_rate),
            "--step-interval-secs",
            str(args.ws_step_interval_secs),
            "--output",
            str(ws_output),
        ]

        print(f"[soak] {cycle_id} local_bench...")
        local_run = run_cmd(
            cmd=local_cmd,
            cwd=backend_dir,
            timeout_secs=args.command_timeout_secs,
            stdout_path=local_stdout,
            stderr_path=local_stderr,
        )

        print(f"[soak] {cycle_id} ws_bench...")
        ws_run = run_cmd(
            cmd=ws_cmd,
            cwd=backend_dir,
            timeout_secs=args.command_timeout_secs,
            stdout_path=ws_stdout,
            stderr_path=ws_stderr,
        )

        resources = sample_backend_resources(args.backend_pid)
        cycle_duration_ms = (time.perf_counter() - cycle_start) * 1000.0
        cycle_ok = local_run["exit_code"] == 0 and ws_run["exit_code"] == 0

        cycles.append(
            {
                "cycle": cycle_index,
                "elapsed_secs": elapsed_secs,
                "ok": cycle_ok,
                "duration_ms": cycle_duration_ms,
                "local_bench": local_run,
                "ws_bench": ws_run,
                "resources": resources,
                "outputs": {"local_json": str(local_output), "ws_json": str(ws_output)},
            }
        )

        target_next = cycle_start + args.cycle_interval_secs
        remaining = target_next - time.perf_counter()
        if remaining > 0:
            time.sleep(remaining)

    end_wall = datetime.now(timezone.utc)
    cycle_durations = [c["duration_ms"] for c in cycles]
    ok_count = sum(1 for c in cycles if c["ok"])
    fail_count = len(cycles) - ok_count

    cpu_samples: List[float] = []
    rss_samples: List[float] = []
    for c in cycles:
        r = c.get("resources")
        if isinstance(r, dict) and r.get("alive"):
            if "cpu_percent" in r:
                cpu_samples.append(float(r["cpu_percent"]))
            if "rss_mb" in r:
                rss_samples.append(float(r["rss_mb"]))

    summary: Dict[str, Any] = {
        "cycles": len(cycles),
        "ok_cycles": ok_count,
        "failed_cycles": fail_count,
        "cycle_success_rate": (ok_count / len(cycles)) if cycles else 0.0,
        "cycle_duration_ms": {
            "mean": statistics.fmean(cycle_durations) if cycle_durations else 0.0,
            "p50": p(cycle_durations, 0.50),
            "p95": p(cycle_durations, 0.95),
            "p99": p(cycle_durations, 0.99),
            "max": max(cycle_durations) if cycle_durations else 0.0,
        },
        "backend_cpu_percent": {
            "samples": len(cpu_samples),
            "mean": statistics.fmean(cpu_samples) if cpu_samples else 0.0,
            "p95": p(cpu_samples, 0.95),
            "max": max(cpu_samples) if cpu_samples else 0.0,
        },
        "backend_rss_mb": {
            "samples": len(rss_samples),
            "mean": statistics.fmean(rss_samples) if rss_samples else 0.0,
            "p95": p(rss_samples, 0.95),
            "max": max(rss_samples) if rss_samples else 0.0,
        },
        "restart_event": restart_event,
    }

    report = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "started_at_utc": start_wall.isoformat(),
        "ended_at_utc": end_wall.isoformat(),
        "config": {
            "base_url": args.base_url,
            "admin_id": args.admin_id,
            "duration_secs": args.duration_secs,
            "cycle_interval_secs": args.cycle_interval_secs,
            "backend_pid": args.backend_pid,
            "local_attendees": args.local_attendees,
            "local_read_requests": args.local_read_requests,
            "local_write_requests": args.local_write_requests,
            "local_read_concurrency": args.local_read_concurrency,
            "local_write_concurrency": args.local_write_concurrency,
            "ws_users": args.ws_users,
            "ws_duration_secs": args.ws_duration_secs,
            "ws_chat_rate": args.ws_chat_rate,
            "ws_step_interval_secs": args.ws_step_interval_secs,
            "restart_cmd": args.restart_cmd,
            "restart_at_secs": restart_trigger,
            "ready_check_url": ready_check_url,
            "ready_timeout_secs": args.ready_timeout_secs,
            "output": str(output_path),
        },
        "summary": summary,
        "cycles": cycles,
    }

    output_path.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"Soak benchmark complete: {output_path}")


if __name__ == "__main__":
    main()
