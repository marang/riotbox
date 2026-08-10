#!/usr/bin/env python3
"""Mutation fixtures for the RIOTBOX-1428 metadata-only preregistration.

The fixtures import the validator and mutate decoded JSON in memory. They never
open, enumerate, hash, render, classify, or play a source/holdout audio file.
"""

from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path
from typing import Any, Callable


REPO_ROOT = Path(__file__).resolve().parents[1]
VALIDATOR_PATH = REPO_ROOT / "scripts/validate_percussive_force_stage_a_protocol.py"
SPEC = importlib.util.spec_from_file_location("stage_a_validator", VALIDATOR_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot import Stage-A validator")
VALIDATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATOR)


def _load(relative: Path) -> tuple[dict[str, Any], str]:
    allowed = {
        VALIDATOR.PROTOCOL_REL,
        VALIDATOR.MATRIX_REL,
        VALIDATOR.MATRIX_V1_REL,
        VALIDATOR.REGISTRY_V1_REL,
        VALIDATOR.REGISTRY_V2_REL,
    }
    if relative not in allowed:
        raise AssertionError(f"fixture attempted non-contract read: {relative}")
    raw = (REPO_ROOT / relative).read_bytes()
    return json.loads(raw), VALIDATOR._sha256(raw)


PROTOCOL, PROTOCOL_SHA = _load(VALIDATOR.PROTOCOL_REL)
MATRIX, _MATRIX_SHA = _load(VALIDATOR.MATRIX_REL)
_MATRIX_V1, MATRIX_V1_SHA = _load(VALIDATOR.MATRIX_V1_REL)
REGISTRY_V1, REGISTRY_V1_SHA = _load(VALIDATOR.REGISTRY_V1_REL)
REGISTRY_V2, REGISTRY_V2_SHA = _load(VALIDATOR.REGISTRY_V2_REL)

# During preregistration authoring the three new canonical pins are placeholders.
# Exercise the exact same semantics in memory; once pinned this block is inert.
_UNPINNED_AUTHORING = False
if VALIDATOR.EXPECTED_PROTOCOL_SHA256.startswith("__"):
    _UNPINNED_AUTHORING = True
    VALIDATOR.EXPECTED_PROTOCOL_SHA256 = PROTOCOL_SHA
    MATRIX["protocol"]["sha256"] = PROTOCOL_SHA
if VALIDATOR.EXPECTED_MATRIX_V2_SHA256.startswith("__"):
    _UNPINNED_AUTHORING = True
    VALIDATOR.EXPECTED_MATRIX_V2_SHA256 = _MATRIX_SHA
if VALIDATOR.EXPECTED_REGISTRY_V2_SHA256.startswith("__"):
    _UNPINNED_AUTHORING = True
    VALIDATOR.EXPECTED_REGISTRY_V2_SHA256 = REGISTRY_V2_SHA
    MATRIX["source_registry"]["sha256"] = REGISTRY_V2_SHA
if _UNPINNED_AUTHORING:
    VALIDATOR.EXPECTED_MATRIX_V2_SEMANTIC_SHA256 = VALIDATOR._semantic_sha256(MATRIX)


def _validate_matrix(matrix: dict[str, Any], registry_v2: dict[str, Any] | None = None) -> None:
    VALIDATOR.validate_matrix(
        matrix,
        protocol_sha256=PROTOCOL_SHA,
        matrix_v1_sha256=MATRIX_V1_SHA,
        registry_v1=REGISTRY_V1,
        registry_v1_sha256=REGISTRY_V1_SHA,
        registry_v2=REGISTRY_V2 if registry_v2 is None else registry_v2,
        registry_v2_sha256=REGISTRY_V2_SHA,
    )


def _expect_fail(name: str, operation: Callable[[], None], token: str) -> None:
    try:
        operation()
    except VALIDATOR.ContractError as exc:
        if token not in str(exc):
            raise AssertionError(f"{name}: wrong failure {exc!s}; expected token {token!r}") from exc
        print(f"PASS mutation {name}: {exc}")
        return
    raise AssertionError(f"{name}: mutation unexpectedly validated")


def _mutated_protocol(mutator: Callable[[dict[str, Any]], None]) -> dict[str, Any]:
    value = copy.deepcopy(PROTOCOL)
    mutator(value)
    return value


def _mutated_matrix(mutator: Callable[[dict[str, Any]], None]) -> dict[str, Any]:
    value = copy.deepcopy(MATRIX)
    mutator(value)
    return value


def main() -> int:
    # Baselines establish that every mutation starts from the same valid decoded
    # contracts. Canonical-byte pins are exercised by the repository validator.
    VALIDATOR.validate_protocol(PROTOCOL)
    _validate_matrix(MATRIX)

    cases: list[tuple[str, Callable[[], None], str]] = []

    def add_protocol_case(
        name: str, mutator: Callable[[dict[str, Any]], None], token: str
    ) -> None:
        cases.append(
            (
                name,
                lambda mutator=mutator: VALIDATOR.validate_protocol(
                    _mutated_protocol(mutator)
                ),
                token,
            )
        )

    def add_matrix_case(
        name: str, mutator: Callable[[dict[str, Any]], None], token: str
    ) -> None:
        cases.append(
            (
                name,
                lambda mutator=mutator: _validate_matrix(_mutated_matrix(mutator)),
                token,
            )
        )

    add_matrix_case(
        "positive_source_sha_pin",
        lambda value: value["positive_sources"][0].__setitem__("sha256", "0" * 64),
        "positive_sources.oga_cinameng_can_be_so_beautiful.sha256",
    )
    add_matrix_case(
        "protocol_hash_pin",
        lambda value: value["protocol"].__setitem__("sha256", "0" * 64),
        "matrix.protocol",
    )
    add_protocol_case(
        "detector_threshold_equation",
        lambda value: value["prequalification"]["event_detector"].__setitem__(
            "threshold", "N >= median"
        ),
        "detector.threshold",
    )
    add_protocol_case(
        "detector_window_convention",
        lambda value: value["prequalification"]["event_detector"].__setitem__(
            "window", "symmetric Hann"
        ),
        "detector.window",
    )
    add_protocol_case(
        "passport_comparator_cannot_accept_everything",
        lambda value: value["numeric_passports"]["output_peak_absolute_max"].__setitem__(
            "comparator_or_formula_role", "accept_everything"
        ),
        "output_peak_absolute_max.metadata",
    )
    add_protocol_case(
        "protocol_top_level_result_injection",
        lambda value: value.__setitem__(
            "actual", {"human_verdict": "pass", "unsafe_output": "okay"}
        ),
        "$.actual",
    )
    add_protocol_case(
        "contradictory_output_gate_prose",
        lambda value: value["precandidate"]["qualification_wrapper"].__setitem__(
            "output_gate",
            value["precandidate"]["qualification_wrapper"]["output_gate"]
            + " Do not require strict abs_peak; unsafe output is okay.",
        ),
        "qualification_wrapper.output_gate",
    )
    add_protocol_case(
        "contradictory_format_screen_prose",
        lambda value: value["precandidate"]["reject_only_mechanical_screens"].__setitem__(
            "format_and_safety",
            value["precandidate"]["reject_only_mechanical_screens"]["format_and_safety"]
            + " Clipping may pass.",
        ),
        "screens.format_and_safety",
    )
    add_protocol_case(
        "type_strict_bool_is_not_integer_one",
        lambda value: value["numeric_passports"]["f3_v2_branch_scale"].__setitem__(
            "value", True
        ),
        "f3_v2_branch_scale.value",
    )
    add_protocol_case(
        "type_strict_float_is_not_integer_four",
        lambda value: value["numeric_passports"]["detector_log_energy_lag_hops"].__setitem__(
            "value", 4.0
        ),
        "detector_log_energy_lag_hops.value",
    )
    add_protocol_case(
        "anatomy_non_circular_onset",
        lambda value: value["prequalification"]["event_anatomy"].__setitem__(
            "physical_onset", "choose a convenient peak"
        ),
        "anatomy.physical_onset",
    )
    add_protocol_case(
        "cluster_gate_threshold",
        lambda value: value["numeric_passports"]["minimum_source_clusters"].__setitem__(
            "value", 2
        ),
        "minimum_source_clusters.value",
    )
    add_protocol_case(
        "unique_partition_rule",
        lambda value: value["prequalification"]["source_contrast"].__setitem__(
            "partitioning", "take the first convenient clustering"
        ),
        "source_contrast.partitioning",
    )
    add_protocol_case(
        "f1_conservation_equation",
        lambda value: value["precandidate"]["algorithm_families"][0].__setitem__(
            "resolver", "gA=2; gB=1"
        ),
        "F1.resolver",
    )
    add_protocol_case(
        "f2_strict_band_trust",
        lambda value: value["precandidate"]["algorithm_families"][1].__setitem__(
            "trusted_band_rule", "all bands trusted"
        ),
        "F2.trusted_band_rule",
    )
    add_protocol_case(
        "f2_reconstruction_preflight",
        lambda value: value["precandidate"]["algorithm_families"][1][
            "preflight"
        ].__setitem__("signals", ["impulse"]),
        "F2.preflight.signals",
    )
    add_protocol_case(
        "f3_v2_attack_time_constant",
        lambda value: value["numeric_passports"]["f3_v2_attack_up_ms"].__setitem__(
            "value", 2
        ),
        "f3_v2_attack_up_ms.value",
    )
    add_protocol_case(
        "f3_exact_M64_preflight",
        lambda value: value["precandidate"]["algorithm_families"][2][
            "preflight"
        ].__setitem__("frame_helper", "round however the library chooses"),
        "F3.preflight.frame_helper",
    )
    add_protocol_case(
        "f3_failed_probe_is_immutable",
        lambda value: value["immutable_source_independent_preflight_records"][0].__setitem__(
            "measured_error_db", -60.0
        ),
        "f3 immutable rejected record",
    )
    add_protocol_case(
        "f3_branch_mean_square_units",
        lambda value: value["precandidate"]["algorithm_families"][2].__setitem__(
            "branch_contribution", "sum residual samples without a mean-square denominator"
        ),
        "F3.branch_contribution",
    )
    add_protocol_case(
        "f3_controller_hashes_are_provenance_only",
        lambda value: value["precandidate"]["algorithm_families"][2][
            "controller_hashes"
        ].__setitem__("evidence_role", "actionable_diversity"),
        "F3.controller_hash.evidence_role",
    )
    add_protocol_case(
        "f3_source_response_component_binding",
        lambda value: value["precandidate"]["algorithm_families"][2].__setitem__(
            "source_response_diversity_component",
            "riotbox.f3_source_response_diversity.unversioned",
        ),
        "F3.source_response_diversity_component",
    )
    add_protocol_case(
        "f3_preflight_actionable_identity_invariance",
        lambda value: value["precandidate"]["algorithm_families"][2][
            "preflight"
        ].__setitem__(
            "source_response_diversity_identity_v1",
            "permit different identities per transform and sample rate",
        ),
        "F3.preflight.source_response_diversity_identity",
    )
    add_protocol_case(
        "f3_expected_refusal_has_no_output_hash",
        lambda value: value["precandidate"]["algorithm_families"][2][
            "preflight"
        ].__setitem__("signal_common", "every run must hash a rendered output"),
        "F3.preflight.signal_common",
    )
    add_protocol_case(
        "f3_preflight_q_goldens_are_frozen",
        lambda value: value["numeric_passports"][
            "f3_source_response_preflight_q_goldens"
        ]["value"]["44100"].__setitem__(7, 3),
        "f3_source_response_preflight_q_goldens.value",
    )
    add_protocol_case(
        "f3_preflight_identity_hash_goldens_are_derived",
        lambda value: value["precandidate"]["algorithm_families"][2][
            "preflight"
        ]["source_response_diversity_hash_goldens_v1"].__setitem__(
            "44100", "f" * 64
        ),
        "F3.preflight.source_response_diversity_hash_goldens",
    )
    add_protocol_case(
        "f3_v2_combined_Q_is_not_a_direction_gate",
        lambda value: value["precandidate"]["algorithm_families"][2][
            "family_falsifiers"
        ].__setitem__("direction", "require combined Q_attack and Q_body increases"),
        "F3.direction",
    )
    add_protocol_case(
        "control_cannot_earn_force",
        lambda value: value["precandidate"]["false_positive_controls"][1].__setitem__(
            "can_earn_force", True
        ),
        "controls.can_earn_force",
    )
    add_protocol_case(
        "control_exact_id_set",
        lambda value: value["precandidate"]["false_positive_controls"][2].__setitem__(
            "control_id", "rate_control_unversioned"
        ),
        "controls.ids",
    )
    add_protocol_case(
        "global_rate_control_exact_rational_renderer",
        lambda value: value["precandidate"]["false_positive_controls"][2].__setitem__(
            "renderer", "generic resampler at 0.8 speed"
        ),
        "controls.rate.renderer",
    )
    add_protocol_case(
        "detached_click_uses_tail_mask",
        lambda value: value["precandidate"]["false_positive_controls"][7].__setitem__(
            "renderer", "attenuate wB only"
        ),
        "controls.detached.renderer",
    )
    add_protocol_case(
        "attenuation_only_matcher",
        lambda value: value["precandidate"]["level_matcher"].__setitem__(
            "target", "boost quieter side"
        ),
        "matcher.target",
    )
    add_protocol_case(
        "blind_reversed_order",
        lambda value: value["precandidate"]["blinding"].__setitem__(
            "reversed_orientation", "hash again and permit the same orientation"
        ),
        "blinding.reversed_orientation",
    )
    add_protocol_case(
        "boundary_discontinuity_definition",
        lambda value: value["precandidate"]["mechanical_metric_definitions"].__setitem__(
            "boundary_discontinuity", "ignore edit boundaries"
        ),
        "metrics.boundary_discontinuity",
    )
    add_protocol_case(
        "policy_diversity_excludes_anatomy_masks",
        lambda value: value["precandidate"]["anti_hardcoding"]["resolved_policy_hash"][
            "f1_field_order"
        ].append("wA_mask_shape"),
        "anti.policy_hash.f1",
    )
    add_protocol_case(
        "f3_ordinary_policy_excludes_region_controller_hashes",
        lambda value: value["precandidate"]["anti_hardcoding"][
            "resolved_policy_hash"
        ]["f3_field_order"].insert(0, "a0_sha256_raw_32_bytes"),
        "anti.policy_hash.f3",
    )
    add_protocol_case(
        "f3_diagnostic_measurement_exclusion_is_typed",
        lambda value: value["precandidate"]["anti_hardcoding"][
            "resolved_policy_hash"
        ]["excluded_fields"].__setitem__(11, "measurements"),
        "anti.policy_hash.excluded_fields",
    )
    add_protocol_case(
        "f3_response_horizon_excludes_anatomy",
        lambda value: value["precandidate"]["anti_hardcoding"][
            "f3_source_response_diversity"
        ].__setitem__("horizon", "normalize across [physical_onset,body_end)"),
        "anti.f3_response.horizon",
    )
    add_protocol_case(
        "f3_response_raw_field_order",
        lambda value: value["precandidate"]["anti_hardcoding"][
            "f3_source_response_diversity"
        ]["raw_field_order"].reverse(),
        "anti.f3_response.raw_field_order",
    )
    add_protocol_case(
        "f3_response_quantization_is_frozen",
        lambda value: value["precandidate"]["anti_hardcoding"][
            "f3_source_response_diversity"
        ].__setitem__("quantization", "hash exact unquantized controller floats"),
        "anti.f3_response.quantization",
    )
    add_protocol_case(
        "f3_response_preimage_excludes_anatomy",
        lambda value: value["precandidate"]["anti_hardcoding"][
            "f3_source_response_diversity"
        ].__setitem__("preimage", "domain||body_end||array_length||controller_hashes"),
        "anti.f3_response.preimage",
    )
    add_protocol_case(
        "f3_response_uses_full_versioned_family_id",
        lambda value: value["precandidate"]["anti_hardcoding"][
            "f3_source_response_diversity"
        ].__setitem__("family_id", "F3"),
        "anti.f3_response.family_id",
    )
    add_protocol_case(
        "f3_actionable_identity_mapping",
        lambda value: value["precandidate"]["anti_hardcoding"][
            "actionable_diversity_identity"
        ].__setitem__("F3", "resolved_policy_hash"),
        "anti.actionable_diversity_identity",
    )
    add_protocol_case(
        "f3_robust_quantized_separation",
        lambda value: value["numeric_passports"][
            "f3_source_response_minimum_quantized_component_distance"
        ].__setitem__("value", 1),
        "f3_source_response_minimum_quantized_component_distance.value",
    )
    add_protocol_case(
        "global_rate_uses_typed_provenance",
        lambda value: value["precandidate"]["mechanical_metric_definitions"].__setitem__(
            "global_rate_or_pitch", "estimate pitch from signal"
        ),
        "metrics.global_rate_or_pitch",
    )
    add_protocol_case(
        "onset_integrity_gate",
        lambda value: value["numeric_passports"]["candidate_onset_tolerance_ms"].__setitem__(
            "value", 2.0
        ),
        "candidate_onset_tolerance_ms.value",
    )
    add_protocol_case(
        "rhythmic_proxy_gate",
        lambda value: value["numeric_passports"][
            "candidate_rhythmic_proxy_tolerance_ms"
        ].__setitem__("value", 10.0),
        "candidate_rhythmic_proxy_tolerance_ms.value",
    )
    add_protocol_case(
        "promotion_desirable_to_trigger",
        lambda value: value["precandidate"]["human_promotion_gate"][
            "required_directional_fields"
        ].__setitem__("desirable_to_trigger", False),
        "desirable_to_trigger",
    )
    add_matrix_case(
        "result_injection",
        lambda value: value.__setitem__("gate_result", {"passed": True}),
        "gate_result",
    )
    add_matrix_case(
        "active_f3_version_binding",
        lambda value: value["required_cross_product"]["family_versions"].__setitem__(
            2, "f3_os4_onset_residual_v1"
        ),
        "matrix.cross.family_versions",
    )
    add_matrix_case(
        "f3_actionable_diversity_component_binding",
        lambda value: value["required_cross_product"].__setitem__(
            "f3_actionable_diversity_component",
            "riotbox.f3_source_response_diversity.unversioned",
        ),
        "matrix.cross.f3_actionable_diversity_component",
    )

    cases.append(
        (
            "duplicate_json_key_rejected",
            lambda: VALIDATOR._reject_duplicate_object_keys(
                [("failure", "first"), ("failure", "second")]
            ),
            "JSON duplicate object key",
        )
    )
    add_matrix_case(
        "matrix_holdout_mutation",
        lambda value: value["active_holdout_union"][0].__setitem__(
            "sha256", "f" * 64
        ),
        "active_holdout_union",
    )

    def mutated_registry_case() -> None:
        registry = copy.deepcopy(REGISTRY_V2)
        for entry in registry["entries"]:
            if entry["case_id"] == "oga_ruok_160bpm":
                entry["source_path"] = "data/not-the-frozen-holdout.wav"
                break
        _validate_matrix(MATRIX, registry)

    cases.append(
        (
            "registry_holdout_mutation",
            mutated_registry_case,
            "registry v2 active holdout metadata",
        )
    )

    add_matrix_case(
        "broader_readiness_overclaim",
        lambda value: value["readiness_scope"].__setitem__(
            "broader_floor_claimed", True
        ),
        "broader_floor_claimed",
    )

    for name, operation, token in cases:
        _expect_fail(name, operation, token)

    print(f"PASS: {len(cases)} fail-closed Stage-A mutation fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
