#!/usr/bin/env python3
"""Analyze the six exact RIOTBOX-1434 natural-dynamic controls once."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import sys
import wave
from pathlib import Path
from typing import Any

import numpy as np

from source_holdout_development_access import read_contained_regular_file


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = Path("docs/benchmarks/percussive_force_natural_velocity_controls_v1.json")
MATRIX = Path("docs/benchmarks/percussive_force_development_matrix_v2.json")
MATRIX_SHA256 = "aba846138246c95b1c3e5e1973e77bdaa41ce971f799dadadba8edc160967fd6"
OUTPUT_REL = Path("artifacts/audio_qa/riotbox-1434/natural-velocity-controls-v1")
MAXIMUM_FILE_BYTES = 32 * 1024 * 1024
ORDER = ("mezzo_forte", "forte", "fortissimo")
FEATURES = (
    "attack_time_ms",
    "decay_time_ms",
    "body_resonance_hz",
    "attack_brightness_centroid_hz",
)


class ControlError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ControlError(message)


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def load_json(path: Path) -> tuple[dict[str, Any], bytes]:
    payload = (ROOT / path).read_bytes()
    def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        value: dict[str, Any] = {}
        for key, item in pairs:
            require(key not in value, f"{path}: duplicate JSON key: {key}")
            value[key] = item
        return value

    value = json.loads(
        payload,
        object_pairs_hook=unique_object,
        parse_constant=lambda token: (_ for _ in ()).throw(ValueError(token)),
    )
    require(isinstance(value, dict), f"{path}: root must be an object")
    return value, payload


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def decode_pcm_wave(payload: bytes, case_id: str) -> tuple[np.ndarray, int, dict[str, Any]]:
    try:
        with wave.open(io.BytesIO(payload), "rb") as source:
            channels = source.getnchannels()
            sample_rate = source.getframerate()
            sample_width = source.getsampwidth()
            frame_count = source.getnframes()
            compression = source.getcomptype()
            frames = source.readframes(frame_count)
            require(source.readframes(1) == b"", f"{case_id}: trailing decoded frames")
    except (wave.Error, EOFError) as error:
        raise ControlError(f"{case_id}: unsupported RIFF/WAVE: {error}") from error
    require(compression == "NONE", f"{case_id}: compressed WAV is forbidden")
    require(channels in (1, 2), f"{case_id}: only mono/stereo is allowed")
    require(sample_rate in (44_100, 48_000, 96_000), f"{case_id}: unsupported sample rate")
    require(sample_width in (2, 3), f"{case_id}: only PCM16/PCM24 is allowed")
    expected_bytes = frame_count * channels * sample_width
    require(len(frames) == expected_bytes, f"{case_id}: PCM byte count mismatch")
    if sample_width == 2:
        integer = np.frombuffer(frames, dtype="<i2").astype(np.int32)
        scale = float(1 << 15)
    else:
        octets = np.frombuffer(frames, dtype=np.uint8).reshape(-1, 3)
        integer = (
            octets[:, 0].astype(np.int32)
            | (octets[:, 1].astype(np.int32) << 8)
            | (octets[:, 2].astype(np.int32) << 16)
        )
        integer = np.where(integer & 0x800000, integer - 0x1000000, integer)
        scale = float(1 << 23)
    samples = (integer.astype(np.float64) / scale).reshape(frame_count, channels)
    require(samples.size > 0 and np.isfinite(samples).all(), f"{case_id}: invalid PCM")
    return samples, sample_rate, {
        "sample_rate_hz": sample_rate,
        "channel_count": channels,
        "sample_width_bits": sample_width * 8,
        "frame_count": frame_count,
        "duration_seconds": frame_count / sample_rate,
    }


def moving_rms(samples: np.ndarray, frames: int) -> np.ndarray:
    energy = np.mean(np.square(samples), axis=1)
    cumulative = np.concatenate(([0.0], np.cumsum(energy)))
    index = np.arange(energy.size)
    start = np.maximum(0, index + 1 - frames)
    counts = index + 1 - start
    return np.sqrt((cumulative[index + 1] - cumulative[start]) / counts)


def first_persistent(values: np.ndarray, start: int, end: int, frames: int, predicate: Any) -> int | None:
    latest = min(end, values.size - frames)
    for index in range(max(0, start), latest + 1):
        if bool(np.all(predicate(values[index : index + frames]))):
            return index
    return None


def spectrum_metrics(samples: np.ndarray, sample_rate: int, start: int, end: int) -> tuple[float, float]:
    segment = samples[start:end]
    require(segment.shape[0] >= 16, "spectral window is too short")
    mono = np.mean(segment, axis=1)
    windowed = (mono - np.mean(mono)) * np.hanning(mono.size)
    size = 1 << max(4, (windowed.size - 1).bit_length())
    power = np.square(np.abs(np.fft.rfft(windowed, n=size)))
    frequencies = np.fft.rfftfreq(size, d=1.0 / sample_rate)
    positive = frequencies > 0.0
    total = float(np.sum(power[positive]))
    require(total > 0.0 and math.isfinite(total), "spectral energy is unavailable")
    centroid = float(np.sum(frequencies[positive] * power[positive]) / total)
    resonance_mask = (frequencies >= 40.0) & (frequencies <= 2_000.0)
    require(bool(np.any(resonance_mask)), "body resonance bins are unavailable")
    resonance_indices = np.flatnonzero(resonance_mask)
    resonance = float(frequencies[resonance_indices[int(np.argmax(power[resonance_mask]))]])
    return centroid, resonance


def analyze(samples: np.ndarray, sample_rate: int, case_id: str, parameters: dict[str, Any]) -> dict[str, Any]:
    centered = samples - np.mean(samples, axis=0, keepdims=True)
    envelope_frames = max(1, round(sample_rate * parameters["envelope_ms"] / 1_000.0))
    persistence_frames = max(1, round(sample_rate * parameters["persistence_ms"] / 1_000.0))
    envelope = moving_rms(centered, envelope_frames)
    peak_index = int(np.argmax(envelope))
    peak = float(envelope[peak_index])
    baseline = float(np.percentile(envelope, parameters["baseline_percentile"], method="lower"))
    require(peak > baseline and peak > 0.0, f"{case_id}: no usable impact peak")
    onset_threshold = baseline + parameters["onset_fraction"] * (peak - baseline)
    below = np.flatnonzero(envelope[: peak_index + 1] < onset_threshold)
    require(below.size > 0, f"{case_id}: physical onset has no resolved lookbehind")
    onset = int(below[-1] + 1)
    attack_limit = min(
        envelope.size - 1,
        onset + round(sample_rate * parameters["attack_search_ms"] / 1_000.0),
    )
    attack_peak = onset + int(np.argmax(envelope[onset : attack_limit + 1]))
    attack_end = first_persistent(
        envelope,
        attack_peak + 1,
        attack_limit,
        persistence_frames,
        lambda values: values <= parameters["attack_end_peak_fraction"] * peak,
    )
    require(attack_end is not None, f"{case_id}: attack end unresolved")
    decay_limit = min(
        envelope.size - 1,
        attack_peak + round(sample_rate * parameters["decay_search_ms"] / 1_000.0),
    )
    decay_end = first_persistent(
        envelope,
        attack_end + 1,
        decay_limit,
        persistence_frames,
        lambda values: values <= baseline + parameters["decay_fraction"] * (peak - baseline),
    )
    require(decay_end is not None, f"{case_id}: decay end unresolved")
    attack_spectral_end = min(
        centered.shape[0],
        onset + round(sample_rate * parameters["attack_spectrum_ms"] / 1_000.0),
    )
    brightness, _ = spectrum_metrics(centered, sample_rate, onset, attack_spectral_end)
    body_start = max(attack_end, onset + envelope_frames)
    _, resonance = spectrum_metrics(centered, sample_rate, body_start, decay_end)
    event = centered[onset:decay_end]
    event_rms = float(np.sqrt(np.mean(np.square(event))))
    require(event_rms > 0.0, f"{case_id}: event RMS unavailable")
    return {
        "physical_onset_frame": onset,
        "attack_peak_frame": attack_peak,
        "attack_end_frame": attack_end,
        "decay_end_frame": decay_end,
        "attack_time_ms": 1_000.0 * (attack_peak - onset) / sample_rate,
        "decay_time_ms": 1_000.0 * (decay_end - attack_peak) / sample_rate,
        "body_resonance_hz": resonance,
        "attack_brightness_centroid_hz": brightness,
        "event_rms": event_rms,
        "absolute_peak": float(np.max(np.abs(event))),
    }


def sign(left: float, right: float) -> str:
    tolerance = 64.0 * sys.float_info.epsilon * max(1.0, abs(left), abs(right))
    if right > left + tolerance:
        return "increase"
    if right < left - tolerance:
        return "decrease"
    return "numerically_equal"


def directions(members: list[dict[str, Any]]) -> dict[str, Any]:
    by_dynamic = {member["provisional_dynamic"]: member for member in members}
    require(tuple(by_dynamic) == ORDER, "control member order changed")
    result: dict[str, Any] = {}
    for feature in FEATURES:
        values = [float(by_dynamic[level]["analysis"][feature]) for level in ORDER]
        adjacent = [sign(values[0], values[1]), sign(values[1], values[2])]
        result[feature] = {
            "values_in_provisional_order": values,
            "adjacent_directions": adjacent,
            "extreme_direction": sign(values[0], values[2]),
            "strictly_monotonic": adjacent[0] == adjacent[1]
            and adjacent[0] != "numerically_equal",
        }
    return result


def validate_contract(contract: dict[str, Any], contract_payload: bytes) -> None:
    require(contract.get("schema") == "riotbox.percussive_force_natural_velocity_controls.v1", "contract schema changed")
    require(contract.get("owner_ticket") == "RIOTBOX-1434", "contract owner changed")
    require(contract.get("matrix_v2", {}).get("raw_sha256") == MATRIX_SHA256, "Matrix-v2 pin changed")
    matrix, matrix_payload = load_json(MATRIX)
    require(sha256(matrix_payload) == MATRIX_SHA256, "Matrix-v2 bytes changed")
    runner = Path(str(contract["runner"]["path"]))
    require(sha256((ROOT / runner).read_bytes()) == contract["runner"]["raw_sha256"], "runner pin changed")
    controls = contract.get("controls")
    require(isinstance(controls, list) and len(controls) == 2, "exactly two control sets required")
    members = [member for control in controls for member in control.get("members", [])]
    require(len(members) == 6, "exactly six control files required")
    require(len({member["repo_path"] for member in members}) == 6, "duplicate control path")
    require(len({member["sha256"] for member in members}) == 6, "duplicate control hash")
    require(
        [member["provisional_dynamic"] for control in controls for member in control["members"]]
        == list(ORDER) * 2,
        "control dynamic order changed",
    )
    catalog = matrix.get("natural_directional_reference_controls", {})
    expected_controls = []
    for source_set in catalog.get("sets", []):
        expected_controls.append(
            {
                "control_set_id": source_set["control_set_id"],
                "instrument": source_set["instrument"],
                "articulation": source_set["articulation"],
                "members": [
                    {
                        "provisional_dynamic": member["provisional_dynamic"],
                        "repo_path": f"data/test_audio/external/{member['local_path']}",
                        "sha256": member["sha256"],
                    }
                    for member in source_set["members"]
                ],
            }
        )
    comparable_controls = [
        {
            "control_set_id": control["control_set_id"],
            "instrument": control["instrument"],
            "articulation": control["articulation"],
            "members": [
                {
                    "provisional_dynamic": member["provisional_dynamic"],
                    "repo_path": member["repo_path"],
                    "sha256": member["sha256"],
                }
                for member in control["members"]
            ],
        }
        for control in controls
    ]
    require(comparable_controls == expected_controls, "control catalog diverged from Matrix-v2")
    require(
        contract.get("analysis")
        == {
            "attack_end_peak_fraction": 0.5,
            "attack_search_ms": 50.0,
            "attack_spectrum_ms": 50.0,
            "baseline_percentile": 20,
            "decay_fraction": 0.1,
            "decay_search_ms": 1000.0,
            "envelope_ms": 1.0,
            "onset_fraction": 0.05,
            "persistence_ms": 2.0,
        },
        "analysis passport changed",
    )
    access = contract.get("access", {})
    require(
        access.get("maximum_file_reads") == 6
        and access.get("one_read_per_file") is True
        and access.get("directory_discovery") is False
        and access.get("development_audio_access") is False
        and access.get("holdout_audio_access") is False
        and access.get("commercial_reference_audio_access") is False,
        "access boundary changed",
    )
    require(
        contract.get("execution", {}).get("exact_output_path") == OUTPUT_REL.as_posix()
        and contract.get("execution", {}).get("rerun_allowed") is False,
        "single-run output boundary changed",
    )


def run(output_dir: Path) -> dict[str, Any]:
    require(output_dir == ROOT / OUTPUT_REL, f"output must be exactly {ROOT / OUTPUT_REL}")
    require(not output_dir.exists(), f"output already exists: {output_dir}")
    contract, contract_payload = load_json(CONTRACT)
    validate_contract(contract, contract_payload)
    output_dir.mkdir(parents=True)
    log_path = output_dir / "access-log.json"
    log: dict[str, Any] = {
        "schema": "riotbox.percussive_force_natural_velocity_access.v1",
        "owner_ticket": "RIOTBOX-1434",
        "status": "started",
        "contract_sha256": sha256(contract_payload),
        "records": [],
        "directory_discovery_performed": False,
        "development_audio_accessed": False,
        "holdout_audio_accessed": False,
        "commercial_reference_audio_accessed": False,
    }
    write_json(log_path, log)
    analyzed_sets = []
    try:
        for control in contract["controls"]:
            analyzed_members = []
            for member in control["members"]:
                record = {
                    "access_ordinal": len(log["records"]) + 1,
                    "case_id": member["case_id"],
                    "repo_path": member["repo_path"],
                    "expected_sha256": member["sha256"],
                    "status": "opening_exact_registered_control",
                }
                log["records"].append(record)
                write_json(log_path, log)
                payload = read_contained_regular_file(
                    ROOT,
                    Path(member["repo_path"]),
                    f"RIOTBOX-1434:{member['case_id']}",
                    maximum_bytes=MAXIMUM_FILE_BYTES,
                )
                actual_sha = sha256(payload)
                require(actual_sha == member["sha256"], f"{member['case_id']}: SHA-256 changed")
                samples, sample_rate, source_format = decode_pcm_wave(payload, member["case_id"])
                analysis = analyze(samples, sample_rate, member["case_id"], contract["analysis"])
                record.update(
                    status="verified_and_analyzed",
                    byte_count=len(payload),
                    actual_sha256=actual_sha,
                    source_format=source_format,
                )
                write_json(log_path, log)
                analyzed_members.append({**member, "source_format": source_format, "analysis": analysis})
            analyzed_sets.append({
                "control_set_id": control["control_set_id"],
                "instrument": control["instrument"],
                "articulation": control["articulation"],
                "members": analyzed_members,
                "technical_directions": directions(analyzed_members),
                "human_directional_sanity": "pending",
            })
        log["status"] = "completed"
        write_json(log_path, log)
    except Exception:
        log["status"] = "rejected_fail_closed"
        write_json(log_path, log)
        raise
    access_payload = log_path.read_bytes()
    result = {
        "schema": "riotbox.percussive_force_natural_velocity_analysis.v1",
        "owner_ticket": "RIOTBOX-1434",
        "status": "technical_analysis_complete_human_sanity_pending",
        "contract_sha256": sha256(contract_payload),
        "access_log_path": log_path.as_posix(),
        "access_log_sha256": sha256(access_payload),
        "control_sets": analyzed_sets,
        "algorithm_selection_allowed": False,
        "perceptual_threshold_fitting_allowed": False,
        "hardness_proof": False,
        "quality_proof": False,
        "human_verdict": "unverified",
        "development_audio_accessed": False,
        "holdout_audio_accessed": False,
        "commercial_reference_audio_accessed": False,
    }
    write_json(output_dir / "technical-analysis.json", result)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    try:
        contract, payload = load_json(CONTRACT)
        validate_contract(contract, payload)
        if args.validate_only:
            print("PASS: natural velocity control contract; source_audio_accessed=false")
            return 0
        require(args.output is not None, "--output is required for the one authorized execution")
        output = args.output if args.output.is_absolute() else ROOT / args.output
        result = run(output)
    except (ControlError, OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        print(f"FAIL: natural velocity controls stopped fail-closed: {error}")
        return 1
    print("PASS: six natural velocity controls analyzed")
    print(f"result={args.output / 'technical-analysis.json'}")
    print(f"control_set_count={len(result['control_sets'])}")
    print("human_verdict=unverified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
