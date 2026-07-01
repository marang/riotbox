#!/usr/bin/env python3
"""Validate automated musical fitness for Riotbox audio QA artifacts."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.automated_musical_fitness.v1"

MIN_FULL_RMS = 0.015
MIN_LOW_BAND_RMS = 0.006
MAX_PEAK_ABS = 0.995
MIN_SUPPORT_GENERATED_TO_SOURCE_RATIO = 0.12
MAX_SUPPORT_GENERATED_TO_SOURCE_RATIO = 0.65
MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RATIO = 0.08
MAX_IDENTITY_CORRELATION = 0.999
MIN_NORMALIZED_RMS_DELTA = 0.012
MIN_SOURCE_ANCHOR_COUNT = 1
MIN_SOURCE_CONTOUR_DELTA_RMS = 0.00025
MAX_BAR_SIMILARITY = 0.992
MIN_EVENT_DENSITY_PER_BAR = 30.0
MAX_EVENT_DENSITY_PER_BAR = 700.0
MIN_W30_DISTINCT_BAR_PATTERNS = 2
MIN_W30_UNIQUE_SLICE_OFFSETS = 3
MIN_W30_VELOCITY_SPAN = 0.10
MIN_LANE_CONTRIBUTION_RATIO = 0.015
MIN_GENERATED_TO_W30_CONTRIBUTION_RATIO = 0.06
MIN_TR909_LOW_BAND_RATIO = 1.04
TR909_SOURCE_EVIDENCE_ROLE = "tr909_source_profile_and_accent_dynamics"
MIN_TR909_KICK_PRESSURE_ANCHORS = 2
MIN_TR909_ACCENT_DISTINCT_ACCENTS = 3
MIN_TR909_ACCENT_SPAN = 0.22
MIN_MC202_BASS_RMS = 0.0025
MIN_TRANSIENT_SCORE = 0.20
MIN_SOURCE_GRID_HIT_RATIO = 0.50
MAX_SOURCE_GRID_PEAK_OFFSET_MS = 70.0


@dataclass(frozen=True)
class Candidate:
    case_id: str
    window_id: str
    manifest_path: Path
    metrics: dict[str, Any]
    score_breakdown: dict[str, Any]
    failure_codes: tuple[str, ...]

    @property
    def score(self) -> float:
        return round(
            sum(
                float(section["score"])
                for section in self.score_breakdown.values()
                if isinstance(section, dict)
            ),
            6,
        )

    @property
    def result(self) -> str:
        return "pass" if not self.failure_codes else "fail"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", nargs="+", type=Path)
    parser.add_argument("--json-output", type=Path)
    parser.add_argument("--markdown-output", type=Path)
    args = parser.parse_args()

    try:
        candidates = collect_candidates(args.paths)
        if not candidates:
            raise ValueError("no manifest candidates found")
        report = build_report(candidates)
        if args.json_output:
            args.json_output.parent.mkdir(parents=True, exist_ok=True)
            args.json_output.write_text(json.dumps(report, indent=2) + "\n")
        if args.markdown_output:
            args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
            args.markdown_output.write_text(render_markdown(report))
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid automated musical fitness: {error}", file=sys.stderr)
        return 1

    selected = report["selected_candidate"]
    if report["result"] != "pass":
        codes = ", ".join(report["failure_codes"])
        print(
            "automated musical fitness failed: "
            f"{selected['case_id']}/{selected['window_id']} codes={codes}",
            file=sys.stderr,
        )
        return 1

    print(
        "automated musical fitness passed: "
        f"{selected['case_id']}/{selected['window_id']} score={selected['score']:.3f}"
    )
    return 0


def collect_candidates(paths: list[Path]) -> list[Candidate]:
    candidates: list[Candidate] = []
    for path in paths:
        for manifest_path in resolve_manifest_paths(path):
            manifest = read_manifest(manifest_path)
            case_id, window_id = infer_candidate_ids(manifest_path, manifest)
            metrics = extract_metrics(manifest)
            score_breakdown = build_score_breakdown(metrics)
            failures = tuple(
                code
                for section in score_breakdown.values()
                for code in section["failure_codes"]
            )
            candidates.append(
                Candidate(
                    case_id=case_id,
                    window_id=window_id,
                    manifest_path=manifest_path,
                    metrics=metrics,
                    score_breakdown=score_breakdown,
                    failure_codes=dedupe(failures),
                )
            )
    return candidates


def resolve_manifest_paths(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    if not path.is_dir():
        raise ValueError(f"path does not exist: {path}")
    packed = sorted((path / "packs").glob("*/*/manifest.json"))
    if packed:
        return packed
    manifest = path / "manifest.json"
    if manifest.is_file():
        return [manifest]
    return sorted(path.glob("*/manifest.json"))


def read_manifest(path: Path) -> dict[str, Any]:
    manifest = json.loads(path.read_text())
    if not isinstance(manifest, dict):
        raise ValueError(f"{path}: manifest must be a JSON object")
    if "metrics" not in manifest or not isinstance(manifest["metrics"], dict):
        raise ValueError(f"{path}: manifest must contain a metrics object")
    return manifest


def infer_candidate_ids(manifest_path: Path, manifest: dict[str, Any]) -> tuple[str, str]:
    case_id = str(
        manifest.get("case_id")
        or manifest.get("source")
        or manifest.get("pack_id")
        or manifest_path.parent.parent.name
    )
    window_id = str(manifest.get("window_id") or manifest_path.parent.name)
    return case_id, window_id


def extract_metrics(manifest: dict[str, Any]) -> dict[str, Any]:
    source_timing = object_or_empty(manifest.get("source_timing"))
    raw = manifest["metrics"]
    full_mix = object_or_empty(raw.get("full_grid_mix") or raw.get("full_mix"))
    full_signal = object_or_empty(full_mix.get("signal"))
    full_low = object_or_empty(full_mix.get("low_band"))
    mix_balance = object_or_empty(raw.get("mix_balance"))
    movement = object_or_empty(raw.get("all_lane_mix_movement"))
    w30_variation = object_or_empty(raw.get("w30_source_trigger_variation"))
    w30_slice = object_or_empty(raw.get("w30_source_slice_choice"))
    w30_accent = object_or_empty(raw.get("w30_source_accent_dynamics"))
    tr909_pressure = object_or_empty(raw.get("tr909_kick_pressure"))
    tr909_accent = object_or_empty(raw.get("tr909_source_accent_dynamics"))
    mc202_pressure = object_or_empty(raw.get("mc202_bass_pressure"))
    mc202_contour = object_or_empty(raw.get("mc202_source_contour"))
    identity = object_or_empty(raw.get("identity_collapse") or raw.get("fallback_collapse"))
    source_relation = object_or_empty(raw.get("source_relation"))
    grid_alignment = object_or_empty(raw.get("mc202_source_grid_alignment"))

    role_metrics = role_metric_values(raw)
    full_rms = first_number(
        full_signal.get("rms"),
        metric_value(raw, "full_grid_mix", "signal", "rms"),
        metric_value(raw, "full_grid_mix", "rms"),
        max(role_metrics.values()) if role_metrics else None,
    )
    low_band_rms = first_number(
        full_low.get("rms"),
        metric_value(raw, "full_grid_mix", "low_band", "rms"),
        metric_value(raw, "full_grid_mix", "low_band_rms"),
        spectral_ratio(raw, "full_grid_mix", "low_band_energy_ratio") * full_rms
        if spectral_ratio(raw, "full_grid_mix", "low_band_energy_ratio") is not None
        and full_rms is not None
        else None,
    )

    source_anchor_count = int(
        first_number(
            nested_number(source_timing, "anchor_evidence", "primary_anchor_count"),
            source_relation.get("anchor_count"),
            0,
        )
    )
    source_contour_delta = first_number(
        mc202_contour.get("source_contour_delta_rms"),
        source_relation.get("source_contour_delta_rms"),
        0.0,
    )

    return {
        "full_rms": full_rms,
        "low_band_rms": low_band_rms,
        "peak_abs": first_number(
            full_signal.get("peak_abs"),
            full_signal.get("peak"),
            metric_value(raw, "full_grid_mix", "signal", "peak_abs"),
        ),
        "event_density_per_bar": first_number(full_signal.get("event_density_per_bar")),
        "transient_score": first_number(full_signal.get("transient_score")),
        "support_generated_to_source_rms_ratio": first_number(
            mix_balance.get("support_generated_to_source_rms_ratio")
        ),
        "source_first_generated_to_source_rms_ratio": first_number(
            mix_balance.get("source_first_generated_to_source_rms_ratio")
        ),
        "identity_correlation": first_number(
            identity.get("correlation"),
            movement.get("source_first_to_support_correlation"),
        ),
        "normalized_rms_delta": first_number(
            identity.get("normalized_rms_delta"),
            movement.get("source_first_to_support_rms_delta"),
        ),
        "fallback_used": bool(identity.get("fallback_used", False)),
        "response_signature": string_or_none(
            identity.get("response_signature"),
            identity.get("full_mix_sha256"),
            manifest.get("response_signature"),
        ),
        "source_anchor_count": source_anchor_count,
        "source_contour_delta_rms": source_contour_delta,
        "source_contour_applied": bool(
            mc202_contour.get("applied", source_relation.get("applied", False))
        ),
        "source_contour_origin": str(
            mc202_contour.get(
                "pattern_origin",
                source_relation.get("origin", "unknown"),
            )
        ),
        "bar_similarity": first_number(
            nested_number(raw, "bar_variation", "full_grid_mix", "bar_similarity"),
            mc202_pressure.get("bar_similarity"),
        ),
        "w30_trigger_variation_applied": bool(w30_variation.get("applied", False)),
        "w30_distinct_bar_pattern_count": int(
            first_number(w30_variation.get("distinct_bar_pattern_count"), 0)
        ),
        "w30_slice_choice_applied": bool(w30_slice.get("applied", False)),
        "w30_unique_slice_offset_count": int(
            first_number(w30_slice.get("unique_source_offset_count"), 0)
        ),
        "w30_accent_dynamics_applied": bool(w30_accent.get("applied", False)),
        "w30_velocity_span": first_number(w30_accent.get("velocity_span"), 0.0),
        "lane_contribution_min": lane_contribution_min(movement, role_metrics),
        "generated_to_w30_contribution_ratio": first_number(
            movement.get("generated_to_w30_contribution_ratio")
        ),
        "tr909_kick_pressure_applied": bool(tr909_pressure.get("applied", False)),
        "tr909_low_band_ratio": first_number(tr909_pressure.get("low_band_rms_ratio")),
        "tr909_pattern_origin": str(tr909_pressure.get("pattern_origin", "unknown")),
        "tr909_source_evidence_role": str(
            tr909_pressure.get("source_evidence_role", "unknown")
        ),
        "tr909_source_profile_reason": str(
            tr909_pressure.get("source_profile_reason", "unknown")
        ),
        "tr909_anchor_count": int(first_number(tr909_pressure.get("anchor_count"), 0)),
        "tr909_accent_pattern_origin": str(tr909_accent.get("pattern_origin", "unknown")),
        "tr909_accent_dynamics_applied": bool(tr909_accent.get("applied", False)),
        "tr909_accent_anchor_count": int(first_number(tr909_accent.get("anchor_count"), 0)),
        "tr909_accent_distinct_accent_count": int(
            first_number(tr909_accent.get("distinct_accent_count"), 0)
        ),
        "tr909_accent_span": first_number(tr909_accent.get("accent_span"), 0.0),
        "tr909_accent_min_required_span": first_number(
            tr909_accent.get("min_required_accent_span"),
            MIN_TR909_ACCENT_SPAN,
        ),
        "mc202_bass_pressure_applied": bool(mc202_pressure.get("applied", False)),
        "mc202_bass_rms": first_number(mc202_pressure.get("signal_rms")),
        "source_grid_hit_ratio": first_number(grid_alignment.get("hit_ratio")),
        "source_grid_peak_offset_ms": first_number(grid_alignment.get("max_peak_offset_ms")),
    }


def build_score_breakdown(metrics: dict[str, Any]) -> dict[str, Any]:
    sections = {
        "technical_sanity": technical_sanity(metrics),
        "anti_collapse": anti_collapse(metrics),
        "source_relation": source_relation(metrics),
        "variation_movement": variation_movement(metrics),
        "lane_balance": lane_balance(metrics),
        "low_end_transients": low_end_transients(metrics),
        "grid_alignment": grid_alignment(metrics),
    }
    return sections


def technical_sanity(metrics: dict[str, Any]) -> dict[str, Any]:
    failures = []
    full_rms = metrics["full_rms"]
    peak_abs = metrics["peak_abs"]
    if full_rms is None:
        failures.append("technical_missing_full_mix_rms")
    elif full_rms < MIN_FULL_RMS:
        failures.append("technical_near_silence")
    if peak_abs is not None and peak_abs >= MAX_PEAK_ABS:
        failures.append("technical_clipping_risk")
    score = score_if(not failures, full_rms or 0.0, MIN_FULL_RMS * 2.0)
    return section(score, failures, {"full_rms": full_rms, "peak_abs": peak_abs})


def anti_collapse(metrics: dict[str, Any]) -> dict[str, Any]:
    failures = []
    support_ratio = metrics["support_generated_to_source_rms_ratio"]
    source_first_ratio = metrics["source_first_generated_to_source_rms_ratio"]
    identity_correlation = metrics["identity_correlation"]
    rms_delta = metrics["normalized_rms_delta"]
    if metrics["fallback_used"]:
        failures.append("fallback_collapse")
    if support_ratio is not None and not (
        MIN_SUPPORT_GENERATED_TO_SOURCE_RATIO
        <= support_ratio
        <= MAX_SUPPORT_GENERATED_TO_SOURCE_RATIO
    ):
        failures.append("generated_support_balance_out_of_range")
    if source_first_ratio is not None and (
        source_first_ratio > MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RATIO
    ):
        failures.append("source_first_generated_support_masks_source")
    if identity_correlation is not None and identity_correlation >= MAX_IDENTITY_CORRELATION:
        failures.append("identity_correlation_too_high")
    if rms_delta is not None and rms_delta < MIN_NORMALIZED_RMS_DELTA:
        failures.append("identity_rms_delta_too_low")
    score = 1.0 - min(len(failures) * 0.28, 1.0)
    return section(
        score,
        failures,
        {
            "support_generated_to_source_rms_ratio": support_ratio,
            "source_first_generated_to_source_rms_ratio": source_first_ratio,
            "identity_correlation": identity_correlation,
            "normalized_rms_delta": rms_delta,
            "fallback_used": metrics["fallback_used"],
            "response_signature": metrics["response_signature"],
        },
    )


def source_relation(metrics: dict[str, Any]) -> dict[str, Any]:
    failures = []
    if metrics["source_anchor_count"] < MIN_SOURCE_ANCHOR_COUNT:
        failures.append("source_relation_missing_anchor_evidence")
    if not metrics["source_contour_applied"]:
        failures.append("source_relation_missing_source_contour_evidence")
    elif metrics["source_contour_origin"] != "source_derived_contour":
        failures.append("source_relation_not_source_derived")
    elif metrics["source_contour_delta_rms"] < MIN_SOURCE_CONTOUR_DELTA_RMS:
        failures.append("source_relation_delta_too_low")
    score = 1.0 - min(len(failures) * 0.34, 1.0)
    return section(
        score,
        failures,
        {
            "source_anchor_count": metrics["source_anchor_count"],
            "source_contour_applied": metrics["source_contour_applied"],
            "source_contour_origin": metrics["source_contour_origin"],
            "source_contour_delta_rms": metrics["source_contour_delta_rms"],
        },
    )


def variation_movement(metrics: dict[str, Any]) -> dict[str, Any]:
    failures = []
    bar_similarity = metrics["bar_similarity"]
    event_density = metrics["event_density_per_bar"]
    if bar_similarity is not None and bar_similarity > MAX_BAR_SIMILARITY:
        failures.append("movement_bar_similarity_too_static")
    if event_density is not None and not (
        MIN_EVENT_DENSITY_PER_BAR <= event_density <= MAX_EVENT_DENSITY_PER_BAR
    ):
        failures.append("movement_event_density_out_of_range")
    if not metrics["w30_trigger_variation_applied"]:
        failures.append("movement_w30_trigger_variation_missing")
    if metrics["w30_distinct_bar_pattern_count"] < MIN_W30_DISTINCT_BAR_PATTERNS:
        failures.append("movement_w30_bar_patterns_too_static")
    if not metrics["w30_slice_choice_applied"]:
        failures.append("movement_w30_slice_choice_missing")
    if metrics["w30_unique_slice_offset_count"] < MIN_W30_UNIQUE_SLICE_OFFSETS:
        failures.append("movement_w30_slice_offsets_too_static")
    if not metrics["w30_accent_dynamics_applied"]:
        failures.append("movement_w30_accent_dynamics_missing")
    if metrics["w30_velocity_span"] < MIN_W30_VELOCITY_SPAN:
        failures.append("movement_w30_velocity_span_too_flat")
    score = 1.0 - min(len(failures) * 0.16, 1.0)
    return section(
        score,
        failures,
        {
            "bar_similarity": bar_similarity,
            "event_density_per_bar": event_density,
            "w30_trigger_variation_applied": metrics["w30_trigger_variation_applied"],
            "w30_distinct_bar_pattern_count": metrics["w30_distinct_bar_pattern_count"],
            "w30_slice_choice_applied": metrics["w30_slice_choice_applied"],
            "w30_unique_slice_offset_count": metrics["w30_unique_slice_offset_count"],
            "w30_accent_dynamics_applied": metrics["w30_accent_dynamics_applied"],
            "w30_velocity_span": metrics["w30_velocity_span"],
        },
    )


def lane_balance(metrics: dict[str, Any]) -> dict[str, Any]:
    failures = []
    lane_min = metrics["lane_contribution_min"]
    generated_to_w30 = metrics["generated_to_w30_contribution_ratio"]
    if lane_min is not None and lane_min < MIN_LANE_CONTRIBUTION_RATIO:
        failures.append("lane_balance_placeholder_lane_dominates")
    if generated_to_w30 is not None and generated_to_w30 < MIN_GENERATED_TO_W30_CONTRIBUTION_RATIO:
        failures.append("lane_balance_generated_support_too_weak")
    score = 1.0 - min(len(failures) * 0.4, 1.0)
    return section(
        score,
        failures,
        {
            "lane_contribution_min": lane_min,
            "generated_to_w30_contribution_ratio": generated_to_w30,
        },
    )


def low_end_transients(metrics: dict[str, Any]) -> dict[str, Any]:
    failures = []
    low_band_rms = metrics["low_band_rms"]
    transient_score = metrics["transient_score"]
    tr909_ratio = metrics["tr909_low_band_ratio"]
    mc202_rms = metrics["mc202_bass_rms"]
    if low_band_rms is None:
        failures.append("low_end_missing_low_band_rms")
    elif low_band_rms < MIN_LOW_BAND_RMS:
        failures.append("low_end_too_weak")
    if transient_score is not None and transient_score < MIN_TRANSIENT_SCORE:
        failures.append("transients_too_weak")
    if not metrics["tr909_kick_pressure_applied"]:
        failures.append("low_end_tr909_kick_pressure_missing")
    else:
        if metrics["tr909_pattern_origin"] != "source_derived":
            failures.append("low_end_tr909_kick_pressure_not_source_derived")
        if metrics["tr909_source_evidence_role"] != TR909_SOURCE_EVIDENCE_ROLE:
            failures.append("low_end_tr909_kick_pressure_missing_source_evidence")
        if not metrics["tr909_source_profile_reason"].startswith("source_"):
            failures.append("low_end_tr909_kick_pressure_source_reason_missing")
        if metrics["tr909_anchor_count"] < MIN_TR909_KICK_PRESSURE_ANCHORS:
            failures.append("low_end_tr909_kick_pressure_anchor_count_too_low")
        if tr909_ratio is None or tr909_ratio < MIN_TR909_LOW_BAND_RATIO:
            failures.append("low_end_tr909_kick_pressure_too_decorative")
    if metrics["tr909_accent_pattern_origin"] != "source_derived":
        failures.append("low_end_tr909_accent_dynamics_not_source_derived")
    if not metrics["tr909_accent_dynamics_applied"]:
        failures.append("low_end_tr909_accent_dynamics_missing")
    if metrics["tr909_accent_distinct_accent_count"] < MIN_TR909_ACCENT_DISTINCT_ACCENTS:
        failures.append("low_end_tr909_accent_count_too_low")
    if metrics["tr909_accent_anchor_count"] < MIN_TR909_KICK_PRESSURE_ANCHORS:
        failures.append("low_end_tr909_accent_anchor_count_too_low")
    if metrics["tr909_accent_span"] < max(
        MIN_TR909_ACCENT_SPAN,
        metrics["tr909_accent_min_required_span"],
    ):
        failures.append("low_end_tr909_accent_span_too_flat")
    if metrics["mc202_bass_pressure_applied"] and (
        mc202_rms is None or mc202_rms < MIN_MC202_BASS_RMS
    ):
        failures.append("low_end_mc202_bass_too_weak")
    score = 1.0 - min(len(failures) * 0.3, 1.0)
    return section(
        score,
        failures,
        {
            "low_band_rms": low_band_rms,
            "transient_score": transient_score,
            "tr909_kick_pressure_applied": metrics["tr909_kick_pressure_applied"],
            "tr909_low_band_ratio": tr909_ratio,
            "tr909_pattern_origin": metrics["tr909_pattern_origin"],
            "tr909_source_evidence_role": metrics["tr909_source_evidence_role"],
            "tr909_source_profile_reason": metrics["tr909_source_profile_reason"],
            "tr909_anchor_count": metrics["tr909_anchor_count"],
            "tr909_accent_pattern_origin": metrics["tr909_accent_pattern_origin"],
            "tr909_accent_dynamics_applied": metrics["tr909_accent_dynamics_applied"],
            "tr909_accent_anchor_count": metrics["tr909_accent_anchor_count"],
            "tr909_accent_distinct_accent_count": metrics[
                "tr909_accent_distinct_accent_count"
            ],
            "tr909_accent_span": metrics["tr909_accent_span"],
            "tr909_accent_min_required_span": metrics["tr909_accent_min_required_span"],
            "mc202_bass_pressure_applied": metrics["mc202_bass_pressure_applied"],
            "mc202_bass_rms": mc202_rms,
        },
    )


def grid_alignment(metrics: dict[str, Any]) -> dict[str, Any]:
    failures = []
    hit_ratio = metrics["source_grid_hit_ratio"]
    peak_offset = metrics["source_grid_peak_offset_ms"]
    if hit_ratio is not None and hit_ratio < MIN_SOURCE_GRID_HIT_RATIO:
        failures.append("grid_drift_alignment_too_weak")
    if peak_offset is not None and peak_offset > MAX_SOURCE_GRID_PEAK_OFFSET_MS:
        failures.append("grid_drift_peak_offset_too_high")
    score = 1.0 - min(len(failures) * 0.5, 1.0)
    return section(
        score,
        failures,
        {
            "source_grid_hit_ratio": hit_ratio,
            "source_grid_peak_offset_ms": peak_offset,
        },
    )


def build_report(candidates: list[Candidate]) -> dict[str, Any]:
    passing = [candidate for candidate in candidates if candidate.result == "pass"]
    corpus_section = cross_source_diversity(candidates)
    corpus_failure_codes = tuple(corpus_section["failure_codes"])
    selected = max(passing, key=lambda candidate: candidate.score) if passing else max(
        candidates,
        key=lambda candidate: candidate.score,
    )
    failure_codes = dedupe((*selected.failure_codes, *corpus_failure_codes))
    result = "pass" if passing and not corpus_failure_codes else "fail"
    score_breakdown = {
        **selected.score_breakdown,
        "cross_source_diversity": corpus_section,
    }
    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "technical_status": "pass"
        if not any(code.startswith("technical_") for code in failure_codes)
        else "fail",
        "automated_musical_fitness_status": result,
        "result": result,
        "selected_candidate": candidate_record(selected),
        "failure_codes": list(failure_codes),
        "score_breakdown": score_breakdown,
        "human_verdict": "unverified",
        "candidate_count": len(candidates),
        "passing_candidate_count": len(passing),
        "candidates": [candidate_record(candidate) for candidate in candidates],
        "thresholds": {
            "min_full_rms": MIN_FULL_RMS,
            "min_low_band_rms": MIN_LOW_BAND_RMS,
            "max_peak_abs": MAX_PEAK_ABS,
            "min_support_generated_to_source_rms_ratio": MIN_SUPPORT_GENERATED_TO_SOURCE_RATIO,
            "max_support_generated_to_source_rms_ratio": MAX_SUPPORT_GENERATED_TO_SOURCE_RATIO,
            "max_source_first_generated_to_source_rms_ratio": MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RATIO,
            "max_identity_correlation": MAX_IDENTITY_CORRELATION,
            "min_normalized_rms_delta": MIN_NORMALIZED_RMS_DELTA,
            "min_source_anchor_count": MIN_SOURCE_ANCHOR_COUNT,
            "min_source_contour_delta_rms": MIN_SOURCE_CONTOUR_DELTA_RMS,
            "max_bar_similarity": MAX_BAR_SIMILARITY,
            "min_event_density_per_bar": MIN_EVENT_DENSITY_PER_BAR,
            "max_event_density_per_bar": MAX_EVENT_DENSITY_PER_BAR,
            "min_lane_contribution_ratio": MIN_LANE_CONTRIBUTION_RATIO,
            "min_generated_to_w30_contribution_ratio": MIN_GENERATED_TO_W30_CONTRIBUTION_RATIO,
            "min_tr909_low_band_ratio": MIN_TR909_LOW_BAND_RATIO,
            "tr909_source_evidence_role": TR909_SOURCE_EVIDENCE_ROLE,
            "min_tr909_kick_pressure_anchors": MIN_TR909_KICK_PRESSURE_ANCHORS,
            "min_tr909_accent_distinct_accent_count": MIN_TR909_ACCENT_DISTINCT_ACCENTS,
            "min_tr909_accent_span": MIN_TR909_ACCENT_SPAN,
            "min_low_end_transient_score": MIN_TRANSIENT_SCORE,
            "min_source_grid_hit_ratio": MIN_SOURCE_GRID_HIT_RATIO,
            "max_source_grid_peak_offset_ms": MAX_SOURCE_GRID_PEAK_OFFSET_MS,
        },
    }


def cross_source_diversity(candidates: list[Candidate]) -> dict[str, Any]:
    failures = []
    signatures: dict[str, set[str]] = {}
    for candidate in candidates:
        signature = candidate.metrics.get("response_signature")
        if not isinstance(signature, str) or not signature:
            continue
        signatures.setdefault(signature, set()).add(candidate.case_id)
    repeated = {
        signature: sorted(case_ids)
        for signature, case_ids in signatures.items()
        if len(case_ids) > 1
    }
    if repeated:
        failures.append("cross_source_identical_response")
    return section(
        0.0 if failures else 1.0,
        failures,
        {"repeated_response_signatures": repeated},
    )


def candidate_record(candidate: Candidate) -> dict[str, Any]:
    return {
        "case_id": candidate.case_id,
        "window_id": candidate.window_id,
        "manifest": str(candidate.manifest_path),
        "result": candidate.result,
        "score": candidate.score,
        "failure_codes": list(candidate.failure_codes),
    }


def render_markdown(report: dict[str, Any]) -> str:
    selected = report["selected_candidate"]
    lines = [
        "# Automated Musical Fitness",
        "",
        f"- Schema: `{report['schema']}`",
        f"- Result: `{report['result']}`",
        f"- Technical status: `{report['technical_status']}`",
        f"- Automated musical fitness status: `{report['automated_musical_fitness_status']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Selected candidate: `{selected['case_id']}/{selected['window_id']}`",
        f"- Score: `{selected['score']}`",
        "",
        "## Failure Codes",
        "",
    ]
    if report["failure_codes"]:
        lines.extend(f"- `{code}`" for code in report["failure_codes"])
    else:
        lines.append("- none")
    lines.extend(["", "## Score Breakdown", ""])
    for name, section_data in report["score_breakdown"].items():
        codes = ", ".join(section_data["failure_codes"]) or "none"
        lines.append(f"- `{name}`: score `{section_data['score']}`, failures `{codes}`")
    return "\n".join(lines) + "\n"


def section(score: float, failure_codes: list[str], metrics: dict[str, Any]) -> dict[str, Any]:
    return {
        "status": "pass" if not failure_codes else "fail",
        "score": round(max(0.0, min(1.0, score)), 6),
        "failure_codes": failure_codes,
        "metrics": metrics,
    }


def score_if(ok: bool, value: float, strong_value: float) -> float:
    if not ok:
        return 0.0
    if strong_value <= 0.0:
        return 1.0
    return max(0.0, min(1.0, value / strong_value))


def role_metric_values(raw: dict[str, Any]) -> dict[str, float]:
    values: dict[str, float] = {}
    for role, metric in raw.items():
        if not isinstance(metric, dict) or role in {"spectral_energy", "bar_variation"}:
            continue
        value = first_number(metric_value(raw, role, "signal", "rms"), metric_value(raw, role, "rms"))
        if value is not None:
            values[role] = value
    return values


def lane_contribution_min(
    movement: dict[str, Any],
    role_metrics: dict[str, float],
) -> float | None:
    explicit = [
        first_number(movement.get("tr909_contribution_ratio")),
        first_number(movement.get("mc202_contribution_ratio")),
        first_number(movement.get("w30_contribution_ratio")),
    ]
    present_explicit = [value for value in explicit if value is not None]
    if present_explicit:
        return min(present_explicit)
    non_full = [
        value
        for role, value in role_metrics.items()
        if "full" not in role and "mix" not in role
    ]
    full = [
        value
        for role, value in role_metrics.items()
        if "full" in role or "mix" in role
    ]
    if len(non_full) < 2 or not full or max(full) <= 0.0:
        return None
    return min(non_full) / max(full)


def spectral_ratio(raw: dict[str, Any], role: str, key: str) -> float | None:
    return metric_value(raw, "spectral_energy", role, key)


def metric_value(raw: dict[str, Any], *keys: str) -> float | None:
    current: Any = raw
    for key in keys:
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    return first_number(current)


def nested_number(value: Any, *keys: str) -> float | None:
    current = value
    for key in keys:
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    return first_number(current)


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def first_number(*values: Any) -> float | None:
    for value in values:
        if isinstance(value, bool):
            continue
        if isinstance(value, (int, float)):
            return float(value)
    return None


def string_or_none(*values: Any) -> str | None:
    for value in values:
        if isinstance(value, str) and value:
            return value
    return None


def dedupe(values: tuple[str, ...] | list[str]) -> tuple[str, ...]:
    seen: set[str] = set()
    output: list[str] = []
    for value in values:
        if value not in seen:
            output.append(value)
            seen.add(value)
    return tuple(output)


if __name__ == "__main__":
    raise SystemExit(main())
