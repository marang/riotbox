#!/usr/bin/env python3
"""Run the one-source RIOTBOX-1469 stereo-pad Development exploration."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import subprocess
import tempfile
import wave
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import generate_dense_break_performance_pack as dense
from source_holdout_development_access import (
    create_exclusive_access_log,
    persist_access_log,
    validate_contained_source_file,
)


CONTRACT = Path("docs/benchmarks/w30_stereo_pad_playback_development_v1.json")
RUNTIME_REPORT = "stereo-pad-runtime-report.json"
SOURCE_FILE = "00_source_capture_loop.wav"
CONTROL_FILE = "01_w30_mono_control.wav"
CANDIDATE_FILE = "02_w30_stereo_candidate_v1.wav"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--session", required=True)
    args = parser.parse_args()
    require(
        args.session and all(char.isalnum() or char in "-_" for char in args.session),
        "unsafe session token",
    )

    repo = Path(__file__).resolve().parent.parent
    contract_payload = (repo / CONTRACT).read_bytes()
    contract_sha256 = hashlib.sha256(contract_payload).hexdigest()
    contract = json.loads(contract_payload)
    validate_contract(contract, repo)
    access = contract["development_access"]

    root = repo / "artifacts" / "development" / "riotbox-1469"
    output = root / f"stereo-pad-{args.session}"
    render_output = output / "render"
    access_log_path = root / f"access-log-{args.session}.json"
    require(not output.exists(), f"output already exists: {output}")
    require(not access_log_path.exists(), f"access log already exists: {access_log_path}")
    output.mkdir(parents=True)

    access_log: dict[str, Any] = {
        "schema": "riotbox.w30_stereo_pad_development_access_log.v1",
        "ticket": "RIOTBOX-1469",
        "session": args.session,
        "started_at_utc": utc_now(),
        "contract_path": CONTRACT.as_posix(),
        "contract_sha256": contract_sha256,
        "registry_path": access["registry_path"],
        "registry_sha256": access["registry_sha256"],
        "mode": "one_exact_development_file_no_glob_or_directory_discovery",
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "requested_case_ids": [access["case_id"]],
        "opened_development_files": [],
        "status": "created_before_first_source_open",
    }

    with create_exclusive_access_log(access_log_path) as access_log_file:
        persist_access_log(access_log_file, access_log)
        opened_record: dict[str, Any] = {
            "case_id": access["case_id"],
            "partition": "development",
            "source_path": access["source_path"],
            "expected_sha256": access["source_sha256"],
            "status": "preflight_pending",
        }
        access_log["opened_development_files"].append(opened_record)
        persist_access_log(access_log_file, access_log)
        try:
            def record_open(_: Path) -> None:
                opened_record["status"] = "opened_for_bounded_verified_read"
                opened_record["opened_at_utc"] = utc_now()
                access_log["status"] = "reading_one_development_source"
                persist_access_log(access_log_file, access_log)

            payload, verification = validate_contained_source_file(
                repo,
                Path(access["source_path"]),
                access["source_sha256"],
                access["source_format"],
                f"RIOTBOX-1469:{access['case_id']}",
                on_open=record_open,
                return_payload=True,
            )
            opened_record.update(verification)
            opened_record["status"] = "verified_once_and_delivered_in_process"
            persist_access_log(access_log_file, access_log)

            with tempfile.TemporaryDirectory(prefix="riotbox-1469-") as temporary:
                temporary_source = Path(temporary) / "registered-development-input.wav"
                temporary_source.write_bytes(payload)
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
                    "--explore-stereo-pad-v1",
                ]
                completed = subprocess.run(
                    command,
                    cwd=repo,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    check=False,
                )
            opened_record["renderer_exit_code"] = completed.returncode
            opened_record["renderer_output_tail"] = completed.stdout.splitlines()[-12:]
            persist_access_log(access_log_file, access_log)
            require(completed.returncode == 0, "exact W-30 stereo renderer failed")

            runtime_report = json.loads(
                (render_output / RUNTIME_REPORT).read_text(encoding="utf-8")
            )
            require(
                runtime_report.get("callback_outputs_sample_exact") is True,
                "callback partition proof failed",
            )
            require(
                runtime_report.get("restart_outputs_sample_exact") is True,
                "restart proof failed",
            )
            require(
                runtime_report.get("missing_source_silence") is True,
                "missing-source silence proof failed",
            )
            require(runtime_report.get("audible_lanes") == ["w30"], "lane isolation changed")
            require(runtime_report.get("additional_effect_count") == 0, "effect count changed")

            capture_window = last_capture_window(render_output / "session.json")
            write_source_capture_loop(payload, capture_window, render_output / SOURCE_FILE, 4)
            preflight = preflight_artifacts(render_output, runtime_report, contract)
            opened_record["technical_preflight"] = preflight
            persist_access_log(access_log_file, access_log)
            require(preflight["result"] == "pass", "technical preflight rejected candidate")

            files = {
                name: {"sha256": file_sha256(render_output / name)}
                for name in (SOURCE_FILE, CONTROL_FILE, CANDIDATE_FILE, RUNTIME_REPORT)
            }
            result = {
                "schema": "riotbox.w30_stereo_pad_development_result.v1",
                "ticket": "RIOTBOX-1469",
                "stage": "development_exploration",
                "result": "technical_pass_human_unverified",
                "contract_path": CONTRACT.as_posix(),
                "contract_sha256": contract_sha256,
                "access_log_path": access_log_path.relative_to(repo).as_posix(),
                "source_case_id": access["case_id"],
                "source_identity_sha256": access["source_sha256"],
                "capture_window": capture_window,
                "technical_preflight": preflight,
                "files": files,
                "presentation": {
                    "order": ["source", "mono_control", "stereo_candidate"],
                    "bars_each": contract["technical_preflight"]["presentation_bars_each"],
                    "pause_seconds": contract["technical_preflight"]["pause_seconds"],
                    "uniform_safety_gain": preflight["uniform_presentation_gain"],
                },
                "human_review": {
                    "status": "unverified",
                    "questions": contract["listening"]["questions"],
                },
                "claim_boundary": contract["claim_boundary"],
            }
            write_json(output / "exploration-result.json", result)
            opened_record["status"] = "technical_pass_human_unverified"
            opened_record["completed_at_utc"] = utc_now()
            access_log["status"] = "completed"
            access_log["completed_at_utc"] = utc_now()
            persist_access_log(access_log_file, access_log)
            print((output / "exploration-result.json").relative_to(repo))
            return 0
        except Exception as error:
            access_log["status"] = "failed_closed"
            access_log["failure"] = str(error)
            access_log["completed_at_utc"] = utc_now()
            persist_access_log(access_log_file, access_log)
            raise


def validate_contract(contract: dict[str, Any], repo: Path) -> None:
    require(
        contract.get("schema") == "riotbox.w30_stereo_pad_playback_development.v1",
        "contract schema changed",
    )
    require(
        contract.get("status") == "frozen_source_blind_before_first_development_access",
        "contract is not source-blind frozen",
    )
    access = contract["development_access"]
    registry_path = repo / access["registry_path"]
    require(file_sha256(registry_path) == access["registry_sha256"], "registry changed")
    registry = json.loads(registry_path.read_text(encoding="utf-8"))
    matching = [
        entry for entry in registry["entries"] if entry.get("case_id") == access["case_id"]
    ]
    require(len(matching) == 1, "Development identity is not unique")
    require(matching[0].get("source_path") == access["source_path"], "source path changed")
    require(access["partition"] == "development", "source partition changed")
    require(access["maximum_unique_source_files"] == 1, "source bound changed")
    require(access["holdout_audio_access"] is False, "Holdout access enabled")
    require(access["commercial_reference_access"] is False, "reference access enabled")
    require(contract["stopping_rule"]["maximum_candidate_variants"] == 1, "variant limit changed")


def preflight_artifacts(
    render_output: Path, runtime_report: dict[str, Any], contract: dict[str, Any]
) -> dict[str, Any]:
    dense.require_numpy()
    np = dense.np
    paths = {
        "source": render_output / SOURCE_FILE,
        "control": render_output / CONTROL_FILE,
        "candidate": render_output / CANDIDATE_FILE,
    }
    decoded = {name: read_pcm_wav(path) for name, path in paths.items()}
    gate = contract["technical_preflight"]
    target_amplitude = 10.0 ** (float(gate["presentation_target_true_peak_dbtp"]) / 20.0)
    raw_true_peaks = {
        name: dense.conservative_true_peak_amplitude(item["samples"])
        for name, item in decoded.items()
    }
    maximum_peak = max(raw_true_peaks.values())
    uniform_gain = min(1.0, target_amplitude / maximum_peak) if maximum_peak > 0.0 else 1.0
    for name, item in decoded.items():
        write_pcm16(paths[name], item["sample_rate_hz"], item["samples"] * uniform_gain)
    decoded = {name: read_pcm_wav(path) for name, path in paths.items()}

    artifact_metrics: dict[str, Any] = {}
    for name, item in decoded.items():
        audio = item["samples"]
        true_peak = dense.conservative_true_peak_amplitude(audio)
        artifact_metrics[name] = {
            "sample_rate_hz": item["sample_rate_hz"],
            "channels": item["channels"],
            "frame_count": int(audio.shape[0]),
            "duration_seconds": float(audio.shape[0] / item["sample_rate_hz"]),
            "finite": bool(np.all(np.isfinite(audio))),
            "rms": rms(audio),
            "true_peak_dbtp": dense.amplitude_to_db(true_peak),
            "clipped_integer_samples": item["clipped_integer_samples"],
        }

    control = decoded["control"]["samples"]
    candidate = decoded["candidate"]["samples"]
    source = decoded["source"]["samples"]
    require(control.shape == candidate.shape and control.shape[1] == 2, "render shape changed")
    control_mid, _ = mid_side(control)
    candidate_mid, candidate_side = mid_side(candidate)
    source_mid, source_side = mid_side(source)
    full_mid_delta_db = ratio_db(rms(candidate_mid), rms(control_mid))
    correlation = pearson(control_mid, candidate_mid)
    source_side_to_mid_db = ratio_db(rms(source_side), rms(source_mid))
    candidate_side_to_mid_db = ratio_db(rms(candidate_side), rms(candidate_mid))
    side_ratio_delta_db = candidate_side_to_mid_db - source_side_to_mid_db
    stereo_delta_rms = rms(candidate - control)

    attack_window_frames = max(
        1,
        round(
            decoded["control"]["sample_rate_hz"]
            * float(gate["center_attack_window_ms_after_each_retrigger"])
            / 1000.0
        ),
    )
    step_frames = decoded["control"]["sample_rate_hz"] * 60.0 / runtime_report["bpm"] / 2.0
    attack_deltas: list[dict[str, Any]] = []
    step = 0
    while True:
        start = round(step * step_frames)
        end = min(control_mid.shape[0], start + attack_window_frames)
        if start >= control_mid.shape[0] or end <= start:
            break
        control_rms = rms(control_mid[start:end])
        candidate_rms = rms(candidate_mid[start:end])
        attack_deltas.append(
            {
                "step": step,
                "start_frame": start,
                "end_frame": end,
                "control_rms": control_rms,
                "candidate_rms": candidate_rms,
                "candidate_to_control_delta_db": ratio_db(candidate_rms, control_rms),
            }
        )
        step += 1

    computed = {
        "artifact_metrics": artifact_metrics,
        "callback_outputs_sample_exact": runtime_report["callback_outputs_sample_exact"],
        "restart_outputs_sample_exact": runtime_report["restart_outputs_sample_exact"],
        "missing_source_silence": runtime_report["missing_source_silence"],
        "center_attack_windows": attack_deltas,
        "center_attack_delta_db_min": min(
            item["candidate_to_control_delta_db"] for item in attack_deltas
        ),
        "center_attack_delta_db_max": max(
            item["candidate_to_control_delta_db"] for item in attack_deltas
        ),
        "candidate_to_control_full_mid_rms_delta_db": full_mid_delta_db,
        "candidate_mid_control_waveform_correlation": correlation,
        "source_side_to_mid_rms_db": source_side_to_mid_db,
        "candidate_side_to_mid_rms_db": candidate_side_to_mid_db,
        "candidate_minus_source_side_to_mid_rms_db": side_ratio_delta_db,
        "stereo_candidate_control_delta_rms": stereo_delta_rms,
        "uniform_presentation_gain": uniform_gain,
    }
    preflight_path = render_output / "stereo-pad-technical-preflight.json"
    write_json(
        preflight_path,
        {
            "schema": "riotbox.w30_stereo_pad_technical_preflight.v1",
            "ticket": "RIOTBOX-1469",
            "aggregate_result": "not_yet_evaluated",
            "computed_metrics": computed,
        },
    )

    attack_range = gate["candidate_to_control_center_attack_rms_delta_db_range"]
    mid_range = gate["candidate_to_control_full_mid_rms_delta_db_range"]
    side_delta_range = gate["candidate_minus_source_side_to_mid_rms_db_range"]
    checks = {
        "artifact_safety": all(
            item["finite"]
            and item["true_peak_dbtp"] <= float(gate["maximum_true_peak_dbtp"])
            and item["clipped_integer_samples"] <= int(gate["maximum_clipped_integer_samples"])
            for item in artifact_metrics.values()
        ),
        "callback_partition": computed["callback_outputs_sample_exact"],
        "restart": computed["restart_outputs_sample_exact"],
        "missing_source": computed["missing_source_silence"],
        "center_attack": computed["center_attack_delta_db_min"] >= attack_range[0]
        and computed["center_attack_delta_db_max"] <= attack_range[1],
        "full_mid_level": mid_range[0] <= full_mid_delta_db <= mid_range[1],
        "mono_compatibility": correlation
        >= float(gate["candidate_mid_control_waveform_correlation_min"]),
        "source_side_eligible": source_side_to_mid_db
        >= float(gate["source_side_to_mid_rms_db_min"]),
        "candidate_side_retained": candidate_side_to_mid_db
        >= float(gate["candidate_side_to_mid_rms_db_min"]),
        "side_stability": side_delta_range[0] <= side_ratio_delta_db <= side_delta_range[1],
        "audible_contrast": stereo_delta_rms
        >= float(gate["stereo_candidate_control_delta_rms_min"]),
    }
    result = "pass" if all(checks.values()) else "fail"
    final = {
        "schema": "riotbox.w30_stereo_pad_technical_preflight.v1",
        "ticket": "RIOTBOX-1469",
        "aggregate_result": result,
        "checks": checks,
        "computed_metrics": computed,
    }
    write_json(preflight_path, final)
    return {
        "result": result,
        "report_path": preflight_path.name,
        "report_sha256": file_sha256(preflight_path),
        "uniform_presentation_gain": uniform_gain,
        "checks": checks,
        "computed_metrics": computed,
    }


def last_capture_window(session_path: Path) -> dict[str, Any]:
    session = json.loads(session_path.read_text(encoding="utf-8"))
    captures = session.get("captures")
    require(isinstance(captures, list) and captures, "renderer produced no capture")
    source_window = captures[-1].get("source_window")
    require(isinstance(source_window, dict), "capture has no source window")
    start = source_window.get("start_frame")
    end = source_window.get("end_frame")
    require(isinstance(start, int) and isinstance(end, int) and end > start, "invalid capture")
    return {
        "source_id": source_window.get("source_id"),
        "start_frame": start,
        "end_frame": end,
        "frame_count": end - start,
        "start_seconds": source_window.get("start_seconds"),
        "end_seconds": source_window.get("end_seconds"),
    }


def write_source_capture_loop(
    payload: bytes, capture: dict[str, Any], output: Path, loop_count: int
) -> None:
    with wave.open(io.BytesIO(payload), "rb") as source:
        start = int(capture["start_frame"])
        frame_count = int(capture["frame_count"])
        require(start + frame_count <= source.getnframes(), "capture exceeds source payload")
        source.setpos(start)
        frames = source.readframes(frame_count)
        channels = source.getnchannels()
        width = source.getsampwidth()
        rate = source.getframerate()
    require(len(frames) == frame_count * channels * width, "short source-window read")
    with wave.open(str(output), "wb") as target:
        target.setnchannels(channels)
        target.setsampwidth(width)
        target.setframerate(rate)
        target.writeframes(frames * loop_count)


def read_pcm_wav(path: Path) -> dict[str, Any]:
    dense.require_numpy()
    np = dense.np
    with wave.open(str(path), "rb") as source:
        channels = source.getnchannels()
        width = source.getsampwidth()
        rate = source.getframerate()
        frames = source.getnframes()
        payload = source.readframes(frames)
    require(channels == 2 and width in (2, 3), f"unsupported WAV format: {path}")
    if width == 2:
        integer = np.frombuffer(payload, dtype="<i2").astype(np.int32)
        scale = float(1 << 15)
        clipped = int(np.count_nonzero((integer == -32768) | (integer == 32767)))
    else:
        raw = np.frombuffer(payload, dtype=np.uint8).reshape(-1, 3)
        integer = (
            raw[:, 0].astype(np.int32)
            | (raw[:, 1].astype(np.int32) << 8)
            | (raw[:, 2].astype(np.int32) << 16)
        )
        integer = np.where(integer & 0x800000, integer - 0x1000000, integer)
        scale = float(1 << 23)
        clipped = int(np.count_nonzero((integer == -8388608) | (integer == 8388607)))
    samples = (integer.astype(np.float64) / scale).reshape(frames, channels)
    return {
        "sample_rate_hz": rate,
        "channels": channels,
        "samples": samples,
        "clipped_integer_samples": clipped,
    }


def write_pcm16(path: Path, sample_rate: int, samples: Any) -> None:
    dense.require_numpy()
    np = dense.np
    pcm = np.rint(np.clip(samples, -1.0, 32767.0 / 32768.0) * 32767.0).astype("<i2")
    with wave.open(str(path), "wb") as target:
        target.setnchannels(2)
        target.setsampwidth(2)
        target.setframerate(sample_rate)
        target.writeframes(pcm.tobytes())


def mid_side(samples: Any) -> tuple[Any, Any]:
    return (samples[:, 0] + samples[:, 1]) / 2.0, (samples[:, 0] - samples[:, 1]) / 2.0


def rms(samples: Any) -> float:
    dense.require_numpy()
    return float(dense.np.sqrt(dense.np.mean(samples.astype(dense.np.float64) ** 2)))


def ratio_db(numerator: float, denominator: float) -> float:
    require(numerator > 0.0 and denominator > 0.0, "non-positive RMS ratio")
    return 20.0 * math.log10(numerator / denominator)


def pearson(first: Any, second: Any) -> float:
    dense.require_numpy()
    np = dense.np
    left = first.astype(np.float64) - float(np.mean(first))
    right = second.astype(np.float64) - float(np.mean(second))
    denominator = float(np.sqrt(np.sum(left * left) * np.sum(right * right)))
    require(denominator > 0.0, "zero waveform-correlation denominator")
    return float(np.sum(left * right) / denominator)


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    raise SystemExit(main())
