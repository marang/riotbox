#!/usr/bin/env python3
"""Validate the committed timing arc of a Feral Break live review take.

This gate is intentionally about arrangement timing, not sound quality.  It
reads the real user-session observer stream and proves that the review take
landed the documented 8 -> 8 -> 4 -> 4 -> 8 beat arc before anyone is asked to
compare its audio.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from validate_user_session_observer_ndjson import validate_events as validate_observer_events


TARGET_INTERVALS = (
    ("hook", 8.0),
    ("slam", 8.0),
    ("fill", 4.0),
    ("scene", 4.0),
    ("return", 8.0),
)
STAGE_COMMANDS = (
    "w30.trigger_pad",
    "tr909.set_slam",
    "tr909.fill_next",
    "scene.launch",
)
RETURN_COMMANDS = {"scene.restore", "w30.apply_damage_profile"}
REVIEW_COMMANDS = set(STAGE_COMMANDS) | RETURN_COMMANDS


@dataclass(frozen=True)
class Commit:
    command: str
    beat: int
    boundary: str
    event_index: int


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("events", type=Path, help="user-session observer NDJSON")
    parser.add_argument(
        "--stop-tolerance-beats",
        type=float,
        default=0.25,
        help="allowed transport-stop error around the eight-beat return (default: 0.25)",
    )
    parser.add_argument("--json-output", type=Path)
    parser.add_argument(
        "--timing-only",
        action="store_true",
        help="skip the base observer-schema gate (fixture testing only)",
    )
    return parser.parse_args()


def load_events(path: Path) -> list[dict[str, Any]]:
    events: list[dict[str, Any]] = []
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValueError(f"line {line_number}: invalid JSON: {error}") from error
        if not isinstance(event, dict):
            raise TypeError(f"line {line_number}: event must be an object")
        events.append(event)
    if not events:
        raise ValueError("observer stream is empty")
    return events


def commits_from_events(events: list[dict[str, Any]]) -> list[Commit]:
    commits: list[Commit] = []
    for event_index, event in enumerate(events):
        if event.get("event") != "transport_commit":
            continue
        snapshot = require_object(event.get("snapshot"), "transport_commit.snapshot")
        queue = require_object(snapshot.get("queue"), "transport_commit.snapshot.queue")
        history = queue.get("recent_history")
        if not isinstance(history, list):
            raise TypeError("transport_commit.snapshot.queue.recent_history must be a list")
        command_by_id = {
            item.get("id"): item.get("command")
            for item in history
            if isinstance(item, dict) and item.get("status") == "Committed"
        }
        committed = event.get("committed")
        if not isinstance(committed, list):
            raise TypeError("transport_commit.committed must be a list")
        for item in committed:
            record = require_object(item, "transport_commit.committed[]")
            action_id = record.get("action_id")
            command = command_by_id.get(action_id)
            if not isinstance(command, str) or not command:
                raise ValueError(f"committed action {action_id!r} has no command in recent_history")
            beat = record.get("beat_index")
            boundary = record.get("boundary")
            if not isinstance(beat, int) or isinstance(beat, bool):
                raise TypeError(f"commit {command}: beat_index must be an integer")
            if not isinstance(boundary, str) or not boundary:
                raise TypeError(f"commit {command}: boundary must be a string")
            commits.append(Commit(command, beat, boundary, event_index))
    return commits


def validate_timing(
    events: list[dict[str, Any]], stop_tolerance_beats: float
) -> dict[str, Any]:
    if not math.isfinite(stop_tolerance_beats) or not 0 <= stop_tolerance_beats <= 0.25:
        raise ValueError("stop tolerance must be finite and between 0 and 0.25 beats")

    commits = commits_from_events(events)
    starts = [index for index, commit in enumerate(commits) if commit.command == STAGE_COMMANDS[0]]
    if not starts:
        raise ValueError("no committed w30.trigger_pad starts a live review arc")

    # A session may contain retries. Only the newest take is reviewable; an
    # older valid take must never hide a later missed boundary.
    stages = find_stage_sequence(commits, starts[-1])
    stop_position = find_stop_position(events, stages[-1].event_index)
    return build_result(stages, stop_position, stop_tolerance_beats)


def find_stage_sequence(commits: list[Commit], start_index: int) -> list[Commit]:
    stages = [commits[start_index]]
    cursor = start_index + 1
    for expected_command in STAGE_COMMANDS[1:]:
        match = next(
            (commit for commit in commits[cursor:] if commit.command in REVIEW_COMMANDS),
            None,
        )
        if match is None:
            raise ValueError(f"live review arc is missing committed {expected_command}")
        if match.command != expected_command:
            raise ValueError(
                f"expected committed {expected_command}, got {match.command}"
            )
        match_index = commits.index(match, cursor)
        stages.append(match)
        cursor = match_index + 1

    first_return = next(
        (commit for commit in commits[cursor:] if commit.command in REVIEW_COMMANDS),
        None,
    )
    if first_return is None:
        raise ValueError("live review arc is missing scene.restore + w30.apply_damage_profile")
    if first_return.command not in RETURN_COMMANDS:
        raise ValueError(
            f"expected scene.restore + w30.apply_damage_profile, got {first_return.command}"
        )
    return_event = first_return.event_index
    same_event = [
        commit
        for commit in commits[cursor:]
        if commit.event_index == return_event and commit.command in REVIEW_COMMANDS
    ]
    commands = {commit.command for commit in same_event}
    beats = {commit.beat for commit in same_event}
    if commands != RETURN_COMMANDS or len(same_event) != 2 or len(beats) != 1:
        raise ValueError("scene.restore and w30.apply_damage_profile must commit together")
    stages.append(next(commit for commit in same_event if commit.command == "scene.restore"))
    return stages


def find_stop_position(events: list[dict[str, Any]], after_event_index: int) -> float:
    for event in events[after_event_index + 1 :]:
        if (
            event.get("event") == "key_outcome"
            and event.get("key") == "space"
            and event.get("outcome") == "toggle_transport"
        ):
            snapshot = require_object(event.get("snapshot"), "transport-stop snapshot")
            transport = require_object(snapshot.get("transport"), "transport-stop snapshot.transport")
            if transport.get("is_playing") is not False:
                continue
            queue = require_object(snapshot.get("queue"), "transport-stop snapshot.queue")
            if queue.get("pending_count") != 0:
                raise ValueError("review transport stopped with pending actions")
            position = transport.get("position_beats")
            if not isinstance(position, (int, float)) or isinstance(position, bool):
                raise TypeError("transport stop position_beats must be numeric")
            if not math.isfinite(float(position)):
                raise ValueError("transport stop position_beats must be finite")
            return float(position)
    raise ValueError("live review arc has no explicit transport stop")


def build_result(
    stages: list[Commit], stop_position: float, stop_tolerance_beats: float
) -> dict[str, Any]:
    invalid_boundaries = [
        f"{stage.command}={stage.boundary}"
        for stage in stages
        if stage.boundary not in {"Bar", "Phrase"}
    ]
    if invalid_boundaries:
        raise ValueError(
            "live review stages must land on bar/phrase boundaries: "
            + ", ".join(invalid_boundaries)
        )

    positions = [float(stage.beat) for stage in stages] + [stop_position]
    actual_intervals = [positions[index + 1] - positions[index] for index in range(5)]
    failures = []
    for (label, expected), actual in zip(TARGET_INTERVALS, actual_intervals, strict=True):
        tolerance = stop_tolerance_beats if label == "return" else 0.0
        if abs(actual - expected) > tolerance:
            failures.append(
                f"{label}: expected {expected:g} beats, got {actual:.6f} beats"
            )
    if failures:
        raise ValueError("timing arc mismatch: " + "; ".join(failures))

    stage_rows = [
        {
            "command": stage.command,
            "beat": stage.beat,
            "boundary": stage.boundary,
        }
        for stage in stages
    ]
    return {
        "schema": "riotbox.feral_break_live_review_timing.v1",
        "schema_version": 1,
        "result": "pass",
        "target_intervals_beats": [expected for _, expected in TARGET_INTERVALS],
        "actual_intervals_beats": actual_intervals,
        "stages": stage_rows,
        "stop_position_beats": stop_position,
        "stop_tolerance_beats": stop_tolerance_beats,
    }


def require_object(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{label} must be an object")
    return value


def main() -> int:
    args = parse_args()
    try:
        events = load_events(args.events)
        if not args.timing_only:
            validate_observer_events(events)
        result = validate_timing(events, args.stop_tolerance_beats)
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid Feral Break live review timing: {error}", file=sys.stderr)
        return 1

    rendered = json.dumps(result, indent=2, sort_keys=True)
    if args.json_output is not None:
        args.json_output.write_text(rendered + "\n")
    print(
        "valid Feral Break live review timing: "
        + " -> ".join(f"{value:g}" for value in result["actual_intervals_beats"])
        + f" beats ({args.events})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
