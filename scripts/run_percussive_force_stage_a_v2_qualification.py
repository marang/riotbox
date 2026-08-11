#!/usr/bin/env python3
"""Run the single frozen RIOTBOX-1430 development-only Stage-A-v2 gate.

The runner validates Protocol v2, Matrix v3, and Registry v3 before opening
exactly the four registered development WAV paths. It never enumerates a
source directory, opens holdout audio, renders a candidate, or plays audio.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import struct
import sys
import uuid
from fractions import Fraction
from pathlib import Path
from typing import Any

import numpy as np

import percussive_force_stage_a_analysis as stage_a
import run_percussive_force_stage_a_qualification as shared
from source_holdout_development_access import (
    PinnedStageARegistry,
    SourceIdentity,
    parse_strict_pcm_wave,
    run_development_access_session,
    validate_source_format,
)
from validate_percussive_force_stage_a_protocol_v2 import (
    validate_repository as validate_protocol_v2_repository,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
PROTOCOL_PATH = Path("docs/benchmarks/percussive_force_stage_a_protocol_v2.json")
MATRIX_PATH = Path("docs/benchmarks/percussive_force_development_matrix_v3.json")
REGISTRY_PATH = Path("docs/benchmarks/source_holdout_rotation_v3.json")
EXPECTED_PROTOCOL_SHA256 = (
    "b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878"
)
EXPECTED_MATRIX_SHA256 = (
    "0dff59b8d871f75eccd75a5df1ff8080c777f4b76b3559957ce415762b16aa5e"
)
EXPECTED_REGISTRY_SHA256 = (
    "9e5e03ad64319061a4baaa6cee7c40fc5e993171b0d11003ec29767f273bc502"
)
POSITIVE_CASE_IDS = (
    "oga_cinameng_can_be_so_beautiful",
    "freesound_djericmark_724939",
    "freesound_cyclez_493560",
    "freesound_justabeat_458897",
)
SESSION_SCHEMA = "riotbox.percussive_force_stage_a_qualification_session.v2"
CATALOG_SCHEMA = "riotbox.percussive_force_stage_a_bound_event_catalog.v2"
REJECTION_SCHEMA = "riotbox.percussive_force_stage_a_qualification_rejection.v2"
COMMIT_SCHEMA = "riotbox.percussive_force_stage_a_qualification_commit.v1"
PCM_F32LE_HASH_DOMAIN = "riotbox.percussive_force_pcm_f32le.v1"
IMPLEMENTATION_PATHS = (
    Path("scripts/run_percussive_force_stage_a_v2_qualification.py"),
    Path("scripts/run_percussive_force_stage_a_qualification.py"),
    Path("scripts/percussive_force_stage_a_analysis.py"),
    Path("scripts/source_holdout_development_access.py"),
    Path("scripts/validate_percussive_force_stage_a_protocol_v2.py"),
    Path("scripts/percussive_force_stage_a_v2_contract.py"),
    PROTOCOL_PATH,
    MATRIX_PATH,
    REGISTRY_PATH,
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise shared.QualificationSessionError(message)


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def source_format_for_entry(
    registry: dict[str, Any], entry: dict[str, Any], case_id: str
) -> dict[str, Any]:
    raw = entry.get("source_format", registry.get("source_format_default"))
    require(isinstance(raw, dict), f"missing source format for {case_id}")
    result = dict(raw)
    validate_source_format(result, f"{REGISTRY_PATH}:{case_id}.source_format")
    return result


def _safe_relative_source_path(value: Any, case_id: str) -> str:
    require(isinstance(value, str) and value, f"missing source path for {case_id}")
    path = Path(value)
    require(
        not path.is_absolute()
        and path.parts
        and all(part not in {"", ".", ".."} for part in path.parts),
        f"unsafe source path for {case_id}",
    )
    return path.as_posix()


def validate_contracts(
    protocol: dict[str, Any], matrix: dict[str, Any], registry: dict[str, Any]
) -> tuple[tuple[shared.FrozenSourceSpec, ...], tuple[SourceIdentity, ...]]:
    require(
        protocol.get("schema") == "riotbox.percussive_force_stage_a_protocol.v2"
        and protocol.get("schema_version") == 2
        and protocol.get("owner_ticket") == "RIOTBOX-1430",
        "Protocol-v2 identity changed",
    )
    require(
        matrix.get("schema") == "riotbox.percussive_force_development_matrix.v3"
        and matrix.get("schema_version") == 3
        and matrix.get("owner_ticket") == "RIOTBOX-1428",
        "Matrix-v3 identity changed",
    )
    require(
        registry.get("schema") == "riotbox.source_holdout_rotation.v3"
        and registry.get("schema_version") == 3
        and registry.get("owner_ticket") == "RIOTBOX-1430",
        "Registry-v3 identity changed",
    )
    require(
        matrix.get("predecessor")
        == {
            "path": "docs/benchmarks/percussive_force_development_matrix_v2.json",
            "schema": "riotbox.percussive_force_development_matrix.v2",
            "sha256": "aba846138246c95b1c3e5e1973e77bdaa41ce971f799dadadba8edc160967fd6",
        },
        "Matrix-v3 predecessor pin changed",
    )
    require(
        registry.get("predecessor")
        == {
            "path": "docs/benchmarks/source_holdout_rotation_v2.json",
            "schema": "riotbox.source_holdout_rotation.v2",
            "sha256": "af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812",
        },
        "Registry-v3 predecessor pin changed",
    )
    require(
        matrix.get("protocol")
        == {
            "path": PROTOCOL_PATH.as_posix(),
            "schema": "riotbox.percussive_force_stage_a_protocol.v2",
            "sha256": EXPECTED_PROTOCOL_SHA256,
            "state": "frozen_before_source_qualification_or_candidate_render",
        },
        "Matrix-v3 Protocol-v2 binding changed",
    )
    source_registry = matrix.get("source_registry")
    require(isinstance(source_registry, dict), "Matrix-v3 source registry binding missing")
    require(
        source_registry.get("path") == REGISTRY_PATH.as_posix()
        and source_registry.get("schema") == "riotbox.source_holdout_rotation.v3"
        and source_registry.get("sha256") == EXPECTED_REGISTRY_SHA256,
        "Matrix-v3 Registry-v3 binding changed",
    )
    require(
        matrix.get("quality_proof") is False
        and matrix.get("product_path_proof") is False,
        "preregistered Matrix-v3 must not claim quality or product proof",
    )

    positive_set = registry.get("stage_a_positive_source_set")
    require(isinstance(positive_set, dict), "Registry-v3 positive source set missing")
    require(
        tuple(positive_set.get("case_ids", ())) == POSITIVE_CASE_IDS
        and positive_set.get("exact_source_count") == 4
        and positive_set.get("exact_author_count") == 4
        and positive_set.get("exact_family_count") == 3
        and positive_set.get("event_qualification_state") == "not_started"
        and positive_set.get("candidate_render_state") == "not_started",
        "Registry-v3 positive source set changed",
    )
    matrix_set = matrix.get("positive_source_set")
    require(
        isinstance(matrix_set, dict)
        and matrix_set.get("exact_source_count") == 4
        and matrix_set.get("exact_author_count") == 4
        and matrix_set.get("exact_family_count") == 3
        and matrix_set.get("event_qualification_state") == "not_started",
        "Matrix-v3 positive source cardinality changed",
    )

    entries = registry.get("entries")
    rows = matrix.get("positive_sources")
    require(isinstance(entries, list), "Registry-v3 entries must be an array")
    require(isinstance(rows, list), "Matrix-v3 positive_sources must be an array")
    require(
        tuple(row.get("case_id") for row in rows if isinstance(row, dict))
        == POSITIVE_CASE_IDS,
        "Matrix-v3 positive source identity or order changed",
    )
    by_id = {
        str(entry.get("case_id")): entry
        for entry in entries
        if isinstance(entry, dict)
    }
    require(len(by_id) == len(entries), "Registry-v3 contains duplicate case IDs")

    specs: list[shared.FrozenSourceSpec] = []
    for row in rows:
        require(isinstance(row, dict), "Matrix-v3 positive source must be an object")
        case_id = str(row["case_id"])
        entry = by_id.get(case_id)
        require(isinstance(entry, dict), f"Registry-v3 entry missing for {case_id}")
        for field in (
            "case_id",
            "source_family",
            "author",
            "source_path",
            "sha256",
            "license",
            "partition",
        ):
            require(
                row.get(field) == entry.get(field),
                f"Matrix-v3/Registry-v3 {field} mismatch for {case_id}",
            )
        require(entry.get("partition") == "development", f"non-development source: {case_id}")
        require(
            row.get("event_qualification") == "not_started"
            and row.get("candidate_render") == "not_started",
            f"Matrix-v3 source already carries result state: {case_id}",
        )
        source_format = source_format_for_entry(registry, entry, case_id)
        matrix_format = row.get("source_format")
        if matrix_format is not None:
            require(
                isinstance(matrix_format, dict)
                and all(source_format.get(key) == value for key, value in matrix_format.items()),
                f"Matrix-v3 source format mismatch for {case_id}",
            )
        specs.append(
            shared.FrozenSourceSpec(
                case_id=case_id,
                source_family=str(row["source_family"]),
                author=str(row["author"]),
                source_path=_safe_relative_source_path(row["source_path"], case_id),
                source_sha256=str(row["sha256"]),
                license=str(row["license"]),
                partition="development",
                source_format=source_format,
            )
        )
    require(
        len({spec.author.casefold() for spec in specs}) == 4
        and len({spec.source_family for spec in specs}) == 3,
        "positive source author/family cardinality changed",
    )

    components = protocol.get("component_versions")
    require(isinstance(components, dict), "Protocol-v2 component versions missing")
    event_catalog = matrix.get("event_catalog")
    cross_product = matrix.get("required_cross_product")
    require(
        isinstance(event_catalog, dict)
        and event_catalog.get("detector_and_anatomy")
        == "riotbox.percussive_force_prequalification.v3"
        and event_catalog.get("development_ordinals") == [1, 2]
        and event_catalog.get("minimum_events_per_source") == 2
        and event_catalog.get("maximum_frozen_events_per_source") == 3,
        "Matrix-v3 event-catalog contract changed",
    )
    require(
        isinstance(cross_product, dict)
        and cross_product.get("families") == ["F1", "F2", "F3"]
        and cross_product.get("family_versions")
        == [components["f1"], components["f2"], components["f3"]]
        and cross_product.get("source_count") == 4
        and cross_product.get("event_ordinals") == [1, 2]
        and cross_product.get("candidate_event_condition_count") == 24
        and cross_product.get("execution") == "not_started",
        "Matrix-v3 frozen 3x4x2 cross-product changed",
    )

    registry_holdouts = {
        (
            str(entry["case_id"]),
            str(entry["partition"]),
            str(entry["source_path"]),
            str(entry["sha256"]),
        )
        for entry in entries
        if isinstance(entry, dict) and str(entry.get("partition", "")).startswith("holdout_")
    }
    matrix_holdouts = {
        (
            str(entry["case_id"]),
            str(entry["partition"]),
            str(entry["source_path"]),
            str(entry["sha256"]),
        )
        for entry in matrix.get("active_holdout_union", ())
        if isinstance(entry, dict)
    }
    require(
        len(registry_holdouts) == 9 and matrix_holdouts == registry_holdouts,
        "Matrix-v3 active holdout metadata union changed",
    )
    protection = matrix.get("holdout_protection")
    require(
        isinstance(protection, dict)
        and protection.get("glob_or_directory_discovery") is False
        and protection.get("read_audio") is False
        and protection.get("hash_audio") is False
        and protection.get("play_audio") is False
        and protection.get("render_audio") is False,
        "Matrix-v3 holdout protection changed",
    )

    identities: list[SourceIdentity] = []
    for entry in entries:
        require(isinstance(entry, dict), "Registry-v3 entry must be an object")
        case_id = str(entry["case_id"])
        identities.append(
            SourceIdentity(
                case_id=case_id,
                source_path=_safe_relative_source_path(entry["source_path"], case_id),
                expected_sha256=str(entry["sha256"]),
                partition=str(entry["partition"]),
                source_format=source_format_for_entry(registry, entry, case_id),
            )
        )
    return tuple(specs), tuple(identities)


def authoritative_dc_mean_bits(
    integer_codes: np.ndarray, valid_bits: int
) -> tuple[tuple[str, ...], tuple[int, ...]]:
    codes = np.asarray(integer_codes)
    require(codes.ndim == 2 and codes.shape[0] > 0, "integer PCM must be frames x channels")
    require(valid_bits in {16, 24}, "authoritative mean supports PCM16 or PCM24 only")
    minimum = -(1 << (valid_bits - 1))
    maximum = (1 << (valid_bits - 1)) - 1
    sums = [0] * int(codes.shape[1])
    i128_min = -(1 << 127)
    i128_max = (1 << 127) - 1
    for frame in range(int(codes.shape[0])):
        for channel in range(int(codes.shape[1])):
            code = int(codes[frame, channel])
            require(minimum <= code <= maximum, "decoded PCM code exceeds valid_bits")
            updated = sums[channel] + code
            require(i128_min <= updated <= i128_max, "authoritative PCM code sum overflowed i128")
            sums[channel] = updated
    denominator = int(codes.shape[0]) * (1 << (valid_bits - 1))
    encoded: list[str] = []
    for total in sums:
        mean = 0.0 if total == 0 else float(Fraction(total, denominator))
        require(math.isfinite(mean), "authoritative source mean became nonfinite")
        encoded.append(struct.pack(">d", mean).hex())
    return tuple(encoded), tuple(sums)


def pcm_f32le_sha256(samples: np.ndarray, sample_rate_hz: int, channels: int) -> str:
    digest = hashlib.sha256()
    domain = PCM_F32LE_HASH_DOMAIN.encode("utf-8")
    digest.update(struct.pack("<I", len(domain)))
    digest.update(domain)
    digest.update(struct.pack("<I", sample_rate_hz))
    digest.update(struct.pack("<I", channels))
    digest.update(struct.pack("<Q", int(samples.shape[0])))
    digest.update(np.asarray(samples, dtype="<f4").tobytes(order="C"))
    return digest.hexdigest()


def decode_captured_source(
    captured: shared.CapturedSource, spec: shared.FrozenSourceSpec
) -> tuple[stage_a.SourceInput, dict[str, Any]]:
    parsed = parse_strict_pcm_wave(
        captured.payload,
        spec.source_format,
        f"StageAQualificationSession-v2:{spec.case_id}",
    )
    sample_bytes = parsed.pop("sample_bytes")
    bits = int(parsed["sample_width_bits"])
    channels = int(parsed["channels"])
    if bits == 16:
        flat_codes = np.frombuffer(sample_bytes, dtype="<i2").astype(np.int32)
        encoding = stage_a.PcmEncoding.PCM_S16LE
    elif bits == 24:
        octets = np.frombuffer(sample_bytes, dtype=np.uint8).reshape(-1, 3)
        unsigned = (
            octets[:, 0].astype(np.int32)
            | (octets[:, 1].astype(np.int32) << 8)
            | (octets[:, 2].astype(np.int32) << 16)
        )
        flat_codes = np.where(
            unsigned & 0x80_0000, unsigned - 0x100_0000, unsigned
        ).astype(np.int32)
        encoding = stage_a.PcmEncoding.PCM_S24LE
    else:
        raise shared.QualificationSessionError(f"unsupported verified PCM width: {bits}")
    require(flat_codes.size % channels == 0, f"PCM alignment changed for {spec.case_id}")
    integer_codes = flat_codes.reshape(-1, channels)
    mean_bits, exact_sums = authoritative_dc_mean_bits(integer_codes, bits)
    divisor = float(1 << (bits - 1))
    samples = integer_codes.astype(np.float64) / divisor
    input_lsb = math.ldexp(1.0, -(bits - 1))
    verified_format = stage_a.VerifiedPcmFormat(
        encoding=encoding,
        sample_rate_hz=int(parsed["sample_rate_hz"]),
        channel_count=channels,
        format_tag=int(parsed["format_tag"]),
        container_bits=bits,
        valid_bits=bits,
        block_align=int(parsed["block_align"]),
        compression_type=str(parsed["compression_type"]),
        input_lsb=input_lsb,
    )
    metadata = stage_a.SourceMetadata(
        case_id=spec.case_id,
        source_family=spec.source_family,
        author=spec.author,
        source_path=spec.source_path,
        source_sha256=spec.source_sha256,
        license=spec.license,
        verified_format=verified_format,
        partition=spec.partition,
    )
    source_input = stage_a.SourceInput(
        metadata=metadata,
        samples=samples,
        sample_rate_hz=verified_format.sample_rate_hz,
        input_lsb=input_lsb,
        per_channel_dc_mean_f64_bits_be_hex=mean_bits,
    )
    binding = {
        "case_id": spec.case_id,
        "source_path": spec.source_path,
        "source_sha256": spec.source_sha256,
        "pcm_encoding": encoding.value,
        "valid_bits": bits,
        "sample_rate_hz": verified_format.sample_rate_hz,
        "channel_count": channels,
        "frame_count": int(samples.shape[0]),
        "per_channel_dc_mean_f64_bits_be_hex": list(mean_bits),
        "exact_signed_code_sums": list(exact_sums),
        "pcm_f32le_sha256": pcm_f32le_sha256(
            samples, verified_format.sample_rate_hz, channels
        ),
        "verified_format": verified_format.to_dict(),
        "access_record": captured.access_record,
    }
    return source_input, binding


def implementation_snapshot() -> dict[str, Any]:
    files = [
        {
            "path": path.as_posix(),
            "sha256": sha256_bytes((REPO_ROOT / path).read_bytes()),
        }
        for path in IMPLEMENTATION_PATHS
    ]
    encoded = json.dumps(files, separators=(",", ":"), sort_keys=True).encode()
    return {
        "schema": "riotbox.percussive_force_stage_a_implementation_snapshot.v2",
        "aggregate_sha256": sha256_bytes(encoded),
        "files": files,
    }


def run_session(session_directory: Path) -> tuple[int, dict[str, Any]]:
    shared.validate_session_directory(session_directory)
    session_id = str(uuid.uuid4())
    owner_id = f"riotbox-1430-stage-a-v2-{session_id}"
    session_path = session_directory / "session.json"
    access_log_path = session_directory / "development-access-log.json"
    commit_path = session_directory / "qualification-commit.json"
    snapshot = shared.ExclusiveJsonSnapshot(session_path)
    session: dict[str, Any] = {
        "schema": SESSION_SCHEMA,
        "session_id": session_id,
        "session_kind": "StageAQualificationSession",
        "owner_ticket": "RIOTBOX-1430",
        "scope": "development_only_exact_four_sources_no_holdout_or_reference_access",
        "started_at_utc": shared.utc_now(),
        "status": "source_blind_contract_validation",
        "qualification_owner_id": owner_id,
        "contracts": {
            "protocol_path": PROTOCOL_PATH.as_posix(),
            "protocol_sha256": EXPECTED_PROTOCOL_SHA256,
            "matrix_path": MATRIX_PATH.as_posix(),
            "matrix_sha256": EXPECTED_MATRIX_SHA256,
            "registry_path": REGISTRY_PATH.as_posix(),
            "registry_sha256": EXPECTED_REGISTRY_SHA256,
        },
        "holdout_audio_accessed": False,
        "commercial_reference_accessed": False,
        "source_directory_discovery_performed": False,
        "candidate_render_started": False,
        "human_verdict": "unverified",
        "quality_proof": False,
        "hardness_proof": False,
        "runtime_environment": {
            "python_version": sys.version,
            "numpy_version": np.__version__,
            "byteorder": sys.byteorder,
        },
    }
    snapshot.persist(session)
    try:
        protocol_json, _ = shared.read_pinned_json(
            PROTOCOL_PATH, EXPECTED_PROTOCOL_SHA256
        )
        matrix, _ = shared.read_pinned_json(MATRIX_PATH, EXPECTED_MATRIX_SHA256)
        registry, _ = shared.read_pinned_json(REGISTRY_PATH, EXPECTED_REGISTRY_SHA256)
        validated_protocol = validate_protocol_v2_repository(REPO_ROOT)
        require(
            validated_protocol.raw_sha256 == EXPECTED_PROTOCOL_SHA256,
            "Protocol-v2 validator returned an unexpected pin",
        )
        protocol = stage_a.load_frozen_protocol(PROTOCOL_PATH)
        specs, identities = validate_contracts(protocol_json, matrix, registry)
        implementation_before = implementation_snapshot()
        session["implementation_snapshot"] = implementation_before
        session["status"] = "development_source_access"
        session["access_log_path"] = access_log_path.as_posix()
        snapshot.persist(session)

        capture_owner = shared.SourceCaptureOwner(specs)
        access_result = run_development_access_session(
            identities,
            list(POSITIVE_CASE_IDS),
            repo=REPO_ROOT,
            registry=PinnedStageARegistry(
                path=REGISTRY_PATH,
                schema="riotbox.source_holdout_rotation.v3",
                raw_sha256=EXPECTED_REGISTRY_SHA256,
            ),
            access_log_path=access_log_path,
            qualification_owner_id=owner_id,
            qualification_owner=capture_owner,
        )
        finalized_records = shared.validate_completed_access_result(
            access_result,
            specs,
            owner_id,
            requested_case_ids=POSITIVE_CASE_IDS,
        )
        require(
            len(capture_owner.captured) == len(specs) == 4,
            "qualification owner did not receive exactly four sources",
        )
        access_bytes, _ = shared.read_and_match_access_log(
            access_log_path, access_result
        )
        access_sha256 = sha256_bytes(access_bytes)
        session["access_evidence"] = {
            "access_session_id": access_result["access_session_id"],
            "access_log_path": access_log_path.as_posix(),
            "access_log_sha256": access_sha256,
            "access_status": access_result["access_status"],
            "owner_delivery_status": access_result["qualification_owner"][
                "delivery_status"
            ],
        }
        session["status"] = "source_feature_computation"
        snapshot.persist(session)

        source_inputs: list[stage_a.SourceInput] = []
        bindings: list[dict[str, Any]] = []
        for captured, spec, record in zip(
            capture_owner.captured, specs, finalized_records, strict=True
        ):
            finalized = shared.CapturedSource(
                identity=captured.identity,
                payload=captured.payload,
                access_record=record,
            )
            source_input, binding = decode_captured_source(finalized, spec)
            require(
                binding["frame_count"] == record["actual_source_format"]["frame_count"],
                f"qualification PCM reparse diverged for {spec.case_id}",
            )
            source_inputs.append(source_input)
            bindings.append(binding)

        require(
            implementation_snapshot() == implementation_before,
            "qualification implementation or contracts changed after source access",
        )
        shared.read_pinned_json(PROTOCOL_PATH, EXPECTED_PROTOCOL_SHA256)
        shared.read_pinned_json(MATRIX_PATH, EXPECTED_MATRIX_SHA256)
        shared.read_pinned_json(REGISTRY_PATH, EXPECTED_REGISTRY_SHA256)
        unbound = stage_a.qualify_four_sources(source_inputs, protocol=protocol)
        require(
            unbound.schema
            == "riotbox.percussive_force_stage_a_unbound_qualification_analysis.v2"
            and unbound.qualification_state == "unbound_analysis_only",
            "analysis bypassed the frozen v2 qualification boundary",
        )
        for source, binding in zip(unbound.sources, bindings, strict=True):
            serialized = source.to_dict()
            require(
                serialized.get("per_channel_dc_mean_f64_bits_be_hex")
                == binding["per_channel_dc_mean_f64_bits_be_hex"]
                and "per_channel_dc_means" not in serialized,
                f"authoritative source mean binding changed for {binding['case_id']}",
            )
        require(
            implementation_snapshot() == implementation_before,
            "qualification implementation or contracts changed during analysis",
        )
        final_access_bytes, _ = shared.read_and_match_access_log(
            access_log_path, access_result
        )
        require(
            sha256_bytes(final_access_bytes) == access_sha256,
            "development access log changed during source analysis",
        )

        artifact = {
            "schema": CATALOG_SCHEMA if unbound.passed else REJECTION_SCHEMA,
            "owner_ticket": "RIOTBOX-1430",
            "session_id": session_id,
            "qualification_owner_id": owner_id,
            "qualification_state": "passed" if unbound.passed else "rejected",
            "stage_a_qualification_passed": unbound.passed,
            "contracts": session["contracts"],
            "implementation_snapshot": implementation_before,
            "access_evidence": session["access_evidence"],
            "source_bindings": bindings,
            "mechanism_blind_analysis": unbound.to_dict(),
            "candidate_render_started": False,
            "holdout_audio_accessed": False,
            "commercial_reference_accessed": False,
            "source_directory_discovery_performed": False,
            "quality_proof": False,
            "hardness_proof": False,
            "human_verdict": "unverified",
            "next_allowed_action": (
                "run_unchanged_3_family_by_4_source_by_2_event_matrix"
                if unbound.passed
                else "stop_without_candidate_render_or_protocol_retuning"
            ),
        }
        artifact_path = session_directory / (
            "event-catalog.json" if unbound.passed else "qualification-rejection.json"
        )
        artifact_sha256 = shared.create_exclusive_json(artifact_path, artifact)
        session["status"] = (
            "qualified_event_catalog_frozen" if unbound.passed else "rejected_fail_closed"
        )
        session["qualification_artifact"] = {
            "path": artifact_path.as_posix(),
            "sha256": artifact_sha256,
            "passed": unbound.passed,
        }
        session["qualification_commit"] = {
            "required_for_downstream": True,
            "schema": COMMIT_SCHEMA,
            "path": commit_path.as_posix(),
        }
        session["completed_at_utc"] = shared.utc_now()
        snapshot.persist(session)
        session_bytes = session_path.read_bytes()
        require(
            json.loads(session_bytes, object_pairs_hook=shared.reject_duplicate_keys)
            == session,
            "final persisted session differs from in-process state",
        )
        shared.create_exclusive_json(
            commit_path,
            {
                "schema": COMMIT_SCHEMA,
                "session_id": session_id,
                "session_path": session_path.as_posix(),
                "session_sha256": sha256_bytes(session_bytes),
                "session_status": session["status"],
                "qualification_artifact_path": artifact_path.as_posix(),
                "qualification_artifact_sha256": artifact_sha256,
                "stage_a_qualification_passed": unbound.passed,
                "access_log_sha256": access_sha256,
                "committed_at_utc": shared.utc_now(),
            },
        )
        return (0 if unbound.passed else 2), session
    except Exception as error:
        session["status"] = "rejected_contract_or_execution_failure"
        session["failure_type"] = type(error).__name__
        session["failure"] = str(error)[:2_000]
        session["candidate_render_started"] = False
        session["completed_at_utc"] = shared.utc_now()
        snapshot.persist(session)
        raise
    finally:
        snapshot.close()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--session-dir",
        required=True,
        type=Path,
        help="fresh absolute directory created for exactly one qualification session",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    os.chdir(REPO_ROOT)
    try:
        exit_code, session = run_session(args.session_dir)
    except Exception as error:
        print(f"FAIL: Stage-A-v2 qualification stopped fail-closed: {error}", file=sys.stderr)
        return 1
    artifact = session["qualification_artifact"]
    print(
        "PASS: frozen development-only Stage-A-v2 qualification"
        if artifact["passed"]
        else "REJECTED: frozen development-only Stage-A-v2 qualification"
    )
    print(f"session_id={session['session_id']}")
    print(f"artifact={artifact['path']}")
    print(f"artifact_sha256={artifact['sha256']}")
    commit_path = Path(session["qualification_commit"]["path"])
    print(f"qualification_commit={commit_path}")
    print(f"qualification_commit_sha256={sha256_bytes(commit_path.read_bytes())}")
    print("candidate_render_started=false")
    print("human_verdict=unverified")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
