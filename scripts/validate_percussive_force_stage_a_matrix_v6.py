#!/usr/bin/env python3
"""Run frozen advanced mechanical screens for RIOTBOX-1430 Matrix v6.

Automation may reject candidates but never award perceived force or quality.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
from pathlib import Path
from typing import Any

import numpy as np

import percussive_force_stage_a_analysis as stage_a
import run_percussive_force_stage_a_qualification as shared
import run_percussive_force_stage_a_v2_qualification as v2
from source_holdout_development_access import (
    SourceIdentity,
    maximum_source_file_bytes,
    read_contained_regular_file,
)


ROOT = Path(__file__).resolve().parents[1]
MATRIX = Path("docs/benchmarks/percussive_force_development_matrix_v6.json")
MATRIX_SHA = "cd29b23fd3d39ac5184f73585b825aabf987b865e6f37253260ce2287ac95c00"
SOURCE_SET = Path("docs/benchmarks/percussive_force_stage_a_bound_source_set_v1.json")
SOURCE_SET_SHA = "7ec185a51233d83c49d8227b0e81acb2ca83c24bc31783a9343dc71d090e47a6"
PROTOCOL = Path("docs/benchmarks/percussive_force_stage_a_protocol_v2.json")
PROTOCOL_SHA = "b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878"
RESULT_NAME = "matrix-result.json"
OUTPUT_NAME = "advanced-mechanical-result.json"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise shared.QualificationSessionError(message)


def load_json(path: Path) -> tuple[dict[str, Any], bytes]:
    return shared.read_pinned_json(path, hashlib.sha256(path.read_bytes()).hexdigest())


def decode_float_wave(path: Path) -> tuple[np.ndarray, int, int, bytes]:
    raw = path.read_bytes()
    require(len(raw) >= 44 and raw[:4] == b"RIFF" and raw[8:12] == b"WAVE", f"not RIFF/WAVE: {path}")
    require(struct.unpack_from("<I", raw, 4)[0] == len(raw) - 8, f"RIFF size changed: {path}")
    fmt = None
    data = None
    offset = 12
    while offset < len(raw):
        require(offset + 8 <= len(raw), f"truncated chunk: {path}")
        chunk_id = raw[offset : offset + 4]
        size = struct.unpack_from("<I", raw, offset + 4)[0]
        start = offset + 8
        end = start + size
        padded = end + (size & 1)
        require(padded <= len(raw), f"chunk exceeds RIFF: {path}")
        if chunk_id == b"fmt ":
            require(fmt is None and size == 16, f"candidate fmt changed: {path}")
            fmt = struct.unpack_from("<HHIIHH", raw, start)
        elif chunk_id == b"data":
            require(fmt is not None and data is None, f"candidate data order changed: {path}")
            data = raw[start:end]
        offset = padded
    require(fmt is not None and data is not None and offset == len(raw), f"candidate chunks invalid: {path}")
    tag, channels, rate, byte_rate, block_align, bits = fmt
    require(tag == 3 and bits == 32, f"candidate must be IEEE float32 WAV: {path}")
    require(block_align == channels * 4 and byte_rate == rate * block_align, f"candidate format mismatch: {path}")
    samples = np.frombuffer(data, dtype="<f4").astype(np.float64)
    require(samples.size % channels == 0, f"candidate alignment changed: {path}")
    return samples.reshape(-1, channels), rate, channels, raw


def frozen_spec(entry: dict[str, Any]) -> shared.FrozenSourceSpec:
    return shared.FrozenSourceSpec(
        case_id=str(entry["case_id"]),
        source_family=str(entry["source_family"]),
        author=str(entry["author"]),
        source_path=str(entry["source_path"]),
        source_sha256=str(entry["sha256"]),
        license=str(entry["license"]),
        partition="development",
        source_format=dict(entry["source_format"]),
    )


def load_source(spec: shared.FrozenSourceSpec) -> tuple[stage_a.SourceInput, dict[str, Any]]:
    payload = read_contained_regular_file(
        ROOT,
        Path(spec.source_path),
        f"Matrix-v6:{spec.case_id}",
        maximum_bytes=maximum_source_file_bytes(spec.source_format),
    )
    actual_sha = hashlib.sha256(payload).hexdigest()
    require(actual_sha == spec.source_sha256, f"source SHA changed: {spec.case_id}")
    identity = SourceIdentity(
        case_id=spec.case_id,
        source_path=spec.source_path,
        expected_sha256=spec.source_sha256,
        partition="development",
        source_format=spec.source_format,
    )
    captured = shared.CapturedSource(
        identity=identity,
        payload=payload,
        access_record={
            "actual_sha256": actual_sha,
            "access_verification_status": "verified",
        },
    )
    return v2.decode_captured_source(captured, spec)


def scaled_mean_bits(values: list[str], gain: float) -> tuple[str, ...]:
    result = []
    for encoded in values:
        value = struct.unpack(">d", bytes.fromhex(encoded))[0] * gain
        result.append(struct.pack(">d", value).hex())
    return tuple(result)


def analyze(
    source: stage_a.SourceInput,
    samples: np.ndarray,
    protocol: stage_a.FrozenStageAProtocol,
    gain: float = 1.0,
) -> stage_a.SourceAnalysis:
    return stage_a.analyze_source(
        source.metadata,
        samples,
        source.sample_rate_hz,
        source.input_lsb * gain,
        protocol=protocol,
        per_channel_dc_mean_f64_bits_be_hex=scaled_mean_bits(
            list(source.per_channel_dc_mean_f64_bits_be_hex), gain
        ),
    )


def event_by_ordinal(analysis: stage_a.SourceAnalysis, ordinal: int) -> stage_a.EventRecord:
    matches = [event for event in analysis.events if event.ordinal == ordinal]
    require(len(matches) == 1, f"missing frozen source event ordinal {ordinal}")
    return matches[0]


def corresponding_event(
    source_event: stage_a.EventRecord,
    candidate: stage_a.SourceAnalysis,
    sample_rate: int,
    protocol: stage_a.FrozenStageAProtocol,
) -> tuple[stage_a.EventRecord | None, dict[str, Any]]:
    onset_tolerance = stage_a._duration_frames(
        protocol, sample_rate, float(protocol.value("candidate_onset_tolerance_ms"))
    )
    proxy_tolerance = stage_a._duration_frames(
        protocol, sample_rate, float(protocol.value("candidate_rhythmic_proxy_tolerance_ms"))
    )
    matches = [
        event
        for event in candidate.events
        if abs(event.physical_onset_frame - source_event.physical_onset_frame)
        <= onset_tolerance
        and abs(event.rhythmic_proxy_frame - source_event.rhythmic_proxy_frame)
        <= proxy_tolerance
    ]
    return (
        matches[0] if len(matches) == 1 else None,
        {
            "source_event_count": len(candidate.event_level_onset_frames),
            "corresponding_cluster_count": len(matches),
            "onset_tolerance_frames": onset_tolerance,
            "proxy_tolerance_frames": proxy_tolerance,
        },
    )


def attack_spectral_cosine(
    source: np.ndarray,
    candidate: np.ndarray,
    onset: int,
    attack_end: int,
    sample_rate: int,
    bands: int,
) -> float:
    frame_count = attack_end - onset
    require(frame_count > 1, "attack spectral window too short")
    window = 0.5 - 0.5 * np.cos(2.0 * np.pi * np.arange(frame_count) / frame_count)
    fft_size = 1 << (frame_count - 1).bit_length()
    frequencies = np.fft.rfftfreq(fft_size, 1.0 / sample_rate)
    upper = min(12_000.0, sample_rate / 2.0)
    require(upper > 20.0, "attack spectral upper bound invalid")
    edges = 20.0 * (upper / 20.0) ** (np.arange(bands + 1) / bands)

    def vector(samples: np.ndarray) -> np.ndarray:
        spectrum = np.fft.rfft(samples[onset:attack_end] * window[:, None], n=fft_size, axis=0)
        power = np.sum(np.abs(spectrum) ** 2, axis=1)
        values = np.zeros(bands, dtype=np.float64)
        for index in range(bands):
            mask = (frequencies >= edges[index]) & (
                frequencies <= edges[index + 1]
                if index == bands - 1
                else frequencies < edges[index + 1]
            )
            values[index] = float(np.sum(power[mask]))
        total = float(np.sum(values))
        require(total > 0.0 and math.isfinite(total), "attack spectrum undefined")
        return values / total

    left = vector(source)
    right = vector(candidate)
    denominator = float(np.linalg.norm(left) * np.linalg.norm(right))
    require(denominator > 0.0 and math.isfinite(denominator), "spectral cosine undefined")
    return float(np.dot(left, right) / denominator)


def normalized_fit_residual(target: np.ndarray, basis: np.ndarray) -> float:
    y = target.reshape(-1)
    x = basis.reshape(-1, basis.shape[-1])
    denominator = float(np.dot(y, y))
    require(denominator > 0.0 and math.isfinite(denominator), "fit denominator undefined")
    coefficients, _, rank, _ = np.linalg.lstsq(x, y, rcond=None)
    require(rank == x.shape[1], "fit basis singular")
    residual = y - x @ coefficients
    return math.sqrt(float(np.dot(residual, residual)) / denominator)


def attack_quantiles(
    source: np.ndarray, onset: int, attack_end: int, sample_rate: int
) -> tuple[float, float]:
    frames = attack_end - onset
    require(frames >= 4, "static EQ attack window too short")
    window = 0.5 - 0.5 * np.cos(2.0 * np.pi * np.arange(frames) / frames)
    spectrum = np.fft.rfft(source[onset:attack_end] * window[:, None], n=frames, axis=0)
    energies = np.sum(np.abs(spectrum) ** 2, axis=1)[1 : (frames + 1) // 2]
    require(energies.size >= 3 and float(np.sum(energies)) > 0.0, "static EQ split unavailable")
    cumulative = np.cumsum(energies)
    total = float(cumulative[-1])
    f25_bin = int(np.searchsorted(cumulative, 0.25 * total)) + 1
    f75_bin = int(np.searchsorted(cumulative, 0.75 * total)) + 1
    require(f75_bin - f25_bin >= 2, "static EQ split separation unavailable")
    return f25_bin * sample_rate / frames, f75_bin * sample_rate / frames


def complementary_bank(
    source: np.ndarray,
    sample_rate: int,
    f25: float,
    f75: float,
    warmup_start: int,
) -> np.ndarray:
    result = np.zeros((source.shape[0], source.shape[1], 3), dtype=np.float64)
    low_state = np.zeros(source.shape[1], dtype=np.float64)
    mid_state = np.zeros(source.shape[1], dtype=np.float64)
    feed_low = 1.0 - math.exp(-2.0 * math.pi * f25 / sample_rate)
    feed_mid = 1.0 - math.exp(-2.0 * math.pi * f75 / sample_rate)
    for frame in range(warmup_start, source.shape[0]):
        low_state += feed_low * (source[frame] - low_state)
        residual = source[frame] - low_state
        mid_state += feed_mid * (residual - mid_state)
        result[frame, :, 0] = low_state
        result[frame, :, 1] = mid_state
        result[frame, :, 2] = source[frame] - low_state - mid_state
    return result


def fit_metrics(
    source: np.ndarray,
    candidate: np.ndarray,
    event: stage_a.EventRecord,
    sample_rate: int,
    protocol: stage_a.FrozenStageAProtocol,
) -> dict[str, float]:
    region = slice(event.physical_onset_frame, event.body_end_frame)
    x = source[region]
    y = candidate[region]
    gain = normalized_fit_residual(y, x[:, :, None])
    distortion = normalized_fit_residual(y, np.stack((x, x**3, x**5), axis=-1))
    f25, f75 = attack_quantiles(
        source, event.physical_onset_frame, event.attack_end_frame, sample_rate
    )
    lookbehind = stage_a._duration_frames(
        protocol, sample_rate, float(protocol.value("lookbehind_ms"))
    )
    bank = complementary_bank(
        source, sample_rate, f25, f75, event.physical_onset_frame - lookbehind
    )
    eq = normalized_fit_residual(y, bank[region])
    return {
        "gain_fit_residual": gain,
        "static_eq_fit_residual": eq,
        "static_distortion_fit_residual": distortion,
        "f25_hz": f25,
        "f75_hz": f75,
    }


def boundary_pass(
    source: np.ndarray,
    candidate: np.ndarray,
    event: stage_a.EventRecord,
    policy: dict[str, Any],
    sample_rate: int,
    protocol: stage_a.FrozenStageAProtocol,
) -> bool:
    count = int(policy["attack_body_crossfade_frames"])
    boundaries = {
        event.attack_end_frame - count // 2,
        event.attack_end_frame - count // 2 + count,
        event.body_end_frame - int(policy["body_fade_frames"]),
        event.body_end_frame,
    }
    boundaries.discard(event.physical_onset_frame)
    delta = candidate - source
    steps = np.max(np.abs(delta[1:] - delta[:-1]), axis=1)
    radius = stage_a._duration_frames(
        protocol,
        sample_rate,
        float(protocol.value("boundary_discontinuity_neighborhood_ms")),
    )
    epsilon = float(protocol.value("floating_comparison_epsilon_multiplier")) * np.finfo(np.float64).eps
    peak = float(np.max(np.abs(delta[event.physical_onset_frame : event.body_end_frame])))
    tolerance = epsilon * max(1.0, peak)
    for boundary in boundaries:
        if boundary <= radius or boundary + radius >= source.shape[0]:
            return False
        indices = [
            index
            for index in range(boundary - radius, boundary + radius + 1)
            if index not in boundaries and index > 0
        ]
        if not indices:
            return False
        jump = float(np.max(np.abs(delta[boundary] - delta[boundary - 1])))
        local = max(float(steps[index - 1]) for index in indices)
        if jump > local + tolerance:
            return False
    return True


def matcher_gains(source_clip: np.ndarray, candidate_clip: np.ndarray) -> tuple[float, float]:
    source_rms = math.sqrt(float(np.mean(source_clip**2)))
    candidate_rms = math.sqrt(float(np.mean(candidate_clip**2)))
    require(source_rms > 0.0 and candidate_rms > 0.0, "matcher RMS undefined")
    target = min(source_rms, candidate_rms)
    source_gain = target / source_rms
    candidate_gain = target / candidate_rms
    require(source_gain <= 1.0 and candidate_gain <= 1.0, "matcher attempted boost")
    ceiling = 10.0 ** (-1.0 / 20.0)
    peak = max(
        source_gain * float(np.max(np.abs(source_clip))),
        candidate_gain * float(np.max(np.abs(candidate_clip))),
    )
    common = min(1.0, ceiling / peak) if peak > 0.0 else 1.0
    return common * source_gain, common * candidate_gain


def screen_view(
    label: str,
    source: np.ndarray,
    candidate: np.ndarray,
    source_input: stage_a.SourceInput,
    source_analysis: stage_a.SourceAnalysis,
    source_event: stage_a.EventRecord,
    policy: dict[str, Any],
    source_gain: float,
    candidate_gain: float,
    protocol: stage_a.FrozenStageAProtocol,
) -> dict[str, Any]:
    scaled_source = source * source_gain
    scaled_candidate = candidate * candidate_gain
    source_reanalysis = analyze(source_input, scaled_source, protocol, source_gain)
    candidate_analysis = analyze(source_input, scaled_candidate, protocol, candidate_gain)
    corresponding, event_metrics = corresponding_event(
        source_event, candidate_analysis, source_input.sample_rate_hz, protocol
    )
    event_count_pass = len(candidate_analysis.event_level_onset_frames) <= len(
        source_reanalysis.event_level_onset_frames
    )
    onset = source_event.physical_onset_frame
    attack_end = source_event.attack_end_frame
    body_end = source_event.body_end_frame
    x = scaled_source[onset:body_end]
    y = scaled_candidate[onset:body_end]
    source_energy = float(np.sum(x**2))
    delta = math.sqrt(float(np.sum((y - x) ** 2)) / source_energy)
    correlation = float(
        np.sum((x - np.mean(x, axis=0)) * (y - np.mean(y, axis=0)))
        / math.sqrt(
            float(np.sum((x - np.mean(x, axis=0)) ** 2))
            * float(np.sum((y - np.mean(y, axis=0)) ** 2))
        )
    )
    cosine = attack_spectral_cosine(
        scaled_source,
        scaled_candidate,
        onset,
        attack_end,
        source_input.sample_rate_hz,
        int(protocol.value("attack_spectral_band_count")),
    )
    body_ratio = float(np.sum(scaled_candidate[attack_end:body_end] ** 2)) / float(
        np.sum(scaled_source[attack_end:body_end] ** 2)
    )
    failures = []
    if corresponding is None or not event_count_pass:
        failures.append("event_integrity")
    if delta < float(protocol.value("near_identity_delta_rms_min")):
        failures.append("near_identity")
    if correlation < float(protocol.value("identity_correlation_min")) or cosine < float(
        protocol.value("attack_spectral_cosine_min")
    ):
        failures.append("identity")
    body_range = protocol.value("body_energy_ratio_range")
    if not float(body_range[0]) <= body_ratio <= float(body_range[1]):
        failures.append("body_energy")
    try:
        fits = fit_metrics(
            scaled_source,
            scaled_candidate,
            source_event,
            source_input.sample_rate_hz,
            protocol,
        )
    except Exception as error:
        fits = {"error": str(error)}
        failures.append("confound_screen_undefined")
    else:
        if fits["gain_fit_residual"] < float(protocol.value("gain_only_fit_residual_min")):
            failures.append("gain_only_confound")
        if fits["static_eq_fit_residual"] < float(protocol.value("static_eq_fit_residual_min")):
            failures.append("static_eq_confound")
        if fits["static_distortion_fit_residual"] < float(
            protocol.value("static_distortion_fit_residual_min")
        ):
            failures.append("static_distortion_confound")
    boundaries = boundary_pass(
        scaled_source,
        scaled_candidate,
        source_event,
        policy,
        source_input.sample_rate_hz,
        protocol,
    )
    if not boundaries:
        failures.append("boundary_discontinuity")
    return {
        "view": label,
        "source_gain": source_gain,
        "candidate_gain": candidate_gain,
        "candidate_event_level_count": len(candidate_analysis.event_level_onset_frames),
        "source_event_level_count": len(source_reanalysis.event_level_onset_frames),
        "event_integrity": event_metrics,
        "near_identity_delta": delta,
        "zero_lag_identity_correlation": correlation,
        "attack_spectral_cosine": cosine,
        "body_energy_ratio": body_ratio,
        "fit_metrics": fits,
        "boundary_discontinuity_pass": boundaries,
        "failures": sorted(set(failures)),
        "passed": not failures,
    }


def run(matrix_result_path: Path) -> dict[str, Any]:
    matrix, matrix_raw = shared.read_pinned_json(MATRIX, MATRIX_SHA)
    source_set, _ = shared.read_pinned_json(SOURCE_SET, SOURCE_SET_SHA)
    protocol = stage_a.load_frozen_protocol(PROTOCOL)
    require(protocol.sha256 == PROTOCOL_SHA, "Protocol-v2 pin changed")
    result, result_raw = load_json(matrix_result_path)
    require(result.get("schema") == "riotbox.percussive_force_development_matrix_result.v1", "matrix result schema changed")
    require(result.get("matrix_sha256") == MATRIX_SHA, "matrix result binding changed")
    require(result.get("condition_count") == 24, "matrix result cardinality changed")
    qualification_path = Path(str(matrix["qualification_artifact_path"]))
    qualification, _ = shared.read_pinned_json(
        qualification_path, str(matrix["qualification_artifact_sha256"])
    )
    selected_analyses = {
        item["metadata"]["case_id"]: item
        for item in qualification["selected_qualification"]["sources"]
    }
    source_entries = {entry["case_id"]: entry for entry in source_set["entries"]}
    source_bindings = {entry["case_id"]: entry for entry in qualification["source_bindings"]}
    loaded_sources: dict[str, tuple[stage_a.SourceInput, stage_a.SourceAnalysis]] = {}
    advanced = []
    for condition in result["conditions"]:
        base = {
            "condition_id": condition["condition_id"],
            "family": condition["family"],
            "case_id": condition["case_id"],
            "event_ordinal": condition["event_ordinal"],
        }
        if condition["render_state"] != "rendered_basic_screens_passed":
            advanced.append(
                base
                | {
                    "advanced_state": "not_run_inherited_renderer_or_basic_rejection",
                    "survived": False,
                    "quality_proof": False,
                    "hardness_proof": False,
                    "human_verdict": "unverified",
                }
            )
            continue
        case_id = str(condition["case_id"])
        if case_id not in loaded_sources:
            source_input, binding = load_source(frozen_spec(source_entries[case_id]))
            require(
                binding["pcm_f32le_sha256"] == source_bindings[case_id]["pcm_f32le_sha256"],
                f"source PCM binding changed: {case_id}",
            )
            source_analysis = analyze(source_input, source_input.samples, protocol)
            require(
                source_analysis.to_dict() == selected_analyses[case_id],
                f"source reanalysis changed after qualification: {case_id}",
            )
            loaded_sources[case_id] = (source_input, source_analysis)
        source_input, source_analysis = loaded_sources[case_id]
        output_path = Path(str(condition["output_path"]))
        candidate, rate, channels, output_raw = decode_float_wave(output_path)
        require(rate == source_input.sample_rate_hz and channels == source_input.samples.shape[1], "candidate format changed")
        require(hashlib.sha256(output_raw).hexdigest() == condition["output_wav_sha256"], "candidate WAV SHA changed")
        require(
            v2.pcm_f32le_sha256(candidate, rate, channels)
            == condition["candidate_pcm_f32le_sha256"],
            "candidate PCM SHA changed",
        )
        source_event = event_by_ordinal(source_analysis, int(condition["event_ordinal"]))
        tail_end = source_event.tail_end_frame
        source_clip = source_input.samples[source_event.physical_onset_frame : tail_end]
        candidate_clip = candidate[source_event.physical_onset_frame : tail_end]
        source_gain, candidate_gain = matcher_gains(source_clip, candidate_clip)
        raw_view = screen_view(
            "raw",
            source_input.samples,
            candidate,
            source_input,
            source_analysis,
            source_event,
            dict(condition["policy"]),
            1.0,
            1.0,
            protocol,
        )
        matched_view = screen_view(
            "event_train_rms_attenuation_match_v1",
            source_input.samples,
            candidate,
            source_input,
            source_analysis,
            source_event,
            dict(condition["policy"]),
            source_gain,
            candidate_gain,
            protocol,
        )
        survived = bool(raw_view["passed"] and matched_view["passed"])
        advanced.append(
            base
            | {
                "advanced_state": "passed" if survived else "rejected",
                "raw_view": raw_view,
                "matched_view": matched_view,
                "survived": survived,
                "quality_proof": False,
                "hardness_proof": False,
                "human_verdict": "unverified",
            }
        )
    survivors = [item["condition_id"] for item in advanced if item["survived"]]
    return {
        "schema": "riotbox.percussive_force_development_matrix_advanced_result.v1",
        "owner_ticket": "RIOTBOX-1430",
        "matrix_path": MATRIX.as_posix(),
        "matrix_sha256": hashlib.sha256(matrix_raw).hexdigest(),
        "matrix_result_path": matrix_result_path.as_posix(),
        "matrix_result_sha256": hashlib.sha256(result_raw).hexdigest(),
        "condition_count": 24,
        "basic_survivor_count": result["rendered_basic_screen_pass_count"],
        "advanced_survivor_count": len(survivors),
        "survivor_condition_ids": survivors,
        "conditions": advanced,
        "advanced_mechanical_screens_complete": True,
        "candidate_playback_started": False,
        "holdout_audio_accessed": False,
        "commercial_reference_accessed": False,
        "quality_proof": False,
        "hardness_proof": False,
        "human_verdict": "unverified",
        "next_allowed_action": (
            "prepare_bounded_blinded_human_review" if survivors else "stop_no_candidate_playback"
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--matrix-result",
        type=Path,
        default=Path("artifacts/audio_qa/riotbox-1430/stage-a-v6-matrix") / RESULT_NAME,
    )
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    try:
        shared.read_pinned_json(MATRIX, MATRIX_SHA)
        shared.read_pinned_json(SOURCE_SET, SOURCE_SET_SHA)
        protocol = stage_a.load_frozen_protocol(PROTOCOL)
        require(protocol.sha256 == PROTOCOL_SHA, "Protocol-v2 pin changed")
        if args.validate_only:
            print("PASS: Matrix-v6 advanced-screen contracts; candidate_audio_accessed=false")
            return 0
        output = run(args.matrix_result)
        output_path = args.matrix_result.parent / OUTPUT_NAME
        shared.create_exclusive_json(output_path, output)
    except Exception as error:
        print(f"FAIL: Matrix-v6 advanced screens stopped fail-closed: {error}")
        return 1
    print("PASS: Matrix-v6 advanced screens complete")
    print(f"advanced_survivor_count={output['advanced_survivor_count']}")
    print(f"result={output_path}")
    print("human_verdict=unverified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
