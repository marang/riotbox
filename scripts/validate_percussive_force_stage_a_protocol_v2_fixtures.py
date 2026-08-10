#!/usr/bin/env python3
"""Source-blind mutation fixtures for the RIOTBOX-1430 Protocol-v2 validator."""

from __future__ import annotations

import copy
import inspect
from pathlib import Path
from typing import Any, Callable

import percussive_force_stage_a_v2_contract as v2_contract
import validate_percussive_force_stage_a_protocol_v2 as validator


REPO_ROOT = Path(__file__).resolve().parents[1]


def _expect_fail(name: str, operation: Callable[[], None], token: str) -> None:
    try:
        operation()
    except validator.ContractError as error:
        if token not in str(error):
            raise AssertionError(
                f"{name}: wrong failure {error!s}; expected token {token!r}"
            ) from error
        print(f"PASS mutation {name}: {error}")
        return
    raise AssertionError(f"{name}: mutation unexpectedly validated")


def _f3(document: dict[str, Any]) -> dict[str, Any]:
    return next(
        family
        for family in document["precandidate"]["algorithm_families"]
        if family["family"] == "F3"
    )


def main() -> int:
    frozen = validator.validate_repository(REPO_ROOT)
    protocol_v1, _, _, _ = validator._load_named_json(  # noqa: SLF001
        REPO_ROOT, validator.PROTOCOL_V1_REL
    )
    protocol_v2, payload, raw_sha256, semantic_sha256 = validator._load_named_json(  # noqa: SLF001
        REPO_ROOT, validator.PROTOCOL_V2_REL
    )
    if frozen.raw_sha256 != raw_sha256 or frozen.semantic_sha256 != semantic_sha256:
        raise AssertionError("repository validation did not return the loaded Protocol-v2 pins")
    if payload != v2_contract.render_protocol_v2(
        v2_contract.upgrade_protocol_v1_to_v2(protocol_v1)
    ):
        raise AssertionError("Protocol-v2 bytes are not the deterministic transform output")
    if "protocol_v1" in inspect.signature(
        validator.validate_protocol_v2_document
    ).parameters:
        raise AssertionError("decoded validation must not accept injectable predecessor content")
    try:
        validator.FrozenProtocolV2(  # type: ignore[call-arg]
            path=validator.PROTOCOL_V2_REL,
            schema="riotbox.percussive_force_stage_a_protocol.v2",
            raw_sha256=raw_sha256,
            semantic_sha256=semantic_sha256,
        )
    except TypeError as error:
        if "must be created by validate_repository" not in str(error):
            raise AssertionError(f"sealed constructor raised the wrong error: {error}") from error
    else:
        raise AssertionError("FrozenProtocolV2 constructor is forgeable")

    forged_without_root = object.__new__(validator.FrozenProtocolV2)
    _expect_fail(
        "forged_binding_without_repository_root",
        lambda: validator.revalidate_frozen_protocol_v2(forged_without_root),
        "no repository root",
    )
    forged_payload = validator.FrozenProtocolV2._from_validated(  # noqa: SLF001
        REPO_ROOT,
        b'{"schema":"forged"}\n',
        b'{}\n',
    )
    revalidated_forgery = validator.revalidate_frozen_protocol_v2(forged_payload)
    if revalidated_forgery.raw_sha256 != raw_sha256:
        raise AssertionError("trusted revalidation accepted forged Protocol-v2 bytes")

    cases: list[tuple[str, Callable[[], None], str]] = []

    def add_case(
        name: str,
        mutate: Callable[[dict[str, Any]], None],
        token: str,
    ) -> None:
        def operation() -> None:
            mutated = copy.deepcopy(protocol_v2)
            mutate(mutated)
            validator.validate_protocol_v2_document(mutated, repo_root=REPO_ROOT)

        cases.append((name, operation, token))

    add_case(
        "schema_version_bool_is_not_integer_two",
        lambda value: value.__setitem__("schema_version", True),
        "protocol_v2.schema_version",
    )
    add_case(
        "wrong_owner",
        lambda value: value.__setitem__("owner_ticket", "RIOTBOX-1428"),
        "protocol_v2.owner_ticket",
    )
    add_case(
        "predecessor_raw_pin",
        lambda value: value["predecessor"].__setitem__("raw_sha256", "0" * 64),
        "$.predecessor.raw_sha256",
    )
    add_case(
        "prequalification_component_version",
        lambda value: value["component_versions"].__setitem__(
            "prequalification", "riotbox.percussive_force_prequalification.v2"
        ),
        "component_versions.prequalification",
    )
    add_case(
        "detector_v1_must_stay_unchanged",
        lambda value: value["component_versions"].__setitem__(
            "event_detector", "riotbox.percussive_event_detector.v2"
        ),
        "component_versions.event_detector",
    )
    add_case(
        "source_analysis_schema_v2",
        lambda value: value["component_versions"].__setitem__(
            "source_analysis", "riotbox.percussive_force_source_analysis.v1"
        ),
        "component_versions.source_analysis",
    )
    add_case(
        "bound_catalog_schema_v2",
        lambda value: value["component_versions"].__setitem__(
            "bound_event_catalog",
            "riotbox.percussive_force_stage_a_bound_event_catalog.v1",
        ),
        "component_versions.bound_event_catalog",
    )
    add_case(
        "generic_edge_only_refusal_rejected",
        lambda value: value["prequalification"]["impact_roles"][
            "refusal_reasons"
        ].__setitem__(0, "edge_only_impulse"),
        "first typed refusal must be physical_onset_unresolved",
    )
    add_case(
        "edge_only_cannot_be_an_extra_generic_refusal",
        lambda value: value["prequalification"]["impact_roles"][
            "refusal_reasons"
        ].append("edge_only_impulse"),
        "generic edge_only_impulse is forbidden",
    )
    add_case(
        "physical_onset_refusal_semantics",
        lambda value: value["prequalification"]["event_anatomy"].__setitem__(
            "physical_onset", "otherwise refuse edge_only_impulse"
        ),
        "event_anatomy.physical_onset",
    )
    add_case(
        "historical_edge_only_reinterpretation_forbidden",
        lambda value: value["historical_v1_tombstone"].__setitem__(
            "generic_edge_only_impulse_interpretation_allowed", True
        ),
        "generic_edge_only_impulse_interpretation_allowed",
    )
    add_case(
        "v1_runner_tombstone",
        lambda value: value["historical_v1_tombstone"].__setitem__(
            "runner_state", "reopened_for_protocol_v2"
        ),
        "historical_v1_tombstone.runner_state",
    )
    add_case(
        "authoritative_mean_bits_field",
        lambda value: value["precandidate"]["qualification_wrapper"][
            "authoritative_source_mean_binding"
        ].__setitem__("field", "per_channel_dc_means_decimal"),
        "mean_binding.field",
    )
    add_case(
        "authoritative_mean_bits_representation",
        lambda value: value["precandidate"]["qualification_wrapper"][
            "authoritative_source_mean_binding"
        ].__setitem__("representation", "decimal_json_numbers"),
        "mean_binding.representation",
    )
    add_case(
        "source_mean_reduction_algorithm",
        lambda value: value["prequalification"]["input"][
            "per_channel_dc_mean_reduction"
        ].__setitem__("algorithm_id", "implementation_native_mean"),
        "mean_reduction.algorithm_id",
    )
    add_case(
        "source_mean_rounding_contract",
        lambda value: value["prequalification"]["input"][
            "per_channel_dc_mean_reduction"
        ].__setitem__("binary64_rounding", "implementation default"),
        "mean_reduction.binary64_rounding",
    )
    add_case(
        "source_mean_cancellation_golden",
        lambda value: value["prequalification"]["input"][
            "per_channel_dc_mean_reduction"
        ]["golden"]["per_channel_dc_mean_f64_bits_be_hex"].__setitem__(
            0, "0000000000000000"
        ),
        "mean_reduction.golden",
    )
    add_case(
        "authoritative_mean_identity_complete",
        lambda value: value["precandidate"]["qualification_wrapper"][
            "authoritative_source_mean_binding"
        ]["bound_identity"].pop(),
        "mean_binding.bound_identity",
    )
    add_case(
        "candidate_local_mean_forbidden",
        lambda value: value["precandidate"]["qualification_wrapper"][
            "authoritative_source_mean_binding"
        ].__setitem__("candidate_analysis", "recompute_candidate_local_mean"),
        "mean_binding.candidate_analysis",
    )
    add_case(
        "decimal_mean_is_not_second_authority",
        lambda value: value["precandidate"]["qualification_wrapper"][
            "authoritative_source_mean_binding"
        ].__setitem__("decimal_diagnostic_policy", "decimal_value_is_authoritative"),
        "mean_binding.decimal_diagnostic_policy",
    )
    add_case(
        "f3_conditional_causality_scope",
        lambda value: _f3(value).__setitem__("causal_scope", "streaming_causal"),
        "F3.causal_scope",
    )
    add_case(
        "f3_not_end_to_end_streaming_causal",
        lambda value: _f3(value).__setitem__("end_to_end_streaming_causal", True),
        "F3.end_to_end_streaming_causal",
    )
    add_case(
        "f3_offline_state_complete",
        lambda value: _f3(value)["offline_frozen_state"].pop(),
        "F3.offline_frozen_state",
    )
    add_case(
        "f3_renderer_must_not_recompute_mean",
        lambda value: _f3(value).__setitem__(
            "input_and_provenance", "renderer recomputes its own source mean"
        ),
        "F3.input_and_provenance",
    )
    add_case(
        "f3_causality_claim_must_be_scoped",
        lambda value: _f3(value).__setitem__(
            "phase_safe_envelopes", "the complete system is streaming causal"
        ),
        "F3.phase_safe_envelopes",
    )
    add_case(
        "f3_policy_binds_mean_bits",
        lambda value: _f3(value).__setitem__(
            "resolved_policy_record", "record only decimal channel means"
        ),
        "F3.resolved_policy_record",
    )
    add_case(
        "future_protocol_v3_change_token",
        lambda value: value["numeric_passport_contract"].__setitem__(
            "change_rule_token", v2_contract.OLD_CHANGE_RULE
        ),
        "numeric_passport_contract.change_rule_token",
    )
    add_case(
        "exact_three_band_change_requires_protocol_v3",
        lambda value: value["precandidate"][
            "exact_complementary_three_band_analysis"
        ].__setitem__(
            "version_binding",
            "Any split, DFT, state, filter, or failure-semantic change requires a protocol v2 and an exact_three_band_analysis component bump before recomputation.",
        ),
        "exact_three_band.version_binding",
    )
    add_case(
        "every_passport_uses_future_v3_change_token",
        lambda value: next(iter(value["numeric_passports"].values())).__setitem__(
            "change_rule", v2_contract.OLD_CHANGE_RULE
        ),
        "numeric_passports.input_channel_counts.change_rule",
    )
    add_case(
        "future_protocol_v3_required",
        lambda value: value["change_control"].__setitem__(
            "single_rule", "Protocol v2 may be edited after results"
        ),
        "change_control.single_rule",
    )
    add_case(
        "numeric_retuning_rejected_by_exact_successor",
        lambda value: value["numeric_passports"]["detector_window_ms"].__setitem__(
            "value", 9
        ),
        "$.numeric_passports.detector_window_ms.value",
    )
    add_case(
        "protocol_self_raw_hash_cycle",
        lambda value: value.__setitem__(
            "protocol_v2_raw_sha256", validator.EXPECTED_PROTOCOL_V2_RAW_SHA256
        ),
        "must not contain its own raw or semantic SHA-256",
    )
    add_case(
        "protocol_self_semantic_hash_cycle",
        lambda value: value.__setitem__(
            "protocol_v2_semantic_sha256",
            validator.EXPECTED_PROTOCOL_V2_SEMANTIC_SHA256,
        ),
        "must not contain its own raw or semantic SHA-256",
    )
    add_case(
        "matrix_v3_forward_hash_cycle",
        lambda value: value.__setitem__(
            "matrix_v3_binding",
            {
                "schema": validator.MATRIX_V3_SCHEMA,
                "raw_sha256": "f" * 64,
            },
        ),
        "must not bind a forward Matrix-v3 hash",
    )
    add_case(
        "registry_v3_pin_requires_later_complete_snapshot",
        lambda value: value.__setitem__("registry_v3_raw_sha256", "f" * 64),
        "key set changed",
    )
    add_case(
        "result_injection",
        lambda value: value.__setitem__("gate_result", {"passed": True}),
        "result/evidence fields are forbidden",
    )
    add_case(
        "acquisition_requires_versioned_gate",
        lambda value: value["stage_boundary"]["allowed_now"].__setitem__(
            -1,
            "run one bounded development acquisition",
        ),
        "stage_boundary.allowed_now.acquisition",
    )
    add_case(
        "acquisition_validation_alone_never_authorizes_access",
        lambda value: value["stage_boundary"][
            "batch_acquisition_exception"
        ].__setitem__("authorization", "protocol validation authorizes access"),
        "acquisition.authorization",
    )
    add_case(
        "acquisition_preregistration_schema_is_pinned",
        lambda value: value["stage_boundary"][
            "batch_acquisition_exception"
        ].__setitem__(
            "required_preregistration_schema",
            "riotbox.percussive_force_stage_a_v2_acquisition_batch.v2",
        ),
        "acquisition.required_preregistration_schema",
    )
    add_case(
        "acquisition_access_log_schema_is_pinned",
        lambda value: value["stage_boundary"][
            "batch_acquisition_exception"
        ].__setitem__(
            "required_access_log_schema",
            "riotbox.percussive_force_stage_a_v2_acquisition_access_log.v2",
        ),
        "acquisition.required_access_log_schema",
    )
    add_case(
        "acquisition_batch_size_bool_is_not_three",
        lambda value: value["stage_boundary"][
            "batch_acquisition_exception"
        ].__setitem__("exact_batch_size", True),
        "acquisition.exact_batch_size",
    )
    add_case(
        "acquisition_byte_cap_is_required",
        lambda value: value["stage_boundary"][
            "batch_acquisition_exception"
        ].__setitem__("permitted_access", "strict header validation only"),
        "acquisition.permitted_access",
    )
    add_case(
        "acquisition_directory_discovery_is_forbidden",
        lambda value: value["stage_boundary"][
            "batch_acquisition_exception"
        ].__setitem__("path_and_byte_boundaries", "bounded exact paths"),
        "acquisition.path_and_byte_boundaries",
    )
    add_case(
        "acquisition_publication_is_all_or_nothing",
        lambda value: value["stage_boundary"][
            "batch_acquisition_exception"
        ].__setitem__("publication", "publish successful files immediately"),
        "acquisition.publication",
    )
    add_case(
        "source_playback_is_forbidden_before_freeze",
        lambda value: value["stage_boundary"][
            "forbidden_before_this_contract_and_matrix_validate"
        ].remove("source_preview_or_source_audio_playback"),
        "source_preview_or_source_audio_playback",
    )
    add_case(
        "source_decode_is_forbidden_before_freeze",
        lambda value: value["stage_boundary"][
            "forbidden_before_this_contract_and_matrix_validate"
        ].remove("audio_decode_or_pcm_sample_iteration_outside_strict_header_validation"),
        "audio_decode_or_pcm_sample_iteration_outside_strict_header_validation",
    )
    add_case(
        "source_playback_is_forbidden_during_acquisition",
        lambda value: value["stage_boundary"]["batch_acquisition_exception"][
            "forbidden"
        ].remove("source_preview_or_source_audio_playback"),
        "source_preview_or_source_audio_playback",
    )
    add_case(
        "source_decode_is_forbidden_during_acquisition",
        lambda value: value["stage_boundary"]["batch_acquisition_exception"][
            "forbidden"
        ].remove("audio_decode_or_pcm_sample_iteration"),
        "audio_decode_or_pcm_sample_iteration",
    )

    cases.extend(
        [
            (
                "duplicate_json_key",
                lambda: validator._reject_duplicate_object_keys(  # noqa: SLF001
                    [("schema", "first"), ("schema", "second")]
                ),
                "duplicate JSON object key",
            ),
            (
                "raw_hash_pin",
                lambda: validator.validate_protocol_v2_pins(
                    "0" * 64, validator.EXPECTED_PROTOCOL_V2_SEMANTIC_SHA256
                ),
                "protocol_v2 raw SHA-256",
            ),
            (
                "semantic_hash_pin",
                lambda: validator.validate_protocol_v2_pins(
                    validator.EXPECTED_PROTOCOL_V2_RAW_SHA256, "0" * 64
                ),
                "protocol_v2 semantic SHA-256",
            ),
        ]
    )

    for name, operation, token in cases:
        _expect_fail(name, operation, token)

    print(f"PASS: {len(cases)} fail-closed Protocol-v2 mutation fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
