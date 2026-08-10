#!/usr/bin/env python3
"""Validate the complete metadata-only RIOTBOX-1430 acquisition batch."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any

import percussive_force_stage_a_v2_acquisition as acquisition
import percussive_force_stage_a_v2_acquisition_contract as batch_contract
import validate_percussive_force_stage_a_protocol_v2 as protocol_v2_validator


EXPECTED_BATCH_RAW_SHA256 = (
    "ada49dc778bebe201c399413122765fce08d4476c445af30c2a1982bd524e6c9"
)
EXPECTED_BATCH_SEMANTIC_SHA256 = (
    "7c103a12b743e8c9406d66008527c0036dac472c78bee5512ee46beb7492b362"
)
SHA256 = re.compile(r"^[0-9a-f]{64}$")
SAFE_ID = re.compile(r"^[a-z0-9][a-z0-9_]*$")
REQUIRED_FAMILIES = ("dense_break", "sparse_drums", "electronic_drums")
COMMERCIAL_MARKERS = {
    ".download-examples",
    "commercial-reference",
    "commercial_reference",
    "the prodigy",
    "tidal",
}
FORBIDDEN_PREEXECUTION_KEYS = {
    "actual",
    "computed",
    "event_records",
    "feature_results",
    "gate_result",
    "human_verdict",
    "measurement",
    "policy_results",
    "qualified",
    "render",
    "result",
    "survivor",
    "verdict",
}


class ContractError(ValueError):
    """Raised when the acquisition batch is not the exact frozen snapshot."""


@dataclass(frozen=True, init=False, slots=True)
class FrozenAcquisitionBatchV1:
    _repo_root: Path
    _payload: bytes

    def __init__(self, *_args: Any, **_kwargs: Any) -> None:
        raise TypeError(
            "FrozenAcquisitionBatchV1 must be created by validate_repository()"
        )

    @classmethod
    def _from_validated(cls, repo_root: Path, payload: bytes) -> "FrozenAcquisitionBatchV1":
        frozen = object.__new__(cls)
        object.__setattr__(frozen, "_repo_root", repo_root.resolve())
        object.__setattr__(frozen, "_payload", bytes(payload))
        return frozen

    @property
    def path(self) -> Path:
        return batch_contract.BATCH_REL

    @property
    def schema(self) -> str:
        return "riotbox.percussive_force_stage_a_v2_acquisition_batch.v1"

    @property
    def raw_sha256(self) -> str:
        return hashlib.sha256(self._payload).hexdigest()

    @property
    def semantic_sha256(self) -> str:
        return batch_contract.semantic_sha256(_parse_json(self._payload, str(self.path)))

    @property
    def document(self) -> dict[str, Any]:
        return _parse_json(self._payload, str(self.path))

    @property
    def repo_root(self) -> Path:
        try:
            root = self._repo_root
        except AttributeError as error:
            raise ContractError("unvalidated acquisition binding has no repository root") from error
        if not isinstance(root, Path):
            raise ContractError("unvalidated acquisition binding has an invalid repository root")
        return root

    def revalidated(self) -> "FrozenAcquisitionBatchV1":
        return validate_repository(self.repo_root)


def revalidate_frozen_batch(value: FrozenAcquisitionBatchV1) -> FrozenAcquisitionBatchV1:
    if type(value) is not FrozenAcquisitionBatchV1:
        raise ContractError(
            "acquisition binding must be the exact FrozenAcquisitionBatchV1 type"
        )
    return FrozenAcquisitionBatchV1.revalidated(value)


def _fail(path: str, message: str) -> None:
    raise ContractError(f"{path}: {message}")


def _strict_equal(actual: Any, expected: Any) -> bool:
    if type(actual) is not type(expected):
        return False
    if isinstance(expected, dict):
        return actual.keys() == expected.keys() and all(
            _strict_equal(actual[key], expected[key]) for key in expected
        )
    if isinstance(expected, list):
        return len(actual) == len(expected) and all(
            _strict_equal(left, right)
            for left, right in zip(actual, expected, strict=True)
        )
    return actual == expected


def _expect(path: str, actual: Any, expected: Any) -> None:
    if not _strict_equal(actual, expected):
        _fail(path, f"expected {expected!r}, got {actual!r}")


def _mapping(path: str, value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        _fail(path, "must be an object")
    return value


def _array(path: str, value: Any) -> list[Any]:
    if not isinstance(value, list):
        _fail(path, "must be an array")
    return value


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, child in pairs:
        if key in value:
            _fail("JSON", f"duplicate object key {key!r}")
        value[key] = child
    return value


def _reject_nonfinite(token: str) -> None:
    _fail("JSON", f"nonfinite number {token!r} is forbidden")


def _parse_json(payload: bytes, path: str) -> dict[str, Any]:
    try:
        decoded = json.loads(
            payload,
            object_pairs_hook=_reject_duplicate_keys,
            parse_constant=_reject_nonfinite,
        )
    except ContractError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        _fail(path, f"invalid UTF-8 JSON: {error}")
    return _mapping(path, decoded)


def _load_named_json(
    repo_root: Path, relative: Path
) -> tuple[dict[str, Any], bytes, str, str]:
    if relative not in {batch_contract.BATCH_REL, batch_contract.REGISTRY_V2_REL}:
        _fail(str(relative), "batch validator may read only its batch and Registry v2")
    path = repo_root / relative
    if path.suffix != ".json" or not path.is_file():
        _fail(str(relative), "named contract is missing or not a regular JSON file")
    payload = path.read_bytes()
    document = _parse_json(payload, str(relative))
    return (
        document,
        payload,
        hashlib.sha256(payload).hexdigest(),
        batch_contract.semantic_sha256(document),
    )


def _first_difference(actual: Any, expected: Any, path: str = "$") -> str | None:
    if type(actual) is not type(expected):
        return f"{path} type changed from {type(expected).__name__} to {type(actual).__name__}"
    if isinstance(expected, dict):
        missing = [key for key in expected if key not in actual]
        extra = [key for key in actual if key not in expected]
        if missing or extra:
            return f"{path} key set changed; missing={missing!r}, extra={extra!r}"
        for key in expected:
            difference = _first_difference(actual[key], expected[key], f"{path}.{key}")
            if difference is not None:
                return difference
        return None
    if isinstance(expected, list):
        if len(actual) != len(expected):
            return f"{path} length changed from {len(expected)} to {len(actual)}"
        for index, (left, right) in enumerate(zip(actual, expected, strict=True)):
            difference = _first_difference(left, right, f"{path}[{index}]")
            if difference is not None:
                return difference
        return None
    if actual != expected:
        return f"{path} changed from {expected!r} to {actual!r}"
    return None


def _ensure_no_preexecution_results(value: Any, path: str = "batch") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if key in FORBIDDEN_PREEXECUTION_KEYS or key.endswith("_result") or key.endswith("_verdict"):
                _fail(f"{path}.{key}", "source-result fields are forbidden in preregistration")
            _ensure_no_preexecution_results(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _ensure_no_preexecution_results(child, f"{path}[{index}]")


def _ensure_no_hash_cycles_or_forward_pins(document: dict[str, Any]) -> None:
    own_hashes = {EXPECTED_BATCH_RAW_SHA256, EXPECTED_BATCH_SEMANTIC_SHA256}

    def visit(value: Any, path: str) -> None:
        if isinstance(value, dict):
            for key, child in value.items():
                visit(child, f"{path}.{key}")
            return
        if isinstance(value, list):
            for index, child in enumerate(value):
                visit(child, f"{path}[{index}]")
            return
        if not isinstance(value, str):
            return
        if value in own_hashes:
            _fail(path, "batch must not contain its own raw or semantic SHA-256")
        lowered = value.casefold()
        if "source_holdout_rotation.v3" in lowered or "development_matrix.v3" in lowered:
            _fail(path, "batch must not bind forward Registry-v3 or Matrix-v3 authority")

    visit(document, "batch")


def _safe_destination(value: Any, prefix: str) -> str:
    if not isinstance(value, str) or not value:
        _fail(prefix, "must be a non-empty path")
    if "\\" in value or "\x00" in value or "//" in value:
        _fail(prefix, "contains a forbidden byte")
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        _fail(prefix, "must be lexically safe and repo-relative")
    expected_parent = PurePosixPath(batch_contract.FINAL_BATCH_DIRECTORY)
    if path.parent != expected_parent or path.suffix.casefold() != ".wav":
        _fail(prefix, "must be a WAV directly inside the one atomic final batch directory")
    lowered = value.casefold()
    if any(marker in lowered for marker in COMMERCIAL_MARKERS):
        _fail(prefix, "contains a commercial/reference marker")
    return value


def _validate_bindings(document: dict[str, Any]) -> None:
    protocol = _mapping("batch.protocol_binding", document.get("protocol_binding"))
    _expect("batch.protocol_binding.path", protocol.get("path"), batch_contract.PROTOCOL_V2_REL.as_posix())
    _expect(
        "batch.protocol_binding.schema",
        protocol.get("schema"),
        "riotbox.percussive_force_stage_a_protocol.v2",
    )
    _expect(
        "batch.protocol_binding.raw_sha256",
        protocol.get("raw_sha256"),
        batch_contract.PROTOCOL_V2_RAW_SHA256,
    )
    _expect(
        "batch.protocol_binding.semantic_sha256",
        protocol.get("semantic_sha256"),
        batch_contract.PROTOCOL_V2_SEMANTIC_SHA256,
    )
    predecessor = _mapping(
        "batch.predecessor_registry_binding",
        document.get("predecessor_registry_binding"),
    )
    _expect(
        "batch.predecessor_registry_binding.path",
        predecessor.get("path"),
        batch_contract.REGISTRY_V2_REL.as_posix(),
    )
    _expect(
        "batch.predecessor_registry_binding.schema",
        predecessor.get("schema"),
        "riotbox.source_holdout_rotation.v2",
    )
    _expect(
        "batch.predecessor_registry_binding.raw_sha256",
        predecessor.get("raw_sha256"),
        batch_contract.REGISTRY_V2_RAW_SHA256,
    )
    _expect(
        "batch.predecessor_registry_binding.semantic_sha256",
        predecessor.get("semantic_sha256"),
        batch_contract.REGISTRY_V2_SEMANTIC_SHA256,
    )
    _expect(
        "batch.predecessor_registry_binding.holdout_union_state",
        predecessor.get("holdout_union_state"),
        "immutable_unopened",
    )


def _validate_entries(
    document: dict[str, Any], registry_v2: dict[str, Any]
) -> None:
    entries = _array("batch.entries", document.get("entries"))
    if len(entries) != 3:
        _fail("batch.entries", "must contain exactly three predeclared sources")
    mapped = [_mapping(f"batch.entries[{index}]", value) for index, value in enumerate(entries)]
    _expect("batch.entries.ordinals", [item.get("ordinal") for item in mapped], [1, 2, 3])
    _expect(
        "batch.entries.source_families",
        [item.get("source_family") for item in mapped],
        list(REQUIRED_FAMILIES),
    )
    authors = [item.get("author") for item in mapped]
    if not all(isinstance(author, str) and author.strip() == author and author for author in authors):
        _fail("batch.entries.author", "authors must be non-empty exact strings")
    folded_authors = [author.casefold() for author in authors]
    if len(set(folded_authors)) != 3 or "cinameng" in folded_authors:
        _fail(
            "batch.entries.author",
            "the three batch authors must be distinct and different from Cinameng",
        )

    prior_entries = _array("registry_v2.entries", registry_v2.get("entries"))
    collision_fields = (
        "case_id",
        "source_pack_id",
        "page_url",
        "download_url",
        "source_path",
    )
    prior_values = {
        field: {item.get(field) for item in prior_entries if isinstance(item, dict)}
        for field in collision_fields
    }
    seen: dict[str, set[Any]] = {
        field: set()
        for field in (
            "case_id",
            "source_pack_id",
            "page_url",
            "download_url",
            "destination_path",
            "provider_attachment_id",
        )
    }
    for index, item in enumerate(mapped):
        prefix = f"batch.entries[{index}]"
        for field in ("case_id", "source_pack_id"):
            value = item.get(field)
            if not isinstance(value, str) or SAFE_ID.fullmatch(value) is None:
                _fail(f"{prefix}.{field}", "must be a lowercase safe identifier")
        _expect(f"{prefix}.provider", item.get("provider"), "OpenGameArt")
        _expect(f"{prefix}.partition", item.get("partition"), "development")
        _expect(f"{prefix}.license", item.get("license"), "CC0-1.0")
        _expect(f"{prefix}.commercial_reference", item.get("commercial_reference"), False)
        _expect(
            f"{prefix}.third_party_source_or_sample_pack_disclosed_on_page",
            item.get("third_party_source_or_sample_pack_disclosed_on_page"),
            False,
        )
        _expect(
            f"{prefix}.family_assignment_state",
            item.get("family_assignment_state"),
            "provisional_metadata_hypothesis_only",
        )
        _expect(
            f"{prefix}.source_qualification_state",
            item.get("source_qualification_state"),
            "unheard_uncomputed_pending_riotbox_1428",
        )
        _expect(f"{prefix}.source_start_seconds", item.get("source_start_seconds"), 0.0)
        acquisition.validate_provider_url(
            item.get("author_profile_url"), kind="profile", prefix=f"{prefix}.author_profile_url"
        )
        acquisition.validate_provider_url(
            item.get("page_url"), kind="page", prefix=f"{prefix}.page_url"
        )
        acquisition.validate_provider_url(
            item.get("download_url"), kind="download", prefix=f"{prefix}.download_url"
        )
        attachment = item.get("attachment_filename")
        if not isinstance(attachment, str) or not attachment.casefold().endswith(".wav"):
            _fail(f"{prefix}.attachment_filename", "must be an exact WAV filename")
        decoded_url_name = PurePosixPath(item["download_url"].split("/sites/default/files/", 1)[1])
        if decoded_url_name.name != attachment:
            _fail(f"{prefix}.attachment_filename", "must match the direct URL basename exactly")
        acquisition.validate_declared_attachment_byte_count(
            item.get("attachment_byte_count"), f"{prefix}.attachment_byte_count"
        )
        attachment_id = item.get("provider_attachment_id")
        if not isinstance(attachment_id, int) or isinstance(attachment_id, bool) or attachment_id <= 0:
            _fail(f"{prefix}.provider_attachment_id", "must be a positive integer")
        destination = _safe_destination(item.get("destination_path"), f"{prefix}.destination_path")
        for field, value in (
            ("case_id", item["case_id"]),
            ("source_pack_id", item["source_pack_id"]),
            ("page_url", item["page_url"]),
            ("download_url", item["download_url"]),
            ("destination_path", destination),
            ("provider_attachment_id", attachment_id),
        ):
            if value in seen[field]:
                _fail(f"{prefix}.{field}", "duplicates another batch identity")
            seen[field].add(value)
        for field in collision_fields:
            candidate_value = destination if field == "source_path" else item.get(field)
            if candidate_value in prior_values[field]:
                _fail(f"{prefix}.{field}", "collides with frozen Registry v2 metadata")


def validate_document(
    document: dict[str, Any], *, repo_root: Path | None = None
) -> str:
    root = repo_root if repo_root is not None else Path(__file__).resolve().parents[1]
    protocol_v2_validator.validate_repository(root)
    registry_v2, _, raw_registry, semantic_registry = _load_named_json(
        root, batch_contract.REGISTRY_V2_REL
    )
    _expect("Registry-v2 raw SHA-256", raw_registry, batch_contract.REGISTRY_V2_RAW_SHA256)
    _expect(
        "Registry-v2 semantic SHA-256",
        semantic_registry,
        batch_contract.REGISTRY_V2_SEMANTIC_SHA256,
    )
    _ensure_no_preexecution_results(document)
    _ensure_no_hash_cycles_or_forward_pins(document)
    _expect(
        "batch.schema",
        document.get("schema"),
        "riotbox.percussive_force_stage_a_v2_acquisition_batch.v1",
    )
    _expect("batch.schema_version", document.get("schema_version"), 1)
    _expect("batch.owner_ticket", document.get("owner_ticket"), "RIOTBOX-1430")
    _expect("batch.work_class", document.get("work_class"), "contract_enabler")
    _expect("batch.quality_proof", document.get("quality_proof"), False)
    _expect(
        "batch.source_qualification_claimed",
        document.get("source_qualification_claimed"),
        False,
    )
    _expect("batch.human_review_claimed", document.get("human_review_claimed"), False)
    _validate_bindings(document)
    try:
        acquisition.validate_format_contract(document.get("format_acceptance_contract"))
    except acquisition.AcquisitionError as error:
        raise ContractError(str(error)) from error
    try:
        _validate_entries(document, registry_v2)
    except acquisition.AcquisitionError as error:
        raise ContractError(str(error)) from error

    expected = batch_contract.build_document()
    difference = _first_difference(document, expected)
    if difference is not None:
        _fail("batch exact preregistration", difference)
    semantic_sha256 = batch_contract.semantic_sha256(document)
    _expect(
        "batch semantic SHA-256",
        semantic_sha256,
        EXPECTED_BATCH_SEMANTIC_SHA256,
    )
    return semantic_sha256


def validate_pins(raw_sha256: str, semantic_sha256: str) -> None:
    if SHA256.fullmatch(raw_sha256) is None or SHA256.fullmatch(semantic_sha256) is None:
        _fail("batch SHA-256", "pins must be lowercase hexadecimal SHA-256")
    _expect("batch raw SHA-256", raw_sha256, EXPECTED_BATCH_RAW_SHA256)
    _expect(
        "batch semantic SHA-256",
        semantic_sha256,
        EXPECTED_BATCH_SEMANTIC_SHA256,
    )


def validate_repository(repo_root: Path) -> FrozenAcquisitionBatchV1:
    document, payload, raw_sha256, semantic_sha256 = _load_named_json(
        repo_root, batch_contract.BATCH_REL
    )
    validated_semantic = validate_document(document, repo_root=repo_root)
    _expect("loaded batch semantic SHA-256", semantic_sha256, validated_semantic)
    validate_pins(raw_sha256, semantic_sha256)
    expected_payload = batch_contract.render(batch_contract.build_document())
    if payload != expected_payload:
        _fail(str(batch_contract.BATCH_REL), "bytes differ from deterministic renderer")
    return FrozenAcquisitionBatchV1._from_validated(repo_root, payload)


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    try:
        frozen = validate_repository(repo_root)
    except (
        ContractError,
        acquisition.AcquisitionError,
        protocol_v2_validator.ContractError,
        OSError,
    ) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: RIOTBOX-1430 acquisition batch v1 is frozen metadata-only")
    print(f"acquisition_batch_v1_raw_sha256={frozen.raw_sha256}")
    print(f"acquisition_batch_v1_semantic_sha256={frozen.semantic_sha256}")
    print(f"exact_entry_count={len(frozen.document['entries'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
