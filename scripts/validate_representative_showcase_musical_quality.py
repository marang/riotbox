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
MAX_SOURCE_FIRST_RATIO = 0.08
MIN_W30_PREVIEW_RMS = 0.14
MIN_ANCHORS = 1
MAX_BAR_SIMILARITY = 0.992
MIN_EVENT_DENSITY = 40.0
MAX_EVENT_DENSITY = 650.0
MIN_W30_OFFBEAT_TRIGGERS = 1
MIN_W30_DISTINCT_BAR_PATTERNS = 2
MIN_W30_UNIQUE_SLICE_OFFSETS = 4
MIN_W30_ACCENT_DISTINCT_VELOCITIES = 3
MIN_W30_ACCENT_VELOCITY_SPAN = 0.12
MIN_TR909_KICK_PRESSURE_LOW_BAND_RATIO = 1.06
TR909_SOURCE_EVIDENCE_ROLE = "tr909_source_profile_and_accent_dynamics"
MIN_TR909_KICK_PRESSURE_ANCHORS = 2
MIN_TR909_ACCENT_DISTINCT_ACCENTS = 3
MIN_TR909_ACCENT_SPAN = 0.22
MIN_MC202_BASS_PRESSURE_RMS = 0.003
MIN_MC202_BASS_PRESSURE_LOW_BAND_RMS = 0.001
MIN_MC202_DISTINCT_BAR_PROFILES = 2
MAX_MC202_BAR_SIMILARITY = 0.985
MIN_MC202_SOURCE_GRID_HIT_RATIO = 0.50
MAX_MC202_SOURCE_GRID_PEAK_OFFSET_MS = 70.0
MIN_MC202_SOURCE_CONTOUR_DELTA_RMS = 0.00025
MIN_ALL_LANE_MIX_RMS_DELTA = 0.012
MAX_ALL_LANE_MIX_CORRELATION = 0.999
MIN_ALL_LANE_MIX_CONTRIBUTION_RATIO = 0.015
MIN_ALL_LANE_MIX_GENERATED_TO_W30_RATIO = 0.08


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
    parser.add_argument("--automated-musical-fitness-report", type=Path)
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
            "min_w30_offbeat_trigger_count": MIN_W30_OFFBEAT_TRIGGERS,
            "min_w30_distinct_bar_pattern_count": MIN_W30_DISTINCT_BAR_PATTERNS,
            "min_w30_unique_slice_offset_count": MIN_W30_UNIQUE_SLICE_OFFSETS,
            "min_w30_accent_distinct_velocity_count": MIN_W30_ACCENT_DISTINCT_VELOCITIES,
            "min_w30_accent_velocity_span": MIN_W30_ACCENT_VELOCITY_SPAN,
            "min_tr909_kick_pressure_low_band_ratio": MIN_TR909_KICK_PRESSURE_LOW_BAND_RATIO,
            "tr909_source_evidence_role": TR909_SOURCE_EVIDENCE_ROLE,
            "min_tr909_kick_pressure_anchor_count": MIN_TR909_KICK_PRESSURE_ANCHORS,
            "min_tr909_accent_distinct_accent_count": MIN_TR909_ACCENT_DISTINCT_ACCENTS,
            "min_tr909_accent_span": MIN_TR909_ACCENT_SPAN,
            "min_mc202_bass_pressure_rms": MIN_MC202_BASS_PRESSURE_RMS,
            "min_mc202_bass_pressure_low_band_rms": MIN_MC202_BASS_PRESSURE_LOW_BAND_RMS,
            "min_mc202_distinct_bar_profiles": MIN_MC202_DISTINCT_BAR_PROFILES,
            "max_mc202_bar_similarity": MAX_MC202_BAR_SIMILARITY,
            "min_mc202_source_grid_hit_ratio": MIN_MC202_SOURCE_GRID_HIT_RATIO,
            "max_mc202_source_grid_peak_offset_ms": MAX_MC202_SOURCE_GRID_PEAK_OFFSET_MS,
            "min_mc202_source_contour_delta_rms": MIN_MC202_SOURCE_CONTOUR_DELTA_RMS,
            "min_all_lane_mix_rms_delta": MIN_ALL_LANE_MIX_RMS_DELTA,
            "max_all_lane_mix_correlation": MAX_ALL_LANE_MIX_CORRELATION,
            "min_all_lane_mix_contribution_ratio": MIN_ALL_LANE_MIX_CONTRIBUTION_RATIO,
            "min_all_lane_mix_generated_to_w30_ratio": MIN_ALL_LANE_MIX_GENERATED_TO_W30_RATIO,
        },
    }
    automated_musical_fitness = load_automated_musical_fitness_summary(
        args.showcase_dir,
        args.automated_musical_fitness_report,
    )
    if automated_musical_fitness is not None:
        result["automated_musical_fitness"] = automated_musical_fitness

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
    w30_variation = metrics.get("w30_source_trigger_variation", {})
    w30_slice_choice = metrics.get("w30_source_slice_choice", {})
    w30_accent_dynamics = metrics.get("w30_source_accent_dynamics", {})
    tr909_kick_pressure = metrics.get("tr909_kick_pressure", {})
    tr909_accent_dynamics = metrics.get("tr909_source_accent_dynamics", {})
    mc202_bass_pressure = metrics.get("mc202_bass_pressure", {})
    mc202_source_contour = metrics.get("mc202_source_contour", {})
    mc202_source_grid_alignment = metrics.get("mc202_source_grid_alignment", {})
    mix_movement = metrics.get("all_lane_mix_movement", {})
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
        "w30_pattern_origin": str(w30_variation.get("pattern_origin", "unknown")),
        "w30_trigger_variation_applied": bool(w30_variation.get("applied", False)),
        "w30_offbeat_trigger_count": int(w30_variation.get("offbeat_trigger_count", 0)),
        "w30_distinct_bar_pattern_count": int(
            w30_variation.get("distinct_bar_pattern_count", 0)
        ),
        "w30_max_quantized_offset_ms": number(
            w30_variation.get("max_quantized_offset_ms", 999.0)
        ),
        "w30_slice_choice_applied": bool(w30_slice_choice.get("applied", False)),
        "w30_unique_slice_offset_count": int(
            w30_slice_choice.get("unique_source_offset_count", 0)
        ),
        "w30_slice_offset_span_samples": int(
            w30_slice_choice.get("selected_offset_span_samples", 0)
        ),
        "w30_accent_dynamics_applied": bool(w30_accent_dynamics.get("applied", False)),
        "w30_accent_distinct_velocity_count": int(
            w30_accent_dynamics.get("distinct_velocity_count", 0)
        ),
        "w30_accent_velocity_span": number(w30_accent_dynamics.get("velocity_span", 0.0)),
        "w30_accent_source_energy_span": number(
            w30_accent_dynamics.get("source_energy_span", 0.0)
        ),
        "tr909_kick_pressure_applied": bool(tr909_kick_pressure.get("applied", False)),
        "tr909_pattern_origin": str(tr909_kick_pressure.get("pattern_origin", "unknown")),
        "tr909_source_evidence_role": str(
            tr909_kick_pressure.get("source_evidence_role", "unknown")
        ),
        "tr909_source_profile_reason": str(
            tr909_kick_pressure.get("source_profile_reason", "unknown")
        ),
        "tr909_kick_pressure_anchor_count": int(tr909_kick_pressure.get("anchor_count", 0)),
        "tr909_kick_pressure_low_band_ratio": number(
            tr909_kick_pressure.get("low_band_rms_ratio", 0.0)
        ),
        "tr909_accent_pattern_origin": str(
            tr909_accent_dynamics.get("pattern_origin", "unknown")
        ),
        "tr909_accent_dynamics_applied": bool(tr909_accent_dynamics.get("applied", False)),
        "tr909_accent_distinct_accent_count": int(
            tr909_accent_dynamics.get("distinct_accent_count", 0)
        ),
        "tr909_accent_span": number(tr909_accent_dynamics.get("accent_span", 0.0)),
        "tr909_accent_min_required_span": number(
            tr909_accent_dynamics.get("min_required_accent_span", MIN_TR909_ACCENT_SPAN)
        ),
        "tr909_accent_source_energy_span": number(
            tr909_accent_dynamics.get("source_energy_span", 0.0)
        ),
        "mc202_bass_pressure_applied": bool(mc202_bass_pressure.get("applied", False)),
        "mc202_pattern_origin": str(mc202_bass_pressure.get("pattern_origin", "unknown")),
        "mc202_phrase_variation_applied": bool(
            mc202_bass_pressure.get("phrase_variation_applied", False)
        ),
        "mc202_distinct_bar_profile_count": int(
            mc202_bass_pressure.get("distinct_bar_profile_count", 0)
        ),
        "mc202_bar_similarity": number(mc202_bass_pressure.get("bar_similarity", 1.0)),
        "mc202_bass_pressure_rms": number(mc202_bass_pressure.get("signal_rms", 0.0)),
        "mc202_bass_pressure_low_band_rms": number(
            mc202_bass_pressure.get("low_band_rms", 0.0)
        ),
        "mc202_source_grid_hit_ratio": number(
            mc202_source_grid_alignment.get("hit_ratio", 0.0)
        ),
        "mc202_source_grid_max_peak_offset_ms": number(
            mc202_source_grid_alignment.get("max_peak_offset_ms", 999.0)
        ),
        "mc202_source_grid_max_allowed_peak_offset_ms": number(
            mc202_source_grid_alignment.get(
                "max_allowed_peak_offset_ms", MAX_MC202_SOURCE_GRID_PEAK_OFFSET_MS
            )
        ),
        "mc202_source_contour_applied": bool(mc202_source_contour.get("applied", False)),
        "mc202_source_contour_origin": str(
            mc202_source_contour.get("pattern_origin", "unknown")
        ),
        "mc202_source_contour_hint": str(mc202_source_contour.get("contour_hint", "unknown")),
        "mc202_source_contour_delta_rms": number(
            mc202_source_contour.get("source_contour_delta_rms", 0.0)
        ),
        "mc202_source_contour_low_band_energy_ratio": number(
            mc202_source_contour.get("low_band_energy_ratio", 0.0)
        ),
        "mc202_source_contour_event_density_per_bar": number(
            mc202_source_contour.get("event_density_per_bar", 0.0)
        ),
        "all_lane_mix_movement_applied": bool(mix_movement.get("applied", False)),
        "all_lane_mix_rms_delta": number(
            mix_movement.get("source_first_to_support_rms_delta", 0.0)
        ),
        "all_lane_mix_correlation": number(
            mix_movement.get("source_first_to_support_correlation", 1.0)
        ),
        "all_lane_mix_tr909_contribution_ratio": number(
            mix_movement.get("tr909_contribution_ratio", 0.0)
        ),
        "all_lane_mix_mc202_contribution_ratio": number(
            mix_movement.get("mc202_contribution_ratio", 0.0)
        ),
        "all_lane_mix_w30_contribution_ratio": number(
            mix_movement.get("w30_contribution_ratio", 0.0)
        ),
        "all_lane_mix_generated_to_w30_contribution_ratio": number(
            mix_movement.get("generated_to_w30_contribution_ratio", 0.0)
        ),
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
        (
            metrics["w30_trigger_variation_applied"],
            "w30_trigger_variation_not_applied",
        ),
        (
            metrics["w30_distinct_bar_pattern_count"] >= MIN_W30_DISTINCT_BAR_PATTERNS,
            "w30_source_offset_variation_too_static",
        ),
        (
            metrics["w30_slice_choice_applied"],
            "w30_slice_choice_not_applied",
        ),
        (
            metrics["w30_unique_slice_offset_count"] >= MIN_W30_UNIQUE_SLICE_OFFSETS,
            "w30_slice_choice_too_static",
        ),
        (
            metrics["w30_accent_dynamics_applied"],
            "w30_accent_dynamics_not_applied",
        ),
        (
            metrics["w30_accent_distinct_velocity_count"] >= MIN_W30_ACCENT_DISTINCT_VELOCITIES,
            "w30_accent_velocity_count_too_low",
        ),
        (
            metrics["w30_accent_velocity_span"] >= MIN_W30_ACCENT_VELOCITY_SPAN,
            "w30_accent_velocity_span_too_flat",
        ),
        (
            metrics["tr909_kick_pressure_applied"],
            "tr909_kick_pressure_not_applied",
        ),
        (
            metrics["tr909_pattern_origin"] == "source_derived",
            "tr909_kick_pressure_not_source_derived",
        ),
        (
            metrics["tr909_source_evidence_role"] == TR909_SOURCE_EVIDENCE_ROLE,
            "tr909_kick_pressure_missing_source_evidence",
        ),
        (
            metrics["tr909_source_profile_reason"].startswith("source_"),
            "tr909_kick_pressure_source_reason_missing",
        ),
        (
            metrics["tr909_kick_pressure_anchor_count"] >= MIN_TR909_KICK_PRESSURE_ANCHORS,
            "tr909_kick_pressure_anchor_count_too_low",
        ),
        (
            metrics["tr909_kick_pressure_low_band_ratio"]
            >= MIN_TR909_KICK_PRESSURE_LOW_BAND_RATIO,
            "tr909_kick_pressure_too_decorative",
        ),
        (
            metrics["tr909_accent_pattern_origin"] == "source_derived",
            "tr909_accent_dynamics_not_source_derived",
        ),
        (
            metrics["tr909_accent_dynamics_applied"],
            "tr909_accent_dynamics_not_applied",
        ),
        (
            metrics["tr909_accent_distinct_accent_count"] >= MIN_TR909_ACCENT_DISTINCT_ACCENTS,
            "tr909_accent_count_too_low",
        ),
        (
            metrics["tr909_accent_span"]
            >= max(MIN_TR909_ACCENT_SPAN, metrics["tr909_accent_min_required_span"]),
            "tr909_accent_span_too_flat",
        ),
        (
            metrics["mc202_bass_pressure_applied"],
            "mc202_bass_pressure_not_applied",
        ),
        (
            metrics["mc202_phrase_variation_applied"],
            "mc202_phrase_variation_not_applied",
        ),
        (
            metrics["mc202_distinct_bar_profile_count"] >= MIN_MC202_DISTINCT_BAR_PROFILES,
            "mc202_bar_profiles_too_static",
        ),
        (
            metrics["mc202_bar_similarity"] <= MAX_MC202_BAR_SIMILARITY,
            "mc202_bar_similarity_too_static",
        ),
        (
            metrics["mc202_bass_pressure_rms"] >= MIN_MC202_BASS_PRESSURE_RMS,
            "mc202_bass_pressure_too_quiet",
        ),
        (
            metrics["mc202_bass_pressure_low_band_rms"] >= MIN_MC202_BASS_PRESSURE_LOW_BAND_RMS,
            "mc202_bass_pressure_low_band_too_weak",
        ),
        (
            metrics["mc202_source_grid_hit_ratio"] >= MIN_MC202_SOURCE_GRID_HIT_RATIO,
            "mc202_source_grid_alignment_too_weak",
        ),
        (
            metrics["mc202_source_grid_max_peak_offset_ms"]
            <= metrics["mc202_source_grid_max_allowed_peak_offset_ms"],
            "mc202_source_grid_peak_offset_too_high",
        ),
        (
            metrics["mc202_source_contour_applied"],
            "mc202_source_contour_not_applied",
        ),
        (
            metrics["mc202_source_contour_origin"] == "source_derived_contour",
            "mc202_source_contour_origin_not_source_derived",
        ),
        (
            metrics["mc202_source_contour_delta_rms"] >= MIN_MC202_SOURCE_CONTOUR_DELTA_RMS,
            "mc202_source_contour_delta_too_low",
        ),
        (
            metrics["all_lane_mix_movement_applied"],
            "all_lane_mix_movement_not_applied",
        ),
        (
            metrics["all_lane_mix_rms_delta"] >= MIN_ALL_LANE_MIX_RMS_DELTA,
            "all_lane_mix_delta_too_low",
        ),
        (
            metrics["all_lane_mix_correlation"] <= MAX_ALL_LANE_MIX_CORRELATION,
            "all_lane_mix_correlation_too_high",
        ),
        (
            metrics["all_lane_mix_tr909_contribution_ratio"]
            >= MIN_ALL_LANE_MIX_CONTRIBUTION_RATIO,
            "all_lane_mix_tr909_too_weak",
        ),
        (
            metrics["all_lane_mix_mc202_contribution_ratio"]
            >= MIN_ALL_LANE_MIX_CONTRIBUTION_RATIO,
            "all_lane_mix_mc202_too_weak",
        ),
        (
            metrics["all_lane_mix_w30_contribution_ratio"]
            >= MIN_ALL_LANE_MIX_CONTRIBUTION_RATIO,
            "all_lane_mix_w30_too_weak",
        ),
        (
            metrics["all_lane_mix_generated_to_w30_contribution_ratio"]
            >= MIN_ALL_LANE_MIX_GENERATED_TO_W30_RATIO,
            "all_lane_mix_generated_support_too_weak",
        ),
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
    trigger_variation = 1.0 if metrics["w30_trigger_variation_applied"] else 0.0
    pattern_variation = clamp(metrics["w30_distinct_bar_pattern_count"] / 4.0, 0.0, 1.0)
    slice_variation = clamp(metrics["w30_unique_slice_offset_count"] / 6.0, 0.0, 1.0)
    w30_accent_variation = clamp(metrics["w30_accent_velocity_span"] / 0.28, 0.0, 1.1)
    w30_accent_count = clamp(metrics["w30_accent_distinct_velocity_count"] / 5.0, 0.0, 1.0)
    kick_pressure = clamp(
        (metrics["tr909_kick_pressure_low_band_ratio"] - 1.0) / 0.18,
        0.0,
        1.0,
    )
    tr909_accent_variation = clamp(metrics["tr909_accent_span"] / 0.50, 0.0, 1.0)
    tr909_accent_count = clamp(metrics["tr909_accent_distinct_accent_count"] / 4.0, 0.0, 1.0)
    bass_pressure = clamp(metrics["mc202_bass_pressure_rms"] / 0.008, 0.0, 1.1)
    bass_low = clamp(metrics["mc202_bass_pressure_low_band_rms"] / 0.004, 0.0, 1.1)
    bass_variation = clamp(metrics["mc202_distinct_bar_profile_count"] / 3.0, 0.0, 1.0)
    bass_movement = clamp((1.0 - metrics["mc202_bar_similarity"]) / 0.150, 0.0, 1.0)
    bass_alignment = clamp(metrics["mc202_source_grid_hit_ratio"], 0.0, 1.0)
    bass_contour = clamp(metrics["mc202_source_contour_delta_rms"] / 0.0012, 0.0, 1.0)
    bass_contour_energy = clamp(
        max(
            metrics["mc202_source_contour_low_band_energy_ratio"],
            metrics["mc202_source_contour_event_density_per_bar"] / 12.0,
        ),
        0.0,
        1.0,
    )
    mix_delta = clamp(metrics["all_lane_mix_rms_delta"] / 0.035, 0.0, 1.1)
    mix_decorr = clamp((1.0 - metrics["all_lane_mix_correlation"]) / 0.120, 0.0, 1.0)
    mix_lane_contribution = clamp(
        min(
            metrics["all_lane_mix_tr909_contribution_ratio"],
            metrics["all_lane_mix_mc202_contribution_ratio"],
            metrics["all_lane_mix_w30_contribution_ratio"],
        )
        / 0.070,
        0.0,
        1.0,
    )
    mix_generated_support = clamp(
        metrics["all_lane_mix_generated_to_w30_contribution_ratio"] / 0.22,
        0.0,
        1.0,
    )
    movement = clamp((1.0 - metrics["bar_similarity"]) / 0.020, 0.0, 1.0)
    density = clamp(metrics["event_density_per_bar"] / 280.0, 0.0, 1.2)
    anchors = clamp(metrics["source_anchor_count"] / 8.0, 0.0, 1.0)
    penalty = len(issues) * 0.35
    return (
        support
        + low
        + chop
        + trigger_variation
        + pattern_variation
        + slice_variation
        + w30_accent_variation
        + w30_accent_count
        + kick_pressure
        + tr909_accent_variation
        + tr909_accent_count
        + bass_pressure
        + bass_low
        + bass_variation
        + bass_movement
        + bass_alignment
        + bass_contour
        + bass_contour_energy
        + mix_delta
        + mix_decorr
        + mix_lane_contribution
        + mix_generated_support
        + movement
        + density
        + anchors
        - penalty
    )


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


def load_automated_musical_fitness_summary(
    showcase_dir: Path,
    explicit_path: Path | None,
) -> dict[str, Any] | None:
    report_path = explicit_path or showcase_dir / "validation" / "automated-musical-fitness.json"
    if not report_path.is_file():
        if explicit_path is not None:
            raise ValueError(f"automated musical fitness report not found: {report_path}")
        return None
    report = json.loads(report_path.read_text())
    required = (
        "technical_status",
        "automated_musical_fitness_status",
        "human_verdict",
        "selected_candidate",
        "failure_codes",
        "score_breakdown",
    )
    missing = [key for key in required if key not in report]
    if missing:
        raise ValueError(
            f"{report_path}: automated musical fitness report missing {', '.join(missing)}"
        )
    selected = report["selected_candidate"]
    if not isinstance(selected, dict):
        raise ValueError(f"{report_path}: selected_candidate must be an object")
    return {
        "schema": report.get("schema"),
        "technical_status": report["technical_status"],
        "automated_musical_fitness_status": report["automated_musical_fitness_status"],
        "human_verdict": report["human_verdict"],
        "selected_candidate": {
            "case_id": selected.get("case_id"),
            "window_id": selected.get("window_id"),
            "manifest": selected.get("manifest"),
            "result": selected.get("result"),
            "score": selected.get("score"),
        },
        "failure_codes": report["failure_codes"],
        "score_breakdown": compact_score_breakdown(report["score_breakdown"]),
    }


def compact_score_breakdown(score_breakdown: Any) -> dict[str, Any]:
    if not isinstance(score_breakdown, dict):
        raise ValueError("automated musical fitness score_breakdown must be an object")
    compact = {}
    for key, value in score_breakdown.items():
        if not isinstance(value, dict):
            continue
        compact[key] = {
            "status": value.get("status"),
            "score": value.get("score"),
            "failure_codes": value.get("failure_codes", []),
        }
    return compact


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
        "- It requires TR-909 kick-pressure proof so the drum layer adds measurable low-end body.",
        "- It requires TR-909 source evidence and source-shaped accent dynamics before drum support counts as product-quality support.",
        "- It requires MC-202 bass-pressure proof so the bass lane is audible, not only named in the manifest.",
        "- It requires MC-202 phrase variation so the bass lane does not collapse into one repeated support cell.",
        "- It requires MC-202 source-grid proof so the bass lane cannot drift behind stronger aligned stems.",
        "- It requires MC-202 source-contour proof so source sections shape bass contour without claiming phrase extraction.",
        "- It requires all-lane mix movement so the two listening mixes differ and every lane contributes.",
        "- It requires W-30 source-chop energy, low-end support, source-anchor evidence, and non-static bar movement.",
        "- It requires W-30 trigger variation to be applied rather than relying on a static repeated chop.",
        "- It requires W-30 slice-choice variation so repeated triggers do not keep reading the same source offset.",
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
    if "automated_musical_fitness" in result:
        fitness = result["automated_musical_fitness"]
        fitness_selected = fitness["selected_candidate"]
        lines.extend(
            [
                "",
                "## Automated Musical Fitness",
                "",
                f"- Technical status: `{fitness['technical_status']}`",
                "- Automated musical fitness status: "
                f"`{fitness['automated_musical_fitness_status']}`",
                f"- Human verdict: `{fitness['human_verdict']}`",
                "- Selected automated candidate: "
                f"`{fitness_selected.get('case_id')}/{fitness_selected.get('window_id')}`",
            ]
        )
        if fitness["failure_codes"]:
            lines.append(
                "- Failure codes: "
                + ", ".join(f"`{code}`" for code in fitness["failure_codes"])
            )
        else:
            lines.append("- Failure codes: none")
    return "\n".join(lines) + "\n"


def number(value: Any) -> float:
    if not isinstance(value, (int, float)):
        raise TypeError(f"expected number, got {value!r}")
    return float(value)


def clamp(value: float, lower: float, upper: float) -> float:
    return max(lower, min(upper, value))


if __name__ == "__main__":
    raise SystemExit(main())
