#!/usr/bin/env python3
"""Render a compact Source Timing probe report for local example WAVs.

The example WAVs are intentionally not committed. Missing files are reported as
skipped rows so the command remains safe for CI and fresh clones.
"""

from __future__ import annotations

import argparse
import json
import math
import subprocess
import sys
import wave
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from source_timing_example_expectations import format_expectation, load_expectations


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
    duration_seconds: str = "-"
    sample_rate: str = "-"
    channels: str = "-"
    audio_rms_dbfs: str = "-"
    audio_peak_abs: str = "-"
    zero_crossings_per_second: str = "-"
    cue: str = "-"
    actionability: str = "-"
    readiness: str = "-"
    manual_confirm: str = "-"
    grid_use: str = "-"
    bpm: str = "-"
    confidence: str = "-"
    drift: str = "-"
    beat: str = "-"
    beat_count: str = "-"
    bar_count: str = "-"
    beat_score: str = "-"
    beat_match: str = "-"
    beat_median: str = "-"
    beat_alternates: str = "-"
    downbeat: str = "-"
    downbeat_offset: str = "-"
    downbeat_score: str = "-"
    downbeat_margin: str = "-"
    downbeat_alternates: str = "-"
    phrase: str = "-"
    phrase_count: str = "-"
    phrase_bars: str = "-"
    alternate_evidence: str = "-"
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
    except (OSError, TypeError, ValueError, subprocess.CalledProcessError) as error:
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
    audio = audio_descriptor_fields(Path(source_path))

    return ReportRow(
        source=source_name,
        status="probed",
        duration_seconds=audio["duration_seconds"],
        sample_rate=audio["sample_rate"],
        channels=audio["channels"],
        audio_rms_dbfs=audio["audio_rms_dbfs"],
        audio_peak_abs=audio["audio_peak_abs"],
        zero_crossings_per_second=audio["zero_crossings_per_second"],
        cue=require_string(payload, "cue"),
        actionability=require_string(payload, "actionability"),
        readiness=require_string(payload, "readiness"),
        manual_confirm=yes_no(require_bool(payload, "requires_manual_confirm")),
        grid_use=require_string(payload, "grid_use"),
        bpm=format_bpm(payload.get("primary_bpm")),
        confidence=require_string(payload, "confidence_result"),
        drift=require_string(payload, "drift_status"),
        beat=require_string(payload, "beat_status"),
        beat_count=str(require_int(payload, "primary_beat_count")),
        bar_count=str(require_int(payload, "primary_bar_count")),
        beat_score=format_optional_score(payload.get("primary_beat_score")),
        beat_match=format_optional_score(payload.get("primary_beat_matched_onset_ratio")),
        beat_median=format_optional_score(payload.get("primary_beat_median_distance_ratio")),
        beat_alternates=str(require_int(payload, "alternate_beat_candidate_count")),
        downbeat=require_string(payload, "downbeat_status"),
        downbeat_offset=format_optional_int(payload.get("primary_downbeat_offset_beats")),
        downbeat_score=format_optional_score(payload.get("primary_downbeat_score")),
        downbeat_margin=format_optional_score(payload.get("primary_downbeat_margin")),
        downbeat_alternates=str(require_int(payload, "alternate_downbeat_phase_count")),
        phrase=require_string(payload, "phrase_status"),
        phrase_count=str(require_int(payload, "primary_phrase_count")),
        phrase_bars=str(require_int(payload, "primary_phrase_bar_count")),
        alternate_evidence=str(require_int(payload, "alternate_evidence_count")),
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
        "| Source | Status | Duration s | Sample rate | Channels | Audio RMS dBFS | Audio peak | ZCR/s | Cue | Action | Readiness | Manual confirm | Grid use | BPM | Confidence | Drift | Beat | Beat count | Bar count | Beat score | Beat match | Beat median | Beat alts | Downbeat | Downbeat offset | Downbeat score | Downbeat margin | Downbeat alts | Phrase | Phrase count | Phrase bars | Alternate evidence | Warnings | Anchors total/kick/backbeat/transient | Groove residuals | Expectation |",
        "| --- | --- | --- | --- | --- | --- | --- | ---: | --- | --- | --- | --- | --- | --- | --- | --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | --- | --- | ---: | --- |",
    ]
    for row in rows:
        lines.append(
            "| "
            + " | ".join(
                [
                    row.source,
                    row.status,
                    row.duration_seconds,
                    row.sample_rate,
                    row.channels,
                    row.audio_rms_dbfs,
                    row.audio_peak_abs,
                    row.zero_crossings_per_second,
                    row.cue,
                    row.actionability,
                    row.readiness,
                    row.manual_confirm,
                    row.grid_use,
                    row.bpm,
                    row.confidence,
                    row.drift,
                    row.beat,
                    row.beat_count,
                    row.bar_count,
                    row.beat_score,
                    row.beat_match,
                    row.beat_median,
                    row.beat_alternates,
                    row.downbeat,
                    row.downbeat_offset,
                    row.downbeat_score,
                    row.downbeat_margin,
                    row.downbeat_alternates,
                    row.phrase,
                    row.phrase_count,
                    row.phrase_bars,
                    row.alternate_evidence,
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


def audio_descriptor_fields(source: Path) -> dict[str, str]:
    if not source.exists():
        return {
            "duration_seconds": "-",
            "sample_rate": "-",
            "channels": "-",
            "audio_rms_dbfs": "-",
            "audio_peak_abs": "-",
            "zero_crossings_per_second": "-",
        }

    with wave.open(str(source), "rb") as wav:
        sample_rate = wav.getframerate()
        channels = wav.getnchannels()
        sample_width = wav.getsampwidth()
        frame_count = wav.getnframes()
        frames = wav.readframes(frame_count)

    sample_sum_sq = 0.0
    peak_abs = 0.0
    zero_crossings = 0
    previous: float | None = None
    mono_count = 0

    for frame in pcm_frames(frames, sample_width, channels):
        mono = sum(frame) / len(frame)
        sample_sum_sq += mono * mono
        peak_abs = max(peak_abs, abs(mono))
        if previous is not None and ((previous < 0.0 <= mono) or (previous >= 0.0 > mono)):
            zero_crossings += 1
        previous = mono
        mono_count += 1

    duration = frame_count / sample_rate if sample_rate else 0.0
    rms = math.sqrt(sample_sum_sq / mono_count) if mono_count else 0.0
    dbfs = 20.0 * math.log10(rms + 1e-12)
    zcr_per_second = zero_crossings / duration if duration > 0.0 else 0.0
    return {
        "duration_seconds": f"{duration:.3f}",
        "sample_rate": str(sample_rate),
        "channels": str(channels),
        "audio_rms_dbfs": f"{dbfs:.1f}",
        "audio_peak_abs": f"{peak_abs:.3f}",
        "zero_crossings_per_second": f"{zcr_per_second:.1f}",
    }


def pcm_frames(raw: bytes, sample_width: int, channels: int) -> list[tuple[float, ...]]:
    if channels <= 0:
        raise ValueError("WAV channel count must be positive")
    if sample_width not in {2, 3, 4}:
        raise ValueError(f"unsupported WAV sample width: {sample_width}")

    stride = sample_width * channels
    frames: list[tuple[float, ...]] = []
    for offset in range(0, len(raw) - stride + 1, stride):
        values = []
        for channel in range(channels):
            start = offset + channel * sample_width
            sample = pcm_sample(raw[start : start + sample_width], sample_width)
            values.append(sample)
        frames.append(tuple(values))
    return frames


def pcm_sample(raw: bytes, sample_width: int) -> float:
    if sample_width == 2:
        return int.from_bytes(raw, "little", signed=True) / 32768.0
    if sample_width == 3:
        unsigned = int.from_bytes(raw, "little", signed=False)
        if unsigned & 0x800000:
            unsigned -= 0x1000000
        return unsigned / 8388608.0
    return int.from_bytes(raw, "little", signed=True) / 2147483648.0


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


def format_optional_score(value: Any) -> str:
    if value is None:
        return "none"
    if not isinstance(value, (int, float)):
        raise TypeError("score fields must be numbers or null")
    return f"{value:.3f}"


def format_optional_int(value: Any) -> str:
    if value is None:
        return "none"
    if type(value) is not int or value < 0:
        raise TypeError("integer fields must be non-negative integers or null")
    return str(value)


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
