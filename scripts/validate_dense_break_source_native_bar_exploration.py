#!/usr/bin/env python3
"""Validate the frozen RIOTBOX-1468 source-native bar exploration contracts."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any


CONTRACT = Path("docs/benchmarks/dense_break_source_native_bar_exploration_v1.json")
CONTRACT_SHA256 = "027c1cb6e007f5e2d8064fa1db40a5b91dbea2c890dd101310297c4375441fef"
CONTRACT_V2 = Path("docs/benchmarks/dense_break_source_native_bar_exploration_v2.json")
CONTRACT_V2_SHA256 = "ac71e8daa9f862a8341910d63e0457cd657e6506808eda4032d132b4fb443517"
RECOVERY = Path("docs/benchmarks/dense_break_source_native_bar_preflight_recovery_v1.json")
RECOVERY_SHA256 = "1683e1aa824dba52c6c0c55d977107cfc535fa0a5fae0da9db0a3c8806ef7278"
REGISTRY_SHA256 = "67b5b8b2882575cf70fa61aacf25ae282c17714fe51ffcb13f905458e025d552"


def main() -> int:
    repo = Path(__file__).resolve().parent.parent
    payload = (repo / CONTRACT).read_bytes()
    require(hashlib.sha256(payload).hexdigest() == CONTRACT_SHA256, "contract SHA-256 changed")
    contract = json.loads(payload)
    validate(contract, repo)
    run_mutation_fixtures(contract, repo)
    v2_payload = (repo / CONTRACT_V2).read_bytes()
    require(hashlib.sha256(v2_payload).hexdigest() == CONTRACT_V2_SHA256, "v2 contract SHA-256 changed")
    contract_v2 = json.loads(v2_payload)
    validate_v2(contract_v2, repo)
    run_v2_mutation_fixtures(contract_v2, repo)
    recovery_payload = (repo / RECOVERY).read_bytes()
    require(hashlib.sha256(recovery_payload).hexdigest() == RECOVERY_SHA256, "recovery contract SHA-256 changed")
    validate_recovery(json.loads(recovery_payload))
    print("valid Dense source-native full-bar v1/v2/recovery contracts and mutation fixtures")
    return 0


def validate(contract: dict[str, Any], repo: Path) -> None:
    require(
        contract.get("schema") == "riotbox.dense_break_source_native_bar_exploration.v1",
        "schema changed",
    )
    require(
        contract.get("status") == "frozen_source_blind_before_development_access",
        "contract is not source-blind frozen",
    )
    mechanism = object_field(contract, "mechanism")
    require(mechanism.get("owner") == "w30_pad_playback_grammar", "owner changed")
    require(mechanism.get("control") == "half_beat_chop_v1", "control changed")
    require(
        mechanism.get("candidate") == "source_native_full_bar_v1", "candidate changed"
    )
    input_contract = object_field(mechanism, "input")
    require(input_contract.get("capture_length_bars") == 1, "capture length changed")
    require(input_contract.get("beats_per_bar") == 4, "meter changed")
    require(
        input_contract.get("maximum_bar_duration_mismatch_frames") == 1,
        "bar-duration gate changed",
    )
    candidate = object_field(mechanism, "candidate_mapping")
    require(candidate.get("playback_rate") == 1.0, "playback rate changed")
    require(candidate.get("pitch_change_semitones") == 0.0, "pitch changed")
    require(candidate.get("transport_retrigger_count_per_bar") == 0, "retrigger count changed")
    require(candidate.get("additional_lane_count") == 0, "additional lane appeared")
    require(candidate.get("additional_effect_count") == 0, "additional effect appeared")
    proof = object_field(mechanism, "synthetic_proof")
    require(proof.get("callback_frame_counts") == [128, 257], "callback matrix changed")
    preflight = object_field(mechanism, "technical_preflight")
    require(preflight.get("maximum_true_peak_dbtp") == -1.0, "true-peak gate changed")
    require(
        preflight.get("candidate_boundary_jump_limit")
        == "max(0.12, source_boundary_jump_abs * 1.5)",
        "boundary gate changed",
    )
    access = object_field(contract, "development_access")
    require(access.get("partition") == "development", "source partition changed")
    require(access.get("maximum_unique_source_files") == 1, "source bound changed")
    require(access.get("case_id") == "dense_beat03_130", "source case changed")
    require(access.get("holdout_audio_access") is False, "Holdout access enabled")
    require(access.get("commercial_reference_access") is False, "reference access enabled")
    registry = repo / str(access.get("registry_path"))
    require(hashlib.sha256(registry.read_bytes()).hexdigest() == REGISTRY_SHA256, "registry changed")
    entries = json.loads(registry.read_text(encoding="utf-8")).get("entries", [])
    matching = [entry for entry in entries if entry.get("case_id") == access.get("case_id")]
    require(len(matching) == 1, "Development case is not uniquely registered")
    require(matching[0].get("source_family") == "dense_break", "source family changed")
    require(matching[0].get("source_path") == access.get("source_path"), "source path changed")
    stopping = object_field(contract, "stopping_rule")
    require(stopping.get("maximum_candidate_variants") == 1, "variant limit changed")
    require(stopping.get("human_reviews") == 1, "review limit changed")
    claims = object_field(contract, "claim_boundary")
    require(claims.get("development_exploration_only") is True, "stage boundary changed")
    for field in (
        "product_behavior",
        "source_general",
        "quality_proof",
        "holdout_authorized",
        "demo_ready",
        "release_ready",
        "p023_complete",
    ):
        require(claims.get(field) is False, f"claim unexpectedly enabled: {field}")


def run_mutation_fixtures(contract: dict[str, Any], repo: Path) -> None:
    mutations = (
        lambda value: value["mechanism"]["candidate_mapping"].update(playback_rate=0.97),
        lambda value: value["mechanism"]["candidate_mapping"].update(additional_lane_count=1),
        lambda value: value["development_access"].update(maximum_unique_source_files=2),
        lambda value: value["development_access"].update(holdout_audio_access=True),
        lambda value: value["stopping_rule"].update(maximum_candidate_variants=2),
    )
    for mutate in mutations:
        invalid = copy.deepcopy(contract)
        mutate(invalid)
        try:
            validate(invalid, repo)
        except ValueError:
            continue
        raise ValueError("mutated source-native full-bar contract unexpectedly passed")


def validate_v2(contract: dict[str, Any], repo: Path) -> None:
    require(
        contract.get("schema") == "riotbox.dense_break_source_native_bar_exploration.v2",
        "v2 schema changed",
    )
    predecessor = object_field(contract, "predecessor")
    require(predecessor.get("sha256") == CONTRACT_SHA256, "v2 predecessor changed")
    require(predecessor.get("outcome") == "technical_rejection_before_audio_render", "v1 outcome changed")
    require(predecessor.get("audio_artifacts_rendered") is False, "v1 audio outcome changed")
    require(predecessor.get("human_playback_performed") is False, "v1 playback outcome changed")
    change = object_field(contract, "sole_change")
    require(change.get("field") == "mechanism.input.runtime_tempo_authority", "v2 scope changed")
    require(change.get("v2") == "confirmed_capture_bar_bpm_v1", "v2 tempo authority changed")
    require(
        change.get("formula")
        == "source_sample_rate_hz * 60 * beats_per_bar / capture_playback_frame_count",
        "v2 tempo formula changed",
    )
    requirements = object_field(change, "requirements")
    require(requirements.get("runtime_render_uses_derived_bpm_exactly") is True, "v2 runtime tempo weakened")
    require(requirements.get("source_bar_to_runtime_bar_mismatch_frames_max") == 1, "v2 frame gate changed")
    unchanged = object_field(contract, "unchanged_v1_contract")
    require(unchanged.get("musical_owner") == "w30_pad_playback_grammar", "v2 owner changed")
    require(unchanged.get("candidate") == "source_native_full_bar_v1", "v2 candidate changed")
    require(unchanged.get("candidate_variants") == 1, "v2 candidate budget changed")
    require(unchanged.get("playback_rate") == 1.0, "v2 playback rate changed")
    require(unchanged.get("additional_lane_count") == 0, "v2 additional lane appeared")
    require(unchanged.get("additional_effect_count") == 0, "v2 effect appeared")
    require(unchanged.get("callback_frame_counts") == [128, 257], "v2 callback matrix changed")
    access = object_field(contract, "development_access")
    require(access.get("partition") == "development", "v2 source partition changed")
    require(access.get("maximum_unique_source_files") == 1, "v2 source bound changed")
    require(access.get("case_id") == "dense_beat03_130", "v2 source changed")
    require(access.get("holdout_audio_access") is False, "v2 Holdout access enabled")
    registry = repo / str(access.get("registry_path"))
    require(hashlib.sha256(registry.read_bytes()).hexdigest() == REGISTRY_SHA256, "v2 registry changed")
    stopping = object_field(contract, "stopping_rule")
    require(stopping.get("maximum_candidate_variants") == 1, "v2 variant limit changed")
    require(stopping.get("remaining_human_reviews") == 1, "v2 review budget changed")
    claims = object_field(contract, "claim_boundary")
    require(claims.get("development_exploration_only") is True, "v2 stage changed")
    for field in (
        "product_behavior",
        "source_general",
        "quality_proof",
        "holdout_authorized",
        "demo_ready",
        "release_ready",
        "p023_complete",
    ):
        require(claims.get(field) is False, f"v2 claim unexpectedly enabled: {field}")


def run_v2_mutation_fixtures(contract: dict[str, Any], repo: Path) -> None:
    mutations = (
        lambda value: value["sole_change"].update(v2="declared_bpm"),
        lambda value: value["unchanged_v1_contract"].update(playback_rate=0.99),
        lambda value: value["unchanged_v1_contract"].update(additional_effect_count=1),
        lambda value: value["development_access"].update(holdout_audio_access=True),
        lambda value: value["stopping_rule"].update(maximum_candidate_variants=2),
    )
    for mutate in mutations:
        invalid = copy.deepcopy(contract)
        mutate(invalid)
        try:
            validate_v2(invalid, repo)
        except ValueError:
            continue
        raise ValueError("mutated source-native full-bar v2 contract unexpectedly passed")


def validate_recovery(contract: dict[str, Any]) -> None:
    require(
        contract.get("schema") == "riotbox.dense_break_source_native_bar_preflight_recovery.v1",
        "recovery schema changed",
    )
    qualification = object_field(contract, "qualification_contract")
    require(qualification.get("sha256") == CONTRACT_V2_SHA256, "recovery v2 pin changed")
    incident = object_field(contract, "incident")
    require(incident.get("source_read_count") == 1, "recovery source-read count changed")
    require(incident.get("renderer_exit_code") == 0, "recovery renderer status changed")
    require(incident.get("human_playback_performed") is False, "recovery playback status changed")
    authorized = object_field(contract, "authorized_recovery")
    require(authorized.get("initialize_existing_numpy_metric_binding") is True, "recovery fix changed")
    for field in (
        "source_audio_reopen",
        "source_directory_discovery",
        "rerender",
        "artifact_replacement",
        "algorithm_or_threshold_change",
        "human_playback_before_technical_pass",
    ):
        require(authorized.get(field) is False, f"recovery unexpectedly authorizes {field}")
    stopping = object_field(contract, "stopping_rule")
    require(stopping.get("further_recovery_or_source_access") is False, "recovery retry enabled")


def object_field(parent: dict[str, Any], field: str) -> dict[str, Any]:
    value = parent.get(field)
    require(isinstance(value, dict), f"{field} must be an object")
    return value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    raise SystemExit(main())
