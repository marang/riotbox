#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import math
import wave
from array import array
from pathlib import Path

MAX_CROSS_SOURCE_ENVELOPE_CORRELATION = 0.95
MIN_CROSS_SOURCE_ENVELOPE_MEAN_ABSOLUTE_DELTA = 0.01
REVIEW_DURATION_TOLERANCE_SECONDS = 0.01


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path)
    return parser.parse_args()


def load_manifest(case_dir: Path) -> dict:
    with (case_dir / "controlled-source-manifest.json").open() as handle:
        return json.load(handle)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def rms_envelope(path: Path) -> tuple[list[float], float]:
    with wave.open(str(path), "rb") as source:
        channel_count = source.getnchannels()
        sample_rate = source.getframerate()
        frame_count = source.getnframes()
        samples = array("h", source.readframes(frame_count))
    mono = [
        sum(samples[index : index + channel_count]) / (32768.0 * channel_count)
        for index in range(0, len(samples), channel_count)
    ]
    window_frames = max(1, sample_rate // 50)
    envelope = []
    for start in range(0, len(mono), window_frames):
        window = mono[start : start + window_frames]
        if window:
            envelope.append(math.sqrt(sum(value * value for value in window) / len(window)))
    return envelope, frame_count / sample_rate


def correlation(left: list[float], right: list[float]) -> float:
    count = min(len(left), len(right))
    if count == 0:
        return 1.0
    left = left[:count]
    right = right[:count]
    left_mean = sum(left) / count
    right_mean = sum(right) / count
    covariance = sum(
        (left_value - left_mean) * (right_value - right_mean)
        for left_value, right_value in zip(left, right)
    )
    left_variance = sum((value - left_mean) ** 2 for value in left)
    right_variance = sum((value - right_mean) ** 2 for value in right)
    denominator = math.sqrt(left_variance * right_variance)
    return covariance / denominator if denominator > 0.0 else 1.0


def mean_absolute_delta(left: list[float], right: list[float]) -> float:
    count = min(len(left), len(right))
    if count == 0:
        return 0.0
    return sum(abs(a - b) for a, b in zip(left[:count], right[:count])) / count


def validate_case(manifest: dict, expected_character: str) -> None:
    policy = manifest["source_character_policy"]
    assert manifest["result"] == "pass"
    assert manifest["quality_proof"] is False
    assert manifest["human_verdict"] == "unverified"
    assert policy["character"] == expected_character
    assert policy["bass_owner"] == "unassigned"
    assert policy["source_evidence"]["confidence"] >= 0.35
    if expected_character == "tonal_hook":
        assert policy["lead"] == "w30_hook"
        assert policy["mc202_intent"] == "stay_out"
        assert manifest["metrics"]["mc202"]["rms"] <= manifest["thresholds"][
            "max_intentional_stay_out_rms"
        ]
        assert policy["destructive_intent"] == "pitch_drag"
        assert policy["resolved_live_defaults"]["damage_playback_rate"] < 1.0
        assert policy["resolved_live_defaults"]["damage_gate_step_fraction"] == 0.0
    elif expected_character == "sparse_pressure":
        assert policy["lead"] == "tr909_pressure"
        assert policy["mc202_intent"] == "punctuate"
        assert manifest["metrics"]["tr909"]["peak_abs"] > manifest["metrics"]["w30"][
            "peak_abs"
        ]
        assert policy["destructive_intent"] == "transient_bite"
        assert policy["resolved_live_defaults"]["damage_playback_rate"] == 1.0
        assert 0.0 < policy["resolved_live_defaults"]["damage_gate_step_fraction"] < 0.5
    else:
        assert expected_character == "dense_break"
        assert policy["mc202_intent"] == "instigate"
        assert policy["destructive_intent"] == "pitch_drag"
        assert policy["resolved_live_defaults"]["damage_playback_rate"] < 1.0
        assert policy["resolved_live_defaults"]["damage_gate_step_fraction"] == 0.0
        assert manifest["metrics"]["mc202"]["rms"] >= manifest["thresholds"][
            "min_audible_lane_rms"
        ]


def main() -> None:
    output = parse_args().output
    cases = {
        "dense_a": ("dense_break", output / "dense-a"),
        "tonal_a": ("tonal_hook", output / "tonal-a"),
        "tonal_b": ("tonal_hook", output / "tonal-b"),
        "sparse_a": ("sparse_pressure", output / "sparse-a"),
        "sparse_b": ("sparse_pressure", output / "sparse-b"),
    }
    manifests = {}
    wavs = {}
    destructive_wavs = {}
    durations = {}
    envelopes = {}
    for case_id, (expected_character, case_dir) in cases.items():
        manifest = load_manifest(case_dir)
        validate_case(manifest, expected_character)
        wav = case_dir / "controlled/01_held_character_loop.wav"
        envelope, duration = rms_envelope(wav)
        expected_duration = manifest["review_duration_bars"] * 4.0 * 60.0 / manifest["bpm"]
        assert abs(duration - expected_duration) <= REVIEW_DURATION_TOLERANCE_SECONDS
        manifests[case_id] = manifest
        wavs[case_id] = sha256(wav)
        destructive_wavs[case_id] = sha256(
            case_dir / "controlled/02_destructive_variation.wav"
        )
        durations[case_id] = duration
        envelopes[case_id] = envelope

    assert manifests["tonal_a"]["source"]["content_hash"] == manifests["tonal_b"]["source"][
        "content_hash"
    ]
    assert manifests["sparse_a"]["source"]["content_hash"] == manifests["sparse_b"]["source"][
        "content_hash"
    ]
    assert wavs["tonal_a"] == wavs["tonal_b"]
    assert wavs["sparse_a"] == wavs["sparse_b"]
    assert destructive_wavs["tonal_a"] == destructive_wavs["tonal_b"]
    assert destructive_wavs["sparse_a"] == destructive_wavs["sparse_b"]
    assert len({wavs["dense_a"], wavs["tonal_a"], wavs["sparse_a"]}) == 3
    assert len(
        {
            destructive_wavs["dense_a"],
            destructive_wavs["tonal_a"],
            destructive_wavs["sparse_a"],
        }
    ) == 3
    assert wavs["tonal_a"] != wavs["sparse_a"]

    comparisons = {}
    for left, right in [
        ("dense_a", "tonal_a"),
        ("dense_a", "sparse_a"),
        ("tonal_a", "sparse_a"),
    ]:
        envelope_correlation = correlation(envelopes[left], envelopes[right])
        envelope_delta = mean_absolute_delta(envelopes[left], envelopes[right])
        assert abs(envelope_correlation) <= MAX_CROSS_SOURCE_ENVELOPE_CORRELATION
        assert envelope_delta >= MIN_CROSS_SOURCE_ENVELOPE_MEAN_ABSOLUTE_DELTA
        comparisons[f"{left}_vs_{right}"] = {
            "rms_envelope_correlation": envelope_correlation,
            "rms_envelope_mean_absolute_delta": envelope_delta,
        }
    assert (
        manifests["tonal_a"]["source_character_policy"]["resolved_live_defaults"]
        != manifests["sparse_a"]["source_character_policy"]["resolved_live_defaults"]
    )

    report = {
        "schema": "riotbox.controlled_source_live_matrix.v1",
        "result": "pass",
        "quality_proof": False,
        "human_verdict": "unverified",
        "same_source_stability": {
            "tonal_exact_wav_match": True,
            "sparse_exact_wav_match": True,
            "tonal_sha256": wavs["tonal_a"],
            "sparse_sha256": wavs["sparse_a"],
            "tonal_destructive_sha256": destructive_wavs["tonal_a"],
            "sparse_destructive_sha256": destructive_wavs["sparse_a"],
        },
        "cross_source_diversity": {
            "characters": ["dense_break", "tonal_hook", "sparse_pressure"],
            "wav_hashes_differ": True,
            "comparisons": comparisons,
            "thresholds": {
                "max_absolute_envelope_correlation": MAX_CROSS_SOURCE_ENVELOPE_CORRELATION,
                "min_envelope_mean_absolute_delta": MIN_CROSS_SOURCE_ENVELOPE_MEAN_ABSOLUTE_DELTA,
            },
        },
        "review_duration_seconds": durations,
    }
    with (output / "source-matrix-report.json").open("w") as handle:
        json.dump(report, handle, indent=2, sort_keys=True)
        handle.write("\n")
    print(f"valid controlled source live matrix: {output / 'source-matrix-report.json'}")


if __name__ == "__main__":
    main()
