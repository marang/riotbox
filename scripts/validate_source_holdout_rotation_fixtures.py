#!/usr/bin/env python3
"""Mutation fixtures for the P023 source holdout-rotation contract."""

from __future__ import annotations

import argparse
import copy
import hashlib
import inspect
import json
import os
import stat
import struct
import sys
import tempfile
import uuid
import wave
from array import array
from pathlib import Path
from typing import Any, Callable

from source_holdout_development_access import (
    LEGACY_SOURCE_FORMAT,
    STAGE_A_REGISTRY_RAW_SHA256,
    V2_SOURCE_FORMATS,
    PinnedStageARegistry,
    SourceIdentity,
    create_exclusive_access_log,
    load_pinned_stage_a_registry,
    maximum_source_file_bytes,
    run_development_access_session,
)
from validate_source_holdout_rotation import (
    SCHEMA_V2,
    STAGE_A_DEVELOPMENT_CASE_IDS,
    STAGE_A_REGISTRY_PATH,
    V2_PREDECESSOR_PATH,
    V2_PREDECESSOR_SHA256,
    load_canonical_predecessor_snapshot,
    read_json_object,
    repo_root,
    require,
    validate_manifest,
    validate_stage_a_development_request,
    validate_v2_transition,
    verify_development_source_files,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    try:
        manifest = read_json_object(args.manifest)
        validate_manifest(
            manifest,
            args.manifest,
            require_existing_source_files=False,
        )
        run_duplicate_key_fixture()
        run_mutation_fixtures(manifest, args.manifest)
        if manifest.get("schema") == SCHEMA_V2:
            run_v2_transition_fixtures(manifest, args.manifest)
    except (OSError, TypeError, ValueError, wave.Error) as error:
        print(f"invalid source holdout-rotation fixtures: {error}", file=sys.stderr)
        return 1
    print(f"valid source holdout-rotation mutation fixtures: {args.manifest}")
    return 0


def run_duplicate_key_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-source-holdout-json-") as temp:
        path = Path(temp) / "duplicate-key.json"
        path.write_text('{"schema":"first","schema":"second"}')
        try:
            read_json_object(path)
        except ValueError as error:
            require(
                "duplicate JSON object key: 'schema'" in str(error),
                f"duplicate-key fixture failed for the wrong reason: {error}",
            )
            return
    raise ValueError("mutation fixture unexpectedly passed: duplicate JSON key")


def run_mutation_fixtures(manifest: dict[str, Any], manifest_path: Path) -> None:
    holdout_a = list(manifest["holdout_sets"][0]["source_case_ids"])

    def collapse_families(mutated: dict[str, Any]) -> None:
        mutated["candidate_matrix"]["source_case_ids"] = [
            "oga_cinameng_can_be_so_beautiful",
            "oga_ruok_160bpm",
            "oga_illin_robotic",
            "oga_marwan_cinematic_percussion",
            "oga_bart_getequipped",
            "oga_bretbernhoft_beatloops",
        ]

    expect_failure(
        "candidate family collapse",
        manifest,
        manifest_path,
        collapse_families,
        "candidate matrix family collapse",
    )

    def insufficient_holdout(mutated: dict[str, Any]) -> None:
        mutated["holdout_sets"][0]["source_case_ids"] = holdout_a[:1]

    expect_failure(
        "insufficient holdouts",
        manifest,
        manifest_path,
        insufficient_holdout,
        "insufficient holdout sources",
    )

    def reuse_holdout(mutated: dict[str, Any]) -> None:
        mutated["candidate_matrix"]["source_case_ids"].append(holdout_a[0])

    expect_failure(
        "reused holdout",
        manifest,
        manifest_path,
        reuse_holdout,
        "development and holdout sources must be disjoint",
    )

    def consumed_still_unseen(mutated: dict[str, Any]) -> None:
        mutated["rotation_history"].append(
            {
                "case_id": holdout_a[0],
                "former_holdout_id": "holdout_a",
                "consumed_by_ticket": "RIOTBOX-1422",
                "consumed_on": "2026-07-24",
                "replacement_case_id": holdout_a[1],
            }
        )

    expect_failure(
        "consumed holdout remains unseen",
        manifest,
        manifest_path,
        consumed_still_unseen,
        "consumed holdout cannot remain unseen/reserve",
    )

    def unsafe_path(mutated: dict[str, Any]) -> None:
        mutated["entries"][0]["source_path"] = "../.download-examples/The Prodigy.wav"

    expect_failure(
        "unsafe source path",
        manifest,
        manifest_path,
        unsafe_path,
        "source_path must be safe and repo-relative",
    )

    def reference_leak(mutated: dict[str, Any]) -> None:
        mutated["entries"][0]["commercial_reference"] = True

    expect_failure(
        "commercial reference leakage",
        manifest,
        manifest_path,
        reference_leak,
        "commercial/reference-only material is forbidden",
    )

    def missing_review_provenance(mutated: dict[str, Any]) -> None:
        del mutated["entries"][0]["reviewed_on"]

    expect_failure(
        "missing development review provenance",
        manifest,
        manifest_path,
        missing_review_provenance,
        "reviewed_on must be non-empty string",
    )

    def forged_rotation_history(mutated: dict[str, Any]) -> None:
        consumed_id = holdout_a[0]
        mutated["holdout_sets"][0]["source_case_ids"].remove(consumed_id)
        for entry in mutated["entries"]:
            if entry["case_id"] == consumed_id:
                entry.update(
                    {
                        "partition": "retired",
                        "classification_status": "consumed_holdout",
                        "source_suitability_verdict": "weak",
                        "reviewed_on": "2026-07-24",
                        "reviewer_role": "project_musician",
                        "family_verdict_owner": "human_review",
                    }
                )
        mutated["rotation_history"].append(
            {
                "case_id": consumed_id,
                "former_holdout_id": "holdout_a",
                "consumed_by_ticket": "RIOTBOX-1422",
                "consumed_on": "2026-07-24",
                "replacement_case_id": holdout_a[1],
            }
        )

    expect_failure(
        "forged rotation history",
        manifest,
        manifest_path,
        forged_rotation_history,
        "consumed source must record previous_partition",
    )

    def source_pack_collapse(mutated: dict[str, Any]) -> None:
        for entry in mutated["entries"]:
            if entry["corpus_eligible"]:
                entry["source_pack_id"] = "one_narrow_pack"

    expect_failure(
        "narrow source-pack collapse",
        manifest,
        manifest_path,
        source_pack_collapse,
        "eligible source-pack count collapsed",
    )
    if manifest.get("schema") != SCHEMA_V2:
        run_missing_file_fixture(manifest, manifest_path)


def run_v2_transition_fixtures(
    manifest: dict[str, Any], manifest_path: Path
) -> None:
    def admitted_entry(mutated: dict[str, Any], case_id: str) -> dict[str, Any]:
        return next(entry for entry in mutated["entries"] if entry["case_id"] == case_id)

    def mutate_holdout_tuple(mutated: dict[str, Any]) -> None:
        holdout_id = mutated["holdout_sets"][0]["source_case_ids"][0]
        entry = next(entry for entry in mutated["entries"] if entry["case_id"] == holdout_id)
        entry["sha256"] = "f" * 64

    expect_failure(
        "active holdout tuple mutation",
        manifest,
        manifest_path,
        mutate_holdout_tuple,
        "active holdout ID/path/SHA-256 tuple changed",
    )

    def mutate_inherited_entry(mutated: dict[str, Any]) -> None:
        entry = next(
            entry
            for entry in mutated["entries"]
            if entry["case_id"] == "oga_cinameng_can_be_so_beautiful"
        )
        entry["title"] = "mutated inherited title"

    expect_failure(
        "inherited entry mutation",
        manifest,
        manifest_path,
        mutate_inherited_entry,
        "inherited entry changed",
    )

    def wrong_predecessor(mutated: dict[str, Any]) -> None:
        mutated["predecessor"]["sha256"] = "0" * 64

    expect_failure(
        "wrong predecessor",
        manifest,
        manifest_path,
        wrong_predecessor,
        "wrong predecessor SHA-256",
    )

    def embed_forged_predecessor_object(mutated: dict[str, Any]) -> None:
        mutated["predecessor"]["manifest"] = {
            "schema": "riotbox.source_holdout_rotation.v1",
            "entries": [],
        }

    expect_failure(
        "embedded forged predecessor object",
        manifest,
        manifest_path,
        embed_forged_predecessor_object,
        "predecessor must contain only path, schema, and sha256",
    )

    def omit_electronic_family(mutated: dict[str, Any]) -> None:
        mutated["required_core_families"].remove("electronic_drums")

    expect_failure(
        "electronic family omission",
        manifest,
        manifest_path,
        omit_electronic_family,
        "required_core_families must be exactly",
    )

    def add_case_outside_allowlist(mutated: dict[str, Any]) -> None:
        template = copy.deepcopy(
            next(
                entry
                for entry in mutated["entries"]
                if entry["case_id"] == "oga_william_hector_horde_war_drums"
            )
        )
        template.update(
            {
                "case_id": "oga_unexpected_stage_a_source",
                "author": "fixture author",
                "title": "Unexpected fixture source",
                "source_pack_id": "oga_unexpected_stage_a_pack",
                "source_path": (
                    "data/test_audio/external/RIOTBOX-1423/wav/"
                    "dense_oga_unexpected_stage_a_source.wav"
                ),
                "sha256": "1" * 64,
            }
        )
        del template["source_format"]
        del template["source_derivation"]
        del template["source_suitability_verdict_owner"]
        template["family_verdict_owner"] = "human_review"
        mutated["entries"].append(template)
        mutated["candidate_matrix"]["source_case_ids"].append(template["case_id"])

    expect_failure(
        "case outside admission allowlist",
        manifest,
        manifest_path,
        add_case_outside_allowlist,
        "exactly the two allowed admissions",
    )

    def wrong_source_suitability_owner(mutated: dict[str, Any]) -> None:
        admitted_entry(
            mutated,
            "oga_william_hector_horde_war_drums",
        )["source_suitability_verdict_owner"] = "technical_review"

    expect_failure(
        "technical owner claims human source suitability",
        manifest,
        manifest_path,
        wrong_source_suitability_owner,
        "source_suitability_verdict_owner must be human_review",
    )

    def wrong_family_owner(mutated: dict[str, Any]) -> None:
        admitted_entry(
            mutated,
            "oga_frosty_ham_osdrums",
        )["family_verdict_owner"] = "human_review"

    expect_failure(
        "human owner claims technical family taxonomy",
        manifest,
        manifest_path,
        wrong_family_owner,
        "family_verdict_owner must be technical_review",
    )

    def mutate_derivation(
        case_id: str,
        field: str,
        value: str,
    ) -> Callable[[dict[str, Any]], None]:
        def mutate(mutated: dict[str, Any]) -> None:
            admitted_entry(mutated, case_id)["source_derivation"][field] = value

        return mutate

    derivation_mutations = (
        (
            "original URL",
            "oga_frosty_ham_osdrums",
            "original_url",
            "https://opengameart.org/sites/default/files/another-source.ogg",
            "source_derivation does not match the frozen v2 provenance",
        ),
        (
            "original SHA-256",
            "oga_frosty_ham_osdrums",
            "original_sha256",
            "1" * 64,
            "source_derivation does not match the frozen v2 provenance",
        ),
        (
            "derived SHA-256",
            "oga_frosty_ham_osdrums",
            "derived_sha256",
            "2" * 64,
            "source_derivation.derived_sha256 must match the registered source",
        ),
        (
            "decoded format",
            "oga_frosty_ham_osdrums",
            "decoded_format",
            "pcm_f32le_wav",
            "source_derivation does not match the frozen v2 provenance",
        ),
        (
            "decoder",
            "oga_william_hector_horde_war_drums",
            "decoder",
            "ffmpeg",
            "source_derivation does not match the frozen v2 provenance",
        ),
        (
            "sample-rate policy",
            "oga_frosty_ham_osdrums",
            "sample_rate_policy",
            "resample_48000",
            "source_derivation.sample_rate_policy must preserve_original",
        ),
    )
    for name, case_id, field, value, expected_fragment in derivation_mutations:
        expect_failure(
            f"frozen derivation {name} mutation",
            manifest,
            manifest_path,
            mutate_derivation(case_id, field, value),
            expected_fragment,
        )

    run_predecessor_binding_fixture(manifest)
    run_development_access_fixtures(manifest, manifest_path)


def run_predecessor_binding_fixture(manifest: dict[str, Any]) -> None:
    _, predecessor_path, actual_sha256 = load_canonical_predecessor_snapshot()
    require(
        predecessor_path == V2_PREDECESSOR_PATH,
        "predecessor loader must return the canonical repo-relative path",
    )
    require(
        actual_sha256 == V2_PREDECESSOR_SHA256,
        "predecessor loader must hash the actual canonical predecessor bytes",
    )
    require(
        hashlib.sha256((repo_root() / predecessor_path).read_bytes()).hexdigest()
        == actual_sha256,
        "predecessor loader hash must match independently read canonical bytes",
    )
    for validator in (validate_manifest, validate_v2_transition):
        parameters = inspect.signature(validator).parameters
        require(
            "predecessor_manifest" not in parameters
            and "predecessor_manifest_path" not in parameters
            and "predecessor_manifest_sha256" not in parameters,
            f"{validator.__name__} must not accept injectable predecessor context",
        )

    forged_predecessor = {
        "schema": "riotbox.source_holdout_rotation.v1",
        "entries": [],
    }
    try:
        validate_v2_transition(  # type: ignore[call-arg]
            manifest,
            "forged predecessor fixture",
            forged_predecessor,
        )
    except TypeError as error:
        require(
            "positional argument" in str(error),
            f"forged predecessor fixture failed for the wrong reason: {error}",
        )
        return
    raise ValueError("forged predecessor object was accepted by imported validation")


def run_development_access_fixtures(
    manifest: dict[str, Any],
    manifest_path: Path,
) -> None:
    registry, parsed_manifest = load_pinned_stage_a_registry(manifest_path, manifest)
    require(
        parsed_manifest == manifest
        and registry.raw_sha256 == STAGE_A_REGISTRY_RAW_SHA256,
        "development access fixtures require the exact pinned registry bytes",
    )
    run_stage_a_entry_contract_fixtures(manifest, registry)
    run_development_access_success_fixture(registry)
    run_crash_snapshot_blocking_fixtures(registry)
    run_parent_directory_fsync_fixture()
    run_qualification_owner_failure_fixture(registry)
    run_registry_pin_fixtures(manifest, registry)
    run_identity_collision_fixtures(registry)
    run_hash_and_format_rejection_fixtures(registry)
    run_strict_riff_rejection_fixtures(registry)
    run_pre_read_size_rejection_fixture(registry)
    run_containment_rejection_fixtures(registry)


def run_development_access_success_fixture(registry: PinnedStageARegistry) -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-source-access-success-") as temp:
        temp_repo = Path(temp)
        identities: list[SourceIdentity] = []
        expected_opened: list[Path] = []
        for seed, case_id in enumerate(V2_SOURCE_FORMATS, start=11):
            source_format = V2_SOURCE_FORMATS[case_id]
            relative_path = Path("sources") / f"{case_id}.wav"
            absolute_path = temp_repo / relative_path
            absolute_path.parent.mkdir(parents=True, exist_ok=True)
            write_fixture_wav(
                absolute_path,
                seed,
                sample_rate_hz=source_format["sample_rate_hz"],
                sample_width_bits=source_format["sample_width_bits"],
            )
            expected_hash = hashlib.sha256(absolute_path.read_bytes()).hexdigest()
            identities.append(
                SourceIdentity(
                    case_id=case_id,
                    source_path=relative_path.as_posix(),
                    expected_sha256=expected_hash,
                    partition="development",
                    source_format=dict(source_format),
                )
            )
            expected_opened.append(absolute_path)
        identities.append(synthetic_holdout_identity())

        opened: list[Path] = []
        callback_transitions: list[str] = []
        requested = [
            identity.case_id
            for identity in identities
            if identity.partition == "development"
        ]
        access_log_path = temp_repo / "access-success.json"

        def record_open(path: Path) -> None:
            case_id = requested[len(opened)]
            durable = json.loads(access_log_path.read_text())
            require(
                durable["access_preflight_status"] == "passed",
                "preflight must be durable before a source opens",
            )
            records = durable["opened_development_files"]
            require(
                records[-1]["case_id"] == case_id
                and records[-1]["access_verification_status"] == "opened",
                "opened state must be durable before source bytes are read",
            )
            require(
                all(
                    record["access_verification_status"]
                    == "verified_and_delivered_to_owner"
                    for record in records[:-1]
                ),
                "each prior source must be delivered before the next opens",
            )
            opened.append(path)
            callback_transitions.append(f"opened:{case_id}")

        def qualification_owner(
            identity: SourceIdentity,
            payload: bytes,
            access_record: dict[str, Any],
        ) -> None:
            durable = json.loads(access_log_path.read_text())
            require(
                durable["opened_development_files"][-1]["access_verification_status"]
                == "opened",
                "owner must run after durable open and before durable delivery",
            )
            require(
                access_record["access_verification_status"] == "verified"
                and access_record["actual_sha256"] == hashlib.sha256(payload).hexdigest(),
                "owner must receive the exact verified payload and access record",
            )
            require(
                access_record["actual_sha256"] == identity.expected_sha256,
                "owner payload must match the selected identity",
            )
            callback_transitions.append(f"delivered:{identity.case_id}")

        result = run_development_access_session(
            identities,
            requested,
            repo=temp_repo,
            registry=registry,
            access_log_path=access_log_path,
            qualification_owner_id="synthetic-fixture-owner",
            qualification_owner=qualification_owner,
            on_file_open=record_open,
        )
        recorded = json.loads(access_log_path.read_text())
        require(result == recorded, "successful access result and written log must match")
        require(recorded["access_status"] == "completed", "access must complete")
        require(
            recorded["session_kind"] == "DevelopmentSourceAccessSession",
            "generic helper must mint only a development-access session",
        )
        require(
            "StageAQualificationSession" not in json.dumps(recorded),
            "generic subset access must never mint a StageAQualificationSession",
        )
        uuid.UUID(recorded["access_session_id"])
        require(
            recorded["started_at_utc"].endswith("Z"),
            "development access must record a UTC start",
        )
        require(
            recorded["registry_sha256"] == STAGE_A_REGISTRY_RAW_SHA256,
            "development access must bind the frozen raw registry SHA-256",
        )
        require(
            recorded["qualification_status"] == "not_evaluated_by_access_layer"
            and recorded["qualification_owner"]["delivery_status"] == "completed"
            and recorded["qualification_owner"]["delivered_source_count"]
            == len(requested),
            "access completion must remain separate from qualification status",
        )
        require(
            recorded["directory_discovery_performed"] is False,
            "development access must record that directory discovery was not performed",
        )
        require(
            recorded["holdout_metadata_comparison"]["audio_files_opened"] is False,
            "successful development access must not open holdout audio",
        )
        require(opened == expected_opened, "only exact requested development files may open")
        require(
            callback_transitions
            == [
                transition
                for case_id in requested
                for transition in (f"opened:{case_id}", f"delivered:{case_id}")
            ],
            "opened and owner-delivery callbacks must remain strictly ordered",
        )
        require(
            "sample_bytes" not in json.dumps(recorded),
            "verified source payload bytes must never be serialized in the access log",
        )
        records = recorded["opened_development_files"]
        require(len(records) == len(expected_opened), "every opened development file must log")
        development_identities = [
            identity for identity in identities if identity.partition == "development"
        ]
        for identity, record in zip(development_identities, records, strict=True):
            require(
                record["expected_sha256"] == identity.expected_sha256,
                "access log must name the registered expected SHA-256",
            )
            require(
                record["actual_sha256"] == identity.expected_sha256,
                "access log must record the verified actual SHA-256",
            )
            require(
                record["access_verification_status"]
                == "verified_and_delivered_to_owner",
                "each verified payload must be delivered to the in-process owner",
            )
            actual_format = record["actual_source_format"]
            require(
                actual_format["sample_rate_hz"] == identity.source_format["sample_rate_hz"]
                and actual_format["channels"] == identity.source_format["channels"]
                and actual_format["sample_width_bits"]
                == identity.source_format["sample_width_bits"]
                and actual_format["compression_type"]
                == identity.source_format["compression_type"],
                "access log must record the verified actual source format",
            )


def run_stage_a_entry_contract_fixtures(
    manifest: dict[str, Any],
    registry: PinnedStageARegistry,
) -> None:
    require(
        registry.path == STAGE_A_REGISTRY_PATH,
        "Stage-A entry fixture requires the canonical registry path",
    )
    validate_stage_a_development_request(
        list(STAGE_A_DEVELOPMENT_CASE_IDS),
        "exact Stage-A request fixture",
    )
    try:
        validate_stage_a_development_request(
            [STAGE_A_DEVELOPMENT_CASE_IDS[0]],
            "partial Stage-A request fixture",
        )
    except ValueError as error:
        require(
            "exactly the four frozen development cases" in str(error),
            f"partial Stage-A fixture failed for the wrong reason: {error}",
        )
    else:
        raise ValueError("partial Stage-A request unexpectedly passed")

    for function in (run_development_access_session, verify_development_source_files):
        parameters = inspect.signature(function).parameters
        for parameter_name in ("qualification_owner_id", "qualification_owner"):
            require(
                parameters[parameter_name].default is inspect.Parameter.empty,
                f"{function.__name__}.{parameter_name} must be mandatory",
            )

    with tempfile.TemporaryDirectory(prefix="riotbox-stage-a-partial-denial-") as temp:
        temp_repo = Path(temp)
        access_log_path = temp_repo / "must-not-exist.json"
        try:
            verify_development_source_files(
                manifest,
                STAGE_A_REGISTRY_PATH,
                [STAGE_A_DEVELOPMENT_CASE_IDS[0]],
                access_log_path,
                qualification_owner_id="synthetic-stage-a-owner",
                qualification_owner=synthetic_qualification_owner,
                repo=temp_repo,
            )
        except ValueError as error:
            require(
                "exactly the four frozen development cases" in str(error),
                f"partial Stage-A route failed for the wrong reason: {error}",
            )
            require(
                not access_log_path.exists(),
                "partial Stage-A route must reject before minting an access log",
            )
        else:
            raise ValueError("partial Stage-A route unexpectedly opened")

        try:
            run_development_access_session(
                [
                    synthetic_development_identity(
                        source_path="sources/not-opened.wav",
                        expected_sha256="1" * 64,
                    )
                ],
                ["synthetic_development"],
                repo=temp_repo,
                registry=registry,
                access_log_path=access_log_path,
                qualification_owner_id="synthetic-stage-a-owner",
                qualification_owner=None,  # type: ignore[arg-type]
            )
        except ValueError as error:
            require(
                "in-process qualification owner" in str(error),
                f"missing owner fixture failed for the wrong reason: {error}",
            )
            require(
                not access_log_path.exists(),
                "missing owner must reject before minting an access log",
            )
            return
    raise ValueError("missing in-process qualification owner unexpectedly passed")


def run_crash_snapshot_blocking_fixtures(registry: PinnedStageARegistry) -> None:
    snapshots = (
        ("empty_after_create", b""),
        ("truncated_in_place", b'{"access_status":"preflight'),
        ("missing_required_fields", b"{}\n"),
    )
    for name, snapshot in snapshots:
        with tempfile.TemporaryDirectory(prefix="riotbox-source-crash-log-") as temp:
            temp_repo = Path(temp)
            access_log_path = temp_repo / f"{name}.json"
            access_log_path.write_bytes(snapshot)
            opened: list[Path] = []
            try:
                run_development_access_session(
                    [
                        synthetic_development_identity(
                            source_path="sources/not-opened.wav",
                            expected_sha256="1" * 64,
                        )
                    ],
                    ["synthetic_development"],
                    repo=temp_repo,
                    registry=registry,
                    access_log_path=access_log_path,
                    qualification_owner_id="synthetic-crash-owner",
                    qualification_owner=synthetic_qualification_owner,
                    on_file_open=opened.append,
                )
            except FileExistsError:
                require(
                    access_log_path.read_bytes() == snapshot,
                    f"{name}: crash snapshot must remain untouched",
                )
                require(
                    not opened,
                    f"{name}: crash snapshot must block before source open",
                )
                continue
            raise ValueError(f"crash snapshot unexpectedly passed: {name}")


def run_parent_directory_fsync_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-source-log-fsync-") as temp:
        access_log_path = Path(temp) / "exclusive.json"
        original_fsync = os.fsync
        fsynced_modes: list[int] = []

        def record_fsync(file_descriptor: int) -> None:
            fsynced_modes.append(os.fstat(file_descriptor).st_mode)
            original_fsync(file_descriptor)

        access_log_file = None
        os.fsync = record_fsync
        try:
            access_log_file = create_exclusive_access_log(access_log_path)
        finally:
            os.fsync = original_fsync
            if access_log_file is not None:
                access_log_file.close()
        require(
            any(stat.S_ISDIR(mode) for mode in fsynced_modes),
            "exclusive access-log creation must fsync its parent directory",
        )
        require(
            access_log_path.exists() and access_log_path.read_bytes() == b"",
            "durable empty post-create snapshot documents fail-closed crash state",
        )


def run_qualification_owner_failure_fixture(registry: PinnedStageARegistry) -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-source-callback-denial-") as temp:
        temp_repo = Path(temp)
        relative_path = Path("sources/development.wav")
        absolute_path = temp_repo / relative_path
        absolute_path.parent.mkdir(parents=True, exist_ok=True)
        write_fixture_wav(absolute_path, 23)
        expected_hash = hashlib.sha256(absolute_path.read_bytes()).hexdigest()
        callback_count = 0

        def reject_owner(
            identity: SourceIdentity,
            payload: bytes,
            access_record: dict[str, Any],
        ) -> None:
            nonlocal callback_count
            callback_count += 1
            require(
                identity.expected_sha256 == hashlib.sha256(payload).hexdigest()
                == access_record["actual_sha256"],
                "owner rejection fixture must receive verified bytes",
            )
            raise RuntimeError("synthetic qualification owner rejection")

        access_log_path = temp_repo / "owner-rejected.json"
        owner_id = "fixture-stage-a-owner"
        try:
            run_development_access_session(
                [
                    synthetic_development_identity(
                        source_path=relative_path.as_posix(),
                        expected_sha256=expected_hash,
                    )
                ],
                ["synthetic_development"],
                repo=temp_repo,
                registry=registry,
                access_log_path=access_log_path,
                qualification_owner_id=owner_id,
                qualification_owner=reject_owner,
            )
        except RuntimeError as error:
            require(
                str(error) == "synthetic qualification owner rejection",
                f"owner rejection fixture failed for the wrong reason: {error}",
            )
            recorded = json.loads(access_log_path.read_text())
            require(callback_count == 1, "qualification owner must run exactly once")
            require(
                recorded["access_status"] == "aborted"
                and recorded["qualification_status"]
                == "not_evaluated_by_access_layer",
                "owner failure must not become access or qualification success",
            )
            require(
                recorded["rejection_type"] == "RuntimeError"
                and recorded["qualification_owner"]["owner_id"] == owner_id
                and recorded["qualification_owner"]["delivery_status"] == "failed",
                "owner rejection must preserve type and owner identity",
            )
            require(
                recorded["opened_development_files"][0]["access_verification_status"]
                == "opened",
                "owner failure must not falsely persist source delivery",
            )
            require(
                "sample_bytes" not in json.dumps(recorded),
                "owner failure must never serialize source payload bytes",
            )
            return
    raise ValueError("mutation fixture unexpectedly passed: qualification owner failure")


def run_registry_pin_fixtures(
    manifest: dict[str, Any],
    registry: PinnedStageARegistry,
) -> None:
    caller_mismatch = copy.deepcopy(manifest)
    caller_mismatch["owner_ticket"] = "RIOTBOX-9999"
    try:
        load_pinned_stage_a_registry(registry.path, caller_mismatch)
    except ValueError as error:
        require(
            "caller manifest does not match" in str(error),
            f"caller-manifest mismatch failed for the wrong reason: {error}",
        )
    else:
        raise ValueError("mismatched caller manifest unexpectedly matched frozen bytes")

    with tempfile.TemporaryDirectory(prefix="riotbox-registry-raw-pin-") as temp:
        temp_root = Path(temp)
        raw_registry = registry.path.read_bytes()
        mutated_path = temp_root / "mutated-registry.json"
        mutated_path.write_bytes(raw_registry + b"\n")
        try:
            load_pinned_stage_a_registry(mutated_path, manifest)
        except ValueError as error:
            require(
                "raw SHA-256 does not match the frozen pin" in str(error),
                f"registry raw-pin fixture failed for the wrong reason: {error}",
            )
        else:
            raise ValueError("mutated registry raw bytes unexpectedly matched the pin")

        changing_path = temp_root / "changing-registry.json"
        changing_path.write_bytes(raw_registry)
        changing_registry, _ = load_pinned_stage_a_registry(changing_path, manifest)
        changing_path.write_bytes(raw_registry + b" ")
        access_log_path = temp_root / "registry-changed-access.json"
        opened: list[Path] = []
        try:
            run_development_access_session(
                [
                    synthetic_development_identity(
                        source_path="sources/not-opened.wav",
                        expected_sha256="1" * 64,
                    )
                ],
                ["synthetic_development"],
                repo=temp_root,
                registry=changing_registry,
                access_log_path=access_log_path,
                qualification_owner_id="synthetic-registry-owner",
                qualification_owner=synthetic_qualification_owner,
                on_file_open=opened.append,
            )
        except ValueError as error:
            require(
                "registry changed before development source open" in str(error),
                f"pre-open registry pin fixture failed for the wrong reason: {error}",
            )
            recorded = json.loads(access_log_path.read_text())
            require(
                recorded["access_status"] == "rejected"
                and recorded["rejection_stage"].startswith("registry_pin_pre_open:")
                and not opened,
                "registry mutation must reject immediately before any source open",
            )
            return
    raise ValueError("pre-open registry mutation unexpectedly opened a source")


def run_identity_collision_fixtures(registry: PinnedStageARegistry) -> None:
    holdout = synthetic_holdout_identity()
    cases = (
        (
            "active holdout ID",
            [holdout],
            [holdout.case_id],
            "rejected active holdout case before file open",
        ),
        (
            "active holdout path",
            [
                holdout,
                synthetic_development_identity(
                    source_path=holdout.source_path,
                    expected_sha256="1" * 64,
                ),
            ],
            ["synthetic_development"],
            "rejected active holdout path before file open",
        ),
        (
            "active holdout SHA-256",
            [
                holdout,
                synthetic_development_identity(
                    source_path="sources/development.wav",
                    expected_sha256=holdout.expected_sha256,
                ),
            ],
            ["synthetic_development"],
            "rejected active holdout SHA-256 before file open",
        ),
    )
    for name, identities, requested, expected_fragment in cases:
        with tempfile.TemporaryDirectory(prefix="riotbox-source-identity-denial-") as temp:
            assert_rejected_access(
                name,
                identities,
                requested,
                expected_fragment,
                repo=Path(temp),
                registry=registry,
                expected_open_count=0,
            )


def run_hash_and_format_rejection_fixtures(registry: PinnedStageARegistry) -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-source-content-denial-") as temp:
        temp_repo = Path(temp)
        relative_path = Path("sources/development.wav")
        absolute_path = temp_repo / relative_path
        absolute_path.parent.mkdir(parents=True, exist_ok=True)
        write_fixture_wav(absolute_path, 31)
        actual_hash = hashlib.sha256(absolute_path.read_bytes()).hexdigest()

        assert_rejected_access(
            "development SHA-256 mismatch",
            [
                synthetic_development_identity(
                    source_path=relative_path.as_posix(),
                    expected_sha256="1" * 64,
                )
            ],
            ["synthetic_development"],
            "source SHA-256 mismatch",
            repo=temp_repo,
            registry=registry,
            expected_open_count=1,
        )
        assert_rejected_access(
            "development source-format mismatch",
            [
                synthetic_development_identity(
                    source_path=relative_path.as_posix(),
                    expected_sha256=actual_hash,
                    source_format={**LEGACY_SOURCE_FORMAT, "sample_rate_hz": 44_100},
                )
            ],
            ["synthetic_development"],
            "source WAV sample-rate mismatch",
            repo=temp_repo,
            registry=registry,
            expected_open_count=1,
        )


def run_strict_riff_rejection_fixtures(registry: PinnedStageARegistry) -> None:
    def rifx(payload: bytearray) -> None:
        payload[0:4] = b"RIFX"

    def float_format(payload: bytearray) -> None:
        struct.pack_into("<H", payload, 20, 3)

    def extensible_format(payload: bytearray) -> None:
        struct.pack_into("<H", payload, 20, 0xFFFE)

    def wrong_block_align(payload: bytearray) -> None:
        struct.pack_into("<H", payload, 32, 2)

    def wrong_byte_rate(payload: bytearray) -> None:
        struct.pack_into("<I", payload, 28, 1)

    def unaligned_data(payload: bytearray) -> None:
        data_size = struct.unpack_from("<I", payload, 40)[0]
        require(data_size > 1 and data_size % 2 == 0, "fixture WAV data must be even")
        struct.pack_into("<I", payload, 40, data_size - 1)

    def duplicate_data(payload: bytearray) -> None:
        payload.extend(b"data\x00\x00\x00\x00")
        struct.pack_into("<I", payload, 4, len(payload) - 8)

    mutations: tuple[tuple[str, Callable[[bytearray], None], str], ...] = (
        ("RIFX container", rifx, "RIFF little-endian container"),
        ("IEEE float format", float_format, "format_tag must be 1 PCM"),
        ("WAVE extensible format", extensible_format, "format_tag must be 1 PCM"),
        ("block align", wrong_block_align, "block_align mismatch"),
        ("byte rate", wrong_byte_rate, "byte_rate mismatch"),
        ("unaligned data", unaligned_data, "data chunk is not block-aligned"),
        ("duplicate data", duplicate_data, "must contain one data chunk"),
    )
    for name, mutate, expected_fragment in mutations:
        with tempfile.TemporaryDirectory(prefix="riotbox-source-riff-denial-") as temp:
            temp_repo = Path(temp)
            relative_path = Path("sources/development.wav")
            absolute_path = temp_repo / relative_path
            absolute_path.parent.mkdir(parents=True, exist_ok=True)
            write_fixture_wav(absolute_path, 37)
            payload = bytearray(absolute_path.read_bytes())
            mutate(payload)
            absolute_path.write_bytes(payload)
            actual_hash = hashlib.sha256(payload).hexdigest()
            assert_rejected_access(
                f"strict RIFF {name}",
                [
                    synthetic_development_identity(
                        source_path=relative_path.as_posix(),
                        expected_sha256=actual_hash,
                    )
                ],
                ["synthetic_development"],
                expected_fragment,
                repo=temp_repo,
                registry=registry,
                expected_open_count=1,
            )


def run_pre_read_size_rejection_fixture(registry: PinnedStageARegistry) -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-source-size-denial-") as temp:
        temp_repo = Path(temp)
        relative_path = Path("sources/oversized.wav")
        absolute_path = temp_repo / relative_path
        absolute_path.parent.mkdir(parents=True, exist_ok=True)
        maximum_bytes = maximum_source_file_bytes(LEGACY_SOURCE_FORMAT)
        with absolute_path.open("wb") as oversized:
            oversized.truncate(maximum_bytes + 1)
        assert_rejected_access(
            "pre-read source byte boundary",
            [
                synthetic_development_identity(
                    source_path=relative_path.as_posix(),
                    expected_sha256="1" * 64,
                )
            ],
            ["synthetic_development"],
            "exceeds its pre-read byte boundary",
            repo=temp_repo,
            registry=registry,
            expected_open_count=1,
        )


def run_containment_rejection_fixtures(registry: PinnedStageARegistry) -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-source-ancestor-symlink-") as temp:
        temp_repo = Path(temp)
        real_directory = temp_repo / "real"
        real_directory.mkdir()
        real_file = real_directory / "source.wav"
        write_fixture_wav(real_file, 41)
        (temp_repo / "linked").symlink_to(real_directory, target_is_directory=True)
        assert_rejected_access(
            "repo-relative ancestor symlink",
            [
                synthetic_development_identity(
                    source_path="linked/source.wav",
                    expected_sha256=hashlib.sha256(real_file.read_bytes()).hexdigest(),
                )
            ],
            ["synthetic_development"],
            "selected source ancestor is a symlink",
            repo=temp_repo,
            registry=registry,
            expected_open_count=0,
        )

    with tempfile.TemporaryDirectory(prefix="riotbox-source-final-symlink-") as temp:
        temp_repo = Path(temp)
        source_directory = temp_repo / "sources"
        source_directory.mkdir()
        real_file = source_directory / "real.wav"
        write_fixture_wav(real_file, 43)
        (source_directory / "selected.wav").symlink_to(real_file)
        assert_rejected_access(
            "final source symlink",
            [
                synthetic_development_identity(
                    source_path="sources/selected.wav",
                    expected_sha256=hashlib.sha256(real_file.read_bytes()).hexdigest(),
                )
            ],
            ["synthetic_development"],
            "selected source file is a symlink",
            repo=temp_repo,
            registry=registry,
            expected_open_count=0,
        )

    with tempfile.TemporaryDirectory(prefix="riotbox-source-hardlink-") as temp:
        temp_repo = Path(temp)
        source_directory = temp_repo / "sources"
        source_directory.mkdir()
        selected_file = source_directory / "selected.wav"
        write_fixture_wav(selected_file, 47)
        os.link(selected_file, source_directory / "second-link.wav")
        assert_rejected_access(
            "multi-link source file",
            [
                synthetic_development_identity(
                    source_path="sources/selected.wav",
                    expected_sha256=hashlib.sha256(selected_file.read_bytes()).hexdigest(),
                )
            ],
            ["synthetic_development"],
            "selected source file must have exactly one hard link",
            repo=temp_repo,
            registry=registry,
            expected_open_count=0,
        )


def assert_rejected_access(
    name: str,
    identities: list[SourceIdentity],
    requested_case_ids: list[str],
    expected_fragment: str,
    *,
    repo: Path,
    registry: PinnedStageARegistry,
    expected_open_count: int,
) -> None:
    opened: list[Path] = []
    access_log_path = repo / f"{name.replace(' ', '-')}-access.json"
    try:
        run_development_access_session(
            identities,
            requested_case_ids,
            repo=repo,
            registry=registry,
            access_log_path=access_log_path,
            qualification_owner_id="synthetic-rejection-owner",
            qualification_owner=synthetic_qualification_owner,
            on_file_open=opened.append,
        )
    except (OSError, ValueError, wave.Error) as error:
        require(
            expected_fragment in str(error),
            f"{name} fixture failed for the wrong reason: {error}",
        )
        recorded = json.loads(access_log_path.read_text())
        require(
            recorded["access_status"] == "rejected",
            f"{name} access log must reject",
        )
        require(
            recorded["qualification_status"] == "not_evaluated_by_access_layer",
            f"{name} must not report qualification success",
        )
        require(
            recorded["directory_discovery_performed"] is False,
            f"{name} must record no directory discovery",
        )
        require(
            recorded["holdout_metadata_comparison"]["audio_files_opened"] is False,
            f"{name} must never open holdout audio",
        )
        require(
            len(opened) == expected_open_count,
            f"{name} opened {len(opened)} files; expected {expected_open_count}",
        )
        if expected_open_count == 0:
            require(
                recorded["opened_development_files"] == [],
                f"{name} must reject before recording any file open",
            )
        else:
            require(
                len(recorded["opened_development_files"]) == expected_open_count,
                f"{name} must record every exact development file open",
            )
            require(
                "expected_sha256" in recorded["opened_development_files"][0],
                f"{name} opened-file record must label the expected SHA-256",
            )
        return
    raise ValueError(f"mutation fixture unexpectedly passed: {name}")


def synthetic_holdout_identity() -> SourceIdentity:
    return SourceIdentity(
        case_id="synthetic_holdout",
        source_path="sources/holdout.wav",
        expected_sha256="f" * 64,
        partition="holdout_a",
        source_format=dict(LEGACY_SOURCE_FORMAT),
    )


def synthetic_development_identity(
    *,
    source_path: str,
    expected_sha256: str,
    source_format: dict[str, Any] | None = None,
) -> SourceIdentity:
    return SourceIdentity(
        case_id="synthetic_development",
        source_path=source_path,
        expected_sha256=expected_sha256,
        partition="development",
        source_format=dict(source_format or LEGACY_SOURCE_FORMAT),
    )


def synthetic_qualification_owner(
    identity: SourceIdentity,
    payload: bytes,
    access_record: dict[str, Any],
) -> None:
    require(
        access_record["access_verification_status"] == "verified"
        and access_record["actual_sha256"] == identity.expected_sha256
        == hashlib.sha256(payload).hexdigest(),
        "synthetic qualification owner requires exact verified source delivery",
    )


def expect_failure(
    name: str,
    manifest: dict[str, Any],
    manifest_path: Path,
    mutate: Callable[[dict[str, Any]], None],
    expected_fragment: str,
) -> None:
    mutated = copy.deepcopy(manifest)
    mutate(mutated)
    try:
        validate_manifest(
            mutated,
            Path(f"{manifest_path}:{name}"),
            require_existing_source_files=False,
        )
    except ValueError as error:
        require(
            expected_fragment in str(error),
            f"mutation fixture {name!r} failed for the wrong reason: {error}",
        )
        return
    raise ValueError(f"mutation fixture unexpectedly passed: {name}")


def run_missing_file_fixture(manifest: dict[str, Any], manifest_path: Path) -> None:
    mutated = copy.deepcopy(manifest)
    with tempfile.TemporaryDirectory(prefix="riotbox-source-holdout-") as temp:
        temp_repo = Path(temp)
        for index, entry in enumerate(mutated["entries"]):
            path = temp_repo / entry["source_path"]
            path.parent.mkdir(parents=True, exist_ok=True)
            write_fixture_wav(path, index)
            entry["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        validate_manifest(
            mutated,
            Path(f"{manifest_path}:complete temporary source corpus"),
            require_existing_source_files=True,
            repo=temp_repo,
        )
        missing_path = temp_repo / mutated["entries"][-1]["source_path"]
        missing_path.unlink()
        try:
            validate_manifest(
                mutated,
                Path(f"{manifest_path}:missing source file"),
                require_existing_source_files=True,
                repo=temp_repo,
            )
        except ValueError as error:
            require(
                "missing source file" in str(error),
                f"missing-file fixture failed for the wrong reason: {error}",
            )
            return
    raise ValueError("mutation fixture unexpectedly passed: missing source file")


def write_fixture_wav(
    path: Path,
    seed: int,
    *,
    sample_rate_hz: int = 48_000,
    sample_width_bits: int = 16,
) -> None:
    payload: bytes
    if sample_width_bits == 16:
        samples = array("h")
        for frame in range(480):
            value = ((frame * (seed + 3)) % 1000) - 500
            samples.extend((value, -value))
        payload = samples.tobytes()
    elif sample_width_bits == 24:
        samples_24 = bytearray()
        for frame in range(480):
            value = ((frame * (seed + 3)) % 100_000) - 50_000
            samples_24.extend(value.to_bytes(3, "little", signed=True))
            samples_24.extend((-value).to_bytes(3, "little", signed=True))
        payload = bytes(samples_24)
    else:
        raise ValueError(f"unsupported synthetic sample width: {sample_width_bits}")
    with wave.open(str(path), "wb") as target:
        target.setnchannels(2)
        target.setsampwidth(sample_width_bits // 8)
        target.setframerate(sample_rate_hz)
        target.writeframes(payload)


if __name__ == "__main__":
    raise SystemExit(main())
