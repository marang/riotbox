"""Validate reuse of a human verdict for a bit-identical product artifact."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any


REUSE_FIELD = "human_verdict_provenance"
REUSE_SCHEMA = "riotbox.hash_identical_human_verdict_reuse.v1"
APPROVED_CONTRACTS = {
    "docs/benchmarks/tonal_riff_release_demo_evidence_reuse_v2.json": (
        "riotbox.tonal_riff_release_demo_evidence_reuse.v2",
        "cfdab651ceae05a494ccee5637a5e4fc3fb47bef24901b4ca5e76531a402cfa0",
    ),
    "docs/benchmarks/sparse_drums_release_demo_evidence_reuse_v1.json": (
        "riotbox.sparse_drums_release_demo_evidence_reuse.v1",
        "d0a658f12e75366d0243a230ddbb28af85746e0c7a5c601d3271b81ee5ed46c5",
    ),
}
VERDICT_DIMENSIONS = (
    "strongest_element",
    "source_recognition",
    "hook_after_two_bars",
)


def validate_reuse_evidence(
    evidence: dict[str, Any],
    path: Path,
    *,
    current_audio_sha256: str,
    current_product_manifest_sha256: str,
    expected_prior_human_verdict: str,
    current_verdict_dimensions: dict[str, str],
) -> None:
    require(evidence.get("schema") == REUSE_SCHEMA, f"{path}: verdict-reuse schema missing")
    require(evidence.get("result") == "pass", f"{path}: verdict-reuse result must pass")
    require(
        evidence.get("current_replay_created_new_verdict") is False,
        f"{path}: duplicate replay cannot create a new verdict",
    )
    require(
        evidence.get("new_quality_evidence") is False,
        f"{path}: hash-identical reuse cannot claim new quality evidence",
    )
    contract_ref = object_field(evidence, "reuse_contract", path)
    contract_path = resolve_repo_path(string_field(contract_ref, "path", path))
    contract_spec = approved_contract(contract_path)
    require(contract_spec is not None, f"{path}: verdict-reuse contract path changed")
    contract_schema, contract_sha256 = contract_spec
    require(
        string_field(contract_ref, "sha256", path) == contract_sha256,
        f"{path}: verdict-reuse contract pin changed",
    )
    require(contract_path.is_file(), f"{path}: verdict-reuse contract missing")
    require(
        sha256_file(contract_path) == string_field(contract_ref, "sha256", path),
        f"{path}: verdict-reuse contract hash changed",
    )
    contract = read_json(contract_path)
    require(contract.get("schema") == contract_schema, f"{path}: verdict-reuse contract schema changed")
    require(contract.get("status") == "frozen", f"{path}: verdict-reuse contract is not frozen")
    prior = object_field(contract, "prior_human_evidence", contract_path)
    current = object_field(contract, "current_exact_evidence", contract_path)
    reuse_rule = object_field(contract, "reuse_rule", contract_path)
    require(reuse_rule.get("schema") == REUSE_SCHEMA, f"{path}: verdict-reuse rule changed")
    require(
        reuse_rule.get("current_replay_created_new_verdict") is False,
        f"{path}: frozen reuse rule claims a new verdict",
    )
    require(
        prior.get("human_verdict") == expected_prior_human_verdict,
        f"{path}: prior human verdict changed",
    )
    for field in VERDICT_DIMENSIONS:
        require(
            current_verdict_dimensions.get(field) == prior.get(field),
            f"{path}: reused verdict dimension changed: {field}",
        )
    require(
        prior.get("audio_sha256") == current_audio_sha256
        and current_hash(current, "audio_sha256") == current_audio_sha256,
        f"{path}: current audio is not bit-identical to prior reviewed audio",
    )
    require(
        prior.get("product_manifest_sha256") == current_product_manifest_sha256
        and current_hash(current, "product_manifest_sha256")
        == current_product_manifest_sha256,
        f"{path}: current product manifest is not identical to prior reviewed product manifest",
    )
    require(
        current_identity_required(current, "bit_identical_to_prior_artifact")
        and current_identity_required(current, "product_manifest_identical_to_prior")
        and current_change_forbidden(current, "algorithm_or_threshold_change"),
        f"{path}: frozen exact-identity conditions are incomplete",
    )
    require(
        evidence.get("prior_ticket") == prior.get("ticket")
        and evidence.get("prior_structured_review_sha256")
        == prior.get("structured_review_sha256"),
        f"{path}: prior review identity changed",
    )
    document_path = resolve_repo_path(string_field(prior, "document_path", contract_path))
    require(document_path.is_file(), f"{path}: prior review document missing")
    require(
        sha256_file(document_path) == string_field(prior, "document_sha256", contract_path),
        f"{path}: prior review document hash changed",
    )
    document = document_path.read_text(encoding="utf-8")
    for token in (
        string_field(prior, "structured_review_sha256", contract_path),
        current_audio_sha256,
        current_product_manifest_sha256,
        f"`human_verdict: {expected_prior_human_verdict}`",
    ):
        require(token in document, f"{path}: prior review document does not bind {token}")


def approved_contract(contract_path: Path) -> tuple[str, str] | None:
    for relative_path, spec in APPROVED_CONTRACTS.items():
        if contract_path.resolve() == resolve_repo_path(relative_path).resolve():
            return spec
    return None


def current_hash(current: dict[str, Any], field: str) -> Any:
    return current.get(field, current.get(f"required_{field}"))


def current_identity_required(current: dict[str, Any], field: str) -> bool:
    return current.get(field) is True or current.get(f"{field}_required") is True


def current_change_forbidden(current: dict[str, Any], field: str) -> bool:
    return current.get(field) is False or current.get(f"{field}_allowed") is False


def resolve_repo_path(value: str) -> Path:
    path = Path(value).expanduser()
    if path.is_absolute():
        return path
    return Path(__file__).resolve().parent.parent / path


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def object_field(data: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{path}: {field} must be object")
    return value


def string_field(data: dict[str, Any], field: str, path: Path) -> str:
    value = data.get(field)
    require(isinstance(value, str) and bool(value.strip()), f"{path}: {field} must be string")
    return value


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)
