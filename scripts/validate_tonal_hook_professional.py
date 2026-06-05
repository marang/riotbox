#!/usr/bin/env python3
"""Validate tonal-hook professional-output fixture manifests."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from audio_qa_evidence_boundary import apply_evidence_boundary


SCHEMA = "riotbox.tonal_hook_professional.v1"
MIN_FULL_RMS = 0.024
MIN_LOW_BAND_RMS = 0.009
MAX_PEAK_ABS = 0.985
MIN_TRANSIENT_SCORE = 0.300
MIN_SUPPORT_GENERATED_TO_SOURCE_RATIO = 0.200
MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RATIO = 0.160
MAX_IDENTITY_CORRELATION = 0.940
MIN_NORMALIZED_RMS_DELTA = 0.030
MAX_BAR_SIMILARITY = 0.985
MIN_W30_CONTRIBUTION_RATIO = 0.400
MIN_W30_DISTINCT_BAR_PATTERNS = 3
MIN_W30_UNIQUE_SLICE_OFFSETS = 5
MIN_W30_VELOCITY_SPAN = 0.200
MIN_SOURCE_ANCHOR_COUNT = 3
MIN_SOURCE_CONTOUR_DELTA_RMS = 0.00050
MIN_SOURCE_GRID_HIT_RATIO = 0.700
MAX_SOURCE_GRID_PEAK_OFFSET_MS = 45.0
MIN_TR909_CONTRIBUTION_RATIO = 0.050
MIN_MC202_CONTRIBUTION_RATIO = 0.080


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--json-output", type=Path)
    parser.add_argument("--markdown-output", type=Path)
    args = parser.parse_args()

    try:
        report = build_report(args.manifest)
        if args.json_output:
            args.json_output.parent.mkdir(parents=True, exist_ok=True)
            args.json_output.write_text(json.dumps(report, indent=2) + "\n")
        if args.markdown_output:
            args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
            args.markdown_output.write_text(render_markdown(report))
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid tonal-hook professional fixture: {error}", file=sys.stderr)
        return 1

    if report["result"] != "pass":
        print(
            "tonal-hook professional fixture failed: "
            + ", ".join(report["failure_codes"]),
            file=sys.stderr,
        )
        return 1
    print(f"tonal-hook professional fixture passed: {report['case_id']}/{report['window_id']}")
    return 0


def build_report(manifest_path: Path) -> dict[str, Any]:
    manifest = json.loads(manifest_path.read_text())
    if not isinstance(manifest, dict):
        raise ValueError("manifest must be a JSON object")
    metrics = extract_metrics(manifest)
    thresholds = {
        "min_full_rms": MIN_FULL_RMS,
        "min_low_band_rms": MIN_LOW_BAND_RMS,
        "max_peak_abs": MAX_PEAK_ABS,
        "min_transient_score": MIN_TRANSIENT_SCORE,
        "min_support_generated_to_source_ratio": MIN_SUPPORT_GENERATED_TO_SOURCE_RATIO,
        "max_source_first_generated_to_source_ratio": MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RATIO,
        "max_identity_correlation": MAX_IDENTITY_CORRELATION,
        "min_normalized_rms_delta": MIN_NORMALIZED_RMS_DELTA,
        "max_bar_similarity": MAX_BAR_SIMILARITY,
        "min_w30_contribution_ratio": MIN_W30_CONTRIBUTION_RATIO,
        "min_w30_distinct_bar_patterns": MIN_W30_DISTINCT_BAR_PATTERNS,
        "min_w30_unique_slice_offsets": MIN_W30_UNIQUE_SLICE_OFFSETS,
        "min_w30_velocity_span": MIN_W30_VELOCITY_SPAN,
        "min_source_anchor_count": MIN_SOURCE_ANCHOR_COUNT,
        "min_source_contour_delta_rms": MIN_SOURCE_CONTOUR_DELTA_RMS,
        "min_source_grid_hit_ratio": MIN_SOURCE_GRID_HIT_RATIO,
        "max_source_grid_peak_offset_ms": MAX_SOURCE_GRID_PEAK_OFFSET_MS,
        "min_tr909_contribution_ratio": MIN_TR909_CONTRIBUTION_RATIO,
        "min_mc202_contribution_ratio": MIN_MC202_CONTRIBUTION_RATIO,
    }
    failures = failure_codes(metrics)
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if not failures else "fail",
        "agent_verdict": "agent_promising" if not failures else "agent_fail",
        "human_verdict": "unverified",
        "source_family": "tonal_hook",
        "case_id": str(manifest.get("case_id", manifest_path.parent.name)),
        "window_id": str(manifest.get("window_id", manifest_path.parent.name)),
        "manifest": str(manifest_path),
        "thresholds": thresholds,
        "metrics": metrics,
        "failure_codes": failures,
    }
    return apply_evidence_boundary(
        report,
        evidence_role="diagnostic",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
        notes=(
            "Tonal-hook professional fixture validates source-family diagnostic "
            "metrics. It is not product-quality proof."
        ),
    )


def extract_metrics(manifest: dict[str, Any]) -> dict[str, Any]:
    raw = object_or_empty(manifest.get("metrics"))
    full_mix = object_or_empty(raw.get("full_grid_mix") or raw.get("full_mix"))
    signal = object_or_empty(full_mix.get("signal"))
    low_band = object_or_empty(full_mix.get("low_band"))
    mix_balance = object_or_empty(raw.get("mix_balance"))
    identity = object_or_empty(raw.get("identity_collapse") or raw.get("fallback_collapse"))
    movement = object_or_empty(raw.get("all_lane_mix_movement"))
    w30_variation = object_or_empty(raw.get("w30_source_trigger_variation"))
    w30_slice = object_or_empty(raw.get("w30_source_slice_choice"))
    w30_accent = object_or_empty(raw.get("w30_source_accent_dynamics"))
    contour = object_or_empty(raw.get("mc202_source_contour"))
    grid = object_or_empty(raw.get("mc202_source_grid_alignment"))
    source_timing = object_or_empty(manifest.get("source_timing"))
    anchors = object_or_empty(source_timing.get("anchor_evidence"))
    bar_variation = object_or_empty(raw.get("bar_variation"))
    full_bar_variation = object_or_empty(bar_variation.get("full_grid_mix"))

    return {
        "full_rms": number(signal.get("rms")),
        "low_band_rms": number(low_band.get("rms")),
        "peak_abs": number(signal.get("peak_abs")),
        "transient_score": number(signal.get("transient_score")),
        "support_generated_to_source_rms_ratio": number(
            mix_balance.get("support_generated_to_source_rms_ratio")
        ),
        "source_first_generated_to_source_rms_ratio": number(
            mix_balance.get("source_first_generated_to_source_rms_ratio")
        ),
        "identity_correlation": number(identity.get("correlation")),
        "normalized_rms_delta": number(identity.get("normalized_rms_delta")),
        "fallback_used": bool(identity.get("fallback_used", False)),
        "response_signature": str(identity.get("response_signature", "")),
        "bar_similarity": number(full_bar_variation.get("bar_similarity")),
        "tr909_contribution_ratio": number(movement.get("tr909_contribution_ratio")),
        "mc202_contribution_ratio": number(movement.get("mc202_contribution_ratio")),
        "w30_contribution_ratio": number(movement.get("w30_contribution_ratio")),
        "w30_trigger_variation_applied": bool(w30_variation.get("applied", False)),
        "w30_distinct_bar_pattern_count": int(number(w30_variation.get("distinct_bar_pattern_count"))),
        "w30_slice_choice_applied": bool(w30_slice.get("applied", False)),
        "w30_unique_source_offset_count": int(number(w30_slice.get("unique_source_offset_count"))),
        "w30_accent_dynamics_applied": bool(w30_accent.get("applied", False)),
        "w30_velocity_span": number(w30_accent.get("velocity_span")),
        "source_anchor_count": int(number(anchors.get("primary_anchor_count"))),
        "mc202_source_contour_applied": bool(contour.get("applied", False)),
        "mc202_source_contour_origin": str(contour.get("pattern_origin", "unknown")),
        "mc202_source_contour_delta_rms": number(contour.get("source_contour_delta_rms")),
        "mc202_source_grid_hit_ratio": number(grid.get("hit_ratio")),
        "mc202_source_grid_max_peak_offset_ms": number(grid.get("max_peak_offset_ms")),
    }


def failure_codes(metrics: dict[str, Any]) -> list[str]:
    checks = [
        ("full_mix_too_quiet", metrics["full_rms"] >= MIN_FULL_RMS),
        ("low_band_support_too_weak", metrics["low_band_rms"] >= MIN_LOW_BAND_RMS),
        ("full_mix_near_clipping", metrics["peak_abs"] <= MAX_PEAK_ABS),
        ("hook_transient_too_weak", metrics["transient_score"] >= MIN_TRANSIENT_SCORE),
        (
            "generated_support_too_buried",
            metrics["support_generated_to_source_rms_ratio"]
            >= MIN_SUPPORT_GENERATED_TO_SOURCE_RATIO,
        ),
        (
            "source_first_support_masks_hook",
            metrics["source_first_generated_to_source_rms_ratio"]
            <= MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RATIO,
        ),
        ("source_copy_not_transformed_enough", metrics["identity_correlation"] <= MAX_IDENTITY_CORRELATION),
        ("source_response_delta_too_small", metrics["normalized_rms_delta"] >= MIN_NORMALIZED_RMS_DELTA),
        ("fallback_used", not metrics["fallback_used"]),
        ("response_signature_missing", bool(metrics["response_signature"])),
        ("bars_too_static", metrics["bar_similarity"] <= MAX_BAR_SIMILARITY),
        ("tr909_support_too_weak", metrics["tr909_contribution_ratio"] >= MIN_TR909_CONTRIBUTION_RATIO),
        ("mc202_support_too_weak", metrics["mc202_contribution_ratio"] >= MIN_MC202_CONTRIBUTION_RATIO),
        ("w30_hook_not_dominant", metrics["w30_contribution_ratio"] >= MIN_W30_CONTRIBUTION_RATIO),
        ("w30_trigger_variation_missing", metrics["w30_trigger_variation_applied"]),
        (
            "w30_trigger_patterns_too_few",
            metrics["w30_distinct_bar_pattern_count"] >= MIN_W30_DISTINCT_BAR_PATTERNS,
        ),
        ("w30_slice_choice_missing", metrics["w30_slice_choice_applied"]),
        (
            "w30_unique_source_offsets_too_few",
            metrics["w30_unique_source_offset_count"] >= MIN_W30_UNIQUE_SLICE_OFFSETS,
        ),
        ("w30_accent_dynamics_missing", metrics["w30_accent_dynamics_applied"]),
        ("w30_velocity_span_too_small", metrics["w30_velocity_span"] >= MIN_W30_VELOCITY_SPAN),
        ("source_anchor_count_too_low", metrics["source_anchor_count"] >= MIN_SOURCE_ANCHOR_COUNT),
        ("mc202_source_contour_not_applied", metrics["mc202_source_contour_applied"]),
        (
            "mc202_source_contour_not_source_derived",
            metrics["mc202_source_contour_origin"] == "source_derived_contour",
        ),
        (
            "mc202_source_contour_too_flat",
            metrics["mc202_source_contour_delta_rms"] >= MIN_SOURCE_CONTOUR_DELTA_RMS,
        ),
        (
            "mc202_source_grid_hit_ratio_too_low",
            metrics["mc202_source_grid_hit_ratio"] >= MIN_SOURCE_GRID_HIT_RATIO,
        ),
        (
            "mc202_source_grid_peak_offset_too_loose",
            metrics["mc202_source_grid_max_peak_offset_ms"] <= MAX_SOURCE_GRID_PEAK_OFFSET_MS,
        ),
    ]
    return [code for code, ok in checks if not ok]


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def number(value: Any) -> float:
    if isinstance(value, bool) or value is None:
        return 0.0
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# Tonal-Hook Professional Fixture",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Evidence role: `{report['evidence_role']}`",
        f"- Quality proof: `{str(report['quality_proof']).lower()}`",
        f"- Source family: `{report['source_family']}`",
        f"- Case: `{report['case_id']}` / `{report['window_id']}`",
        "",
        "## Failure Codes",
        "",
    ]
    if report["failure_codes"]:
        lines.extend(f"- `{code}`" for code in report["failure_codes"])
    else:
        lines.append("- `none`")
    lines.extend(["", "## Boundary", ""])
    lines.append(
        "This fixture proves deterministic tonal-hook output shape. "
        "It does not claim human musical pass."
    )
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    sys.exit(main())
