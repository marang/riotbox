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
    validate_recovery(require_object_field(snapshot, "recovery"))


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

    for index, item in enumerate(candidates):
        candidate = require_object(item, f"recovery.candidates[{index}]")
        require_string(candidate, "path")
        require_string(candidate, "kind")
        require_string(candidate, "status")
        require_string(candidate, "artifact_availability")
        require_string(candidate, "replay_readiness")
        require_string(candidate, "payload_readiness")
        require_string(candidate, "replay_suffix")
        require_string(candidate, "replay_unsupported")
        require_string(candidate, "trust")
        require_string(candidate, "action_hint")
        guidance = candidate.get("guidance")
        if guidance is not None and not isinstance(guidance, str):
            raise TypeError(f"recovery.candidates[{index}].guidance must be a string or null")
        require_string(candidate, "decision")


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


def require_string(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")
    return value


if __name__ == "__main__":
    raise SystemExit(main())
