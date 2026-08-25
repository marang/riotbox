#!/usr/bin/env python3
"""Retained source-blind falsification for rejected Dense foundation v2."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path
from typing import Any

import numpy as np

import generate_dense_break_performance_pack as dense


CONTRACT = Path("docs/benchmarks/dense_break_foundation_chop_v2.json")
CONTRACT_SHA256 = "b0338bd87e9c536f80f725e9a1f891f4df67b43a370de10564f2fa60884b0020"
REGISTRY = Path("docs/benchmarks/source_holdout_rotation_v2.json")
CASE_ID = "dense_beat03_130"
ANSWER_START_BEAT = 6
ANSWER_BEATS = 2
CANDIDATE_START_BEATS = (0, 2, 4)
CROSSFADE_SECONDS = 0.002
WHOLE_LOW_BAND_LIMIT_DB = 3.0
WHOLE_LEVEL_LIMIT_DB = 3.0
LOCAL_LOW_BAND_LIMIT_DB = 4.0
LOCAL_LEVEL_LIMIT_DB = 4.0
MIN_NORMALIZED_DELTA = 0.20
MAX_ABSOLUTE_CORRELATION = 0.96


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()

    try:
        dense.require_numpy()
        repo = Path(__file__).resolve().parent.parent
        validate_frozen_contract(repo)
        require(args.fixtures, "rejected v2 retains only --fixtures")
        run_source_blind_fixtures()
    except (OSError, ValueError) as error:
        print(f"Dense foundation clarity v2 failed: {error}", file=sys.stderr)
        return 1

    print("valid Dense foundation clarity v2 source-blind fixtures")
    return 0


def select_answer_cell(
    source: np.ndarray,
    beat_frames: int,
    sample_rate_hz: int = dense.SAMPLE_RATE,
) -> dict[str, Any]:
    require(source.shape == (beat_frames * 8, dense.CHANNELS), "source shape mismatch")
    target = source[ANSWER_START_BEAT * beat_frames : 8 * beat_frames]
    candidates = []
    for start_beat in CANDIDATE_START_BEATS:
        cell = source[start_beat * beat_frames : (start_beat + ANSWER_BEATS) * beat_frames]
        whole_low_db = db_delta(
            low_band_rms(cell, sample_rate_hz),
            low_band_rms(target, sample_rate_hz),
        )
        whole_level_db = db_delta(dense.rms(cell), dense.rms(target))
        local_low_db = []
        local_level_db = []
        half_frames = beat_frames // 2
        for local_start in range(0, ANSWER_BEATS * beat_frames, half_frames):
            local_end = local_start + half_frames
            candidate_half = cell[local_start:local_end]
            target_half = target[local_start:local_end]
            local_low_db.append(
                db_delta(
                    low_band_rms(candidate_half, sample_rate_hz),
                    low_band_rms(target_half, sample_rate_hz),
                )
            )
            local_level_db.append(
                db_delta(dense.rms(candidate_half), dense.rms(target_half))
            )
        normalized_delta = dense.normalized_rms_delta(target, cell)
        correlation = dense.waveform_correlation(target, cell)
        clarity_cost = (
            abs(whole_low_db)
            + abs(whole_level_db)
            + float(np.mean(np.abs(local_low_db)))
            + float(np.mean(np.abs(local_level_db)))
        )
        eligible = (
            abs(whole_low_db) <= WHOLE_LOW_BAND_LIMIT_DB
            and abs(whole_level_db) <= WHOLE_LEVEL_LIMIT_DB
            and max(abs(value) for value in local_low_db) <= LOCAL_LOW_BAND_LIMIT_DB
            and max(abs(value) for value in local_level_db) <= LOCAL_LEVEL_LIMIT_DB
            and normalized_delta >= MIN_NORMALIZED_DELTA
            and abs(correlation) <= MAX_ABSOLUTE_CORRELATION
        )
        candidates.append(
            {
                "start_beat": start_beat,
                "whole_low_band_delta_db": whole_low_db,
                "whole_level_delta_db": whole_level_db,
                "local_half_beat_low_band_delta_db": local_low_db,
                "local_half_beat_level_delta_db": local_level_db,
                "normalized_delta": normalized_delta,
                "waveform_correlation": correlation,
                "clarity_cost": clarity_cost,
                "eligible": eligible,
            }
        )
    eligible = [candidate for candidate in candidates if candidate["eligible"]]
    require(eligible, "no clarity-preserving two-beat answer cell")
    selected = min(
        eligible,
        key=lambda candidate: (
            candidate["clarity_cost"],
            abs(candidate["waveform_correlation"]),
            -candidate["normalized_delta"],
            candidate["start_beat"],
        ),
    )
    return {"selected": selected, "candidates": candidates}


def render_answer(
    source: np.ndarray,
    beat_frames: int,
    sample_rate_hz: int = dense.SAMPLE_RATE,
) -> tuple[np.ndarray, dict[str, Any]]:
    selection = select_answer_cell(source, beat_frames, sample_rate_hz)
    selected = selection["selected"]
    cell_start = selected["start_beat"] * beat_frames
    cell = source[cell_start : cell_start + ANSWER_BEATS * beat_frames]
    candidate = source.astype(np.float32, copy=True)
    replace(
        candidate,
        ANSWER_START_BEAT * beat_frames,
        cell,
        max(1, round(CROSSFADE_SECONDS * sample_rate_hz)),
    )
    require(
        np.array_equal(
            candidate[: ANSWER_START_BEAT * beat_frames],
            source[: ANSWER_START_BEAT * beat_frames],
        ),
        "anchor changed",
    )
    require(np.all(np.isfinite(candidate)), "candidate contains non-finite samples")
    require(not np.array_equal(candidate, source), "candidate is identical to source")
    return candidate, selection


def replace(target: np.ndarray, start: int, chunk: np.ndarray, fade: int) -> None:
    original = target[start : start + chunk.shape[0]].copy()
    replacement = chunk.astype(np.float32, copy=True)
    fade = min(fade, replacement.shape[0] // 4)
    ramp = np.linspace(0.0, 1.0, fade, dtype=np.float32)[:, None]
    replacement[:fade] = original[:fade] * (1.0 - ramp) + replacement[:fade] * ramp
    replacement[-fade:] = replacement[-fade:] * (1.0 - ramp) + original[-fade:] * ramp
    target[start : start + replacement.shape[0]] = replacement


def run_source_blind_fixtures() -> None:
    beat_frames = 4096
    phrase = synthetic_phrase(beat_frames, candidate_mode="matched")
    candidate, selection = render_answer(phrase, beat_frames)
    require(selection["selected"]["start_beat"] == 2, "matched cell not selected")
    fade = max(1, round(CROSSFADE_SECONDS * dense.SAMPLE_RATE))
    require(
        np.array_equal(
            candidate[6 * beat_frames + fade : 8 * beat_frames - fade],
            phrase[2 * beat_frames + fade : 4 * beat_frames - fade],
        ),
        "selected coherent cell drifted",
    )
    require(
        np.array_equal(candidate[: 6 * beat_frames], phrase[: 6 * beat_frames]),
        "exact anchor drifted",
    )
    locally_thin = synthetic_phrase(beat_frames, candidate_mode="thin")
    thin_cell = locally_thin[: 2 * beat_frames]
    thin_target = locally_thin[6 * beat_frames :]
    require(
        abs(
            db_delta(
                low_band_rms(thin_cell, dense.SAMPLE_RATE),
                low_band_rms(thin_target, dense.SAMPLE_RATE),
            )
        )
        <= WHOLE_LOW_BAND_LIMIT_DB,
        "local-thin fixture must preserve whole-cell low band",
    )
    require(
        abs(db_delta(dense.rms(thin_cell), dense.rms(thin_target)))
        <= WHOLE_LEVEL_LIMIT_DB,
        "local-thin fixture must preserve whole-cell level",
    )
    expect_rejection(locally_thin, beat_frames, "locally thin cells must fail closed")
    expect_rejection(
        synthetic_phrase(beat_frames, candidate_mode="identical"),
        beat_frames,
        "near-identical cells must fail closed",
    )
    tied = synthetic_phrase(beat_frames, candidate_mode="tied")
    tied_selection = select_answer_cell(tied, beat_frames)
    require(tied_selection["selected"]["start_beat"] == 0, "tie-break drifted")


def synthetic_phrase(beat_frames: int, *, candidate_mode: str) -> np.ndarray:
    target = [
        synthetic_beat(beat_frames, 0.42, 0.18, 0.12),
        synthetic_beat(beat_frames, 0.38, 0.20, 0.58),
    ]
    if candidate_mode == "matched":
        cells = [
            [synthetic_beat(beat_frames, 0.09, 0.18, 0.25)] * 2,
            [
                synthetic_beat(beat_frames, 0.42, 0.18, 0.32, bass_hz=73.0),
                synthetic_beat(beat_frames, 0.38, 0.20, 0.78, bass_hz=73.0),
            ],
            [synthetic_beat(beat_frames, 0.78, 0.18, 0.42)] * 2,
        ]
    elif candidate_mode == "thin":
        local_thin = [
            synthetic_beat(beat_frames, 0.05, 0.18, 0.25),
            synthetic_beat(beat_frames, 0.56, 0.20, 0.72),
        ]
        cells = [[local_thin[0].copy(), local_thin[1].copy()] for _ in range(3)]
    elif candidate_mode == "identical":
        cells = [[target[0].copy(), target[1].copy()]] * 3
    elif candidate_mode == "tied":
        shared = [
            synthetic_beat(beat_frames, 0.42, 0.18, 0.32, bass_hz=73.0),
            synthetic_beat(beat_frames, 0.38, 0.20, 0.78, bass_hz=73.0),
        ]
        cells = [[shared[0].copy(), shared[1].copy()] for _ in range(3)]
    else:
        raise ValueError(f"unknown synthetic candidate mode: {candidate_mode}")
    beats = [beat for cell in cells for beat in cell] + target
    return np.concatenate(beats, axis=0).astype(np.float32)


def synthetic_beat(
    frames: int,
    bass_amplitude: float,
    high_amplitude: float,
    accent_position: float,
    bass_hz: float = 58.0,
) -> np.ndarray:
    time = np.arange(frames, dtype=np.float32) / dense.SAMPLE_RATE
    bass = bass_amplitude * np.sin(2.0 * math.pi * bass_hz * time)
    accent = np.zeros(frames, dtype=np.float32)
    start = min(frames - 1, max(0, round(accent_position * frames)))
    length = min(frames - start, max(8, frames // 20))
    accent[start : start + length] = high_amplitude * np.linspace(
        1.0, 0.0, length, dtype=np.float32
    )
    mono = bass.astype(np.float32) + accent
    return np.stack((mono, mono * 0.97), axis=1)


def expect_rejection(source: np.ndarray, beat_frames: int, message: str) -> None:
    try:
        select_answer_cell(source, beat_frames)
    except ValueError:
        return
    raise ValueError(message)


def db_delta(value: float, reference: float) -> float:
    return 20.0 * math.log10(max(value, 1.0e-9) / max(reference, 1.0e-9))


def low_band_rms(samples: np.ndarray, sample_rate_hz: int) -> float:
    require(sample_rate_hz > 0, "sample rate must be positive")
    if samples.size == 0:
        return 0.0
    dt = 1.0 / sample_rate_hz
    rc = 1.0 / (2.0 * math.pi * 165.0)
    alpha = dt / (rc + dt)
    filtered = np.zeros_like(samples, dtype=np.float32)
    state = np.zeros(samples.shape[1], dtype=np.float64)
    for index, frame in enumerate(samples):
        state += alpha * (frame.astype(np.float64) - state)
        filtered[index] = state
    return dense.rms(filtered)


def validate_frozen_contract(repo: Path) -> dict[str, Any]:
    path = repo / CONTRACT
    require(sha256_file(path) == CONTRACT_SHA256, "frozen v2 contract changed")
    contract = json.loads(path.read_text())
    require(
        contract.get("schema") == "riotbox.dense_break_foundation_chop.v2",
        "frozen v2 contract schema changed",
    )
    mechanism = contract["mechanism"]
    input_contract = mechanism["input"]
    require(input_contract["bars"] == 2, "frozen input bar count changed")
    require(input_contract["beats_per_bar"] == 4, "frozen meter changed")
    require(
        input_contract["requires_confirmed_beat_grid"] is True,
        "frozen grid requirement changed",
    )
    require(input_contract["playback_rate"] == 1.0, "frozen playback rate changed")
    require(
        mechanism["anchor"]
        == {
            "target": "beats_0_through_5",
            "source": "same_positions",
            "operation": "sample_exact_passthrough",
        },
        "frozen anchor changed",
    )
    selector = mechanism["answer_selector"]
    require(selector["target_cell"] == "beats_6_through_7", "frozen target changed")
    require(
        selector["candidate_cells"]
        == ["beats_0_through_1", "beats_2_through_3", "beats_4_through_5"],
        "frozen candidate cells changed",
    )
    require(
        selector["analysis_sample_rate_hz"] == "source_sample_rate_hz",
        "frozen analysis sample-rate rule changed",
    )
    require(
        selector["low_band_estimator"]
        == "per_channel_one_pole_lowpass_165_hz_stereo_rms_v1",
        "frozen low-band estimator changed",
    )
    require(
        selector["level_estimator"] == "stereo_broadband_rms_v1",
        "frozen level estimator changed",
    )
    require(
        selector["local_partition"] == "four_corresponding_half_beat_windows",
        "frozen local partition changed",
    )
    require(
        selector["eligibility"]
        == {
            "whole_cell_absolute_low_band_delta_db_max": WHOLE_LOW_BAND_LIMIT_DB,
            "whole_cell_absolute_level_delta_db_max": WHOLE_LEVEL_LIMIT_DB,
            "each_half_beat_absolute_low_band_delta_db_max": LOCAL_LOW_BAND_LIMIT_DB,
            "each_half_beat_absolute_level_delta_db_max": LOCAL_LEVEL_LIMIT_DB,
            "normalized_rms_delta_min": MIN_NORMALIZED_DELTA,
            "absolute_waveform_correlation_max": MAX_ABSOLUTE_CORRELATION,
        },
        "frozen eligibility gates changed",
    )
    require(
        selector["selection_order"]
        == [
            "lowest_sum_of_absolute_whole_and_mean_local_low_band_and_level_deltas",
            "lowest_absolute_waveform_correlation",
            "highest_normalized_rms_delta",
            "lowest_source_start_beat",
        ],
        "frozen selection order changed",
    )
    splice = mechanism["splice"]
    require(splice["crossfade_shape"] == "linear", "frozen crossfade shape changed")
    require(
        splice["crossfade_duration_ms"] == CROSSFADE_SECONDS * 1000.0,
        "frozen crossfade duration changed",
    )
    require(splice["apply_at_answer_entry"] is True, "frozen entry splice changed")
    require(
        splice["apply_at_phrase_end_return"] is True,
        "frozen return splice changed",
    )
    require(splice["internal_answer_splice_count"] == 0, "frozen answer splice changed")
    development = contract["development_exploration"]
    require(development["variant_limit"] == 1, "frozen variant limit changed")
    require(development["source"]["case_id"] == CASE_ID, "frozen source case changed")
    require(
        development["active_holdout_collision_registry"]["path"] == REGISTRY.as_posix(),
        "frozen collision registry path changed",
    )
    require(
        development["active_holdout_collision_registry"]["raw_sha256"]
        == "af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812",
        "frozen collision registry hash changed",
    )
    return contract


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
