#!/usr/bin/env python3
"""Reject exact-live W-30 template collapse across a bounded source matrix."""

from __future__ import annotations

import argparse
import json
import math
import sys
import wave
from array import array
from pathlib import Path


MAX_ENVELOPE_CORRELATION = 0.95
ENVELOPE_WINDOW_MS = 10
ENVELOPE_POINT_COUNT = 512


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("matrix_dir", type=Path)
    parser.add_argument("case_ids", nargs="+")
    return parser.parse_args()


def mono_envelope(path: Path) -> list[float]:
    with wave.open(str(path), "rb") as wav:
        if wav.getsampwidth() != 2:
            raise ValueError(f"{path} must be PCM16")
        channel_count = wav.getnchannels()
        sample_rate = wav.getframerate()
        samples = array("h")
        samples.frombytes(wav.readframes(wav.getnframes()))
        if sys.byteorder != "little":
            samples.byteswap()
    if channel_count <= 0 or not samples:
        raise ValueError(f"{path} contains no complete audio frames")

    window_frames = max(1, sample_rate * ENVELOPE_WINDOW_MS // 1_000)
    envelope = []
    frame_count = len(samples) // channel_count
    for start in range(0, frame_count - window_frames + 1, window_frames):
        square_sum = 0.0
        for frame in range(start, start + window_frames):
            base = frame * channel_count
            mono = sum(samples[base : base + channel_count]) / (
                channel_count * 32_768.0
            )
            square_sum += mono * mono
        envelope.append(math.sqrt(square_sum / window_frames))
    if len(envelope) < 8:
        raise ValueError(f"{path} is too short for envelope comparison")
    return normalize(resample(envelope, ENVELOPE_POINT_COUNT))


def resample(values: list[float], point_count: int) -> list[float]:
    if point_count < 2:
        raise ValueError("resampled envelope requires at least two points")
    last = len(values) - 1
    result = []
    for index in range(point_count):
        position = index * last / (point_count - 1)
        lower = int(position)
        upper = min(lower + 1, last)
        fraction = position - lower
        result.append(values[lower] * (1.0 - fraction) + values[upper] * fraction)
    return result


def normalize(values: list[float]) -> list[float]:
    mean = sum(values) / len(values)
    variance = sum((value - mean) ** 2 for value in values) / len(values)
    stddev = math.sqrt(variance)
    if stddev <= 1.0e-12:
        raise ValueError("audio envelope has no usable movement")
    return [(value - mean) / stddev for value in values]


def correlation(left: list[float], right: list[float]) -> float:
    if len(left) != len(right):
        raise ValueError("normalized envelope lengths differ")
    return sum(a * b for a, b in zip(left, right, strict=True)) / len(left)


def main() -> int:
    args = parse_args()
    if len(args.case_ids) < 3:
        raise ValueError("source matrix requires at least three contrasting cases")
    if len(set(args.case_ids)) != len(args.case_ids):
        raise ValueError("source matrix case IDs must be unique")
    source_hashes: set[str] = set()
    envelopes: dict[str, list[float]] = {}
    cases = []
    for case_id in args.case_ids:
        case_dir = args.matrix_dir / case_id
        manifest_path = case_dir / "gesture-manifest.json"
        manifest = json.loads(manifest_path.read_text())
        if manifest.get("result") != "pass":
            raise ValueError(f"{case_id} exact-live manifest did not pass")
        source_hash = manifest["source"]["content_hash"]
        if source_hash in source_hashes:
            raise ValueError(f"{case_id} reused a source already present in the matrix")
        source_hashes.add(source_hash)
        hook_path = case_dir / "stems/01_w30_hook.wav"
        envelopes[case_id] = mono_envelope(hook_path)
        cases.append(
            {
                "case_id": case_id,
                "source_hash": source_hash,
                "hook_artifact": str(hook_path.relative_to(args.matrix_dir)),
            }
        )

    comparisons = []
    maximum_correlation = 0.0
    ids = list(envelopes)
    for left_index, left_id in enumerate(ids):
        for right_id in ids[left_index + 1 :]:
            value = correlation(envelopes[left_id], envelopes[right_id])
            maximum_correlation = max(maximum_correlation, abs(value))
            comparisons.append(
                {
                    "left": left_id,
                    "right": right_id,
                    "envelope_correlation": value,
                }
            )
    if maximum_correlation >= MAX_ENVELOPE_CORRELATION:
        raise ValueError(
            "W-30 source matrix collapsed toward one timing template: "
            f"maximum envelope correlation {maximum_correlation:.6f}"
        )

    report = {
        "schema": "riotbox.dense_break_live_source_matrix.v1",
        "evidence_role": "diagnostic",
        "quality_proof": False,
        "human_verdict": "unverified",
        "result": "pass",
        "case_count": len(cases),
        "cases": cases,
        "comparisons": comparisons,
        "maximum_envelope_correlation": maximum_correlation,
        "thresholds": {
            "maximum_envelope_correlation_exclusive": MAX_ENVELOPE_CORRELATION,
            "envelope_window_ms": ENVELOPE_WINDOW_MS,
            "normalized_envelope_point_count": ENVELOPE_POINT_COUNT,
        },
    }
    report_path = args.matrix_dir / "source-matrix-report.json"
    report_path.write_text(json.dumps(report, indent=2) + "\n")
    print(
        f"source matrix pass: {len(cases)} cases, "
        f"maximum envelope correlation {maximum_correlation:.6f}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
