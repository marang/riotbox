"""Exact product-path evidence gate for structured demo-bank promotion."""

from __future__ import annotations

from pathlib import Path
from typing import Any


GATE_FIELD = "exact_product_path_review_gate"
GATE_SCHEMA = "riotbox.exact_product_path_review_gate.v1"
PRODUCT_PATH_KIND = "exact_runtime_mix_live_journey"


def validate_promotion_gate(
    gate: dict[str, Any], path: Path, *, expected_source_family: str
) -> None:
    require(gate.get("schema") == GATE_SCHEMA, f"{path}: exact product-path gate schema missing")
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
    require(
        gate.get("human_verdict") == "unverified",
        f"{path}: exact product-path technical gate must not claim a human verdict",
    )
    require(
        gate.get("failure_codes") == [],
        f"{path}: exact product-path gate has failure codes",
    )


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)
