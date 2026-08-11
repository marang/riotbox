#!/usr/bin/env python3
"""Split the dense v1 sanity artifact into repeated per-instrument reviews."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import wave
from pathlib import Path
from typing import Any

import numpy as np


ROOT = Path(__file__).resolve().parents[1]
INPUT_DIR = Path("artifacts/audio_qa/riotbox-1434/natural-velocity-human-sanity-v1")
INPUT_WAV = INPUT_DIR / "natural-velocity-bidirectional.wav"
INPUT_WAV_SHA256 = "fd61c45f3d50c1e4a4d4dae55575ce1508d69bbc454d5325312aa83074236847"
INPUT_ANALYSIS = INPUT_DIR / "technical-analysis.json"
INPUT_ANALYSIS_SHA256 = "aef4b230a3aaa8894b1a74abf22d1c78dc1dd8246bb1c0a3f6a04fcc8a65e0e0"
OUTPUT_DIR = Path("artifacts/audio_qa/riotbox-1434/natural-velocity-human-sanity-v2")
SAMPLE_RATE = 48_000
CHANNELS = 2
SAMPLE_WIDTH = 2
HIT_GAP_FRAMES = round(0.25 * SAMPLE_RATE)
PAIR_GAP_FRAMES = round(0.25 * SAMPLE_RATE)
ORDER_GAP_FRAMES = round(0.75 * SAMPLE_RATE)
ENDPOINT_FRAMES = round(0.2 * SAMPLE_RATE)
REPETITIONS = 3


class ArtifactError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ArtifactError(message)


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def load_input() -> tuple[dict[str, Any], bytes]:
    analysis_payload = (ROOT / INPUT_ANALYSIS).read_bytes()
    wav_payload = (ROOT / INPUT_WAV).read_bytes()
    require(sha256(analysis_payload) == INPUT_ANALYSIS_SHA256, "v1 analysis changed")
    require(sha256(wav_payload) == INPUT_WAV_SHA256, "v1 WAV changed")
    analysis = json.loads(analysis_payload)
    require(analysis.get("artifact_sha256") == INPUT_WAV_SHA256, "v1 manifest binding changed")
    return analysis, wav_payload


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def append_audio(output: bytearray, frames: bytes, segments: list[dict[str, Any]], case_id: str, block: str) -> None:
    start = len(output) // (CHANNELS * SAMPLE_WIDTH)
    output.extend(frames)
    segments.append(
        {
            "kind": "control_audio_from_verified_v1_artifact",
            "case_id": case_id,
            "block": block,
            "start_frame": start,
            "end_frame": len(output) // (CHANNELS * SAMPLE_WIDTH),
            "frame_sha256": sha256(frames),
        }
    )


def append_silence(output: bytearray, segments: list[dict[str, Any]], frames: int, role: str) -> None:
    start = len(output) // (CHANNELS * SAMPLE_WIDTH)
    output.extend(bytes(frames * CHANNELS * SAMPLE_WIDTH))
    segments.append({"kind": "exact_zero_silence", "role": role, "start_frame": start, "end_frame": start + frames})


def render_instrument(
    name: str,
    primary: list[str],
    source_frames: dict[str, bytes],
    output_dir: Path,
) -> dict[str, Any]:
    output = bytearray()
    segments: list[dict[str, Any]] = []
    for order_name, order in (("primary", primary), ("reversed", list(reversed(primary)))):
        for repetition in range(REPETITIONS):
            append_audio(output, source_frames[order[0]], segments, order[0], f"{order_name}_{repetition + 1}")
            append_silence(output, segments, HIT_GAP_FRAMES, "between_hits")
            append_audio(output, source_frames[order[1]], segments, order[1], f"{order_name}_{repetition + 1}")
            if repetition + 1 < REPETITIONS:
                append_silence(output, segments, PAIR_GAP_FRAMES, "between_repetitions")
        if order_name == "primary":
            append_silence(output, segments, ORDER_GAP_FRAMES, "between_orders")
    append_silence(output, segments, ENDPOINT_FRAMES, "endpoint")
    wav_path = output_dir / f"{name}-bidirectional-repeated.wav"
    with wave.open(str(wav_path), "wb") as target:
        target.setnchannels(CHANNELS)
        target.setsampwidth(SAMPLE_WIDTH)
        target.setframerate(SAMPLE_RATE)
        target.writeframes(output)
    samples = np.frombuffer(output, dtype="<i2").astype(np.float64) / float(1 << 15)
    duration = len(output) / (SAMPLE_RATE * CHANNELS * SAMPLE_WIDTH)
    require(duration <= 10.0, f"{name}: duration exceeds 10 seconds")
    require(float(np.max(np.abs(samples))) < 1.0, f"{name}: artifact clips")
    require(bool(np.all(samples[-ENDPOINT_FRAMES * CHANNELS :] == 0.0)), f"{name}: endpoint is not silent")
    return {
        "instrument": name,
        "artifact_path": wav_path.as_posix(),
        "artifact_sha256": sha256(wav_path.read_bytes()),
        "duration_seconds": duration,
        "absolute_peak": float(np.max(np.abs(samples))),
        "rms": float(math.sqrt(float(np.mean(np.square(samples))))),
        "clipped": False,
        "endpoint_silence_seconds": ENDPOINT_FRAMES / SAMPLE_RATE,
        "segments": segments,
    }


def build() -> dict[str, Any]:
    require(not (ROOT / OUTPUT_DIR).exists(), f"output already exists: {ROOT / OUTPUT_DIR}")
    analysis, _ = load_input()
    with wave.open(str(ROOT / INPUT_WAV), "rb") as source:
        require((source.getframerate(), source.getnchannels(), source.getsampwidth()) == (SAMPLE_RATE, CHANNELS, SAMPLE_WIDTH), "v1 WAV format changed")
        all_frames = source.readframes(source.getnframes())
    source_frames: dict[str, bytes] = {}
    primary: dict[str, list[str]] = {"snare": [], "whip": []}
    for segment in analysis["segments"]:
        if segment["kind"] != "control_audio" or not segment["block"].endswith("_primary"):
            continue
        instrument = "snare" if segment["block"].startswith("snare") else "whip"
        case_id = segment["case_id"]
        if case_id in source_frames:
            continue
        start = segment["start_frame"] * CHANNELS * SAMPLE_WIDTH
        end = segment["end_frame"] * CHANNELS * SAMPLE_WIDTH
        frames = all_frames[start:end]
        require(sha256(frames) == segment["frame_sha256"], f"{case_id}: segment identity changed")
        source_frames[case_id] = frames
        primary[instrument].append(case_id)
    require(len(source_frames) == 4 and all(len(order) == 2 for order in primary.values()), "v1 primary pairs unresolved")
    (ROOT / OUTPUT_DIR).mkdir(parents=True)
    artifacts = [
        render_instrument("snare", primary["snare"], source_frames, ROOT / OUTPUT_DIR),
        render_instrument("whip", primary["whip"], source_frames, ROOT / OUTPUT_DIR),
    ]
    result = {
        "schema": "riotbox.percussive_force_natural_velocity_human_artifact.v2",
        "owner_ticket": "RIOTBOX-1434",
        "status": "technically_analyzed_human_readiness_pending",
        "input_artifact_sha256": INPUT_WAV_SHA256,
        "source_control_audio_reread": False,
        "repetitions_per_order": REPETITIONS,
        "artifacts": artifacts,
        "question": "For each instrument, identify the stronger member across three primary repetitions and confirm that the positional answer reverses across three reversed repetitions.",
        "human_verdict": "unverified",
        "algorithm_selection_allowed": False,
        "hardness_proof": False,
    }
    write_json(ROOT / OUTPUT_DIR / "technical-analysis.json", result)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    try:
        load_input()
        if args.validate_only:
            print("PASS: v2 repeated presentation preflight; control_source_audio_accessed=false")
            return 0
        result = build()
    except (ArtifactError, OSError, ValueError, KeyError, TypeError, json.JSONDecodeError, wave.Error) as error:
        print(f"FAIL: v2 repeated presentation stopped: {error}")
        return 1
    for artifact in result["artifacts"]:
        print(f"PASS {artifact['instrument']}: duration_seconds={artifact['duration_seconds']}")
    print("human_verdict=unverified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
