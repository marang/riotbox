#!/usr/bin/env python3
"""Validate the long stable Source Timing locked-grid probe fixture."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Callable


def die(message: str) -> None:
    raise SystemExit(message)


def require_equal(summary: dict[str, Any], key: str, expected: Any) -> None:
    actual = summary.get(key)
    if actual != expected:
        die(f"{key} must be {expected!r}, got {actual!r}")


def require_number(
    summary: dict[str, Any],
    key: str,
    predicate: Callable[[float], bool],
    description: str,
) -> None:
    value = summary.get(key)
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        die(f"{key} must be a number, got {value!r}")
    if not predicate(float(value)):
        die(f"{key} must be {description}, got {value!r}")


def require_int(
    summary: dict[str, Any],
    key: str,
    predicate: Callable[[int], bool],
    description: str,
) -> None:
    value = summary.get(key)
    if not isinstance(value, int) or isinstance(value, bool):
        die(f"{key} must be an integer, got {value!r}")
    if not predicate(value):
        die(f"{key} must be {description}, got {value!r}")


def main() -> int:
    if len(sys.argv) != 2:
        print(
            "usage: validate_source_timing_locked_grid_fixture.py <probe.json>",
            file=sys.stderr,
        )
        return 2

    path = Path(sys.argv[1])
    summary = json.loads(path.read_text(encoding="utf-8"))

    expected = {
        "cue": "grid locked",
        "readiness": "ready",
        "requires_manual_confirm": False,
        "grid_use": "locked_grid",
        "beat_status": "stable",
        "downbeat_status": "stable",
        "confidence_result": "candidate_cautious",
        "drift_status": "stable",
        "phrase_status": "stable",
        "primary_downbeat_offset_beats": 0,
        "warning_codes": [],
    }
    for key, value in expected.items():
        require_equal(summary, key, value)

    require_number(
        summary,
        "primary_bpm",
        lambda value: 127.0 <= value <= 130.0,
        "127..130 BPM",
    )
    require_number(
        summary,
        "duration_seconds",
        lambda value: value >= 15.9,
        "at least 15.9",
    )
    require_number(
        summary,
        "primary_beat_score",
        lambda value: value >= 0.9,
        "at least 0.9",
    )
    require_number(
        summary,
        "primary_downbeat_score",
        lambda value: value >= 0.3,
        "at least 0.3",
    )
    require_int(summary, "alternate_evidence_count", lambda value: value == 0, "zero")
    require_int(summary, "alternate_beat_candidate_count", lambda value: value == 0, "zero")
    require_int(summary, "alternate_downbeat_phase_count", lambda value: value == 0, "zero")

    anchors = summary.get("anchor_evidence")
    if not isinstance(anchors, dict):
        die("anchor_evidence must be an object")
    require_int(anchors, "primary_anchor_count", lambda value: value > 0, "positive")
    require_int(
        anchors,
        "primary_kick_anchor_count",
        lambda value: value > 0,
        "positive",
    )
    require_int(
        anchors,
        "primary_backbeat_anchor_count",
        lambda value: value > 0,
        "positive",
    )
    require_int(
        anchors,
        "primary_transient_anchor_count",
        lambda value: value >= 0,
        "non-negative",
    )

    groove = summary.get("groove_evidence")
    if not isinstance(groove, dict):
        die("groove_evidence must be an object")
    require_int(
        groove,
        "primary_groove_residual_count",
        lambda value: value > 0,
        "positive",
    )
    require_number(
        groove,
        "primary_max_abs_offset_ms",
        lambda value: 0.0 <= value <= 15.0,
        "inside 0..15 ms",
    )

    print(
        "locked-grid source timing fixture ok: "
        f"bpm={summary['primary_bpm']:.3f} "
        f"anchors={anchors['primary_anchor_count']} "
        f"groove={groove['primary_groove_residual_count']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
