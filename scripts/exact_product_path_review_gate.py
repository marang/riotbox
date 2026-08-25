"""Exact product-path evidence gate for structured demo-bank promotion."""

from __future__ import annotations

from pathlib import Path
from typing import Any


GATE_FIELD = "exact_product_path_review_gate"
GATE_SCHEMA_V1 = "riotbox.exact_product_path_review_gate.v1"
GATE_SCHEMA_V2 = "riotbox.exact_product_path_review_gate.v2"
GATE_SCHEMAS = {GATE_SCHEMA_V1, GATE_SCHEMA_V2}
PRODUCT_PATH_KIND = "exact_runtime_mix_live_journey"
SPARSE_DRUMS_LANE_ROLES = {
    "w30": "source_transform",
    "tr909": "hardest_transient",
    "mc202": "punctuation",
    "source_monitor": "stay_out",
    "bass_owner": "unassigned",
}


def validate_promotion_gate(
    gate: dict[str, Any], path: Path, *, expected_source_family: str
) -> None:
    schema = gate.get("schema")
    require(schema in GATE_SCHEMAS, f"{path}: exact product-path gate schema missing")
    require(gate.get("result") == "pass", f"{path}: exact product-path gate must pass")
    require(
        gate.get("source_family") == expected_source_family,
        f"{path}: exact product-path source_family mismatch",
    )
    require(
        gate.get("product_path_kind") == PRODUCT_PATH_KIND,
        f"{path}: exact product-path kind invalid",
    )
    required_true = (
        "source_backed",
        "source_timing_backed",
        "source_graph_capture_lineage_proven",
        "action_lexicon_queue_commit_proven",
        "session_replay_proven",
        "callback_partitions_sample_exact",
        "restart_recall_sample_exact",
        "source_role_decision_proven",
        "promotion_blocked_until_human_pass",
    )
    for field in required_true:
        require(gate.get(field) is True, f"{path}: exact product-path {field} must be true")
    required_false = (
        "hardcoded_musical_output",
        "primitive_or_template_only",
        "fallback_music_present",
        "quality_proof",
    )
    for field in required_false:
        require(gate.get(field) is False, f"{path}: exact product-path {field} must be false")
    require(
        isinstance(gate.get("scripted_performer_driver"), bool),
        f"{path}: exact product-path scripted_performer_driver must be boolean",
    )
    if schema == GATE_SCHEMA_V2:
        validate_active_lane_roles(gate, path, expected_source_family)
    require(
        gate.get("human_verdict") == "unverified",
        f"{path}: exact product-path technical gate must not claim a human verdict",
    )
    require(
        gate.get("failure_codes") == [],
        f"{path}: exact product-path gate has failure codes",
    )


def validate_active_lane_roles(
    gate: dict[str, Any], path: Path, expected_source_family: str
) -> None:
    require(
        expected_source_family == "sparse_drums",
        f"{path}: exact product-path v2 currently qualifies sparse_drums only",
    )
    require(
        gate.get("active_contributors_sample_exact") is True,
        f"{path}: exact product-path active contributors must be sample-exact",
    )
    require(
        gate.get("unassigned_role_not_claimed") is True,
        f"{path}: exact product-path must not claim an unassigned role",
    )
    require(
        gate.get("lane_roles") == SPARSE_DRUMS_LANE_ROLES,
        f"{path}: exact product-path sparse lane roles changed",
    )


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)
