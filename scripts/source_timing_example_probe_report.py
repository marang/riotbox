#!/usr/bin/env python3
"""Render a compact Source Timing probe report for local example WAVs.

The example WAVs are intentionally not committed. Missing files are reported as
skipped rows so the command remains safe for CI and fresh clones.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


DEFAULT_SOURCES = [
    "data/test_audio/examples/Beat03_130BPM(Full).wav",
    "data/test_audio/examples/Beat08_128BPM(Full).wav",
    "data/test_audio/examples/Beat20_128BPM(Full).wav",
    "data/test_audio/examples/DH_BeatC_120-01.wav",
    "data/test_audio/examples/DH_BeatC_KickSnr_120-01.wav",
    "data/test_audio/examples/DH_Fadapad_120_A.wav",
    "data/test_audio/examples/DH_RushArp_120_A.wav",
]


@dataclass
class ReportRow:
    source: str
    status: str
    cue: str = "-"
    readiness: str = "-"
    manual_confirm: str = "-"
    bpm: str = "-"
    beat: str = "-"
    downbeat: str = "-"
    phrase: str = "-"
    warnings: str = "-"
    anchors: str = "-"
    groove: str = "-"
    expectation: str = "-"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Render a Markdown Source Timing report for local example WAVs.",
    )
    parser.add_argument(
        "--source",
        action="append",
        default=[],
        help="Source WAV to probe. Defaults to the documented local examples.",
    )
    parser.add_argument(
        "--fixture-json",
        action="append",
        default=[],
        help="Read an existing source_timing_probe --json payload instead of running cargo.",
    )
    parser.add_argument(
        "--output",
        help="Write Markdown report to this path instead of stdout.",
    )
    parser.add_argument(
        "--expectations",
        help="Optional JSON expectations for probed source rows.",
    )
    args = parser.parse_args()

    try:
        expectations = load_expectations(Path(args.expectations)) if args.expectations else {}
        rows = collect_rows(args, expectations)
        report = render_markdown(rows)
        if args.output:
            output_path = Path(args.output)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(report)
        else:
            print(report)
        failures = expectation_failures(rows)
        if failures:
            for failure in failures:
                print(failure, file=sys.stderr)
            return 1
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"source timing example probe report error: {error}", file=sys.stderr)
        return 1

    return 0


def collect_rows(
    args: argparse.Namespace,
    expectations: dict[str, dict[str, Any]],
) -> list[ReportRow]:
    if args.fixture_json:
        return [
            row_from_payload(load_json(Path(path)), expectations)
            for path in args.fixture_json
        ]

    sources = [Path(path) for path in (args.source or DEFAULT_SOURCES)]
    rows = []
    for source in sources:
        if not source.exists():
            rows.append(missing_row(source, expectations))
            continue
        payload = run_probe(source)
        rows.append(row_from_payload(payload, expectations))
    return rows


def run_probe(source: Path) -> dict[str, Any]:
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "riotbox-audio",
        "--bin",
        "source_timing_probe",
        "--",
        "--json",
        str(source),
    ]
    completed = subprocess.run(
        command,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    try:
        return require_object(json.loads(completed.stdout), "probe output")
    except json.JSONDecodeError as error:
        raise ValueError(f"invalid probe JSON for {source}: {error}") from error


def load_json(path: Path) -> dict[str, Any]:
    return require_object(json.loads(path.read_text()), str(path))


def load_expectations(path: Path) -> dict[str, dict[str, Any]]:
    payload = require_object(json.loads(path.read_text()), str(path))
    sources = require_object(payload.get("sources"), "expectations.sources")
    expectations = {}
    for source, expectation in sources.items():
        if not isinstance(source, str) or not source:
            raise TypeError("expectation source keys must be non-empty strings")
        expectations[source] = require_object(expectation, f"expectations[{source}]")
    return expectations


def missing_row(source: Path, expectations: dict[str, dict[str, Any]]) -> ReportRow:
    expectation = "skipped" if source.name in expectations else "-"
    return ReportRow(source=source.name, status="missing", expectation=expectation)


def row_from_payload(
    payload: dict[str, Any],
    expectations: dict[str, dict[str, Any]],
) -> ReportRow:
    source_path = require_string(payload, "source_path")
    warnings = require_string_list(payload, "warning_codes")
    anchors = require_object(payload.get("anchor_evidence"), "anchor_evidence")
    groove = require_object(payload.get("groove_evidence"), "groove_evidence")
    source_name = Path(source_path).name

    return ReportRow(
        source=source_name,
        status="probed",
        cue=require_string(payload, "cue"),
        readiness=require_string(payload, "readiness"),
        manual_confirm=yes_no(require_bool(payload, "requires_manual_confirm")),
        bpm=format_bpm(payload.get("primary_bpm")),
        beat=require_string(payload, "beat_status"),
        downbeat=require_string(payload, "downbeat_status"),
        phrase=require_string(payload, "phrase_status"),
        warnings="none" if not warnings else ",".join(warnings),
        anchors=format_anchors(anchors),
        groove=str(require_int(groove, "primary_groove_residual_count")),
        expectation=format_expectation(source_name, payload, expectations),
    )


def render_markdown(rows: list[ReportRow]) -> str:
    lines = [
        "# Source Timing Example Probe Report",
        "",
        "Missing rows mean the local example WAV is not present in this checkout.",
        "",
        "| Source | Status | Cue | Readiness | Manual confirm | BPM | Beat | Downbeat | Phrase | Warnings | Anchors total/kick/backbeat/transient | Groove residuals | Expectation |",
        "| --- | --- | --- | --- | --- | ---: | --- | --- | --- | --- | --- | ---: | --- |",
    ]
    for row in rows:
        lines.append(
            "| "
            + " | ".join(
                [
                    row.source,
                    row.status,
                    row.cue,
                    row.readiness,
                    row.manual_confirm,
                    row.bpm,
                    row.beat,
                    row.downbeat,
                    row.phrase,
                    row.warnings,
                    row.anchors,
                    row.groove,
                    row.expectation,
                ]
            )
            + " |"
        )
    lines.append("")
    return "\n".join(lines)


def format_expectation(
    source_name: str,
    payload: dict[str, Any],
    expectations: dict[str, dict[str, Any]],
) -> str:
    expectation = expectations.get(source_name)
    if expectation is None:
        return "-"

    issues = expectation_issues(payload, expectation)
    if not issues:
        return "ok"
    return "mismatch: " + "; ".join(issues)


def expectation_issues(payload: dict[str, Any], expectation: dict[str, Any]) -> list[str]:
    issues = []
    compare_string(payload, expectation, issues, "cue")
    compare_string(payload, expectation, issues, "readiness")
    compare_bool(payload, expectation, issues, "requires_manual_confirm")
    compare_string(payload, expectation, issues, "beat_status")
    compare_string(payload, expectation, issues, "downbeat_status")
    compare_string(payload, expectation, issues, "phrase_status")
    compare_bpm(payload, expectation, issues)
    compare_warning_includes(payload, expectation, issues)
    return issues


def compare_string(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
    key: str,
) -> None:
    if key not in expectation:
        return
    expected = require_string(expectation, key)
    actual = require_string(payload, key)
    if actual != expected:
        issues.append(f"{key} expected {expected!r} got {actual!r}")


def compare_bool(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
    key: str,
) -> None:
    if key not in expectation:
        return
    expected = require_bool(expectation, key)
    actual = require_bool(payload, key)
    if actual != expected:
        issues.append(f"{key} expected {expected!r} got {actual!r}")


def compare_bpm(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
) -> None:
    if "primary_bpm" not in expectation:
        return
    expected = require_object(expectation["primary_bpm"], "primary_bpm expectation")
    target = require_number(expected, "target")
    tolerance = require_number(expected, "tolerance")
    if tolerance < 0:
        raise ValueError("primary_bpm tolerance must be non-negative")
    actual = payload.get("primary_bpm")
    if type(actual) not in {int, float}:
        issues.append("primary_bpm expected numeric value got none")
        return
    delta = abs(float(actual) - target)
    if delta > tolerance:
        issues.append(
            f"primary_bpm delta {delta:.6f} exceeds tolerance {tolerance:.6f}"
        )


def compare_warning_includes(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
) -> None:
    if "warning_codes_include" not in expectation:
        return
    expected_warnings = require_string_list(expectation, "warning_codes_include")
    actual_warnings = set(require_string_list(payload, "warning_codes"))
    for warning in expected_warnings:
        if warning not in actual_warnings:
            issues.append(f"warning_codes missing {warning!r}")


def expectation_failures(rows: list[ReportRow]) -> list[str]:
    return [
        f"{row.source}: {row.expectation}"
        for row in rows
        if row.expectation.startswith("mismatch:")
    ]


def format_bpm(value: Any) -> str:
    if value is None:
        return "none"
    if not isinstance(value, (int, float)):
        raise TypeError("primary_bpm must be a number or null")
    return f"{value:.3f}"


def format_anchors(value: dict[str, Any]) -> str:
    return "/".join(
        str(require_int(value, key))
        for key in [
            "primary_anchor_count",
            "primary_kick_anchor_count",
            "primary_backbeat_anchor_count",
            "primary_transient_anchor_count",
        ]
    )


def yes_no(value: bool) -> str:
    return "yes" if value else "no"


def require_object(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{label} must be an object")
    return value


def require_string(value: dict[str, Any], key: str) -> str:
    item = value.get(key)
    if not isinstance(item, str) or not item:
        raise TypeError(f"{key} must be a non-empty string")
    return item


def require_bool(value: dict[str, Any], key: str) -> bool:
    item = value.get(key)
    if not isinstance(item, bool):
        raise TypeError(f"{key} must be a boolean")
    return item


def require_number(value: dict[str, Any], key: str) -> float:
    item = value.get(key)
    if type(item) not in {int, float}:
        raise TypeError(f"{key} must be a number")
    return float(item)


def require_int(value: dict[str, Any], key: str) -> int:
    item = value.get(key)
    if type(item) is not int or item < 0:
        raise TypeError(f"{key} must be a non-negative integer")
    return item


def require_string_list(value: dict[str, Any], key: str) -> list[str]:
    item = value.get(key)
    if not isinstance(item, list) or any(
        not isinstance(entry, str) or not entry for entry in item
    ):
        raise TypeError(f"{key} must be a list of non-empty strings")
    return item


if __name__ == "__main__":
    raise SystemExit(main())
