#!/usr/bin/env python3
"""Validate Riotbox source timing probe CLI JSON v1."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.source_timing_probe_cli.v1"
SCHEMA_VERSION = 1
SOURCE_TIMING_CUES = {
    "grid locked",
    "needs confirm",
    "listen first",
    "not available",
}
GRID_USE = {
    "locked_grid",
    "short_loop_manual_confirm",
    "manual_confirm_only",
    "fallback_grid",
    "unavailable",
}
ANCHOR_TYPES = {
    "kick",
    "snare",
    "backbeat",
    "fill",
    "loop_window",
    "answer_slot",
    "capture_candidate",
    "transient_cluster",
}
GROOVE_SUBDIVISIONS = {
    "eighth",
    "triplet",
    "sixteenth",
    "thirty_second",
}


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate_source_timing_probe_json.py <probe.json>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    try:
        summary = json.loads(path.read_text())
        validate_summary(summary)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid source timing probe JSON: {error}", file=sys.stderr)
        return 1

    print(f"valid {SCHEMA} summary: {path}")
    return 0


def validate_summary(summary: Any) -> None:
    require_object(summary, "summary")
    require_equal(summary, "schema", SCHEMA)
    require_equal(summary, "schema_version", SCHEMA_VERSION)
    require_string(summary, "source_path")
    require_string(summary, "source_id")
    cue = require_one_of(summary, "cue", SOURCE_TIMING_CUES)
    readiness = require_one_of(summary, "readiness", {"ready", "needs_review", "weak", "unavailable"})
    requires_manual_confirm = require_bool(summary, "requires_manual_confirm")
    require_probe_cue_match(cue, readiness, requires_manual_confirm)
    grid_use = require_one_of(summary, "grid_use", GRID_USE)
    require_optional_number(summary, "primary_bpm")
    require_optional_number(summary, "primary_beat_score")
    require_optional_number(summary, "primary_beat_matched_onset_ratio")
    require_optional_number(summary, "primary_beat_median_distance_ratio")
    require_optional_int(summary, "primary_downbeat_offset_beats")
    require_optional_number(summary, "primary_downbeat_score")
    require_one_of(summary, "beat_status", {"unavailable", "weak", "stable", "ambiguous"})
    require_one_of(summary, "downbeat_status", {"unavailable", "weak", "stable", "ambiguous"})
    require_one_of(
        summary,
        "confidence_result",
        {"degraded", "candidate_cautious", "candidate_ambiguous"},
    )
    require_one_of(summary, "drift_status", {"unavailable", "not_enough_material", "stable", "high"})
    require_one_of(
        summary,
        "phrase_status",
        {"unavailable", "not_enough_material", "ambiguous_downbeat", "high_drift", "stable"},
    )
    require_non_negative_int(summary, "alternate_evidence_count")
    require_non_negative_int(summary, "alternate_beat_candidate_count")
    require_non_negative_int(summary, "alternate_downbeat_phase_count")
    validate_anchor_evidence(summary)
    validate_groove_evidence(summary)
    require_string_list(summary, "warning_codes")
    require_non_negative_int(summary, "onset_count")
    require_non_negative_number(summary, "onset_density_per_second")
    require_non_negative_number(summary, "duration_seconds")
    require_grid_use_match(summary, grid_use)


def validate_anchor_evidence(summary: dict[str, Any]) -> None:
    anchor_evidence = require_object(summary.get("anchor_evidence"), "anchor_evidence")
    total = require_non_negative_int(anchor_evidence, "primary_anchor_count")
    kick = require_non_negative_int(anchor_evidence, "primary_kick_anchor_count")
    backbeat = require_non_negative_int(anchor_evidence, "primary_backbeat_anchor_count")
    transient = require_non_negative_int(anchor_evidence, "primary_transient_anchor_count")
    if kick + backbeat + transient > total:
        raise ValueError(
            "primary kick/backbeat/transient anchor counts must not exceed primary_anchor_count"
        )
    preview = require_array(anchor_evidence, "primary_anchor_preview")
    if len(preview) > min(total, 8):
        raise ValueError(
            "primary_anchor_preview must contain at most the first eight primary anchors"
        )
    for index, item in enumerate(preview):
        validate_anchor_preview(require_object(item, f"primary_anchor_preview[{index}]"))


def validate_anchor_preview(anchor: dict[str, Any]) -> None:
    require_one_of(anchor, "anchor_type", ANCHOR_TYPES)
    require_non_negative_number(anchor, "time_seconds")
    require_optional_int(anchor, "bar_index")
    require_optional_int(anchor, "beat_index")
    confidence = require_non_negative_number(anchor, "confidence")
    if confidence > 1:
        raise ValueError(f"anchor confidence must be <= 1, got {confidence!r}")
    require_non_negative_number(anchor, "strength")
    require_string_list(anchor, "tags")


def validate_groove_evidence(summary: dict[str, Any]) -> None:
    groove_evidence = require_object(summary.get("groove_evidence"), "groove_evidence")
    total = require_non_negative_int(groove_evidence, "primary_groove_residual_count")
    require_non_negative_number(groove_evidence, "primary_max_abs_offset_ms")
    preview = require_array(groove_evidence, "primary_groove_preview")
    if len(preview) > min(total, 4):
        raise ValueError(
            "primary_groove_preview must contain at most the first four primary residuals"
        )
    for index, item in enumerate(preview):
        validate_groove_preview(require_object(item, f"primary_groove_preview[{index}]"))


def validate_groove_preview(residual: dict[str, Any]) -> None:
    require_one_of(residual, "subdivision", GROOVE_SUBDIVISIONS)
    require_number(residual, "offset_ms")
    confidence = require_non_negative_number(residual, "confidence")
    if confidence > 1:
        raise ValueError(f"groove confidence must be <= 1, got {confidence!r}")


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_equal(parent: dict[str, Any], field: str, expected: Any) -> None:
    actual = parent.get(field)
    if actual != expected:
        raise ValueError(f"{field} must be {expected!r}, got {actual!r}")


def require_bool(parent: dict[str, Any], field: str) -> bool:
    value = parent.get(field)
    if not isinstance(value, bool):
        raise TypeError(f"{field} must be a boolean")
    return value


def require_string(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")
    return value


def require_string_list(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if not isinstance(value, list) or any(not isinstance(item, str) or not item for item in value):
        raise TypeError(f"{field} must be an array of non-empty strings")


def require_array(parent: dict[str, Any], field: str) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{field} must be an array")
    return value


def require_one_of(parent: dict[str, Any], field: str, allowed: set[str]) -> str:
    value = require_string(parent, field)
    if value not in allowed:
        raise ValueError(f"{field} must be one of {sorted(allowed)}, got {value!r}")
    return value


def require_optional_number(parent: dict[str, Any], field: str) -> float | int | None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a number or null")
    value = parent.get(field)
    if value is None:
        return None
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise TypeError(f"{field} must be a number or null")
    return value


def require_number(parent: dict[str, Any], field: str) -> float | int:
    value = require_optional_number(parent, field)
    if value is None:
        raise ValueError(f"{field} must be a number")
    return value


def require_non_negative_number(parent: dict[str, Any], field: str) -> float | int:
    value = require_optional_number(parent, field)
    if value is None or value < 0:
        raise ValueError(f"{field} must be a non-negative number")
    return value


def require_optional_int(parent: dict[str, Any], field: str) -> int | None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an integer or null")
    value = parent.get(field)
    if value is None:
        return None
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer or null")
    return value


def require_non_negative_int(parent: dict[str, Any], field: str) -> int:
    value = require_optional_int(parent, field)
    if value is None or value < 0:
        raise ValueError(f"{field} must be a non-negative integer")
    return value


def require_probe_cue_match(cue: str, readiness: str, requires_manual_confirm: bool) -> None:
    expected = source_timing_readiness_cue(readiness, requires_manual_confirm)
    if cue != expected:
        raise ValueError(
            "cue must match readiness/manual-confirm state "
            f"{readiness!r}/{requires_manual_confirm!r}: expected {expected!r}, got {cue!r}"
        )


def require_grid_use_match(summary: dict[str, Any], grid_use: str) -> None:
    primary_bpm = summary.get("primary_bpm")
    readiness = summary["readiness"]
    requires_manual_confirm = summary["requires_manual_confirm"]
    expected = source_timing_grid_use(summary)
    if grid_use != expected:
        raise ValueError(f"grid_use must be {expected!r}, got {grid_use!r}")
    if grid_use == "locked_grid" and requires_manual_confirm:
        raise ValueError("locked_grid must not require manual confirmation")
    if grid_use == "short_loop_manual_confirm":
        if not requires_manual_confirm:
            raise ValueError("short_loop_manual_confirm must require manual confirmation")
        if readiness != "needs_review":
            raise ValueError("short_loop_manual_confirm requires readiness == needs_review")
    if grid_use != "unavailable" and primary_bpm is None:
        raise ValueError(f"{grid_use} requires primary_bpm")


def source_timing_grid_use(summary: dict[str, Any]) -> str:
    if summary.get("primary_bpm") is None or summary["readiness"] == "unavailable":
        return "unavailable"
    if summary["readiness"] == "ready" and not summary["requires_manual_confirm"]:
        return "locked_grid"
    if is_stable_short_loop_manual_confirm(summary):
        return "short_loop_manual_confirm"
    if summary["requires_manual_confirm"]:
        return "manual_confirm_only"
    return "fallback_grid"


def is_stable_short_loop_manual_confirm(summary: dict[str, Any]) -> bool:
    return (
        summary["readiness"] == "needs_review"
        and summary["requires_manual_confirm"] is True
        and summary.get("primary_bpm") is not None
        and summary["beat_status"] == "stable"
        and summary["downbeat_status"] == "stable"
        and summary["phrase_status"] == "not_enough_material"
        and summary["confidence_result"] == "candidate_cautious"
        and summary["alternate_evidence_count"] == 0
    )


def source_timing_readiness_cue(readiness: str, requires_manual_confirm: bool) -> str:
    if requires_manual_confirm:
        return "needs confirm"
    if readiness == "ready":
        return "grid locked"
    if readiness in {"needs_review", "weak"}:
        return "listen first"
    return "not available"


if __name__ == "__main__":
    raise SystemExit(main())
