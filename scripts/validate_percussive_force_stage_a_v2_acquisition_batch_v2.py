#!/usr/bin/env python3
"""Validate the complete source-blind RIOTBOX-1430 acquisition batch v2."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any
from urllib.parse import unquote, urlsplit

import percussive_force_stage_a_v2_acquisition as acquisition
import percussive_force_stage_a_v2_acquisition_batch_v2_contract as batch_contract
import validate_percussive_force_stage_a_protocol_v2 as protocol_v2_validator


EXPECTED_BATCH_RAW_SHA256 = (
    "d9b92635734e65d0154a7c17143c8759cc758d9b9cf756cda740b08623c53067"
)
EXPECTED_BATCH_SEMANTIC_SHA256 = (
    "af1605b781004aab984ff75845962130ca22fe936bf9f59a89de7e7ab8942dfb"
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
FORBIDDEN_ENTRY_KEYS = {
    "actual",
    "computed",
    "event_records",
    "feature_results",
    "gate_result",
    "human_verdict",
    "measurement",
    "observed_payload_sha256",
    "payload_sha256",
    "policy_results",
    "qualified",
    "render",
    "result",
    "survivor",
    "verdict",
}


class ContractError(ValueError):
    """Raised when acquisition batch v2 is not the exact frozen snapshot."""


@dataclass(frozen=True, init=False, slots=True)
class FrozenAcquisitionBatchV2:
    _repo_root: Path
    _payload: bytes

    def __init__(self, *_args: Any, **_kwargs: Any) -> None:
        raise TypeError(
            "FrozenAcquisitionBatchV2 must be created by validate_repository()"
        )

    @classmethod
    def _from_validated(
        cls, repo_root: Path, payload: bytes
    ) -> "FrozenAcquisitionBatchV2":
        frozen = object.__new__(cls)
        object.__setattr__(frozen, "_repo_root", repo_root.resolve())
        object.__setattr__(frozen, "_payload", bytes(payload))
        return frozen

    @property
    def path(self) -> Path:
        return batch_contract.BATCH_REL

    @property
    def schema(self) -> str:
        return "riotbox.percussive_force_stage_a_v2_acquisition_batch.v2"

    @property
    def raw_sha256(self) -> str:
        return hashlib.sha256(self._payload).hexdigest()

    @property
    def semantic_sha256(self) -> str:
        return batch_contract.semantic_sha256(
            _parse_json(self._payload, str(self.path))
        )

    @property
    def document(self) -> dict[str, Any]:
        return _parse_json(self._payload, str(self.path))

    @property
    def repo_root(self) -> Path:
        try:
            root = self._repo_root
        except AttributeError as error:
            raise ContractError(
                "unvalidated acquisition-v2 binding has no repository root"
            ) from error
        if not isinstance(root, Path):
            raise ContractError(
                "unvalidated acquisition-v2 binding has an invalid repository root"
            )
        return root

    def revalidated(self) -> "FrozenAcquisitionBatchV2":
        return validate_repository(self.repo_root)


def revalidate_frozen_batch(
    value: FrozenAcquisitionBatchV2,
) -> FrozenAcquisitionBatchV2:
    if type(value) is not FrozenAcquisitionBatchV2:
        raise ContractError(
            "acquisition-v2 binding must be the exact FrozenAcquisitionBatchV2 type"
        )
    return FrozenAcquisitionBatchV2.revalidated(value)


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
    if relative not in {
        batch_contract.BATCH_REL,
        batch_contract.REGISTRY_V2_REL,
        batch_contract.PREDECESSOR_BATCH_REL,
    }:
        _fail(str(relative), "batch-v2 validator refused an unnamed JSON path")
    path = repo_root / relative
    if path.is_symlink() or path.suffix != ".json" or not path.is_file():
        _fail(str(relative), "named contract is missing or not a regular JSON file")
    payload = path.read_bytes()
    document = _parse_json(payload, str(relative))
    return (
        document,
        payload,
        hashlib.sha256(payload).hexdigest(),
        batch_contract.semantic_sha256(document),
    )


def _load_rejection_report(repo_root: Path) -> bytes:
    relative = batch_contract.PREDECESSOR_REJECTION_REPORT_REL
    path = repo_root / relative
    if path.is_symlink() or path.suffix != ".md" or not path.is_file():
        _fail(str(relative), "predecessor rejection report is missing or not regular")
    payload = path.read_bytes()
    _expect(
        "predecessor rejection report raw SHA-256",
        hashlib.sha256(payload).hexdigest(),
        batch_contract.PREDECESSOR_REJECTION_REPORT_RAW_SHA256,
    )
    return payload


def _first_difference(actual: Any, expected: Any, path: str = "$") -> str | None:
    if type(actual) is not type(expected):
        return (
            f"{path} type changed from {type(expected).__name__} "
            f"to {type(actual).__name__}"
        )
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


def _ensure_entries_have_no_results(entries: list[dict[str, Any]]) -> None:
    def visit(value: Any, path: str) -> None:
        if isinstance(value, dict):
            for key, child in value.items():
                if (
                    key in FORBIDDEN_ENTRY_KEYS
                    or key.endswith("_result")
                    or key.endswith("_verdict")
                ):
                    _fail(path + "." + key, "source-result fields are forbidden")
                visit(child, path + "." + key)
        elif isinstance(value, list):
            for index, child in enumerate(value):
                visit(child, f"{path}[{index}]")

    for index, entry in enumerate(entries):
        visit(entry, f"batch.entries[{index}]")


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
            _fail(path, "batch v2 must not contain its own raw or semantic SHA-256")
        lowered = value.casefold()
        if (
            "source_holdout_rotation.v3" in lowered
            or "development_matrix.v3" in lowered
        ):
            _fail(path, "batch v2 must not bind forward Registry-v3 or Matrix-v3 authority")

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
        _fail(prefix, "must be a WAV directly inside the one atomic v2 directory")
    lowered = value.casefold()
    if any(marker in lowered for marker in COMMERCIAL_MARKERS):
        _fail(prefix, "contains a commercial/reference marker")
    return value


def _fold(value: Any, path: str) -> str:
    if not isinstance(value, str) or not value or value.strip() != value:
        _fail(path, "must be a non-empty exact string")
    return value.casefold()


def _decoded_download_filename(url: Any, path: str) -> str:
    if not isinstance(url, str):
        _fail(path, "must be a string")
    decoded = unquote(PurePosixPath(urlsplit(url).path).name)
    if not decoded or decoded in {".", ".."} or "/" in decoded or "\\" in decoded:
        _fail(path, "does not have one safe decoded filename")
    return decoded


def _prior_identity_sets(
    registry_v2: dict[str, Any], predecessor_v1: dict[str, Any]
) -> dict[str, set[Any]]:
    sets: dict[str, set[Any]] = {
        "author": set(),
        "case_id": set(),
        "source_pack_id": set(),
        "page_url": set(),
        "download_url": set(),
        "provider_attachment_id": set(),
        "attachment_name": set(),
    }

    def add_string(field: str, value: Any) -> None:
        if isinstance(value, str) and value:
            sets[field].add(value.casefold())

    for collection_name, prior_entries in (
        ("Registry v2", _array("registry_v2.entries", registry_v2.get("entries"))),
        (
            "acquisition batch v1",
            _array("batch_v1.entries", predecessor_v1.get("entries")),
        ),
    ):
        for index, raw in enumerate(prior_entries):
            item = _mapping(f"{collection_name}.entries[{index}]", raw)
            for field in ("author", "case_id", "source_pack_id", "page_url", "download_url"):
                add_string(field, item.get(field))
            attachment_id = item.get("provider_attachment_id")
            if isinstance(attachment_id, int) and not isinstance(attachment_id, bool):
                sets["provider_attachment_id"].add(attachment_id)
            add_string("attachment_name", item.get("attachment_filename"))
            download = item.get("download_url")
            if isinstance(download, str) and download:
                add_string(
                    "attachment_name",
                    _decoded_download_filename(
                        download,
                        f"{collection_name}.entries[{index}].download_url",
                    ),
                )
    return sets


def _validate_bindings(document: dict[str, Any]) -> None:
    expected = batch_contract.build_document()
    for key in (
        "protocol_binding",
        "predecessor_registry_binding",
        "predecessor_acquisition_rejection_binding",
    ):
        _expect(f"batch.{key}", document.get(key), expected[key])
    rejection = _mapping(
        "batch.predecessor_acquisition_rejection_binding",
        document.get("predecessor_acquisition_rejection_binding"),
    )
    for field in (
        "batch_raw_sha256",
        "batch_semantic_sha256",
        "rejection_report_raw_sha256",
        "access_log_raw_sha256",
    ):
        value = rejection.get(field)
        if not isinstance(value, str) or SHA256.fullmatch(value) is None:
            _fail(f"batch.predecessor_acquisition_rejection_binding.{field}", "invalid SHA-256")
    _expect(
        "batch.predecessor_acquisition_rejection_binding.request_count type",
        type(rejection.get("request_count")),
        int,
    )
    _expect(
        "batch.predecessor_acquisition_rejection_binding.successful_request_count type",
        type(rejection.get("successful_request_count")),
        int,
    )


def _validate_entries(
    document: dict[str, Any],
    registry_v2: dict[str, Any],
    predecessor_v1: dict[str, Any],
) -> None:
    entries = _array("batch.entries", document.get("entries"))
    if len(entries) != 3:
        _fail("batch.entries", "must contain exactly three predeclared sources")
    mapped = [
        _mapping(f"batch.entries[{index}]", value)
        for index, value in enumerate(entries)
    ]
    _ensure_entries_have_no_results(mapped)
    _expect("batch.entries.ordinals", [item.get("ordinal") for item in mapped], [1, 2, 3])
    _expect(
        "batch.entries.source_families",
        [item.get("source_family") for item in mapped],
        list(REQUIRED_FAMILIES),
    )
    prior = _prior_identity_sets(registry_v2, predecessor_v1)
    seen: dict[str, set[Any]] = {key: set() for key in prior}
    seen["destination_path"] = set()

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
            f"{prefix}.third_party_disclosure_interpretation",
            item.get("third_party_disclosure_interpretation"),
            "false_means_not_disclosed_on_the_named_page_not_proof_of_no_third_party_material",
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
            item.get("author_profile_url"),
            kind="profile",
            prefix=f"{prefix}.author_profile_url",
        )
        acquisition.validate_provider_url(
            item.get("page_url"), kind="page", prefix=f"{prefix}.page_url"
        )
        acquisition.validate_provider_url(
            item.get("download_url"),
            kind="download",
            prefix=f"{prefix}.download_url",
        )
        display_name = item.get("attachment_filename")
        url_name = item.get("download_url_filename")
        if not isinstance(display_name, str) or not display_name.casefold().endswith(".wav"):
            _fail(f"{prefix}.attachment_filename", "must be an exact WAV display filename")
        if not isinstance(url_name, str) or not url_name.casefold().endswith(".wav"):
            _fail(f"{prefix}.download_url_filename", "must be an exact WAV URL filename")
        _expect(
            f"{prefix}.download_url_filename",
            _decoded_download_filename(item.get("download_url"), f"{prefix}.download_url"),
            url_name,
        )
        acquisition.validate_declared_attachment_byte_count(
            item.get("attachment_byte_count"), f"{prefix}.attachment_byte_count"
        )
        attachment_id = item.get("provider_attachment_id")
        if (
            not isinstance(attachment_id, int)
            or isinstance(attachment_id, bool)
            or attachment_id <= 0
        ):
            _fail(f"{prefix}.provider_attachment_id", "must be a positive integer")
        destination = _safe_destination(
            item.get("destination_path"), f"{prefix}.destination_path"
        )

        scalar_identities: tuple[tuple[str, Any], ...] = (
            ("author", _fold(item.get("author"), f"{prefix}.author")),
            ("case_id", item["case_id"].casefold()),
            ("source_pack_id", item["source_pack_id"].casefold()),
            ("page_url", _fold(item.get("page_url"), f"{prefix}.page_url")),
            ("download_url", _fold(item.get("download_url"), f"{prefix}.download_url")),
            ("provider_attachment_id", attachment_id),
        )
        for field, value in scalar_identities:
            if value in prior[field]:
                _fail(f"{prefix}.{field}", "reuses a Registry-v2 or batch-v1 identity")
            if value in seen[field]:
                _fail(f"{prefix}.{field}", "duplicates another batch-v2 identity")
            seen[field].add(value)
        entry_names: dict[str, str] = {}
        for field, name in (
            ("attachment_filename", display_name),
            ("download_url_filename", url_name),
        ):
            folded_name = name.casefold()
            if folded_name in prior["attachment_name"]:
                _fail(f"{prefix}.{field}", "reuses a Registry-v2 or batch-v1 attachment name")
            if folded_name in seen["attachment_name"]:
                _fail(f"{prefix}.{field}", "duplicates another batch-v2 attachment name")
            entry_names.setdefault(folded_name, field)
        seen["attachment_name"].update(entry_names)
        if destination in seen["destination_path"]:
            _fail(f"{prefix}.destination_path", "duplicates another batch-v2 destination")
        seen["destination_path"].add(destination)


def validate_document(
    document: dict[str, Any], *, repo_root: Path | None = None
) -> str:
    root = repo_root if repo_root is not None else Path(__file__).resolve().parents[1]
    protocol_v2_validator.validate_repository(root)
    predecessor, _, raw_predecessor, semantic_predecessor = _load_named_json(
        root, batch_contract.PREDECESSOR_BATCH_REL
    )
    _expect(
        "predecessor batch-v1 raw SHA-256",
        raw_predecessor,
        batch_contract.PREDECESSOR_BATCH_RAW_SHA256,
    )
    _expect(
        "predecessor batch-v1 semantic SHA-256",
        semantic_predecessor,
        batch_contract.PREDECESSOR_BATCH_SEMANTIC_SHA256,
    )
    _load_rejection_report(root)
    registry_v2, _, raw_registry, semantic_registry = _load_named_json(
        root, batch_contract.REGISTRY_V2_REL
    )
    _expect("Registry-v2 raw SHA-256", raw_registry, batch_contract.REGISTRY_V2_RAW_SHA256)
    _expect(
        "Registry-v2 semantic SHA-256",
        semantic_registry,
        batch_contract.REGISTRY_V2_SEMANTIC_SHA256,
    )
    _ensure_no_hash_cycles_or_forward_pins(document)
    _expect(
        "batch.schema",
        document.get("schema"),
        "riotbox.percussive_force_stage_a_v2_acquisition_batch.v2",
    )
    _expect("batch.schema_version", document.get("schema_version"), 2)
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
        _validate_entries(document, registry_v2, predecessor)
    except acquisition.AcquisitionError as error:
        raise ContractError(str(error)) from error
    expected = batch_contract.build_document()
    difference = _first_difference(document, expected)
    if difference is not None:
        _fail("batch-v2 exact preregistration", difference)
    semantic_sha256 = batch_contract.semantic_sha256(document)
    _expect(
        "batch-v2 semantic SHA-256",
        semantic_sha256,
        EXPECTED_BATCH_SEMANTIC_SHA256,
    )
    return semantic_sha256


def validate_pins(raw_sha256: str, semantic_sha256: str) -> None:
    if SHA256.fullmatch(raw_sha256) is None or SHA256.fullmatch(semantic_sha256) is None:
        _fail("batch-v2 SHA-256", "pins must be lowercase hexadecimal SHA-256")
    _expect("batch-v2 raw SHA-256", raw_sha256, EXPECTED_BATCH_RAW_SHA256)
    _expect(
        "batch-v2 semantic SHA-256",
        semantic_sha256,
        EXPECTED_BATCH_SEMANTIC_SHA256,
    )


def validate_repository(repo_root: Path) -> FrozenAcquisitionBatchV2:
    document, payload, raw_sha256, semantic_sha256 = _load_named_json(
        repo_root, batch_contract.BATCH_REL
    )
    validated_semantic = validate_document(document, repo_root=repo_root)
    _expect("loaded batch-v2 semantic SHA-256", semantic_sha256, validated_semantic)
    validate_pins(raw_sha256, semantic_sha256)
    expected_payload = batch_contract.render(batch_contract.build_document())
    if payload != expected_payload:
        _fail(str(batch_contract.BATCH_REL), "bytes differ from deterministic renderer")
    return FrozenAcquisitionBatchV2._from_validated(repo_root, payload)


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
    print("PASS: RIOTBOX-1430 acquisition batch v2 is frozen source-blind metadata")
    print(f"acquisition_batch_v2_raw_sha256={frozen.raw_sha256}")
    print(f"acquisition_batch_v2_semantic_sha256={frozen.semantic_sha256}")
    print(f"exact_entry_count={len(frozen.document['entries'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
