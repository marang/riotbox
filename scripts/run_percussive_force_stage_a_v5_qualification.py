#!/usr/bin/env python3
"""Run the single frozen RIOTBOX-1430 Stage-A-v5 qualification session."""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import os
import sys
import uuid
from pathlib import Path
from typing import Any, Sequence

import numpy as np

import percussive_force_stage_a_analysis as stage_a
import run_percussive_force_stage_a_qualification as shared
import run_percussive_force_stage_a_v2_qualification as v2
from source_holdout_development_access import (
    SourceIdentity,
    maximum_source_file_bytes,
    parse_strict_pcm_wave,
    read_contained_regular_file,
)


ROOT = Path(__file__).resolve().parents[1]
PROTOCOL_V2 = Path("docs/benchmarks/percussive_force_stage_a_protocol_v2.json")
PROTOCOL_V5 = Path("docs/benchmarks/percussive_force_stage_a_protocol_v5.json")
SOURCE_SET = Path("docs/benchmarks/percussive_force_stage_a_bound_source_set_v1.json")
PROTOCOL_V2_SHA = "b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878"
PROTOCOL_V5_SHA = "455440aabc1a433bbc7fbcc2093b85f6d1c66e1bba081526e082c50ed8248519"
SOURCE_SET_SHA = "7ec185a51233d83c49d8227b0e81acb2ca83c24bc31783a9343dc71d090e47a6"
SESSION_SCHEMA = "riotbox.percussive_force_stage_a_qualification_session.v5"
CATALOG_SCHEMA = "riotbox.percussive_force_stage_a_bound_event_catalog.v5"
REJECTION_SCHEMA = "riotbox.percussive_force_stage_a_qualification_rejection.v5"
COMMIT_SCHEMA = "riotbox.percussive_force_stage_a_qualification_commit.v2"
ACCESS_SCHEMA = "riotbox.percussive_force_stage_a_qualification_access.v1"
IMPLEMENTATION_PATHS = (
    Path("scripts/run_percussive_force_stage_a_v5_qualification.py"),
    Path("scripts/run_percussive_force_stage_a_v2_qualification.py"),
    Path("scripts/run_percussive_force_stage_a_qualification.py"),
    Path("scripts/percussive_force_stage_a_analysis.py"),
    Path("scripts/source_holdout_development_access.py"),
    PROTOCOL_V2,
    PROTOCOL_V5,
    SOURCE_SET,
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise shared.QualificationSessionError(message)


def load_contracts() -> tuple[stage_a.FrozenStageAProtocol, tuple[shared.FrozenSourceSpec, ...], dict[str, Any]]:
    protocol_v5, _ = shared.read_pinned_json(PROTOCOL_V5, PROTOCOL_V5_SHA)
    source_set, _ = shared.read_pinned_json(SOURCE_SET, SOURCE_SET_SHA)
    require(
        protocol_v5.get("schema") == "riotbox.percussive_force_stage_a_protocol.v5"
        and protocol_v5.get("unchanged_contracts", {}).get("algorithm_source_raw_sha256")
        == PROTOCOL_V2_SHA,
        "Protocol-v5 algorithm binding changed",
    )
    require(
        source_set.get("schema") == "riotbox.percussive_force_stage_a_bound_source_set.v1"
        and source_set.get("status")
        == "frozen_before_pcm_iteration_or_source_feature_computation"
        and source_set.get("protocol", {}).get("raw_sha256") == PROTOCOL_V5_SHA,
        "bound source-set identity changed",
    )
    entries = source_set.get("entries")
    order = source_set.get("qualification_order")
    require(isinstance(entries, list) and len(entries) == 13, "bound source set must contain 13 entries")
    require(
        order == [1, 2, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15]
        and [entry.get("ordinal") for entry in entries] == order,
        "bound source qualification order changed",
    )
    specs: list[shared.FrozenSourceSpec] = []
    for entry in entries:
        require(isinstance(entry, dict), "bound source entry must be an object")
        source_format = entry.get("source_format")
        require(isinstance(source_format, dict), "bound source format missing")
        path = Path(str(entry.get("source_path", "")))
        require(
            not path.is_absolute()
            and path.parts[:5]
            == ("data", "test_audio", "external", "RIOTBOX-1430", "freesound-v3-pool")
            and all(part not in {"", ".", ".."} for part in path.parts),
            f"unsafe bound source path: {path}",
        )
        specs.append(
            shared.FrozenSourceSpec(
                case_id=str(entry["case_id"]),
                source_family=str(entry["source_family"]),
                author=str(entry["author"]),
                source_path=path.as_posix(),
                source_sha256=str(entry["sha256"]),
                license=str(entry["license"]),
                partition=str(entry["partition"]),
                source_format=dict(source_format),
            )
        )
    require(len({spec.case_id for spec in specs}) == 13, "duplicate bound source case ID")
    require(len({spec.author.casefold() for spec in specs}) == 13, "duplicate bound source author")
    require(
        {spec.source_family for spec in specs}
        == {"dense_break", "sparse_drums", "electronic_drums"},
        "bound source families changed",
    )
    protocol = stage_a.load_frozen_protocol(PROTOCOL_V2)
    require(protocol.sha256 == PROTOCOL_V2_SHA, "loaded Protocol-v2 pin changed")
    return protocol, tuple(specs), source_set


def implementation_snapshot() -> dict[str, Any]:
    files = [
        {"path": path.as_posix(), "sha256": hashlib.sha256((ROOT / path).read_bytes()).hexdigest()}
        for path in IMPLEMENTATION_PATHS
    ]
    encoded = json.dumps(files, separators=(",", ":"), sort_keys=True).encode()
    return {
        "schema": "riotbox.percussive_force_stage_a_implementation_snapshot.v5",
        "aggregate_sha256": hashlib.sha256(encoded).hexdigest(),
        "files": files,
    }


def access_sources(
    specs: Sequence[shared.FrozenSourceSpec],
    access_path: Path,
) -> tuple[list[stage_a.SourceInput], list[dict[str, Any]], str]:
    snapshot = shared.ExclusiveJsonSnapshot(access_path)
    log: dict[str, Any] = {
        "schema": ACCESS_SCHEMA,
        "owner_ticket": "RIOTBOX-1430",
        "scope": "development_only_exact_bound_source_set",
        "status": "started",
        "source_set_sha256": SOURCE_SET_SHA,
        "requested_case_ids": [spec.case_id for spec in specs],
        "records": [],
        "holdout_audio_accessed": False,
        "commercial_reference_accessed": False,
        "source_directory_discovery_performed": False,
        "candidate_render_started": False,
    }
    snapshot.persist(log)
    source_inputs: list[stage_a.SourceInput] = []
    bindings: list[dict[str, Any]] = []
    try:
        for ordinal, spec in enumerate(specs, start=1):
            record: dict[str, Any] = {
                "access_ordinal": ordinal,
                "case_id": spec.case_id,
                "source_path": spec.source_path,
                "expected_sha256": spec.source_sha256,
                "status": "opening_exact_registered_path",
            }
            log["records"].append(record)
            snapshot.persist(log)
            payload = read_contained_regular_file(
                ROOT,
                Path(spec.source_path),
                f"StageAQualificationSession-v5:{spec.case_id}",
                maximum_bytes=maximum_source_file_bytes(spec.source_format),
            )
            actual_sha = hashlib.sha256(payload).hexdigest()
            require(actual_sha == spec.source_sha256, f"source SHA changed: {spec.case_id}")
            parsed = parse_strict_pcm_wave(
                payload,
                spec.source_format,
                f"StageAQualificationSession-v5:{spec.case_id}",
            )
            parsed.pop("sample_bytes")
            record.update(
                status="verified_and_delivered_to_qualification_owner",
                actual_bytes=len(payload),
                actual_sha256=actual_sha,
                actual_source_format=parsed,
            )
            snapshot.persist(log)
            identity = SourceIdentity(
                case_id=spec.case_id,
                source_path=spec.source_path,
                expected_sha256=spec.source_sha256,
                partition="development",
                source_format=spec.source_format,
            )
            captured = shared.CapturedSource(identity=identity, payload=payload, access_record=record)
            source_input, binding = v2.decode_captured_source(captured, spec)
            source_inputs.append(source_input)
            bindings.append(binding)
        log["status"] = "completed"
        log["completed_at_utc"] = shared.utc_now()
        snapshot.persist(log)
    except Exception:
        log["status"] = "rejected_fail_closed"
        log["completed_at_utc"] = shared.utc_now()
        snapshot.persist(log)
        raise
    finally:
        snapshot.close()
    payload = access_path.read_bytes()
    require(json.loads(payload)["status"] == "completed", "qualification access did not complete")
    return source_inputs, bindings, hashlib.sha256(payload).hexdigest()


def contrast_passes(
    analyses: Sequence[stage_a.SourceAnalysis], protocol: stage_a.FrozenStageAProtocol
) -> bool:
    pairs = []
    by_pair = {}
    for left, right in itertools.combinations(analyses, 2):
        pair = stage_a._pair_contrast(left, right, protocol)
        pairs.append(pair)
        by_pair[frozenset((pair.left_case_id, pair.right_case_id))] = pair.classification
    ids = tuple(item.metadata.case_id for item in analyses)
    valid = []
    for partition in stage_a._set_partitions(ids):
        cluster_for = {
            case_id: cluster_index
            for cluster_index, cluster in enumerate(partition)
            for case_id in cluster
        }
        accepted = True
        for left_id, right_id in itertools.combinations(ids, 2):
            same = cluster_for[left_id] == cluster_for[right_id]
            classification = by_pair[frozenset((left_id, right_id))]
            if (same and classification is not stage_a.PairClassification.SIMILAR) or (
                not same and classification is not stage_a.PairClassification.DISTINCT
            ):
                accepted = False
                break
        if accepted:
            valid.append(partition)
    return len(valid) == int(protocol.value("valid_source_partition_count")) and all(
        len(partition) >= int(protocol.value("minimum_source_clusters"))
        for partition in valid
    )


def select_sources(
    inputs: Sequence[stage_a.SourceInput],
    analyses: Sequence[stage_a.SourceAnalysis],
    protocol: stage_a.FrozenStageAProtocol,
) -> tuple[tuple[int, ...] | None, stage_a.StageAQualification | None]:
    qualified = [index for index, analysis in enumerate(analyses) if analysis.qualified]
    for indices in itertools.combinations(qualified, 4):
        selected_analyses = tuple(analyses[index] for index in indices)
        if len({item.metadata.author.casefold() for item in selected_analyses}) != 4:
            continue
        if len({item.metadata.source_family for item in selected_analyses}) != 3:
            continue
        if not contrast_passes(selected_analyses, protocol):
            continue
        selected_inputs = tuple(inputs[index] for index in indices)
        qualification = stage_a.qualify_four_sources(selected_inputs, protocol=protocol)
        require(qualification.passed, "selected combination diverged on frozen revalidation")
        require(
            [source.to_dict() for source in qualification.sources]
            == [analysis.to_dict() for analysis in selected_analyses],
            "selected source reanalysis diverged",
        )
        return tuple(indices), qualification
    return None, None


def run_session(session_dir: Path) -> tuple[int, dict[str, Any]]:
    shared.validate_session_directory(session_dir)
    protocol, specs, _ = load_contracts()
    session_id = str(uuid.uuid4())
    session_path = session_dir / "session.json"
    access_path = session_dir / "development-access-log.json"
    artifact_path = session_dir / "event-catalog.json"
    rejection_path = session_dir / "qualification-rejection.json"
    commit_path = session_dir / "qualification-commit.json"
    snapshot = shared.ExclusiveJsonSnapshot(session_path)
    session: dict[str, Any] = {
        "schema": SESSION_SCHEMA,
        "session_id": session_id,
        "owner_ticket": "RIOTBOX-1430",
        "scope": "development_only_thirteen_bound_sources_no_holdout_or_reference_access",
        "started_at_utc": shared.utc_now(),
        "status": "contract_validation",
        "contracts": {
            "algorithm_protocol_sha256": PROTOCOL_V2_SHA,
            "execution_protocol_sha256": PROTOCOL_V5_SHA,
            "bound_source_set_sha256": SOURCE_SET_SHA,
        },
        "candidate_render_started": False,
        "holdout_audio_accessed": False,
        "commercial_reference_accessed": False,
        "source_directory_discovery_performed": False,
        "quality_proof": False,
        "hardness_proof": False,
        "human_verdict": "unverified",
        "runtime_environment": {
            "python_version": sys.version,
            "numpy_version": np.__version__,
            "byteorder": sys.byteorder,
        },
    }
    snapshot.persist(session)
    try:
        implementation = implementation_snapshot()
        session["implementation_snapshot"] = implementation
        session["status"] = "development_source_access"
        snapshot.persist(session)
        inputs, bindings, access_sha = access_sources(specs, access_path)
        require(implementation_snapshot() == implementation, "implementation changed during access")
        session["access_evidence"] = {
            "path": access_path.as_posix(),
            "sha256": access_sha,
            "source_count": len(inputs),
        }
        session["status"] = "source_analysis"
        snapshot.persist(session)
        analyses = tuple(
            stage_a.analyze_source(
                source.metadata,
                source.samples,
                source.sample_rate_hz,
                source.input_lsb,
                protocol=protocol,
                per_channel_dc_mean_f64_bits_be_hex=source.per_channel_dc_mean_f64_bits_be_hex,
            )
            for source in inputs
        )
        require(implementation_snapshot() == implementation, "implementation changed during analysis")
        indices, qualification = select_sources(inputs, analyses, protocol)
        selected = [] if indices is None else [specs[index].case_id for index in indices]
        selected_ordinals = [] if indices is None else [int(json.loads((ROOT / SOURCE_SET).read_text())["entries"][index]["ordinal"]) for index in indices]
        passed = qualification is not None
        artifact = {
            "schema": CATALOG_SCHEMA if passed else REJECTION_SCHEMA,
            "owner_ticket": "RIOTBOX-1430",
            "session_id": session_id,
            "qualification_state": "passed" if passed else "rejected",
            "stage_a_qualification_passed": passed,
            "contracts": session["contracts"],
            "implementation_snapshot": implementation,
            "access_evidence": session["access_evidence"],
            "source_bindings": bindings,
            "all_source_analyses": [analysis.to_dict() for analysis in analyses],
            "qualified_source_count": sum(analysis.qualified for analysis in analyses),
            "selected_case_ids": selected,
            "selected_pool_ordinals": selected_ordinals,
            "selected_qualification": None if qualification is None else qualification.to_dict(),
            "candidate_render_started": False,
            "holdout_audio_accessed": False,
            "commercial_reference_accessed": False,
            "source_directory_discovery_performed": False,
            "quality_proof": False,
            "hardness_proof": False,
            "human_verdict": "unverified",
            "next_allowed_action": (
                "bind_and_execute_unchanged_3_family_by_4_source_by_2_event_matrix_v6"
                if passed
                else "stop_without_candidate_render_or_retuning"
            ),
        }
        output_path = artifact_path if passed else rejection_path
        artifact_sha = shared.create_exclusive_json(output_path, artifact)
        session["status"] = "qualified_event_catalog_frozen" if passed else "rejected_fail_closed"
        session["qualification_artifact"] = {
            "path": output_path.as_posix(),
            "sha256": artifact_sha,
            "passed": passed,
        }
        session["completed_at_utc"] = shared.utc_now()
        snapshot.persist(session)
        session_bytes = session_path.read_bytes()
        commit = {
            "schema": COMMIT_SCHEMA,
            "session_id": session_id,
            "session_path": session_path.as_posix(),
            "session_sha256": hashlib.sha256(session_bytes).hexdigest(),
            "session_status": session["status"],
            "qualification_artifact_path": output_path.as_posix(),
            "qualification_artifact_sha256": artifact_sha,
            "stage_a_qualification_passed": passed,
            "access_log_sha256": access_sha,
            "committed_at_utc": shared.utc_now(),
        }
        shared.create_exclusive_json(commit_path, commit)
        return (0 if passed else 2), session
    except Exception as error:
        session["status"] = "rejected_contract_or_execution_failure"
        session["failure_type"] = type(error).__name__
        session["failure"] = str(error)[:2000]
        session["candidate_render_started"] = False
        session["completed_at_utc"] = shared.utc_now()
        snapshot.persist(session)
        raise
    finally:
        snapshot.close()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--session-dir", type=Path)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    os.chdir(ROOT)
    if args.validate_only:
        try:
            load_contracts()
        except Exception as error:
            print(f"FAIL: {error}", file=sys.stderr)
            return 1
        print("PASS: Stage-A-v5 qualification contracts; source_audio_accessed=false")
        return 0
    if args.session_dir is None:
        parser.error("--session-dir is required unless --validate-only is used")
    try:
        exit_code, session = run_session(args.session_dir)
    except Exception as error:
        print(f"FAIL: Stage-A-v5 qualification stopped fail-closed: {error}", file=sys.stderr)
        return 1
    artifact = session["qualification_artifact"]
    print("PASS" if artifact["passed"] else "REJECTED", "Stage-A-v5 qualification")
    print(f"session_id={session['session_id']}")
    print(f"artifact={artifact['path']}")
    print(f"artifact_sha256={artifact['sha256']}")
    print("candidate_render_started=false")
    print("human_verdict=unverified")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
