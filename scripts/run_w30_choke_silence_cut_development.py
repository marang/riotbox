#!/usr/bin/env python3
"""Build one bounded non-product W-30 silence-cut exploration artifact."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
import wave
from array import array
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from source_holdout_development_access import (
    create_exclusive_access_log,
    persist_access_log,
    validate_contained_source_file,
    validate_safe_token,
)


EXPECTED_CONTRACT_SHA256 = "2781cdc38bce556653d8c7584dbc8807f60a20747b4b2fee6f11f8678a90bd3f"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def load_pcm16(path: Path) -> tuple[wave._wave_params, array]:
    with wave.open(str(path), "rb") as source:
        params = source.getparams()
        payload = source.readframes(params.nframes)
    require(params.comptype == "NONE", "foundation must be uncompressed PCM")
    require(params.sampwidth == 2, "foundation must be PCM16")
    require(sys.byteorder == "little", "local exploration requires little-endian PCM")
    samples = array("h")
    samples.frombytes(payload)
    return params, samples


def write_pcm16(path: Path, params: wave._wave_params, samples: array) -> None:
    with wave.open(str(path), "wb") as target:
        target.setparams(params)
        target.writeframes(samples.tobytes())


def apply_choke(
    control: array,
    channels: int,
    cut_start: int,
    cut_end: int,
    fade_frames: int,
) -> tuple[array, int, int]:
    require(cut_start >= fade_frames, "fade-out begins before the artifact")
    require(cut_end + fade_frames < len(control) // channels, "fade-in exceeds artifact")
    candidate = array("h", control)
    fade_out_start = cut_start - fade_frames
    fade_in_end = cut_end + fade_frames

    for frame in range(fade_out_start, cut_start):
        progress = (frame - fade_out_start + 1) / fade_frames
        gain = math.cos(progress * math.pi / 2.0)
        for channel in range(channels):
            index = frame * channels + channel
            candidate[index] = round(control[index] * gain)
    for index in range(cut_start * channels, cut_end * channels):
        candidate[index] = 0
    for frame in range(cut_end, fade_in_end):
        progress = (frame - cut_end + 1) / fade_frames
        gain = math.sin(progress * math.pi / 2.0)
        for channel in range(channels):
            index = frame * channels + channel
            candidate[index] = round(control[index] * gain)
    return candidate, fade_out_start, fade_in_end


def boundary_delta(samples: array, channels: int, frames: list[int]) -> int:
    return max(
        abs(samples[frame * channels + channel] - samples[(frame - 1) * channels + channel])
        for frame in frames
        for channel in range(channels)
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--session", required=True)
    args = parser.parse_args()
    session = validate_safe_token(args.session, "session")

    repo = Path(__file__).resolve().parents[1]
    contract_path = repo / "docs/benchmarks/w30_choke_silence_cut_development_v1.json"
    contract_payload = contract_path.read_bytes()
    require(sha256(contract_payload) == EXPECTED_CONTRACT_SHA256, "frozen contract changed")
    contract = json.loads(contract_payload)
    require(contract["status"] == "frozen", "contract is not frozen")

    root = repo / "artifacts/development/riotbox-1472"
    access_log_path = root / f"access-log-{session}.json"
    output = root / "exploration-v1"
    require(not output.exists(), f"exploration output already exists: {output}")
    access_log: dict[str, Any] = {
        "schema": "riotbox.w30_choke_silence_cut_development_access.v1",
        "ticket": "RIOTBOX-1472",
        "session": session,
        "status": "started",
        "started_at_utc": utc_now(),
        "contract_sha256": EXPECTED_CONTRACT_SHA256,
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_opened": False,
        "source_open_count": 0,
        "candidate_rerendered": False,
    }

    with create_exclusive_access_log(access_log_path) as access_file:
        persist_access_log(access_file, access_log)
        try:
            registry = contract["development_source"]
            registry_payload = (repo / registry["registry_path"]).read_bytes()
            require(sha256(registry_payload) == registry["registry_sha256"], "registry changed")
            entry = next(
                item for item in json.loads(registry_payload)["entries"]
                if item["case_id"] == registry["case_id"]
            )
            require(entry["source_path"] == registry["source_path"], "registered path changed")
            require(entry["source_family"] == registry["source_family"], "source family changed")
            require(entry["bpm_hint"] == registry["bpm"], "registered BPM changed")

            foundation = contract["existing_foundation"]
            foundation_path = repo / foundation["path"]
            foundation_payload = foundation_path.read_bytes()
            require(sha256(foundation_payload) == foundation["sha256"], "foundation hash changed")
            params, control = load_pcm16(foundation_path)
            require(params.framerate == foundation["sample_rate_hz"], "sample rate changed")
            require(params.nchannels == foundation["channels"], "channel count changed")
            require(params.nframes == foundation["frame_count"], "frame count changed")

            def on_source_open(_: Path) -> None:
                access_log["source_open_count"] = 1
                access_log["status"] = "opening_exact_registered_source"
                persist_access_log(access_file, access_log)

            source_payload, source_result = validate_contained_source_file(
                repo,
                Path(registry["source_path"]),
                registry["source_sha256"],
                registry["source_format"],
                "RIOTBOX-1472 development source",
                on_open=on_source_open,
                return_payload=True,
            )
            require(access_log["source_open_count"] == 1, "source open was not recorded")

            variant = contract["variant_v1"]
            frames_per_beat = params.framerate * 60.0 / foundation["product_bpm"]
            cut_start = round(variant["silence_cut_start_beat"] * frames_per_beat)
            cut_end = round(variant["silence_cut_end_beat"] * frames_per_beat)
            fade_frames = round(params.framerate * variant["fade_milliseconds"] / 1000.0)
            candidate, fade_out_start, fade_in_end = apply_choke(
                control, params.nchannels, cut_start, cut_end, fade_frames
            )

            require(control[: fade_out_start * params.nchannels] == candidate[: fade_out_start * params.nchannels], "prefix changed")
            require(all(value == 0 for value in candidate[cut_start * params.nchannels : cut_end * params.nchannels]), "cut is not silent")
            require(control[fade_in_end * params.nchannels :] == candidate[fade_in_end * params.nchannels :], "return changed")
            require(max(abs(value) for value in candidate) <= max(abs(value) for value in control), "candidate peak increased")
            require(max(abs(value) for value in candidate) < 32768, "candidate clipped")
            edges = [fade_out_start, cut_start, cut_end, fade_in_end]
            allowed_edge_delta = max(boundary_delta(control, params.nchannels, edges), round(0.02 * 32768))
            require(boundary_delta(candidate, params.nchannels, edges) <= allowed_edge_delta, "candidate edge discontinuity failed")
            require(candidate != control, "candidate is identical to control")

            output.mkdir(parents=True, exist_ok=False)
            source_copy = output / "00_verified_source.wav"
            control_copy = output / "01_w30_control.wav"
            candidate_path = output / "02_w30_choke_silence_cut_v1.wav"
            source_copy.write_bytes(source_payload)
            control_copy.write_bytes(foundation_payload)
            write_pcm16(candidate_path, params, candidate)

            preflight = {
                "schema": "riotbox.w30_choke_silence_cut_development_preflight.v1",
                "ticket": "RIOTBOX-1472",
                "result": "technically_eligible_for_one_early_human_review",
                "source": {"sha256": sha256(source_payload), "format": source_result},
                "control_sha256": sha256(foundation_payload),
                "candidate_sha256": sha256(candidate_path.read_bytes()),
                "format": {"sample_rate_hz": params.framerate, "channels": params.nchannels, "sample_width_bits": 16, "frame_count": params.nframes},
                "window_frames": {"fade_out_start": fade_out_start, "cut_start": cut_start, "cut_end": cut_end, "fade_in_end": fade_in_end, "fade_frames": fade_frames},
                "silence_cut_frames": cut_end - cut_start,
                "control_peak_abs_pcm16": max(abs(value) for value in control),
                "candidate_peak_abs_pcm16": max(abs(value) for value in candidate),
                "candidate_clipped_samples": sum(abs(value) >= 32768 for value in candidate),
                "maximum_candidate_edge_delta_pcm16": boundary_delta(candidate, params.nchannels, edges),
                "allowed_edge_delta_pcm16": allowed_edge_delta,
                "candidate_rerendered": False,
                "product_behavior_changed": False,
                "playback_order": [str(source_copy), str(control_copy), str(candidate_path)],
            }
            preflight_path = output / "preflight.json"
            preflight_path.write_text(json.dumps(preflight, indent=2) + "\n")
            access_log.update({
                "status": "completed",
                "completed_at_utc": utc_now(),
                "source_sha256": sha256(source_payload),
                "control_sha256": sha256(foundation_payload),
                "candidate_sha256": preflight["candidate_sha256"],
                "preflight_sha256": sha256(preflight_path.read_bytes()),
            })
            persist_access_log(access_file, access_log)
            print(preflight_path.relative_to(repo))
            return 0
        except Exception as error:
            access_log["status"] = "failed_closed"
            access_log["completed_at_utc"] = utc_now()
            access_log["failure"] = f"{type(error).__name__}: {error}"
            persist_access_log(access_file, access_log)
            raise


if __name__ == "__main__":
    raise SystemExit(main())
