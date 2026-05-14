#!/usr/bin/env python3
"""Validate and summarize a representative showcase musical candidate."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.representative_showcase_musical_quality.v1"
MIN_FULL_RMS = 0.018
MIN_LOW_BAND_RMS = 0.008
MIN_SUPPORT_RATIO = 0.16
MAX_SUPPORT_RATIO = 0.55
MAX_SOURCE_FIRST_RATIO = 0.45
MIN_W30_PREVIEW_RMS = 0.14
MIN_ANCHORS = 1
MAX_BAR_SIMILARITY = 0.992
MIN_EVENT_DENSITY = 80.0
MAX_EVENT_DENSITY = 650.0


@dataclass(frozen=True)
class Candidate:
    case_id: str
    window_id: str
    manifest_path: Path
    score: float
    metrics: dict[str, Any]
    issues: list[str]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("showcase_dir", type=Path)
    parser.add_argument("--json-output", type=Path)
    parser.add_argument("--markdown-output", type=Path)
    args = parser.parse_args()

    candidates = collect_candidates(args.showcase_dir)
    if not candidates:
        raise SystemExit(f"no manifest.json files found under {args.showcase_dir / 'packs'}")

    passing = [candidate for candidate in candidates if not candidate.issues]
    best = max(candidates, key=lambda candidate: candidate.score)
    best_passing = max(passing, key=lambda candidate: candidate.score) if passing else None
    result = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if passing else "fail",
        "candidate_count": len(candidates),
        "passing_candidate_count": len(passing),
        "selected_candidate": candidate_record(best_passing if best_passing else best),
        "candidates": [candidate_record(candidate) for candidate in candidates],
        "thresholds": {
            "min_full_rms": MIN_FULL_RMS,
            "min_low_band_rms": MIN_LOW_BAND_RMS,
            "min_support_generated_to_source_rms_ratio": MIN_SUPPORT_RATIO,
            "max_support_generated_to_source_rms_ratio": MAX_SUPPORT_RATIO,
            "max_source_first_generated_to_source_rms_ratio": MAX_SOURCE_FIRST_RATIO,
            "min_w30_preview_rms": MIN_W30_PREVIEW_RMS,
            "min_source_anchor_count": MIN_ANCHORS,
            "max_full_mix_bar_similarity": MAX_BAR_SIMILARITY,
            "min_event_density_per_bar": MIN_EVENT_DENSITY,
            "max_event_density_per_bar": MAX_EVENT_DENSITY,
        },
    }

    if args.json_output:
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(json.dumps(result, indent=2) + "\n")
    if args.markdown_output:
        args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_output.write_text(render_markdown(result))

    if result["result"] != "pass":
        selected = result["selected_candidate"]
        raise SystemExit(
            "no representative showcase musical candidate passed; "
            f"best={selected['case_id']}/{selected['window_id']} "
            f"issues={', '.join(selected['issues'])}"
        )

    selected = result["selected_candidate"]
    print(
        "representative showcase musical candidate: "
        f"{selected['case_id']}/{selected['window_id']} score={selected['score']:.3f}"
    )
    return 0


def collect_candidates(showcase_dir: Path) -> list[Candidate]:
    candidates = []
    for manifest_path in sorted((showcase_dir / "packs").glob("*/*/manifest.json")):
        case_id = manifest_path.parent.parent.name
        window_id = manifest_path.parent.name
        manifest = json.loads(manifest_path.read_text())
        metrics = candidate_metrics(manifest)
        issues = candidate_issues(metrics)
        candidates.append(
            Candidate(
                case_id=case_id,
                window_id=window_id,
                manifest_path=manifest_path,
                score=candidate_score(metrics, issues),
                metrics=metrics,
                issues=issues,
            )
        )
    return candidates


def candidate_metrics(manifest: dict[str, Any]) -> dict[str, Any]:
    metrics = manifest["metrics"]
    full = metrics["full_grid_mix"]
    source_timing = manifest["source_timing"]
    return {
        "full_rms": number(full["signal"]["rms"]),
        "low_band_rms": number(full["low_band"]["rms"]),
        "event_density_per_bar": number(full["signal"]["event_density_per_bar"]),
        "bar_similarity": number(metrics["bar_variation"]["full_grid_mix"]["bar_similarity"]),
        "support_generated_to_source_rms_ratio": number(
            metrics["mix_balance"]["support_generated_to_source_rms_ratio"]
        ),
        "source_first_generated_to_source_rms_ratio": number(
            metrics["mix_balance"]["source_first_generated_to_source_rms_ratio"]
        ),
        "w30_preview_rms": number(metrics["w30_source_chop_profile"]["preview_rms"]),
        "source_anchor_count": int(source_timing["anchor_evidence"]["primary_anchor_count"]),
        "tr909_reason": metrics["tr909_source_profile"]["reason"],
        "grid_use": source_timing["grid_use"],
        "readiness": source_timing["readiness"],
    }


def candidate_issues(metrics: dict[str, Any]) -> list[str]:
    checks = [
        (metrics["full_rms"] >= MIN_FULL_RMS, "full_mix_rms_too_low"),
        (metrics["low_band_rms"] >= MIN_LOW_BAND_RMS, "low_band_rms_too_low"),
        (
            MIN_SUPPORT_RATIO
            <= metrics["support_generated_to_source_rms_ratio"]
            <= MAX_SUPPORT_RATIO,
            "generated_support_balance_out_of_range",
        ),
        (
            metrics["source_first_generated_to_source_rms_ratio"] <= MAX_SOURCE_FIRST_RATIO,
            "source_first_generated_support_masks_source",
        ),
        (metrics["w30_preview_rms"] >= MIN_W30_PREVIEW_RMS, "w30_chop_too_weak"),
        (metrics["source_anchor_count"] >= MIN_ANCHORS, "missing_source_anchor_evidence"),
        (metrics["bar_similarity"] <= MAX_BAR_SIMILARITY, "full_mix_too_static"),
        (
            MIN_EVENT_DENSITY <= metrics["event_density_per_bar"] <= MAX_EVENT_DENSITY,
            "event_density_out_of_range",
        ),
    ]
    return [issue for ok, issue in checks if not ok]


def candidate_score(metrics: dict[str, Any], issues: list[str]) -> float:
    support = clamp(metrics["support_generated_to_source_rms_ratio"] / 0.30, 0.0, 1.4)
    low = clamp(metrics["low_band_rms"] / 0.030, 0.0, 1.3)
    chop = clamp(metrics["w30_preview_rms"] / 0.24, 0.0, 1.3)
    movement = clamp((1.0 - metrics["bar_similarity"]) / 0.020, 0.0, 1.0)
    density = clamp(metrics["event_density_per_bar"] / 280.0, 0.0, 1.2)
    anchors = clamp(metrics["source_anchor_count"] / 8.0, 0.0, 1.0)
    penalty = len(issues) * 0.35
    return support + low + chop + movement + density + anchors - penalty


def candidate_record(candidate: Candidate) -> dict[str, Any]:
    return {
        "case_id": candidate.case_id,
        "window_id": candidate.window_id,
        "manifest": str(candidate.manifest_path),
        "score": round(candidate.score, 6),
        "result": "pass" if not candidate.issues else "fail",
        "issues": candidate.issues,
        "metrics": candidate.metrics,
        "listening_verdict": (
            "musically_convincing_candidate"
            if not candidate.issues
            else "technical_artifact_only"
        ),
    }


def render_markdown(result: dict[str, Any]) -> str:
    selected = result["selected_candidate"]
    lines = [
        "# Representative Showcase Musical Quality",
        "",
        f"- Result: `{result['result']}`",
        f"- Passing candidates: `{result['passing_candidate_count']}` / `{result['candidate_count']}`",
        f"- Selected candidate: `{selected['case_id']}/{selected['window_id']}`",
        f"- Listening verdict: `{selected['listening_verdict']}`",
        "",
        "## Why This Candidate Is Stronger",
        "",
        "- It keeps the source-first mix below the masking threshold.",
        "- It requires generated TR-909 support to be audible rather than decorative.",
        "- It requires W-30 source-chop energy, low-end support, source-anchor evidence, and non-static bar movement.",
        "- It remains a review gate, not an automatic taste oracle.",
        "",
        "## Selected Metrics",
        "",
    ]
    for key, value in selected["metrics"].items():
        lines.append(f"- `{key}`: `{value}`")
    if selected["issues"]:
        lines.extend(["", "## Issues", ""])
        lines.extend(f"- `{issue}`" for issue in selected["issues"])
    return "\n".join(lines) + "\n"


def number(value: Any) -> float:
    if not isinstance(value, (int, float)):
        raise TypeError(f"expected number, got {value!r}")
    return float(value)


def clamp(value: float, lower: float, upper: float) -> float:
    return max(lower, min(upper, value))


if __name__ == "__main__":
    raise SystemExit(main())
