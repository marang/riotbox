#!/usr/bin/env python3
"""Run the active one-source RIOTBOX-1468 Development exploration fail-closed."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import subprocess
import tempfile
import wave
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import numpy as np

import generate_dense_break_performance_pack as dense
from source_holdout_development_access import validate_contained_source_file


CONTRACT = Path("docs/benchmarks/dense_break_source_native_bar_exploration_v2.json")
CONTRACT_SHA256 = "ac71e8daa9f862a8341910d63e0457cd657e6506808eda4032d132b4fb443517"
RECOVERY = Path("docs/benchmarks/dense_break_source_native_bar_preflight_recovery_v1.json")
RECOVERY_SHA256 = "1683e1aa824dba52c6c0c55d977107cfc535fa0a5fae0da9db0a3c8806ef7278"
RUNTIME_REPORT = "source-native-full-bar-runtime-report.json"
SOURCE_FILE = "00_source_full_bar_loop.wav"
CONTROL_FILE = "01_w30_half_beat_chop_control.wav"
CANDIDATE_FILE = "02_w30_source_native_full_bar_candidate_v1.wav"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--session", required=True)
    parser.add_argument("--recover-preflight", action="store_true")
    args = parser.parse_args()
    require(args.session and all(char.isalnum() or char in "-_" for char in args.session), "unsafe session")

    repo = Path(__file__).resolve().parent.parent
    contract_bytes = (repo / CONTRACT).read_bytes()
    require(hashlib.sha256(contract_bytes).hexdigest() == CONTRACT_SHA256, "frozen contract changed")
    contract = json.loads(contract_bytes)
    access = contract["development_access"]
    registry_path = repo / access["registry_path"]
    require(file_sha256(registry_path) == access["registry_sha256"], "source registry changed")
    registry = json.loads(registry_path.read_text(encoding="utf-8"))
    matching = [entry for entry in registry["entries"] if entry.get("case_id") == access["case_id"]]
    require(len(matching) == 1, "registered Development identity is not unique")
    require(matching[0]["source_path"] == access["source_path"], "registered path changed")

    if args.recover_preflight:
        return recover_preflight(repo, contract, args.session)

    root = repo / "artifacts" / "development" / "riotbox-1468"
    output = root / f"source-native-bar-{args.session}"
    access_log_path = root / f"access-log-{args.session}.json"
    root.mkdir(parents=True, exist_ok=True)
    require(not output.exists(), f"output already exists: {output}")
    require(not access_log_path.exists(), f"access log already exists: {access_log_path}")
    output.mkdir()

    log: dict[str, Any] = {
        "schema": "riotbox.dense_break_source_native_bar_access_log.v2",
        "ticket": "RIOTBOX-1468",
        "session": args.session,
        "started_at_utc": utc_now(),
        "contract_path": CONTRACT.as_posix(),
        "contract_sha256": CONTRACT_SHA256,
        "registry_path": access["registry_path"],
        "registry_sha256": access["registry_sha256"],
        "mode": "one_exact_development_file_no_glob_or_directory_discovery",
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "maximum_unique_development_files": 1,
        "requested_case_ids": [access["case_id"]],
        "records": [],
        "status": "created_before_first_source_open",
    }
    access_log_path.touch(mode=0o600, exist_ok=False)
    write_json(access_log_path, log)

    record: dict[str, Any] = {
        "case_id": access["case_id"],
        "partition": "development",
        "source_path": access["source_path"],
        "expected_sha256": access["source_sha256"],
        "status": "preflight_pending",
    }
    log["records"].append(record)
    write_json(access_log_path, log)

    try:
        def record_open(_: Path) -> None:
            record["opened_at_utc"] = utc_now()
            record["status"] = "opened_for_bounded_verified_read"
            log["status"] = "reading_one_development_source"
            write_json(access_log_path, log)

        payload, source_verification = validate_contained_source_file(
            repo,
            Path(access["source_path"]),
            access["source_sha256"],
            access["source_format"],
            f"RIOTBOX-1468:{access['case_id']}",
            on_open=record_open,
            return_payload=True,
        )
        record.update(source_verification)
        record["status"] = "verified_once_and_delivered_in_process"
        write_json(access_log_path, log)

        with tempfile.TemporaryDirectory(prefix="riotbox-1468-") as temporary:
            temporary_source = Path(temporary) / "registered-development-input.wav"
            temporary_source.write_bytes(payload)
            render_output = output / "render"
            command = [
                "cargo",
                "run",
                "--quiet",
                "-p",
                "riotbox-app",
                "--bin",
                "w30_live_path_render",
                "--",
                "--source",
                str(temporary_source),
                "--output",
                str(render_output),
                "--bpm",
                str(access["declared_bpm_metadata_only"]),
                "--explore-source-native-full-bar-v1",
            ]
            completed = subprocess.run(
                command,
                cwd=repo,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                check=False,
            )
        record["renderer_exit_code"] = completed.returncode
        record["renderer_output_tail"] = completed.stdout.splitlines()[-12:]
        require(completed.returncode == 0, "exact W-30 exploration renderer failed")

        runtime_report_path = render_output / RUNTIME_REPORT
        runtime_report = json.loads(runtime_report_path.read_text(encoding="utf-8"))
        require(runtime_report.get("contract_sha256") == CONTRACT_SHA256, "runtime contract pin changed")
        require(runtime_report.get("callback_outputs_sample_exact") is True, "callback proof failed")
        require(runtime_report.get("lane") == "w30", "unexpected audible lane")
        require(runtime_report.get("additional_effect_count") == 0, "unexpected added effect")

        capture_window = last_capture_window(render_output / "session.json")
        write_source_bar_loop(payload, capture_window, render_output / SOURCE_FILE, 4)
        preflight = preflight_artifacts(render_output, runtime_report, contract)
        record["technical_preflight"] = preflight
        record["status"] = "technical_preflight_evaluated"
        write_json(access_log_path, log)
        require(preflight["result"] == "pass", "exact artifact technical preflight failed")

        record.update(
            {
                "runtime_report_sha256": file_sha256(runtime_report_path),
                "source_sha256": file_sha256(render_output / SOURCE_FILE),
                "control_sha256": file_sha256(render_output / CONTROL_FILE),
                "candidate_sha256": file_sha256(render_output / CANDIDATE_FILE),
                "status": "completed",
                "completed_at_utc": utc_now(),
            }
        )
        log["status"] = "passed_one_source_preflight"
        log["completed_at_utc"] = utc_now()
        write_json(access_log_path, log)

        report = {
            "schema": "riotbox.dense_break_source_native_bar_exploration_result.v2",
            "ticket": "RIOTBOX-1468",
            "stage": "development_exploration",
            "result": "technical_pass_human_unverified",
            "contract_path": CONTRACT.as_posix(),
            "contract_sha256": CONTRACT_SHA256,
            "decision": "RBX-334",
            "access_log_path": access_log_path.relative_to(repo).as_posix(),
            "access_log_sha256": file_sha256(access_log_path),
            "source_case_id": access["case_id"],
            "source_identity_sha256": access["source_sha256"],
            "runtime_report_sha256": file_sha256(runtime_report_path),
            "capture_window": capture_window,
            "technical_preflight": preflight,
            "files": {
                name: {"sha256": file_sha256(render_output / name)}
                for name in (SOURCE_FILE, CONTROL_FILE, CANDIDATE_FILE)
            },
            "presentation": {
                "source_context_bars": 4,
                "control_bars": 4,
                "candidate_bars": 4,
                "order": ["source", "control", "candidate"],
                "pause_seconds": contract["unchanged_v1_contract"]["pause_seconds"],
                "uniform_safety_gain_only": True,
            },
            "human_review": {
                "status": "unverified",
                "question": "Does the continuous full-bar candidate preserve the break's groove, clarity, weight, and identity while remaining a useful W-30 transformation?",
            },
            "claim_boundary": contract["claim_boundary"],
        }
        write_json(output / "exploration-result.json", report)
        print((output / "exploration-result.json").relative_to(repo))
        return 0
    except Exception as error:
        log["status"] = "failed_closed"
        log["failure"] = str(error)
        log["completed_at_utc"] = utc_now()
        write_json(access_log_path, log)
        raise


def last_capture_window(session_path: Path) -> dict[str, Any]:
    session = json.loads(session_path.read_text(encoding="utf-8"))
    captures = session.get("captures")
    require(isinstance(captures, list) and captures, "renderer produced no capture")
    source_window = captures[-1].get("source_window")
    require(isinstance(source_window, dict), "capture has no source window")
    start = source_window.get("start_frame")
    end = source_window.get("end_frame")
    require(isinstance(start, int) and isinstance(end, int) and end > start, "invalid capture window")
    return {
        "source_id": source_window.get("source_id"),
        "start_frame": start,
        "end_frame": end,
        "frame_count": end - start,
        "start_seconds": source_window.get("start_seconds"),
        "end_seconds": source_window.get("end_seconds"),
    }


def write_source_bar_loop(
    payload: bytes, capture_window: dict[str, Any], output: Path, loop_count: int
) -> None:
    with wave.open(io.BytesIO(payload), "rb") as source:
        channels = source.getnchannels()
        sample_width = source.getsampwidth()
        sample_rate = source.getframerate()
        start = int(capture_window["start_frame"])
        frame_count = int(capture_window["frame_count"])
        require(start + frame_count <= source.getnframes(), "capture exceeds source payload")
        source.setpos(start)
        frames = source.readframes(frame_count)
    require(len(frames) == frame_count * channels * sample_width, "short source-window read")
    with wave.open(str(output), "wb") as target:
        target.setnchannels(channels)
        target.setsampwidth(sample_width)
        target.setframerate(sample_rate)
        target.writeframes(frames * loop_count)


def preflight_artifacts(
    render_output: Path, runtime_report: dict[str, Any], contract: dict[str, Any]
) -> dict[str, Any]:
    dense.require_numpy()
    decoded = {
        name: read_pcm_wav(render_output / name)
        for name in (SOURCE_FILE, CONTROL_FILE, CANDIDATE_FILE)
    }
    gate = contract["unchanged_v1_contract"]
    metrics: dict[str, Any] = {}
    result = "pass"
    for name, item in decoded.items():
        audio = item["samples"]
        true_peak = dense.conservative_true_peak_amplitude(audio)
        true_peak_dbtp = dense.amplitude_to_db(true_peak)
        finite = bool(np.all(np.isfinite(audio)))
        metrics[name] = {
            "sample_rate_hz": item["sample_rate_hz"],
            "channels": item["channels"],
            "sample_width_bits": item["sample_width_bits"],
            "frame_count": int(audio.shape[0]),
            "duration_seconds": float(audio.shape[0] / item["sample_rate_hz"]),
            "rms": float(np.sqrt(np.mean(audio.astype(np.float64) ** 2))),
            "true_peak_dbtp": true_peak_dbtp,
            "clipped_integer_samples": item["clipped_integer_samples"],
            "finite": finite,
        }
        if (
            not finite
            or true_peak_dbtp > float(gate["maximum_true_peak_dbtp"])
            or item["clipped_integer_samples"] > int(gate["maximum_clipped_integer_samples"])
        ):
            result = "fail"

    source_frames = int(decoded[SOURCE_FILE]["samples"].shape[0] / 4)
    candidate_frames = round(
        decoded[CANDIDATE_FILE]["sample_rate_hz"] * 60.0 / runtime_report["bpm"] * 4.0
    )
    source_jump = maximum_boundary_jump(decoded[SOURCE_FILE]["samples"], source_frames)
    candidate_jump = maximum_boundary_jump(
        decoded[CANDIDATE_FILE]["samples"], candidate_frames
    )
    boundary_limit = max(0.12, source_jump * 1.5)
    if candidate_jump > boundary_limit:
        result = "fail"
    delta_rms = float(runtime_report["control_candidate_delta"]["rms"])
    if delta_rms < float(gate["control_candidate_delta_rms_min"]):
        result = "fail"
    return {
        "result": result,
        "uniform_presentation_gain": 1.0,
        "metrics": metrics,
        "source_boundary_jump_abs": source_jump,
        "candidate_boundary_jump_abs": candidate_jump,
        "candidate_boundary_jump_limit": boundary_limit,
        "control_candidate_delta_rms": delta_rms,
        "callback_outputs_sample_exact": runtime_report["callback_outputs_sample_exact"],
    }


def recover_preflight(repo: Path, contract: dict[str, Any], session: str) -> int:
    recovery_bytes = (repo / RECOVERY).read_bytes()
    require(hashlib.sha256(recovery_bytes).hexdigest() == RECOVERY_SHA256, "recovery contract changed")
    recovery = json.loads(recovery_bytes)
    require(session == "2026-08-25-b", "recovery is bound to the exact failed v2 session")
    incident = recovery["incident"]
    access_log_path = repo / incident["access_log_path"]
    require(file_sha256(access_log_path) == incident["access_log_sha256"], "failed access log changed")
    artifact_contract = recovery["exact_artifacts"]
    render_output = repo / artifact_contract["root"]
    root = render_output.parent.parent
    recovery_log_path = root / f"preflight-recovery-{session}.json"
    result_path = render_output.parent / "exploration-result.json"
    require(not recovery_log_path.exists(), f"recovery log already exists: {recovery_log_path}")
    require(not result_path.exists(), f"exploration result already exists: {result_path}")
    recovery_log: dict[str, Any] = {
        "schema": "riotbox.dense_break_source_native_bar_preflight_recovery_log.v1",
        "ticket": "RIOTBOX-1468",
        "session": session,
        "started_at_utc": utc_now(),
        "recovery_contract_path": RECOVERY.as_posix(),
        "recovery_contract_sha256": RECOVERY_SHA256,
        "source_audio_reopened": False,
        "source_directory_discovery_performed": False,
        "rerender_performed": False,
        "artifact_replacement_performed": False,
        "status": "created_before_generated_artifact_analysis",
    }
    recovery_log_path.touch(mode=0o600, exist_ok=False)
    write_json(recovery_log_path, recovery_log)
    try:
        for field in ("source", "control", "candidate", "runtime_report", "session"):
            entry = artifact_contract[field]
            require(file_sha256(render_output / entry["path"]) == entry["sha256"], f"{field} identity changed")
        runtime_report_path = render_output / artifact_contract["runtime_report"]["path"]
        runtime_report = json.loads(runtime_report_path.read_text(encoding="utf-8"))
        require(runtime_report.get("contract_sha256") == CONTRACT_SHA256, "runtime contract pin changed")
        capture_window = last_capture_window(render_output / artifact_contract["session"]["path"])
        preflight = preflight_artifacts(render_output, runtime_report, contract)
        recovery_log["technical_preflight"] = preflight
        recovery_log["status"] = "technical_preflight_evaluated"
        write_json(recovery_log_path, recovery_log)
        require(preflight["result"] == "pass", "recovered artifact technical preflight failed")
        recovery_log.update(
            {
                "status": "passed_exact_existing_artifacts",
                "technical_preflight": preflight,
                "completed_at_utc": utc_now(),
            }
        )
        write_json(recovery_log_path, recovery_log)
        result = {
            "schema": "riotbox.dense_break_source_native_bar_exploration_result.v2",
            "ticket": "RIOTBOX-1468",
            "stage": "development_exploration",
            "result": "technical_pass_human_unverified",
            "contract_path": CONTRACT.as_posix(),
            "contract_sha256": CONTRACT_SHA256,
            "decision": "RBX-335",
            "access_log_path": incident["access_log_path"],
            "access_log_sha256": incident["access_log_sha256"],
            "preflight_recovery_contract_path": RECOVERY.as_posix(),
            "preflight_recovery_contract_sha256": RECOVERY_SHA256,
            "preflight_recovery_log_path": recovery_log_path.relative_to(repo).as_posix(),
            "preflight_recovery_log_sha256": file_sha256(recovery_log_path),
            "source_case_id": contract["development_access"]["case_id"],
            "source_identity_sha256": contract["development_access"]["source_sha256"],
            "runtime_report_sha256": file_sha256(runtime_report_path),
            "capture_window": capture_window,
            "technical_preflight": preflight,
            "files": {
                entry["path"]: {"sha256": entry["sha256"]}
                for entry in (
                    artifact_contract["source"],
                    artifact_contract["control"],
                    artifact_contract["candidate"],
                )
            },
            "presentation": {
                "source_context_bars": 4,
                "control_bars": 4,
                "candidate_bars": 4,
                "order": ["source", "control", "candidate"],
                "pause_seconds": contract["unchanged_v1_contract"]["pause_seconds"],
                "uniform_safety_gain_only": True,
            },
            "human_review": {
                "status": "unverified",
                "question": "Does the continuous full-bar candidate preserve the break's groove, clarity, weight, and identity while remaining a useful W-30 transformation?",
            },
            "claim_boundary": contract["claim_boundary"],
        }
        write_json(result_path, result)
        print(result_path.relative_to(repo))
        return 0
    except Exception as error:
        recovery_log["status"] = "failed_closed"
        recovery_log["failure"] = str(error)
        recovery_log["completed_at_utc"] = utc_now()
        write_json(recovery_log_path, recovery_log)
        raise


def read_pcm_wav(path: Path) -> dict[str, Any]:
    with wave.open(str(path), "rb") as handle:
        channels = handle.getnchannels()
        sample_width = handle.getsampwidth()
        sample_rate = handle.getframerate()
        frame_count = handle.getnframes()
        payload = handle.readframes(frame_count)
    require(sample_width in (2, 3), f"unsupported PCM width: {sample_width * 8}")
    if sample_width == 2:
        integers = np.frombuffer(payload, dtype="<i2").astype(np.int32)
        limit = 32768.0
        clipped = int(np.count_nonzero(np.abs(integers) >= 32767))
    else:
        triplets = np.frombuffer(payload, dtype=np.uint8).reshape(-1, 3)
        integers = (
            triplets[:, 0].astype(np.int32)
            | (triplets[:, 1].astype(np.int32) << 8)
            | (triplets[:, 2].astype(np.int32) << 16)
        )
        integers = np.where(integers & 0x800000, integers - 0x1000000, integers)
        limit = 8388608.0
        clipped = int(np.count_nonzero(np.abs(integers) >= 8388607))
    samples = (integers.astype(np.float32) / limit).reshape(-1, channels)
    return {
        "sample_rate_hz": sample_rate,
        "channels": channels,
        "sample_width_bits": sample_width * 8,
        "clipped_integer_samples": clipped,
        "samples": samples,
    }


def maximum_boundary_jump(samples: np.ndarray, bar_frames: int) -> float:
    require(bar_frames > 0 and samples.shape[0] >= bar_frames * 2, "invalid bar boundary")
    jumps = []
    for boundary in range(bar_frames, samples.shape[0], bar_frames):
        if boundary < samples.shape[0]:
            jumps.append(float(np.max(np.abs(samples[boundary] - samples[boundary - 1]))))
    require(jumps, "no bar boundary available")
    return max(jumps)


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_json(path: Path, value: dict[str, Any]) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temporary, path)


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    raise SystemExit(main())
