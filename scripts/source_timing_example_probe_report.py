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
    args = parser.parse_args()

    try:
        rows = collect_rows(args)
        report = render_markdown(rows)
        if args.output:
            output_path = Path(args.output)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(report)
        else:
            print(report)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"source timing example probe report error: {error}", file=sys.stderr)
        return 1

    return 0


def collect_rows(args: argparse.Namespace) -> list[ReportRow]:
    if args.fixture_json:
        return [row_from_payload(load_json(Path(path))) for path in args.fixture_json]

    sources = [Path(path) for path in (args.source or DEFAULT_SOURCES)]
    rows = []
    for source in sources:
        if not source.exists():
            rows.append(ReportRow(source=source.name, status="missing"))
            continue
        payload = run_probe(source)
        rows.append(row_from_payload(payload))
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


def row_from_payload(payload: dict[str, Any]) -> ReportRow:
    source_path = require_string(payload, "source_path")
    warnings = require_string_list(payload, "warning_codes")
    anchors = require_object(payload.get("anchor_evidence"), "anchor_evidence")
    groove = require_object(payload.get("groove_evidence"), "groove_evidence")

    return ReportRow(
        source=Path(source_path).name,
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
    )


def render_markdown(rows: list[ReportRow]) -> str:
    lines = [
        "# Source Timing Example Probe Report",
        "",
        "Missing rows mean the local example WAV is not present in this checkout.",
        "",
        "| Source | Status | Cue | Readiness | Manual confirm | BPM | Beat | Downbeat | Phrase | Warnings | Anchors total/kick/backbeat/transient | Groove residuals |",
        "| --- | --- | --- | --- | --- | ---: | --- | --- | --- | --- | --- | ---: |",
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
                ]
            )
            + " |"
        )
    lines.append("")
    return "\n".join(lines)


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
