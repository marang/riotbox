#!/usr/bin/env python3
"""Synthetic, in-memory fixtures for Stage-A source qualification analysis.

No fixture opens, enumerates, hashes, renders, or plays a real source or
holdout file.  The only repository payload read is the frozen protocol JSON.
"""

from __future__ import annotations

import hashlib
import math

import numpy as np

import percussive_force_stage_a_analysis as analysis


SAMPLE_RATE = 48_000
CHANNEL_COUNT = 2
INPUT_LSB = math.ldexp(1.0, -15)


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def _expect_contract_error(operation: object, code: str) -> None:
    try:
        operation()  # type: ignore[operator]
    except analysis.StageAContractError as error:
        _require(error.code == code, f"expected {code}, got {error.code}")
        return
    raise AssertionError(f"expected StageAContractError {code}")


def _source(
    onsets_seconds: tuple[float, ...],
    decay_seconds: float,
    band_weights: tuple[float, float, float],
    *,
    duration_seconds: float = 2.0,
    doubled_offset_seconds: float | None = None,
    sample_rate_hz: int = SAMPLE_RATE,
) -> np.ndarray:
    result = np.zeros(
        (round(duration_seconds * sample_rate_hz), CHANNEL_COUNT), dtype=np.float64
    )

    def add_hit(onset_seconds: float, amplitude: float) -> None:
        start = round(onset_seconds * sample_rate_hz)
        length = min(round(0.35 * sample_rate_hz), result.shape[0] - start)
        frames = np.arange(length, dtype=np.float64)
        seconds = frames / sample_rate_hz
        envelope = np.exp(-seconds / decay_seconds)
        low, middle, high = band_weights
        carrier = (
            low * np.cos(2.0 * np.pi * 100.0 * seconds)
            + middle * np.cos(2.0 * np.pi * 900.0 * seconds)
            + high * np.cos(2.0 * np.pi * 5_000.0 * seconds)
        )
        hit = amplitude * envelope * carrier
        result[start : start + length, 0] += hit
        result[start : start + length, 1] += 0.97 * hit

    for onset in onsets_seconds:
        add_hit(onset, 0.65)
        if doubled_offset_seconds is not None:
            add_hit(onset + doubled_offset_seconds, 0.50)
    return result


def _metadata(
    case_id: str,
    family: str,
    author: str,
    samples: np.ndarray,
    *,
    partition: str = "development",
    source_path: str | None = None,
    sample_rate_hz: int = SAMPLE_RATE,
    valid_bits: int = 16,
) -> analysis.SourceMetadata:
    return analysis.SourceMetadata(
        case_id=case_id,
        source_family=family,
        author=author,
        source_path=source_path or f"memory/{case_id}.wav",
        source_sha256=hashlib.sha256(samples.tobytes()).hexdigest(),
        license="synthetic-test-only",
        verified_format=analysis.VerifiedPcmFormat.signed_pcm(
            valid_bits=valid_bits,
            sample_rate_hz=sample_rate_hz,
            channel_count=CHANNEL_COUNT,
        ),
        partition=partition,
    )


def _analyze(
    case_id: str,
    family: str,
    author: str,
    samples: np.ndarray,
    protocol: analysis.FrozenStageAProtocol,
    *,
    sample_rate_hz: int = SAMPLE_RATE,
    valid_bits: int = 16,
) -> analysis.SourceAnalysis:
    input_lsb = math.ldexp(1.0, -(valid_bits - 1))
    return analysis.analyze_source(
        _metadata(
            case_id,
            family,
            author,
            samples,
            sample_rate_hz=sample_rate_hz,
            valid_bits=valid_bits,
        ),
        samples,
        sample_rate_hz,
        input_lsb,
        protocol=protocol,
    )


def protocol_and_numeric_fixtures() -> analysis.FrozenStageAProtocol:
    protocol = analysis.load_frozen_protocol()
    payload = analysis.CANONICAL_PROTOCOL_PATH.read_bytes()
    _expect_contract_error(
        lambda: analysis.FrozenStageAProtocol.from_bytes(payload + b" "),
        "protocol_pin_mismatch",
    )
    _require(
        analysis._duration_frames(protocol, 44_100, 8.0) == 353,
        "positive-duration nearest-frame conversion drifted",
    )
    _require(
        analysis._signed_offset_frames(protocol, 44_100, -20.0) == -882,
        "signed offset conversion drifted",
    )
    _require(
        analysis._signed_offset_frames(protocol, 44_100, 0.0) == 0,
        "signed zero offset must remain zero",
    )
    _require(
        analysis._median(np.asarray([1.0, 2.0, 9.0, 10.0])) == 5.5,
        "even-count median must average the two middle values",
    )
    _require(
        analysis._lower_order_percentile(np.arange(6.0), 20.0) == 1.0,
        "20th-percentile floor-index convention drifted",
    )
    separated_equal_maxima = np.asarray([0.0, 2.0, 2.0, 1.0, 2.0, 2.0, 0.0])
    _require(
        analysis._is_earliest_on_contiguous_plateau(separated_equal_maxima, 1)
        and not analysis._is_earliest_on_contiguous_plateau(
            separated_equal_maxima, 2
        )
        and analysis._is_earliest_on_contiguous_plateau(separated_equal_maxima, 4)
        and not analysis._is_earliest_on_contiguous_plateau(
            separated_equal_maxima, 5
        ),
        "equal maxima separated by a valley must remain distinct plateaus",
    )
    return protocol


def fail_closed_input_fixtures(protocol: analysis.FrozenStageAProtocol) -> None:
    quiet = np.full((SAMPLE_RATE, CHANNEL_COUNT), 16.0 * INPUT_LSB)
    quiet[::2] *= -1.0
    quiet_result = _analyze("floor", "dense_break", "floor-author", quiet, protocol)
    _require(
        quiet_result.refusals[0].reason
        is analysis.SourceRefusalReason.INSUFFICIENT_SIGNAL,
        "the whole-source LSB comparator must remain strict",
    )

    nonfinite = quiet.copy()
    nonfinite[10, 0] = np.nan
    nonfinite_result = _analyze(
        "nonfinite", "dense_break", "nan-author", nonfinite, protocol
    )
    _require(
        nonfinite_result.refusals[0].reason
        is analysis.SourceRefusalReason.NONFINITE_ANALYSIS,
        "nonfinite PCM must refuse without imputation",
    )

    class MustNotOpen:
        def __array__(self, *_args: object, **_kwargs: object) -> np.ndarray:
            raise AssertionError("holdout array was accessed")

    holdout_pcm = np.zeros((1, CHANNEL_COUNT), dtype=np.float64)
    holdout = _metadata(
        "holdout",
        "dense_break",
        "holdout-author",
        holdout_pcm,
        partition="holdout_a",
    )
    holdout_result = analysis.analyze_source(
        holdout,
        MustNotOpen(),  # type: ignore[arg-type]
        SAMPLE_RATE,
        INPUT_LSB,
        protocol=protocol,
    )
    _require(
        holdout_result.refusals[0].reason
        is analysis.SourceRefusalReason.HOLDOUT_ACCESS_FORBIDDEN,
        "holdout partition must refuse before touching samples",
    )


def detector_catalog_and_provenance_fixtures(
    protocol: analysis.FrozenStageAProtocol,
) -> np.ndarray:
    source = _source((0.30, 0.70, 1.10, 1.50), 0.025, (0.75, 0.20, 0.05))
    result = _analyze("golden", "dense_break", "author-a", source, protocol)
    _require(result.qualified, f"synthetic golden source refused: {result.refusals}")
    _require(result.detector.window_frames == 384, "8ms detector window drifted")
    _require(result.detector.hop_frames == 48, "1ms detector hop drifted")
    _require(result.detector.log_energy_lag_hops == 4, "4-hop novelty lag drifted")
    _require(
        result.detector.pre_nms_peak_frames[0] == 14_352,
        "detector timestamp/lag golden drifted",
    )
    _require(
        all(
            (frame - result.detector.window_frames // 2)
            % result.detector.hop_frames
            == 0
            for frame in result.detector.pre_nms_peak_frames
        ),
        "detector timestamps must be start+floor(window/2)",
    )
    _require(
        result.event_level_onset_frames == (14_400, 33_600, 52_800, 72_000),
        "NMS-owned physical onset golden drifted",
    )
    _require(
        result.resolved_body_event_count == 4 and len(result.events) == 3,
        "whole-source feature events must remain separate from max-3 catalog freeze",
    )
    _require(
        result.source_features is not None
        and result.source_features.all_event_onset_density_per_second == 2.0,
        "source density must use all four event-level onsets",
    )
    encoded = result.to_dict()
    _require(
        encoded["metadata"]["source_path"] == "memory/golden.wav"
        and encoded["metadata"]["source_sha256"]
        == hashlib.sha256(source.tobytes()).hexdigest()
        and encoded["metadata"]["license"] == "synthetic-test-only"
        and encoded["metadata"]["verified_format"]["valid_bits"] == 16,
        "catalog serialization lost source provenance",
    )

    alternate_metadata = analysis.SourceMetadata(
        case_id="golden",
        source_family="dense_break",
        author="author-a",
        source_path="renamed/without_branching.wav",
        source_sha256="f" * 64,
        license="record-only-alternate",
        verified_format=result.metadata.verified_format,
    )
    alternate = analysis.analyze_source(
        alternate_metadata,
        source,
        SAMPLE_RATE,
        INPUT_LSB,
        protocol=protocol,
    )
    _require(
        alternate.event_level_onset_frames == result.event_level_onset_frames
        and alternate.events == result.events
        and alternate.source_features == result.source_features,
        "path/SHA/license provenance must be recorded without branching analysis",
    )
    return source


def nms_and_composite_fixtures(protocol: analysis.FrozenStageAProtocol) -> None:
    doubled = _source(
        (0.35, 0.75, 1.15, 1.55),
        0.018,
        (0.50, 0.30, 0.20),
        doubled_offset_seconds=0.010,
    )
    result = _analyze("doubled", "dense_break", "double-author", doubled, protocol)
    _require(result.qualified, f"doubled synthetic source refused: {result.refusals}")
    _require(
        len(result.detector.pre_nms_peak_frames)
        > len(result.detector.nms_peak_frames)
        >= len(result.event_level_onset_frames),
        "pre-NMS peaks must not silently own catalog events",
    )
    _require(
        result.event_level_onset_frames == (16_800, 36_000, 55_200, 74_400),
        "suppressed pre-NMS peaks became independent source events",
    )

    def refinement(coarse: int, onset: int) -> analysis._Refinement:
        return analysis._Refinement(
            coarse_peak_frame=coarse,
            coarse_novelty=1.0,
            physical_onset_frame=onset,
            baseline_rms=0.0,
            attack_peak_frame=onset,
            attack_peak_rms=1.0,
            anatomy=None,
            refusal=None,
        )

    primary = analysis._DetectorPeak(0, 1_000, 1.0)
    r1 = np.zeros(4_000, dtype=np.float64)
    fused = analysis._composite_ownership(
        (refinement(1_000, 1_000), refinement(1_020, 1_400)),
        (primary,),
        r1,
        SAMPLE_RATE,
        protocol,
    )
    _require(
        fused[1_000][0] == 2
        and fused[1_000][1] is None
        and fused[1_000][2] == (),
        "<=12ms refined micropeaks must establish the fused role",
    )
    flam = analysis._composite_ownership(
        (refinement(1_000, 1_000), refinement(1_020, 1_800)),
        (primary,),
        r1,
        SAMPLE_RATE,
        protocol,
    )
    _require(
        flam[1_000][1] is not None
        and flam[1_000][1].reason is analysis.EventRefusalReason.MULTI_EVENT_OR_FLAM,
        ">12ms peak plus frozen valley must refuse as multi-event/flam",
    )
    _require(
        flam[1_000][2] == (1_000, 1_800),
        "valley-separated pre-NMS evidence must enter the event-level onset sequence",
    )


def partial_onset_fixture(protocol: analysis.FrozenStageAProtocol) -> None:
    source = _source((0.35, 0.75, 1.15), 0.025, (0.55, 0.28, 0.17))
    start = round(1.55 * SAMPLE_RATE)
    stop = start + round(0.20 * SAMPLE_RATE)
    source[start:stop, 0] += 0.40
    source[start:stop, 1] += 0.38
    result = _analyze("partial", "dense_break", "partial-author", source, protocol)
    _require(result.qualified, f"partial-onset source refused: {result.refusals}")
    _require(
        len(result.event_level_onset_frames) == 4
        and result.resolved_body_event_count == 3
        and len(result.events) == 3,
        "partial physical onset must count for density/IOI without becoming resolved body",
    )
    _require(
        any(
            refusal.reason is analysis.EventRefusalReason.ATTACK_TURNOVER_UNRESOLVED
            for refusal in result.event_refusals
        ),
        "partial-onset fixture did not exercise attack-turnover refusal",
    )


def valley_separated_event_level_fixture(
    protocol: analysis.FrozenStageAProtocol,
) -> None:
    source = _source(
        (0.30, 0.70, 1.10, 1.50),
        0.002,
        (0.50, 0.30, 0.20),
        doubled_offset_seconds=0.027,
    )
    result = _analyze(
        "valley-separated",
        "dense_break",
        "valley-author",
        source,
        protocol,
    )
    _require(
        len(result.event_level_onset_frames) == 8,
        "valley-separated pre-NMS onsets must enter the event-level sequence",
    )
    _require(
        result.source_features is not None
        and result.source_features.all_event_onset_density_per_second == 4.0
        and result.resolved_body_event_count == 8,
        "valley-separated events must feed whole-source density and resolved features",
    )
    _require(
        not result.qualified and len(result.events) == 0,
        "valley-separated support must not become an independent primary catalog record",
    )


def unique_partition_fixture(protocol: analysis.FrozenStageAProtocol) -> None:
    specifications = (
        (
            "a",
            "dense_break",
            "author-a",
            (0.30, 0.70, 1.10, 1.50),
            0.025,
            (0.75, 0.20, 0.05),
            48_000,
            16,
        ),
        (
            "b",
            "dense_break",
            "author-b",
            (0.30, 0.70, 1.10, 1.50),
            0.026,
            (0.73, 0.21, 0.06),
            48_000,
            16,
        ),
        (
            "c",
            "sparse_drums",
            "author-c",
            (0.25, 0.47, 0.69, 0.91, 1.13, 1.35, 1.57),
            0.010,
            (0.05, 0.10, 0.85),
            44_100,
            24,
        ),
        (
            "d",
            "electronic_drums",
            "author-d",
            (0.25, 0.55, 1.35, 1.70),
            0.070,
            (0.05, 0.90, 0.05),
            44_100,
            16,
        ),
    )
    source_inputs: list[analysis.SourceInput] = []
    for (
        case_id,
        family,
        author,
        onsets,
        decay,
        weights,
        sample_rate_hz,
        valid_bits,
    ) in specifications:
        samples = _source(onsets, decay, weights, sample_rate_hz=sample_rate_hz)
        input_lsb = math.ldexp(1.0, -(valid_bits - 1))
        source_inputs.append(
            analysis.SourceInput(
                metadata=_metadata(
                    case_id,
                    family,
                    author,
                    samples,
                    sample_rate_hz=sample_rate_hz,
                    valid_bits=valid_bits,
                ),
                samples=samples,
                sample_rate_hz=sample_rate_hz,
                input_lsb=input_lsb,
            )
        )
    result = analysis.qualify_four_sources(source_inputs, protocol=protocol)
    _require(result.passed, f"four-source contrast gate refused: {result.refusals}")
    _require(len(analysis._set_partitions(("a", "b", "c", "d"))) == 15, "Bell(4) drifted")
    _require(len(result.pair_contrasts) == 6, "four sources require six pairs")
    _require(
        tuple(source.sample_rate_hz for source in result.sources)
        == (48_000, 48_000, 44_100, 44_100)
        and result.sources[2].metadata.verified_format.valid_bits == 24
        and result.sources[2].metadata.verified_format.input_lsb
        == math.ldexp(1.0, -23),
        "mixed native-rate PCM16/PCM24 qualification binding drifted",
    )
    _require(
        result.valid_partitions
        == (analysis.SourcePartition((("a", "b"), ("c",), ("d",))),),
        "unique >=3-cluster partition golden drifted",
    )
    classifications = {
        frozenset((pair.left_case_id, pair.right_case_id)): pair.classification
        for pair in result.pair_contrasts
    }
    _require(
        classifications[frozenset(("a", "b"))]
        is analysis.PairClassification.SIMILAR
        and all(
            classification is analysis.PairClassification.DISTINCT
            for pair, classification in classifications.items()
            if pair != frozenset(("a", "b"))
        ),
        "pair-classification golden drifted",
    )
    _require(
        not result.quality_proof
        and not result.hardness_proof
        and result.qualification_state == "unbound_analysis_only"
        and result.next_allowed_action.startswith("bind_exact_stage_a"),
        "unbound analysis must not grant catalog, matrix, hardness, or quality proof",
    )
    result.to_json(indent=None)


def qualification_shape_failures(protocol: analysis.FrozenStageAProtocol) -> None:
    no_array = np.zeros((1, CHANNEL_COUNT), dtype=np.float64)
    one = analysis.SourceInput(
        _metadata("one", "dense_break", "one-author", no_array),
        no_array,
        SAMPLE_RATE,
        INPUT_LSB,
    )
    result = analysis.qualify_four_sources((one,), protocol=protocol)
    _require(
        result.refusals[0].reason is analysis.QualificationRefusalReason.SOURCE_COUNT,
        "wrong source count must stop before source analysis",
    )


def main() -> int:
    protocol = protocol_and_numeric_fixtures()
    fail_closed_input_fixtures(protocol)
    detector_catalog_and_provenance_fixtures(protocol)
    nms_and_composite_fixtures(protocol)
    partial_onset_fixture(protocol)
    valley_separated_event_level_fixture(protocol)
    qualification_shape_failures(protocol)
    unique_partition_fixture(protocol)
    print("PASS: RIOTBOX-1428 synthetic Stage-A analysis fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
