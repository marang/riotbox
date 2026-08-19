#!/usr/bin/env python3
"""Run RIOTBOX-1446's exact four-source Development qualification fail-closed."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import stat
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


CONTRACT = Path("docs/benchmarks/w30_filter_slam_product_qualification_v1.json")
EXPECTED_SCHEMA = "riotbox.w30_filter_slam_product_qualification.v1"
EXPECTED_CONTRACT_SHA256 = "05181ea58ffa060d6f0626e5363dbf6f804e5ea9d3d656192b0d513715f67d6e"


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def write_log(path: Path, payload: dict[str, Any]) -> None:
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


def hash_exact_regular_file(path: Path) -> str:
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0)
    descriptor = os.open(path, flags)
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            raise RuntimeError(f"not a regular file: {path}")
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


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def limiter_is_clean(report: dict[str, Any], role: str) -> bool:
    metrics = report[role]
    return (
        metrics["pre_limiter_clip_count"] == 0
        and metrics["limited_sample_count"] == 0
        and metrics["post_limiter_clip_count"] == 0
    )


def validate_contract(contract: dict[str, Any]) -> list[dict[str, Any]]:
    require(contract.get("schema") == EXPECTED_SCHEMA, "unexpected qualification schema")
    require(contract.get("partition") == "development", "qualification must stay Development-only")
    require(
        contract.get("mechanism_contract", {}).get("mechanism") == "w30_filter_slam_v1",
        "unexpected mechanism contract",
    )
    require(
        contract.get("product_path", {}).get("action") == "w30.filter_slam",
        "unexpected product action",
    )
    access = contract.get("access", {})
    require(access.get("exact_paths_and_hashes_only") is True, "exact paths are required")
    require(access.get("directory_discovery_allowed") is False, "directory discovery forbidden")
    require(access.get("holdout_audio_allowed") is False, "Holdout access forbidden")
    require(
        access.get("commercial_reference_audio_allowed") is False,
        "commercial-reference access forbidden",
    )
    require(access.get("source_replacement_allowed") is False, "source replacement forbidden")
    sources = contract.get("source_set")
    require(isinstance(sources, list) and len(sources) == 4, "contract must contain four sources")
    require(
        len(sources) <= int(access.get("maximum_unique_development_files", 0)),
        "source count exceeds frozen access maximum",
    )
    paths = [source.get("path") for source in sources]
    case_ids = [source.get("case_id") for source in sources]
    require(len(paths) == len(set(paths)), "source paths must be unique")
    require(len(case_ids) == len(set(case_ids)), "case IDs must be unique")
    require(all(isinstance(path, str) and path for path in paths), "invalid source path")
    require(
        all(
            isinstance(source.get("sha256"), str) and len(source["sha256"]) == 64
            for source in sources
        ),
        "every source requires an exact SHA-256",
    )
    return sources


def prepare_review(repo: Path, contract: dict[str, Any], session: str) -> int:
    root = repo / "artifacts/development/riotbox-1446"
    qualification_log = root / f"access-log-{session}.json"
    qualification = json.loads(qualification_log.read_text(encoding="utf-8"))
    require(
        qualification.get("status") == "passed_all_four_cases",
        "technical matrix must pass before review preparation",
    )
    representative_id = contract["human_review"]["representative_case"]
    source = next(
        item for item in contract["source_set"] if item["case_id"] == representative_id
    )
    source_path = repo / source["path"]
    case_output = root / f"product-qualification-{session}" / representative_id
    control_path = case_output / "05_w30_filter_slam_control.wav"
    candidate_path = case_output / "06_w30_filter_slam_candidate_v1.wav"
    review_dir = root / f"product-review-{session}"
    review_log_path = root / f"review-access-log-{session}.json"
    require(not review_log_path.exists(), f"review access log already exists: {review_log_path}")
    require(not review_dir.exists(), f"review directory already exists: {review_dir}")
    review_dir.mkdir()
    review_path = review_dir / "01_source_A_B_product_review.wav"
    review_log: dict[str, Any] = {
        "schema": "riotbox.w30_filter_slam_product_review_access_log.v1",
        "ticket": "RIOTBOX-1446",
        "session": session,
        "started_at_utc": utc_now(),
        "technical_access_log_sha256": file_sha256(qualification_log),
        "mode": "one_exact_representative_development_source_no_directory_discovery",
        "case_id": representative_id,
        "source_path": source["path"],
        "expected_source_sha256": source["sha256"],
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "status": "started_before_source_open",
    }
    review_log_path.touch(mode=0o600, exist_ok=False)
    write_log(review_log_path, review_log)
    try:
        pre_hash = hash_exact_regular_file(source_path)
        require(pre_hash == source["sha256"], "representative source SHA-256 mismatch")
        review_log["pre_render_source_sha256"] = pre_hash
        review_log["status"] = "building_review_artifact"
        write_log(review_log_path, review_log)

        filter_graph = (
            "[0:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,"
            "volume=0.501187,asplit=4[s1][s2][s3][s4];"
            "[s1][s2]acrossfade=d=0.02:c1=tri:c2=tri[s12];"
            "[s12][s3]acrossfade=d=0.02:c1=tri:c2=tri[s123];"
            "[s123][s4]acrossfade=d=0.02:c1=tri:c2=tri[src];"
            "[1:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,asplit=2[a1][a2];"
            "[a1][a2]acrossfade=d=0.02:c1=tri:c2=tri[A];"
            "[2:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,asplit=2[b1][b2];"
            "[b1][b2]acrossfade=d=0.02:c1=tri:c2=tri[B];"
            "anullsrc=r=48000:cl=stereo:d=1[sil1];anullsrc=r=48000:cl=stereo:d=1[sil2];"
            "[src][sil1][A][sil2][B]concat=n=5:v=0:a=1,"
            "areverse,afade=t=in:d=0.05,areverse[out]"
        )
        completed = subprocess.run(
            [
                "ffmpeg",
                "-v",
                "error",
                "-nostdin",
                "-i",
                str(source_path),
                "-i",
                str(control_path),
                "-i",
                str(candidate_path),
                "-filter_complex",
                filter_graph,
                "-map",
                "[out]",
                "-c:a",
                "pcm_s16le",
                str(review_path),
            ],
            cwd=repo,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        require(completed.returncode == 0, f"review ffmpeg failed: {completed.stdout[-1000:]}")
        post_hash = hash_exact_regular_file(source_path)
        require(post_hash == pre_hash, "representative source changed during review build")
        review_log.update(
            {
                "post_render_source_sha256": post_hash,
                "control_sha256": file_sha256(control_path),
                "candidate_sha256": file_sha256(candidate_path),
                "review_artifact_path": review_path.relative_to(repo).as_posix(),
                "review_artifact_sha256": file_sha256(review_path),
                "audible_order": contract["human_review"]["order"],
                "status": "passed",
                "completed_at_utc": utc_now(),
            }
        )
        write_log(review_log_path, review_log)
        print(review_path.relative_to(repo))
        return 0
    except Exception as error:
        review_log["status"] = "failed_closed"
        review_log["failure"] = str(error)
        review_log["completed_at_utc"] = utc_now()
        write_log(review_log_path, review_log)
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--session", default="v1")
    parser.add_argument("--prepare-review-from-passed-session", action="store_true")
    args = parser.parse_args()
    require(args.session.replace("-", "").isalnum(), "session must be a safe token")

    repo = Path(__file__).resolve().parent.parent
    contract_path = repo / CONTRACT
    contract_bytes = contract_path.read_bytes()
    require(
        hashlib.sha256(contract_bytes).hexdigest() == EXPECTED_CONTRACT_SHA256,
        "frozen qualification contract SHA-256 changed; create a new version and Decision",
    )
    contract = json.loads(contract_bytes)
    sources = validate_contract(contract)
    if args.prepare_review_from_passed_session:
        return prepare_review(repo, contract, args.session)

    root = repo / "artifacts/development/riotbox-1446"
    output_root = root / f"product-qualification-{args.session}"
    access_log_path = root / f"access-log-{args.session}.json"
    root.mkdir(parents=True, exist_ok=True)
    require(not access_log_path.exists(), f"access log already exists: {access_log_path}")
    require(not output_root.exists(), f"output directory already exists: {output_root}")
    output_root.mkdir()

    access_log: dict[str, Any] = {
        "schema": "riotbox.w30_filter_slam_product_access_log.v1",
        "ticket": "RIOTBOX-1446",
        "session": args.session,
        "started_at_utc": utc_now(),
        "contract_path": CONTRACT.as_posix(),
        "contract_sha256": hashlib.sha256(contract_bytes).hexdigest(),
        "mode": "exact_development_paths_only_no_glob_or_directory_discovery",
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "maximum_unique_development_files": 4,
        "requested_case_ids": [source["case_id"] for source in sources],
        "cases": [],
        "status": "started_before_first_source_open",
    }
    access_log_path.touch(mode=0o600, exist_ok=False)
    write_log(access_log_path, access_log)

    try:
        for source in sources:
            case_id = source["case_id"]
            relative_source = Path(source["path"])
            source_path = repo / relative_source
            case_output = output_root / case_id
            case_output.mkdir()
            record: dict[str, Any] = {
                "case_id": case_id,
                "family": source["family"],
                "path": relative_source.as_posix(),
                "expected_sha256": source["sha256"],
                "opened_at_utc": utc_now(),
                "status": "pre_render_hashing",
            }
            access_log["cases"].append(record)
            write_log(access_log_path, access_log)

            pre_hash = hash_exact_regular_file(source_path)
            record["pre_render_sha256"] = pre_hash
            require(pre_hash == source["sha256"], f"{case_id}: source SHA-256 mismatch")
            record["status"] = "rendering_exact_product_path"
            write_log(access_log_path, access_log)

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
                str(source_path),
                "--output",
                str(case_output),
                "--bpm",
                str(source["confirmation_bpm"]),
                "--qualify-filter-slam-v1",
            ]
            if source.get("downbeat_seconds") is not None:
                command.extend(["--downbeat-seconds", str(source["downbeat_seconds"])])
            if case_id == contract["human_review"]["representative_case"]:
                command.append("--prepare-filter-slam-review")
            completed = subprocess.run(
                command,
                cwd=repo,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                check=False,
            )
            record["renderer_exit_code"] = completed.returncode
            record["renderer_output_tail"] = completed.stdout.splitlines()[-8:]
            require(completed.returncode == 0, f"{case_id}: product renderer failed")

            post_hash = hash_exact_regular_file(source_path)
            record["post_render_sha256"] = post_hash
            require(post_hash == pre_hash, f"{case_id}: source changed during render")
            report_path = case_output / "filter-slam-qualification.json"
            report = json.loads(report_path.read_text(encoding="utf-8"))
            product_bpm = float(report["exact_product_tempo_bpm"])
            require(
                abs(product_bpm - float(source["expected_product_bpm"]))
                <= float(contract["admission"]["maximum_absolute_product_bpm_delta"]),
                f"{case_id}: product BPM admission failed",
            )
            require(report["sample_exact_after_return"], f"{case_id}: return changed")
            require(
                report["callback_partition_128_vs_257_sample_exact"],
                f"{case_id}: callback partition mismatch",
            )
            require(
                float(report["effect_through_return_delta_rms"])
                > float(
                    contract["technical_gates"][
                        "minimum_effect_through_return_delta_rms_exclusive"
                    ]
                ),
                f"{case_id}: effect delta gate failed",
            )
            for key in (
                "capture_lineage_unchanged",
                "grit_unchanged",
                "music_bus_level_unchanged",
                "source_monitor_unchanged",
                "other_lanes_unchanged",
            ):
                require(report[key], f"{case_id}: {key} gate failed")
            require(report["missing_source_active_samples"] == 0, f"{case_id}: fallback audio")
            require(limiter_is_clean(report, "control"), f"{case_id}: control limiter gate")
            require(limiter_is_clean(report, "candidate"), f"{case_id}: candidate limiter gate")
            record.update(
                {
                    "product_bpm": product_bpm,
                    "qualification_report_sha256": file_sha256(report_path),
                    "control_sha256": file_sha256(
                        case_output / "05_w30_filter_slam_control.wav"
                    ),
                    "candidate_sha256": file_sha256(
                        case_output / "06_w30_filter_slam_candidate_v1.wav"
                    ),
                    "effect_through_return_delta_rms": report[
                        "effect_through_return_delta_rms"
                    ],
                    "status": "passed",
                    "completed_at_utc": utc_now(),
                }
            )
            write_log(access_log_path, access_log)

        access_log["status"] = "passed_all_four_cases"
        access_log["completed_at_utc"] = utc_now()
        write_log(access_log_path, access_log)
        print(access_log_path.relative_to(repo))
        return 0
    except Exception as error:
        access_log["status"] = "failed_closed"
        access_log["failure"] = str(error)
        access_log["completed_at_utc"] = utc_now()
        write_log(access_log_path, access_log)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
