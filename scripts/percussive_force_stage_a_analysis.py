#!/usr/bin/env python3
"""Frozen Stage-A source and event qualification analysis.

This module is deliberately decode-independent.  Callers must supply an
already verified, normalized PCM array with shape ``frames x channels``.  The
module never opens a source path, discovers a directory, renders a candidate,
or assigns perceptual hardness.  Its results are reject-only source/event and
source-contrast evidence under the pinned Stage-A v1 or v2 preregistration.
"""

from __future__ import annotations

import hashlib
import json
import math
import struct
from dataclasses import dataclass, fields, is_dataclass
from enum import Enum
from pathlib import Path
from types import MappingProxyType
from typing import Any, Mapping, Sequence

import numpy as np


EXPECTED_PROTOCOL_SHA256 = (
    "35091e697cacb3c187f9a33f4f41ac85aba26832a4214bf3251dfc703edad840"
)
EXPECTED_PROTOCOL_SCHEMA = "riotbox.percussive_force_stage_a_protocol.v1"
EXPECTED_PROTOCOL_V2_SHA256 = (
    "b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878"
)
EXPECTED_PROTOCOL_V2_SCHEMA = "riotbox.percussive_force_stage_a_protocol.v2"
CANONICAL_PROTOCOL_PATH = (
    Path(__file__).resolve().parents[1]
    / "docs/benchmarks/percussive_force_stage_a_protocol_v1.json"
)
CANONICAL_PROTOCOL_V2_PATH = (
    Path(__file__).resolve().parents[1]
    / "docs/benchmarks/percussive_force_stage_a_protocol_v2.json"
)

_PROTOCOL_IDENTITIES = {
    EXPECTED_PROTOCOL_SHA256: {
        "schema": EXPECTED_PROTOCOL_SCHEMA,
        "schema_version": 1,
        "owner_ticket": "RIOTBOX-1428",
        "execution_state": "preregistered_no_source_qualification_or_candidate_render",
        "prequalification": "riotbox.percussive_force_prequalification.v2",
    },
    EXPECTED_PROTOCOL_V2_SHA256: {
        "schema": EXPECTED_PROTOCOL_V2_SCHEMA,
        "schema_version": 2,
        "owner_ticket": "RIOTBOX-1430",
        "execution_state": "preregistered_no_v2_source_qualification_or_candidate_render",
        "prequalification": "riotbox.percussive_force_prequalification.v3",
    },
}


class StageAContractError(ValueError):
    """Raised when a caller attempts to leave the frozen contract."""

    def __init__(self, code: str, detail: str) -> None:
        super().__init__(f"{code}: {detail}")
        self.code = code
        self.detail = detail


class ImpactRole(str, Enum):
    BODY_BEARING_SINGLE_PERCUSSIVE = "body_bearing_single_percussive"
    BODY_BEARING_FUSED_COMPOSITE_PERCUSSIVE = (
        "body_bearing_fused_composite_percussive"
    )


class EventRefusalReason(str, Enum):
    EDGE_ONLY_IMPULSE = "edge_only_impulse"
    PHYSICAL_ONSET_UNRESOLVED = "physical_onset_unresolved"
    MULTI_EVENT_OR_FLAM = "multi_event_or_flam"
    SLOW_OR_SUSTAINED = "slow_or_sustained"
    LOOKBEHIND_MASKED = "lookbehind_masked"
    ATTACK_TURNOVER_UNRESOLVED = "attack_turnover_unresolved"
    BODY_UNRESOLVED = "body_unresolved"
    TAIL_UNRESOLVED = "tail_unresolved"
    OVERLAPPED_EVENT = "overlapped_event"
    INSUFFICIENT_SIGNAL = "insufficient_signal"
    UNSUPPORTED_FORMAT = "unsupported_format"
    NONFINITE_ANALYSIS = "nonfinite_analysis"


class SourceRefusalReason(str, Enum):
    HOLDOUT_ACCESS_FORBIDDEN = "holdout_access_forbidden"
    EMPTY_INPUT = "empty_input"
    UNSUPPORTED_CHANNEL_COUNT = "unsupported_channel_count"
    UNSUPPORTED_SAMPLE_RATE = "unsupported_sample_rate"
    UNSUPPORTED_FORMAT = "unsupported_format"
    NONFINITE_ANALYSIS = "nonfinite_analysis"
    INSUFFICIENT_SIGNAL = "insufficient_signal"
    INSUFFICIENT_ELIGIBLE_EVENTS = "insufficient_eligible_events"
    SOURCE_FEATURE_REQUIREMENTS_UNMET = "source_feature_requirements_unmet"


class PairClassification(str, Enum):
    DISTINCT = "distinct"
    SIMILAR = "similar"
    AMBIGUOUS = "ambiguous"


class PcmEncoding(str, Enum):
    PCM_S16LE = "pcm_s16le"
    PCM_S24LE = "pcm_s24le"


class QualificationRefusalReason(str, Enum):
    SOURCE_COUNT = "source_count_must_equal_four"
    DUPLICATE_CASE_ID = "duplicate_case_id"
    AUTHOR_COUNT = "author_count_must_equal_four"
    FAMILY_COUNT = "family_count_must_equal_three"
    SOURCE_FAILED = "positive_source_failed"
    PARTITION_GATE = "source_contrast_partition_gate_failed"


def _json_value(value: Any) -> Any:
    custom = getattr(value, "_to_json_value", None)
    if callable(custom):
        return custom()
    if isinstance(value, Enum):
        return value.value
    if is_dataclass(value):
        return {field.name: _json_value(getattr(value, field.name)) for field in fields(value)}
    if isinstance(value, Mapping):
        return {str(key): _json_value(child) for key, child in value.items()}
    if isinstance(value, (tuple, list)):
        return [_json_value(child) for child in value]
    if isinstance(value, np.generic):
        return value.item()
    return value


class Serializable:
    def to_dict(self) -> dict[str, Any]:
        value = _json_value(self)
        if not isinstance(value, dict):
            raise TypeError("serialized dataclass root must be an object")
        return value

    def to_json(self, *, indent: int | None = 2) -> str:
        return json.dumps(self.to_dict(), indent=indent, sort_keys=True, allow_nan=False)


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise StageAContractError(
                "duplicate_protocol_key", f"duplicate JSON key {key!r}"
            )
        result[key] = value
    return result


def _deep_freeze_json(value: Any) -> Any:
    if isinstance(value, dict):
        return MappingProxyType(
            {key: _deep_freeze_json(child) for key, child in value.items()}
        )
    if isinstance(value, list):
        return tuple(_deep_freeze_json(child) for child in value)
    return value


@dataclass(frozen=True, init=False, slots=True)
class FrozenStageAProtocol:
    sha256: str
    _payload: bytes
    _document: Mapping[str, Any]

    def __init__(self, *_args: Any, **_kwargs: Any) -> None:
        raise TypeError("FrozenStageAProtocol must be created with from_bytes()")

    @classmethod
    def from_bytes(cls, payload: bytes) -> "FrozenStageAProtocol":
        if not isinstance(payload, bytes):
            raise StageAContractError(
                "invalid_protocol_bytes", "protocol payload must be immutable bytes"
            )
        actual = hashlib.sha256(payload).hexdigest()
        identity = _PROTOCOL_IDENTITIES.get(actual)
        if identity is None:
            raise StageAContractError(
                "protocol_pin_mismatch",
                f"expected one of {sorted(_PROTOCOL_IDENTITIES)}, got {actual}",
            )
        try:
            document = json.loads(payload, object_pairs_hook=_reject_duplicate_keys)
        except (json.JSONDecodeError, UnicodeDecodeError) as error:
            raise StageAContractError("invalid_protocol_json", str(error)) from error
        if not isinstance(document, dict):
            raise StageAContractError("invalid_protocol_root", "root must be an object")
        if document.get("schema") != identity["schema"]:
            raise StageAContractError(
                "protocol_schema_mismatch", "unexpected Stage-A protocol schema"
            )
        if document.get("schema_version") != identity["schema_version"]:
            raise StageAContractError(
                "protocol_version_mismatch", "schema_version does not match the raw pin"
            )
        if document.get("owner_ticket") != identity["owner_ticket"]:
            raise StageAContractError(
                "protocol_owner_mismatch", "owner_ticket does not match the raw pin"
            )
        if document.get("execution_state") != identity["execution_state"]:
            raise StageAContractError(
                "protocol_state_mismatch", "unexpected preregistration state"
            )
        passports = document.get("numeric_passports")
        if not isinstance(passports, dict):
            raise StageAContractError(
                "protocol_passports_missing", "numeric_passports must be an object"
            )
        required = {
            "input_channel_counts",
            "input_sample_rate_range_hz",
            "analysis_epsilon_lsb_squared",
            "minimum_signal_peak_lsb",
            "frame_rounding_offset",
            "periodic_hann_coefficients",
            "detector_window_ms",
            "detector_hop_ms",
            "detector_log_energy_lag_hops",
            "detector_median_hops",
            "detector_baseline_radius_ms",
            "detector_baseline_exclusion_ms",
            "detector_minimum_baseline_hops",
            "mad_consistency_scale",
            "detector_mad_multiplier",
            "detector_zero_mad_delta",
            "detector_coarse_rms_floor_ratio",
            "detector_local_rms_percentile",
            "detector_peak_search_radius_ms",
            "detector_nms_ms",
            "analysis_band_edges_hz",
            "rms_envelope_windows_ms",
            "anchor_search_ms",
            "lookbehind_ms",
            "onset_fraction_above_baseline",
            "onset_persistence_ms",
            "anatomy_peak_baseline_ratio",
            "lookbehind_peak_ratio_max",
            "attack_peak_search_ms",
            "attack_turnover_fraction",
            "attack_turnover_persistence_ms",
            "body_baseline_multiplier",
            "body_peak_fraction",
            "body_minimum_ms",
            "body_below_floor_ms",
            "body_maximum_ms",
            "tail_baseline_multiplier",
            "tail_peak_fraction",
            "tail_minimum_ms",
            "tail_below_floor_ms",
            "tail_maximum_ms",
            "composite_fusion_ms",
            "event_valley_peak_fraction",
            "event_valley_persistence_ms",
            "rhythmic_proxy_window_ms",
            "rhythmic_proxy_quantile",
            "source_welch_window_ms",
            "source_welch_hop_ms",
            "source_minimum_onsets",
            "source_minimum_resolved_body_events",
            "normalization_density_scale_per_second",
            "normalization_ioi_scale_ms",
            "normalization_ioi_cv_scale",
            "normalization_duration_scale_ms",
            "source_distinct_distance_min",
            "source_changed_domain_min_delta",
            "source_changed_domain_minimum_count",
            "positive_source_count",
            "positive_author_count",
            "positive_family_count",
            "source_distance_domain_count",
            "minimum_source_clusters",
            "four_source_partition_count",
            "valid_source_partition_count",
            "minimum_events_per_source",
            "maximum_frozen_events_per_source",
            "development_event_ordinals",
            "confirmation_event_ordinal",
        }
        missing = sorted(required - set(passports))
        if missing:
            raise StageAContractError(
                "protocol_passports_missing", f"missing passports: {missing}"
            )
        for name in required:
            passport = passports[name]
            if not isinstance(passport, dict) or "value" not in passport:
                raise StageAContractError(
                    "invalid_numeric_passport", f"{name} has no value"
                )
        components = document.get("component_versions")
        prequalification = document.get("prequalification")
        if not isinstance(components, dict) or components.get("prequalification") != identity[
            "prequalification"
        ]:
            raise StageAContractError(
                "prequalification_version_mismatch",
                "prequalification component does not match the raw pin",
            )
        if not isinstance(prequalification, dict) or prequalification.get("purpose") != (
            "Mechanism-blind rejection and partitioning only; no hardness score and no algorithm-family selection."
        ):
            raise StageAContractError(
                "prequalification_semantics_mismatch", "reject-only purpose changed"
            )
        frozen = object.__new__(cls)
        object.__setattr__(frozen, "sha256", actual)
        object.__setattr__(frozen, "_payload", bytes(payload))
        object.__setattr__(frozen, "_document", _deep_freeze_json(document))
        return frozen

    def revalidated(self) -> "FrozenStageAProtocol":
        try:
            payload = self._payload
        except AttributeError as error:
            raise StageAContractError(
                "unvalidated_protocol", "frozen protocol has no validated byte payload"
            ) from error
        if not isinstance(payload, bytes):
            raise StageAContractError(
                "unvalidated_protocol", "frozen protocol payload must be immutable bytes"
            )
        return FrozenStageAProtocol.from_bytes(payload)

    def value(self, name: str) -> Any:
        passports = self._document["numeric_passports"]
        return passports[name]["value"]

    @property
    def schema_version(self) -> int:
        return int(self._document["schema_version"])


def load_frozen_protocol(
    path: Path | str = CANONICAL_PROTOCOL_PATH,
) -> FrozenStageAProtocol:
    return FrozenStageAProtocol.from_bytes(Path(path).read_bytes())


def _revalidate_frozen_protocol(
    protocol: FrozenStageAProtocol | None,
) -> FrozenStageAProtocol:
    frozen = protocol if protocol is not None else load_frozen_protocol()
    if type(frozen) is not FrozenStageAProtocol:
        raise StageAContractError(
            "unvalidated_protocol", "entry point requires the exact frozen protocol"
        )
    return FrozenStageAProtocol.revalidated(frozen)


@dataclass(frozen=True)
class VerifiedPcmFormat(Serializable):
    encoding: PcmEncoding
    sample_rate_hz: int
    channel_count: int
    format_tag: int
    container_bits: int
    valid_bits: int
    block_align: int
    compression_type: str
    input_lsb: float

    @classmethod
    def signed_pcm(
        cls, *, valid_bits: int, sample_rate_hz: int, channel_count: int
    ) -> "VerifiedPcmFormat":
        if valid_bits == 16:
            encoding = PcmEncoding.PCM_S16LE
        elif valid_bits == 24:
            encoding = PcmEncoding.PCM_S24LE
        else:
            raise StageAContractError(
                "unsupported_format", "verified PCM valid_bits must be 16 or 24"
            )
        return cls(
            encoding=encoding,
            sample_rate_hz=int(sample_rate_hz),
            channel_count=int(channel_count),
            format_tag=1,
            container_bits=valid_bits,
            valid_bits=valid_bits,
            block_align=int(channel_count) * (valid_bits // 8),
            compression_type="NONE",
            input_lsb=math.ldexp(1.0, -(valid_bits - 1)),
        )

    @classmethod
    def coerce(cls, value: "VerifiedPcmFormat | Mapping[str, Any]") -> "VerifiedPcmFormat":
        if isinstance(value, cls):
            return value
        if not isinstance(value, Mapping):
            raise StageAContractError(
                "missing_verified_format",
                "case metadata must carry a verified PCM format record",
            )
        try:
            encoding = PcmEncoding(str(value["encoding"]))
            return cls(
                encoding=encoding,
                sample_rate_hz=int(value["sample_rate_hz"]),
                channel_count=int(value.get("channel_count", value.get("channels"))),
                format_tag=int(value["format_tag"]),
                container_bits=int(value["container_bits"]),
                valid_bits=int(value["valid_bits"]),
                block_align=int(value["block_align"]),
                compression_type=str(value["compression_type"]),
                input_lsb=float(value["input_lsb"]),
            )
        except (KeyError, TypeError, ValueError) as error:
            raise StageAContractError(
                "invalid_verified_format", "verified PCM format record is incomplete"
            ) from error


@dataclass(frozen=True)
class SourceMetadata(Serializable):
    case_id: str
    source_family: str
    author: str
    source_path: str
    source_sha256: str
    license: str
    verified_format: VerifiedPcmFormat
    partition: str = "development"

    @classmethod
    def coerce(cls, value: "SourceMetadata | Mapping[str, Any]") -> "SourceMetadata":
        if isinstance(value, cls):
            metadata = value
        else:
            if not isinstance(value, Mapping):
                raise StageAContractError(
                    "invalid_source_metadata",
                    "metadata must be a mapping or SourceMetadata",
                )
            verified_format = VerifiedPcmFormat.coerce(value.get("verified_format"))
            metadata = cls(
                case_id=str(value.get("case_id", "")).strip(),
                source_family=str(value.get("source_family", "")).strip(),
                author=str(value.get("author", "")).strip(),
                source_path=str(value.get("source_path", "")).strip(),
                source_sha256=str(
                    value.get("source_sha256", value.get("sha256", ""))
                ).strip(),
                license=str(value.get("license", "")).strip(),
                verified_format=verified_format,
                partition=str(value.get("partition", "")).strip(),
            )
        if not all(
            (
                metadata.case_id,
                metadata.source_family,
                metadata.author,
                metadata.source_path,
                metadata.source_sha256,
                metadata.license,
            )
        ):
            raise StageAContractError(
                "invalid_source_metadata",
                "case, family, author, path, SHA-256, license, and format provenance are required",
            )
        if len(metadata.source_sha256) != 64 or any(
            character not in "0123456789abcdef"
            for character in metadata.source_sha256
        ):
            raise StageAContractError(
                "invalid_source_metadata", "source SHA-256 must be lowercase hex"
            )
        return metadata


@dataclass(frozen=True)
class SourceInput:
    metadata: SourceMetadata | Mapping[str, Any]
    samples: np.ndarray
    sample_rate_hz: int
    input_lsb: float
    per_channel_dc_mean_f64_bits_be_hex: tuple[str, ...] | None = None


@dataclass(frozen=True)
class Refusal(Serializable):
    reason: SourceRefusalReason | QualificationRefusalReason
    detail: str


@dataclass(frozen=True)
class EventRefusal(Serializable):
    coarse_peak_frame: int
    reason: EventRefusalReason
    detail: str


@dataclass(frozen=True)
class EventRecord(Serializable):
    ordinal: int
    coarse_peak_frame: int
    coarse_novelty: float
    physical_onset_frame: int
    lookbehind_start_frame: int
    baseline_rms: float
    attack_peak_frame: int
    attack_peak_rms: float
    attack_end_frame: int
    body_end_frame: int
    tail_end_frame: int
    rhythmic_proxy_frame: int
    impact_role: ImpactRole
    refined_micropeak_count: int


@dataclass(frozen=True)
class DetectorSummary(Serializable):
    window_frames: int
    hop_frames: int
    log_energy_lag_hops: int
    frame_timestamp_rule: str
    pre_nms_peak_frames: tuple[int, ...]
    nms_peak_frames: tuple[int, ...]


@dataclass(frozen=True)
class SourceFeatureVector(Serializable):
    all_event_onset_density_per_second: float
    median_inter_onset_interval_ms: float
    population_inter_onset_interval_cv: float
    low_mid_high_energy_fractions: tuple[float, float, float]
    median_attack_to_body_mean_power_ratio: float
    median_resolved_event_duration_onset_to_tail_ms: float
    normalized_density: float
    normalized_ioi_rate: float
    normalized_irregularity: float
    normalized_attack_body_articulation: float
    normalized_duration: float


@dataclass(frozen=True)
class SourceAnalysis(Serializable):
    schema: str
    protocol_sha256: str
    metadata: SourceMetadata
    sample_rate_hz: int
    channel_count: int
    frame_count: int
    per_channel_dc_means: tuple[float, ...]
    detector: DetectorSummary
    event_level_onset_frames: tuple[int, ...]
    resolved_body_event_count: int
    events: tuple[EventRecord, ...]
    event_refusals: tuple[EventRefusal, ...]
    source_features: SourceFeatureVector | None
    qualified: bool
    quality_proof: bool
    hardness_proof: bool
    refusals: tuple[Refusal, ...]

    def _to_json_value(self) -> dict[str, Any]:
        value = {
            field.name: _json_value(getattr(self, field.name)) for field in fields(self)
        }
        if self.schema == "riotbox.percussive_force_source_analysis.v2":
            value["per_channel_dc_mean_f64_bits_be_hex"] = [
                struct.pack(">d", mean).hex() for mean in self.per_channel_dc_means
            ]
            del value["per_channel_dc_means"]
        return value


@dataclass(frozen=True)
class PairContrast(Serializable):
    left_case_id: str
    right_case_id: str
    rate_delta: float
    irregularity_delta: float
    spectrum_hellinger: float
    articulation_delta: float
    duration_delta: float
    overall_distance: float
    changed_domain_count: int
    classification: PairClassification


@dataclass(frozen=True)
class SourcePartition(Serializable):
    clusters: tuple[tuple[str, ...], ...]


@dataclass(frozen=True)
class StageAQualification(Serializable):
    schema: str
    qualification_state: str
    protocol_sha256: str
    sources: tuple[SourceAnalysis, ...]
    pair_contrasts: tuple[PairContrast, ...]
    valid_partitions: tuple[SourcePartition, ...]
    passed: bool
    quality_proof: bool
    hardness_proof: bool
    next_allowed_action: str
    refusals: tuple[Refusal, ...]


@dataclass(frozen=True)
class _DetectorPeak:
    hop_index: int
    frame: int
    novelty: float


@dataclass(frozen=True)
class _DetectorResult:
    pre_nms: tuple[_DetectorPeak, ...]
    nms: tuple[_DetectorPeak, ...]


@dataclass(frozen=True)
class _Anatomy:
    coarse_peak_frame: int
    coarse_novelty: float
    physical_onset_frame: int
    lookbehind_start_frame: int
    baseline_rms: float
    attack_peak_frame: int
    attack_peak_rms: float
    attack_end_frame: int
    body_end_frame: int
    tail_end_frame: int


@dataclass
class _Refinement:
    coarse_peak_frame: int
    coarse_novelty: float
    physical_onset_frame: int | None
    baseline_rms: float | None
    attack_peak_frame: int | None
    attack_peak_rms: float | None
    anatomy: _Anatomy | None
    refusal: EventRefusal | None


def _duration_frames(protocol: FrozenStageAProtocol, sample_rate_hz: int, ms: float) -> int:
    if not math.isfinite(ms) or ms <= 0.0:
        raise StageAContractError("invalid_positive_duration", f"duration must be >0: {ms}")
    offset = float(protocol.value("frame_rounding_offset"))
    return max(1, math.floor(sample_rate_hz * ms / 1000.0 + offset))


def _signed_offset_frames(
    protocol: FrozenStageAProtocol, sample_rate_hz: int, ms: float
) -> int:
    if not math.isfinite(ms):
        raise StageAContractError("invalid_signed_offset", "offset must be finite")
    if ms == 0.0:
        return 0
    sign = -1 if ms < 0.0 else 1
    offset = float(protocol.value("frame_rounding_offset"))
    return sign * math.floor(sample_rate_hz * abs(ms) / 1000.0 + offset)


def _median(values: np.ndarray) -> float:
    ordered = np.sort(np.asarray(values, dtype=np.float64))
    count = int(ordered.size)
    if count == 0:
        raise StageAContractError("empty_order_statistic", "median pool is empty")
    middle = count // 2
    if count % 2:
        return float(ordered[middle])
    return float((ordered[middle - 1] + ordered[middle]) / 2.0)


def _mad(values: np.ndarray, median: float) -> float:
    return _median(np.abs(np.asarray(values, dtype=np.float64) - median))


def _lower_order_percentile(values: np.ndarray, percent: float) -> float:
    ordered = np.sort(np.asarray(values, dtype=np.float64))
    if ordered.size == 0:
        raise StageAContractError("empty_order_statistic", "percentile pool is empty")
    index = math.floor((percent / 100.0) * (int(ordered.size) - 1))
    return float(ordered[index])


def _periodic_hann(protocol: FrozenStageAProtocol, length: int) -> np.ndarray:
    coefficients = protocol.value("periodic_hann_coefficients")
    a0 = float(coefficients[0])
    a1 = float(coefficients[1])
    indices = np.arange(length, dtype=np.float64)
    return a0 + a1 * np.cos(2.0 * np.pi * indices / float(length))


def _is_earliest_on_contiguous_plateau(values: np.ndarray, index: int) -> bool:
    """Return whether ``index`` is the first hop of its exact flat plateau.

    Equal-height local maxima separated by a lower hop are distinct plateaus.
    This helper therefore consults only the immediately preceding valid hop;
    the caller separately owns the inclusive local-maximum-radius test.
    """

    if index <= 0:
        return True
    value = float(values[index])
    previous = float(values[index - 1])
    return not (math.isfinite(previous) and previous == value)


def _next_power_of_two(value: int) -> int:
    return 1 << (value - 1).bit_length()


def _causal_rms(q_squared: np.ndarray, window: int) -> np.ndarray:
    result = np.full(q_squared.shape, np.nan, dtype=np.float64)
    if q_squared.size < window:
        return result
    prefix = np.empty(q_squared.size + 1, dtype=np.float64)
    prefix[0] = 0.0
    np.cumsum(q_squared, dtype=np.float64, out=prefix[1:])
    sums = prefix[window:] - prefix[:-window]
    result[window - 1 :] = np.sqrt(np.maximum(sums / float(window), 0.0))
    return result


def _band_masks(
    protocol: FrozenStageAProtocol, sample_rate_hz: int, fft_size: int
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    edges = [float(value) for value in protocol.value("analysis_band_edges_hz")]
    frequencies = np.fft.rfftfreq(fft_size, d=1.0 / float(sample_rate_hz))
    low = (frequencies >= edges[0]) & (frequencies < edges[1])
    middle = (frequencies >= edges[1]) & (frequencies < edges[2])
    high = (frequencies >= edges[2]) & (frequencies <= edges[3])
    if not low.any() or not middle.any() or not high.any():
        raise StageAContractError(
            "analysis_band_unavailable", "native-rate FFT has an empty frozen band"
        )
    return low, middle, high


def _detect_events(
    x_prime: np.ndarray,
    sample_rate_hz: int,
    input_lsb: float,
    protocol: FrozenStageAProtocol,
) -> _DetectorResult:
    frame_count, channel_count = x_prime.shape
    window_frames = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("detector_window_ms"))
    )
    hop_frames = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("detector_hop_ms"))
    )
    if frame_count < window_frames:
        return _DetectorResult((), ())
    starts = np.arange(0, frame_count - window_frames + 1, hop_frames, dtype=np.int64)
    hop_count = int(starts.size)
    fft_size = _next_power_of_two(window_frames)
    window = _periodic_hann(protocol, window_frames)
    window_energy = float(np.sum(window * window, dtype=np.float64))
    masks = _band_masks(protocol, sample_rate_hz, fft_size)
    band_energy = np.empty((hop_count, 3), dtype=np.float64)
    coarse_rms = np.empty(hop_count, dtype=np.float64)
    for hop_index, start_value in enumerate(starts):
        start = int(start_value)
        block = x_prime[start : start + window_frames]
        coarse_rms[hop_index] = math.sqrt(
            float(np.sum(block * block, dtype=np.float64))
            / float(window_frames * channel_count)
        )
        spectrum = np.fft.rfft(block * window[:, None], n=fft_size, axis=0)
        per_bin = np.sum(
            spectrum.real * spectrum.real + spectrum.imag * spectrum.imag,
            axis=1,
            dtype=np.float64,
        ) / window_energy
        for band_index, mask in enumerate(masks):
            band_energy[hop_index, band_index] = float(
                np.sum(per_bin[mask], dtype=np.float64)
            )

    lag = int(protocol.value("detector_log_energy_lag_hops"))
    novelty = np.full(hop_count, np.nan, dtype=np.float64)
    epsilon = float(protocol.value("analysis_epsilon_lsb_squared")) * input_lsb**2
    for hop_index in range(lag, hop_count):
        delta = np.log(band_energy[hop_index] + epsilon) - np.log(
            band_energy[hop_index - lag] + epsilon
        )
        positive = np.maximum(delta, 0.0)
        novelty[hop_index] = math.sqrt(float(np.mean(positive * positive)))

    median_hops = int(protocol.value("detector_median_hops"))
    if median_hops != 3:
        raise StageAContractError(
            "unsupported_frozen_median_width", "detector_median_hops must equal 3"
        )
    smoothed = np.full(hop_count, np.nan, dtype=np.float64)
    for hop_index in range(lag + 1, hop_count - 1):
        triplet = novelty[hop_index - 1 : hop_index + 2]
        if np.isfinite(triplet).all():
            smoothed[hop_index] = _median(triplet)

    timestamps = starts + window_frames // 2
    valid_indices = np.flatnonzero(np.isfinite(smoothed))
    baseline_radius = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("detector_baseline_radius_ms"))
    )
    baseline_exclusion = _duration_frames(
        protocol,
        sample_rate_hz,
        float(protocol.value("detector_baseline_exclusion_ms")),
    )
    minimum_baseline = int(protocol.value("detector_minimum_baseline_hops"))
    mad_multiplier = float(protocol.value("detector_mad_multiplier"))
    mad_scale = float(protocol.value("mad_consistency_scale"))
    zero_mad_delta = float(protocol.value("detector_zero_mad_delta"))
    rms_ratio = float(protocol.value("detector_coarse_rms_floor_ratio"))
    rms_percentile = float(protocol.value("detector_local_rms_percentile"))
    signal_floor = float(protocol.value("minimum_signal_peak_lsb")) * input_lsb
    peak_radius = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("detector_peak_search_radius_ms"))
    )

    candidates: list[_DetectorPeak] = []
    valid_times = timestamps[valid_indices]
    for hop_index_value in valid_indices:
        hop_index = int(hop_index_value)
        frame = int(timestamps[hop_index])
        deltas = np.abs(valid_times - frame)
        pool_indices = valid_indices[
            (deltas <= baseline_radius) & (deltas > baseline_exclusion)
        ]
        if pool_indices.size < minimum_baseline:
            continue
        novelty_pool = smoothed[pool_indices]
        center = _median(novelty_pool)
        spread = _mad(novelty_pool, center)
        threshold = center + mad_multiplier * mad_scale * spread
        value = float(smoothed[hop_index])
        if not value > threshold:
            continue
        if spread == 0.0 and not value > center + zero_mad_delta:
            continue
        local_rms = _lower_order_percentile(coarse_rms[pool_indices], rms_percentile)
        if coarse_rms[hop_index] < rms_ratio * local_rms:
            continue
        if coarse_rms[hop_index] < signal_floor:
            continue
        local_indices = valid_indices[np.abs(valid_times - frame) <= peak_radius]
        local_values = smoothed[local_indices]
        local_maximum = float(np.max(local_values))
        if value != local_maximum:
            continue
        if not _is_earliest_on_contiguous_plateau(smoothed, hop_index):
            continue
        candidates.append(_DetectorPeak(hop_index, frame, value))

    nms_radius = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("detector_nms_ms"))
    )
    retained: list[_DetectorPeak] = []
    for candidate in sorted(candidates, key=lambda item: (-item.novelty, item.frame)):
        if all(abs(candidate.frame - other.frame) >= nms_radius for other in retained):
            retained.append(candidate)
    retained.sort(key=lambda item: item.frame)
    candidates.sort(key=lambda item: item.frame)
    return _DetectorResult(tuple(candidates), tuple(retained))


def _persistence_passes(
    values: np.ndarray,
    start: int,
    length: int,
    predicate: Any,
    *,
    latest_inclusive: int | None = None,
) -> bool:
    end = start + length
    if start < 0 or end > values.size:
        return False
    if latest_inclusive is not None and end - 1 > latest_inclusive:
        return False
    segment = values[start:end]
    return bool(np.isfinite(segment).all() and np.all(predicate(segment)))


def _resolve_anatomy(
    peak: _DetectorPeak,
    q_squared: np.ndarray,
    r1: np.ndarray,
    r8: np.ndarray,
    r20: np.ndarray,
    sample_rate_hz: int,
    input_lsb: float,
    protocol: FrozenStageAProtocol,
) -> _Refinement:
    frame_count = int(q_squared.size)
    anchor_bounds = protocol.value("anchor_search_ms")
    search_start = peak.frame + _signed_offset_frames(
        protocol, sample_rate_hz, float(anchor_bounds[0])
    )
    search_end = peak.frame + _signed_offset_frames(
        protocol, sample_rate_hz, float(anchor_bounds[1])
    )
    lookbehind = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("lookbehind_ms"))
    )
    attack_search = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("attack_peak_search_ms"))
    )
    onset_persistence = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("onset_persistence_ms"))
    )
    baseline_multiplier = float(protocol.value("detector_mad_multiplier"))
    mad_scale = float(protocol.value("mad_consistency_scale"))
    peak_ratio = float(protocol.value("anatomy_peak_baseline_ratio"))
    onset_fraction = float(protocol.value("onset_fraction_above_baseline"))
    signal_floor = float(protocol.value("minimum_signal_peak_lsb")) * input_lsb

    physical_onset: int | None = None
    frozen_baseline = 0.0
    frozen_peak = 0.0
    for frame in range(search_start, search_end + 1):
        if frame - lookbehind < 0 or frame < 0 or frame + attack_search >= frame_count:
            continue
        baseline_values = r1[frame - lookbehind : frame]
        peak_values = r1[frame : frame + attack_search + 1]
        if not np.isfinite(baseline_values).all() or not np.isfinite(peak_values).all():
            continue
        median = _median(baseline_values)
        baseline = median + baseline_multiplier * mad_scale * _mad(
            baseline_values, median
        )
        local_peak = float(np.max(peak_values))
        if local_peak < peak_ratio * baseline or local_peak < signal_floor:
            continue
        threshold = baseline + onset_fraction * (local_peak - baseline)
        if not _persistence_passes(
            r1,
            frame,
            onset_persistence,
            lambda values, threshold=threshold: values >= threshold,
        ):
            continue
        physical_onset = frame
        frozen_baseline = baseline
        frozen_peak = local_peak
        break
    if physical_onset is None:
        refusal_reason = (
            EventRefusalReason.PHYSICAL_ONSET_UNRESOLVED
            if protocol.schema_version == 2
            else EventRefusalReason.EDGE_ONLY_IMPULSE
        )
        refusal = EventRefusal(
            peak.frame,
            refusal_reason,
            "no physical onset passed the frozen baseline, peak, and persistence gates",
        )
        return _Refinement(
            peak.frame,
            peak.novelty,
            None,
            None,
            None,
            None,
            None,
            refusal,
        )

    attack_region = r1[physical_onset : physical_onset + attack_search + 1]
    attack_peak_relative = int(np.flatnonzero(attack_region == np.max(attack_region))[0])
    attack_peak_frame = physical_onset + attack_peak_relative
    turnover_fraction = float(protocol.value("attack_turnover_fraction"))
    turnover_persistence = _duration_frames(
        protocol,
        sample_rate_hz,
        float(protocol.value("attack_turnover_persistence_ms")),
    )
    attack_limit = physical_onset + attack_search
    attack_end: int | None = None
    for frame in range(attack_peak_frame + 1, attack_limit + 1):
        if _persistence_passes(
            r1,
            frame,
            turnover_persistence,
            lambda values: values <= turnover_fraction * frozen_peak,
            latest_inclusive=attack_limit,
        ):
            attack_end = frame
            break
    if attack_end is None:
        refusal = EventRefusal(
            peak.frame,
            EventRefusalReason.ATTACK_TURNOVER_UNRESOLVED,
            "attack did not turn over inside the frozen attack window",
        )
        return _Refinement(
            peak.frame,
            peak.novelty,
            physical_onset,
            frozen_baseline,
            attack_peak_frame,
            frozen_peak,
            None,
            refusal,
        )

    body_floor = max(
        float(protocol.value("body_baseline_multiplier")) * frozen_baseline,
        float(protocol.value("body_peak_fraction")) * frozen_peak,
    )
    body_start = attack_end + _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("body_minimum_ms"))
    )
    body_persistence = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("body_below_floor_ms"))
    )
    body_limit = physical_onset + _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("body_maximum_ms"))
    )
    body_end: int | None = None
    for frame in range(body_start, min(body_limit, frame_count - 1) + 1):
        if _persistence_passes(
            r8,
            frame,
            body_persistence,
            lambda values: values < body_floor,
            latest_inclusive=body_limit,
        ):
            body_end = frame
            break
    if body_end is None:
        refusal = EventRefusal(
            peak.frame,
            EventRefusalReason.BODY_UNRESOLVED,
            "body did not resolve below the frozen floor",
        )
        return _Refinement(
            peak.frame,
            peak.novelty,
            physical_onset,
            frozen_baseline,
            attack_peak_frame,
            frozen_peak,
            None,
            refusal,
        )

    tail_floor = max(
        float(protocol.value("tail_baseline_multiplier")) * frozen_baseline,
        float(protocol.value("tail_peak_fraction")) * frozen_peak,
    )
    tail_start = body_end + _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("tail_minimum_ms"))
    )
    tail_persistence = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("tail_below_floor_ms"))
    )
    tail_limit = physical_onset + _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("tail_maximum_ms"))
    )
    tail_end: int | None = None
    for frame in range(tail_start, min(tail_limit, frame_count - 1) + 1):
        if _persistence_passes(
            r20,
            frame,
            tail_persistence,
            lambda values: values < tail_floor,
            latest_inclusive=tail_limit,
        ):
            tail_end = frame
            break
    if tail_end is None:
        refusal = EventRefusal(
            peak.frame,
            EventRefusalReason.TAIL_UNRESOLVED,
            "tail did not resolve below the frozen floor",
        )
        return _Refinement(
            peak.frame,
            peak.novelty,
            physical_onset,
            frozen_baseline,
            attack_peak_frame,
            frozen_peak,
            None,
            refusal,
        )
    anatomy = _Anatomy(
            coarse_peak_frame=peak.frame,
            coarse_novelty=peak.novelty,
            physical_onset_frame=physical_onset,
            lookbehind_start_frame=physical_onset - lookbehind,
            baseline_rms=frozen_baseline,
            attack_peak_frame=attack_peak_frame,
            attack_peak_rms=frozen_peak,
            attack_end_frame=attack_end,
            body_end_frame=body_end,
            tail_end_frame=tail_end,
        )
    return _Refinement(
        peak.frame,
        peak.novelty,
        physical_onset,
        frozen_baseline,
        attack_peak_frame,
        frozen_peak,
        anatomy,
        None,
    )


def _has_valley_before(
    earlier: _Refinement,
    later: _Refinement,
    r1: np.ndarray,
    sample_rate_hz: int,
    protocol: FrozenStageAProtocol,
) -> bool:
    if (
        earlier.physical_onset_frame is None
        or later.physical_onset_frame is None
        or earlier.attack_peak_rms is None
        or later.attack_peak_rms is None
    ):
        return False
    fusion = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("composite_fusion_ms"))
    )
    persistence = _duration_frames(
        protocol,
        sample_rate_hz,
        float(protocol.value("event_valley_persistence_ms")),
    )
    threshold = float(protocol.value("event_valley_peak_fraction")) * min(
        earlier.attack_peak_rms, later.attack_peak_rms
    )
    start = later.physical_onset_frame - fusion
    stop = later.physical_onset_frame
    for frame in range(max(0, start), stop):
        if _persistence_passes(
            r1,
            frame,
            persistence,
            lambda values: values <= threshold,
            latest_inclusive=stop - 1,
        ):
            return True
    return False


def _composite_ownership(
    refinements: Sequence[_Refinement],
    primary_peaks: Sequence[_DetectorPeak],
    r1: np.ndarray,
    sample_rate_hz: int,
    protocol: FrozenStageAProtocol,
) -> dict[int, tuple[int, EventRefusal | None, tuple[int, ...]]]:
    """Bind pre-NMS micropeaks to NMS-owned primary events.

    The returned mapping is keyed by the primary coarse-peak frame.  A
    pre-NMS peak may establish fused-composite anatomy or expose a separate
    event/flam, but it never becomes an independent catalog event.
    """

    if not primary_peaks:
        return {}
    fusion = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("composite_fusion_ms"))
    )
    primary_frames = tuple(peak.frame for peak in primary_peaks)
    owned: dict[int, list[_Refinement]] = {frame: [] for frame in primary_frames}
    for refinement in refinements:
        if refinement.physical_onset_frame is None:
            continue
        owner = min(
            primary_frames,
            key=lambda frame: (abs(refinement.coarse_peak_frame - frame), frame),
        )
        owned[owner].append(refinement)

    result: dict[int, tuple[int, EventRefusal | None, tuple[int, ...]]] = {}
    for primary_frame in primary_frames:
        members_by_onset: dict[int, _Refinement] = {}
        for member in owned[primary_frame]:
            assert member.physical_onset_frame is not None
            previous = members_by_onset.get(member.physical_onset_frame)
            if previous is None or (
                -member.coarse_novelty,
                member.coarse_peak_frame,
            ) < (-previous.coarse_novelty, previous.coarse_peak_frame):
                members_by_onset[member.physical_onset_frame] = member
        members = sorted(
            members_by_onset.values(),
            key=lambda item: (item.physical_onset_frame, item.coarse_peak_frame),
        )
        primary_index = next(
            (
                index
                for index, member in enumerate(members)
                if member.coarse_peak_frame == primary_frame
            ),
            None,
        )
        if primary_index is None:
            result[primary_frame] = (1, None, ())
            continue
        left = primary_index
        while left > 0:
            previous_onset = members[left - 1].physical_onset_frame
            current_onset = members[left].physical_onset_frame
            assert previous_onset is not None and current_onset is not None
            if current_onset - previous_onset > fusion:
                break
            left -= 1
        right = primary_index
        while right + 1 < len(members):
            current_onset = members[right].physical_onset_frame
            next_onset = members[right + 1].physical_onset_frame
            assert current_onset is not None and next_onset is not None
            if next_onset - current_onset > fusion:
                break
            right += 1
        refusal: EventRefusal | None = None
        separate_onsets: set[int] = set()
        for index in range(1, len(members)):
            if left <= index - 1 and index <= right:
                continue
            earlier = members[index - 1]
            later = members[index]
            if _has_valley_before(earlier, later, r1, sample_rate_hz, protocol):
                assert earlier.physical_onset_frame is not None
                assert later.physical_onset_frame is not None
                separate_onsets.update(
                    (earlier.physical_onset_frame, later.physical_onset_frame)
                )
                if refusal is None:
                    refusal = EventRefusal(
                        primary_frame,
                        EventRefusalReason.MULTI_EVENT_OR_FLAM,
                        "pre-NMS evidence resolves a separate event owned by one NMS primary",
                    )
        result[primary_frame] = (
            right - left + 1,
            refusal,
            tuple(sorted(separate_onsets)),
        )
    return result


def _rhythmic_proxy(
    anatomy: _Anatomy,
    r1: np.ndarray,
    sample_rate_hz: int,
    protocol: FrozenStageAProtocol,
) -> int | None:
    window = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("rhythmic_proxy_window_ms"))
    )
    stop = min(anatomy.body_end_frame, anatomy.physical_onset_frame + window)
    values = r1[anatomy.physical_onset_frame : stop]
    if values.size == 0 or not np.isfinite(values).all():
        return None
    weights = np.maximum(values - anatomy.baseline_rms, 0.0) ** 2
    total = float(np.sum(weights, dtype=np.float64))
    if not math.isfinite(total) or total <= 0.0:
        return None
    target = float(protocol.value("rhythmic_proxy_quantile")) * total
    relative = int(np.searchsorted(np.cumsum(weights, dtype=np.float64), target, side="left"))
    return anatomy.physical_onset_frame + relative


def _welch_band_fractions(
    x_prime: np.ndarray,
    sample_rate_hz: int,
    protocol: FrozenStageAProtocol,
) -> tuple[float, float, float] | None:
    window_frames = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("source_welch_window_ms"))
    )
    hop_frames = _duration_frames(
        protocol, sample_rate_hz, float(protocol.value("source_welch_hop_ms"))
    )
    if x_prime.shape[0] < window_frames:
        return None
    fft_size = _next_power_of_two(window_frames)
    window = _periodic_hann(protocol, window_frames)
    denominator = float(np.sum(window * window, dtype=np.float64))
    masks = _band_masks(protocol, sample_rate_hz, fft_size)
    totals = np.zeros(3, dtype=np.float64)
    for start in range(0, x_prime.shape[0] - window_frames + 1, hop_frames):
        block = x_prime[start : start + window_frames] * window[:, None]
        spectrum = np.fft.rfft(block, n=fft_size, axis=0)
        powers = np.sum(
            spectrum.real * spectrum.real + spectrum.imag * spectrum.imag,
            axis=1,
            dtype=np.float64,
        ) / denominator
        for index, mask in enumerate(masks):
            totals[index] += float(np.sum(powers[mask], dtype=np.float64))
    total = float(np.sum(totals, dtype=np.float64))
    if not np.isfinite(totals).all() or not math.isfinite(total) or total <= 0.0:
        return None
    fractions = totals / total
    return (float(fractions[0]), float(fractions[1]), float(fractions[2]))


def _bounded_map(value: float, scale: float) -> float:
    return 2.0 / math.pi * math.atan(value / scale)


def _source_features(
    x_prime: np.ndarray,
    sample_rate_hz: int,
    event_level_onsets: Sequence[int],
    resolved_events: Sequence[_Anatomy],
    protocol: FrozenStageAProtocol,
) -> SourceFeatureVector | None:
    minimum_onsets = int(protocol.value("source_minimum_onsets"))
    minimum_bodies = int(protocol.value("source_minimum_resolved_body_events"))
    if len(event_level_onsets) < minimum_onsets or len(resolved_events) < minimum_bodies:
        return None
    ordered_onsets = np.asarray(sorted(event_level_onsets), dtype=np.float64)
    intervals_ms = np.diff(ordered_onsets) * 1000.0 / float(sample_rate_hz)
    if intervals_ms.size == 0 or not np.isfinite(intervals_ms).all():
        return None
    median_ioi = _median(intervals_ms)
    mean_ioi = float(np.mean(intervals_ms, dtype=np.float64))
    if mean_ioi <= 0.0 or not math.isfinite(mean_ioi):
        return None
    population_cv = float(np.std(intervals_ms, ddof=0, dtype=np.float64)) / mean_ioi
    duration_seconds = x_prime.shape[0] / float(sample_rate_hz)
    density = len(event_level_onsets) / duration_seconds
    fractions = _welch_band_fractions(x_prime, sample_rate_hz, protocol)
    if fractions is None:
        return None

    articulation_ratios: list[float] = []
    durations_ms: list[float] = []
    channel_count = x_prime.shape[1]
    for event in resolved_events:
        attack = x_prime[event.physical_onset_frame : event.attack_end_frame]
        body = x_prime[event.attack_end_frame : event.body_end_frame]
        if attack.size == 0 or body.size == 0:
            return None
        attack_power = float(np.sum(attack * attack, dtype=np.float64)) / float(
            attack.shape[0] * channel_count
        )
        body_power = float(np.sum(body * body, dtype=np.float64)) / float(
            body.shape[0] * channel_count
        )
        if (
            not math.isfinite(attack_power)
            or not math.isfinite(body_power)
            or body_power <= 0.0
        ):
            return None
        articulation_ratios.append(attack_power / body_power)
        durations_ms.append(
            (event.tail_end_frame - event.physical_onset_frame)
            * 1000.0
            / float(sample_rate_hz)
        )
    articulation = _median(np.asarray(articulation_ratios, dtype=np.float64))
    duration = _median(np.asarray(durations_ms, dtype=np.float64))
    if articulation <= 0.0 or not math.isfinite(articulation) or not math.isfinite(duration):
        return None
    normalized_articulation = 0.5 + math.atan(math.log2(articulation)) / math.pi
    return SourceFeatureVector(
        all_event_onset_density_per_second=density,
        median_inter_onset_interval_ms=median_ioi,
        population_inter_onset_interval_cv=population_cv,
        low_mid_high_energy_fractions=fractions,
        median_attack_to_body_mean_power_ratio=articulation,
        median_resolved_event_duration_onset_to_tail_ms=duration,
        normalized_density=_bounded_map(
            density, float(protocol.value("normalization_density_scale_per_second"))
        ),
        normalized_ioi_rate=1.0
        - _bounded_map(
            median_ioi, float(protocol.value("normalization_ioi_scale_ms"))
        ),
        normalized_irregularity=_bounded_map(
            population_cv, float(protocol.value("normalization_ioi_cv_scale"))
        ),
        normalized_attack_body_articulation=normalized_articulation,
        normalized_duration=_bounded_map(
            duration, float(protocol.value("normalization_duration_scale_ms"))
        ),
    )


def _source_analysis_schema(protocol: FrozenStageAProtocol) -> str:
    return (
        "riotbox.percussive_force_source_analysis.v2"
        if protocol.schema_version == 2
        else "riotbox.percussive_force_source_analysis.v1"
    )


def _qualification_analysis_schema(protocol: FrozenStageAProtocol) -> str:
    return (
        "riotbox.percussive_force_stage_a_unbound_qualification_analysis.v2"
        if protocol.schema_version == 2
        else "riotbox.percussive_force_stage_a_unbound_qualification_analysis.v1"
    )


def _decode_authoritative_dc_means(
    bits: Sequence[str] | None,
    channel_count: int,
    protocol: FrozenStageAProtocol,
) -> np.ndarray | None:
    if protocol.schema_version == 1:
        if bits is not None:
            raise StageAContractError(
                "authoritative_dc_mean_not_allowed",
                "Protocol v1 requires its historical implementation-native mean",
            )
        return None
    if bits is None or len(bits) != channel_count:
        raise StageAContractError(
            "authoritative_dc_mean_missing",
            "Protocol v2 requires one frozen binary64 mean bit pattern per channel",
        )
    means: list[float] = []
    for channel, encoded in enumerate(bits):
        if (
            not isinstance(encoded, str)
            or len(encoded) != 16
            or encoded != encoded.lower()
            or any(character not in "0123456789abcdef" for character in encoded)
        ):
            raise StageAContractError(
                "invalid_authoritative_dc_mean",
                f"channel {channel} mean must be exactly 16 lowercase hex characters",
            )
        mean = struct.unpack(">d", bytes.fromhex(encoded))[0]
        if not math.isfinite(mean) or (mean == 0.0 and encoded != "0000000000000000"):
            raise StageAContractError(
                "invalid_authoritative_dc_mean",
                f"channel {channel} mean must be finite with canonical positive zero",
            )
        means.append(mean)
    return np.asarray(means, dtype=np.float64)


def _empty_analysis(
    metadata: SourceMetadata,
    protocol: FrozenStageAProtocol,
    sample_rate_hz: int,
    reason: SourceRefusalReason,
    detail: str,
    *,
    channel_count: int = 0,
    frame_count: int = 0,
    per_channel_dc_means: Sequence[float] = (),
) -> SourceAnalysis:
    return SourceAnalysis(
        schema=_source_analysis_schema(protocol),
        protocol_sha256=protocol.sha256,
        metadata=metadata,
        sample_rate_hz=sample_rate_hz,
        channel_count=channel_count,
        frame_count=frame_count,
        per_channel_dc_means=tuple(float(value) for value in per_channel_dc_means),
        detector=DetectorSummary(
            window_frames=0,
            hop_frames=0,
            log_energy_lag_hops=0,
            frame_timestamp_rule="start_frame_plus_floor_window_frames_over_two",
            pre_nms_peak_frames=(),
            nms_peak_frames=(),
        ),
        event_level_onset_frames=(),
        resolved_body_event_count=0,
        events=(),
        event_refusals=(),
        source_features=None,
        qualified=False,
        quality_proof=False,
        hardness_proof=False,
        refusals=(Refusal(reason, detail),),
    )


def analyze_source(
    case_metadata: SourceMetadata | Mapping[str, Any],
    samples: np.ndarray,
    sample_rate_hz: int,
    input_lsb: float,
    *,
    protocol: FrozenStageAProtocol | None = None,
    per_channel_dc_mean_f64_bits_be_hex: Sequence[str] | None = None,
) -> SourceAnalysis:
    """Run frozen mechanism-blind source/event qualification on in-memory PCM.

    ``samples`` must be normalized floating-point PCM in frame-major shape.
    No filename, title, or path field is consumed by the analysis.
    """

    frozen = _revalidate_frozen_protocol(protocol)
    metadata = SourceMetadata.coerce(case_metadata)
    if metadata.partition != "development":
        return _empty_analysis(
            metadata,
            frozen,
            int(sample_rate_hz),
            SourceRefusalReason.HOLDOUT_ACCESS_FORBIDDEN,
            "only an explicitly registered development partition may be analyzed",
        )
    sample_rate = int(sample_rate_hz)
    allowed_rates = frozen.value("input_sample_rate_range_hz")
    if sample_rate < int(allowed_rates[0]) or sample_rate > int(allowed_rates[1]):
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.UNSUPPORTED_SAMPLE_RATE,
            "native sample rate is outside the frozen inclusive range",
        )
    verified_format = metadata.verified_format
    if verified_format.valid_bits not in (16, 24):
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.UNSUPPORTED_FORMAT,
            "only verified signed PCM16 or PCM24 provenance is admitted",
        )
    expected_encoding = (
        PcmEncoding.PCM_S16LE
        if verified_format.valid_bits == 16
        else PcmEncoding.PCM_S24LE
    )
    expected_lsb = math.ldexp(1.0, -(verified_format.valid_bits - 1))
    format_is_exact = (
        verified_format.encoding is expected_encoding
        and verified_format.sample_rate_hz == sample_rate
        and verified_format.format_tag == 1
        and verified_format.container_bits == verified_format.valid_bits
        and verified_format.compression_type == "NONE"
        and verified_format.block_align
        == verified_format.channel_count * (verified_format.container_bits // 8)
        and verified_format.input_lsb == expected_lsb
    )
    if (
        not format_is_exact
        or
        not math.isfinite(float(input_lsb))
        or float(input_lsb) <= 0.0
        or float(input_lsb) != expected_lsb
    ):
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.UNSUPPORTED_FORMAT,
            "input_lsb does not match verified PCM valid_bits provenance",
        )
    raw = np.asarray(samples)
    if raw.ndim != 2:
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.UNSUPPORTED_CHANNEL_COUNT,
            "samples must have shape frames x channels",
        )
    frame_count, channel_count = raw.shape
    if frame_count == 0:
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.EMPTY_INPUT,
            "registered PCM is empty",
            channel_count=channel_count,
        )
    if channel_count not in {int(value) for value in frozen.value("input_channel_counts")}:
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.UNSUPPORTED_CHANNEL_COUNT,
            "channel count is outside the frozen supported set",
            channel_count=channel_count,
            frame_count=frame_count,
        )
    if verified_format.channel_count != channel_count:
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.UNSUPPORTED_FORMAT,
            "verified RIFF channel count does not match decoded PCM",
            channel_count=channel_count,
            frame_count=frame_count,
        )
    if not np.issubdtype(raw.dtype, np.floating):
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.UNSUPPORTED_FORMAT,
            "decode-independent input must be normalized floating-point PCM",
            channel_count=channel_count,
            frame_count=frame_count,
        )
    pcm = np.asarray(raw, dtype=np.float64)
    if not np.isfinite(pcm).all():
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.NONFINITE_ANALYSIS,
            "PCM contains a nonfinite sample",
            channel_count=channel_count,
            frame_count=frame_count,
        )
    authoritative_means = _decode_authoritative_dc_means(
        per_channel_dc_mean_f64_bits_be_hex,
        channel_count,
        frozen,
    )
    signal_floor = float(frozen.value("minimum_signal_peak_lsb")) * float(input_lsb)
    if not float(np.max(np.abs(pcm))) > signal_floor:
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.INSUFFICIENT_SIGNAL,
            "whole-source peak is not strictly above the frozen LSB floor",
            channel_count=channel_count,
            frame_count=frame_count,
            per_channel_dc_means=(
                authoritative_means if authoritative_means is not None else ()
            ),
        )

    means = (
        authoritative_means
        if authoritative_means is not None
        else np.mean(pcm, axis=0, dtype=np.float64)
    )
    x_prime = pcm - means[None, :]
    q_squared = np.mean(x_prime * x_prime, axis=1, dtype=np.float64)
    if not np.isfinite(q_squared).all():
        return _empty_analysis(
            metadata,
            frozen,
            sample_rate,
            SourceRefusalReason.NONFINITE_ANALYSIS,
            "DC-subtracted phase-safe power became nonfinite",
            channel_count=channel_count,
            frame_count=frame_count,
        )
    envelope_windows = [
        _duration_frames(frozen, sample_rate, float(ms))
        for ms in frozen.value("rms_envelope_windows_ms")
    ]
    if len(envelope_windows) != 3:
        raise StageAContractError(
            "envelope_contract_mismatch", "exactly R1, R8, and R20 are required"
        )
    r1, r8, r20 = (
        _causal_rms(q_squared, window) for window in envelope_windows
    )
    detector = _detect_events(x_prime, sample_rate, float(input_lsb), frozen)

    refinements = tuple(
        _resolve_anatomy(
            peak,
            q_squared,
            r1,
            r8,
            r20,
            sample_rate,
            float(input_lsb),
            frozen,
        )
        for peak in detector.pre_nms
    )
    by_coarse_frame = {item.coarse_peak_frame: item for item in refinements}
    primary_refinements = [
        by_coarse_frame[peak.frame]
        for peak in detector.nms
        if peak.frame in by_coarse_frame
    ]
    event_refusals = [
        item.refusal
        for item in primary_refinements
        if item.refusal is not None
    ]
    composite = _composite_ownership(
        refinements, detector.nms, r1, sample_rate, frozen
    )

    # NMS coarse peaks alone own primary catalog identities.  Pre-NMS
    # refinements may add an event-level onset only when the frozen composite
    # policy resolves a qualifying valley-separated event; they never become
    # independent primary catalog records.
    primary_by_onset: dict[int, _Refinement] = {}
    for item in primary_refinements:
        if item.physical_onset_frame is None:
            continue
        previous = primary_by_onset.get(item.physical_onset_frame)
        if previous is None or (
            -item.coarse_novelty,
            item.coarse_peak_frame,
        ) < (-previous.coarse_novelty, previous.coarse_peak_frame):
            primary_by_onset[item.physical_onset_frame] = item
    separate_event_onsets = {
        onset
        for _, _, onsets in composite.values()
        for onset in onsets
    }
    event_level_onsets = tuple(
        sorted(set(primary_by_onset).union(separate_event_onsets))
    )
    owned_primaries = sorted(
        primary_by_onset.values(),
        key=lambda item: (item.physical_onset_frame, item.coarse_peak_frame),
    )

    # A pre-NMS micropeak never owns a primary catalog record.  Once the
    # frozen composite policy resolves it as a separate event through a
    # qualifying valley, however, its physical onset must enter the exact
    # event-level sequence used by density/IOI.  A complete anatomy also
    # contributes to the whole-source resolved-event feature pool.
    feature_refinements = dict(primary_by_onset)
    for item in refinements:
        onset = item.physical_onset_frame
        if onset is None or onset not in separate_event_onsets:
            continue
        previous = feature_refinements.get(onset)
        if previous is None or (
            -item.coarse_novelty,
            item.coarse_peak_frame,
        ) < (-previous.coarse_novelty, previous.coarse_peak_frame):
            feature_refinements[onset] = item
        if item.refusal is not None and item.refusal not in event_refusals:
            event_refusals.append(item.refusal)
    resolved_events = tuple(
        item.anatomy
        for _, item in sorted(feature_refinements.items())
        if item.anatomy is not None
    )

    eligible: list[tuple[_Anatomy, ImpactRole, int, int]] = []
    lookbehind_peak_ratio = float(frozen.value("lookbehind_peak_ratio_max"))
    lookbehind_frames = _duration_frames(
        frozen, sample_rate, float(frozen.value("lookbehind_ms"))
    )
    for item in owned_primaries:
        if item.anatomy is None:
            continue
        anatomy = item.anatomy
        fused_count, composite_refusal, _ = composite.get(
            anatomy.coarse_peak_frame, (1, None, ())
        )
        if composite_refusal is not None:
            event_refusals.append(composite_refusal)
            continue
        prior_onset_inside = any(
            anatomy.lookbehind_start_frame <= onset < anatomy.physical_onset_frame
            for onset in event_level_onsets
        )
        lookbehind_power = q_squared[
            anatomy.lookbehind_start_frame : anatomy.physical_onset_frame
        ]
        lookbehind_rms = math.sqrt(
            float(np.mean(lookbehind_power, dtype=np.float64))
        )
        if prior_onset_inside or lookbehind_rms / anatomy.attack_peak_rms > lookbehind_peak_ratio:
            event_refusals.append(
                EventRefusal(
                    anatomy.coarse_peak_frame,
                    EventRefusalReason.LOOKBEHIND_MASKED,
                    "lookbehind contains an event onset or exceeds the frozen peak ratio",
                )
            )
            continue
        next_onset = next(
            (
                onset
                for onset in event_level_onsets
                if onset > anatomy.physical_onset_frame
            ),
            None,
        )
        next_lookbehind = (
            next_onset - lookbehind_frames if next_onset is not None else None
        )
        if next_lookbehind is not None and not (
            anatomy.body_end_frame < next_lookbehind
            and anatomy.tail_end_frame < next_lookbehind
        ):
            event_refusals.append(
                EventRefusal(
                    anatomy.coarse_peak_frame,
                    EventRefusalReason.OVERLAPPED_EVENT,
                    "body and tail do not resolve strictly before the next lookbehind",
                )
            )
            continue
        proxy = _rhythmic_proxy(anatomy, r1, sample_rate, frozen)
        if proxy is None:
            event_refusals.append(
                EventRefusal(
                    anatomy.coarse_peak_frame,
                    EventRefusalReason.INSUFFICIENT_SIGNAL,
                    "rhythmic-location proxy has zero or invalid weight",
                )
            )
            continue
        role = (
            ImpactRole.BODY_BEARING_FUSED_COMPOSITE_PERCUSSIVE
            if fused_count >= 2
            else ImpactRole.BODY_BEARING_SINGLE_PERCUSSIVE
        )
        eligible.append((anatomy, role, fused_count, proxy))

    eligible.sort(key=lambda item: (item[0].physical_onset_frame, item[0].coarse_peak_frame))
    maximum_events = int(frozen.value("maximum_frozen_events_per_source"))
    selected: list[tuple[_Anatomy, ImpactRole, int, int]] = []
    for candidate in eligible:
        anatomy = candidate[0]
        if any(
            not (
                anatomy.lookbehind_start_frame > prior.tail_end_frame
                or anatomy.tail_end_frame < prior.lookbehind_start_frame
            )
            for prior, _, _, _ in selected
        ):
            continue
        selected.append(candidate)
        if len(selected) == maximum_events:
            break
    records = tuple(
        EventRecord(
            ordinal=index + 1,
            coarse_peak_frame=anatomy.coarse_peak_frame,
            coarse_novelty=anatomy.coarse_novelty,
            physical_onset_frame=anatomy.physical_onset_frame,
            lookbehind_start_frame=anatomy.lookbehind_start_frame,
            baseline_rms=anatomy.baseline_rms,
            attack_peak_frame=anatomy.attack_peak_frame,
            attack_peak_rms=anatomy.attack_peak_rms,
            attack_end_frame=anatomy.attack_end_frame,
            body_end_frame=anatomy.body_end_frame,
            tail_end_frame=anatomy.tail_end_frame,
            rhythmic_proxy_frame=proxy,
            impact_role=role,
            refined_micropeak_count=count,
        )
        for index, (anatomy, role, count, proxy) in enumerate(selected)
    )
    features = _source_features(
        x_prime, sample_rate, event_level_onsets, resolved_events, frozen
    )
    refusals: list[Refusal] = []
    if len(records) < int(frozen.value("minimum_events_per_source")):
        refusals.append(
            Refusal(
                SourceRefusalReason.INSUFFICIENT_ELIGIBLE_EVENTS,
                "fewer than the frozen minimum eligible non-overlapping events",
            )
        )
    if features is None:
        refusals.append(
            Refusal(
                SourceRefusalReason.SOURCE_FEATURE_REQUIREMENTS_UNMET,
                "whole-source contrast vector is undefined without imputation",
            )
        )
    return SourceAnalysis(
        schema=_source_analysis_schema(frozen),
        protocol_sha256=frozen.sha256,
        metadata=metadata,
        sample_rate_hz=sample_rate,
        channel_count=channel_count,
        frame_count=frame_count,
        per_channel_dc_means=tuple(float(value) for value in means),
        detector=DetectorSummary(
            window_frames=_duration_frames(
                frozen, sample_rate, float(frozen.value("detector_window_ms"))
            ),
            hop_frames=_duration_frames(
                frozen, sample_rate, float(frozen.value("detector_hop_ms"))
            ),
            log_energy_lag_hops=int(frozen.value("detector_log_energy_lag_hops")),
            frame_timestamp_rule="start_frame_plus_floor_window_frames_over_two",
            pre_nms_peak_frames=tuple(peak.frame for peak in detector.pre_nms),
            nms_peak_frames=tuple(peak.frame for peak in detector.nms),
        ),
        event_level_onset_frames=event_level_onsets,
        resolved_body_event_count=len(resolved_events),
        events=records,
        event_refusals=tuple(event_refusals),
        source_features=features,
        qualified=not refusals,
        quality_proof=False,
        hardness_proof=False,
        refusals=tuple(refusals),
    )


def _pair_contrast(
    left: SourceAnalysis,
    right: SourceAnalysis,
    protocol: FrozenStageAProtocol,
) -> PairContrast:
    if left.source_features is None or right.source_features is None:
        raise StageAContractError(
            "missing_source_features", "pair contrast requires two complete vectors"
        )
    a = left.source_features
    b = right.source_features
    rate = math.sqrt(
        (
            (a.normalized_density - b.normalized_density) ** 2
            + (a.normalized_ioi_rate - b.normalized_ioi_rate) ** 2
        )
        / 2.0
    )
    irregularity = abs(a.normalized_irregularity - b.normalized_irregularity)
    affinity = sum(
        math.sqrt(left_value * right_value)
        for left_value, right_value in zip(
            a.low_mid_high_energy_fractions,
            b.low_mid_high_energy_fractions,
            strict=True,
        )
    )
    spectrum = math.sqrt(max(0.0, 1.0 - affinity))
    articulation = abs(
        a.normalized_attack_body_articulation
        - b.normalized_attack_body_articulation
    )
    duration = abs(a.normalized_duration - b.normalized_duration)
    domains = (rate, irregularity, spectrum, articulation, duration)
    domain_count = int(protocol.value("source_distance_domain_count"))
    if domain_count != len(domains):
        raise StageAContractError(
            "source_domain_count_mismatch", "frozen source domain count changed"
        )
    overall = math.sqrt(sum(value * value for value in domains) / domain_count)
    change_floor = float(protocol.value("source_changed_domain_min_delta"))
    changed = sum(value >= change_floor for value in domains)
    distance_floor = float(protocol.value("source_distinct_distance_min"))
    required_changes = int(protocol.value("source_changed_domain_minimum_count"))
    if overall < distance_floor:
        classification = PairClassification.SIMILAR
    elif changed >= required_changes:
        classification = PairClassification.DISTINCT
    else:
        classification = PairClassification.AMBIGUOUS
    left_id, right_id = sorted((left.metadata.case_id, right.metadata.case_id))
    return PairContrast(
        left_case_id=left_id,
        right_case_id=right_id,
        rate_delta=rate,
        irregularity_delta=irregularity,
        spectrum_hellinger=spectrum,
        articulation_delta=articulation,
        duration_delta=duration,
        overall_distance=overall,
        changed_domain_count=changed,
        classification=classification,
    )


def _set_partitions(case_ids: Sequence[str]) -> list[tuple[tuple[str, ...], ...]]:
    partitions: list[tuple[tuple[str, ...], ...]] = []

    def visit(index: int, blocks: list[list[str]]) -> None:
        if index == len(case_ids):
            partitions.append(tuple(tuple(block) for block in blocks))
            return
        value = case_ids[index]
        for block_index in range(len(blocks)):
            blocks[block_index].append(value)
            visit(index + 1, blocks)
            blocks[block_index].pop()
        blocks.append([value])
        visit(index + 1, blocks)
        blocks.pop()

    visit(0, [])
    return partitions


def _qualification_refusal(
    protocol: FrozenStageAProtocol,
    sources: Sequence[SourceAnalysis],
    reason: QualificationRefusalReason,
    detail: str,
    *,
    pairs: Sequence[PairContrast] = (),
    partitions: Sequence[SourcePartition] = (),
) -> StageAQualification:
    return StageAQualification(
        schema=_qualification_analysis_schema(protocol),
        qualification_state="unbound_analysis_only",
        protocol_sha256=protocol.sha256,
        sources=tuple(sources),
        pair_contrasts=tuple(pairs),
        valid_partitions=tuple(partitions),
        passed=False,
        quality_proof=False,
        hardness_proof=False,
        next_allowed_action="stop_without_candidate_render",
        refusals=(Refusal(reason, detail),),
    )


def qualify_four_sources(
    sources: Sequence[SourceInput],
    *,
    protocol: FrozenStageAProtocol | None = None,
) -> StageAQualification:
    """Run unbound mechanism-blind analysis for exactly four source inputs.

    A passing result still requires the separate exact Registry/Matrix/Session
    binding wrapper before it may freeze an event catalog or permit the already
    preregistered candidate matrix.  This function never grants force, hardness,
    human, or product-path evidence.
    """

    frozen = _revalidate_frozen_protocol(protocol)
    expected_count = int(frozen.value("positive_source_count"))
    if len(sources) != expected_count:
        return _qualification_refusal(
            frozen,
            (),
            QualificationRefusalReason.SOURCE_COUNT,
            f"expected exactly {expected_count} development sources",
        )
    metadata = [SourceMetadata.coerce(source.metadata) for source in sources]
    case_ids = [item.case_id for item in metadata]
    if len(case_ids) != len(set(case_ids)):
        return _qualification_refusal(
            frozen,
            (),
            QualificationRefusalReason.DUPLICATE_CASE_ID,
            "source case IDs must be unique",
        )
    expected_authors = int(frozen.value("positive_author_count"))
    if len({item.author.casefold() for item in metadata}) != expected_authors:
        return _qualification_refusal(
            frozen,
            (),
            QualificationRefusalReason.AUTHOR_COUNT,
            f"expected exactly {expected_authors} distinct authors",
        )
    expected_families = int(frozen.value("positive_family_count"))
    if len({item.source_family for item in metadata}) != expected_families:
        return _qualification_refusal(
            frozen,
            (),
            QualificationRefusalReason.FAMILY_COUNT,
            f"expected exactly {expected_families} source families",
        )
    analyses = tuple(
        analyze_source(
            source.metadata,
            source.samples,
            source.sample_rate_hz,
            source.input_lsb,
            protocol=frozen,
            per_channel_dc_mean_f64_bits_be_hex=(
                source.per_channel_dc_mean_f64_bits_be_hex
            ),
        )
        for source in sources
    )
    failures = [analysis.metadata.case_id for analysis in analyses if not analysis.qualified]
    if failures:
        return _qualification_refusal(
            frozen,
            analyses,
            QualificationRefusalReason.SOURCE_FAILED,
            f"positive source qualification failed: {failures}",
        )

    pairs: list[PairContrast] = []
    by_pair: dict[frozenset[str], PairClassification] = {}
    for left_index in range(len(analyses)):
        for right_index in range(left_index + 1, len(analyses)):
            pair = _pair_contrast(analyses[left_index], analyses[right_index], frozen)
            pairs.append(pair)
            by_pair[frozenset((pair.left_case_id, pair.right_case_id))] = pair.classification

    ordered_ids = tuple(analysis.metadata.case_id for analysis in analyses)
    all_partitions = _set_partitions(ordered_ids)
    expected_partitions = int(frozen.value("four_source_partition_count"))
    if len(all_partitions) != expected_partitions:
        raise StageAContractError(
            "partition_enumeration_mismatch",
            f"expected {expected_partitions} set partitions, got {len(all_partitions)}",
        )
    valid: list[SourcePartition] = []
    for partition in all_partitions:
        cluster_for = {
            case_id: cluster_index
            for cluster_index, cluster in enumerate(partition)
            for case_id in cluster
        }
        accepted = True
        for left_index in range(len(ordered_ids)):
            for right_index in range(left_index + 1, len(ordered_ids)):
                left_id = ordered_ids[left_index]
                right_id = ordered_ids[right_index]
                classification = by_pair[frozenset((left_id, right_id))]
                same_cluster = cluster_for[left_id] == cluster_for[right_id]
                if same_cluster and classification is not PairClassification.SIMILAR:
                    accepted = False
                if not same_cluster and classification is not PairClassification.DISTINCT:
                    accepted = False
        if accepted:
            valid.append(SourcePartition(tuple(tuple(cluster) for cluster in partition)))

    required_valid = int(frozen.value("valid_source_partition_count"))
    minimum_clusters = int(frozen.value("minimum_source_clusters"))
    if len(valid) != required_valid or any(
        len(partition.clusters) < minimum_clusters for partition in valid
    ):
        return _qualification_refusal(
            frozen,
            analyses,
            QualificationRefusalReason.PARTITION_GATE,
            (
                f"required exactly {required_valid} valid partition with at least "
                f"{minimum_clusters} clusters; observed {len(valid)}"
            ),
            pairs=pairs,
            partitions=valid,
        )
    return StageAQualification(
        schema=_qualification_analysis_schema(frozen),
        qualification_state="unbound_analysis_only",
        protocol_sha256=frozen.sha256,
        sources=analyses,
        pair_contrasts=tuple(pairs),
        valid_partitions=tuple(valid),
        passed=True,
        quality_proof=False,
        hardness_proof=False,
        next_allowed_action=(
            "bind_exact_stage_a_qualification_session_before_event_catalog_or_candidate_matrix"
        ),
        refusals=(),
    )


__all__ = [
    "CANONICAL_PROTOCOL_PATH",
    "DetectorSummary",
    "EXPECTED_PROTOCOL_SHA256",
    "EventRecord",
    "EventRefusal",
    "EventRefusalReason",
    "FrozenStageAProtocol",
    "ImpactRole",
    "PairClassification",
    "PairContrast",
    "PcmEncoding",
    "QualificationRefusalReason",
    "Refusal",
    "SourceAnalysis",
    "SourceFeatureVector",
    "SourceInput",
    "SourceMetadata",
    "SourcePartition",
    "SourceRefusalReason",
    "StageAContractError",
    "StageAQualification",
    "VerifiedPcmFormat",
    "analyze_source",
    "load_frozen_protocol",
    "qualify_four_sources",
]
