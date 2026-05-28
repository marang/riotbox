#!/usr/bin/env python3
"""Validate Riotbox user-session observer NDJSON v1.

This validator is intentionally narrow and dependency-free. It accepts the
current synthetic observer fixtures, and validates app-generated snapshots more
strictly when those snapshots are present.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.user_session_observer.v1"
SOURCE_TIMING_CUE_BY_POLICY = {
    "locked": "grid locked",
    "manual_confirm": "needs confirm",
    "cautious": "listen first",
    "fallback_grid": "fallback grid",
    "disabled": "not available",
    "unknown": "unknown",
}
SOURCE_TIMING_ACTIONABILITY_BY_POLICY = {
    "locked": "grid can steer moves",
    "manual_confirm": "confirm grid first",
    "cautious": "listen first",
    "fallback_grid": "using safe fallback grid",
    "disabled": "timing unavailable",
    "unknown": "unknown",
}
SOURCE_TIMING_GRID_USE = {
    "locked_grid",
    "short_loop_manual_confirm",
    "manual_confirm_only",
    "fallback_grid",
    "unavailable",
}
GROOVE_SUBDIVISIONS = {
    "eighth",
    "triplet",
    "sixteenth",
    "thirty_second",
}
SOURCE_TIMING_WARNING_PRIORITY = (
    "drift_high",
    "ambiguous_downbeat",
    "sparse_onsets",
    "low_timing_confidence",
    "weak_kick_anchor",
    "weak_backbeat_anchor",
    "half_time_possible",
    "double_time_possible",
    "phrase_uncertain",
)


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate_user_session_observer_ndjson.py <events.ndjson>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    try:
        events = load_events(path)
        validate_events(events)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid user-session observer NDJSON: {error}", file=sys.stderr)
        return 1

    print(f"valid {SCHEMA} events: {path}")
    return 0


def load_events(path: Path) -> list[dict[str, Any]]:
    events: list[dict[str, Any]] = []
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValueError(f"line {line_number}: invalid JSON: {error}") from error
        events.append(require_object(event, f"line {line_number}"))
    if not events:
        raise ValueError("file must contain at least one observer event")
    return events


def validate_events(events: list[dict[str, Any]]) -> None:
    first = events[0]
    require_equal(first, "event", "observer_started")
    require_equal(first, "schema", SCHEMA)
    validate_launch(require_object_field(first, "launch"))
    validate_snapshot_if_present(first)

    for index, event in enumerate(events[1:], start=2):
        event_type = require_string(event, "event")
        if event_type == "audio_runtime":
            require_string(event, "status")
        elif event_type == "key_outcome":
            require_string(event, "key")
            require_string(event, "outcome")
        elif event_type == "transport_commit":
            validate_committed(event)
        else:
            raise ValueError(f"line {index}: unknown observer event {event_type!r}")
        validate_snapshot_if_present(event)


def validate_launch(launch: dict[str, Any]) -> None:
    mode = require_string(launch, "mode")
    if mode not in {"ingest", "load"}:
        raise ValueError(f"launch mode must be 'ingest' or 'load', got {mode!r}")

    if mode == "ingest":
        if "source_path" in launch:
            require_string(launch, "source_path")
        elif "source" in launch:
            require_string(launch, "source")
        else:
            raise TypeError("ingest launch must include source_path or legacy source")
    else:
        require_string(launch, "session_path")


def validate_committed(event: dict[str, Any]) -> None:
    committed = require_list(event, "committed")
    for index, item in enumerate(committed):
        commit = require_object(item, f"committed[{index}]")
        require_int(commit, "action_id")
        require_string(commit, "boundary")
        require_int(commit, "beat_index")
        require_int(commit, "bar_index")
        require_int(commit, "phrase_index")
        require_int(commit, "commit_sequence")


def validate_snapshot_if_present(event: dict[str, Any]) -> None:
    if "snapshot" not in event:
        return
    snapshot = require_object_field(event, "snapshot")
    require_object_field(snapshot, "transport")
    require_object_field(snapshot, "queue")
    require_object_field(snapshot, "runtime")
    validate_source_timing(snapshot.get("source_timing"))
    validate_recovery(require_object_field(snapshot, "recovery"))


def validate_source_timing(value: Any) -> None:
    if value is None:
        return
    source_timing = require_object(value, "source_timing")
    require_bool(source_timing, "present")
    require_string(source_timing, "source_id")
    bpm_estimate = require_optional_number(source_timing, "bpm_estimate")
    require_number(source_timing, "bpm_confidence")
    require_one_of(source_timing, "quality", {"low", "medium", "high", "unknown"})
    cue = require_one_of(
        source_timing,
        "cue",
        set(SOURCE_TIMING_CUE_BY_POLICY.values()),
    )
    degraded_policy = require_one_of(
        source_timing,
        "degraded_policy",
        set(SOURCE_TIMING_CUE_BY_POLICY),
    )
    require_source_timing_policy_cue_match(cue, degraded_policy)
    actionability = require_one_of(
        source_timing,
        "actionability",
        set(SOURCE_TIMING_ACTIONABILITY_BY_POLICY.values()),
    )
    require_source_timing_policy_actionability_match(actionability, degraded_policy)
    grid_use = require_one_of(source_timing, "grid_use", SOURCE_TIMING_GRID_USE)
    beat_status = require_one_of(source_timing, "beat_status", {"grid", "tempo_only", "unknown"})
    beat_count = require_int_value(source_timing, "beat_count")
    downbeat_status = require_one_of(
        source_timing, "downbeat_status", {"ambiguous", "bar_locked", "unknown"}
    )
    require_nullable_non_negative_int(source_timing, "primary_downbeat_offset_beats")
    primary_downbeat_score = require_nullable_number(source_timing, "primary_downbeat_score")
    if primary_downbeat_score is not None and (
        primary_downbeat_score < 0 or primary_downbeat_score > 1
    ):
        raise ValueError("source_timing.primary_downbeat_score must be between 0 and 1")
    require_nullable_non_negative_number(source_timing, "primary_downbeat_score_gap")
    require_non_negative_int(source_timing, "alternate_downbeat_phase_count")
    bar_count = require_int_value(source_timing, "bar_count")
    phrase_status = require_one_of(
        source_timing, "phrase_status", {"uncertain", "phrase_locked", "unknown"}
    )
    phrase_count = require_int_value(source_timing, "phrase_count")
    require_source_timing_count_match(
        beat_status,
        beat_count,
        downbeat_status,
        bar_count,
        phrase_status,
        phrase_count,
    )
    primary = source_timing.get("primary_hypothesis_id")
    if primary is not None and not isinstance(primary, str):
        raise TypeError("source_timing.primary_hypothesis_id must be a string or null")
    require_int(source_timing, "hypothesis_count")
    validate_source_timing_anchor_evidence(source_timing.get("anchor_evidence"))
    require_string(source_timing, "primary_anchor_cue")
    validate_source_timing_groove_evidence(source_timing.get("groove_evidence"))
    warning = source_timing.get("primary_warning_code")
    if warning is not None and not isinstance(warning, str):
        raise TypeError("source_timing.primary_warning_code must be a string or null")
    warning_codes = require_list(source_timing, "warning_codes")
    if any(not isinstance(code, str) or not code for code in warning_codes):
        raise TypeError("source_timing.warning_codes must contain non-empty strings")
    if degraded_policy == "locked" and (warning is not None or warning_codes):
        raise ValueError("locked source_timing must not carry warning evidence")
    require_source_timing_primary_warning_match(warning, warning_codes)
    require_source_timing_grid_use_match(
        grid_use,
        degraded_policy,
        bpm_estimate,
        beat_count,
        bar_count,
        phrase_count,
        warning_codes,
    )


def validate_source_timing_anchor_evidence(value: Any) -> None:
    anchor_evidence = require_object(value, "source_timing.anchor_evidence")
    total = require_non_negative_int(anchor_evidence, "primary_anchor_count")
    kick = require_non_negative_int(anchor_evidence, "primary_kick_anchor_count")
    backbeat = require_non_negative_int(anchor_evidence, "primary_backbeat_anchor_count")
    transient = require_non_negative_int(
        anchor_evidence,
        "primary_transient_anchor_count",
    )
    if kick + backbeat + transient > total:
        raise ValueError(
            "source_timing.anchor_evidence typed anchor counts cannot exceed total anchors"
        )


def validate_source_timing_groove_evidence(value: Any) -> None:
    groove_evidence = require_object(value, "source_timing.groove_evidence")
    total = require_non_negative_int(
        groove_evidence,
        "primary_groove_residual_count",
    )
    max_abs = require_number(groove_evidence, "primary_max_abs_offset_ms")
    if max_abs < 0:
        raise ValueError("primary_max_abs_offset_ms must be non-negative")
    preview = require_list(groove_evidence, "primary_groove_preview")
    if len(preview) > min(total, 4):
        raise ValueError(
            "source_timing.groove_evidence preview must contain at most the first four residuals"
        )
    for index, item in enumerate(preview):
        validate_source_timing_groove_residual(item, index)


def validate_source_timing_groove_residual(value: Any, index: int) -> None:
    residual = require_object(value, f"source_timing.groove_evidence[{index}]")
    require_one_of(residual, "subdivision", GROOVE_SUBDIVISIONS)
    require_number(residual, "offset_ms")
    confidence = require_number(residual, "confidence")
    if confidence < 0 or confidence > 1:
        raise ValueError("source_timing.groove_evidence confidence must be between 0 and 1")


def validate_recovery(recovery: dict[str, Any]) -> None:
    require_bool(recovery, "present")
    require_bool(recovery, "has_manual_candidates")
    selected = recovery.get("selected_candidate")
    if selected is not None and not isinstance(selected, str):
        raise TypeError("selected_candidate must be a string or null")
    require_int(recovery, "candidate_count")
    candidates = require_list(recovery, "candidates")
    if recovery["candidate_count"] != len(candidates):
        raise ValueError("candidate_count must match candidates length")
    dry_run = recovery.get("manual_choice_dry_run")
    if dry_run is not None:
        validate_manual_choice_dry_run(require_object(dry_run, "manual_choice_dry_run"))

    for index, item in enumerate(candidates):
        candidate = require_object(item, f"recovery.candidates[{index}]")
        require_string(candidate, "path")
        require_string(candidate, "kind")
        require_string(candidate, "status")
        require_string(candidate, "artifact_availability")
        require_string(candidate, "replay_readiness")
        require_string(candidate, "payload_readiness")
        require_string(candidate, "replay_suffix")
        require_replay_family(candidate, "replay_family")
        require_string(candidate, "replay_unsupported")
        require_string(candidate, "trust")
        require_string(candidate, "action_hint")
        guidance = candidate.get("guidance")
        if guidance is not None and not isinstance(guidance, str):
            raise TypeError(f"recovery.candidates[{index}].guidance must be a string or null")
        require_string(candidate, "decision")


def validate_manual_choice_dry_run(dry_run: dict[str, Any]) -> None:
    require_string(dry_run, "candidate_path")
    require_string(dry_run, "decision")
    require_string(dry_run, "artifact_availability")
    require_string(dry_run, "replay_readiness")
    require_string(dry_run, "payload_readiness")
    require_string(dry_run, "replay_suffix")
    require_replay_family(dry_run, "replay_family")
    require_string(dry_run, "replay_unsupported")
    guidance = dry_run.get("guidance")
    if guidance is not None and not isinstance(guidance, str):
        raise TypeError("manual_choice_dry_run.guidance must be a string or null")
    require_string(dry_run, "trust")
    require_string(dry_run, "action_hint")
    require_bool(dry_run, "selected_for_restore")
    if dry_run["selected_for_restore"]:
        raise ValueError("manual_choice_dry_run.selected_for_restore must stay false")
    require_string(dry_run, "safety_note")


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_object_field(parent: dict[str, Any], field: str) -> dict[str, Any]:
    return require_object(parent.get(field), field)


def require_list(parent: dict[str, Any], field: str) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{field} must be an array")
    return value


def require_equal(parent: dict[str, Any], field: str, expected: Any) -> None:
    actual = parent.get(field)
    if actual != expected:
        raise ValueError(f"{field} must be {expected!r}, got {actual!r}")


def require_bool(parent: dict[str, Any], field: str) -> None:
    if not isinstance(parent.get(field), bool):
        raise TypeError(f"{field} must be a boolean")


def require_int(parent: dict[str, Any], field: str) -> None:
    require_int_value(parent, field)


def require_int_value(parent: dict[str, Any], field: str) -> int:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer")
    return value


def require_non_negative_int(parent: dict[str, Any], field: str) -> int:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer")
    if value < 0:
        raise ValueError(f"{field} must be non-negative")
    return value


def require_optional_non_negative_int(parent: dict[str, Any], field: str) -> int | None:
    value = parent.get(field)
    if value is None:
        return None
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer or null")
    if value < 0:
        raise ValueError(f"{field} must be non-negative")
    return value


def require_nullable_non_negative_int(parent: dict[str, Any], field: str) -> int | None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an integer or null")
    return require_optional_non_negative_int(parent, field)


def require_number(parent: dict[str, Any], field: str) -> float | int:
    value = parent.get(field)
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise TypeError(f"{field} must be a number")
    return value


def require_optional_number(parent: dict[str, Any], field: str) -> float | int | None:
    value = parent.get(field)
    if value is None:
        return None
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise TypeError(f"{field} must be a number or null")
    return value


def require_nullable_number(parent: dict[str, Any], field: str) -> float | int | None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a number or null")
    return require_optional_number(parent, field)


def require_nullable_non_negative_number(
    parent: dict[str, Any], field: str
) -> float | int | None:
    value = require_nullable_number(parent, field)
    if value is not None and value < 0:
        raise ValueError(f"{field} must be non-negative")
    return value


def require_string(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")
    return value


def require_one_of(parent: dict[str, Any], field: str, allowed: set[str]) -> str:
    value = require_string(parent, field)
    if value not in allowed:
        raise ValueError(f"{field} must be one of {sorted(allowed)}, got {value!r}")
    return value


def require_source_timing_policy_cue_match(cue: str, degraded_policy: str) -> None:
    expected = SOURCE_TIMING_CUE_BY_POLICY[degraded_policy]
    if cue != expected:
        raise ValueError(
            "source_timing.cue must match degraded_policy "
            f"{degraded_policy!r}: expected {expected!r}, got {cue!r}"
        )


def require_source_timing_policy_actionability_match(
    actionability: str, degraded_policy: str
) -> None:
    expected = SOURCE_TIMING_ACTIONABILITY_BY_POLICY[degraded_policy]
    if actionability != expected:
        raise ValueError(
            "source_timing.actionability must match degraded_policy "
            f"{degraded_policy!r}: expected {expected!r}, got {actionability!r}"
        )


def require_source_timing_grid_use_match(
    grid_use: str,
    degraded_policy: str,
    bpm_estimate: float | int | None,
    beat_count: int,
    bar_count: int,
    phrase_count: int,
    warning_codes: list[Any],
) -> None:
    expected = source_timing_grid_use(
        degraded_policy,
        bpm_estimate,
        beat_count,
        bar_count,
        phrase_count,
        warning_codes,
    )
    if grid_use != expected:
        raise ValueError(
            "source_timing.grid_use must match degraded timing evidence "
            f"{degraded_policy!r}: expected {expected!r}, got {grid_use!r}"
        )


def require_source_timing_count_match(
    beat_status: str,
    beat_count: int,
    downbeat_status: str,
    bar_count: int,
    phrase_status: str,
    phrase_count: int,
) -> None:
    if beat_count < 0:
        raise ValueError("source_timing.beat_count must be non-negative")
    if bar_count < 0:
        raise ValueError("source_timing.bar_count must be non-negative")
    if phrase_count < 0:
        raise ValueError("source_timing.phrase_count must be non-negative")
    if beat_status == "grid" and beat_count == 0:
        raise ValueError("source_timing grid beat_status requires positive beat_count")
    if downbeat_status == "bar_locked" and bar_count == 0:
        raise ValueError("source_timing bar_locked downbeat_status requires positive bar_count")
    if phrase_status == "phrase_locked" and (phrase_count == 0 or bar_count == 0):
        raise ValueError(
            "source_timing phrase_locked requires positive bar_count and phrase_count"
        )
    if phrase_status != "phrase_locked" and phrase_count != 0:
        raise ValueError(
            "source_timing non-locked phrase status must not report primary phrases"
        )


def source_timing_grid_use(
    degraded_policy: str,
    bpm_estimate: float | int | None,
    beat_count: int,
    bar_count: int,
    phrase_count: int,
    warning_codes: list[Any],
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


def require_source_timing_primary_warning_match(
    primary_warning_code: str | None, warning_codes: list[Any]
) -> None:
    if not warning_codes:
        if primary_warning_code is not None:
            raise ValueError(
                "source_timing.primary_warning_code must be null when warning_codes is empty"
            )
        return

    expected = primary_source_timing_warning(warning_codes)
    if primary_warning_code != expected:
        raise ValueError(
            "source_timing.primary_warning_code must match warning priority "
            f"expected {expected!r}, got {primary_warning_code!r}"
        )


def primary_source_timing_warning(warning_codes: list[Any]) -> str:
    priority = {
        code: index for index, code in enumerate(SOURCE_TIMING_WARNING_PRIORITY)
    }
    return min(
        enumerate(warning_codes),
        key=lambda indexed_code: (
            priority.get(indexed_code[1], len(SOURCE_TIMING_WARNING_PRIORITY)),
            indexed_code[0],
        ),
    )[1]


def require_replay_family(parent: dict[str, Any], field: str) -> str:
    value = require_string(parent, field)
    if not value.startswith("families "):
        raise ValueError(f"{field} must start with 'families ', got {value!r}")
    return value


if __name__ == "__main__":
    raise SystemExit(main())
