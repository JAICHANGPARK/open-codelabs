#!/usr/bin/env python3
"""
SWE quality metrics collector for local benchmark artifacts.

Collects (best effort):
- test results
- coverage (if cargo-llvm-cov is available)
- mutation score (if explicitly enabled and tool is available)
- static analysis warnings
- security scan vulnerabilities

All command logs are saved for reproducibility.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


@dataclass
class CmdResult:
    cmd: List[str]
    cwd: str
    exit_code: int
    duration_ms: float
    stdout_log: str
    stderr_log: str


def run_cmd(
    cmd: List[str],
    cwd: Path,
    timeout_secs: int,
    logs_dir: Path,
    label: str,
) -> CmdResult:
    logs_dir.mkdir(parents=True, exist_ok=True)
    stdout_log = logs_dir / f"{label}.stdout.log"
    stderr_log = logs_dir / f"{label}.stderr.log"
    start = time.perf_counter()
    proc = subprocess.run(
        cmd,
        cwd=str(cwd),
        text=True,
        capture_output=True,
        timeout=timeout_secs,
        check=False,
    )
    duration_ms = (time.perf_counter() - start) * 1000.0
    stdout_log.write_text(proc.stdout, encoding="utf-8", errors="replace")
    stderr_log.write_text(proc.stderr, encoding="utf-8", errors="replace")
    return CmdResult(
        cmd=cmd,
        cwd=str(cwd),
        exit_code=proc.returncode,
        duration_ms=duration_ms,
        stdout_log=str(stdout_log),
        stderr_log=str(stderr_log),
    )


def cmd_exists(name: str) -> bool:
    return shutil.which(name) is not None


def read_text(path: str) -> str:
    try:
        return Path(path).read_text(encoding="utf-8", errors="replace")
    except Exception:
        return ""


def parse_clippy_warnings(stdout_text: str) -> Dict[str, Any]:
    warning_count = 0
    error_count = 0
    lint_counts: Dict[str, int] = {}
    malformed_lines = 0
    for line in stdout_text.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            malformed_lines += 1
            continue
        if obj.get("reason") != "compiler-message":
            continue
        msg = obj.get("message") or {}
        level = msg.get("level")
        if level == "warning":
            warning_count += 1
        elif level == "error":
            error_count += 1
        code = msg.get("code") or {}
        code_name = code.get("code")
        if code_name:
            lint_counts[code_name] = lint_counts.get(code_name, 0) + 1
    top_lints = sorted(lint_counts.items(), key=lambda x: x[1], reverse=True)[:20]
    return {
        "warnings": warning_count,
        "errors": error_count,
        "top_lints": [{"lint": k, "count": v} for k, v in top_lints],
        "malformed_json_lines": malformed_lines,
    }


def parse_llvm_cov_summary(text: str) -> Dict[str, Any]:
    # Typical summary includes a TOTAL row with three percentage values.
    # Example:
    # TOTAL ... 83.12% ... 79.45% ... 81.90%
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    total_line = ""
    for line in reversed(lines):
        if line.startswith("TOTAL"):
            total_line = line
            break
    if not total_line:
        return {"available": False, "reason": "TOTAL line not found in llvm-cov summary"}
    percentages = [float(x) for x in re.findall(r"([0-9]+(?:\.[0-9]+)?)%", total_line)]
    if len(percentages) < 3:
        return {"available": False, "reason": "Could not parse coverage percentages from TOTAL line"}
    return {
        "available": True,
        "regions_percent": percentages[0],
        "functions_percent": percentages[1],
        "lines_percent": percentages[2],
        "raw_total_line": total_line,
    }


def parse_cargo_audit_json(text: str) -> Dict[str, Any]:
    try:
        obj = json.loads(text)
    except Exception:
        return {"available": False, "reason": "invalid_json"}
    vulns = obj.get("vulnerabilities") or {}
    items = vulns.get("list") or []
    high = 0
    critical = 0
    for item in items:
        advisory = item.get("advisory") or {}
        cvss = advisory.get("cvss")
        score = None
        if isinstance(cvss, (int, float)):
            score = float(cvss)
        elif isinstance(cvss, str):
            try:
                score = float(cvss)
            except ValueError:
                score = None
        if score is not None:
            if score >= 9.0:
                critical += 1
            elif score >= 7.0:
                high += 1
    return {
        "available": True,
        "found": bool(vulns.get("found")),
        "count": int(vulns.get("count", len(items))),
        "high": high,
        "critical": critical,
    }


def parse_bun_audit_json(text: str) -> Dict[str, Any]:
    # Bun's JSON format can vary by version. Parse defensively.
    try:
        obj = json.loads(text)
    except Exception:
        return {"available": False, "reason": "invalid_json"}

    counts = {"low": 0, "moderate": 0, "high": 0, "critical": 0}

    def walk(node: Any) -> None:
        if isinstance(node, dict):
            sev = node.get("severity")
            if isinstance(sev, str):
                sev_l = sev.lower()
                if sev_l in counts:
                    counts[sev_l] += 1
            for value in node.values():
                walk(value)
        elif isinstance(node, list):
            for item in node:
                walk(item)

    walk(obj)
    total = sum(counts.values())
    return {
        "available": True,
        "count": total,
        "high": counts["high"],
        "critical": counts["critical"],
        "by_severity": counts,
    }


def parse_mutation_score(text: str) -> Optional[float]:
    # Try common patterns.
    patterns = [
        r"mutation score[^0-9]*([0-9]+(?:\.[0-9]+)?)\s*%",
        r"([0-9]+(?:\.[0-9]+)?)\s*%\s*mutants?\s*(?:caught|killed)",
    ]
    lower = text.lower()
    for pattern in patterns:
        match = re.search(pattern, lower, re.IGNORECASE)
        if match:
            try:
                return float(match.group(1))
            except ValueError:
                pass
    return None


def run_version(tool_cmd: List[str], cwd: Path) -> str:
    try:
        proc = subprocess.run(
            tool_cmd,
            cwd=str(cwd),
            text=True,
            capture_output=True,
            timeout=30,
            check=False,
        )
        raw = (proc.stdout or proc.stderr).strip()
        return raw.splitlines()[0] if raw else ""
    except Exception:
        return ""


def main() -> None:
    parser = argparse.ArgumentParser(description="Collect SWE quality metrics (local-only)")
    parser.add_argument("--backend-dir", default="", help="Backend directory path")
    parser.add_argument("--frontend-dir", default="", help="Frontend directory path")
    parser.add_argument("--timeout-secs", type=int, default=3600)
    parser.add_argument("--run-mutation", action="store_true", help="Attempt mutation testing if tool exists")
    parser.add_argument("--mutation-score", type=float, default=None, help="Manual mutation score override")
    parser.add_argument(
        "--output",
        default=f"bench-results/swe-metrics-{datetime.now().strftime('%Y%m%d-%H%M%S')}.json",
    )
    args = parser.parse_args()

    this_file = Path(__file__).resolve()
    inferred_backend_dir = this_file.parents[1]
    inferred_repo_root = inferred_backend_dir.parent
    backend_dir = Path(args.backend_dir).resolve() if args.backend_dir else inferred_backend_dir
    frontend_dir = Path(args.frontend_dir).resolve() if args.frontend_dir else (inferred_repo_root / "frontend")

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    logs_dir = output_path.parent / "swe-logs"
    logs_dir.mkdir(parents=True, exist_ok=True)

    commands: List[Dict[str, Any]] = []

    def run(label: str, cmd: List[str], cwd: Path) -> CmdResult:
        print(f"[swe] running: {' '.join(cmd)}")
        result = run_cmd(
            cmd=cmd,
            cwd=cwd,
            timeout_secs=args.timeout_secs,
            logs_dir=logs_dir,
            label=label,
        )
        commands.append(
            {
                "label": label,
                "cmd": result.cmd,
                "cwd": result.cwd,
                "exit_code": result.exit_code,
                "duration_ms": result.duration_ms,
                "stdout_log": result.stdout_log,
                "stderr_log": result.stderr_log,
            }
        )
        return result

    # Tests
    backend_test = run("backend_tests", ["cargo", "test"], backend_dir)
    frontend_test = None
    if frontend_dir.exists() and cmd_exists("bun"):
        frontend_test = run("frontend_tests", ["bun", "test"], frontend_dir)

    # Coverage
    coverage: Dict[str, Any]
    if cmd_exists("cargo-llvm-cov"):
        cov = run(
            "backend_coverage",
            ["cargo", "llvm-cov", "--workspace", "--all-features", "--summary-only"],
            backend_dir,
        )
        cov_stdout = read_text(cov.stdout_log)
        parsed = parse_llvm_cov_summary(cov_stdout)
        coverage = {
            "tool": "cargo-llvm-cov",
            "exit_code": cov.exit_code,
            **parsed,
        }
    else:
        coverage = {"available": False, "reason": "cargo-llvm-cov not installed"}

    # Mutation score
    mutation: Dict[str, Any]
    if args.mutation_score is not None:
        mutation = {
            "available": True,
            "source": "manual_argument",
            "score_percent": args.mutation_score,
        }
    elif args.run_mutation:
        mut = run("backend_mutation", ["cargo", "mutants"], backend_dir)
        combined = read_text(mut.stdout_log) + "\n" + read_text(mut.stderr_log)
        score = parse_mutation_score(combined)
        mutation = {
            "attempted": True,
            "exit_code": mut.exit_code,
            "score_percent": score,
            "available": score is not None,
            "reason": None if score is not None else "could_not_parse_mutation_score",
        }
    else:
        mutation = {
            "available": False,
            "reason": "not_run (use --run-mutation or --mutation-score)",
        }

    # Static analysis
    clippy = run(
        "backend_clippy",
        ["cargo", "clippy", "--all-targets", "--all-features", "--message-format=json"],
        backend_dir,
    )
    clippy_parsed = parse_clippy_warnings(read_text(clippy.stdout_log))
    frontend_check = None
    frontend_check_summary: Dict[str, Any] = {"available": False, "reason": "bun or frontend dir missing"}
    if frontend_dir.exists() and cmd_exists("bun"):
        frontend_check = run("frontend_check", ["bun", "run", "check"], frontend_dir)
        check_text = read_text(frontend_check.stdout_log) + "\n" + read_text(frontend_check.stderr_log)
        error_lines = len(re.findall(r"\berror\b", check_text, flags=re.IGNORECASE))
        warning_lines = len(re.findall(r"\bwarning\b", check_text, flags=re.IGNORECASE))
        frontend_check_summary = {
            "available": True,
            "exit_code": frontend_check.exit_code,
            "error_line_hits": error_lines,
            "warning_line_hits": warning_lines,
        }

    static_analysis = {
        "backend_clippy": {
            "available": True,
            "exit_code": clippy.exit_code,
            **clippy_parsed,
        },
        "frontend_check": frontend_check_summary,
    }

    # Security scans
    backend_security: Dict[str, Any]
    if cmd_exists("cargo-audit"):
        audit = run("backend_security_audit", ["cargo", "audit", "--json"], backend_dir)
        backend_security = {
            "tool": "cargo-audit",
            "exit_code": audit.exit_code,
            **parse_cargo_audit_json(read_text(audit.stdout_log)),
        }
    else:
        backend_security = {"available": False, "reason": "cargo-audit not installed"}

    frontend_security: Dict[str, Any]
    if frontend_dir.exists() and cmd_exists("bun"):
        audit_front = run("frontend_security_audit", ["bun", "audit", "--json"], frontend_dir)
        frontend_security = {
            "tool": "bun audit",
            "exit_code": audit_front.exit_code,
            **parse_bun_audit_json(read_text(audit_front.stdout_log)),
        }
    else:
        frontend_security = {"available": False, "reason": "bun or frontend dir missing"}

    report = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "backend_dir": str(backend_dir),
            "frontend_dir": str(frontend_dir),
            "timeout_secs": args.timeout_secs,
            "run_mutation": args.run_mutation,
            "mutation_score_override": args.mutation_score,
            "output": str(output_path),
        },
        "environment": {
            "platform": platform.platform(),
            "python": sys.version.split()[0],
            "tools": {
                "cargo": run_version(["cargo", "--version"], backend_dir),
                "rustc": run_version(["rustc", "--version"], backend_dir),
                "bun": run_version(["bun", "--version"], frontend_dir if frontend_dir.exists() else backend_dir),
                "cargo-llvm-cov": run_version(["cargo", "llvm-cov", "--version"], backend_dir) if cmd_exists("cargo-llvm-cov") else "",
                "cargo-audit": run_version(["cargo", "audit", "--version"], backend_dir) if cmd_exists("cargo-audit") else "",
            },
        },
        "metrics": {
            "tests": {
                "backend": {"exit_code": backend_test.exit_code, "passed": backend_test.exit_code == 0},
                "frontend": (
                    {"exit_code": frontend_test.exit_code, "passed": frontend_test.exit_code == 0}
                    if frontend_test
                    else {"available": False, "reason": "bun or frontend dir missing"}
                ),
            },
            "coverage": coverage,
            "mutation": mutation,
            "static_analysis": static_analysis,
            "security": {
                "backend": backend_security,
                "frontend": frontend_security,
                "high_critical_total": {
                    "high": int(backend_security.get("high", 0)) + int(frontend_security.get("high", 0)),
                    "critical": int(backend_security.get("critical", 0)) + int(frontend_security.get("critical", 0)),
                },
            },
        },
        "commands": commands,
        "notes": [
            "Some metrics are best-effort and may be unavailable if required tools are not installed locally.",
            "Raw command logs are saved under swe-logs for reproducibility and auditability.",
        ],
    }

    output_path.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"SWE metrics complete: {output_path}")


if __name__ == "__main__":
    main()
