#!/usr/bin/env python3
"""Build the source-blind RIOTBOX-1430 Stage-A Protocol-v2 contract.

The transform is deliberately deterministic and reads only the immutable v1
JSON contract.  It does not inspect registries, source paths, audio, results,
or generated artifacts.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
from typing import Any


PROTOCOL_V1_REL = Path("docs/benchmarks/percussive_force_stage_a_protocol_v1.json")
PROTOCOL_V2_REL = Path("docs/benchmarks/percussive_force_stage_a_protocol_v2.json")
PROTOCOL_V1_RAW_SHA256 = (
    "35091e697cacb3c187f9a33f4f41ac85aba26832a4214bf3251dfc703edad840"
)
PROTOCOL_V1_SEMANTIC_SHA256 = (
    "7681ab68a9fe2261c97b7499e298e9e6dcb7cbe60df64355dceff577d7fc0848"
)
MATRIX_V2_RAW_SHA256 = (
    "aba846138246c95b1c3e5e1973e77bdaa41ce971f799dadadba8edc160967fd6"
)
MATRIX_V2_SEMANTIC_SHA256 = (
    "57edb217b8dd17166826274d96ef091bd6fd2a88a9688e37b3ef0b7a6d27e94b"
)
REGISTRY_V2_RAW_SHA256 = (
    "af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812"
)
REGISTRY_V2_SEMANTIC_SHA256 = (
    "6cfe11cd10a5947427a09335fbd4795706c71530b6f6a7e5b9883259bcca8ce1"
)
OLD_CHANGE_RULE = (
    "stage_a_protocol_v2_plus_relevant_component_bump_plus_decision_log_before_recompute"
)
NEW_CHANGE_RULE = (
    "stage_a_protocol_v3_plus_relevant_component_bump_plus_decision_log_before_recompute"
)
SOURCE_MEAN_REDUCTION_ALGORITHM = (
    "signed_pcm_code_sum_i128_exact_rational_to_ieee754_binary64_round_ties_to_even_v1"
)
SOURCE_MEAN_REDUCTION_GOLDEN_ID = "pcm24_stereo_cancellation_7_frames_v1"


class ContractBuildError(ValueError):
    """Raised when the immutable v1 input or deterministic transform drifts."""


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, child in pairs:
        if key in value:
            raise ContractBuildError(f"duplicate JSON key: {key!r}")
        value[key] = child
    return value


def semantic_sha256(value: Any) -> str:
    encoded = json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def load_protocol_v1(path: Path) -> dict[str, Any]:
    payload = path.read_bytes()
    actual_raw = hashlib.sha256(payload).hexdigest()
    if actual_raw != PROTOCOL_V1_RAW_SHA256:
        raise ContractBuildError(
            f"Protocol v1 raw SHA-256 drift: expected {PROTOCOL_V1_RAW_SHA256}, "
            f"got {actual_raw}"
        )
    value = json.loads(payload, object_pairs_hook=reject_duplicate_keys)
    if not isinstance(value, dict):
        raise ContractBuildError("Protocol v1 root must be an object")
    actual_semantic = semantic_sha256(value)
    if actual_semantic != PROTOCOL_V1_SEMANTIC_SHA256:
        raise ContractBuildError(
            "Protocol v1 semantic SHA-256 drift: "
            f"expected {PROTOCOL_V1_SEMANTIC_SHA256}, got {actual_semantic}"
        )
    return value


def _replace_change_rule_tokens(value: Any) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if key == "change_rule" and child == OLD_CHANGE_RULE:
                value[key] = NEW_CHANGE_RULE
            else:
                _replace_change_rule_tokens(child)
    elif isinstance(value, list):
        for child in value:
            _replace_change_rule_tokens(child)


def _f3_contract(document: dict[str, Any]) -> dict[str, Any]:
    families = document["precandidate"]["algorithm_families"]
    matches = [family for family in families if family.get("family") == "F3"]
    if len(matches) != 1:
        raise ContractBuildError("Protocol v1 must contain exactly one F3 family")
    return matches[0]


def upgrade_protocol_v1_to_v2(protocol_v1: dict[str, Any]) -> dict[str, Any]:
    """Return the exact source-blind Protocol-v2 successor document."""

    document = copy.deepcopy(protocol_v1)
    document["schema"] = "riotbox.percussive_force_stage_a_protocol.v2"
    document["schema_version"] = 2
    document["owner_ticket"] = "RIOTBOX-1430"
    document["work_class"] = "contract_enabler"
    document["execution_state"] = (
        "preregistered_no_v2_source_qualification_or_candidate_render"
    )
    document["evidence_role"] = "machine_validated_stage_a_v2_retry_preregistration"
    document["directly_enabled_outcome"] = (
        "Enable exactly one fresh RIOTBOX-1428 development-only StageAQualificationSession "
        "against the complete frozen Protocol-v2, Registry-v3, and Matrix-v3 stack; "
        "the three-family candidate matrix remains forbidden unless all four positive "
        "sources pass mechanism-blind admission."
    )
    document["version_decision"] = "RBX-254"
    document["predecessor"] = {
        "path": PROTOCOL_V1_REL.as_posix(),
        "schema": "riotbox.percussive_force_stage_a_protocol.v1",
        "raw_sha256": PROTOCOL_V1_RAW_SHA256,
        "semantic_sha256": PROTOCOL_V1_SEMANTIC_SHA256,
        "state": "immutable_historical_rejected_execution_only",
    }
    document["historical_v1_tombstone"] = {
        "qualification_session_id": "2f1e5ba2-ca1e-42b4-b0e1-3faa7591dde9",
        "qualification_outcome": "positive_source_failed",
        "executed_implementation_commit": (
            "c60cbb392491950fdbb2edaf15a9f8926db51c71"
        ),
        "executed_implementation_aggregate_sha256": (
            "8285107a8c396c7fcbfd52cbe24e3dc8c3a108c56a57487cdb178977a5b2de94"
        ),
        "matrix_v2_raw_sha256": MATRIX_V2_RAW_SHA256,
        "matrix_v2_semantic_sha256": MATRIX_V2_SEMANTIC_SHA256,
        "registry_v2_raw_sha256": REGISTRY_V2_RAW_SHA256,
        "registry_v2_semantic_sha256": REGISTRY_V2_SEMANTIC_SHA256,
        "generic_edge_only_impulse_interpretation_allowed": False,
        "runner_state": "stage_a_v1_execution_closed_by_rbx_254",
    }

    boundary = document["stage_boundary"]
    boundary["allowed_now"] = [
        "validate_this_protocol_v2_json_contract",
        "validate_a_complete_metadata_only_registry_v3_and_matrix_v3_snapshot",
        "implement_and_validate_source_blind_v2_artifact_schemas",
        "run_one_exact_predeclared_three_file_development_acquisition_and_header_format_session_before_registry_v3_freeze_only_after_the_versioned_acquisition_gate_validates",
    ]
    boundary["batch_acquisition_exception"] = {
        "purpose": "identity_hash_format_and_source_suitability_pre_admission_only",
        "authorization": (
            "protocol_v2_validation_alone_never_authorizes_network_or_source_file_access"
        ),
        "required_preregistration_schema": (
            "riotbox.percussive_force_stage_a_v2_acquisition_batch.v1"
        ),
        "required_access_log_schema": (
            "riotbox.percussive_force_stage_a_v2_acquisition_access_log.v1"
        ),
        "exact_batch_size": 3,
        "required_before_network_bytes": (
            "one_complete_validator_pinned_metadata_only_three_source_batch_with_exact_page "
            "download attachment_byte_count case author pack family and destination identities"
        ),
        "permitted_access": (
            "one_all_or_nothing_no_redirect_session_over_exact_declared_https_urls_and_exact "
            "repo_relative_development_destinations_only; stream_at_most_each_declared_attachment "
            "byte_count_plus_one_for_sha256_and_strict_RIFF_PCM_header_validation_without_sample "
            "decoding_iteration_or_analysis"
        ),
        "path_and_byte_boundaries": (
            "the_validator_pinned_preregistration_is_the_only_authority_for_URL_order_destination "
            "and_exact_attachment_bytes; no directory discovery wildcard retry substitute "
            "redirect partial survivor or undeclared response byte is permitted"
        ),
        "publication": (
            "publish_no_final_destination_until_all_three_exact_downloads_and_headers_validate"
        ),
        "forbidden": [
            "feature_or_event_computation",
            "audio_decode_or_pcm_sample_iteration",
            "source_preview_or_source_audio_playback",
            "candidate_or_control_rendering",
            "holdout_audio_access",
            "commercial_reference_access",
            "sequential_survivor_selection",
        ],
        "failure": "reject_entire_batch_and_require_a_new_versioned_metadata_decision",
    }
    boundary["forbidden_before_this_contract_and_matrix_validate"] = [
        "source_feature_computation",
        "unregistered_or_nonpredeclared_wav_or_pcm_reading",
        "source_event_qualification",
        "candidate_or_control_rendering",
        "generated_audio_artifacts",
        "human_candidate_playback",
        "source_preview_or_source_audio_playback",
        "audio_decode_or_pcm_sample_iteration_outside_strict_header_validation",
        "holdout_audio_access",
    ]

    document["change_control"]["single_rule"] = (
        "Any overall contract value, equation, comparator, topology, role, partition, "
        "or stop-rule change requires riotbox.percussive_force_stage_a_protocol.v3 "
        "plus the relevant component-version bump and a versioned decision-log entry "
        "before any recomputation; it invalidates all earlier Stage-A-v2 results and "
        "is forbidden after the first candidate render, human candidate evidence, or "
        "holdout access."
    )
    document["change_control"]["prequalification_specific_rule"] = (
        "Any detector, anatomy, rhythmic-proxy, source-contrast, ordinal, refusal, or "
        "authoritative source-mean binding change additionally bumps "
        "riotbox.percussive_force_prequalification.v3 to v4."
    )

    exact_three_band = document["precandidate"][
        "exact_complementary_three_band_analysis"
    ]
    exact_three_band["version_binding"] = (
        "Any split, DFT, state, filter, or failure-semantic change requires a protocol v3 "
        "and an exact_three_band_analysis component bump before recomputation."
    )

    versions = document["component_versions"]
    versions["prequalification"] = "riotbox.percussive_force_prequalification.v3"
    versions["impact_role"] = "riotbox.impact_role.v2"
    versions["event_anatomy"] = "riotbox.percussive_event_anatomy.v2"
    versions["source_analysis"] = "riotbox.percussive_force_source_analysis.v2"
    versions["unbound_qualification_analysis"] = (
        "riotbox.percussive_force_stage_a_unbound_qualification_analysis.v2"
    )
    versions["bound_event_catalog"] = (
        "riotbox.percussive_force_stage_a_bound_event_catalog.v2"
    )
    versions["qualification_rejection"] = (
        "riotbox.percussive_force_stage_a_qualification_rejection.v2"
    )
    versions["qualification_session"] = (
        "riotbox.percussive_force_stage_a_qualification_session.v2"
    )
    versions["qualification_commit"] = (
        "riotbox.percussive_force_stage_a_qualification_commit.v1"
    )

    passport_contract = document["numeric_passport_contract"]
    passport_contract["change_rule_token"] = NEW_CHANGE_RULE
    _replace_change_rule_tokens(document["numeric_passports"])

    refusal_reasons = document["prequalification"]["impact_roles"]["refusal_reasons"]
    if refusal_reasons[0] != "edge_only_impulse":
        raise ContractBuildError("Protocol v1 refusal order drifted")
    refusal_reasons[0] = "physical_onset_unresolved"
    anatomy = document["prequalification"]["event_anatomy"]
    anatomy["physical_onset"] = (
        "Choose the earliest s whose peak_s is at least anatomy_peak_baseline_ratio*b_s "
        "and minimum_signal_peak_lsb*input_lsb and whose R1[s]>=b_s+"
        "onset_fraction_above_baseline*(peak_s-b_s) persists at s and the following "
        "N(onset_persistence_ms)-1 values. Freeze that s, b_s, and peak_s; if no tested "
        "s passes the complete baseline, peak, signal-floor, and persistence gate, "
        "refuse physical_onset_unresolved without claiming edge-only anatomy."
    )
    source_input = document["prequalification"]["input"]
    source_input["analysis_signal"] = (
        "For each channel c, mu_c is the authoritative binary64 result of "
        "per_channel_dc_mean_reduction over every sample of the complete registered source; "
        "analysis-only x_prime[c,n]=x[c,n]-mu_c, instantaneous phase-safe power "
        "p[n]=(1/C)*sum_c(x_prime[c,n]^2), and instantaneous phase-safe magnitude "
        "q[n]=sqrt(p[n]). STFT and Welch sum per-channel powers of Hann-windowed x_prime "
        "and never mono-sum channels."
    )
    source_input["per_channel_dc_mean_reduction"] = {
        "algorithm_id": SOURCE_MEAN_REDUCTION_ALGORITHM,
        "decode": (
            "Decode each verified PCM16 or PCM24 sample to its exact signed integer code; "
            "do not convert individual samples to floating point before accumulation."
        ),
        "accumulation": (
            "For each channel in channel order, initialize signed i128 S_c=0 and checked-add "
            "codes in strictly ascending frame index. Any i128 overflow is a typed refusal."
        ),
        "exact_rational": "mu_c_exact=S_c/(frame_count*2^(valid_bits-1))",
        "binary64_rounding": (
            "Convert mu_c_exact exactly once to finite IEEE-754 binary64 using roundTiesToEven; "
            "no intermediate binary floating operation, reassociation, pairwise or compensated "
            "sum, extended precision, fused operation, or implementation-native mean is allowed. "
            "An exact zero numerator serializes canonical positive zero."
        ),
        "serialization": (
            "Serialize the resulting binary64 bit pattern as exactly 16 lowercase big-endian "
            "hexadecimal characters per channel in verified channel order."
        ),
        "golden": {
            "golden_id": SOURCE_MEAN_REDUCTION_GOLDEN_ID,
            "pcm_encoding": "signed_little_endian_integer_pcm24",
            "valid_bits": 24,
            "frame_count": 7,
            "channel_codes_in_frame_order": [
                [8_388_607, -8_388_608, 8_388_607, -8_388_608, 3, -2, 2],
                [8_388_607, -8_388_608, 8_388_607, -8_388_608, 3, -2, 0],
            ],
            "exact_signed_code_sums": [1, -1],
            "per_channel_dc_mean_f64_bits_be_hex": [
                "3e52492492492492",
                "be52492492492492",
            ],
        },
        "failure": "typed_refusal_no_source_analysis_no_event_catalog_no_fallback",
    }

    wrapper = document["precandidate"]["qualification_wrapper"]
    wrapper["required_bindings"][0] = (
        "exact source-registry v3 path schema and raw SHA-256 pinned by Matrix v3"
    )
    wrapper["authoritative_source_mean_binding"] = {
        "field": "per_channel_dc_mean_f64_bits_be_hex",
        "representation": (
            "one_16_character_lowercase_f64_bit_pattern_per_verified_channel_in_channel_order"
        ),
        "reduction_algorithm_id": SOURCE_MEAN_REDUCTION_ALGORITHM,
        "reduction_golden_id": SOURCE_MEAN_REDUCTION_GOLDEN_ID,
        "bound_identity": [
            "protocol_raw_sha256",
            "source_sha256",
            "pcm_encoding",
            "valid_bits",
            "sample_rate_hz",
            "channel_count",
            "frame_count",
        ],
        "catalog_ownership": (
            "bound_event_catalog_v2_carries_the_source_analysis_v2_bits_and_renderer_decodes_them_exactly"
        ),
        "candidate_analysis": "subtract_the_same_source_frozen_means_never_candidate_local_means",
        "decimal_diagnostic_policy": (
            "optional_only_if_reencoding_matches_authoritative_bits_never_a_second_authority"
        ),
        "failure": "typed_refusal_no_candidate_audio_no_fallback",
    }

    f3 = _f3_contract(document)
    f3["causal_scope"] = "conditional_on_source_frozen_offline_state"
    f3["end_to_end_streaming_causal"] = False
    f3["offline_frozen_state"] = [
        "whole_source_dc_means",
        "event_anatomy",
        "attack_body_masks",
    ]
    f3["input_and_provenance"] = (
        "Use frozen raw PCM x, event anatomy, masks, lookbehind, and the exact source-analysis-v2 "
        "per-channel whole-source DC mean f64 bit patterns produced only by "
        f"{SOURCE_MEAN_REDUCTION_ALGORITHM} plus registry encoding and valid-bits "
        "LSB binding. The renderer decodes those exact channel-ordered bits and never recomputes "
        "a source or candidate mean. Signed PCM16 means input_lsb=2^-15; signed PCM24 means "
        "input_lsb=2^-23. The resolved policy binds protocol, source, format, sample-rate, channel, "
        "and frame-count identity. Missing, malformed, nonfinite, reordered, mismatched, or "
        "recomputed mean evidence refuses with no candidate audio or fallback."
    )
    f3["phase_safe_envelopes"] = (
        "Conditional on the frozen offline means, anatomy, and masks: on source-mean-subtracted "
        "x_prime, p[n]=(1/C)*sum_c(x_prime[c,n]^2), q[n]=sqrt(p[n]), and R_t[n]=sqrt(mean(q^2)) "
        "over the complete causal right-aligned N(t)-frame window. N(t)=max(1,floor(fs*t/1000+"
        "frame_rounding_offset)). Require complete R20 at physical_onset. This is not an "
        "end-to-end streaming-causality claim."
    )
    f3["resolved_policy_record"] = (
        "Record sample_rate_hz, channel_count, registry/LSB provenance, authoritative channel-ordered "
        "per_channel_dc_mean_f64_bits_be_hex, b, F, N1/N8/N20 and alpha values, source weighted MS "
        "values and floor, masks and regions, cA/cB, exact provenance-only controller hashes, "
        "controller implementation hash, ordinary [sA,sB] resolved-policy hash, and the F3 "
        "source-response raw eight-vector, quantized eight-vector, and actionable-diversity hash. "
        "Filename, title, and path never enter policy selection, controller provenance hashes, or "
        "actionable diversity identity."
    )
    return document


def render_protocol_v2(document: dict[str, Any]) -> bytes:
    return (
        json.dumps(document, indent=2, ensure_ascii=False, allow_nan=False) + "\n"
    ).encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    document = upgrade_protocol_v1_to_v2(load_protocol_v1(repo / PROTOCOL_V1_REL))
    payload = render_protocol_v2(document)
    output = repo / PROTOCOL_V2_REL
    if args.check:
        if not output.exists() or output.read_bytes() != payload:
            raise ContractBuildError(f"{PROTOCOL_V2_REL} is missing or drifted")
    else:
        output.write_bytes(payload)
    print(f"protocol_v2_raw_sha256={hashlib.sha256(payload).hexdigest()}")
    print(f"protocol_v2_semantic_sha256={semantic_sha256(document)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
