#!/usr/bin/env python3
"""Assert committed Source Timing example report fixtures by report field."""

from __future__ import annotations

from pathlib import Path
from typing import Callable

from source_timing_example_expectations import load_expectations
from source_timing_example_probe_report import (
    ReportRow,
    expectation_failures,
    load_json,
    missing_row,
    render_markdown,
    row_from_payload,
)
from validate_source_timing_probe_json import validate_summary


FIXTURE_DIR = Path("scripts/fixtures/source_timing_example_probe_report")
AUDIO_FIXTURE_DIR = Path("crates/riotbox-audio/tests/fixtures/source_timing_probe")
POSITIVE_PAYLOAD_PATHS = [
    FIXTURE_DIR / "beat08_source_timing_probe.json",
    AUDIO_FIXTURE_DIR / "probe_valid_locked_grid.json",
    FIXTURE_DIR / "probe_weak_ambiguous_flat_pulse.json",
    FIXTURE_DIR / "probe_unavailable_silence.json",
]


EXPECTED_ROWS = {
    "Beat08_128BPM(Full).wav": {
        "status": "probed",
        "cue": "needs confirm",
        "readiness": "needs_review",
        "manual_confirm": "yes",
        "grid_use": "short_loop_manual_confirm",
        "bpm": "128.397",
        "confidence": "candidate_cautious",
        "drift": "not_enough_material",
        "beat": "stable",
        "beat_score": "0.979",
        "beat_match": "0.920",
        "beat_median": "0.990",
        "beat_alternates": "0",
        "downbeat": "stable",
        "downbeat_score": "0.565",
        "downbeat_alternates": "0",
        "phrase": "not_enough_material",
        "alternate_evidence": "0",
        "warnings": "phrase_uncertain",
        "anchors": "9/2/4/3",
        "groove": "1",
        "expectation": "ok",
    },
    "long_stable_lock.wav": {
        "status": "probed",
        "cue": "grid locked",
        "readiness": "ready",
        "manual_confirm": "no",
        "grid_use": "locked_grid",
        "bpm": "128.397",
        "confidence": "candidate_cautious",
        "drift": "stable",
        "beat": "stable",
        "beat_score": "0.979",
        "beat_match": "1.000",
        "beat_median": "0.667",
        "beat_alternates": "0",
        "downbeat": "stable",
        "downbeat_score": "0.565",
        "downbeat_alternates": "0",
        "phrase": "stable",
        "alternate_evidence": "0",
        "warnings": "none",
        "anchors": "11/6/3/2",
        "groove": "4",
        "expectation": "ok",
    },
    "flat_pulse_ambiguous.wav": {
        "status": "probed",
        "cue": "needs confirm",
        "readiness": "weak",
        "manual_confirm": "yes",
        "grid_use": "manual_confirm_only",
        "bpm": "128.397",
        "confidence": "candidate_ambiguous",
        "drift": "stable",
        "beat": "stable",
        "beat_score": "0.977",
        "beat_match": "1.000",
        "beat_median": "0.006",
        "beat_alternates": "0",
        "downbeat": "weak",
        "downbeat_score": "0.268",
        "downbeat_alternates": "3",
        "phrase": "ambiguous_downbeat",
        "alternate_evidence": "6",
        "warnings": "phrase_uncertain,ambiguous_downbeat",
        "anchors": "16/0/0/16",
        "groove": "4",
        "expectation": "-",
    },
    "silence.wav": {
        "status": "probed",
        "cue": "needs confirm",
        "readiness": "unavailable",
        "manual_confirm": "yes",
        "grid_use": "unavailable",
        "bpm": "none",
        "confidence": "degraded",
        "drift": "unavailable",
        "beat": "unavailable",
        "beat_score": "none",
        "beat_match": "none",
        "beat_median": "none",
        "beat_alternates": "0",
        "downbeat": "unavailable",
        "downbeat_score": "none",
        "downbeat_alternates": "0",
        "phrase": "unavailable",
        "alternate_evidence": "0",
        "warnings": "low_timing_confidence,weak_kick_anchor",
        "anchors": "0/0/0/0",
        "groove": "0",
        "expectation": "-",
    },
}


def main() -> int:
    rows = load_positive_rows()
    assert_rows(rows)
    assert_markdown_renders(rows)
    assert_missing_expected_source_skips()
    assert_mismatch_expectations_fail()
    assert_invalid_expectations_fail()
    return 0


def load_positive_rows() -> list[ReportRow]:
    expectations = load_expectations(FIXTURE_DIR / "beat08_expectations.json")
    rows = []
    for path in POSITIVE_PAYLOAD_PATHS:
        payload = load_json(path)
        validate_summary(payload)
        rows.append(row_from_payload(payload, expectations))
    return rows


def assert_rows(rows: list[ReportRow]) -> None:
    by_source = {row.source: row for row in rows}
    if set(by_source) != set(EXPECTED_ROWS):
        raise AssertionError(f"unexpected sources: {sorted(by_source)}")

    for source, expected_fields in EXPECTED_ROWS.items():
        row = by_source[source]
        for field, expected in expected_fields.items():
            actual = getattr(row, field)
            if actual != expected:
                raise AssertionError(
                    f"{source}.{field} expected {expected!r} got {actual!r}"
                )


def assert_markdown_renders(rows: list[ReportRow]) -> None:
    markdown = render_markdown(rows)
    for source in EXPECTED_ROWS:
        if f"| {source} |" not in markdown:
            raise AssertionError(f"rendered Markdown missing {source}")


def assert_missing_expected_source_skips() -> None:
    expectations = load_expectations(FIXTURE_DIR / "beat08_expectations.json")
    row = missing_row(
        Path("/tmp/riotbox-missing/Beat08_128BPM(Full).wav"),
        expectations,
    )
    if row.status != "missing" or row.expectation != "skipped":
        raise AssertionError(
            "expected missing source with expectations to render as missing/skipped"
        )
    markdown = render_markdown([row])
    expected_fragment = "| Beat08_128BPM(Full).wav | missing |"
    if expected_fragment not in markdown:
        raise AssertionError("rendered Markdown missing expected missing-source row")


def assert_mismatch_expectations_fail() -> None:
    expectations = load_expectations(FIXTURE_DIR / "beat08_expectations_mismatch.json")
    row = row_from_payload(
        load_json(FIXTURE_DIR / "beat08_source_timing_probe.json"),
        expectations,
    )
    failures = expectation_failures([row])
    if not failures or "mismatch:" not in failures[0]:
        raise AssertionError("expected mismatched source timing expectations to fail")


def assert_invalid_expectations_fail() -> None:
    invalid_paths = [
        FIXTURE_DIR / "beat08_expectations_invalid_empty_range.json",
        FIXTURE_DIR / "beat08_expectations_invalid_inverted_range.json",
        FIXTURE_DIR / "beat08_expectations_invalid_unknown_range_key.json",
        FIXTURE_DIR / "beat08_expectations_invalid_unknown_key.json",
    ]
    for path in invalid_paths:
        assert_raises(
            lambda path=path: row_from_payload(
                load_json(FIXTURE_DIR / "beat08_source_timing_probe.json"),
                load_expectations(path),
            ),
            f"expected invalid source timing expectation range fixture to fail: {path}",
        )


def assert_raises(callback: Callable[[], object], message: str) -> None:
    try:
        callback()
    except (TypeError, ValueError):
        return
    raise AssertionError(message)


if __name__ == "__main__":
    raise SystemExit(main())
