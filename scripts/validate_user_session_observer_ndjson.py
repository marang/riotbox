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
    require_optional_number(source_timing, "bpm_estimate")
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
    require_one_of(source_timing, "beat_status", {"grid", "tempo_only", "unknown"})
    require_int(source_timing, "beat_count")
    require_one_of(source_timing, "downbeat_status", {"ambiguous", "bar_locked", "unknown"})
    require_int(source_timing, "bar_count")
    require_one_of(source_timing, "phrase_status", {"uncertain", "phrase_locked", "unknown"})
    require_int(source_timing, "phrase_count")
    primary = source_timing.get("primary_hypothesis_id")
    if primary is not None and not isinstance(primary, str):
        raise TypeError("source_timing.primary_hypothesis_id must be a string or null")
    require_int(source_timing, "hypothesis_count")
    warning = source_timing.get("primary_warning_code")
    if warning is not None and not isinstance(warning, str):
        raise TypeError("source_timing.primary_warning_code must be a string or null")
    warning_codes = require_list(source_timing, "warning_codes")
    if any(not isinstance(code, str) or not code for code in warning_codes):
        raise TypeError("source_timing.warning_codes must contain non-empty strings")


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
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer")


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


def require_replay_family(parent: dict[str, Any], field: str) -> str:
    value = require_string(parent, field)
    if not value.startswith("families "):
        raise ValueError(f"{field} must start with 'families ', got {value!r}")
    return value


if __name__ == "__main__":
    raise SystemExit(main())
