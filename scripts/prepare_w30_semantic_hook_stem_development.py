#!/usr/bin/env python3
"""Prepare the single frozen RIOTBOX-1482 Development listening candidate."""

from __future__ import annotations

import hashlib
import json
import math
import os
import stat
import sys
import wave
from array import array
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
CONTRACT_PATH = ROOT / "docs/benchmarks/w30_semantic_hook_stem_development_v1.json"
CONTRACT_SHA256 = "8e8e96b75ae525148fb94a99248cf98427ff882cb7e06c1bb1500abef2f22da7"
OUTPUT_DIR = ROOT / "artifacts/development/riotbox-1482/semantic-hook-v1"
ACCESS_LOG_PATH = OUTPUT_DIR / "access-log.json"
REPORT_PATH = OUTPUT_DIR / "technical-report.json"
LOOP_PATH = OUTPUT_DIR / "01_w30_hook_loop_v1.wav"
RERENDER_PATH = OUTPUT_DIR / "01_w30_hook_loop_v1_rerender.wav"
REVIEW_PATH = OUTPUT_DIR / "02_w30_hook_loop_v1_review.wav"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def contained_regular_file(relative: str) -> Path:
    path = Path(relative)
    require(not path.is_absolute() and path.parts, f"unsafe path: {relative}")
    require(all(part not in {"", ".", ".."} for part in path.parts), f"unsafe path: {relative}")
    current = ROOT
    for part in path.parts:
        current = current / part
        info = os.lstat(current)
        require(not stat.S_ISLNK(info.st_mode), f"symlink rejected: {relative}")
    require(current.is_file(), f"missing regular file: {relative}")
    return current


def read_json(path: Path) -> tuple[bytes, dict[str, Any]]:
    payload = path.read_bytes()
    value = json.loads(payload)
    require(isinstance(value, dict), f"JSON root is not an object: {path}")
    return payload, value


def read_wav_once(path: Path, opened: list[dict[str, Any]]) -> tuple[bytes, bytes, dict[str, int]]:
    payload = path.read_bytes()
    opened.append({"path": path.relative_to(ROOT).as_posix(), "sha256": sha256(payload)})
    with wave.open(str(path), "rb") as reader:
        format_info = {
            "sample_rate_hz": reader.getframerate(),
            "channels": reader.getnchannels(),
            "sample_width_bits": reader.getsampwidth() * 8,
            "frame_count": reader.getnframes(),
        }
        require(reader.getcomptype() == "NONE", f"compressed WAV rejected: {path}")
        pcm = reader.readframes(reader.getnframes())
        require(reader.readframes(1) == b"", f"WAV frame count changed: {path}")
    expected_bytes = (
        format_info["frame_count"]
        * format_info["channels"]
        * (format_info["sample_width_bits"] // 8)
    )
    require(len(pcm) == expected_bytes, f"WAV PCM length mismatch: {path}")
    return payload, pcm, format_info


def write_pcm16(path: Path, sample_rate_hz: int, channels: int, pcm: bytes) -> bytes:
    with wave.open(str(path), "wb") as writer:
        writer.setnchannels(channels)
        writer.setsampwidth(2)
        writer.setframerate(sample_rate_hz)
        writer.writeframes(pcm)
    return path.read_bytes()


def metrics(pcm: bytes, channels: int) -> dict[str, Any]:
    samples = array("h")
    samples.frombytes(pcm)
    if sys.byteorder != "little":
        samples.byteswap()
    require(len(samples) % channels == 0, "PCM is not frame-aligned")
    normalized = [sample / 32768.0 for sample in samples]
    return {
        "active_sample_count": sum(sample != 0 for sample in samples),
        "rms": math.sqrt(sum(sample * sample for sample in normalized) / len(normalized)),
        "peak_abs": max((abs(sample) for sample in normalized), default=0.0),
        "clipped_sample_count": sum(sample in {-32768, 32767} for sample in samples),
        "samples": samples,
    }


def seam_metrics(
    samples: array[int],
    channels: int,
    internal_boundary_frame: int,
    window_frames: int,
) -> dict[str, Any]:
    frame_count = len(samples) // channels
    require(0 < internal_boundary_frame < frame_count, "internal boundary is outside loop")
    seam_jumps: list[float] = []
    reference_jumps: list[float] = []
    ratios: list[float] = []
    for channel in range(channels):
        seam = abs(samples[channel] - samples[(frame_count - 1) * channels + channel]) / 32768.0
        start = max(1, internal_boundary_frame - window_frames)
        end = min(frame_count, internal_boundary_frame + window_frames + 1)
        reference = max(
            abs(samples[frame * channels + channel] - samples[(frame - 1) * channels + channel])
            / 32768.0
            for frame in range(start, end)
        )
        require(reference > 0.0, "internal boundary reference has no adjacent-sample activity")
        seam_jumps.append(seam)
        reference_jumps.append(reference)
        ratios.append(seam / reference)
    return {
        "absolute_sample_jump_per_channel": seam_jumps,
        "internal_boundary_max_adjacent_jump_per_channel": reference_jumps,
        "jump_ratio_per_channel": ratios,
    }


def persist_json(path: Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    contract_payload, contract = read_json(CONTRACT_PATH)
    require(sha256(contract_payload) == CONTRACT_SHA256, "frozen contract SHA-256 mismatch")
    require(contract.get("status") == "frozen", "contract is not frozen")
    frozen = contract["frozen_input"]
    extraction = contract["loop_extraction"]
    gates = contract["technical_gates"]
    access = contract["development_access"]
    require(access["original_source_open"] is False, "original source access was enabled")
    require(access["holdout_audio_access"] is False, "Holdout access was enabled")
    require(access["commercial_reference_access"] is False, "commercial access was enabled")

    manifest_path = contained_regular_file(frozen["product_manifest_path"])
    manifest_payload, manifest = read_json(manifest_path)
    require(sha256(manifest_payload) == frozen["product_manifest_sha256"], "product manifest hash mismatch")
    require(manifest["product_path"] == frozen["product_path"], "product path mismatch")
    require(manifest["source"]["content_hash"] == f"sha256:{frozen['source_sha256']}", "source lineage mismatch")
    require(manifest["render"]["isolated_contributors"] == frozen["isolated_contributors"], "contributor mismatch")
    require(manifest["lane_roles"]["w30"] == "source_transform_foundation", "W-30 owner mismatch")
    for role in ("tr909", "mc202", "source_monitor"):
        require(manifest["lane_roles"][role] == "stay_out", f"{role} did not stay out")
    require(manifest["timing"]["product_bpm"] == frozen["product_bpm"], "product tempo mismatch")
    require(manifest["render"]["start_beat"] == frozen["start_beat"], "render start mismatch")

    require(not OUTPUT_DIR.exists(), f"fresh output directory required: {OUTPUT_DIR}")
    OUTPUT_DIR.mkdir(parents=True)
    access_fd = os.open(ACCESS_LOG_PATH, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    opened: list[dict[str, Any]] = []
    access_log: dict[str, Any] = {
        "schema": "riotbox.w30_semantic_hook_stem_access_log.v1",
        "ticket": "RIOTBOX-1482",
        "started_at_utc": utc_now(),
        "mode": "exact_prior_artifacts_only_no_source_directory_discovery",
        "allowed_audio_paths": [access["source_context_path"], frozen["audio_path"]],
        "opened_audio": opened,
        "original_source_opened": False,
        "holdout_audio_opened": False,
        "commercial_reference_opened": False,
        "status": "started",
    }
    with os.fdopen(access_fd, "w", encoding="utf-8") as access_file:
        access_file.write(json.dumps(access_log, indent=2, sort_keys=True) + "\n")
        access_file.flush()
        os.fsync(access_file.fileno())
        try:
            source_path = contained_regular_file(access["source_context_path"])
            input_path = contained_regular_file(frozen["audio_path"])
            source_payload, _, _ = read_wav_once(source_path, opened)
            require(sha256(source_payload) == access["source_context_sha256"], "source-context hash mismatch")
            input_payload, input_pcm, input_format = read_wav_once(input_path, opened)
            require(sha256(input_payload) == frozen["audio_sha256"], "frozen input hash mismatch")
            require(input_format == frozen["audio_format"], "frozen input format mismatch")

            frame_count = extraction["expected_frame_count"]
            block_align = input_format["channels"] * 2
            loop_pcm = input_pcm[: frame_count * block_align]
            require(len(loop_pcm) == frame_count * block_align, "input is shorter than frozen loop")
            signal = metrics(loop_pcm, input_format["channels"])
            seam = seam_metrics(
                signal.pop("samples"),
                input_format["channels"],
                gates["loop_seam"]["internal_bar_boundary_frame"],
                round(input_format["sample_rate_hz"] * 0.010),
            )
            passed = (
                signal["rms"] >= gates["minimum_output_rms"]
                and signal["peak_abs"] < gates["maximum_output_peak_abs"]
                and signal["clipped_sample_count"] <= gates["maximum_clipped_sample_count"]
                and all(
                    value <= gates["loop_seam"]["maximum_absolute_sample_jump_per_channel"]
                    for value in seam["absolute_sample_jump_per_channel"]
                )
                and all(
                    value <= gates["loop_seam"]["maximum_jump_ratio_to_loudest_adjacent_sample_delta_within_10ms_of_internal_bar_boundary"]
                    for value in seam["jump_ratio_per_channel"]
                )
            )
            require(passed, "frozen technical or loop-seam gate failed")

            first = write_pcm16(LOOP_PATH, input_format["sample_rate_hz"], input_format["channels"], loop_pcm)
            second = write_pcm16(RERENDER_PATH, input_format["sample_rate_hz"], input_format["channels"], loop_pcm)
            require(first == second, "independent loop writes are not byte-identical")
            review = write_pcm16(
                REVIEW_PATH,
                input_format["sample_rate_hz"],
                input_format["channels"],
                loop_pcm * 3,
            )
            report = {
                "schema": "riotbox.w30_semantic_hook_stem_technical_report.v1",
                "ticket": "RIOTBOX-1482",
                "result": "pass",
                "contract_sha256": CONTRACT_SHA256,
                "product_manifest_sha256": sha256(manifest_payload),
                "source_context_sha256": sha256(source_payload),
                "input_sha256": sha256(input_payload),
                "role": contract["semantic_role"],
                "output": {
                    "path": LOOP_PATH.relative_to(ROOT).as_posix(),
                    "sha256": sha256(first),
                    "sample_rate_hz": input_format["sample_rate_hz"],
                    "channels": input_format["channels"],
                    "sample_width_bits": 16,
                    "frame_count": frame_count,
                    "duration_seconds": frame_count / input_format["sample_rate_hz"],
                    **signal,
                    "seam": seam,
                    "pcm_exact_input_prefix": True,
                    "independent_write_sha256": sha256(second),
                },
                "review_artifact": {
                    "path": REVIEW_PATH.relative_to(ROOT).as_posix(),
                    "sha256": sha256(review),
                    "repeat_count": 3,
                    "duration_seconds": frame_count * 3 / input_format["sample_rate_hz"],
                },
                "isolated_contributors": frozen["isolated_contributors"],
                "silent_contributors": frozen["silent_contributors"],
                "source_or_product_rerendered": False,
                "human_playback_occurred": False,
            }
            persist_json(REPORT_PATH, report)
            access_log["status"] = "completed"
            access_log["completed_at_utc"] = utc_now()
        except Exception as error:
            access_log["status"] = "rejected"
            access_log["completed_at_utc"] = utc_now()
            access_log["rejection"] = str(error)
            raise
        finally:
            access_file.seek(0)
            access_file.write(json.dumps(access_log, indent=2, sort_keys=True) + "\n")
            access_file.truncate()
            access_file.flush()
            os.fsync(access_file.fileno())

    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
