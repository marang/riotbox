#!/usr/bin/env python3
"""Render one bounded Development-only dense-break foundation exploration."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
import wave
from pathlib import Path
from typing import Any

import numpy as np

import generate_dense_break_performance_pack as dense


SCHEMA = "riotbox.dense_break_foundation_chop_exploration.v1"
VARIANT_ID = "anchored_long_slice_chop_v1"
LOOP_COUNT = 4
CROSSFADE_SECONDS = 0.002
FROZEN_CONTRACT = Path("docs/benchmarks/dense_break_foundation_chop_v1.json")
FROZEN_CONTRACT_SHA256 = (
    "79671dd532459d2c4dd25636a989c3c088e515555eae1936fa7145e5aef3b2a6"
)
EXACT_SOURCE = Path("data/test_audio/examples/Beat03_130BPM(Full).wav")
EXACT_SOURCE_SHA256 = (
    "e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a"
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--bpm", type=float, default=130.0)
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()

    try:
        if args.fixtures:
            run_sanity_check()
            print("valid dense-break foundation exploration sanity check")
            return 0
        require(args.source is not None, "--source is required")
        require(args.output is not None, "--output is required")
        require(math.isfinite(args.bpm) and args.bpm > 0.0, "--bpm must be positive")
        result = render(args.source, args.output, args.bpm)
    except (OSError, ValueError, wave.Error) as error:
        print(f"dense-break foundation exploration failed: {error}", file=sys.stderr)
        return 1

    print(f"dense-break foundation exploration written: {args.output} ({result})")
    return 0 if result == "pass" else 1


def render(source_arg: Path, output_arg: Path, bpm: float) -> str:
    repo = Path(__file__).resolve().parent.parent
    validate_frozen_contract(repo)
    require(source_arg == EXACT_SOURCE, f"source must be exact registered path: {EXACT_SOURCE}")
    source_path = resolve(repo, source_arg)
    output = resolve(repo, output_arg)
    allowed = (repo / "artifacts" / "audio_qa").resolve()
    require(allowed in output.resolve().parents, f"output must be below {allowed}")
    require(not output.exists(), f"refusing to overwrite existing output: {output}")
    require(source_path.is_file(), f"missing source: {source_path}")
    require(sha256_file(source_path) == EXACT_SOURCE_SHA256, "source SHA-256 mismatch")

    beat_frames = dense.frames_for_beats(bpm, 1)
    phrase_frames = beat_frames * 8
    source = dense.read_wav_window_looped(
        source_path,
        0.0,
        phrase_frames / dense.SAMPLE_RATE,
    )[:phrase_frames]
    candidate = anchored_long_slice_chop(source, beat_frames)
    require(np.all(np.isfinite(candidate)), "candidate contains non-finite samples")
    require(not np.array_equal(candidate, source), "candidate is identical to source")

    source_loop = np.tile(source, (LOOP_COUNT, 1))
    candidate_loop = np.tile(candidate, (LOOP_COUNT, 1))
    safety = presentation_safety({"source": source_loop, "candidate": candidate_loop})
    gain = float(safety["uniform_gain"])
    rendered = {
        "00_source_two_bar_loop.wav": dense.apply_presentation_gain(source_loop, gain),
        "01_candidate_v1_two_bar_loop.wav": dense.apply_presentation_gain(
            candidate_loop, gain
        ),
    }
    output.mkdir(parents=True, exist_ok=True)
    for name, audio in rendered.items():
        dense.write_wav(output / name, audio)

    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": safety["result"],
        "ticket": "RIOTBOX-1462",
        "stage": "development_exploration",
        "variant": {"id": VARIANT_ID, "number": 1, "limit": 3},
        "question": (
            "Can a sparse long-slice anchor/deviation/return pattern preserve the "
            "break's identity, groove, and clarity while creating a two-bar hook?"
        ),
        "source": {
            "path": str(source_arg),
            "sha256": sha256_file(source_path),
            "bpm": bpm,
            "sample_rate_hz": dense.SAMPLE_RATE,
            "channels": dense.CHANNELS,
        },
        "recipe": {
            "topology": "first bar exact; second bar anchor, deviation, clean return",
            "exact_beats": [0, 1, 2, 3, 4, 5],
            "beat_6_source": 4,
            "beat_7_first_half_source": "beat_5_second_half",
            "beat_7_second_half": "exact",
            "crossfade_seconds": CROSSFADE_SECONDS,
            "reverse_count": 0,
            "support_lane_count": 0,
            "additive_layer_count": 0,
            "bus_effect_count": 0,
        },
        "comparison": {
            "normalized_rms_delta": dense.normalized_rms_delta(source, candidate),
            "waveform_correlation": dense.waveform_correlation(source, candidate),
            "source_rms": dense.rms(source),
            "candidate_rms": dense.rms(candidate),
            "source_high_band_ratio": dense.high_band_ratio(source),
            "candidate_high_band_ratio": dense.high_band_ratio(candidate),
            "source_transient_score": dense.transient_score(source),
            "candidate_transient_score": dense.transient_score(candidate),
        },
        "files": {
            name: {"sha256": sha256_file(output / name)} for name in rendered
        },
        "presentation": {
            "loop_count": LOOP_COUNT,
            "duration_seconds_each": source_loop.shape[0] / dense.SAMPLE_RATE,
            "playback_order": ["source", "candidate"],
            "pause_seconds": 1.0,
        },
        "presentation_safety": safety,
        "evidence_boundary": {
            "human_verdict": "unverified",
            "quality_proof": False,
            "product_behavior": False,
            "source_general": False,
            "holdout_access": False,
            "release_readiness": "blocked",
        },
    }
    (output / "exploration-report.json").write_text(json.dumps(report, indent=2) + "\n")
    return str(safety["result"])


def anchored_long_slice_chop(source: np.ndarray, beat_frames: int) -> np.ndarray:
    """Keep 6.5 beats exact and form one sparse answer in the second bar."""
    require(source.shape == (beat_frames * 8, dense.CHANNELS), "source shape mismatch")
    candidate = source.astype(np.float32, copy=True)
    fade_frames = max(1, round(CROSSFADE_SECONDS * dense.SAMPLE_RATE))

    replace(
        candidate,
        6 * beat_frames,
        source[4 * beat_frames : 5 * beat_frames],
        fade_frames,
    )
    half = beat_frames // 2
    replace(
        candidate,
        7 * beat_frames,
        source[5 * beat_frames + half : 6 * beat_frames],
        fade_frames,
    )
    return candidate


def replace(target: np.ndarray, start: int, chunk: np.ndarray, fade: int) -> None:
    original = target[start : start + chunk.shape[0]].copy()
    replacement = chunk.astype(np.float32, copy=True)
    fade = min(fade, replacement.shape[0] // 4)
    ramp = np.linspace(0.0, 1.0, fade, dtype=np.float32)[:, None]
    replacement[:fade] = original[:fade] * (1.0 - ramp) + replacement[:fade] * ramp
    replacement[-fade:] = replacement[-fade:] * (1.0 - ramp) + original[-fade:] * ramp
    target[start : start + replacement.shape[0]] = replacement


def presentation_safety(signals: dict[str, np.ndarray]) -> dict[str, Any]:
    peaks = {
        name: dense.conservative_true_peak_amplitude(audio)
        for name, audio in signals.items()
    }
    maximum = max(peaks.values(), default=0.0)
    target = dense.db_to_amplitude(dense.TARGET_PRESENTATION_TRUE_PEAK_DBTP)
    gain = min(1.0, target / maximum) if maximum > 1e-12 else 1.0
    post = {name: peak * gain for name, peak in peaks.items()}
    maximum_post = max(post.values(), default=0.0)
    result = (
        "pass"
        if dense.amplitude_to_db(maximum_post)
        <= dense.MAX_PRESENTATION_TRUE_PEAK_DBTP
        else "fail"
    )
    return {
        "schema": dense.PRESENTATION_SAFETY_SCHEMA,
        "schema_version": 1,
        "result": result,
        "estimator": "conservative_four_x_bandlimited_fft_v1",
        "oversample_factor": dense.TRUE_PEAK_OVERSAMPLE_FACTOR,
        "max_allowed_true_peak_dbtp": dense.MAX_PRESENTATION_TRUE_PEAK_DBTP,
        "normalization_target_true_peak_dbtp": dense.TARGET_PRESENTATION_TRUE_PEAK_DBTP,
        "uniform_gain": gain,
        "uniform_gain_db": dense.amplitude_to_db(gain),
        "pre_gain_true_peak_dbtp": {
            name: dense.amplitude_to_db(value) for name, value in peaks.items()
        },
        "post_gain_true_peak_dbtp": {
            name: dense.amplitude_to_db(value) for name, value in post.items()
        },
    }


def run_sanity_check() -> None:
    validate_frozen_contract(Path(__file__).resolve().parent.parent)
    beat_frames = 1000
    source = np.repeat(np.arange(8, dtype=np.float32), beat_frames * 2).reshape(-1, 2)
    candidate = anchored_long_slice_chop(source, beat_frames)
    half = beat_frames // 2
    require(
        np.array_equal(candidate[: 6 * beat_frames], source[: 6 * beat_frames]),
        "anchor changed",
    )
    require(
        np.array_equal(
            candidate[6 * beat_frames + 100 : 7 * beat_frames - 100],
            source[4 * beat_frames + 100 : 5 * beat_frames - 100],
        ),
        "full-beat answer drifted",
    )
    require(
        np.array_equal(
            candidate[7 * beat_frames + 100 : 7 * beat_frames + half - 100],
            source[5 * beat_frames + half + 100 : 6 * beat_frames - 100],
        ),
        "half-beat answer drifted",
    )
    require(
        np.array_equal(
            candidate[7 * beat_frames + half :], source[7 * beat_frames + half :]
        ),
        "return changed",
    )
    require(np.all(np.isfinite(candidate)), "candidate contains non-finite samples")


def validate_frozen_contract(repo: Path) -> None:
    path = repo / FROZEN_CONTRACT
    require(sha256_file(path) == FROZEN_CONTRACT_SHA256, "frozen contract changed")
    contract = json.loads(path.read_text())
    require(
        contract.get("schema") == "riotbox.dense_break_foundation_chop.v1",
        "frozen contract schema changed",
    )
    mechanism = contract.get("mechanism", {})
    require(
        mechanism.get("splice", {}).get("crossfade_duration_ms") == 2.0,
        "frozen splice changed",
    )
    mappings = mechanism.get("mapping", [])
    require(
        [item.get("target") for item in mappings]
        == [
            "beats_0_through_5",
            "beat_6",
            "beat_7_first_half",
            "beat_7_second_half",
        ],
        "frozen mapping changed",
    )


def resolve(repo: Path, path: Path) -> Path:
    return path if path.is_absolute() else repo / path


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    raise SystemExit(main())
