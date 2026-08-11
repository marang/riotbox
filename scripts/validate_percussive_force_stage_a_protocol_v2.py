#!/usr/bin/env python3
"""Validate the source-blind RIOTBOX-1430 Stage-A Protocol-v2 successor.

The historical Protocol-v1/Matrix-v2/Registry-v1-v2 validator remains the
authority for the executed RIOTBOX-1428 stack.  This validator runs that stack
first, then proves that Protocol v2 is exactly the deterministic
``upgrade_protocol_v1_to_v2`` successor.  It reads only the two named protocol
JSON files and the named historical contracts read by the legacy validator; it
never discovers or opens source, holdout, reference, or generated audio.

Registry v3 and Matrix v3 deliberately have no hashes here.  Their future
validators can consume ``FrozenProtocolV2`` after their complete contracts are
known, without changing or weakening the historical validator.
"""

from __future__ import annotations

import hashlib
import json
import re
import struct
import sys
from dataclasses import dataclass
from fractions import Fraction
from pathlib import Path
from typing import Any

import percussive_force_stage_a_v2_contract as v2_contract
import validate_percussive_force_stage_a_protocol as legacy_validator


PROTOCOL_V1_REL = v2_contract.PROTOCOL_V1_REL
PROTOCOL_V2_REL = v2_contract.PROTOCOL_V2_REL
REGISTRY_V3_REL = Path("docs/benchmarks/source_holdout_rotation_v3.json")
MATRIX_V3_REL = Path("docs/benchmarks/percussive_force_development_matrix_v3.json")
REGISTRY_V3_SCHEMA = "riotbox.source_holdout_rotation.v3"
MATRIX_V3_SCHEMA = "riotbox.percussive_force_development_matrix.v3"

EXPECTED_PROTOCOL_V2_RAW_SHA256 = (
    "b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878"
)
EXPECTED_PROTOCOL_V2_SEMANTIC_SHA256 = (
    "6f8db5d1488168c11bbd13be6c8862b2ae9b70424ce9e3e4887fd87d311b74fb"
)
SHA256 = re.compile(r"^[0-9a-f]{64}$")

EXPECTED_V2_COMPONENTS = {
    "prequalification": "riotbox.percussive_force_prequalification.v3",
    "impact_role": "riotbox.impact_role.v2",
    "event_detector": "riotbox.percussive_event_detector.v1",
    "event_anatomy": "riotbox.percussive_event_anatomy.v2",
    "source_contrast": "riotbox.percussive_source_contrast.v1",
    "rhythmic_location_proxy": "riotbox.rhythmic_location_proxy.v1",
    "event_ordinal_policy": "riotbox.event_ordinal_policy.v1",
    "f1": "f1_ab_energy_redistribution_v1",
    "f2": "f2_exact_complementary_three_band_v1",
    "f3": "f3_causal_envelope_contrast_dynamic_residual_v2",
    "source_analysis": "riotbox.percussive_force_source_analysis.v2",
    "unbound_qualification_analysis": (
        "riotbox.percussive_force_stage_a_unbound_qualification_analysis.v2"
    ),
    "bound_event_catalog": "riotbox.percussive_force_stage_a_bound_event_catalog.v2",
    "qualification_rejection": (
        "riotbox.percussive_force_stage_a_qualification_rejection.v2"
    ),
    "qualification_session": (
        "riotbox.percussive_force_stage_a_qualification_session.v2"
    ),
    "qualification_commit": (
        "riotbox.percussive_force_stage_a_qualification_commit.v1"
    ),
}

RESULT_KEYS = {
    "actual",
    "candidate",
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
    """Raised when Protocol v2 is not the exact frozen successor."""


@dataclass(frozen=True, init=False, slots=True)
class FrozenProtocolV2:
    """Revalidatable binding for the future Registry-v3/Matrix-v3 layer.

    The constructor is intentionally sealed.  A downstream consumer must call
    ``revalidated()`` (or validate the repository itself) at its trust boundary;
    public hash-looking strings never establish protocol authority.
    """

    _repo_root: Path
    _payload: bytes
    _predecessor_payload: bytes

    def __init__(self, *_args: Any, **_kwargs: Any) -> None:
        raise TypeError("FrozenProtocolV2 must be created by validate_repository()")

    @classmethod
    def _from_validated(
        cls,
        repo_root: Path,
        payload: bytes,
        predecessor_payload: bytes,
    ) -> "FrozenProtocolV2":
        frozen = object.__new__(cls)
        object.__setattr__(frozen, "_repo_root", repo_root.resolve())
        object.__setattr__(frozen, "_payload", bytes(payload))
        object.__setattr__(
            frozen,
            "_predecessor_payload",
            bytes(predecessor_payload),
        )
        return frozen

    @property
    def path(self) -> Path:
        return PROTOCOL_V2_REL

    @property
    def schema(self) -> str:
        return "riotbox.percussive_force_stage_a_protocol.v2"

    @property
    def raw_sha256(self) -> str:
        return hashlib.sha256(self._payload).hexdigest()

    @property
    def semantic_sha256(self) -> str:
        document = json.loads(
            self._payload,
            object_pairs_hook=_reject_duplicate_object_keys,
            parse_constant=_reject_nonfinite_constant,
        )
        return v2_contract.semantic_sha256(document)

    @property
    def predecessor_raw_sha256(self) -> str:
        return hashlib.sha256(self._predecessor_payload).hexdigest()

    @property
    def predecessor_semantic_sha256(self) -> str:
        document = json.loads(
            self._predecessor_payload,
            object_pairs_hook=_reject_duplicate_object_keys,
            parse_constant=_reject_nonfinite_constant,
        )
        return v2_contract.semantic_sha256(document)

    def revalidated(self) -> "FrozenProtocolV2":
        try:
            repo_root = self._repo_root
        except AttributeError as error:
            raise ContractError(
                "unvalidated Protocol-v2 binding has no repository root"
            ) from error
        if not isinstance(repo_root, Path):
            raise ContractError(
                "unvalidated Protocol-v2 binding has an invalid repository root"
            )
        return validate_repository(repo_root)


def revalidate_frozen_protocol_v2(value: FrozenProtocolV2) -> FrozenProtocolV2:
    """Re-establish Protocol-v2 authority at a downstream trust boundary.

    Never trust fields or a potentially overridden instance method supplied by
    a caller.  Exact-type checking plus the class-owned implementation forces a
    fresh read and validation of every pinned repository contract.
    """

    if type(value) is not FrozenProtocolV2:
        raise ContractError(
            "Protocol-v2 binding must be the exact FrozenProtocolV2 type"
        )
    return FrozenProtocolV2.revalidated(value)


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


def _list(path: str, value: Any) -> list[Any]:
    if not isinstance(value, list):
        _fail(path, "must be an array")
    return value


def _reject_duplicate_object_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, child in pairs:
        if key in value:
            raise ContractError(f"duplicate JSON object key: {key!r}")
        value[key] = child
    return value


def _reject_nonfinite_constant(token: str) -> None:
    raise ContractError(f"nonfinite JSON number is forbidden: {token}")


def _load_named_json(
    repo_root: Path, relative: Path
) -> tuple[dict[str, Any], bytes, str, str]:
    if relative not in {PROTOCOL_V1_REL, PROTOCOL_V2_REL}:
        _fail(str(relative), "v2 validator may read only the two named protocol contracts")
    path = repo_root / relative
    if path.suffix != ".json" or not path.is_file():
        _fail(str(relative), "named protocol contract is missing or not a regular file")
    payload = path.read_bytes()
    try:
        decoded = json.loads(
            payload,
            object_pairs_hook=_reject_duplicate_object_keys,
            parse_constant=_reject_nonfinite_constant,
        )
    except ContractError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        _fail(str(relative), f"invalid JSON: {error}")
    document = _mapping(str(relative), decoded)
    raw_sha256 = hashlib.sha256(payload).hexdigest()
    try:
        semantic_sha256 = v2_contract.semantic_sha256(document)
    except (TypeError, ValueError) as error:
        _fail(str(relative), f"cannot canonicalize semantic JSON: {error}")
    return document, payload, raw_sha256, semantic_sha256


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


def _ensure_no_result_keys(value: Any, path: str = "$") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if key in RESULT_KEYS or key.endswith("_result") or key.endswith("_verdict"):
                _fail(f"{path}.{key}", "result/evidence fields are forbidden before execution")
            _ensure_no_result_keys(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _ensure_no_result_keys(child, f"{path}[{index}]")


def _ensure_no_forward_or_self_hash_cycle(protocol: dict[str, Any]) -> None:
    self_hashes = {
        EXPECTED_PROTOCOL_V2_RAW_SHA256,
        EXPECTED_PROTOCOL_V2_SEMANTIC_SHA256,
    }

    def visit(value: Any, path: str) -> None:
        if isinstance(value, dict):
            if value.get("schema") == MATRIX_V3_SCHEMA:
                for key in value:
                    if "sha256" in key.casefold():
                        _fail(path, "Protocol v2 must not bind a forward Matrix-v3 hash")
            for key, child in value.items():
                visit(child, f"{path}.{key}")
            return
        if isinstance(value, list):
            for index, child in enumerate(value):
                visit(child, f"{path}[{index}]")
            return
        if not isinstance(value, str):
            return
        if value in self_hashes:
            _fail(path, "Protocol v2 must not contain its own raw or semantic SHA-256")
        lowered_path = path.casefold()
        if (
            "sha256" in lowered_path
            and (
                "matrix_v3" in lowered_path
                or "development_matrix_v3" in lowered_path
            )
        ):
            _fail(path, "Protocol v2 must not bind a forward Matrix-v3 hash")

    visit(protocol, "protocol_v2")


def _find_f3(protocol: dict[str, Any]) -> dict[str, Any]:
    precandidate = _mapping("protocol_v2.precandidate", protocol.get("precandidate"))
    families = _list(
        "protocol_v2.precandidate.algorithm_families",
        precandidate.get("algorithm_families"),
    )
    matches = [
        _mapping("protocol_v2.precandidate.algorithm_families[]", family)
        for family in families
        if isinstance(family, dict) and family.get("family") == "F3"
    ]
    if len(matches) != 1:
        _fail("protocol_v2.precandidate.algorithm_families", "must contain exactly one F3")
    return matches[0]


def _validate_component_versions(protocol: dict[str, Any]) -> None:
    versions = _mapping(
        "protocol_v2.component_versions", protocol.get("component_versions")
    )
    for component, expected in EXPECTED_V2_COMPONENTS.items():
        _expect(f"protocol_v2.component_versions.{component}", versions.get(component), expected)


def _validate_refusal_semantics(protocol: dict[str, Any]) -> None:
    prequalification = _mapping(
        "protocol_v2.prequalification", protocol.get("prequalification")
    )
    roles = _mapping(
        "protocol_v2.prequalification.impact_roles",
        prequalification.get("impact_roles"),
    )
    reasons = _list(
        "protocol_v2.prequalification.impact_roles.refusal_reasons",
        roles.get("refusal_reasons"),
    )
    if not reasons or reasons[0] != "physical_onset_unresolved":
        _fail(
            "protocol_v2.prequalification.impact_roles.refusal_reasons",
            "the first typed refusal must be physical_onset_unresolved",
        )
    if "edge_only_impulse" in reasons:
        _fail(
            "protocol_v2.prequalification.impact_roles.refusal_reasons",
            "generic edge_only_impulse is forbidden without separate edge evidence",
        )
    anatomy = _mapping(
        "protocol_v2.prequalification.event_anatomy",
        prequalification.get("event_anatomy"),
    )
    physical_onset = anatomy.get("physical_onset")
    if (
        not isinstance(physical_onset, str)
        or "refuse physical_onset_unresolved" not in physical_onset
        or "without claiming edge-only anatomy" not in physical_onset
    ):
        _fail(
            "protocol_v2.prequalification.event_anatomy.physical_onset",
            "must bind unresolved onset to the non-edge-specific typed refusal",
        )
    tombstone = _mapping(
        "protocol_v2.historical_v1_tombstone",
        protocol.get("historical_v1_tombstone"),
    )
    _expect(
        "protocol_v2.historical_v1_tombstone.generic_edge_only_impulse_interpretation_allowed",
        tombstone.get("generic_edge_only_impulse_interpretation_allowed"),
        False,
    )
    _expect(
        "protocol_v2.historical_v1_tombstone.runner_state",
        tombstone.get("runner_state"),
        "stage_a_v1_execution_closed_by_rbx_254",
    )


def _validate_source_mean_and_f3_boundary(protocol: dict[str, Any]) -> None:
    prequalification = _mapping(
        "protocol_v2.prequalification", protocol.get("prequalification")
    )
    source_input = _mapping(
        "protocol_v2.prequalification.input", prequalification.get("input")
    )
    reduction = _mapping(
        "protocol_v2.prequalification.input.per_channel_dc_mean_reduction",
        source_input.get("per_channel_dc_mean_reduction"),
    )
    _expect(
        "protocol_v2.mean_reduction.keys",
        set(reduction),
        {
            "algorithm_id",
            "decode",
            "accumulation",
            "exact_rational",
            "binary64_rounding",
            "serialization",
            "golden",
            "failure",
        },
    )
    _expect(
        "protocol_v2.mean_reduction.algorithm_id",
        reduction.get("algorithm_id"),
        v2_contract.SOURCE_MEAN_REDUCTION_ALGORITHM,
    )
    for field, token in (
        ("decode", "do not convert individual samples to floating point"),
        ("accumulation", "strictly ascending frame index"),
        ("exact_rational", "S_c/(frame_count*2^(valid_bits-1))"),
        ("binary64_rounding", "roundTiesToEven"),
        ("serialization", "16 lowercase big-endian hexadecimal characters"),
        ("failure", "typed_refusal"),
    ):
        value = reduction.get(field)
        if not isinstance(value, str) or token not in value:
            _fail(f"protocol_v2.mean_reduction.{field}", f"must contain {token!r}")
    golden = _mapping(
        "protocol_v2.mean_reduction.golden", reduction.get("golden")
    )
    _expect(
        "protocol_v2.mean_reduction.golden",
        golden,
        {
            "golden_id": v2_contract.SOURCE_MEAN_REDUCTION_GOLDEN_ID,
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
    )
    denominator = golden["frame_count"] * (1 << (golden["valid_bits"] - 1))
    observed_sums = [sum(channel) for channel in golden["channel_codes_in_frame_order"]]
    observed_bits = [
        struct.pack(">d", float(Fraction(total, denominator))).hex()
        for total in observed_sums
    ]
    _expect(
        "protocol_v2.mean_reduction.golden.exact_signed_code_sums",
        observed_sums,
        golden["exact_signed_code_sums"],
    )
    _expect(
        "protocol_v2.mean_reduction.golden.per_channel_dc_mean_f64_bits_be_hex",
        observed_bits,
        golden["per_channel_dc_mean_f64_bits_be_hex"],
    )

    precandidate = _mapping("protocol_v2.precandidate", protocol.get("precandidate"))
    wrapper = _mapping(
        "protocol_v2.precandidate.qualification_wrapper",
        precandidate.get("qualification_wrapper"),
    )
    binding = _mapping(
        "protocol_v2.precandidate.qualification_wrapper.authoritative_source_mean_binding",
        wrapper.get("authoritative_source_mean_binding"),
    )
    _expect(
        "protocol_v2.mean_binding.field",
        binding.get("field"),
        "per_channel_dc_mean_f64_bits_be_hex",
    )
    _expect(
        "protocol_v2.mean_binding.representation",
        binding.get("representation"),
        "one_16_character_lowercase_f64_bit_pattern_per_verified_channel_in_channel_order",
    )
    _expect(
        "protocol_v2.mean_binding.reduction_algorithm_id",
        binding.get("reduction_algorithm_id"),
        v2_contract.SOURCE_MEAN_REDUCTION_ALGORITHM,
    )
    _expect(
        "protocol_v2.mean_binding.reduction_golden_id",
        binding.get("reduction_golden_id"),
        v2_contract.SOURCE_MEAN_REDUCTION_GOLDEN_ID,
    )
    _expect(
        "protocol_v2.mean_binding.bound_identity",
        binding.get("bound_identity"),
        [
            "protocol_raw_sha256",
            "source_sha256",
            "pcm_encoding",
            "valid_bits",
            "sample_rate_hz",
            "channel_count",
            "frame_count",
        ],
    )
    _expect(
        "protocol_v2.mean_binding.catalog_ownership",
        binding.get("catalog_ownership"),
        "bound_event_catalog_v2_carries_the_source_analysis_v2_bits_and_renderer_decodes_them_exactly",
    )
    _expect(
        "protocol_v2.mean_binding.candidate_analysis",
        binding.get("candidate_analysis"),
        "subtract_the_same_source_frozen_means_never_candidate_local_means",
    )
    _expect(
        "protocol_v2.mean_binding.decimal_diagnostic_policy",
        binding.get("decimal_diagnostic_policy"),
        "optional_only_if_reencoding_matches_authoritative_bits_never_a_second_authority",
    )
    _expect(
        "protocol_v2.mean_binding.failure",
        binding.get("failure"),
        "typed_refusal_no_candidate_audio_no_fallback",
    )

    f3 = _find_f3(protocol)
    _expect(
        "protocol_v2.F3.causal_scope",
        f3.get("causal_scope"),
        "conditional_on_source_frozen_offline_state",
    )
    _expect(
        "protocol_v2.F3.end_to_end_streaming_causal",
        f3.get("end_to_end_streaming_causal"),
        False,
    )
    _expect(
        "protocol_v2.F3.offline_frozen_state",
        f3.get("offline_frozen_state"),
        ["whole_source_dc_means", "event_anatomy", "attack_body_masks"],
    )
    for field, token in (
        ("input_and_provenance", "never recomputes a source or candidate mean"),
        ("phase_safe_envelopes", "not an end-to-end streaming-causality claim"),
        ("resolved_policy_record", "per_channel_dc_mean_f64_bits_be_hex"),
    ):
        value = f3.get(field)
        if not isinstance(value, str) or token not in value:
            _fail(f"protocol_v2.F3.{field}", f"must contain {token!r}")


def _validate_acquisition_gate(protocol: dict[str, Any]) -> None:
    boundary = _mapping(
        "protocol_v2.stage_boundary", protocol.get("stage_boundary")
    )
    allowed = _list(
        "protocol_v2.stage_boundary.allowed_now", boundary.get("allowed_now")
    )
    _expect(
        "protocol_v2.stage_boundary.allowed_now.acquisition",
        allowed[-1] if allowed else None,
        "run_one_exact_predeclared_three_file_development_acquisition_and_header_format_session_before_registry_v3_freeze_only_after_the_versioned_acquisition_gate_validates",
    )
    forbidden_before = _list(
        "protocol_v2.stage_boundary.forbidden_before_this_contract_and_matrix_validate",
        boundary.get("forbidden_before_this_contract_and_matrix_validate"),
    )
    for required in (
        "source_preview_or_source_audio_playback",
        "audio_decode_or_pcm_sample_iteration_outside_strict_header_validation",
    ):
        if required not in forbidden_before:
            _fail(
                "protocol_v2.stage_boundary.forbidden_before_this_contract_and_matrix_validate",
                f"missing fail-closed acquisition prohibition {required!r}",
            )

    gate = _mapping(
        "protocol_v2.stage_boundary.batch_acquisition_exception",
        boundary.get("batch_acquisition_exception"),
    )
    _expect(
        "protocol_v2.acquisition.authorization",
        gate.get("authorization"),
        "protocol_v2_validation_alone_never_authorizes_network_or_source_file_access",
    )
    _expect(
        "protocol_v2.acquisition.required_preregistration_schema",
        gate.get("required_preregistration_schema"),
        "riotbox.percussive_force_stage_a_v2_acquisition_batch.v1",
    )
    _expect(
        "protocol_v2.acquisition.required_access_log_schema",
        gate.get("required_access_log_schema"),
        "riotbox.percussive_force_stage_a_v2_acquisition_access_log.v1",
    )
    _expect("protocol_v2.acquisition.exact_batch_size", gate.get("exact_batch_size"), 3)
    for field, token in (
        ("required_before_network_bytes", "attachment_byte_count"),
        ("permitted_access", "declared_attachment byte_count_plus_one"),
        ("permitted_access", "without_sample decoding_iteration_or_analysis"),
        ("path_and_byte_boundaries", "no directory discovery"),
        ("publication", "publish_no_final_destination_until_all_three"),
    ):
        value = gate.get(field)
        if not isinstance(value, str) or token not in value:
            _fail(f"protocol_v2.acquisition.{field}", f"must contain {token!r}")
    forbidden = _list(
        "protocol_v2.stage_boundary.batch_acquisition_exception.forbidden",
        gate.get("forbidden"),
    )
    for required in (
        "audio_decode_or_pcm_sample_iteration",
        "source_preview_or_source_audio_playback",
        "holdout_audio_access",
        "commercial_reference_access",
        "sequential_survivor_selection",
    ):
        if required not in forbidden:
            _fail(
                "protocol_v2.stage_boundary.batch_acquisition_exception.forbidden",
                f"missing fail-closed acquisition prohibition {required!r}",
            )


def _validate_exact_three_band_version_boundary(protocol: dict[str, Any]) -> None:
    precandidate = _mapping("protocol_v2.precandidate", protocol.get("precandidate"))
    three_band = _mapping(
        "protocol_v2.precandidate.exact_complementary_three_band_analysis",
        precandidate.get("exact_complementary_three_band_analysis"),
    )
    _expect(
        "protocol_v2.exact_three_band.version_binding",
        three_band.get("version_binding"),
        "Any split, DFT, state, filter, or failure-semantic change requires a protocol v3 and an exact_three_band_analysis component bump before recomputation.",
    )


def _validate_future_change_rule(protocol: dict[str, Any]) -> None:
    passport_contract = _mapping(
        "protocol_v2.numeric_passport_contract",
        protocol.get("numeric_passport_contract"),
    )
    _expect(
        "protocol_v2.numeric_passport_contract.change_rule_token",
        passport_contract.get("change_rule_token"),
        v2_contract.NEW_CHANGE_RULE,
    )
    passports = _mapping(
        "protocol_v2.numeric_passports", protocol.get("numeric_passports")
    )
    for passport_id, raw_passport in passports.items():
        passport = _mapping(f"protocol_v2.numeric_passports.{passport_id}", raw_passport)
        _expect(
            f"protocol_v2.numeric_passports.{passport_id}.change_rule",
            passport.get("change_rule"),
            v2_contract.NEW_CHANGE_RULE,
        )
    change_control = _mapping(
        "protocol_v2.change_control", protocol.get("change_control")
    )
    single_rule = change_control.get("single_rule")
    specific_rule = change_control.get("prequalification_specific_rule")
    if not isinstance(single_rule, str) or "stage_a_protocol.v3" not in single_rule:
        _fail("protocol_v2.change_control.single_rule", "must require Protocol v3")
    if not isinstance(specific_rule, str) or "prequalification.v3 to v4" not in specific_rule:
        _fail(
            "protocol_v2.change_control.prequalification_specific_rule",
            "must require Prequalification v4 after a v3 change",
        )


def _validate_protocol_v2_successor(
    protocol_v1: dict[str, Any], protocol_v2: dict[str, Any]
) -> str:
    """Validate one decoded successor against an already pinned predecessor."""

    _ensure_no_result_keys(protocol_v2)
    _ensure_no_forward_or_self_hash_cycle(protocol_v2)
    _expect(
        "protocol_v2.schema",
        protocol_v2.get("schema"),
        "riotbox.percussive_force_stage_a_protocol.v2",
    )
    _expect("protocol_v2.schema_version", protocol_v2.get("schema_version"), 2)
    _expect("protocol_v2.owner_ticket", protocol_v2.get("owner_ticket"), "RIOTBOX-1430")
    _validate_component_versions(protocol_v2)
    _validate_refusal_semantics(protocol_v2)
    _validate_source_mean_and_f3_boundary(protocol_v2)
    _validate_acquisition_gate(protocol_v2)
    _validate_exact_three_band_version_boundary(protocol_v2)
    _validate_future_change_rule(protocol_v2)

    try:
        expected = v2_contract.upgrade_protocol_v1_to_v2(protocol_v1)
    except v2_contract.ContractBuildError as error:
        _fail("protocol_v1", f"cannot construct the deterministic successor: {error}")
    difference = _first_difference(protocol_v2, expected)
    if difference is not None:
        _fail("protocol_v2 exact successor", difference)

    semantic_sha256 = v2_contract.semantic_sha256(protocol_v2)
    _expect(
        "protocol_v2 semantic SHA-256",
        semantic_sha256,
        EXPECTED_PROTOCOL_V2_SEMANTIC_SHA256,
    )
    return semantic_sha256


def validate_protocol_v2_document(
    protocol_v2: dict[str, Any], *, repo_root: Path | None = None
) -> str:
    """Validate a decoded v2 document against the canonical pinned predecessor.

    The predecessor is intentionally not a caller argument.  Mutation fixtures
    may inject the candidate document, but never predecessor content or hashes.
    """

    resolved_root = (
        repo_root if repo_root is not None else Path(__file__).resolve().parents[1]
    )
    protocol_v1, _, v1_raw, v1_semantic = _load_named_json(
        resolved_root, PROTOCOL_V1_REL
    )
    _expect("protocol_v1 raw SHA-256", v1_raw, v2_contract.PROTOCOL_V1_RAW_SHA256)
    _expect(
        "protocol_v1 semantic SHA-256",
        v1_semantic,
        v2_contract.PROTOCOL_V1_SEMANTIC_SHA256,
    )
    return _validate_protocol_v2_successor(protocol_v1, protocol_v2)


def validate_protocol_v2_pins(raw_sha256: str, semantic_sha256: str) -> None:
    if SHA256.fullmatch(raw_sha256) is None:
        _fail("protocol_v2 raw SHA-256", "must be lowercase hexadecimal SHA-256")
    if SHA256.fullmatch(semantic_sha256) is None:
        _fail("protocol_v2 semantic SHA-256", "must be lowercase hexadecimal SHA-256")
    _expect(
        "protocol_v2 raw SHA-256",
        raw_sha256,
        EXPECTED_PROTOCOL_V2_RAW_SHA256,
    )
    _expect(
        "protocol_v2 semantic SHA-256",
        semantic_sha256,
        EXPECTED_PROTOCOL_V2_SEMANTIC_SHA256,
    )


def validate_repository(repo_root: Path) -> FrozenProtocolV2:
    """Validate the immutable legacy stack, then its exact Protocol-v2 successor."""

    legacy_validator.validate_repository(repo_root)
    protocol_v1, v1_payload, v1_raw, v1_semantic = _load_named_json(
        repo_root, PROTOCOL_V1_REL
    )
    protocol_v2, v2_payload, v2_raw, v2_semantic = _load_named_json(
        repo_root, PROTOCOL_V2_REL
    )
    _expect("protocol_v1 raw SHA-256", v1_raw, v2_contract.PROTOCOL_V1_RAW_SHA256)
    _expect(
        "protocol_v1 semantic SHA-256",
        v1_semantic,
        v2_contract.PROTOCOL_V1_SEMANTIC_SHA256,
    )
    validated_semantic = _validate_protocol_v2_successor(protocol_v1, protocol_v2)
    _expect("protocol_v2 loaded semantic SHA-256", v2_semantic, validated_semantic)
    validate_protocol_v2_pins(v2_raw, v2_semantic)

    expected_payload = v2_contract.render_protocol_v2(
        v2_contract.upgrade_protocol_v1_to_v2(protocol_v1)
    )
    if v2_payload != expected_payload:
        _fail(
            str(PROTOCOL_V2_REL),
            "raw bytes do not match the deterministic Protocol-v2 renderer",
        )

    return FrozenProtocolV2._from_validated(repo_root, v2_payload, v1_payload)


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    try:
        frozen = validate_repository(repo_root)
    except (ContractError, legacy_validator.ContractError, OSError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: RIOTBOX-1430 Stage-A Protocol v2 is an exact source-blind successor")
    print(f"protocol_v2_raw_sha256={frozen.raw_sha256}")
    print(f"protocol_v2_semantic_sha256={frozen.semantic_sha256}")
    print(f"registry_v3_extension_path={REGISTRY_V3_REL}")
    print(f"matrix_v3_extension_path={MATRIX_V3_REL}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
