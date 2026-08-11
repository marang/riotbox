#!/usr/bin/env python3
"""Build one sub-10-second bidirectional natural-control listening artifact."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import wave
from pathlib import Path
from typing import Any

import numpy as np

import run_percussive_force_natural_velocity_controls_v1 as v1
import run_percussive_force_natural_velocity_controls_v2 as v2
from source_holdout_development_access import read_contained_regular_file


TECHNICAL_ANALYSIS = Path("artifacts/audio_qa/riotbox-1434/natural-velocity-controls-v2/technical-analysis.json")
TECHNICAL_ANALYSIS_SHA256 = "1db9d36445402baf07374dc9846906d42c1a258057653a524b51b5abe6e32ade"
ACCESS_LOG = Path("artifacts/audio_qa/riotbox-1434/natural-velocity-controls-v2/access-log.json")
ACCESS_LOG_SHA256 = "4c95af0e816a5ed87c4f9fc0d648a0b59d433d0992314f1ba02d4c67847f04ca"
OUTPUT_REL = Path("artifacts/audio_qa/riotbox-1434/natural-velocity-human-sanity-v1")
SAMPLE_RATE = 48_000
CHANNELS = 2
SAMPLE_WIDTH = 2
GAPS_SECONDS = {"between_hits": 0.35, "between_orders": 0.8, "between_sets": 1.0, "endpoint": 0.2}


def validate_inputs() -> tuple[dict[str, Any], list[dict[str, Any]]]:
    contract, _ = v1.load_json(v2.CONTRACT)
    predecessor, matrix_v2 = v2.validate_contract(contract)
    v1.require(v1.sha256((v1.ROOT / TECHNICAL_ANALYSIS).read_bytes()) == TECHNICAL_ANALYSIS_SHA256, "technical analysis changed")
    v1.require(v1.sha256((v1.ROOT / ACCESS_LOG).read_bytes()) == ACCESS_LOG_SHA256, "completed access log changed")
    analysis, _ = v1.load_json(TECHNICAL_ANALYSIS)
    v1.require(analysis.get("status") == "technical_analysis_complete_human_sanity_pending", "technical gate status changed")
    v1.require(analysis.get("human_verdict") == "unverified", "human gate already resolved")
    controls = v2.exact_controls(predecessor, matrix_v2)
    for control in controls:
        orientation_hash = hashlib.sha256(
            f"{contract['runner']['raw_sha256']}:{control['control_set_id']}:primary".encode()
        ).hexdigest()
        v1.require(len(orientation_hash) == 64, "primary orientation hash failed")
    return contract, controls


def read_frames(member: dict[str, Any], log: dict[str, Any], log_path: Path) -> tuple[bytes, dict[str, Any]]:
    record = {
        "access_ordinal": len(log["records"]) + 1,
        "case_id": member["case_id"],
        "repo_path": member["repo_path"],
        "expected_sha256": member["sha256"],
        "status": "opening_exact_registered_control_for_human_artifact",
    }
    log["records"].append(record)
    v1.write_json(log_path, log)
    payload = read_contained_regular_file(
        v1.ROOT,
        Path(member["repo_path"]),
        f"RIOTBOX-1434-human:{member['case_id']}",
        maximum_bytes=v1.MAXIMUM_FILE_BYTES,
    )
    actual_sha = v1.sha256(payload)
    v1.require(actual_sha == member["sha256"], f"{member['case_id']}: SHA-256 changed")
    with wave.open(io.BytesIO(payload), "rb") as source:
        v1.require(source.getframerate() == SAMPLE_RATE, f"{member['case_id']}: sample rate changed")
        v1.require(source.getnchannels() == CHANNELS, f"{member['case_id']}: channels changed")
        v1.require(source.getsampwidth() == SAMPLE_WIDTH, f"{member['case_id']}: sample width changed")
        v1.require(source.getcomptype() == "NONE", f"{member['case_id']}: compression changed")
        frame_count = source.getnframes()
        frames = source.readframes(frame_count)
    v1.require(len(frames) == frame_count * CHANNELS * SAMPLE_WIDTH, f"{member['case_id']}: frame bytes changed")
    record.update(status="verified_and_loaded", actual_sha256=actual_sha, frame_count=frame_count)
    log["control_audio_accessed"] = True
    v1.write_json(log_path, log)
    return frames, {"frame_count": frame_count, "frame_sha256": v1.sha256(frames)}


def append_audio(output: bytearray, segments: list[dict[str, Any]], frames: bytes, member: dict[str, Any], block: str) -> None:
    start = len(output) // (CHANNELS * SAMPLE_WIDTH)
    output.extend(frames)
    end = len(output) // (CHANNELS * SAMPLE_WIDTH)
    segments.append({"kind": "control_audio", "block": block, "case_id": member["case_id"], "start_frame": start, "end_frame": end, "frame_sha256": v1.sha256(frames)})


def append_silence(output: bytearray, segments: list[dict[str, Any]], seconds: float, role: str) -> None:
    start = len(output) // (CHANNELS * SAMPLE_WIDTH)
    frame_count = round(SAMPLE_RATE * seconds)
    output.extend(bytes(frame_count * CHANNELS * SAMPLE_WIDTH))
    segments.append({"kind": "exact_zero_silence", "role": role, "start_frame": start, "end_frame": start + frame_count})


def build(output_dir: Path) -> dict[str, Any]:
    v1.require(output_dir == v1.ROOT / OUTPUT_REL, f"output must be exactly {v1.ROOT / OUTPUT_REL}")
    v1.require(not output_dir.exists(), f"output already exists: {output_dir}")
    contract, controls = validate_inputs()
    output_dir.mkdir(parents=True)
    log_path = output_dir / "source-access-log.json"
    log: dict[str, Any] = {
        "schema": "riotbox.percussive_force_natural_velocity_human_access.v1",
        "owner_ticket": "RIOTBOX-1434",
        "status": "started",
        "records": [],
        "control_audio_accessed": False,
        "directory_discovery_performed": False,
        "development_audio_accessed": False,
        "holdout_audio_accessed": False,
        "commercial_reference_audio_accessed": False,
    }
    v1.write_json(log_path, log)
    loaded: dict[str, tuple[bytes, dict[str, Any], dict[str, Any]]] = {}
    try:
        for control in controls:
            by_dynamic = {member["provisional_dynamic"]: member for member in control["members"]}
            for dynamic in ("mezzo_forte", "fortissimo"):
                member = by_dynamic[dynamic]
                frames, metadata = read_frames(member, log, log_path)
                loaded[member["case_id"]] = (frames, metadata, member)
        output = bytearray()
        segments: list[dict[str, Any]] = []
        for set_index, control in enumerate(controls):
            by_dynamic = {member["provisional_dynamic"]: member for member in control["members"]}
            orientation_hash = hashlib.sha256(
                f"{contract['runner']['raw_sha256']}:{control['control_set_id']}:primary".encode()
            ).digest()
            dynamics = ("mezzo_forte", "fortissimo") if orientation_hash[0] % 2 == 0 else ("fortissimo", "mezzo_forte")
            primary = [by_dynamic[dynamic] for dynamic in dynamics]
            for order_name, order in (("primary", primary), ("reversed", list(reversed(primary)))):
                for hit_index, member in enumerate(order):
                    frames, _, _ = loaded[member["case_id"]]
                    append_audio(output, segments, frames, member, f"{control['instrument']}_{order_name}")
                    if hit_index == 0:
                        append_silence(output, segments, GAPS_SECONDS["between_hits"], "between_hits")
                if order_name == "primary":
                    append_silence(output, segments, GAPS_SECONDS["between_orders"], "between_orders")
            if set_index == 0:
                append_silence(output, segments, GAPS_SECONDS["between_sets"], "between_sets")
        append_silence(output, segments, GAPS_SECONDS["endpoint"], "endpoint")
        wav_path = output_dir / "natural-velocity-bidirectional.wav"
        with wave.open(str(wav_path), "wb") as target:
            target.setnchannels(CHANNELS)
            target.setsampwidth(SAMPLE_WIDTH)
            target.setframerate(SAMPLE_RATE)
            target.writeframes(output)
        log["status"] = "completed"
        v1.write_json(log_path, log)
    except Exception:
        log["status"] = "rejected_fail_closed"
        v1.write_json(log_path, log)
        raise
    samples = np.frombuffer(output, dtype="<i2").astype(np.float64) / float(1 << 15)
    duration = len(output) / (SAMPLE_RATE * CHANNELS * SAMPLE_WIDTH)
    result = {
        "schema": "riotbox.percussive_force_natural_velocity_human_artifact.v1",
        "owner_ticket": "RIOTBOX-1434",
        "status": "technically_analyzed_human_readiness_pending",
        "source_technical_analysis_sha256": TECHNICAL_ANALYSIS_SHA256,
        "artifact_path": wav_path.as_posix(),
        "artifact_sha256": v1.sha256(wav_path.read_bytes()),
        "format": {"sample_rate_hz": SAMPLE_RATE, "channel_count": CHANNELS, "sample_width_bits": 16},
        "duration_seconds": duration,
        "absolute_peak": float(np.max(np.abs(samples))),
        "rms": float(math.sqrt(float(np.mean(np.square(samples))))),
        "clipped": bool(np.max(np.abs(samples)) >= 1.0),
        "endpoint_silence_seconds": GAPS_SECONDS["endpoint"],
        "segments": segments,
        "source_access_log_sha256": v1.sha256(log_path.read_bytes()),
        "question": "For snare and whip separately, which member sounds more forcefully struck in the primary order, and does the answer stay the same in the reversed order?",
        "human_verdict": "unverified",
        "hardness_proof": False,
        "algorithm_selection_allowed": False,
    }
    v1.require(duration <= 10.0, "human artifact exceeds 10 seconds")
    v1.require(not result["clipped"], "human artifact clips")
    v1.write_json(output_dir / "technical-analysis.json", result)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    try:
        validate_inputs()
        if args.validate_only:
            print("PASS: human sanity builder preflight; source_audio_accessed=false")
            return 0
        v1.require(args.output is not None, "--output is required")
        output = args.output if args.output.is_absolute() else v1.ROOT / args.output
        result = build(output)
    except (v1.ControlError, OSError, ValueError, KeyError, TypeError, json.JSONDecodeError, wave.Error) as error:
        print(f"FAIL: human sanity artifact stopped fail-closed: {error}")
        return 1
    print(f"PASS: human sanity artifact built; duration_seconds={result['duration_seconds']}")
    print("human_verdict=unverified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
