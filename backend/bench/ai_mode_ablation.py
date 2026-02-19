#!/usr/bin/env python3
"""Small-scale Basic vs Pro-mode ablation for Open Codelabs AI generation.

This script calls Gemini directly and compares:
- Basic mode: single codelab generation call
- Pro mode: plan -> draft -> review -> revise

Then it audits both outputs with the same reviewer prompt/schema to derive issue counts.
"""

from __future__ import annotations

import json
import os
import re
import statistics
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib import error, request

ROOT = Path(__file__).resolve().parents[1]  # backend/
REPO = ROOT.parent

MODEL = os.environ.get("AI_ABLATION_MODEL", "gemini-3-flash-preview")
API_KEY = os.environ.get("GEMINI_API_KEY", "")
API_URL = (
    f"https://generativelanguage.googleapis.com/v1beta/models/{MODEL}:generateContent?key={API_KEY}"
)

INPUT_PRICE_PER_1M = 0.5
OUTPUT_PRICE_PER_1M = 3.0

TARGET_LANGUAGE = "English"
TARGET_DURATION_MIN = 60

DEFAULT_TASKS = [
    "backend/src/api/handlers/ai.rs",
]

TASKS = [
    item.strip()
    for item in os.environ.get("AI_ABLATION_TASKS", ",".join(DEFAULT_TASKS)).split(",")
    if item.strip()
]

MAX_CONTEXT_CHARS = int(os.environ.get("AI_ABLATION_CONTEXT_CHARS", "1800"))


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def extract_backtick_const(path: Path, const_name: str) -> str:
    text = read_text(path)
    pat = re.compile(rf"const\s+{re.escape(const_name)}\s*=\s*`([\s\S]*?)`;", re.MULTILINE)
    m = pat.search(text)
    if not m:
        raise RuntimeError(f"Could not find {const_name} in {path}")
    return m.group(1).strip()


def find_json_block(s: str) -> str:
    s = s.strip()
    try:
        json.loads(s)
        return s
    except Exception:
        pass

    first = s.find("{")
    last = s.rfind("}")
    if first != -1 and last != -1 and last > first:
        candidate = s[first : last + 1]
        json.loads(candidate)
        return candidate
    raise ValueError("No valid JSON object in response")


@dataclass
class Usage:
    prompt: int = 0
    candidate: int = 0
    total: int = 0

    def add(self, other: "Usage") -> None:
        self.prompt += other.prompt
        self.candidate += other.candidate
        self.total += other.total

    @property
    def cost_usd(self) -> float:
        return (self.prompt / 1_000_000) * INPUT_PRICE_PER_1M + (
            self.candidate / 1_000_000
        ) * OUTPUT_PRICE_PER_1M


def call_structured(prompt: str, system_prompt: str, schema: dict[str, Any]) -> tuple[dict[str, Any], Usage]:
    payload = {
        "system_instruction": {"parts": [{"text": system_prompt}]},
        "contents": [{"role": "user", "parts": [{"text": prompt}]}],
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseSchema": schema,
            "temperature": 0.2,
        },
    }
    data = json.dumps(payload).encode("utf-8")
    req = request.Request(
        API_URL,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    try:
        with request.urlopen(req, timeout=120) as resp:
            raw = resp.read().decode("utf-8")
    except error.HTTPError as e:
        body = e.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"HTTP {e.code}: {body[:1000]}") from e

    obj = json.loads(raw)
    try:
        text = obj["candidates"][0]["content"]["parts"][0]["text"]
    except Exception as e:
        raise RuntimeError(f"Malformed response payload: {raw[:1200]}") from e

    parsed = json.loads(find_json_block(text))
    um = obj.get("usageMetadata", {})
    usage = Usage(
        prompt=int(um.get("promptTokenCount", 0) or 0),
        candidate=int(um.get("candidatesTokenCount", 0) or 0),
        total=int(um.get("totalTokenCount", 0) or 0),
    )
    return parsed, usage


def make_codelab_schema() -> dict[str, Any]:
    return {
        "type": "OBJECT",
        "properties": {
            "title": {"type": "STRING"},
            "description": {"type": "STRING"},
            "steps": {
                "type": "ARRAY",
                "items": {
                    "type": "OBJECT",
                    "properties": {
                        "title": {"type": "STRING"},
                        "content": {"type": "STRING"},
                    },
                    "required": ["title", "content"],
                },
            },
        },
        "required": ["title", "description", "steps"],
    }


def make_plan_schema() -> dict[str, Any]:
    return {
        "type": "OBJECT",
        "properties": {
            "title": {"type": "STRING"},
            "description": {"type": "STRING"},
            "audience": {"type": "STRING"},
            "learning_objectives": {"type": "ARRAY", "items": {"type": "STRING"}},
            "prerequisites": {"type": "ARRAY", "items": {"type": "STRING"}},
            "environment_setup": {"type": "ARRAY", "items": {"type": "STRING"}},
            "steps": {
                "type": "ARRAY",
                "items": {
                    "type": "OBJECT",
                    "properties": {
                        "title": {"type": "STRING"},
                        "goal": {"type": "STRING"},
                        "files": {"type": "ARRAY", "items": {"type": "STRING"}},
                        "verification": {"type": "STRING"},
                    },
                    "required": ["title", "goal", "files", "verification"],
                },
            },
            "search_terms": {"type": "ARRAY", "items": {"type": "STRING"}},
        },
        "required": [
            "title",
            "description",
            "audience",
            "learning_objectives",
            "prerequisites",
            "environment_setup",
            "steps",
            "search_terms",
        ],
    }


def make_review_schema() -> dict[str, Any]:
    return {
        "type": "OBJECT",
        "properties": {
            "summary": {"type": "STRING"},
            "issues": {
                "type": "ARRAY",
                "items": {
                    "type": "OBJECT",
                    "properties": {
                        "severity": {"type": "STRING"},
                        "issue": {"type": "STRING"},
                        "recommendation": {"type": "STRING"},
                    },
                    "required": ["severity", "issue", "recommendation"],
                },
            },
            "missing_items": {"type": "ARRAY", "items": {"type": "STRING"}},
            "improvements": {"type": "ARRAY", "items": {"type": "STRING"}},
        },
        "required": ["summary", "issues", "missing_items", "improvements"],
    }


def normalize_codelab_text(codelab: dict[str, Any]) -> str:
    parts = [str(codelab.get("title", "")), str(codelab.get("description", ""))]
    for step in codelab.get("steps", []) or []:
        parts.append(str(step.get("title", "")))
        parts.append(str(step.get("content", "")))
    return "\n".join(parts).lower()


def section_hits(codelab: dict[str, Any]) -> dict[str, int]:
    text = normalize_codelab_text(codelab)
    checks = {
        "has_prereq": int("prerequisite" in text or "prerequisites" in text),
        "has_environment_setup": int(
            "environment setup" in text or "setup" in text or "path" in text
        ),
        "has_verification": int("verify" in text or "verification" in text),
        "has_summary": int("summary" in text or "takeaway" in text),
    }
    checks["total"] = sum(checks.values())
    return checks


def issue_metrics(review: dict[str, Any]) -> dict[str, Any]:
    issues = review.get("issues", []) or []
    missing = review.get("missing_items", []) or review.get("missing", []) or []
    improvements = review.get("improvements", []) or []

    severe = 0
    for it in issues:
        sev = str(it.get("severity", "")).lower()
        if "high" in sev or "critical" in sev or "major" in sev:
            severe += 1

    return {
        "issues": len(issues),
        "severe_issues": severe,
        "missing_items": len(missing),
        "improvements": len(improvements),
    }


def mean(values: list[float]) -> float:
    return float(statistics.mean(values)) if values else 0.0


def run() -> dict[str, Any]:
    if not API_KEY:
        raise RuntimeError("GEMINI_API_KEY is not set")

    ai_gen_file = REPO / "frontend/src/lib/components/admin/AiCodelabGenerator.svelte"
    system_prompt = extract_backtick_const(ai_gen_file, "SYSTEM_PROMPT")
    plan_system_prompt = extract_backtick_const(ai_gen_file, "PLAN_SYSTEM_PROMPT")
    review_system_prompt = extract_backtick_const(ai_gen_file, "REVIEW_SYSTEM_PROMPT")

    codelab_schema = make_codelab_schema()
    plan_schema = make_plan_schema()
    review_schema = make_review_schema()

    task_results: list[dict[str, Any]] = []
    aggregate = {
        "basic_usage": Usage(),
        "pro_usage": Usage(),
        "audit_usage": Usage(),
    }

    for rel in TASKS:
        print(f"[task] {rel}", flush=True)
        src_path = REPO / rel
        context = read_text(src_path)
        context = context[:MAX_CONTEXT_CHARS]

        duration_text = (
            f"The target duration for this hands-on session is approximately {TARGET_DURATION_MIN} minutes. "
            f"Please adjust the depth and number of steps to fit this timeframe."
        )

        plan_prompt = (
            f"Design a codelab plan from the following source code and context. {duration_text} "
            f"Write all content in {TARGET_LANGUAGE}. For \"search_terms\", use short English queries "
            f"to find the latest versions, commands, or best practices (3-8 items). "
            f"Keep step count aligned with the target duration. If something is unknown, return empty arrays.\n\n"
            f"Source code/Context:\n{context}"
        )
        print("  - plan", flush=True)
        plan_data, u_plan = call_structured(plan_prompt, plan_system_prompt, plan_schema)

        basic_prompt = (
            f"Create a codelab tutorial from the following source code and context. {duration_text} "
            f"Write ALL content in {TARGET_LANGUAGE}. "
            f"For every code block, include inline comments on each logical line, specify the filename before the block, "
            f"and append a numbered line-by-line explanation list immediately after the block (same language).\n\n"
            f"Source code/Context:\n{context}"
        )
        print("  - basic", flush=True)
        basic_data, u_basic = call_structured(basic_prompt, system_prompt, codelab_schema)

        search_terms = plan_data.get("search_terms", []) or []
        if search_terms:
            search_hint = (
                "Use the Google Search tool to verify the latest information for these queries: "
                + ", ".join(search_terms)
                + "."
            )
        else:
            search_hint = "Use the Google Search tool if any versions, commands, or APIs need verification."

        draft_prompt = (
            f"Create a codelab using the plan and source context. {duration_text} "
            f"Write ALL content in {TARGET_LANGUAGE}. {search_hint}\n\n"
            f"Plan JSON:\n{json.dumps(plan_data, ensure_ascii=False, indent=2)}\n\n"
            f"Source code/Context:\n{context}"
        )
        print("  - pro_draft", flush=True)
        pro_draft, u_draft = call_structured(draft_prompt, system_prompt, codelab_schema)

        review_prompt = (
            f"Review the draft codelab as a third-party facilitator expert. "
            f"Use the plan to verify structure and completeness. Write ALL content in {TARGET_LANGUAGE}.\n\n"
            f"Plan JSON:\n{json.dumps(plan_data, ensure_ascii=False, indent=2)}\n\n"
            f"Draft JSON:\n{json.dumps(pro_draft, ensure_ascii=False, indent=2)}\n\n"
            f"Source code/Context:\n{context}"
        )
        print("  - pro_review", flush=True)
        pro_review, u_review = call_structured(review_prompt, review_system_prompt, review_schema)

        revise_prompt = (
            f"Revise the draft codelab based on the expert review. {duration_text} "
            f"Write ALL content in {TARGET_LANGUAGE}. {search_hint}\n\n"
            f"Plan JSON:\n{json.dumps(plan_data, ensure_ascii=False, indent=2)}\n\n"
            f"Draft JSON:\n{json.dumps(pro_draft, ensure_ascii=False, indent=2)}\n\n"
            f"Review JSON:\n{json.dumps(pro_review, ensure_ascii=False, indent=2)}\n\n"
            f"Source code/Context:\n{context}"
        )
        print("  - pro_revise", flush=True)
        pro_revised, u_revise = call_structured(revise_prompt, system_prompt, codelab_schema)

        audit_basic_prompt = (
            f"Review the draft codelab as a third-party facilitator expert. "
            f"Use the plan to verify structure and completeness. Write ALL content in {TARGET_LANGUAGE}.\n\n"
            f"Plan JSON:\n{json.dumps(plan_data, ensure_ascii=False, indent=2)}\n\n"
            f"Draft JSON:\n{json.dumps(basic_data, ensure_ascii=False, indent=2)}\n\n"
            f"Source code/Context:\n{context}"
        )
        print("  - audit_basic", flush=True)
        audit_basic, u_audit_basic = call_structured(
            audit_basic_prompt, review_system_prompt, review_schema
        )

        audit_pro_prompt = (
            f"Review the draft codelab as a third-party facilitator expert. "
            f"Use the plan to verify structure and completeness. Write ALL content in {TARGET_LANGUAGE}.\n\n"
            f"Plan JSON:\n{json.dumps(plan_data, ensure_ascii=False, indent=2)}\n\n"
            f"Draft JSON:\n{json.dumps(pro_revised, ensure_ascii=False, indent=2)}\n\n"
            f"Source code/Context:\n{context}"
        )
        print("  - audit_pro", flush=True)
        audit_pro, u_audit_pro = call_structured(audit_pro_prompt, review_system_prompt, review_schema)

        basic_issues = issue_metrics(audit_basic)
        pro_issues = issue_metrics(audit_pro)
        basic_hits = section_hits(basic_data)
        pro_hits = section_hits(pro_revised)

        pro_usage = Usage()
        for u in (u_plan, u_draft, u_review, u_revise):
            pro_usage.add(u)

        aggregate["basic_usage"].add(u_basic)
        aggregate["pro_usage"].add(pro_usage)
        aggregate["audit_usage"].add(u_audit_basic)
        aggregate["audit_usage"].add(u_audit_pro)

        task_results.append(
            {
                "task": rel,
                "context_chars": len(context),
                "basic": {
                    "usage": u_basic.__dict__,
                    "cost_usd": u_basic.cost_usd,
                    "steps": len(basic_data.get("steps", []) or []),
                    "issue_metrics": basic_issues,
                    "section_hits": basic_hits,
                },
                "pro": {
                    "usage_pipeline": pro_usage.__dict__,
                    "cost_usd_pipeline": pro_usage.cost_usd,
                    "steps": len(pro_revised.get("steps", []) or []),
                    "issue_metrics": pro_issues,
                    "section_hits": pro_hits,
                },
            }
        )

        time.sleep(1.0)

    basic_issue_counts = [float(t["basic"]["issue_metrics"]["issues"]) for t in task_results]
    pro_issue_counts = [float(t["pro"]["issue_metrics"]["issues"]) for t in task_results]
    basic_missing_counts = [float(t["basic"]["issue_metrics"]["missing_items"]) for t in task_results]
    pro_missing_counts = [float(t["pro"]["issue_metrics"]["missing_items"]) for t in task_results]

    basic_hits_totals = [float(t["basic"]["section_hits"]["total"]) for t in task_results]
    pro_hits_totals = [float(t["pro"]["section_hits"]["total"]) for t in task_results]

    def pct_improve(old: float, new: float) -> float:
        if old == 0:
            return 0.0
        return (old - new) / old * 100.0

    summary = {
        "n_tasks": len(task_results),
        "mean_issues_basic": mean(basic_issue_counts),
        "mean_issues_pro": mean(pro_issue_counts),
        "issues_reduction_percent": pct_improve(mean(basic_issue_counts), mean(pro_issue_counts)),
        "mean_missing_basic": mean(basic_missing_counts),
        "mean_missing_pro": mean(pro_missing_counts),
        "missing_reduction_percent": pct_improve(mean(basic_missing_counts), mean(pro_missing_counts)),
        "mean_section_hits_basic": mean(basic_hits_totals),
        "mean_section_hits_pro": mean(pro_hits_totals),
        "basic_generation_usage": aggregate["basic_usage"].__dict__,
        "pro_generation_usage": aggregate["pro_usage"].__dict__,
        "basic_generation_cost_usd": aggregate["basic_usage"].cost_usd,
        "pro_generation_cost_usd": aggregate["pro_usage"].cost_usd,
        "audit_usage": aggregate["audit_usage"].__dict__,
    }

    return {
        "model": MODEL,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "target_language": TARGET_LANGUAGE,
        "target_duration_min": TARGET_DURATION_MIN,
        "tasks": task_results,
        "summary": summary,
        "notes": [
            "This is a small-sample internal ablation; reviewer and generator model family is the same.",
            "Use independent human raters and held-out workshop traces for publication-grade generalization claims.",
        ],
    }


if __name__ == "__main__":
    out = run()
    out_path = ROOT / "bench-results/reviewer-20260219/ai-ablation.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(out, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"Wrote: {out_path}")
    print(json.dumps(out["summary"], indent=2))
