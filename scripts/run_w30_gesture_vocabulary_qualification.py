#!/usr/bin/env python3
"""Run RIOTBOX-1447's exact one-source Development Golden Path qualification."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import stat
import subprocess
import wave
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


CONTRACT = Path("docs/benchmarks/w30_gesture_vocabulary_golden_path_qualification_v1.json")
EXPECTED_SHA256 = "80b3433c20168e8b1cc42c399af017faa21a06e509b8240e569485a001d3a7fb"


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def write_log(path: Path, payload: dict[str, Any]) -> None:
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


def hash_exact_regular_file(path: Path) -> str:
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0)
    descriptor = os.open(path, flags)
    try:
        require(stat.S_ISREG(os.fstat(descriptor).st_mode), f"not a regular file: {path}")
        digest = hashlib.sha256()
        while chunk := os.read(descriptor, 1024 * 1024):
            digest.update(chunk)
        return digest.hexdigest()
    finally:
        os.close(descriptor)


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while chunk := handle.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def clean_limiter(report: dict[str, Any], role: str) -> bool:
    metrics = report[role]
    return (
        metrics["pre_limiter_clip_count"] == 0
        and metrics["limited_sample_count"] == 0
        and metrics["post_limiter_clip_count"] == 0
    )


def validate_reports(output: Path, expected_bpm: float) -> dict[str, Any]:
    hook = json.loads((output / "hook-turnaround-qualification.json").read_text())
    pitch = json.loads((output / "pitch-dive-qualification.json").read_text())
    filter_slam = json.loads((output / "filter-slam-qualification.json").read_text())
    journey = json.loads((output / "gesture-vocabulary-qualification.json").read_text())

    for report in (hook, pitch, filter_slam, journey):
        require(
            abs(float(report["exact_product_tempo_bpm"]) - expected_bpm) <= 1.0,
            "product tempo admission failed",
        )
    require(hook["committed_start_beat"] == 8, "Hook Turnaround boundary changed")
    require(hook["reverse_delta_rms"] > 0.001, "Hook Turnaround reverse collapsed")
    require(hook["choke_delta_rms"] > 0.001, "Hook Turnaround choke collapsed")
    require(hook["first_relative_beat_sample_exact"], "Hook Turnaround pre-roll changed")
    require(hook["return_from_relative_beat_four_sample_exact"], "Hook Turnaround return changed")
    require(hook["callback_partition_128_vs_257_sample_exact"], "Hook partition mismatch")

    require(pitch["committed_start_beat"] == 16, "Pitch Dive boundary changed")
    require(pitch["final_four_beat_delta_rms"] > 0.001, "Pitch Dive collapsed")
    require(pitch["first_eight_beats_sample_exact"], "Pitch Dive control changed")
    require(pitch["silence_from_beat_twelve"], "Pitch Dive terminal exit changed")
    require(pitch["callback_partition_128_vs_257_sample_exact"], "Pitch partition mismatch")

    require(filter_slam["committed_start_beat"] == 32, "Filter Slam boundary changed")
    require(filter_slam["effect_through_return_delta_rms"] > 0.001, "Filter Slam collapsed")
    require(filter_slam["sample_exact_after_return"], "Filter Slam return changed")
    require(filter_slam["callback_partition_128_vs_257_sample_exact"], "Filter partition mismatch")

    for report in (hook, pitch, filter_slam):
        require(clean_limiter(report, "control"), "control limiter gate failed")
        require(clean_limiter(report, "candidate"), "candidate limiter gate failed")
        require(report["capture_lineage_unchanged"], "capture lineage changed")
        require(report["source_monitor_unchanged"], "Source Monitor changed")
        require(report["missing_source_active_samples"] == 0, "missing source emitted audio")

    require(
        journey["action_order"]
        == [
            "w30.hook_turnaround",
            "w30.trigger_pad",
            "w30.pitch_dive",
            "w30.trigger_pad",
            "w30.filter_slam",
            "w30.trigger_pad",
        ],
        "journey action order changed",
    )
    require(journey["session_round_trip_exact"], "Session round trip failed")
    require(journey["suffix_replay_equivalent"], "journey replay failed")
    require(journey["final_articulation_cleared"], "final articulation remained active")
    continuous = journey["continuous_journey"]
    require(continuous["duration_beats"] == 37, "continuous journey duration changed")
    require(
        continuous["callback_partition_128_vs_257_sample_exact"],
        "continuous journey callback partition mismatch",
    )
    require(
        continuous["active_samples"] > 0
        and continuous["rms"] > 0.001
        and continuous["pre_limiter_clip_count"] == 0
        and continuous["limited_sample_count"] == 0
        and continuous["post_limiter_clip_count"] == 0,
        "continuous journey was silent, clipped, or limited",
    )
    journey_path = output / continuous["path"]
    with wave.open(str(journey_path), "rb") as handle:
        require(handle.getframerate() == 48000 and handle.getnchannels() == 2, "journey WAV format changed")
        actual_frames = handle.getnframes()
    expected_frames = sum(
        round(beats * 60.0 / float(journey["exact_product_tempo_bpm"]) * 48000)
        for beats in (5, 3, 13, 3, 9, 4)
    )
    require(actual_frames == expected_frames, "continuous journey WAV frame count changed")
    require(len(journey["ordinary_reentries"]) == 3, "re-entry count changed")
    for reentry in journey["ordinary_reentries"]:
        require(reentry["articulation_cleared"], "re-entry did not clear articulation")
        require(reentry["active_samples"] > 0 and reentry["rms"] > 0.001, "re-entry was silent")
        require(
            reentry["pre_limiter_clip_count"] == 0
            and reentry["limited_sample_count"] == 0
            and reentry["post_limiter_clip_count"] == 0,
            "re-entry limiter gate failed",
        )
    return journey


def prepare_review(repo: Path, contract: dict[str, Any], session: str) -> int:
    root = repo / "artifacts/development/riotbox-1447"
    technical_log_path = root / f"access-log-{session}.json"
    technical_log = json.loads(technical_log_path.read_text(encoding="utf-8"))
    require(technical_log["status"] == "passed", "technical qualification has not passed")
    output = root / f"golden-path-{session}"
    journey_report = validate_reports(output, float(contract["source"]["expected_product_bpm"]))
    product_bpm = float(journey_report["exact_product_tempo_bpm"])
    journey_path = output / journey_report["continuous_journey"]["path"]
    review_dir = root / f"review-{session}"
    review_log_path = root / f"review-access-log-{session}.json"
    require(not review_dir.exists(), f"review directory already exists: {review_dir}")
    require(not review_log_path.exists(), f"review access log already exists: {review_log_path}")
    review_dir.mkdir()
    review_path = review_dir / "01_source_then_continuous_gesture_journey.wav"
    manifest_path = review_dir / "review-manifest.json"
    source = contract["source"]
    source_path = repo / source["path"]
    log: dict[str, Any] = {
        "schema": "riotbox.w30_gesture_vocabulary_review_access_log.v1",
        "ticket": "RIOTBOX-1447",
        "session": session,
        "started_at_utc": utc_now(),
        "technical_access_log_sha256": file_sha256(technical_log_path),
        "mode": "one_exact_development_source_no_directory_discovery",
        "source_path": source["path"],
        "expected_source_sha256": source["sha256"],
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "status": "started_before_source_open",
    }
    review_log_path.touch(mode=0o600, exist_ok=False)
    write_log(review_log_path, log)
    try:
        pre_hash = hash_exact_regular_file(source_path)
        require(pre_hash == source["sha256"], "review source SHA-256 mismatch")
        log.update({"pre_build_source_sha256": pre_hash, "status": "building_exact_review_artifact"})
        write_log(review_log_path, log)
        source_context_seconds = 8.0 * 60.0 / product_bpm
        filter_graph = (
            f"[0:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,"
            f"atrim=0:{source_context_seconds:.9f},asetpts=PTS-STARTPTS,volume=0.501187[src];"
            "anullsrc=r=48000:cl=stereo:d=1[sil];"
            "[1:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo[journey];"
            "[src][sil][journey]concat=n=3:v=0:a=1[out]"
        )
        completed = subprocess.run(
            [
                "ffmpeg", "-v", "error", "-nostdin", "-i", str(source_path), "-i", str(journey_path),
                "-filter_complex", filter_graph, "-map", "[out]", "-c:a", "pcm_s16le", str(review_path),
            ],
            cwd=repo,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        require(completed.returncode == 0, f"review artifact build failed: {completed.stdout[-1000:]}")
        post_hash = hash_exact_regular_file(source_path)
        require(post_hash == pre_hash, "review source changed during artifact build")
        seconds_per_beat = 60.0 / product_bpm
        journey_start = source_context_seconds + 1.0
        boundaries = [
            ("source_context", 0.0, source_context_seconds, ["registered_development_source"]),
            ("silence", source_context_seconds, journey_start, []),
            ("hook_turnaround_phrase_variation", journey_start, journey_start + 5 * seconds_per_beat, ["w30_preview"]),
            ("ordinary_reentry_after_hook", journey_start + 5 * seconds_per_beat, journey_start + 8 * seconds_per_beat, ["w30_preview"]),
            ("pitch_dive_destructive_exit", journey_start + 8 * seconds_per_beat, journey_start + 21 * seconds_per_beat, ["w30_preview"]),
            ("ordinary_reentry_after_pitch", journey_start + 21 * seconds_per_beat, journey_start + 24 * seconds_per_beat, ["w30_preview"]),
            ("filter_slam_long_build_return", journey_start + 24 * seconds_per_beat, journey_start + 33 * seconds_per_beat, ["w30_preview"]),
            ("ordinary_reentry_after_filter", journey_start + 33 * seconds_per_beat, journey_start + 37 * seconds_per_beat, ["w30_preview"]),
        ]
        manifest = {
            "schema": "riotbox.w30_gesture_vocabulary_human_review.v1",
            "ticket": "RIOTBOX-1447",
            "human_verdict": "unverified",
            "artifact": review_path.relative_to(repo).as_posix(),
            "artifact_sha256": file_sha256(review_path),
            "sample_rate": 48000,
            "channels": 2,
            "product_bpm": product_bpm,
            "continuous_product_journey_sha256": file_sha256(journey_path),
            "segments": [
                {"role": role, "start_seconds": start, "end_seconds": end, "audible_contributors": contributors}
                for role, start, end, contributors in boundaries
            ],
            "questions": contract["human_review"]["questions"],
        }
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        log.update(
            {
                "post_build_source_sha256": post_hash,
                "continuous_journey_sha256": file_sha256(journey_path),
                "review_artifact_path": review_path.relative_to(repo).as_posix(),
                "review_artifact_sha256": manifest["artifact_sha256"],
                "review_manifest_sha256": file_sha256(manifest_path),
                "status": "passed",
                "completed_at_utc": utc_now(),
            }
        )
        write_log(review_log_path, log)
        print(review_path.relative_to(repo))
        return 0
    except Exception as error:
        log.update({"status": "failed_closed", "failure": str(error), "completed_at_utc": utc_now()})
        write_log(review_log_path, log)
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--session", required=True)
    parser.add_argument("--prepare-review-from-passed-session", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    contract_path = repo / CONTRACT
    contract_bytes = contract_path.read_bytes()
    contract_hash = hashlib.sha256(contract_bytes).hexdigest()
    require(contract_hash == EXPECTED_SHA256, "frozen qualification contract changed")
    contract = json.loads(contract_bytes)
    require(
        contract["schema"] == "riotbox.w30_gesture_vocabulary_golden_path_qualification.v1",
        "unexpected qualification schema",
    )
    access = contract["access"]
    require(access["maximum_unique_development_files"] == 1, "access bound changed")
    require(not access["directory_discovery_allowed"], "directory discovery must stay forbidden")
    require(not access["holdout_audio_allowed"], "Holdout access must stay forbidden")
    require(not access["commercial_reference_audio_allowed"], "commercial references forbidden")

    if args.prepare_review_from_passed_session:
        return prepare_review(repo, contract, args.session)

    source = contract["source"]
    source_path = repo / source["path"]
    root = repo / "artifacts/development/riotbox-1447"
    output = root / f"golden-path-{args.session}"
    log_path = root / f"access-log-{args.session}.json"
    root.mkdir(parents=True, exist_ok=True)
    require(not log_path.exists(), f"access log already exists: {log_path}")
    require(not output.exists(), f"output already exists: {output}")
    output.mkdir()
    log: dict[str, Any] = {
        "schema": "riotbox.w30_gesture_vocabulary_access_log.v1",
        "ticket": "RIOTBOX-1447",
        "session": args.session,
        "started_at_utc": utc_now(),
        "contract_path": CONTRACT.as_posix(),
        "contract_sha256": contract_hash,
        "mode": "one_exact_development_path_no_directory_discovery",
        "source_path": source["path"],
        "expected_source_sha256": source["sha256"],
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "status": "started_before_source_open",
    }
    log_path.touch(mode=0o600, exist_ok=False)
    write_log(log_path, log)

    try:
        pre_hash = hash_exact_regular_file(source_path)
        require(pre_hash == source["sha256"], "source SHA-256 mismatch")
        log.update({"pre_render_source_sha256": pre_hash, "status": "rendering_exact_product_path"})
        write_log(log_path, log)
        command = [
            "cargo", "run", "--quiet", "-p", "riotbox-app", "--bin", "w30_live_path_render", "--",
            "--source", str(source_path), "--output", str(output), "--bpm", str(source["confirmation_bpm"]),
            "--qualify-gesture-vocabulary-v1",
        ]
        completed = subprocess.run(
            command, cwd=repo, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False
        )
        log["renderer_exit_code"] = completed.returncode
        log["renderer_output_tail"] = completed.stdout.splitlines()[-12:]
        require(completed.returncode == 0, "exact product renderer failed")
        post_hash = hash_exact_regular_file(source_path)
        require(post_hash == pre_hash, "source changed during qualification")
        journey = validate_reports(output, float(source["expected_product_bpm"]))
        log.update(
            {
                "post_render_source_sha256": post_hash,
                "journey_report_sha256": file_sha256(output / "gesture-vocabulary-qualification.json"),
                "action_order": journey["action_order"],
                "status": "passed",
                "completed_at_utc": utc_now(),
            }
        )
        write_log(log_path, log)
        print(log_path.relative_to(repo))
        return 0
    except Exception as error:
        log.update({"status": "failed_closed", "failure": str(error), "completed_at_utc": utc_now()})
        write_log(log_path, log)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
