#!/usr/bin/env python3
"""Run the one-shot RIOTBOX-1470 Dense W-30 foundation qualification."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from dense_w30_foundation_manifest import (
    PRIOR_AUDIO_SHA256,
    build_current_manifest,
    reconstruct_prior_manifest,
    serialize_manifest,
)
from hash_identical_human_verdict_reuse import validate_reuse_evidence
from source_holdout_development_access import (
    create_exclusive_access_log,
    persist_access_log,
    validate_contained_source_file,
)


CONTRACT_PATH = Path("docs/benchmarks/dense_w30_foundation_qualification_v1.json")
CONTRACT_SHA256 = "fa49cce7617d2445328a98e225e5756110907cdabe4828a151a25129b9d11715"
REGISTRY_PATH = Path("docs/benchmarks/sound_excellence_source_corpus_v1.json")
REGISTRY_SHA256 = "67b5b8b2882575cf70fa61aacf25ae282c17714fe51ffcb13f905458e025d552"
PRIOR_MANIFEST_PATH = Path("docs/benchmarks/dense_w30_foundation_prior_product_manifest_v1.json")
PRIOR_MANIFEST_SHA256 = "6b593663a24ae130e2352ea1dcbe09489ba86ac3f32d8efb68b6ac7c4709c69a"
CASE_ID = "dense_beat03_130"
SOURCE_PATH = Path("data/test_audio/examples/Beat03_130BPM(Full).wav")
SOURCE_SHA256 = "e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a"
SOURCE_FORMAT = {
    "sample_rate_hz": 44_100,
    "channels": 2,
    "sample_width_bits": 24,
    "compression_type": "NONE",
    "maximum_duration_seconds": 16,
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


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
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def repo_path(repo: Path, path: Path) -> str:
    return path.resolve().relative_to(repo.resolve()).as_posix()


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def validate_manifest_identity(manifest: dict[str, Any], audio_sha256: str) -> list[dict[str, Any]]:
    prior_payload = PRIOR_MANIFEST_PATH.read_bytes()
    current_payload = serialize_manifest(manifest)
    return [
        {
            "gate": "audio_bit_identical_to_prior_reviewed_control",
            "expected": PRIOR_AUDIO_SHA256,
            "actual": audio_sha256,
            "pass": audio_sha256 == PRIOR_AUDIO_SHA256,
        },
        {
            "gate": "product_manifest_bit_identical_to_prior_reconstruction",
            "expected": PRIOR_MANIFEST_SHA256,
            "actual": sha256_bytes(current_payload),
            "pass": current_payload == prior_payload,
        },
    ]


def validate_source_blind(repo: Path) -> None:
    require(sha256_file(repo / REGISTRY_PATH) == REGISTRY_SHA256, "registry hash changed")
    require(
        sha256_file(repo / PRIOR_MANIFEST_PATH) == PRIOR_MANIFEST_SHA256,
        "prior product manifest hash changed",
    )
    reconstructed = reconstruct_prior_manifest(repo)
    require(
        serialize_manifest(reconstructed) == (repo / PRIOR_MANIFEST_PATH).read_bytes(),
        "historical reconstruction no longer matches its frozen manifest",
    )
    require(
        all(gate["pass"] for gate in validate_manifest_identity(reconstructed, PRIOR_AUDIO_SHA256)),
        "exact historical identity rejected",
    )
    mutations: list[tuple[str, dict[str, Any], str]] = []
    changed_lane = copy.deepcopy(reconstructed)
    changed_lane["lane_roles"]["tr909"] = "support"
    mutations.append(("lane_role", changed_lane, PRIOR_AUDIO_SHA256))
    changed_action = copy.deepcopy(reconstructed)
    changed_action["performer_path"]["actions"][5]["command"] = "W30PitchDive"
    mutations.append(("action_path", changed_action, PRIOR_AUDIO_SHA256))
    mutations.append(("audio_hash", copy.deepcopy(reconstructed), "0" * 64))
    for label, manifest, audio_sha256 in mutations:
        require(
            not all(gate["pass"] for gate in validate_manifest_identity(manifest, audio_sha256)),
            f"source-blind mutation was admitted: {label}",
        )
    print("Dense W-30 foundation source-blind identity fixtures: pass")


def validate_contract(repo: Path) -> dict[str, Any]:
    path = repo / CONTRACT_PATH
    require(path.is_file(), f"missing frozen contract: {path}")
    require(sha256_file(path) == CONTRACT_SHA256, "frozen contract hash changed")
    contract = read_json(path)
    require(contract.get("schema") == "riotbox.dense_w30_foundation_qualification.v1", "contract schema changed")
    require(contract.get("status") == "frozen", "contract is not frozen")
    access = contract.get("development_access")
    require(isinstance(access, dict), "contract development_access missing")
    require(access.get("case_id") == CASE_ID, "contract case changed")
    require(access.get("source_path") == SOURCE_PATH.as_posix(), "contract source path changed")
    require(access.get("source_sha256") == SOURCE_SHA256, "contract source hash changed")
    require(access.get("source_format") == SOURCE_FORMAT, "contract source format changed")
    roles = contract.get("lane_roles")
    require(
        roles
        == {
            "w30": "source_transform_foundation",
            "tr909": "stay_out",
            "mc202": "stay_out",
            "source_monitor": "stay_out",
        },
        "contract lane ownership changed",
    )
    return contract


def run_qualification(repo: Path, output_dir: Path, access_log_path: Path) -> None:
    validate_source_blind(repo)
    contract = validate_contract(repo)
    require(not output_dir.exists(), f"qualification output already exists: {output_dir}")
    output_dir.mkdir(parents=True)
    access_log: dict[str, Any] = {
        "schema": "riotbox.dense_w30_foundation_access_log.v1",
        "ticket": "RIOTBOX-1470",
        "started_at_utc": utc_now(),
        "mode": "one_exact_development_file_no_directory_discovery",
        "contract": {"path": CONTRACT_PATH.as_posix(), "sha256": CONTRACT_SHA256},
        "registry": {"path": REGISTRY_PATH.as_posix(), "sha256": REGISTRY_SHA256},
        "requested_case_ids": [CASE_ID],
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_opened": False,
        "opened_development_files": [],
        "status": "preflight_pending",
        "qualification_status": "not_run",
    }
    with create_exclusive_access_log(access_log_path) as log_file:
        persist_access_log(log_file, access_log)
        try:
            registry = read_json(repo / REGISTRY_PATH)
            require(sha256_file(repo / REGISTRY_PATH) == REGISTRY_SHA256, "registry hash changed")
            entries = registry.get("entries")
            require(isinstance(entries, list), "registry entries missing")
            selected = [entry for entry in entries if entry.get("case_id") == CASE_ID]
            require(len(selected) == 1, "registered source identity is not unique")
            entry = selected[0]
            require(entry.get("source_family") == "dense_break", "source family changed")
            require(entry.get("source_path") == SOURCE_PATH.as_posix(), "registered source path changed")
            require(entry.get("bpm_hint") == 130.0, "registered source BPM hint changed")
            access_log["status"] = "preflight_passed"
            persist_access_log(log_file, access_log)

            opened: dict[str, Any] = {
                "case_id": CASE_ID,
                "source_path": SOURCE_PATH.as_posix(),
                "expected_sha256": SOURCE_SHA256,
                "access_verification_status": "opening",
            }

            def record_open(_: Path) -> None:
                access_log["opened_development_files"].append(opened)
                access_log["status"] = "source_opened"
                persist_access_log(log_file, access_log)

            payload, source_result = validate_contained_source_file(
                repo,
                SOURCE_PATH,
                SOURCE_SHA256,
                SOURCE_FORMAT,
                f"{CONTRACT_PATH}: {CASE_ID}",
                on_open=record_open,
                return_payload=True,
            )
            opened.update(source_result)
            opened.pop("sample_bytes", None)
            opened["access_verification_status"] = "verified_and_delivered_to_qualification"
            access_log["status"] = "rendering"
            persist_access_log(log_file, access_log)

            with tempfile.TemporaryDirectory(prefix="riotbox-1470-source-") as temporary:
                temporary_source = Path(temporary) / "verified-source.wav"
                temporary_source.write_bytes(payload)
                completed = subprocess.run(
                    [
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
                        str(output_dir),
                        "--bpm",
                        "130.0",
                        "--qualify-dense-w30-foundation-v1",
                    ],
                    cwd=repo,
                    check=False,
                )
            require(completed.returncode == 0, f"product renderer exited {completed.returncode}")

            audio_path = output_dir / "05_w30_dense_foundation_control.wav"
            session_path = output_dir / "session.json"
            graph_path = output_dir / "source-graph.json"
            runtime_path = output_dir / "dense-w30-foundation-runtime.json"
            for path in (audio_path, session_path, graph_path, runtime_path):
                require(path.is_file(), f"renderer output missing: {path}")
            audio_sha256 = sha256_file(audio_path)
            current_manifest = build_current_manifest(
                session_path,
                graph_path,
                runtime_path,
                audio_sha256,
            )
            manifest_path = output_dir / "dense-w30-foundation-product-manifest.json"
            manifest_path.write_bytes(serialize_manifest(current_manifest))
            identity_gates = validate_manifest_identity(current_manifest, audio_sha256)
            runtime = read_json(runtime_path)
            render = runtime["render"]
            technical_gates = [
                {"gate": "callback_partitions_sample_exact", "pass": render.get("callback_partitions_sample_exact") is True},
                {"gate": "restart_sample_exact", "pass": render.get("restart_sample_exact") is True},
                {"gate": "missing_source_silent", "actual": render.get("missing_source_active_samples"), "pass": render.get("missing_source_active_samples") == 0},
                {"gate": "no_pre_limiter_clips", "actual": render.get("pre_limiter_clip_count"), "pass": render.get("pre_limiter_clip_count") == 0},
                {"gate": "no_limiter_intervention", "actual": render.get("limited_sample_count"), "pass": render.get("limited_sample_count") == 0},
                {"gate": "no_post_limiter_clips", "actual": render.get("post_limiter_clip_count"), "pass": render.get("post_limiter_clip_count") == 0},
                {"gate": "non_silent", "actual": render.get("active_samples"), "pass": isinstance(render.get("active_samples"), int) and render["active_samples"] > 0},
            ]
            all_gates = technical_gates + identity_gates
            passed = all(gate["pass"] for gate in all_gates)
            result = {
                "schema": "riotbox.dense_w30_foundation_qualification_result.v1",
                "ticket": "RIOTBOX-1470",
                "contract": {"path": CONTRACT_PATH.as_posix(), "sha256": CONTRACT_SHA256},
                "source": {"case_id": CASE_ID, "sha256": SOURCE_SHA256},
                "artifacts": {
                    "audio_path": repo_path(repo, audio_path),
                    "audio_sha256": audio_sha256,
                    "product_manifest_path": repo_path(repo, manifest_path),
                    "product_manifest_sha256": sha256_file(manifest_path),
                    "session_sha256": sha256_file(session_path),
                    "source_graph_sha256": sha256_file(graph_path),
                    "runtime_sha256": sha256_file(runtime_path),
                },
                "gates": all_gates,
                "result": "pass_hash_identical_reuse" if passed else "failed_closed",
                "human_playback_required": False,
                "new_human_verdict": False,
            }
            result_path = output_dir / "dense-w30-foundation-qualification.json"
            write_json(result_path, result)
            access_log["qualification_result"] = {
                "path": repo_path(repo, result_path),
                "sha256": sha256_file(result_path),
                "result": result["result"],
            }
            access_log["qualification_status"] = result["result"]
            access_log["status"] = "identity_pass_pending_reuse" if passed else "failed_closed"
            if not passed:
                access_log["completed_at_utc"] = utc_now()
            persist_access_log(log_file, access_log)
            require(passed, "Dense W-30 foundation identity qualification failed closed")

            prior = contract["prior_human_evidence"]
            reuse = {
                "schema": "riotbox.hash_identical_human_verdict_reuse.v1",
                "result": "pass",
                "reuse_contract": {"path": CONTRACT_PATH.as_posix(), "sha256": CONTRACT_SHA256},
                "prior_ticket": prior["ticket"],
                "prior_structured_review_sha256": prior["structured_review_sha256"],
                "current_audio_sha256": audio_sha256,
                "current_product_manifest_sha256": sha256_file(manifest_path),
                "current_replay_created_new_verdict": False,
                "new_quality_evidence": False,
                "additional_human_playback_required": False,
            }
            reuse_path = output_dir / "human-verdict-reuse.json"
            write_json(reuse_path, reuse)
            validate_reuse_evidence(
                reuse,
                reuse_path,
                current_audio_sha256=audio_sha256,
                current_product_manifest_sha256=sha256_file(manifest_path),
                expected_prior_human_verdict="keep",
                current_verdict_dimensions={
                    "strongest_element": "chop",
                    "source_recognition": "source_transformed_but_present",
                    "hook_after_two_bars": "clear",
                },
            )
            access_log["human_verdict_reuse"] = {
                "path": repo_path(repo, reuse_path),
                "sha256": sha256_file(reuse_path),
                "result": "pass",
            }
            access_log["status"] = "completed"
            access_log["completed_at_utc"] = utc_now()
            persist_access_log(log_file, access_log)
            print(json.dumps(result, indent=2, sort_keys=True))
        except Exception as error:
            if access_log.get("completed_at_utc") is None:
                access_log["status"] = "failed_closed"
                access_log["qualification_status"] = "failed_closed"
                access_log["failure_type"] = type(error).__name__
                access_log["failure"] = str(error)
                access_log["completed_at_utc"] = utc_now()
                persist_access_log(log_file, access_log)
            raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--validate-source-blind", action="store_true")
    parser.add_argument("--run", action="store_true")
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("artifacts/development/riotbox-1470/qualification-v1"),
    )
    parser.add_argument(
        "--access-log",
        type=Path,
        default=Path("artifacts/development/riotbox-1470/access-log-2026-08-26-a.json"),
    )
    args = parser.parse_args()
    require(args.validate_source_blind != args.run, "select exactly one mode")
    repo = Path(__file__).resolve().parent.parent
    if args.validate_source_blind:
        validate_source_blind(repo)
    else:
        output = args.output if args.output.is_absolute() else repo / args.output
        access_log = args.access_log if args.access_log.is_absolute() else repo / args.access_log
        run_qualification(repo, output, access_log)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
