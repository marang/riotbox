#!/usr/bin/env python3
"""Validate destructive-variation professional-output reports."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from audio_qa_evidence_boundary import apply_evidence_boundary


SCHEMA = "riotbox.destructive_variation_professional.v1"
SOURCE_SCHEMA = "riotbox.dense_break_performance_pack.v1"
MAX_DROPOUT_TO_STUTTER_RMS_RATIO = 0.007
MAX_DROPOUT_SILENCE_TO_STUTTER_RMS_RATIO = 0.007
MIN_STUTTER_TO_HOOK_TRANSIENT_RATIO = 1.20
MIN_RESTORE_TO_HOOK_TRANSIENT_RATIO = 1.20
MIN_RESTORE_TO_PRESSURE_RMS_RATIO = 1.22
MIN_RESTORE_TO_DROPOUT_SILENCE_RMS_RATIO = 6.00
MAX_ADJACENT_BAR_CORRELATION = 0.95
MIN_SOURCE_TO_PERFORMANCE_CORRELATION = 0.20
MAX_SOURCE_TO_PERFORMANCE_CORRELATION = 0.80
MIN_DROPOUT_STUTTER_RMS = 0.09
MIN_RESTORE_RMS = 0.18
MAX_FULL_PEAK_ABS = 0.985
MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES = 256.0
MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES = 512.0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("performance_report", type=Path)
    parser.add_argument("--json-output", type=Path)
    parser.add_argument("--markdown-output", type=Path)
    args = parser.parse_args()

    try:
        report = build_report(args.performance_report)
        if args.json_output:
            args.json_output.parent.mkdir(parents=True, exist_ok=True)
            args.json_output.write_text(json.dumps(report, indent=2) + "\n")
        if args.markdown_output:
            args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
            args.markdown_output.write_text(render_markdown(report))
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid destructive variation professional report: {error}", file=sys.stderr)
        return 1

    if report["result"] != "pass":
        print(
            "destructive variation professional report failed: "
            + ", ".join(report["failure_codes"]),
            file=sys.stderr,
        )
        return 1
    print("destructive variation professional report passed")
    return 0


def build_report(performance_report: Path) -> dict[str, Any]:
    source = json.loads(performance_report.read_text())
    proof = object_or_empty(source.get("proof"))
    metrics = object_or_empty(source.get("metrics"))
    extracted = {
        "dropout_to_stutter_rms_ratio": number(proof.get("dropout_to_stutter_rms_ratio")),
        "dropout_silence_to_stutter_rms_ratio": number(
            proof.get("dropout_silence_to_stutter_rms_ratio")
        ),
        "stutter_to_hook_transient_ratio": number(proof.get("stutter_to_hook_transient_ratio")),
        "restore_to_hook_transient_ratio": number(proof.get("restore_to_hook_transient_ratio")),
        "restore_to_pressure_rms_ratio": number(proof.get("restore_to_pressure_rms_ratio")),
        "restore_to_dropout_silence_rms_ratio": number(
            proof.get("restore_to_dropout_silence_rms_ratio")
        ),
        "max_adjacent_bar_correlation": number(proof.get("max_adjacent_bar_correlation")),
        "source_to_performance_correlation": number(proof.get("source_to_performance_correlation")),
        "destructive_gesture_source_derived": number(
            proof.get("destructive_gesture_source_derived")
        ),
        "destructive_static_distance_frames": number(
            proof.get("destructive_static_distance_frames")
        ),
        "destructive_offset_distance_frames": number(
            proof.get("destructive_offset_distance_frames")
        ),
        "dropout_stutter_rms": metric_number(metrics, "dropout_stutter", "rms"),
        "restore_hit_rms": metric_number(metrics, "restore_hit", "rms"),
        "rebuild_only_performance_peak_abs": metric_number(
            metrics, "rebuild_only_performance", "peak_abs"
        ),
    }
    thresholds = {
        "max_dropout_to_stutter_rms_ratio": MAX_DROPOUT_TO_STUTTER_RMS_RATIO,
        "max_dropout_silence_to_stutter_rms_ratio": (
            MAX_DROPOUT_SILENCE_TO_STUTTER_RMS_RATIO
        ),
        "min_stutter_to_hook_transient_ratio": MIN_STUTTER_TO_HOOK_TRANSIENT_RATIO,
        "min_restore_to_hook_transient_ratio": MIN_RESTORE_TO_HOOK_TRANSIENT_RATIO,
        "min_restore_to_pressure_rms_ratio": MIN_RESTORE_TO_PRESSURE_RMS_RATIO,
        "min_restore_to_dropout_silence_rms_ratio": (
            MIN_RESTORE_TO_DROPOUT_SILENCE_RMS_RATIO
        ),
        "max_adjacent_bar_correlation": MAX_ADJACENT_BAR_CORRELATION,
        "min_source_to_performance_correlation": MIN_SOURCE_TO_PERFORMANCE_CORRELATION,
        "max_source_to_performance_correlation": MAX_SOURCE_TO_PERFORMANCE_CORRELATION,
        "min_dropout_stutter_rms": MIN_DROPOUT_STUTTER_RMS,
        "min_restore_rms": MIN_RESTORE_RMS,
        "max_full_peak_abs": MAX_FULL_PEAK_ABS,
        "min_destructive_static_distance_frames": MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES,
        "min_destructive_offset_distance_frames": MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES,
    }
    failures = failure_codes(source, extracted)
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if not failures else "fail",
        "agent_verdict": "agent_promising" if not failures else "agent_fail",
        "human_verdict": "unverified",
        "source_report_schema": source.get("schema"),
        "source_report_result": source.get("result"),
        "performance_report": str(performance_report),
        "thresholds": thresholds,
        "metrics": extracted,
        "failure_codes": failures,
    }
    return apply_evidence_boundary(
        report,
        evidence_role="diagnostic",
        source_backed=bool(source.get("source_backed", True)),
        source_timing_backed=bool(source.get("source_timing_backed", True)),
        scripted_generation=bool(source.get("scripted_generation", True)),
        notes=(
            "Destructive-variation report validates a diagnostic render shape. "
            "It is not product-quality proof while the source report is scripted."
        ),
    )


def failure_codes(source: dict[str, Any], metrics: dict[str, float]) -> list[str]:
    proof = object_or_empty(source.get("proof"))
    checks = [
        ("source_report_schema_mismatch", source.get("schema") == SOURCE_SCHEMA),
        ("source_report_not_passed", source.get("result") == "pass"),
        (
            "dropout_not_contrasting_with_stutter",
            metrics["dropout_to_stutter_rms_ratio"] <= MAX_DROPOUT_TO_STUTTER_RMS_RATIO,
        ),
        (
            "dropout_silence_metric_missing",
            "dropout_silence_to_stutter_rms_ratio" in proof,
        ),
        (
            "dropout_silence_not_deep_enough_before_stutter",
            metrics["dropout_silence_to_stutter_rms_ratio"]
            <= MAX_DROPOUT_SILENCE_TO_STUTTER_RMS_RATIO,
        ),
        (
            "stutter_lacks_transient_impact",
            metrics["stutter_to_hook_transient_ratio"] >= MIN_STUTTER_TO_HOOK_TRANSIENT_RATIO,
        ),
        (
            "restore_hit_lacks_break_transient_impact",
            metrics["restore_to_hook_transient_ratio"] >= MIN_RESTORE_TO_HOOK_TRANSIENT_RATIO,
        ),
        (
            "restore_not_bigger_than_pressure",
            metrics["restore_to_pressure_rms_ratio"] >= MIN_RESTORE_TO_PRESSURE_RMS_RATIO,
        ),
        (
            "restore_cut_slam_metric_missing",
            "restore_to_dropout_silence_rms_ratio" in proof,
        ),
        (
            "restore_does_not_slam_out_of_cut",
            metrics["restore_to_dropout_silence_rms_ratio"]
            >= MIN_RESTORE_TO_DROPOUT_SILENCE_RMS_RATIO,
        ),
        ("bars_too_static", metrics["max_adjacent_bar_correlation"] <= MAX_ADJACENT_BAR_CORRELATION),
        (
            "source_not_transformed_but_present",
            MIN_SOURCE_TO_PERFORMANCE_CORRELATION
            <= metrics["source_to_performance_correlation"]
            <= MAX_SOURCE_TO_PERFORMANCE_CORRELATION,
        ),
        (
            "destructive_gesture_not_source_derived",
            metrics["destructive_gesture_source_derived"] >= 1.0,
        ),
        (
            "destructive_gesture_collapsed_to_fixed_choice",
            metrics["destructive_static_distance_frames"] >= MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES,
        ),
        (
            "destructive_gesture_not_enough_offset_contrast",
            metrics["destructive_offset_distance_frames"] >= MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES,
        ),
        ("dropout_stutter_too_quiet", metrics["dropout_stutter_rms"] >= MIN_DROPOUT_STUTTER_RMS),
        ("restore_too_quiet_after_cut", metrics["restore_hit_rms"] >= MIN_RESTORE_RMS),
        (
            "rebuild_only_performance_near_clipping",
            metrics["rebuild_only_performance_peak_abs"] <= MAX_FULL_PEAK_ABS,
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


def metric_number(metrics: dict[str, Any], section: str, field: str) -> float:
    return number(object_or_empty(metrics.get(section)).get(field))


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# Destructive Variation Professional Report",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Evidence role: `{report['evidence_role']}`",
        f"- Quality proof: `{str(report['quality_proof']).lower()}`",
        f"- Scripted generation: `{str(report['scripted_generation']).lower()}`",
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
        "This report proves deterministic destructive variation shape. "
        "It is diagnostic evidence, not product-quality proof or human musical pass."
    )
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    sys.exit(main())
