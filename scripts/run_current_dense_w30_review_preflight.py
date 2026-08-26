#!/usr/bin/env python3
"""Prepare the one exact RIOTBOX-1471 source-first review without rerendering."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import shutil
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
)


CONTRACT = Path("docs/benchmarks/dense_w30_current_artifact_review_v2.json")
CONTRACT_SHA256 = "18f531c5fdbf899058d173c8cd839cdf8533775f4f51e1c7259f0933a2293e5a"
V1_CONTRACT = Path("docs/benchmarks/dense_w30_current_artifact_review_v1.json")
V1_CONTRACT_SHA256 = "e674807ba5c5fdb9572f50d82f2a1a8975bfc92960e655b19605d6780bbb86d3"
V1_FAILED_ACCESS_LOG = Path("artifacts/development/riotbox-1471/access-log-2026-08-26-a.json")
V1_FAILED_ACCESS_LOG_SHA256 = "5c990a6294f1eb4672252c822243f4051ff2c668b910ea520f6e037b7696f561"
SOURCE = Path("data/test_audio/examples/Beat03_130BPM(Full).wav")
SOURCE_SHA256 = "e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a"
SOURCE_FORMAT = {
    "sample_rate_hz": 44_100,
    "channels": 2,
    "sample_width_bits": 24,
    "compression_type": "NONE",
    "maximum_duration_seconds": 16,
}
CANDIDATE = Path(
    "artifacts/development/riotbox-1470/qualification-v1/05_w30_dense_foundation_control.wav"
)
CANDIDATE_SHA256 = "baccaa2dcff86e2965571ed3ba4dd4443904fa7c8f3e6b3bb9fd02725637627c"
PRIOR_FILES = {
    "access_log": (
        Path("artifacts/development/riotbox-1470/access-log-2026-08-26-a.json"),
        "238a19378ade5c1bf9c94e4450f10a3374cf9eedc9b515424fb7fe65cd750eac",
    ),
    "qualification_result": (
        Path("artifacts/development/riotbox-1470/qualification-v1/dense-w30-foundation-qualification.json"),
        "a4c3a6d25a900a26748906bebe26434a8820d6ae26817b12bdb74dd8f9a8fb40",
    ),
    "product_manifest": (
        Path("artifacts/development/riotbox-1470/qualification-v1/dense-w30-foundation-product-manifest.json"),
        "3906af09b1bce218baf973cdd2bd3e054eefc12ece7b637f7f95e364e64dca1f",
    ),
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def validate_source_blind(repo: Path) -> None:
    contract_path = repo / CONTRACT
    require(sha256_file(contract_path) == CONTRACT_SHA256, "frozen review contract hash changed")
    contract = read_json(contract_path)
    require(contract.get("schema") == "riotbox.dense_w30_current_artifact_review.v2", "contract schema changed")
    require(contract.get("status") == "frozen", "contract is not frozen")
    require(contract["timing_contract_correction"]["current_product_behavior_is_intentional"] is True, "timing correction changed")
    require(contract["product_path"]["product_code_change_allowed"] is False, "product-code boundary changed")
    require(contract["stopping_rule"]["maximum_registered_source_opens"] == 1, "source-open budget changed")
    require(contract["human_review"]["maximum_reviews"] == 1, "review budget changed")
    require(sha256_file(repo / V1_CONTRACT) == V1_CONTRACT_SHA256, "v1 contract changed")
    require(sha256_file(repo / V1_FAILED_ACCESS_LOG) == V1_FAILED_ACCESS_LOG_SHA256, "v1 failed access log changed")
    resolved = {}
    for label, (relative, expected) in PRIOR_FILES.items():
        path = repo / relative
        require(path.is_file(), f"missing RIOTBOX-1470 {label}")
        require(sha256_file(path) == expected, f"RIOTBOX-1470 {label} hash changed")
        resolved[label] = read_json(path)
    result = resolved["qualification_result"]
    gates = {gate["gate"]: gate["pass"] for gate in result["gates"]}
    required_passes = {
        "callback_partitions_sample_exact",
        "restart_sample_exact",
        "missing_source_silent",
        "no_pre_limiter_clips",
        "no_limiter_intervention",
        "no_post_limiter_clips",
        "non_silent",
    }
    require(all(gates.get(gate) is True for gate in required_passes), "technical gate changed")
    require(gates.get("audio_bit_identical_to_prior_reviewed_control") is False, "identity result changed")
    require(gates.get("product_manifest_bit_identical_to_prior_reconstruction") is False, "manifest identity result changed")
    require(result["artifacts"]["audio_sha256"] == CANDIDATE_SHA256, "candidate identity changed")
    manifest = resolved["product_manifest"]
    require(manifest["timing"]["product_bpm"] == 130.0, "current product BPM changed")
    require(manifest["lane_roles"] == {
        "mc202": "stay_out",
        "source_monitor": "stay_out",
        "tr909": "stay_out",
        "w30": "source_transform_foundation",
    }, "lane ownership changed")
    print("RIOTBOX-1471 source-blind artifact boundary: pass")


def pcm16_metrics(path: Path) -> dict[str, Any]:
    with wave.open(str(path), "rb") as wav:
        require(wav.getcomptype() == "NONE", "candidate WAV must be PCM")
        require(wav.getsampwidth() == 2, "candidate WAV must be PCM16")
        require(wav.getframerate() == 48_000, "candidate sample rate changed")
        require(wav.getnchannels() == 2, "candidate channel count changed")
        require(wav.getnframes() == 288_000, "candidate frame count changed")
        payload = wav.readframes(wav.getnframes())
    samples = array("h")
    samples.frombytes(payload)
    if sys.byteorder != "little":
        samples.byteswap()
    peak = max((abs(value) for value in samples), default=0)
    active = sum(value != 0 for value in samples)
    rms = math.sqrt(sum(value * value for value in samples) / max(1, len(samples))) / 32768.0
    require(active > 0, "candidate is silent")
    require(peak < 32767, "candidate contains clipped PCM16 samples")
    return {
        "sample_rate_hz": 48_000,
        "channels": 2,
        "sample_width_bits": 16,
        "frame_count": 288_000,
        "duration_seconds": 6.0,
        "active_samples": active,
        "peak_abs": peak / 32768.0,
        "rms": rms,
        "clipped_samples": 0,
    }


def run(repo: Path, output: Path, access_log_path: Path) -> None:
    validate_source_blind(repo)
    require(sha256_file(repo / CONTRACT) == CONTRACT_SHA256, "frozen contract hash changed")
    require(not output.exists(), f"review output already exists: {output}")
    output.mkdir(parents=True)
    log: dict[str, Any] = {
        "schema": "riotbox.dense_w30_current_artifact_review_access.v2",
        "ticket": "RIOTBOX-1471",
        "started_at_utc": utc_now(),
        "contract": {"path": CONTRACT.as_posix(), "sha256": CONTRACT_SHA256},
        "mode": "one_exact_development_source_no_directory_discovery_no_rerender",
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_opened": False,
        "candidate_rerendered": False,
        "human_playback_occurred": False,
        "status": "preflight_pending",
    }
    with create_exclusive_access_log(access_log_path) as handle:
        persist_access_log(handle, log)
        try:
            candidate = repo / CANDIDATE
            require(candidate.is_file(), "exact current candidate is missing")
            require(sha256_file(candidate) == CANDIDATE_SHA256, "candidate hash changed")
            candidate_metrics = pcm16_metrics(candidate)
            log["candidate"] = {
                "path": CANDIDATE.as_posix(),
                "sha256": CANDIDATE_SHA256,
                "metrics": candidate_metrics,
            }
            log["status"] = "candidate_verified"
            persist_access_log(handle, log)

            opened = {"path": SOURCE.as_posix(), "expected_sha256": SOURCE_SHA256, "status": "opening"}

            def record_open(_: Path) -> None:
                log["source"] = opened
                log["status"] = "source_opened"
                persist_access_log(handle, log)

            source_payload, source_result = validate_contained_source_file(
                repo,
                SOURCE,
                SOURCE_SHA256,
                SOURCE_FORMAT,
                f"{CONTRACT}: dense_beat03_130",
                on_open=record_open,
                return_payload=True,
            )
            opened.update(source_result)
            opened.update({"status": "verified"})

            source_copy = output / "00_verified_source.wav"
            candidate_copy = output / "01_current_w30_foundation.wav"
            source_copy.write_bytes(source_payload)
            shutil.copyfile(candidate, candidate_copy)
            require(sha256_file(source_copy) == SOURCE_SHA256, "source presentation changed bytes")
            require(sha256_file(candidate_copy) == CANDIDATE_SHA256, "candidate presentation changed bytes")
            preflight = {
                "schema": "riotbox.dense_w30_current_artifact_review_preflight.v2",
                "ticket": "RIOTBOX-1471",
                "result": "technically_eligible_for_one_source_first_human_review",
                "playback_order": [source_copy.as_posix(), candidate_copy.as_posix()],
                "source": opened,
                "candidate": {"sha256": CANDIDATE_SHA256, **candidate_metrics},
                "presentation_byte_identical": True,
                "candidate_rerendered": False,
                "human_playback_occurred": False,
                "review_question": "is the W-30 transformation musically useful as the Dense foundation before support lanes?",
            }
            preflight_path = output / "preflight.json"
            write_json(preflight_path, preflight)
            log["preflight"] = {"path": preflight_path.as_posix(), "sha256": sha256_file(preflight_path), "result": preflight["result"]}
            log["status"] = "completed"
            log["completed_at_utc"] = utc_now()
            persist_access_log(handle, log)
            print(json.dumps(preflight, indent=2, sort_keys=True))
        except Exception as error:
            log["status"] = "failed_closed"
            log["failure_type"] = type(error).__name__
            log["failure"] = str(error)
            log["completed_at_utc"] = utc_now()
            persist_access_log(handle, log)
            raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--validate-source-blind", action="store_true")
    parser.add_argument("--run", action="store_true")
    args = parser.parse_args()
    require(args.validate_source_blind != args.run, "select exactly one mode")
    repo = Path(__file__).resolve().parent.parent
    if args.validate_source_blind:
        validate_source_blind(repo)
    else:
        run(
            repo,
            repo / "artifacts/development/riotbox-1471/review-v2",
            repo / "artifacts/development/riotbox-1471/access-log-2026-08-26-b.json",
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
