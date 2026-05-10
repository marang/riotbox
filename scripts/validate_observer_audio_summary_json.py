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

    metrics = require_object_field(output_path, "metrics")
    require_optional_number(metrics, "full_mix_rms")
    require_optional_number(metrics, "full_mix_low_band_rms")
    require_optional_number(metrics, "mc202_question_answer_delta_rms")
    require_optional_source_grid_output_drift(metrics)
    require_optional_source_grid_alignment(metrics, "tr909_source_grid_alignment")
    require_optional_source_grid_alignment(metrics, "w30_source_grid_alignment")
    require_optional_number(metrics, "w30_candidate_rms")
    require_optional_number(metrics, "w30_candidate_active_sample_ratio")
    require_optional_number(metrics, "w30_rms_delta")


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


def require_string_list(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise TypeError(f"{field} must be an array of strings")


def require_array(parent: dict[str, Any], field: str) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{field} must be an array")
    return value


def require_int(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer")


def require_optional_number(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a number or null")
    value = parent.get(field)
    if value is not None and (not isinstance(value, (int, float)) or isinstance(value, bool)):
        raise TypeError(f"{field} must be a number or null")


def require_optional_source_grid_output_drift(parent: dict[str, Any]) -> None:
    require_optional_source_grid_alignment(parent, "source_grid_output_drift")


def require_optional_source_grid_alignment(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    drift = require_object(value, field)
    require_number(drift, "hit_ratio")
    require_number(drift, "max_peak_offset_ms")
    require_number(drift, "max_allowed_peak_offset_ms")


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
    require_string(timing, "policy_profile")
    readiness = require_string(timing, "readiness")
    requires_manual_confirm = require_bool(timing, "requires_manual_confirm")
    require_source_timing_readiness_cue_match(cue, readiness, requires_manual_confirm)
    require_optional_number(timing, "primary_bpm")
    require_optional_bool(timing, "bpm_agrees_with_grid")
    require_string(timing, "beat_status")
    require_string(timing, "downbeat_status")
    require_optional_int(timing, "primary_downbeat_offset_beats")
    require_string(timing, "confidence_result")
    require_string(timing, "drift_status")
    require_string(timing, "phrase_status")
    require_int(timing, "alternate_evidence_count")
    require_optional_source_timing_anchor_evidence(timing, "anchor_evidence")
    require_optional_source_timing_groove_evidence(timing, "groove_evidence")
    require_string_list(timing, "warning_codes")


def require_optional_source_timing_alignment(parent: dict[str, Any]) -> None:
    field = "source_timing_alignment"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    alignment = require_object(value, field)
    require_one_of(alignment, "status", {"aligned", "partial", "mismatch"})
    require_optional_number(alignment, "bpm_delta")
    require_number(alignment, "bpm_tolerance")
    require_string_list(alignment, "warning_overlap")
    require_string_list(alignment, "issues")


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
    require_one_of(alignment, "status", {"aligned", "partial", "mismatch"})
    require_optional_source_timing_anchor_evidence(alignment, "observer")
    require_optional_source_timing_anchor_evidence(alignment, "manifest")
    require_string_list(alignment, "issues")


def require_optional_source_timing_groove_alignment(parent: dict[str, Any]) -> None:
    field = "source_timing_groove_alignment"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    alignment = require_object(value, field)
    require_one_of(alignment, "status", {"aligned", "partial", "mismatch"})
    require_optional_source_timing_groove_evidence(alignment, "observer")
    require_optional_source_timing_groove_evidence(alignment, "manifest")
    require_string_list(alignment, "issues")


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
    require_one_of(timing, "beat_status", {"grid", "tempo_only", "unknown"})
    require_int(timing, "beat_count")
    require_one_of(timing, "downbeat_status", {"ambiguous", "bar_locked", "unknown"})
    require_int(timing, "bar_count")
    require_one_of(timing, "phrase_status", {"uncertain", "phrase_locked", "unknown"})
    require_int(timing, "phrase_count")
    require_optional_string(timing, "primary_hypothesis_id")
    require_int(timing, "hypothesis_count")
    require_optional_source_timing_anchor_evidence(timing, "anchor_evidence")
    require_optional_source_timing_groove_evidence(timing, "groove_evidence")
    require_optional_string(timing, "primary_warning_code")
    require_string_list(timing, "warning_codes")


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


def require_optional_int(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an integer or null")
    value = parent.get(field)
    if value is not None and (not isinstance(value, int) or isinstance(value, bool)):
        raise TypeError(f"{field} must be an integer or null")


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


def require_source_timing_policy_cue_match(cue: str, degraded_policy: str) -> None:
    expected = SOURCE_TIMING_CUE_BY_POLICY[degraded_policy]
    if cue != expected:
        raise ValueError(
            "observer_source_timing.cue must match degraded_policy "
            f"{degraded_policy!r}: expected {expected!r}, got {cue!r}"
        )


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
    if requires_manual_confirm:
        return "needs confirm"
    if readiness == "ready":
        return "grid locked"
    if readiness in {"needs_review", "weak"}:
        return "listen first"
    if readiness == "unavailable":
        return "not available"
    return "unknown"


if __name__ == "__main__":
    raise SystemExit(main())
