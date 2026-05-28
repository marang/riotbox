#!/usr/bin/env python3
"""Validate Riotbox observer/audio summary JSON v1.

This is intentionally narrower than a general JSON Schema validator: it checks
the repo's current stable summary contract without adding a runtime dependency.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.observer_audio_summary.v1"
SCHEMA_VERSION = 1
SOURCE_TIMING_BPM_MATCH_TOLERANCE = 1.0
EPSILON = 0.000001
SOURCE_TIMING_CUE_BY_POLICY = {
    "locked": "grid locked",
    "manual_confirm": "needs confirm",
    "cautious": "listen first",
    "fallback_grid": "fallback grid",
    "disabled": "not available",
    "unknown": "unknown",
}
SOURCE_TIMING_CUES = set(SOURCE_TIMING_CUE_BY_POLICY.values())
SOURCE_TIMING_ACTIONABILITY = {
    "grid can steer moves",
    "confirm grid first",
    "listen first",
    "timing unavailable",
    "using safe fallback grid",
    "unknown",
}
GROOVE_SUBDIVISIONS = {
    "eighth",
    "triplet",
    "sixteenth",
    "thirty_second",
}
GRID_BPM_SOURCES = {
    "unknown",
    "user_override",
    "source_timing",
    "static_default",
}
GRID_BPM_DECISION_REASONS = {
    "unknown",
    "user_override",
    "source_timing_ready",
    "source_timing_needs_review_manual_confirm",
    "source_timing_requires_manual_confirm",
    "source_timing_not_ready",
    "source_timing_missing_bpm",
    "source_timing_invalid_bpm",
}
STATIC_DEFAULT_GRID_BPM_REASONS = {
    "source_timing_requires_manual_confirm",
    "source_timing_not_ready",
    "source_timing_missing_bpm",
    "source_timing_invalid_bpm",
}
SOURCE_TIMING_GRID_USE = {
    "locked_grid",
    "short_loop_manual_confirm",
    "manual_confirm_only",
    "fallback_grid",
    "unavailable",
}
SOURCE_TIMING_GRID_USE_COMPATIBILITY = {
    "aligned",
    "compatible",
    "partial",
    "mismatch",
}
SOURCE_TIMING_READINESS = {"unavailable", "weak", "needs_review", "ready"}
SOURCE_TIMING_BEAT_STATUSES = {"unavailable", "weak", "stable", "ambiguous"}
SOURCE_TIMING_DOWNBEAT_STATUSES = {"unavailable", "weak", "stable", "ambiguous"}
SOURCE_TIMING_CONFIDENCE_RESULTS = {
    "degraded",
    "candidate_cautious",
    "candidate_ambiguous",
}
SOURCE_TIMING_DRIFT_STATUSES = {
    "unavailable",
    "not_enough_material",
    "stable",
    "high",
}
SOURCE_TIMING_PHRASE_STATUSES = {
    "unavailable",
    "not_enough_material",
    "ambiguous_downbeat",
    "high_drift",
    "stable",
}
LANE_RECIPE_CASE_RESULTS = {"pass", "fail"}
MC202_PHRASE_GRID_MIN_HIT_RATIO = 0.95


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate_observer_audio_summary_json.py <summary.json>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    try:
        summary = json.loads(path.read_text())
        validate_summary(summary)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid observer/audio summary JSON: {error}", file=sys.stderr)
        return 1

    print(f"valid {SCHEMA} summary: {path}")
    return 0


def validate_summary(summary: Any) -> None:
    require_object(summary, "summary")
    require_equal(summary, "schema", SCHEMA)
    require_equal(summary, "schema_version", SCHEMA_VERSION)
    require_bool(summary, "needs_human_listening")

    control_path = require_object_field(summary, "control_path")
    require_bool(control_path, "present")
    require_string(control_path, "observer_schema")
    require_string(control_path, "launch_mode")
    require_string(control_path, "audio_runtime_status")
    require_string(control_path, "first_commit")
    require_int(control_path, "commit_count")
    require_string_list(control_path, "commit_boundaries")
    require_optional_observer_source_timing(control_path)
    require_string_list(control_path, "key_outcomes")

    output_path = require_object_field(summary, "output_path")
    require_bool(output_path, "present")
    require_string_list(output_path, "issues")
    require_string(output_path, "pack_id")
    require_string(output_path, "manifest_result")
    require_int(output_path, "artifact_count")
    require_one_of(output_path, "grid_bpm_source", GRID_BPM_SOURCES)
    require_one_of(output_path, "grid_bpm_decision_reason", GRID_BPM_DECISION_REASONS)
    require_optional_number(output_path, "source_timing_bpm_delta")
    require_optional_source_timing(output_path)
    require_output_grid_bpm_decision(output_path)
    require_source_timing_bpm_delta_consistency(output_path)
    require_optional_source_timing_alignment(output_path)
    require_optional_source_timing_anchor_alignment(output_path)
    require_optional_source_timing_groove_alignment(output_path)
    require_lane_recipe_cases(output_path)

    metrics = require_object_field(output_path, "metrics")
    require_optional_non_negative_number(metrics, "full_mix_rms")
    require_optional_non_negative_number(metrics, "full_mix_low_band_rms")
    require_optional_non_negative_number(metrics, "mc202_question_answer_delta_rms")
    require_optional_source_grid_output_drift(metrics)
    require_optional_source_grid_alignment(metrics, "tr909_source_grid_alignment")
    require_optional_source_grid_alignment(metrics, "mc202_source_grid_alignment")
    require_optional_source_grid_alignment(metrics, "w30_source_grid_alignment")
    require_optional_w30_source_loop_closure(metrics)
    require_optional_non_negative_number(metrics, "w30_candidate_rms")
    require_optional_ratio(metrics, "w30_candidate_active_sample_ratio")
    require_optional_non_negative_number(metrics, "w30_rms_delta")


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_object_field(parent: dict[str, Any], field: str) -> dict[str, Any]:
    return require_object(parent.get(field), field)


def require_equal(parent: dict[str, Any], field: str, expected: Any) -> None:
    actual = parent.get(field)
    if actual != expected:
        raise ValueError(f"{field} must be {expected!r}, got {actual!r}")


def require_bool(parent: dict[str, Any], field: str) -> bool:
    value = parent.get(field)
    if not isinstance(value, bool):
        raise TypeError(f"{field} must be a boolean")
    return value


def require_optional_bool(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a boolean or null")
    value = parent.get(field)
    if value is not None and not isinstance(value, bool):
        raise TypeError(f"{field} must be a boolean or null")


def require_string(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")
    return value


def require_string_list(parent: dict[str, Any], field: str) -> list[str]:
    value = parent.get(field)
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise TypeError(f"{field} must be an array of strings")
    return value


def require_array(parent: dict[str, Any], field: str) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{field} must be an array")
    return value


def require_int(parent: dict[str, Any], field: str) -> None:
    require_int_value(parent, field)


def require_int_value(parent: dict[str, Any], field: str) -> int:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer")
    return value


def require_optional_number(parent: dict[str, Any], field: str) -> None:
    require_optional_number_value(parent, field)


def require_optional_number_value(parent: dict[str, Any], field: str) -> float | int | None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a number or null")
    value = parent.get(field)
    if value is not None and (not isinstance(value, (int, float)) or isinstance(value, bool)):
        raise TypeError(f"{field} must be a number or null")
    return value


def require_optional_non_negative_number(parent: dict[str, Any], field: str) -> None:
    value = require_optional_number_value(parent, field)
    if value is not None and value < 0.0:
        raise ValueError(f"{field} must be non-negative")


def require_optional_ratio(parent: dict[str, Any], field: str) -> None:
    value = require_optional_number_value(parent, field)
    if value is not None and (value < 0.0 or value > 1.0):
        raise ValueError(f"{field} must be between 0 and 1")


def require_optional_source_grid_output_drift(parent: dict[str, Any]) -> None:
    require_optional_source_grid_alignment(parent, "source_grid_output_drift")


def require_optional_source_grid_alignment(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    drift = require_object(value, field)
    hit_ratio = require_number_value(drift, "hit_ratio")
    if hit_ratio < 0.0 or hit_ratio > 1.0:
        raise ValueError(f"{field}.hit_ratio must be between 0 and 1")
    max_peak_offset_ms = require_number_value(drift, "max_peak_offset_ms")
    if max_peak_offset_ms < 0.0:
        raise ValueError(f"{field}.max_peak_offset_ms must be non-negative")
    max_allowed_peak_offset_ms = require_number_value(
        drift, "max_allowed_peak_offset_ms"
    )
    if max_allowed_peak_offset_ms < 0.0:
        raise ValueError(f"{field}.max_allowed_peak_offset_ms must be non-negative")


def require_optional_w30_source_loop_closure(parent: dict[str, Any]) -> None:
    field = "w30_source_loop_closure"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    closure = require_object(value, field)
    require_bool(closure, "passed")
    require_non_negative_number(closure, "preview_rms", field)
    require_non_negative_number(closure, "edge_delta_abs", field)
    require_non_negative_number(closure, "max_allowed_edge_delta_abs", field)
    require_non_negative_number(closure, "edge_abs_max", field)
    require_non_negative_number(closure, "max_allowed_edge_abs", field)
    require_bool(closure, "source_contains_selection")


def require_optional_source_timing(parent: dict[str, Any]) -> None:
    field = "source_timing"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    timing = require_object(value, field)
    require_string(timing, "source_id")
    cue = require_one_of(timing, "cue", SOURCE_TIMING_CUES)
    actionability = require_one_of(timing, "actionability", SOURCE_TIMING_ACTIONABILITY)
    require_string(timing, "policy_profile")
    grid_use = require_nullable_one_of(timing, "grid_use", SOURCE_TIMING_GRID_USE)
    readiness = require_one_of(timing, "readiness", SOURCE_TIMING_READINESS)
    requires_manual_confirm = require_bool(timing, "requires_manual_confirm")
    require_source_timing_readiness_cue_match(cue, readiness, requires_manual_confirm)
    require_source_timing_readiness_actionability_match(
        actionability, readiness, requires_manual_confirm
    )
    require_optional_number(timing, "primary_bpm")
    require_optional_bool(timing, "bpm_agrees_with_grid")
    require_one_of(timing, "beat_status", SOURCE_TIMING_BEAT_STATUSES)
    require_one_of(timing, "downbeat_status", SOURCE_TIMING_DOWNBEAT_STATUSES)
    require_optional_int(timing, "primary_downbeat_offset_beats")
    require_one_of(timing, "confidence_result", SOURCE_TIMING_CONFIDENCE_RESULTS)
    require_one_of(timing, "drift_status", SOURCE_TIMING_DRIFT_STATUSES)
    require_one_of(timing, "phrase_status", SOURCE_TIMING_PHRASE_STATUSES)
    require_non_negative_int(timing, "primary_phrase_count")
    require_non_negative_int(timing, "primary_phrase_bar_count")
    require_source_timing_phrase_evidence_match(timing)
    require_non_negative_int(timing, "alternate_evidence_count")
    require_optional_source_timing_anchor_evidence(timing, "anchor_evidence")
    require_optional_source_timing_groove_evidence(timing, "groove_evidence")
    require_string_list(timing, "warning_codes")
    if grid_use is not None:
        require_source_timing_grid_use_match(timing, grid_use)


def require_optional_source_timing_alignment(parent: dict[str, Any]) -> None:
    field = "source_timing_alignment"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    alignment = require_object(value, field)
    status = require_one_of(alignment, "status", {"aligned", "partial", "mismatch"})
    bpm_delta = require_optional_number_value(alignment, "bpm_delta")
    bpm_tolerance = require_number_value(alignment, "bpm_tolerance")
    observer_grid_use = require_one_of(
        alignment, "observer_grid_use", SOURCE_TIMING_GRID_USE
    )
    manifest_grid_use = require_optional_one_of(
        alignment, "manifest_grid_use", SOURCE_TIMING_GRID_USE
    )
    grid_use_compatibility = require_one_of(
        alignment, "grid_use_compatibility", SOURCE_TIMING_GRID_USE_COMPATIBILITY
    )
    observer_downbeat_offset = require_optional_int_value(
        alignment, "observer_downbeat_offset_beats"
    )
    manifest_downbeat_offset = require_optional_int_value(
        alignment, "manifest_downbeat_offset_beats"
    )
    downbeat_offset_compatibility = require_one_of(
        alignment, "downbeat_offset_compatibility", {"aligned", "partial", "mismatch"}
    )
    downbeat_ambiguity_compatibility = require_one_of(
        alignment, "downbeat_ambiguity_compatibility", {"aligned", "partial", "mismatch"}
    )
    require_string_list(alignment, "warning_overlap")
    issues = require_string_list(alignment, "issues")
    require_source_timing_grid_use_compatibility_match(
        observer_grid_use, manifest_grid_use, grid_use_compatibility, issues
    )
    require_source_timing_downbeat_offset_compatibility_match(
        observer_downbeat_offset,
        manifest_downbeat_offset,
        downbeat_offset_compatibility,
        issues,
    )
    require_source_timing_downbeat_ambiguity_compatibility_match(
        downbeat_ambiguity_compatibility,
        issues,
    )
    require_alignment_status_issues_consistency(
        field,
        status,
        issues,
        "source_timing_alignment.",
    )
    require_source_timing_bpm_delta_alignment_match(
        bpm_delta,
        bpm_tolerance,
        issues,
    )


def require_output_grid_bpm_decision(output_path: dict[str, Any]) -> None:
    pack_id = output_path.get("pack_id")
    source = output_path["grid_bpm_source"]
    reason = output_path["grid_bpm_decision_reason"]
    source_timing = output_path.get("source_timing")

    if not output_path.get("present"):
        if source != "unknown" or reason != "unknown":
            raise ValueError("absent output_path requires unknown grid BPM source and reason")
        return

    if source == "unknown" or reason == "unknown":
        if source == "unknown" and reason == "unknown" and pack_id != "feral-grid-demo":
            return
        raise ValueError(
            "feral-grid output_path requires known grid BPM source and reason"
        )
    if source == "user_override" and reason != "user_override":
        raise ValueError("user_override grid BPM source requires user_override decision reason")
    if source == "static_default" and reason not in STATIC_DEFAULT_GRID_BPM_REASONS:
        raise ValueError("static_default grid BPM source requires a fallback decision reason")
    if source == "source_timing":
        if reason not in {"source_timing_ready", "source_timing_needs_review_manual_confirm"}:
            raise ValueError(
                "source_timing grid BPM source requires a source-timing decision reason"
            )
        if not isinstance(source_timing, dict):
            raise ValueError("source_timing grid BPM source requires source_timing evidence")

    if not isinstance(source_timing, dict):
        return
    if reason == "source_timing_ready":
        if source_timing.get("readiness") != "ready":
            raise ValueError("source_timing_ready requires source_timing.readiness == ready")
        if source_timing.get("requires_manual_confirm") is not False:
            raise ValueError(
                "source_timing_ready requires source_timing.requires_manual_confirm == false"
            )
    if reason == "source_timing_requires_manual_confirm":
        if source_timing.get("requires_manual_confirm") is not True:
            raise ValueError(
                "source_timing_requires_manual_confirm requires manual confirmation evidence"
            )
    if reason == "source_timing_needs_review_manual_confirm":
        if source_timing.get("readiness") != "needs_review":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires source_timing.readiness == needs_review"
            )
        if source_timing.get("requires_manual_confirm") is not True:
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires manual confirmation evidence"
            )
        if source_timing.get("beat_status") != "stable":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires stable beat evidence"
            )
        if source_timing.get("downbeat_status") != "stable":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires stable downbeat evidence"
            )
        if source_timing.get("confidence_result") != "candidate_cautious":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires candidate_cautious confidence"
            )
        if source_timing.get("alternate_evidence_count") != 0:
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires no alternate evidence"
            )


def require_source_timing_bpm_delta_consistency(output_path: dict[str, Any]) -> None:
    source = output_path["grid_bpm_source"]
    reason = output_path["grid_bpm_decision_reason"]
    delta = output_path.get("source_timing_bpm_delta")
    source_timing = output_path.get("source_timing")

    if not output_path.get("present") or source == "unknown":
        if delta is not None:
            raise ValueError("unknown or absent grid BPM evidence requires null source_timing_bpm_delta")
        return

    if source == "source_timing":
        if not isinstance(delta, (int, float)) or isinstance(delta, bool):
            raise TypeError("source_timing grid BPM source requires numeric source_timing_bpm_delta")
        if abs(float(delta)) > EPSILON:
            raise ValueError("source_timing grid BPM source requires source_timing_bpm_delta == 0")
        require_bpm_agreement(source_timing, True, "source_timing grid BPM source")
        return

    if reason in {"source_timing_missing_bpm", "source_timing_invalid_bpm"}:
        if delta is not None:
            raise ValueError(f"{reason} requires null source_timing_bpm_delta")
        require_bpm_agreement(source_timing, None, reason)
        return

    if source == "user_override" and delta is None:
        require_bpm_agreement(source_timing, None, "user_override without usable source BPM")
        return

    if not isinstance(delta, (int, float)) or isinstance(delta, bool):
        raise TypeError(
            f"{source}/{reason} requires numeric source_timing_bpm_delta when source BPM is usable"
        )
    expected_agrees = float(delta) <= SOURCE_TIMING_BPM_MATCH_TOLERANCE
    require_bpm_agreement(source_timing, expected_agrees, f"{source}/{reason}")


def require_bpm_agreement(
    source_timing: Any, expected: bool | None, context: str
) -> None:
    if not isinstance(source_timing, dict):
        return
    actual = source_timing.get("bpm_agrees_with_grid")
    if actual is not expected:
        raise ValueError(
            f"{context} requires source_timing.bpm_agrees_with_grid == {expected!r}"
        )


def require_optional_source_timing_anchor_alignment(parent: dict[str, Any]) -> None:
    field = "source_timing_anchor_alignment"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    alignment = require_object(value, field)
    status = require_one_of(alignment, "status", {"aligned", "partial", "mismatch"})
    require_optional_source_timing_anchor_evidence(alignment, "observer")
    require_optional_source_timing_anchor_evidence(alignment, "manifest")
    issues = require_string_list(alignment, "issues")
    require_alignment_status_issues_consistency(
        field,
        status,
        issues,
        "source_timing_anchor_alignment.",
    )


def require_optional_source_timing_groove_alignment(parent: dict[str, Any]) -> None:
    field = "source_timing_groove_alignment"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    alignment = require_object(value, field)
    status = require_one_of(alignment, "status", {"aligned", "partial", "mismatch"})
    require_optional_source_timing_groove_evidence(alignment, "observer")
    require_optional_source_timing_groove_evidence(alignment, "manifest")
    issues = require_string_list(alignment, "issues")
    require_alignment_status_issues_consistency(
        field,
        status,
        issues,
        "source_timing_groove_alignment.",
    )


def require_alignment_status_issues_consistency(
    field: str, status: str, issues: list[str], issue_prefix: str
) -> None:
    if status == "mismatch":
        if not issues:
            raise ValueError(f"{field} mismatch must include an issue")
        for issue in issues:
            if not issue.startswith(issue_prefix):
                raise ValueError(f"{field} mismatch issue must start with {issue_prefix!r}")
        return

    if issues:
        raise ValueError(f"{field} non-mismatch status must not include issues")


def require_lane_recipe_cases(parent: dict[str, Any]) -> None:
    cases = require_array(parent, "lane_recipe_cases")
    for index, case in enumerate(cases):
        require_lane_recipe_case(case, f"lane_recipe_cases[{index}]")


def require_lane_recipe_case(value: Any, name: str) -> None:
    case = require_object(value, name)
    require_string(case, "id")
    require_one_of(case, "result", LANE_RECIPE_CASE_RESULTS)
    require_optional_non_negative_number(case, "candidate_rms")
    require_optional_non_negative_number(case, "signal_delta_rms")
    require_optional_non_negative_number(case, "min_signal_delta_rms")
    require_bool(case, "mc202_phrase_grid_malformed")
    require_bool(case, "mc202_source_phrase_slot_malformed")
    require_optional_mc202_phrase_grid(case, "mc202_phrase_grid")
    require_optional_mc202_source_phrase_slot(case, "mc202_source_phrase_slot")


def require_optional_mc202_phrase_grid(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    grid = require_object(value, field)
    hit_ratio = require_number_value(grid, "hit_ratio")
    if hit_ratio < 0.0 or hit_ratio > 1.0:
        raise ValueError(f"{field}.hit_ratio must be between 0 and 1")
    starts_on_phrase_boundary = require_bool(grid, "starts_on_phrase_boundary")
    candidate_onset_count = require_non_negative_int(grid, "candidate_onset_count")
    grid_aligned_onset_count = require_non_negative_int(grid, "grid_aligned_onset_count")
    if grid_aligned_onset_count > candidate_onset_count:
        raise ValueError(f"{field}.grid_aligned_onset_count cannot exceed candidate_onset_count")
    require_mc202_phrase_grid_hit_ratio_derivation(
        field, hit_ratio, candidate_onset_count, grid_aligned_onset_count
    )
    max_onset_offset_ms = require_non_negative_number(grid, "max_onset_offset_ms", field)
    max_allowed_onset_offset_ms = require_non_negative_number(
        grid, "max_allowed_onset_offset_ms", field
    )
    passed = require_bool(grid, "passed")
    require_mc202_phrase_grid_pass_consistency(
        field,
        passed,
        starts_on_phrase_boundary,
        candidate_onset_count,
        hit_ratio,
        max_onset_offset_ms,
        max_allowed_onset_offset_ms,
    )


def require_mc202_phrase_grid_hit_ratio_derivation(
    field: str,
    hit_ratio: float | int,
    candidate_onset_count: int,
    grid_aligned_onset_count: int,
) -> None:
    expected = (
        0.0
        if candidate_onset_count == 0
        else grid_aligned_onset_count / candidate_onset_count
    )
    if abs(float(hit_ratio) - expected) <= EPSILON:
        return
    raise ValueError(
        f"{field}.hit_ratio must match grid_aligned_onset_count / candidate_onset_count"
    )


def require_mc202_phrase_grid_pass_consistency(
    field: str,
    passed: bool,
    starts_on_phrase_boundary: bool,
    candidate_onset_count: int,
    hit_ratio: float | int,
    max_onset_offset_ms: float | int,
    max_allowed_onset_offset_ms: float | int,
) -> None:
    expected = (
        starts_on_phrase_boundary
        and candidate_onset_count > 0
        and hit_ratio >= MC202_PHRASE_GRID_MIN_HIT_RATIO
        and max_onset_offset_ms <= max_allowed_onset_offset_ms
    )
    if passed == expected:
        return
    if passed:
        if not starts_on_phrase_boundary:
            raise ValueError(f"{field}.passed requires starts_on_phrase_boundary")
        if candidate_onset_count == 0:
            raise ValueError(f"{field}.passed requires candidate_onset_count > 0")
        if hit_ratio < MC202_PHRASE_GRID_MIN_HIT_RATIO:
            raise ValueError(
                f"{field}.passed requires hit_ratio >= {MC202_PHRASE_GRID_MIN_HIT_RATIO}"
            )
        raise ValueError(
            f"{field}.passed requires max_onset_offset_ms <= max_allowed_onset_offset_ms"
        )
    raise ValueError(f"{field}.passed=false contradicts passing phrase-grid evidence")


def require_optional_mc202_source_phrase_slot(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    slot = require_object(value, field)
    phrase_grid_available = require_bool(slot, "phrase_grid_available")
    phrase_index = require_optional_int_value(slot, "phrase_index")
    if phrase_index is not None and phrase_index < 0:
        raise ValueError(f"{field}.phrase_index must be non-negative")
    starts_on_source_phrase_boundary = require_bool(slot, "starts_on_source_phrase_boundary")
    passed = require_bool(slot, "passed")
    if passed:
        if not phrase_grid_available:
            raise ValueError(f"{field}.passed requires phrase_grid_available")
        if phrase_index is None:
            raise ValueError(f"{field}.passed requires phrase_index")
        if not starts_on_source_phrase_boundary:
            raise ValueError(f"{field}.passed requires starts_on_source_phrase_boundary")
    if not phrase_grid_available:
        if phrase_index is not None:
            raise ValueError(f"{field}.phrase_index requires phrase_grid_available")
        if starts_on_source_phrase_boundary:
            raise ValueError(
                f"{field}.starts_on_source_phrase_boundary requires phrase_grid_available"
            )
    if starts_on_source_phrase_boundary and phrase_index is None:
        raise ValueError(f"{field}.starts_on_source_phrase_boundary requires phrase_index")


def require_optional_observer_source_timing(parent: dict[str, Any]) -> None:
    field = "observer_source_timing"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    timing = require_object(value, field)
    require_string(timing, "source_id")
    cue = require_one_of(timing, "cue", SOURCE_TIMING_CUES)
    require_optional_number(timing, "bpm_estimate")
    require_number(timing, "bpm_confidence")
    require_one_of(timing, "quality", {"low", "medium", "high", "unknown"})
    degraded_policy = require_one_of(
        timing,
        "degraded_policy",
        set(SOURCE_TIMING_CUE_BY_POLICY),
    )
    require_source_timing_policy_cue_match(cue, degraded_policy)
    grid_use = require_one_of(timing, "grid_use", SOURCE_TIMING_GRID_USE)
    beat_status = require_one_of(
        timing, "beat_status", {"grid", "tempo_only", "unknown"}
    )
    beat_count = require_int_value(timing, "beat_count")
    downbeat_status = require_one_of(
        timing, "downbeat_status", {"ambiguous", "bar_locked", "unknown"}
    )
    require_optional_int(timing, "primary_downbeat_offset_beats")
    bar_count = require_int_value(timing, "bar_count")
    phrase_status = require_one_of(
        timing, "phrase_status", {"uncertain", "phrase_locked", "unknown"}
    )
    phrase_count = require_int_value(timing, "phrase_count")
    require_observer_source_timing_count_match(
        beat_status,
        beat_count,
        downbeat_status,
        bar_count,
        phrase_status,
        phrase_count,
    )
    require_optional_string(timing, "primary_hypothesis_id")
    require_int(timing, "hypothesis_count")
    require_optional_source_timing_anchor_evidence(timing, "anchor_evidence")
    require_string(timing, "primary_anchor_cue")
    require_optional_source_timing_groove_evidence(timing, "groove_evidence")
    require_optional_string(timing, "primary_warning_code")
    warning_codes = require_string_list(timing, "warning_codes")
    require_observer_source_timing_grid_use_match(
        grid_use,
        degraded_policy,
        timing.get("bpm_estimate"),
        beat_count,
        bar_count,
        phrase_count,
        warning_codes,
    )


def require_optional_source_timing_anchor_evidence(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    anchor_evidence = require_object(value, field)
    total = require_non_negative_int(anchor_evidence, "primary_anchor_count")
    kick = require_non_negative_int(anchor_evidence, "primary_kick_anchor_count")
    backbeat = require_non_negative_int(anchor_evidence, "primary_backbeat_anchor_count")
    transient = require_non_negative_int(anchor_evidence, "primary_transient_anchor_count")
    if kick + backbeat + transient > total:
        raise ValueError(f"{field} typed anchor counts cannot exceed primary_anchor_count")


def require_optional_source_timing_groove_evidence(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    groove_evidence = require_object(value, field)
    total = require_non_negative_int(groove_evidence, "primary_groove_residual_count")
    max_abs = require_number_value(groove_evidence, "primary_max_abs_offset_ms")
    if max_abs < 0:
        raise ValueError(f"{field}.primary_max_abs_offset_ms must be non-negative")
    preview = require_array(groove_evidence, "primary_groove_preview")
    if len(preview) > min(total, 4):
        raise ValueError(f"{field} preview must contain at most the first four residuals")
    for index, residual in enumerate(preview):
        require_source_timing_groove_residual(residual, f"{field}.primary_groove_preview[{index}]")


def require_source_timing_groove_residual(value: Any, name: str) -> None:
    residual = require_object(value, name)
    require_one_of(residual, "subdivision", GROOVE_SUBDIVISIONS)
    require_number(residual, "offset_ms")
    confidence = require_number_value(residual, "confidence")
    if confidence < 0 or confidence > 1:
        raise ValueError(f"{name}.confidence must be between 0 and 1")


def require_number(parent: dict[str, Any], field: str) -> None:
    require_number_value(parent, field)


def require_number_value(parent: dict[str, Any], field: str) -> float | int:
    value = parent.get(field)
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise TypeError(f"{field} must be a number")
    return value


def require_non_negative_number(
    parent: dict[str, Any], field: str, prefix: str
) -> float | int:
    value = require_number_value(parent, field)
    if value < 0.0:
        raise ValueError(f"{prefix}.{field} must be non-negative")
    return value


def require_optional_int(parent: dict[str, Any], field: str) -> None:
    require_optional_int_value(parent, field)


def require_optional_int_value(parent: dict[str, Any], field: str) -> int | None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an integer or null")
    value = parent.get(field)
    if value is not None and (not isinstance(value, int) or isinstance(value, bool)):
        raise TypeError(f"{field} must be an integer or null")
    return value


def require_non_negative_int(parent: dict[str, Any], field: str) -> int:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer")
    if value < 0:
        raise ValueError(f"{field} must be non-negative")
    return value


def require_optional_string(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a string or null")
    value = parent.get(field)
    if value is not None and (not isinstance(value, str) or not value):
        raise TypeError(f"{field} must be a non-empty string or null")


def require_one_of(parent: dict[str, Any], field: str, allowed: set[str]) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")
    if value not in allowed:
        raise ValueError(f"{field} must be one of {sorted(allowed)}, got {value!r}")
    return value


def require_optional_one_of(
    parent: dict[str, Any], field: str, allowed: set[str]
) -> str | None:
    if field not in parent:
        return None
    value = parent.get(field)
    if value is None:
        return None
    return require_one_of(parent, field, allowed)


def require_nullable_one_of(
    parent: dict[str, Any], field: str, allowed: set[str]
) -> str | None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a string or null")
    value = parent.get(field)
    if value is None:
        return None
    return require_one_of(parent, field, allowed)


def require_source_timing_policy_cue_match(cue: str, degraded_policy: str) -> None:
    expected = SOURCE_TIMING_CUE_BY_POLICY[degraded_policy]
    if cue != expected:
        raise ValueError(
            "observer_source_timing.cue must match degraded_policy "
            f"{degraded_policy!r}: expected {expected!r}, got {cue!r}"
        )


def require_observer_source_timing_grid_use_match(
    grid_use: str,
    degraded_policy: str,
    bpm_estimate: Any,
    beat_count: int,
    bar_count: int,
    phrase_count: int,
    warning_codes: list[str],
) -> None:
    expected = observer_source_timing_grid_use(
        degraded_policy,
        bpm_estimate,
        beat_count,
        bar_count,
        phrase_count,
        warning_codes,
    )
    if grid_use != expected:
        raise ValueError(
            "observer_source_timing.grid_use must match degraded timing evidence "
            f"{degraded_policy!r}: expected {expected!r}, got {grid_use!r}"
        )


def require_observer_source_timing_count_match(
    beat_status: str,
    beat_count: int,
    downbeat_status: str,
    bar_count: int,
    phrase_status: str,
    phrase_count: int,
) -> None:
    if beat_count < 0:
        raise ValueError("observer_source_timing.beat_count must be non-negative")
    if bar_count < 0:
        raise ValueError("observer_source_timing.bar_count must be non-negative")
    if phrase_count < 0:
        raise ValueError("observer_source_timing.phrase_count must be non-negative")
    if beat_status == "grid" and beat_count == 0:
        raise ValueError(
            "observer_source_timing grid beat_status requires positive beat_count"
        )
    if downbeat_status == "bar_locked" and bar_count == 0:
        raise ValueError(
            "observer_source_timing bar_locked downbeat_status requires positive bar_count"
        )
    if phrase_status == "phrase_locked" and (phrase_count == 0 or bar_count == 0):
        raise ValueError(
            "observer_source_timing phrase_locked requires positive bar_count and phrase_count"
        )
    if phrase_status != "phrase_locked" and phrase_count != 0:
        raise ValueError(
            "observer_source_timing non-locked phrase status must not report primary phrases"
        )


def observer_source_timing_grid_use(
    degraded_policy: str,
    bpm_estimate: Any,
    beat_count: int,
    bar_count: int,
    phrase_count: int,
    warning_codes: list[str],
) -> str:
    if bpm_estimate is None or degraded_policy in {"disabled", "unknown"}:
        return "unavailable"
    if degraded_policy == "locked":
        return "locked_grid"
    if degraded_policy == "fallback_grid":
        return "fallback_grid"
    if (
        degraded_policy == "cautious"
        and beat_count > 0
        and bar_count > 0
        and phrase_count == 0
        and "phrase_uncertain" in warning_codes
    ):
        return "short_loop_manual_confirm"
    return "manual_confirm_only"


def require_source_timing_grid_use_compatibility_match(
    observer_grid_use: str,
    manifest_grid_use: str | None,
    grid_use_compatibility: str,
    issues: list[str],
) -> None:
    expected, should_have_issue = source_timing_grid_use_compatibility(
        observer_grid_use, manifest_grid_use
    )
    if grid_use_compatibility != expected:
        raise ValueError(
            "source_timing_alignment.grid_use_compatibility must match "
            f"observer/manifest grid_use: expected {expected!r}, got {grid_use_compatibility!r}"
        )
    has_grid_use_issue = any(
        issue.startswith("source_timing_alignment.grid_use") for issue in issues
    )
    if should_have_issue and not has_grid_use_issue:
        raise ValueError("source_timing_alignment grid-use mismatch must include an issue")
    if not should_have_issue and has_grid_use_issue:
        raise ValueError("source_timing_alignment grid-use issue present without mismatch")


def require_source_timing_downbeat_offset_compatibility_match(
    observer_offset: int | None,
    manifest_offset: int | None,
    downbeat_offset_compatibility: str,
    issues: list[str],
) -> None:
    expected, should_have_issue = source_timing_downbeat_offset_compatibility(
        observer_offset, manifest_offset
    )
    if downbeat_offset_compatibility != expected:
        raise ValueError(
            "source_timing_alignment.downbeat_offset_compatibility must match "
            "observer/manifest offsets: "
            f"expected {expected!r}, got {downbeat_offset_compatibility!r}"
        )
    has_offset_issue = any(
        issue.startswith("source_timing_alignment.downbeat_offset") for issue in issues
    )
    if should_have_issue and not has_offset_issue:
        raise ValueError("source_timing_alignment downbeat-offset mismatch must include an issue")
    if not should_have_issue and has_offset_issue:
        raise ValueError(
            "source_timing_alignment downbeat-offset issue present without mismatch"
        )


def require_source_timing_downbeat_ambiguity_compatibility_match(
    downbeat_ambiguity_compatibility: str,
    issues: list[str],
) -> None:
    has_ambiguity_issue = any(
        issue.startswith(
            (
                "source_timing_alignment.downbeat_alternates",
                "source_timing_alignment.downbeat_score",
                "source_timing_alignment.downbeat_gap",
            )
        )
        for issue in issues
    )
    if downbeat_ambiguity_compatibility == "mismatch" and not has_ambiguity_issue:
        raise ValueError(
            "source_timing_alignment downbeat-ambiguity mismatch must include an issue"
        )
    if downbeat_ambiguity_compatibility != "mismatch" and has_ambiguity_issue:
        raise ValueError(
            "source_timing_alignment downbeat-ambiguity issue present without mismatch"
        )


def require_source_timing_bpm_delta_alignment_match(
    bpm_delta: float | int | None,
    bpm_tolerance: float | int,
    issues: list[str],
) -> None:
    if bpm_delta is not None and bpm_delta < 0:
        raise ValueError("source_timing_alignment.bpm_delta must be non-negative")
    if bpm_tolerance < 0:
        raise ValueError("source_timing_alignment.bpm_tolerance must be non-negative")
    has_bpm_issue = any(
        issue.startswith("source_timing_alignment.bpm_delta") for issue in issues
    )
    out_of_tolerance = bpm_delta is not None and bpm_delta > bpm_tolerance
    if out_of_tolerance and not has_bpm_issue:
        raise ValueError("source_timing_alignment BPM mismatch must include an issue")
    if not out_of_tolerance and has_bpm_issue:
        raise ValueError("source_timing_alignment BPM issue present without mismatch")


def source_timing_downbeat_offset_compatibility(
    observer_offset: int | None, manifest_offset: int | None
) -> tuple[str, bool]:
    if observer_offset is None or manifest_offset is None:
        return ("partial", False)
    if observer_offset == manifest_offset:
        return ("aligned", False)
    return ("mismatch", True)


def source_timing_grid_use_compatibility(
    observer_grid_use: str, manifest_grid_use: str | None
) -> tuple[str, bool]:
    if manifest_grid_use is None:
        return ("partial", False)
    if observer_grid_use == manifest_grid_use:
        return ("aligned", False)
    if observer_grid_use == "locked_grid":
        return ("mismatch", True)
    if observer_grid_use in {"unavailable", "fallback_grid"} and manifest_grid_use in {
        "locked_grid",
        "short_loop_manual_confirm",
    }:
        return ("mismatch", True)
    if observer_grid_use in {
        "manual_confirm_only",
        "short_loop_manual_confirm",
    } and manifest_grid_use in {
        "manual_confirm_only",
        "short_loop_manual_confirm",
        "locked_grid",
    }:
        return ("compatible", False)
    return ("partial", False)


def require_source_timing_readiness_cue_match(
    cue: str, readiness: str, requires_manual_confirm: bool
) -> None:
    expected = source_timing_readiness_cue(readiness, requires_manual_confirm)
    if cue != expected:
        raise ValueError(
            "source_timing.cue must match readiness/manual-confirm state "
            f"{readiness!r}/{requires_manual_confirm!r}: expected {expected!r}, got {cue!r}"
        )


def source_timing_readiness_cue(readiness: str, requires_manual_confirm: bool) -> str:
    if readiness == "unavailable":
        return "not available"
    if requires_manual_confirm:
        return "needs confirm"
    if readiness == "ready":
        return "grid locked"
    if readiness in {"needs_review", "weak"}:
        return "listen first"
    return "unknown"


def require_source_timing_readiness_actionability_match(
    actionability: str, readiness: str, requires_manual_confirm: bool
) -> None:
    expected = source_timing_readiness_actionability(readiness, requires_manual_confirm)
    if actionability != expected:
        raise ValueError(
            "source_timing.actionability must match readiness/manual-confirm state "
            f"{readiness!r}/{requires_manual_confirm!r}: expected {expected!r}, got {actionability!r}"
        )


def source_timing_readiness_actionability(
    readiness: str, requires_manual_confirm: bool
) -> str:
    if readiness == "unavailable":
        return "timing unavailable"
    if requires_manual_confirm:
        return "confirm grid first"
    if readiness == "ready":
        return "grid can steer moves"
    if readiness in {"needs_review", "weak"}:
        return "listen first"
    return "unknown"


def require_source_timing_grid_use_match(
    source_timing: dict[str, Any], grid_use: str
) -> None:
    expected = source_timing_grid_use(source_timing)
    if grid_use != expected:
        raise ValueError(f"source_timing.grid_use must be {expected!r}, got {grid_use!r}")


def require_source_timing_phrase_evidence_match(source_timing: dict[str, Any]) -> None:
    phrase_status = source_timing["phrase_status"]
    phrase_count = source_timing["primary_phrase_count"]
    phrase_bar_count = source_timing["primary_phrase_bar_count"]
    if phrase_status == "stable" and (phrase_count == 0 or phrase_bar_count == 0):
        raise ValueError(
            "source_timing stable phrase evidence requires positive "
            "primary_phrase_count and primary_phrase_bar_count"
        )
    if phrase_status == "unavailable" and (phrase_count != 0 or phrase_bar_count != 0):
        raise ValueError(
            "source_timing unavailable phrase evidence requires zero "
            "primary_phrase_count and primary_phrase_bar_count"
        )
    if phrase_status == "not_enough_material" and phrase_count != 0:
        raise ValueError(
            "source_timing not_enough_material phrase evidence must not report "
            "primary phrases"
        )


def source_timing_grid_use(source_timing: dict[str, Any]) -> str:
    if source_timing.get("primary_bpm") is None or source_timing["readiness"] == "unavailable":
        return "unavailable"
    if source_timing["readiness"] == "ready" and not source_timing["requires_manual_confirm"]:
        return "locked_grid"
    if is_stable_short_loop_manual_confirm(source_timing):
        return "short_loop_manual_confirm"
    if source_timing["requires_manual_confirm"]:
        return "manual_confirm_only"
    return "fallback_grid"


def is_stable_short_loop_manual_confirm(source_timing: dict[str, Any]) -> bool:
    return (
        source_timing["readiness"] == "needs_review"
        and source_timing["requires_manual_confirm"] is True
        and source_timing.get("primary_bpm") is not None
        and source_timing["beat_status"] == "stable"
        and source_timing["downbeat_status"] == "stable"
        and source_timing["phrase_status"] == "not_enough_material"
        and source_timing["confidence_result"] == "candidate_cautious"
        and source_timing["alternate_evidence_count"] == 0
    )


if __name__ == "__main__":
    raise SystemExit(main())
