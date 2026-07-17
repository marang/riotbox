#!/usr/bin/env python3
"""Validate Riotbox listening manifest JSON v1.

This checks the stable manifest envelope documented in
docs/benchmarks/listening_manifest_v1_json_contract_2026-04-29.md. Pack-specific
metrics, thresholds, cases, and source metadata remain flexible, but known
optional QA contracts such as Feral scorecards are validated when present.
"""

from __future__ import annotations

import json
import math
import re
import sys
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1
VERSIONED_PRIMITIVE_SCHEMA = re.compile(
    r"^riotbox\.[a-z0-9][a-z0-9_.-]*\.v[1-9][0-9]*$"
)
VERSIONED_RECIPE_ID = re.compile(r"^[a-z0-9]+(?:_[a-z0-9]+)*_v[1-9][0-9]*$")
CANONICAL_GESTURE_TRANSITION_REF = re.compile(
    r"^/gesture_transitions/(?:0|[1-9][0-9]*)$"
)
TR909_FILL_PRIMITIVE_SCHEMA = "riotbox.tr909_fill_recipe.v1"
TR909_FILL_SOURCE_MODULATION_SCHEMA = "riotbox.tr909_fill_source_modulation.v2"
TR909_FILL_RUNTIME_PATHS = {
    "runtime_mix.tr909.fill_recipe",
    "runtime_mix.tr909.drum_bus_level",
    "runtime_mix.tr909.slam_intensity",
    "runtime_mix.tr909.source_bar_grid_phase",
    "runtime_mix.non_tr909_bed.fill_focus",
    "runtime_mix.source_monitor.blend_fill_focus",
}
TR909_FILL_MODULATION_RUNTIME_PARAMETERS = {
    "runtime_mix.tr909.drum_bus_level",
    "runtime_mix.tr909.slam_intensity",
    "runtime_mix.tr909.source_bar_grid_phase",
}
TR909_FILL_SELECTION_KEYS = {
    "mode",
    "routing",
    "pattern_adoption",
    "phrase_variation",
}
SOURCE_TIMING_BPM_MATCH_TOLERANCE = 1.0
EPSILON = 0.000001
SOURCE_TIMING_POLICY_PROFILES = {
    "broad_research",
    "dance_loop_auto_readiness",
    "p014_scene_movement_probe",
}
SOURCE_TIMING_READINESS = {"unavailable", "weak", "needs_review", "ready"}
SOURCE_TIMING_CUES = {"grid locked", "needs confirm", "listen first", "not available"}
SOURCE_TIMING_ACTIONABILITY = {
    "grid can steer moves",
    "confirm grid first",
    "listen first",
    "timing unavailable",
}
SOURCE_TIMING_GRID_USE = {
    "locked_grid",
    "short_loop_manual_confirm",
    "manual_confirm_only",
    "fallback_grid",
    "unavailable",
}
SOURCE_TIMING_BEAT_STATUSES = {"unavailable", "weak", "stable", "ambiguous"}
SOURCE_TIMING_DOWNBEAT_STATUSES = {"unavailable", "weak", "stable", "ambiguous"}
SOURCE_TIMING_CONFIDENCE_RESULTS = {
    "degraded",
    "candidate_cautious",
    "candidate_ambiguous",
}
SOURCE_TIMING_DRIFT_STATUSES = {
    "unavailable",
    "not_enough_material",
    "stable",
    "high",
}
SOURCE_TIMING_PHRASE_STATUSES = {
    "unavailable",
    "not_enough_material",
    "ambiguous_downbeat",
    "high_drift",
    "stable",
}
GROOVE_SUBDIVISIONS = {
    "eighth",
    "triplet",
    "sixteenth",
    "thirty_second",
}
GRID_BPM_DECISION_REASONS = {
    "user_override",
    "source_timing_ready",
    "source_timing_needs_review_manual_confirm",
    "source_timing_requires_manual_confirm",
    "source_timing_not_ready",
    "source_timing_missing_bpm",
    "source_timing_invalid_bpm",
}
GRID_BPM_SOURCES = {
    "user_override",
    "source_timing",
    "static_default",
}
TR909_GROOVE_TIMING_REASONS = {
    "not_source_timing_grid",
    "no_groove_residuals",
    "invalid_groove_offset",
    "groove_offset_too_small",
    "source_timing_groove_residual",
    "source_timing_not_locked",
}
STATIC_DEFAULT_GRID_BPM_REASONS = {
    "source_timing_requires_manual_confirm",
    "source_timing_not_ready",
    "source_timing_missing_bpm",
    "source_timing_invalid_bpm",
}


def main() -> int:
    try:
        path, require_existing_artifacts = parse_args(sys.argv[1:])
    except ValueError as error:
        print(error, file=sys.stderr)
        print(
            "usage: validate_listening_manifest_json.py [--require-existing-artifacts] <manifest.json>",
            file=sys.stderr,
        )
        return 2

    try:
        manifest = json.loads(path.read_text())
        validate_manifest(manifest)
        if require_existing_artifacts:
            validate_artifact_paths(manifest, path.parent)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid listening manifest JSON: {error}", file=sys.stderr)
        return 1

    print(f"valid riotbox listening manifest v{SCHEMA_VERSION}: {path}")
    return 0


def parse_args(args: list[str]) -> tuple[Path, bool]:
    require_existing_artifacts = False
    paths: list[str] = []

    for arg in args:
        if arg == "--require-existing-artifacts":
            require_existing_artifacts = True
        elif arg.startswith("-"):
            raise ValueError(f"unknown option: {arg}")
        else:
            paths.append(arg)

    if len(paths) != 1:
        raise ValueError("expected exactly one manifest path")

    return Path(paths[0]), require_existing_artifacts


def validate_manifest(manifest: Any) -> None:
    require_object(manifest, "manifest")
    require_schema_version(manifest)
    require_string(manifest, "pack_id")
    require_one_of(manifest, "result", {"pass", "fail"})

    artifacts = require_list(manifest, "artifacts")
    if not artifacts:
        raise ValueError("artifacts must not be empty")

    for index, artifact in enumerate(artifacts):
        validate_artifact(artifact, index)

    scorecard = manifest.get("feral_scorecard")
    if scorecard is not None:
        validate_feral_scorecard(scorecard)

    source_timing = manifest.get("source_timing")
    if source_timing is not None:
        validate_source_timing(source_timing)
    elif manifest.get("pack_id") == "feral-grid-demo" and "grid_bpm_source" in manifest:
        raise ValueError("source_timing must be present for feral-grid-demo grid BPM manifests")

    if manifest.get("pack_id") == "feral-grid-demo" and "grid_bpm_source" in manifest:
        validate_generated_feral_grid_source_timing(source_timing)
        validate_grid_bpm_decision(manifest, source_timing)
        validate_source_timing_bpm_delta_consistency(manifest, source_timing)

    validate_primitive_renderer_boundary(manifest)
    validate_source_grid_output_drift(manifest)


def validate_artifact(artifact: Any, index: int) -> None:
    require_object(artifact, f"artifact {index}")
    require_string(artifact, "role", f"artifact {index} role")
    require_string(artifact, "kind", f"artifact {index} kind")
    require_string(artifact, "path", f"artifact {index} path")
    require_optional_string_or_null(artifact, "metrics_path", f"artifact {index} metrics_path")
    require_optional_string_or_null(artifact, "case_id", f"artifact {index} case_id")


def validate_feral_scorecard(scorecard: Any) -> None:
    require_object(scorecard, "feral_scorecard")
    require_string(scorecard, "readiness", "feral_scorecard readiness")
    require_string(
        scorecard,
        "break_rebuild_potential",
        "feral_scorecard break_rebuild_potential",
    )
    require_non_negative_int(scorecard, "hook_fragment_count")
    require_non_negative_int(scorecard, "break_support_count")
    require_non_negative_int(scorecard, "quote_risk_count")
    require_non_negative_int(scorecard, "capture_candidate_count")
    require_string(scorecard, "top_reason", "feral_scorecard top_reason")
    require_bool(scorecard, "source_backed")
    require_bool(scorecard, "generated")
    require_bool(scorecard, "fallback_like")
    require_non_empty_string_list(scorecard, "lane_gestures")
    require_non_empty_string_list(scorecard, "material_sources")
    require_non_empty_string_list(scorecard, "warnings")


def validate_primitive_renderer_boundary(manifest: dict[str, Any]) -> None:
    primitive_records = find_string_value_records(manifest, "primitive_renderer")
    if not primitive_records:
        return
    primitive_paths = sorted(path for path, _record in primitive_records)

    boundary = require_object_field(manifest, "primitive_renderer_boundary")
    require_equal(boundary, "quality_proof", False)
    require_equal(boundary, "demo_readiness", "unverified")
    require_equal(boundary, "promotion_blocked", True)
    require_string(boundary, "musician_message", "primitive_renderer_boundary musician_message")
    evidence_role = boundary.get("evidence_role")
    if evidence_role == "non_product_diagnostic_control":
        require_equal(boundary, "schema", "riotbox.primitive_renderer_boundary.v1")
        require_equal(boundary, "product_output_allowed", False)
    elif evidence_role == "product_primitive_vocabulary":
        require_equal(boundary, "schema", "riotbox.primitive_renderer_boundary.v2")
        require_equal(boundary, "product_output_allowed", True)
        require_equal(boundary, "recipe_derivation_claimed", False)
        require_equal(boundary, "pattern_selection_claimed", False)
        require_equal(boundary, "source_output_modulation_claimed", True)
        require_equal(boundary, "source_failure_fallback", False)
        require_equal(
            boundary,
            "promotion_target",
            "source_derived_musical_intelligence",
        )
        require_equal(boundary, "promotion_target_scope", "recipe_and_pattern_selection")
        validate_product_primitive_vocabulary(manifest, boundary, primitive_records)
    else:
        raise ValueError(
            "primitive_renderer_boundary evidence_role must be "
            "'non_product_diagnostic_control' or 'product_primitive_vocabulary', "
            f"got {evidence_role!r}"
        )
    affected_paths = require_non_empty_string_list(
        boundary,
        "affected_paths",
        "primitive_renderer_boundary affected_paths",
    )
    if sorted(affected_paths) != primitive_paths:
        raise ValueError(
            "primitive_renderer_boundary affected_paths must exactly match primitive_renderer paths: "
            f"expected {primitive_paths!r}, got {sorted(affected_paths)!r}"
        )


def validate_product_primitive_vocabulary(
    manifest: dict[str, Any],
    boundary: dict[str, Any],
    primitive_records: list[tuple[str, dict[str, Any]]],
) -> None:
    require_equal(manifest, "quality_proof", False)
    if "demo_readiness" in manifest:
        require_equal(manifest, "demo_readiness", "unverified")
    for field in ("source_derivation_claimed", "source_failure_fallback"):
        if field in manifest:
            require_equal(manifest, field, False)

    boundary_runtime_paths = require_non_empty_string_list(
        boundary,
        "affected_runtime_paths",
        "primitive_renderer_boundary affected_runtime_paths",
    )
    boundary_artifacts = require_non_empty_string_list(
        boundary,
        "affected_artifacts",
        "primitive_renderer_boundary affected_artifacts",
    )
    activation = require_object_field(boundary, "activation")
    require_equal(activation, "kind", "explicit_committed_performer_gesture")
    boundary_activation_refs = require_non_empty_string_list(
        activation,
        "references",
        "primitive_renderer_boundary activation references",
    )
    if len(boundary_activation_refs) != len(set(boundary_activation_refs)):
        raise ValueError("primitive_renderer_boundary activation references must be unique")

    manifest_artifact_paths = {artifact["path"] for artifact in manifest["artifacts"]}
    record_activation_refs: set[str] = set()
    record_artifacts: set[str] = set()
    registered_runtime_paths: set[str] = set()
    for path, record in primitive_records:
        require_string(record, "primitive_schema", f"{path} primitive_schema")
        primitive_schema = record["primitive_schema"]
        if not VERSIONED_PRIMITIVE_SCHEMA.fullmatch(primitive_schema):
            raise ValueError(
                f"{path} primitive_schema must be a versioned riotbox schema, "
                f"got {primitive_schema!r}"
            )
        require_string(record, "recipe_id", f"{path} recipe_id")
        recipe_id = record["recipe_id"]
        if not VERSIONED_RECIPE_ID.fullmatch(recipe_id):
            raise ValueError(
                f"{path} recipe_id must end in a positive _vN version, got {recipe_id!r}"
            )
        selection_inputs = require_object_field(record, "selection_inputs")
        if not selection_inputs:
            raise ValueError(f"{path} selection_inputs must not be empty")
        for key, value in selection_inputs.items():
            if not key.strip():
                raise ValueError(f"{path} selection_inputs keys must not be empty")
            if isinstance(value, str):
                if not value.strip():
                    raise ValueError(
                        f"{path} selection_inputs.{key} must not be an empty string"
                    )
            elif isinstance(value, bool):
                pass
            elif isinstance(value, (int, float)) and not isinstance(value, bool):
                if not math.isfinite(float(value)):
                    raise ValueError(
                        f"{path} selection_inputs.{key} must be finite"
                    )
            else:
                raise TypeError(
                    f"{path} selection_inputs.{key} must be a typed scalar"
                )

        require_string(record, "activation_ref", f"{path} activation_ref")
        activation_ref = record["activation_ref"]
        if not CANONICAL_GESTURE_TRANSITION_REF.fullmatch(activation_ref):
            raise ValueError(
                f"{path} activation_ref must target the canonical gesture_transitions "
                f"collection, got {activation_ref!r}"
            )
        activation_record = require_object(
            resolve_json_pointer(manifest, activation_ref),
            f"{path} activation_ref target",
        )
        require_string(activation_record, "command", f"{path} activation command")
        require_non_negative_int(
            activation_record,
            "action_id",
            f"{path} activation",
        )
        require_string(activation_record, "boundary", f"{path} activation boundary")
        record_activation_refs.add(activation_ref)

        affected_artifacts = require_non_empty_string_list(
            record,
            "affected_artifacts",
            f"{path} affected_artifacts",
        )
        if len(affected_artifacts) != len(set(affected_artifacts)):
            raise ValueError(f"{path} affected_artifacts must be unique")
        for artifact_path in affected_artifacts:
            if artifact_path not in manifest_artifact_paths:
                raise ValueError(
                    f"{path} affected artifact is not declared in artifacts: {artifact_path}"
                )
            record_artifacts.add(artifact_path)
        registered_runtime_paths.update(
            validate_registered_product_primitive(
                path,
                record,
                selection_inputs,
                activation_record,
                affected_artifacts,
                manifest["artifacts"],
            )
        )

    if sorted(boundary_activation_refs) != sorted(record_activation_refs):
        raise ValueError(
            "primitive_renderer_boundary activation references must exactly match "
            f"primitive activation_ref values: expected {sorted(record_activation_refs)!r}, "
            f"got {sorted(boundary_activation_refs)!r}"
        )
    if sorted(boundary_runtime_paths) != sorted(registered_runtime_paths):
        raise ValueError(
            "primitive_renderer_boundary affected_runtime_paths must exactly match "
            f"registered primitive schemas: expected {sorted(registered_runtime_paths)!r}, "
            f"got {sorted(boundary_runtime_paths)!r}"
        )
    if len(boundary_artifacts) != len(set(boundary_artifacts)):
        raise ValueError("primitive_renderer_boundary affected_artifacts must be unique")
    if sorted(boundary_artifacts) != sorted(record_artifacts):
        raise ValueError(
            "primitive_renderer_boundary affected_artifacts must exactly match primitive "
            f"records: expected {sorted(record_artifacts)!r}, "
            f"got {sorted(boundary_artifacts)!r}"
        )


def validate_registered_product_primitive(
    path: str,
    record: dict[str, Any],
    selection_inputs: dict[str, Any],
    activation_record: dict[str, Any],
    affected_artifacts: list[str],
    manifest_artifacts: list[dict[str, Any]],
) -> set[str]:
    primitive_schema = record["primitive_schema"]
    if primitive_schema != TR909_FILL_PRIMITIVE_SCHEMA:
        raise ValueError(
            f"{path} primitive_schema is not registered for product output: "
            f"{primitive_schema!r}"
        )

    require_equal(
        record,
        "source_evidence_role",
        "availability_timing_and_pressure_modulation",
    )
    require_equal(record, "source_evidence_selects_pattern", False)
    require_equal(record, "source_evidence_modulates_output", True)
    for field in ("source_derivation_claimed", "source_failure_fallback"):
        if field in record:
            require_equal(record, field, False)
    for field, expected in (
        ("quality_proof", False),
        ("demo_readiness", "unverified"),
        ("product_output_allowed", True),
        ("evidence_role", "product_primitive_vocabulary"),
    ):
        if field in record:
            require_equal(record, field, expected)

    input_keys = set(selection_inputs)
    if input_keys != TR909_FILL_SELECTION_KEYS:
        raise ValueError(
            f"{path} selection_inputs keys must exactly match the registered "
            f"{TR909_FILL_PRIMITIVE_SCHEMA} contract: expected "
            f"{sorted(TR909_FILL_SELECTION_KEYS)!r}, got {sorted(input_keys)!r}"
        )
    require_equal(selection_inputs, "mode", "fill")
    require_equal(selection_inputs, "routing", "drum_bus_support")
    validate_tr909_fill_source_modulation(path, record, activation_record)

    recipe_id = record["recipe_id"]
    adoption = selection_inputs["pattern_adoption"]
    variation = selection_inputs["phrase_variation"]
    if recipe_id in {
        "phrase_drive_choke_dive_stomp_v1",
        "phrase_drive_long_choke_dive_stomp_v2",
        "phrase_drive_break_cut_stomp_v1",
    }:
        if adoption != "mainline_drive" or variation != "phrase_drive":
            raise ValueError(
                f"{path} recipe_id {recipe_id!r} requires "
                "pattern_adoption='mainline_drive' and phrase_variation='phrase_drive'"
            )
    elif recipe_id == "phrase_drive_accent_ghost_v1":
        if adoption not in {"support_pulse", "takeover_grid"} or variation != "phrase_drive":
            raise ValueError(
                f"{path} recipe_id {recipe_id!r} requires support/takeover adoption "
                "and phrase_variation='phrase_drive'"
            )
    elif recipe_id == "generic_fill_v1":
        if adoption not in {"support_pulse", "mainline_drive", "takeover_grid"}:
            raise ValueError(
                f"{path} recipe_id {recipe_id!r} has an unsupported pattern_adoption "
                f"{adoption!r}"
            )
        if variation not in {"phrase_anchor", "phrase_lift", "phrase_release"}:
            raise ValueError(
                f"{path} recipe_id {recipe_id!r} has an unsupported phrase_variation "
                f"{variation!r}"
            )
    else:
        raise ValueError(
            f"{path} recipe_id is not registered for {TR909_FILL_PRIMITIVE_SCHEMA}: "
            f"{recipe_id!r}"
        )

    require_equal(activation_record, "command", "tr909.fill_next")
    require_equal(activation_record, "actor", "performer")
    require_equal(activation_record, "status", "committed")
    require_equal(activation_record, "boundary", "Bar")
    if activation_record["action_id"] <= 0:
        raise ValueError(f"{path} activation action_id must be a positive integer")

    require_string(
        activation_record,
        "candidate_artifact",
        f"{path} activation candidate_artifact",
    )
    candidate_artifact = activation_record["candidate_artifact"]
    if candidate_artifact not in affected_artifacts:
        raise ValueError(
            f"{path} activation candidate_artifact must be included in affected_artifacts: "
            f"{candidate_artifact!r}"
        )
    artifacts_by_path: dict[str, list[dict[str, Any]]] = {}
    for artifact in manifest_artifacts:
        artifacts_by_path.setdefault(artifact["path"], []).append(artifact)
    allowed_roles = {
        "candidate",
        "committed_fill",
        "performance_stage",
        "continuous_performance_sequence",
    }
    for artifact_path in affected_artifacts:
        artifact_records = artifacts_by_path.get(artifact_path, [])
        if len(artifact_records) != 1:
            raise ValueError(
                f"{path} affected artifact must resolve to exactly one declared artifact, "
                f"got {len(artifact_records)} for {artifact_path!r}"
            )
        artifact_record = artifact_records[0]
        if artifact_record["role"] not in allowed_roles:
            raise ValueError(
                f"{path} affected artifact has a role outside the registered Fill output "
                f"contract: {artifact_record['role']!r}"
            )
        if artifact_record["kind"] not in {"wav", "audio_wav"} or not artifact_path.endswith(
            ".wav"
        ):
            raise ValueError(f"{path} affected artifact must be a declared WAV artifact")

    candidate_records = artifacts_by_path.get(candidate_artifact, [])
    if len(candidate_records) != 1:
        raise ValueError(
            f"{path} activation candidate_artifact must resolve to exactly one declared "
            f"artifact, got {len(candidate_records)} for {candidate_artifact!r}"
        )
    candidate_record = candidate_records[0]
    if candidate_record["role"] not in {"candidate", "committed_fill"}:
        raise ValueError(
            f"{path} activation candidate_artifact must have candidate/committed_fill role, "
            f"got {candidate_record['role']!r}"
        )

    return TR909_FILL_RUNTIME_PATHS


def validate_tr909_fill_source_modulation(
    path: str,
    record: dict[str, Any],
    activation_record: dict[str, Any],
) -> None:
    modulation = require_object_field(record, "source_modulation")
    require_exact_keys(
        modulation,
        {
            "schema",
            "source_feature_path",
            "source_feature_value",
            "source_timing_path",
            "derived_policy",
            "resolved_render_inputs",
            "affected_runtime_parameters",
            "pattern_selection_changed",
        },
        f"{path} source_modulation",
    )
    require_equal(modulation, "schema", TR909_FILL_SOURCE_MODULATION_SCHEMA)
    require_equal(
        modulation,
        "source_feature_path",
        "session.runtime_state.lane_state.mc202.source_phrase_plan."
        "source_expression.transient_backbeat",
    )
    require_equal(
        modulation,
        "source_timing_path",
        "source_graph.timing.primary_hypothesis.transport_bar_grid_anchor.beat_cursor",
    )
    transient_backbeat = require_finite_unit_number(
        modulation,
        "source_feature_value",
        f"{path} source_modulation source_feature_value",
    )
    require_equal(modulation, "pattern_selection_changed", False)
    affected_parameters = require_non_empty_string_list(
        modulation,
        "affected_runtime_parameters",
        f"{path} source_modulation affected_runtime_parameters",
    )
    if len(affected_parameters) != len(set(affected_parameters)) or set(
        affected_parameters
    ) != TR909_FILL_MODULATION_RUNTIME_PARAMETERS:
        raise ValueError(
            f"{path} source_modulation affected_runtime_parameters must exactly match "
            f"{sorted(TR909_FILL_MODULATION_RUNTIME_PARAMETERS)!r}"
        )

    derived_policy = require_object_field(modulation, "derived_policy")
    require_exact_keys(
        derived_policy,
        {
            "tr909_drum_level",
            "tr909_slam_floor",
            "source_bar_grid_anchor_beat_cursor",
        },
        f"{path} source_modulation derived_policy",
    )
    tr909_drum_level = require_finite_unit_number(
        derived_policy,
        "tr909_drum_level",
        f"{path} source_modulation derived tr909_drum_level",
    )
    tr909_slam_floor = require_finite_unit_number(
        derived_policy,
        "tr909_slam_floor",
        f"{path} source_modulation derived tr909_slam_floor",
    )
    require_non_negative_int(
        derived_policy,
        "source_bar_grid_anchor_beat_cursor",
        f"{path} source_modulation derived policy",
    )
    source_bar_anchor = derived_policy["source_bar_grid_anchor_beat_cursor"]
    expected_drum_level = 0.68 + transient_backbeat * 0.16
    expected_slam_floor = 0.54 + transient_backbeat * 0.16
    if abs(tr909_drum_level - expected_drum_level) > 0.00001:
        raise ValueError(
            f"{path} derived tr909_drum_level does not match registered source policy"
        )
    if abs(tr909_slam_floor - expected_slam_floor) > 0.00001:
        raise ValueError(
            f"{path} derived tr909_slam_floor does not match registered source policy"
        )

    resolved = require_object_field(modulation, "resolved_render_inputs")
    require_exact_keys(
        resolved,
        {
            "drum_bus_level",
            "slam_intensity",
            "slam_enabled",
            "source_bar_grid_anchor_position_beats",
        },
        f"{path} source_modulation resolved_render_inputs",
    )
    resolved_drum_level = require_finite_unit_number(
        resolved,
        "drum_bus_level",
        f"{path} source_modulation resolved drum_bus_level",
    )
    resolved_slam_intensity = require_finite_unit_number(
        resolved,
        "slam_intensity",
        f"{path} source_modulation resolved slam_intensity",
    )
    require_bool(resolved, "slam_enabled", f"{path} source_modulation resolved")
    resolved_source_bar_anchor = require_number(
        resolved,
        "source_bar_grid_anchor_position_beats",
        f"{path} source_modulation resolved",
    )
    if not math.isfinite(resolved_source_bar_anchor) or resolved_source_bar_anchor < 0.0:
        raise ValueError(f"{path} resolved source-bar phase must be finite and non-negative")
    if resolved_source_bar_anchor != source_bar_anchor:
        raise ValueError(
            f"{path} resolved source-bar phase drifted from the derived confirmed anchor"
        )
    if resolved_drum_level + EPSILON < tr909_drum_level:
        raise ValueError(f"{path} resolved drum_bus_level lost its source-derived floor")
    if resolved_slam_intensity + EPSILON < tr909_slam_floor:
        raise ValueError(f"{path} resolved slam_intensity lost its source-derived floor")

    controls = require_object_field(activation_record, "control_values")
    require_equal(controls, "tr909_mode_after", "fill")
    activation_drum_level = require_finite_unit_number(
        controls,
        "tr909_drum_bus_level_after",
        f"{path} activation tr909_drum_bus_level_after",
    )
    activation_slam_intensity = require_finite_unit_number(
        controls,
        "tr909_slam_after",
        f"{path} activation tr909_slam_after",
    )
    require_bool(controls, "tr909_slam_enabled_after", f"{path} activation controls")
    if abs(activation_drum_level - resolved_drum_level) > EPSILON:
        raise ValueError(
            f"{path} source modulation drum_bus_level does not match activation render state"
        )
    if abs(activation_slam_intensity - resolved_slam_intensity) > EPSILON:
        raise ValueError(
            f"{path} source modulation slam_intensity does not match activation render state"
        )
    if controls["tr909_slam_enabled_after"] != resolved["slam_enabled"]:
        raise ValueError(
            f"{path} source modulation slam_enabled does not match activation render state"
        )


def resolve_json_pointer(value: Any, pointer: str) -> Any:
    if not pointer.startswith("/"):
        raise ValueError(f"activation_ref must be an absolute JSON pointer, got {pointer!r}")
    current = value
    for raw_token in pointer[1:].split("/"):
        token = raw_token.replace("~1", "/").replace("~0", "~")
        if isinstance(current, dict):
            if token not in current:
                raise ValueError(f"activation_ref path does not exist: {pointer}")
            current = current[token]
        elif isinstance(current, list):
            if not token.isdigit() or int(token) >= len(current):
                raise ValueError(f"activation_ref array index does not exist: {pointer}")
            current = current[int(token)]
        else:
            raise ValueError(f"activation_ref traverses a scalar value: {pointer}")
    return current


def find_string_value_records(
    value: Any,
    needle: str,
    path: str = "",
) -> list[tuple[str, dict[str, Any]]]:
    if isinstance(value, dict):
        records = []
        for key, child in value.items():
            child_path = f"{path}.{key}" if path else str(key)
            if child == needle:
                records.append((child_path, value))
            else:
                records.extend(find_string_value_records(child, needle, child_path))
        return records
    if isinstance(value, list):
        records = []
        for index, child in enumerate(value):
            child_path = f"{path}[{index}]"
            records.extend(find_string_value_records(child, needle, child_path))
        return records
    return []


def validate_source_timing(source_timing: Any) -> None:
    require_object(source_timing, "source_timing")
    require_string(source_timing, "schema", "source_timing schema")
    require_schema_version(source_timing)
    require_string(source_timing, "source_id", "source_timing source_id")
    require_one_of(source_timing, "policy_profile", SOURCE_TIMING_POLICY_PROFILES)
    require_one_of(source_timing, "readiness", SOURCE_TIMING_READINESS)
    require_bool(source_timing, "requires_manual_confirm", "source_timing")
    if "cue" in source_timing:
        require_one_of(source_timing, "cue", SOURCE_TIMING_CUES)
        require_source_timing_readiness_cue_match(
            source_timing["cue"],
            source_timing["readiness"],
            source_timing["requires_manual_confirm"],
        )
    if "actionability" in source_timing:
        require_one_of(source_timing, "actionability", SOURCE_TIMING_ACTIONABILITY)
        require_source_timing_readiness_actionability_match(
            source_timing["actionability"],
            source_timing["readiness"],
            source_timing["requires_manual_confirm"],
        )
    if "grid_use" in source_timing:
        require_one_of(source_timing, "grid_use", SOURCE_TIMING_GRID_USE)
        require_source_timing_grid_use_match(source_timing, source_timing["grid_use"])
    require_optional_float_or_null(source_timing, "primary_bpm", "source_timing primary_bpm")
    require_optional_bool_or_null(
        source_timing,
        "bpm_agrees_with_grid",
        "source_timing bpm_agrees_with_grid",
    )
    require_optional_non_negative_int_or_null(
        source_timing,
        "primary_downbeat_offset_beats",
        "source_timing primary_downbeat_offset_beats",
    )
    require_optional_unit_float_or_null(
        source_timing,
        "primary_downbeat_score",
        "source_timing primary_downbeat_score",
        require_present=False,
    )
    require_optional_unit_float_or_null(
        source_timing,
        "primary_downbeat_margin",
        "source_timing primary_downbeat_margin",
        require_present=False,
    )
    if "alternate_downbeat_phase_count" in source_timing:
        require_non_negative_int(
            source_timing,
            "alternate_downbeat_phase_count",
            "source_timing",
        )
    require_one_of(source_timing, "beat_status", SOURCE_TIMING_BEAT_STATUSES)
    require_one_of(source_timing, "downbeat_status", SOURCE_TIMING_DOWNBEAT_STATUSES)
    require_one_of(source_timing, "confidence_result", SOURCE_TIMING_CONFIDENCE_RESULTS)
    require_one_of(source_timing, "drift_status", SOURCE_TIMING_DRIFT_STATUSES)
    require_one_of(source_timing, "phrase_status", SOURCE_TIMING_PHRASE_STATUSES)
    require_non_negative_int(source_timing, "primary_phrase_count", "source_timing")
    require_non_negative_int(source_timing, "primary_phrase_bar_count", "source_timing")
    validate_source_timing_phrase_evidence(source_timing)
    validate_source_timing_anchor_evidence(source_timing.get("anchor_evidence"))
    validate_source_timing_groove_evidence(source_timing.get("groove_evidence"))
    require_non_negative_int(
        source_timing,
        "alternate_evidence_count",
        "source_timing",
    )
    require_string_list(source_timing, "warning_codes", "source_timing warning_codes")


def validate_generated_feral_grid_source_timing(source_timing: Any | None) -> None:
    timing = require_object(source_timing, "source_timing")
    require_one_of(timing, "cue", SOURCE_TIMING_CUES)
    require_source_timing_readiness_cue_match(
        timing["cue"],
        timing["readiness"],
        timing["requires_manual_confirm"],
    )
    require_one_of(timing, "actionability", SOURCE_TIMING_ACTIONABILITY)
    require_source_timing_readiness_actionability_match(
        timing["actionability"],
        timing["readiness"],
        timing["requires_manual_confirm"],
    )
    require_optional_unit_float_or_null(
        timing,
        "primary_downbeat_score",
        "source_timing primary_downbeat_score",
        require_present=True,
    )
    require_optional_unit_float_or_null(
        timing,
        "primary_downbeat_margin",
        "source_timing primary_downbeat_margin",
        require_present=True,
    )
    require_non_negative_int(
        timing,
        "alternate_downbeat_phase_count",
        "source_timing",
    )


def require_source_timing_grid_use_match(source_timing: dict[str, Any], grid_use: str) -> None:
    expected = source_timing_grid_use(source_timing)
    if grid_use != expected:
        raise ValueError(f"source_timing grid_use must be {expected!r}, got {grid_use!r}")


def require_source_timing_readiness_cue_match(
    cue: str, readiness: str, requires_manual_confirm: bool
) -> None:
    expected = source_timing_readiness_cue(readiness, requires_manual_confirm)
    if cue != expected:
        raise ValueError(
            "source_timing cue must match readiness/manual-confirm state "
            f"{readiness!r}/{requires_manual_confirm!r}: expected {expected!r}, got {cue!r}"
        )


def source_timing_readiness_cue(readiness: str, requires_manual_confirm: bool) -> str:
    if readiness == "unavailable":
        return "not available"
    if requires_manual_confirm:
        return "needs confirm"
    if readiness == "ready":
        return "grid locked"
    if readiness in {"needs_review", "weak"}:
        return "listen first"
    return "unknown"


def require_source_timing_readiness_actionability_match(
    actionability: str, readiness: str, requires_manual_confirm: bool
) -> None:
    expected = source_timing_readiness_actionability(readiness, requires_manual_confirm)
    if actionability != expected:
        raise ValueError(
            "source_timing actionability must match readiness/manual-confirm state "
            f"{readiness!r}/{requires_manual_confirm!r}: expected {expected!r}, got {actionability!r}"
        )


def source_timing_readiness_actionability(
    readiness: str, requires_manual_confirm: bool
) -> str:
    if readiness == "unavailable":
        return "timing unavailable"
    if requires_manual_confirm:
        return "confirm grid first"
    if readiness == "ready":
        return "grid can steer moves"
    if readiness in {"needs_review", "weak"}:
        return "listen first"
    return "unknown"


def source_timing_grid_use(source_timing: dict[str, Any]) -> str:
    if source_timing.get("primary_bpm") is None or source_timing["readiness"] == "unavailable":
        return "unavailable"
    if source_timing["readiness"] == "ready" and not source_timing["requires_manual_confirm"]:
        return "locked_grid"
    if is_stable_short_loop_manual_confirm(source_timing):
        return "short_loop_manual_confirm"
    if source_timing["requires_manual_confirm"]:
        return "manual_confirm_only"
    return "fallback_grid"


def is_stable_short_loop_manual_confirm(source_timing: dict[str, Any]) -> bool:
    return (
        source_timing["readiness"] == "needs_review"
        and source_timing["requires_manual_confirm"] is True
        and source_timing.get("primary_bpm") is not None
        and source_timing["beat_status"] == "stable"
        and source_timing["downbeat_status"] == "stable"
        and source_timing["phrase_status"] == "not_enough_material"
        and source_timing["confidence_result"] == "candidate_cautious"
        and source_timing["alternate_evidence_count"] == 0
    )


def validate_source_timing_phrase_evidence(source_timing: dict[str, Any]) -> None:
    phrase_status = source_timing["phrase_status"]
    phrase_count = source_timing["primary_phrase_count"]
    phrase_bar_count = source_timing["primary_phrase_bar_count"]
    if phrase_status == "stable" and (phrase_count == 0 or phrase_bar_count == 0):
        raise ValueError(
            "source_timing stable phrase evidence requires positive "
            "primary_phrase_count and primary_phrase_bar_count"
        )
    if phrase_status == "unavailable" and (phrase_count != 0 or phrase_bar_count != 0):
        raise ValueError(
            "source_timing unavailable phrase evidence requires zero "
            "primary_phrase_count and primary_phrase_bar_count"
        )
    if phrase_status == "not_enough_material" and phrase_count != 0:
        raise ValueError(
            "source_timing not_enough_material phrase evidence must not report "
            "primary phrases"
        )


def validate_source_timing_anchor_evidence(anchor_evidence: Any) -> None:
    evidence = require_object(anchor_evidence, "source_timing anchor_evidence")
    require_non_negative_int(
        evidence,
        "primary_anchor_count",
        "source_timing anchor_evidence",
    )
    require_non_negative_int(
        evidence,
        "primary_kick_anchor_count",
        "source_timing anchor_evidence",
    )
    require_non_negative_int(
        evidence,
        "primary_backbeat_anchor_count",
        "source_timing anchor_evidence",
    )
    require_non_negative_int(
        evidence,
        "primary_transient_anchor_count",
        "source_timing anchor_evidence",
    )
    typed_count = (
        evidence["primary_kick_anchor_count"]
        + evidence["primary_backbeat_anchor_count"]
        + evidence["primary_transient_anchor_count"]
    )
    if typed_count > evidence["primary_anchor_count"]:
        raise ValueError(
            "source_timing anchor_evidence typed anchor counts must not exceed primary_anchor_count"
        )


def validate_source_timing_groove_evidence(groove_evidence: Any) -> None:
    evidence = require_object(groove_evidence, "source_timing groove_evidence")
    require_non_negative_int(
        evidence,
        "primary_groove_residual_count",
        "source_timing groove_evidence",
    )
    total = evidence["primary_groove_residual_count"]
    require_non_negative_number(
        evidence,
        "primary_max_abs_offset_ms",
        "source_timing groove_evidence",
    )
    preview = require_list(
        evidence,
        "primary_groove_preview",
        "source_timing groove_evidence primary_groove_preview",
    )
    if len(preview) > min(total, 4):
        raise ValueError(
            "source_timing groove_evidence preview must contain at most the first four residuals"
        )
    for index, item in enumerate(preview):
        validate_source_timing_groove_preview(item, index)


def validate_source_timing_groove_preview(item: Any, index: int) -> None:
    residual = require_object(item, f"source_timing groove residual {index}")
    require_one_of(residual, "subdivision", GROOVE_SUBDIVISIONS)
    require_number(residual, "offset_ms", f"source_timing groove residual {index} offset_ms")
    require_non_negative_number(
        residual,
        "confidence",
        f"source_timing groove residual {index} confidence",
    )
    confidence = residual["confidence"]
    if confidence > 1.0:
        raise ValueError("source_timing groove residual confidence must be <= 1")


def validate_grid_bpm_decision(
    manifest: dict[str, Any], source_timing: Any | None
) -> None:
    require_one_of(manifest, "grid_bpm_source", GRID_BPM_SOURCES)
    require_one_of(manifest, "grid_bpm_decision_reason", GRID_BPM_DECISION_REASONS)
    source = manifest.get("grid_bpm_source")
    reason = manifest.get("grid_bpm_decision_reason")

    if source == "user_override" and reason != "user_override":
        raise ValueError("user_override grid BPM source requires user_override decision reason")
    if source == "source_timing" and reason not in {
        "source_timing_ready",
        "source_timing_needs_review_manual_confirm",
    }:
        raise ValueError("source_timing grid BPM source requires a source-timing decision reason")
    if source == "static_default" and reason not in STATIC_DEFAULT_GRID_BPM_REASONS:
        raise ValueError("static_default grid BPM source requires a source-timing fallback reason")

    if not isinstance(source_timing, dict):
        return
    if reason == "source_timing_ready":
        if source_timing.get("readiness") != "ready":
            raise ValueError("source_timing_ready requires source_timing.readiness == ready")
        if source_timing.get("requires_manual_confirm") is not False:
            raise ValueError(
                "source_timing_ready requires source_timing.requires_manual_confirm == false"
            )
    if reason == "source_timing_requires_manual_confirm":
        if source_timing.get("requires_manual_confirm") is not True:
            raise ValueError(
                "source_timing_requires_manual_confirm requires manual confirmation evidence"
            )
    if reason == "source_timing_needs_review_manual_confirm":
        if source_timing.get("readiness") != "needs_review":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires source_timing.readiness == needs_review"
            )
        if source_timing.get("requires_manual_confirm") is not True:
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires manual confirmation evidence"
            )
        if source_timing.get("beat_status") != "stable":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires stable beat evidence"
            )
        if source_timing.get("downbeat_status") != "stable":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires stable downbeat evidence"
            )
        if source_timing.get("confidence_result") != "candidate_cautious":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires candidate_cautious confidence"
            )
        if source_timing.get("alternate_evidence_count") != 0:
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires no alternate evidence"
            )
    if reason == "source_timing_not_ready" and source_timing.get("readiness") == "ready":
        raise ValueError("source_timing_not_ready cannot be used with ready source timing")


def validate_source_timing_bpm_delta_consistency(
    manifest: dict[str, Any], source_timing: Any | None
) -> None:
    if "source_timing_bpm_delta" not in manifest:
        raise TypeError("source_timing_bpm_delta must be present as a number or null")
    require_optional_float_or_null(
        manifest,
        "source_timing_bpm_delta",
        "source_timing_bpm_delta",
    )
    source = manifest.get("grid_bpm_source")
    reason = manifest.get("grid_bpm_decision_reason")
    delta = manifest.get("source_timing_bpm_delta")

    if source == "source_timing":
        if not isinstance(delta, (int, float)) or isinstance(delta, bool):
            raise TypeError("source_timing grid BPM source requires numeric source_timing_bpm_delta")
        if abs(float(delta)) > EPSILON:
            raise ValueError("source_timing grid BPM source requires source_timing_bpm_delta == 0")
        require_bpm_agreement(source_timing, True, "source_timing grid BPM source")
        return

    if reason in {"source_timing_missing_bpm", "source_timing_invalid_bpm"}:
        if delta is not None:
            raise ValueError(f"{reason} requires null source_timing_bpm_delta")
        require_bpm_agreement(source_timing, None, reason)
        return

    if source == "user_override" and delta is None:
        require_bpm_agreement(source_timing, None, "user_override without usable source BPM")
        return

    if source in {"static_default", "user_override"}:
        if not isinstance(delta, (int, float)) or isinstance(delta, bool):
            raise TypeError(
                f"{source}/{reason} requires numeric source_timing_bpm_delta when source BPM is usable"
            )
        expected_agrees = float(delta) <= SOURCE_TIMING_BPM_MATCH_TOLERANCE
        require_bpm_agreement(source_timing, expected_agrees, f"{source}/{reason}")


def require_bpm_agreement(
    source_timing: Any | None, expected: bool | None, context: str
) -> None:
    if not isinstance(source_timing, dict):
        return
    actual = source_timing.get("bpm_agrees_with_grid")
    if actual is not expected:
        raise ValueError(
            f"{context} requires source_timing.bpm_agrees_with_grid == {expected!r}"
        )


def validate_source_grid_output_drift(manifest: dict[str, Any]) -> None:
    metrics = manifest.get("metrics")
    if not isinstance(metrics, dict):
        return

    for metric_key in (
        "source_grid_output_drift",
        "tr909_source_grid_alignment",
        "mc202_source_grid_alignment",
        "w30_source_grid_alignment",
    ):
        if metric_key in metrics:
            validate_source_grid_output_drift_metric(metrics, metric_key)

    if "tr909_groove_timing" in metrics:
        validate_tr909_groove_timing(metrics["tr909_groove_timing"])


def validate_tr909_groove_timing(value: Any) -> None:
    timing = require_object(value, "metrics tr909_groove_timing")
    require_bool(timing, "applied", "metrics tr909_groove_timing")
    require_one_of(timing, "reason", TR909_GROOVE_TIMING_REASONS)
    require_number(timing, "offset_ms", "metrics tr909_groove_timing offset_ms")
    require_non_negative_int(
        timing,
        "source_residual_count",
        "metrics tr909_groove_timing",
    )
    require_non_negative_number(
        timing,
        "source_max_abs_offset_ms",
        "metrics tr909_groove_timing source_max_abs_offset_ms",
    )
    subdivision = timing.get("source_subdivision")
    if subdivision is not None and subdivision not in GROOVE_SUBDIVISIONS:
        raise ValueError(
            "metrics tr909_groove_timing source_subdivision must be a known groove subdivision or null"
        )
    if timing["applied"] and timing["reason"] != "source_timing_groove_residual":
        raise ValueError("applied tr909_groove_timing requires source_timing_groove_residual")
    if not timing["applied"] and timing["offset_ms"] != 0:
        raise ValueError("inactive tr909_groove_timing must use offset_ms 0")


def validate_source_grid_output_drift_metric(metrics: dict[str, Any], metric_key: str) -> None:
    drift = require_object(
        metrics.get(metric_key),
        f"metrics {metric_key}",
    )
    require_non_negative_int(drift, "beat_count", metric_key)
    require_non_negative_int(drift, "hit_count", metric_key)
    if drift["hit_count"] > drift["beat_count"]:
        raise ValueError(f"{metric_key} hit_count must not exceed beat_count")
    hit_ratio = require_number(drift, "hit_ratio", f"{metric_key} hit_ratio")
    if hit_ratio < 0.0 or hit_ratio > 1.0:
        raise ValueError(f"{metric_key} hit_ratio must be between 0 and 1")
    require_non_negative_number(
        drift,
        "max_peak_offset_ms",
        f"{metric_key} max_peak_offset_ms",
    )
    require_non_negative_number(
        drift,
        "max_allowed_peak_offset_ms",
        f"{metric_key} max_allowed_peak_offset_ms",
    )


def validate_artifact_paths(manifest: dict[str, Any], manifest_dir: Path) -> None:
    for index, artifact in enumerate(manifest["artifacts"]):
        require_file(manifest_dir / artifact["path"], f"artifact {index} path")
        metrics_path = artifact.get("metrics_path")
        if metrics_path is not None:
            require_file(manifest_dir / metrics_path, f"artifact {index} metrics_path")


def require_file(path: Path, name: str) -> None:
    if not path.is_file():
        raise ValueError(f"{name} does not exist: {path}")


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_object_field(parent: dict[str, Any], field: str) -> dict[str, Any]:
    return require_object(parent.get(field), field)


def require_exact_keys(value: dict[str, Any], expected: set[str], name: str) -> None:
    actual = set(value)
    if actual != expected:
        raise ValueError(
            f"{name} keys must exactly match {sorted(expected)!r}, got {sorted(actual)!r}"
        )


def require_equal(parent: dict[str, Any], field: str, expected: Any) -> None:
    value = parent.get(field)
    if value != expected:
        raise ValueError(f"{field} must be {expected!r}, got {value!r}")


def require_schema_version(parent: dict[str, Any]) -> None:
    value = parent.get("schema_version")
    if not isinstance(value, int) or isinstance(value, bool) or value != SCHEMA_VERSION:
        raise ValueError(f"schema_version must be integer {SCHEMA_VERSION}, got {value!r}")


def require_string(parent: dict[str, Any], field: str, name: str | None = None) -> None:
    value = parent.get(field)
    if not isinstance(value, str) or not value.strip():
        raise TypeError(f"{name or field} must be a non-empty string")


def require_one_of(parent: dict[str, Any], field: str, expected: set[str]) -> None:
    value = parent.get(field)
    if value not in expected:
        choices = ", ".join(sorted(expected))
        raise ValueError(f"{field} must be one of {choices}, got {value!r}")


def require_list(parent: dict[str, Any], field: str, name: str | None = None) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{name or field} must be an array")
    return value


def require_bool(parent: dict[str, Any], field: str, prefix: str = "feral_scorecard") -> None:
    value = parent.get(field)
    if not isinstance(value, bool):
        raise TypeError(f"{prefix} {field} must be a boolean")


def require_non_negative_int(
    parent: dict[str, Any],
    field: str,
    prefix: str = "feral_scorecard",
) -> None:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise TypeError(f"{prefix} {field} must be a non-negative integer")


def require_number(parent: dict[str, Any], field: str, name: str) -> float:
    value = parent.get(field)
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise TypeError(f"{name} must be a number")
    return float(value)


def require_finite_unit_number(parent: dict[str, Any], field: str, name: str) -> float:
    value = require_number(parent, field, name)
    if not math.isfinite(value) or not 0.0 <= value <= 1.0:
        raise ValueError(f"{name} must be finite and between 0 and 1")
    return value


def require_non_negative_number(parent: dict[str, Any], field: str, name: str) -> None:
    value = require_number(parent, field, name)
    if value < 0.0:
        raise ValueError(f"{name} must be non-negative")


def require_non_empty_string_list(parent: dict[str, Any], field: str, name: str | None = None) -> list[str]:
    label = name or f"feral_scorecard {field}"
    values = require_list(parent, field, label)
    if not values:
        raise ValueError(f"{label} must not be empty")
    require_string_list_values(values, label)
    return values


def require_string_list(parent: dict[str, Any], field: str, name: str) -> None:
    values = require_list(parent, field, name)
    require_string_list_values(values, name)


def require_string_list_values(values: list[Any], name: str) -> None:
    for index, value in enumerate(values):
        if not isinstance(value, str) or not value.strip():
            raise TypeError(f"{name} entry {index} must be a non-empty string")


def require_optional_string_or_null(parent: dict[str, Any], field: str, name: str) -> None:
    value = parent.get(field)
    if value is not None and not isinstance(value, str):
        raise TypeError(f"{name} must be a string or null")


def require_optional_bool_or_null(parent: dict[str, Any], field: str, name: str) -> None:
    value = parent.get(field)
    if value is not None and not isinstance(value, bool):
        raise TypeError(f"{name} must be a boolean or null")


def require_optional_float_or_null(parent: dict[str, Any], field: str, name: str) -> None:
    value = parent.get(field)
    if value is not None and (not isinstance(value, (int, float)) or isinstance(value, bool)):
        raise TypeError(f"{name} must be a number or null")


def require_optional_unit_float_or_null(
    parent: dict[str, Any],
    field: str,
    name: str,
    *,
    require_present: bool,
) -> None:
    if require_present and field not in parent:
        raise TypeError(f"{name} must be present as a number or null")
    value = parent.get(field)
    if value is None:
        return
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise TypeError(f"{name} must be a number or null")
    if not 0.0 <= float(value) <= 1.0:
        raise ValueError(f"{name} must be between 0 and 1")


def require_optional_non_negative_int_or_null(
    parent: dict[str, Any],
    field: str,
    name: str,
) -> None:
    value = parent.get(field)
    if value is not None and (
        not isinstance(value, int) or isinstance(value, bool) or value < 0
    ):
        raise TypeError(f"{name} must be a non-negative integer or null")


if __name__ == "__main__":
    raise SystemExit(main())
