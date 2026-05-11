#!/usr/bin/env python3
"""Validate the short-loop manual-confirm Source Timing probe fixture."""

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


def require_warning(summary: dict[str, Any], warning: str) -> None:
    warnings = summary.get("warning_codes")
    if not isinstance(warnings, list) or warning not in warnings:
        die(f"warning_codes must include {warning!r}, got {warnings!r}")


def main() -> int:
    if len(sys.argv) != 2:
        print(
            "usage: validate_source_timing_short_loop_fixture.py <probe.json>",
            file=sys.stderr,
        )
        return 2

    path = Path(sys.argv[1])
    summary = json.loads(path.read_text(encoding="utf-8"))

    expected = {
        "cue": "needs confirm",
        "readiness": "needs_review",
        "requires_manual_confirm": True,
        "grid_use": "short_loop_manual_confirm",
        "beat_status": "stable",
        "downbeat_status": "stable",
        "confidence_result": "candidate_cautious",
        "drift_status": "not_enough_material",
        "phrase_status": "not_enough_material",
        "alternate_evidence_count": 0,
    }
    for key, value in expected.items():
        require_equal(summary, key, value)

    require_warning(summary, "phrase_uncertain")
    require_number(
        summary,
        "primary_bpm",
        lambda value: 127.0 <= value <= 130.0,
        "127..130 BPM",
    )
    require_number(
        summary,
        "duration_seconds",
        lambda value: 3.0 <= value <= 8.0,
        "3..8 seconds",
    )

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

    groove = summary.get("groove_evidence")
    if not isinstance(groove, dict):
        die("groove_evidence must be an object")
    require_int(
        groove,
        "primary_groove_residual_count",
        lambda value: value >= 0,
        "non-negative",
    )

    print(
        "short-loop source timing fixture ok: "
        f"bpm={summary['primary_bpm']:.3f} "
        f"anchors={anchors['primary_anchor_count']} "
        f"warnings={','.join(summary['warning_codes'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
