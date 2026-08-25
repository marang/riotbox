#!/usr/bin/env python3
"""Source-blind falsification and bounded Development render for Dense v3."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any

import numpy as np

import generate_dense_break_foundation_clarity_v2 as clarity
import generate_dense_break_performance_pack as dense
import percussive_force_stage_a_analysis as stage_a


CONTRACT = Path("docs/benchmarks/dense_break_foundation_chop_v3.json")
CONTRACT_SHA256 = "7bcd88833101988fbb08b9761d169f234163abc40f1f9524ca5d478a597248a2"
REGISTRY = Path("docs/benchmarks/source_holdout_rotation_v2.json")
CASE_ID = "dense_beat03_130"
ANSWER_START_BEAT = 6
LOOKBEHIND_MS = 20.0
CROSSFADE_MS = 2.0
TARGET_ROLES = ("low_body", "upper_attack")
MAX_ACCENT_RANK_DISTANCE = 1
CONTEXT_LOW_BAND_LIMIT_DB = 4.0
CONTEXT_LEVEL_LIMIT_DB = 4.0
ATTACK_LEVEL_LIMIT_DB = 3.0
ANSWER_LOW_BAND_LIMIT_DB = 3.0
ANSWER_LEVEL_LIMIT_DB = 3.0
LOCAL_LOW_BAND_LIMIT_DB = 4.0
LOCAL_LEVEL_LIMIT_DB = 4.0
MIN_NORMALIZED_DELTA = 0.20
MAX_ABSOLUTE_CORRELATION = 0.96


class TechnicalRejection(ValueError):
    def __init__(self, code: str, detail: str, evidence: dict[str, Any] | None = None):
        super().__init__(f"{code}: {detail}")
        self.code = code
        self.detail = detail
        self.evidence = evidence or {}


@dataclass(frozen=True)
class EventContext:
    physical_onset_frame: int
    lookbehind_start_frame: int
    tail_end_frame: int
    attack_peak_rms: float
    low_band_rms: float
    level_rms: float
    low_band_share: float
    role: str = "ambiguous"
    accent_rank: int = -1

    @property
    def frame_count(self) -> int:
        return self.tail_end_frame - self.lookbehind_start_frame


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    try:
        dense.require_numpy()
        repo = Path(__file__).resolve().parent.parent
        validate_contract(repo, require_frozen=True)
        require(args.fixtures, "rejected v3 retains only --fixtures")
        run_source_blind_fixtures()
    except (OSError, ValueError) as error:
        print(f"Dense foundation event-context v3 failed: {error}", file=sys.stderr)
        return 1
    print("valid Dense foundation event-context v3 source-blind fixtures")
    return 0


def extract_event_contexts(
    samples: np.ndarray,
    sample_rate_hz: int,
    input_lsb: float,
    dc_means: np.ndarray,
) -> tuple[list[EventContext], dict[str, Any]]:
    protocol = stage_a.load_frozen_protocol(stage_a.CANONICAL_PROTOCOL_V2_PATH)
    require(samples.ndim == 2 and samples.shape[1] == 2, "source must be stereo")
    require(dc_means.shape == (2,), "authoritative stereo DC means are required")
    centered = samples.astype(np.float64) - dc_means[None, :]
    phase_safe_power = np.mean(centered * centered, axis=1, dtype=np.float64)
    windows = [
        stage_a._duration_frames(protocol, sample_rate_hz, float(ms))
        for ms in protocol.value("rms_envelope_windows_ms")
    ]
    r1, r8, r20 = (
        stage_a._causal_rms(phase_safe_power, window) for window in windows
    )
    detector = stage_a._detect_events(centered, sample_rate_hz, input_lsb, protocol)
    refinements = tuple(
        stage_a._resolve_anatomy(
            peak,
            phase_safe_power,
            r1,
            r8,
            r20,
            sample_rate_hz,
            input_lsb,
            protocol,
        )
        for peak in detector.pre_nms
    )
    by_coarse_frame = {item.coarse_peak_frame: item for item in refinements}
    primaries = [
        by_coarse_frame[peak.frame]
        for peak in detector.nms
        if peak.frame in by_coarse_frame
    ]
    composite = stage_a._composite_ownership(
        refinements, detector.nms, r1, sample_rate_hz, protocol
    )
    physical_onsets = sorted(
        {
            item.physical_onset_frame
            for item in primaries
            if item.physical_onset_frame is not None
        }
    )
    lookbehind_frames = stage_a._duration_frames(
        protocol, sample_rate_hz, LOOKBEHIND_MS
    )
    raw_contexts: list[EventContext] = []
    refusal_counts: dict[str, int] = {}

    def refuse(code: str) -> None:
        refusal_counts[code] = refusal_counts.get(code, 0) + 1

    for item in primaries:
        anatomy = item.anatomy
        if anatomy is None:
            refuse("incomplete_anatomy")
            continue
        fused_count, composite_refusal, _ = composite.get(
            anatomy.coarse_peak_frame, (1, None, ())
        )
        if composite_refusal is not None or fused_count != 1:
            refuse("fused_or_ambiguous_event")
            continue
        if any(
            anatomy.lookbehind_start_frame <= onset < anatomy.physical_onset_frame
            for onset in physical_onsets
        ):
            refuse("lookbehind_contains_onset")
            continue
        next_onset = next(
            (onset for onset in physical_onsets if onset > anatomy.physical_onset_frame),
            None,
        )
        if next_onset is not None and anatomy.tail_end_frame >= next_onset - lookbehind_frames:
            refuse("tail_overlaps_next_event")
            continue
        context = samples[anatomy.lookbehind_start_frame : anatomy.tail_end_frame]
        level = dense.rms(context)
        low = clarity.low_band_rms(context, sample_rate_hz)
        if level <= 1.0e-9 or low <= 1.0e-9:
            refuse("insufficient_context_signal")
            continue
        raw_contexts.append(
            EventContext(
                physical_onset_frame=anatomy.physical_onset_frame,
                lookbehind_start_frame=anatomy.lookbehind_start_frame,
                tail_end_frame=anatomy.tail_end_frame,
                attack_peak_rms=anatomy.attack_peak_rms,
                low_band_rms=low,
                level_rms=level,
                low_band_share=low / level,
            )
        )
    require(len(raw_contexts) >= 4, "fewer than four isolated complete event contexts")
    shares = sorted(context.low_band_share for context in raw_contexts)
    split_candidates = [
        (shares[index + 1] - shares[index], index)
        for index in range(1, len(shares) - 2)
    ]
    require(split_candidates, "too few contexts for two source-relative roles")
    role_gap, split_index = max(split_candidates, key=lambda item: (item[0], -item[1]))
    share_span = shares[-1] - shares[0]
    require(
        share_span > 0.0 and role_gap > 0.5 * share_span,
        "source-relative event roles are not cleanly bimodal",
    )
    upper_attack_max = shares[split_index]
    low_body_min = shares[split_index + 1]
    classified = []
    for context in raw_contexts:
        if context.low_band_share <= upper_attack_max:
            classified.append(replace(context, role="upper_attack"))
        elif context.low_band_share >= low_body_min:
            classified.append(replace(context, role="low_body"))
    ranked: list[EventContext] = []
    for role in TARGET_ROLES:
        role_contexts = sorted(
            (context for context in classified if context.role == role),
            key=lambda context: (context.attack_peak_rms, context.physical_onset_frame),
        )
        ranked.extend(
            replace(context, accent_rank=index)
            for index, context in enumerate(role_contexts)
        )
    ranked.sort(key=lambda context: context.physical_onset_frame)
    return ranked, {
        "detector_pre_nms_count": len(detector.pre_nms),
        "detector_nms_count": len(detector.nms),
        "complete_isolated_context_count": len(raw_contexts),
        "classified_context_count": len(ranked),
        "role_split": {
            "upper_attack_max_share": upper_attack_max,
            "low_body_min_share": low_body_min,
            "gap": role_gap,
            "total_span": share_span,
        },
        "refusal_counts": refusal_counts,
    }


def build_candidate(
    source: np.ndarray,
    sample_rate_hz: int,
    bpm: float,
    input_lsb: float,
    dc_means: np.ndarray,
) -> tuple[np.ndarray, dict[str, Any]]:
    beat_frames = round(sample_rate_hz * 60.0 / bpm)
    phrase_frames = beat_frames * 8
    require(source.shape == (phrase_frames, 2), "source phrase shape mismatch")
    contexts, detector_evidence = extract_event_contexts(
        source, sample_rate_hz, input_lsb, dc_means
    )
    answer_start = ANSWER_START_BEAT * beat_frames
    fade_frames = max(1, round(sample_rate_hz * CROSSFADE_MS / 1000.0))
    targets = []
    for role in TARGET_ROLES:
        choices = [
            context
            for context in contexts
            if context.role == role
            and context.lookbehind_start_frame >= answer_start
            and context.tail_end_frame <= phrase_frames
        ]
        if not choices:
            raise TechnicalRejection(
                "missing_target_role",
                f"no isolated {role} target event in final two beats",
                detector_evidence,
            )
        targets.append(
            max(
                choices,
                key=lambda context: (
                    context.attack_peak_rms,
                    -context.physical_onset_frame,
                ),
            )
        )
    targets.sort(key=lambda context: context.physical_onset_frame)
    used_donor_onsets: set[int] = set()
    mappings = []
    candidate = source.astype(np.float32, copy=True)
    all_onsets = {context.physical_onset_frame for context in contexts}
    for target in targets:
        eligible = []
        rejection_counts: dict[str, int] = {}

        def reject_donor(code: str) -> None:
            rejection_counts[code] = rejection_counts.get(code, 0) + 1

        for donor in contexts:
            if donor.role != target.role or donor.physical_onset_frame >= answer_start:
                continue
            if donor.physical_onset_frame in used_donor_onsets:
                reject_donor("donor_already_used")
                continue
            if abs(donor.accent_rank - target.accent_rank) > MAX_ACCENT_RANK_DISTANCE:
                reject_donor("accent_rank_distance")
                continue
            render_length = max(target.frame_count, donor.frame_count) + fade_frames
            target_end = target.lookbehind_start_frame + render_length
            donor_end = donor.lookbehind_start_frame + render_length
            if (
                target_end > phrase_frames
                or donor_end > source.shape[0]
                or donor_end > answer_start
            ):
                reject_donor("complete_context_or_exit_fade_unavailable")
                continue
            if any(
                donor.physical_onset_frame < onset < donor_end for onset in all_onsets
            ):
                reject_donor("donor_window_contains_another_event")
                continue
            if any(
                target.physical_onset_frame < onset < target_end for onset in all_onsets
            ):
                reject_donor("target_window_contains_another_event")
                continue
            target_audio = source[target.lookbehind_start_frame:target_end]
            donor_audio = source[donor.lookbehind_start_frame:donor_end]
            low_delta = clarity.db_delta(
                clarity.low_band_rms(donor_audio, sample_rate_hz),
                clarity.low_band_rms(target_audio, sample_rate_hz),
            )
            level_delta = clarity.db_delta(dense.rms(donor_audio), dense.rms(target_audio))
            attack_delta = clarity.db_delta(
                donor.attack_peak_rms, target.attack_peak_rms
            )
            if abs(low_delta) > CONTEXT_LOW_BAND_LIMIT_DB:
                reject_donor("context_low_band_delta")
                continue
            if abs(level_delta) > CONTEXT_LEVEL_LIMIT_DB:
                reject_donor("context_level_delta")
                continue
            if abs(attack_delta) > ATTACK_LEVEL_LIMIT_DB:
                reject_donor("attack_level_delta")
                continue
            eligible.append(
                (
                    (
                        abs(donor.accent_rank - target.accent_rank),
                        abs(low_delta),
                        abs(level_delta),
                        abs(attack_delta),
                        donor.physical_onset_frame,
                    ),
                    donor,
                    donor_audio,
                    render_length,
                    {
                        "context_low_band_delta_db": low_delta,
                        "context_level_delta_db": level_delta,
                        "attack_level_delta_db": attack_delta,
                    },
                )
            )
        if not eligible:
            raise TechnicalRejection(
                "no_eligible_role_matched_donor",
                f"no eligible unique donor for {target.role}",
                {**detector_evidence, "donor_rejection_counts": rejection_counts},
            )
        _, donor, donor_audio, render_length, metrics = min(
            eligible, key=lambda item: item[0]
        )
        clarity.replace(candidate, target.lookbehind_start_frame, donor_audio, fade_frames)
        used_donor_onsets.add(donor.physical_onset_frame)
        mappings.append(
            {
                "role": target.role,
                "target_onset_frame": target.physical_onset_frame,
                "target_context_start_frame": target.lookbehind_start_frame,
                "target_anatomy_tail_end_frame": target.tail_end_frame,
                "target_context_end_frame": target.lookbehind_start_frame
                + render_length,
                "target_accent_rank": target.accent_rank,
                "donor_onset_frame": donor.physical_onset_frame,
                "donor_context_start_frame": donor.lookbehind_start_frame,
                "donor_accent_rank": donor.accent_rank,
                "render_window_frame_count": render_length,
                **metrics,
            }
        )
    require(
        np.array_equal(candidate[:answer_start], source[:answer_start]),
        "six-beat anchor changed",
    )
    answer_source = source[answer_start:phrase_frames]
    answer_candidate = candidate[answer_start:phrase_frames]
    whole_low_delta = clarity.db_delta(
        clarity.low_band_rms(answer_candidate, sample_rate_hz),
        clarity.low_band_rms(answer_source, sample_rate_hz),
    )
    whole_level_delta = clarity.db_delta(
        dense.rms(answer_candidate), dense.rms(answer_source)
    )
    local_low = []
    local_level = []
    half_beat = beat_frames // 2
    for start in range(0, 2 * beat_frames, half_beat):
        source_half = answer_source[start : start + half_beat]
        candidate_half = answer_candidate[start : start + half_beat]
        local_low.append(
            clarity.db_delta(
                clarity.low_band_rms(candidate_half, sample_rate_hz),
                clarity.low_band_rms(source_half, sample_rate_hz),
            )
        )
        local_level.append(
            clarity.db_delta(dense.rms(candidate_half), dense.rms(source_half))
        )
    normalized_delta = dense.normalized_rms_delta(source, candidate)
    correlation = dense.waveform_correlation(source, candidate)
    gate_evidence = {
        "whole_answer_low_band_delta_db": whole_low_delta,
        "whole_answer_level_delta_db": whole_level_delta,
        "local_half_beat_low_band_delta_db": local_low,
        "local_half_beat_level_delta_db": local_level,
        "normalized_rms_delta": normalized_delta,
        "waveform_correlation": correlation,
    }
    gates_pass = (
        abs(whole_low_delta) <= ANSWER_LOW_BAND_LIMIT_DB
        and abs(whole_level_delta) <= ANSWER_LEVEL_LIMIT_DB
        and max(abs(value) for value in local_low) <= LOCAL_LOW_BAND_LIMIT_DB
        and max(abs(value) for value in local_level) <= LOCAL_LEVEL_LIMIT_DB
        and normalized_delta >= MIN_NORMALIZED_DELTA
        and abs(correlation) <= MAX_ABSOLUTE_CORRELATION
    )
    if not gates_pass:
        raise TechnicalRejection(
            "answer_clarity_or_contrast_gate_failed",
            "mapped answer failed frozen phrase gates",
            {**detector_evidence, **gate_evidence, "mappings": mappings},
        )
    return candidate, {
        "detector_and_anatomy": detector_evidence,
        "mappings": mappings,
        "answer_gates": gate_evidence,
    }


def run_source_blind_fixtures() -> None:
    sample_rate = 44_100
    bpm = 130.0
    source = synthetic_phrase(sample_rate, bpm, identical_roles=False)
    candidate, evidence = build_candidate(
        source,
        sample_rate,
        bpm,
        math.ldexp(1.0, -23),
        np.mean(source, axis=0, dtype=np.float64),
    )
    beat_frames = round(sample_rate * 60.0 / bpm)
    require(
        np.array_equal(candidate[: beat_frames * 6], source[: beat_frames * 6]),
        "fixture anchor changed",
    )
    mappings = evidence["mappings"]
    require(
        [mapping["role"] for mapping in mappings] == list(TARGET_ROLES),
        "fixture did not map one event per role",
    )
    require(
        len({mapping["donor_onset_frame"] for mapping in mappings}) == 2,
        "fixture reused one canonical donor",
    )
    lookbehind = round(sample_rate * LOOKBEHIND_MS / 1000.0)
    require(
        all(
            mapping["target_onset_frame"] - mapping["target_context_start_frame"]
            == lookbehind
            for mapping in mappings
        ),
        "fixture target onset alignment changed",
    )
    expect_rejection(
        synthetic_phrase(sample_rate, bpm, identical_roles=True),
        sample_rate,
        bpm,
        "near-identical role variants must fail closed",
    )


def synthetic_phrase(
    sample_rate_hz: int, bpm: float, *, identical_roles: bool
) -> np.ndarray:
    beat_frames = round(sample_rate_hz * 60.0 / bpm)
    frame_count = beat_frames * 8
    result = np.zeros((frame_count, 2), dtype=np.float64)
    for beat in range(8):
        role_low = beat % 2 == 0
        onset = beat * beat_frames + round(0.060 * sample_rate_hz)
        length = min(round(0.110 * sample_rate_hz), frame_count - onset)
        time = np.arange(length, dtype=np.float64) / sample_rate_hz
        envelope = np.exp(-time / (0.024 if role_low else 0.020))
        if identical_roles:
            frequency = 72.0 if role_low else 1_050.0
            amplitude = 0.58 if role_low else 0.54
        else:
            family_index = beat // 2
            frequency = (
                62.0 + family_index * 5.0
                if role_low
                else 840.0 + family_index * 115.0
            )
            amplitude = (0.54 + family_index * 0.018) if role_low else (0.50 + family_index * 0.017)
        hit = amplitude * envelope * np.sin(2.0 * math.pi * frequency * time)
        result[onset : onset + length, 0] += hit
        result[onset : onset + length, 1] += hit * 0.97
    return result.astype(np.float32)


def expect_rejection(
    source: np.ndarray, sample_rate_hz: int, bpm: float, message: str
) -> None:
    try:
        build_candidate(
            source,
            sample_rate_hz,
            bpm,
            math.ldexp(1.0, -23),
            np.mean(source, axis=0, dtype=np.float64),
        )
    except (TechnicalRejection, ValueError):
        return
    raise ValueError(message)


def validate_contract(repo: Path, *, require_frozen: bool) -> dict[str, Any]:
    path = repo / CONTRACT
    contract = json.loads(path.read_text())
    require(
        contract.get("schema") == "riotbox.dense_break_foundation_chop.v3",
        "v3 contract schema changed",
    )
    mechanism = contract["mechanism"]
    require(
        mechanism["input"]["bars"] == 2
        and mechanism["input"]["beats_per_bar"] == 4
        and mechanism["input"]["requires_confirmed_beat_grid"] is True
        and mechanism["input"]["playback_rate"] == 1.0,
        "v3 input contract changed",
    )
    require(
        mechanism["anchor"]["target"] == "beats_0_through_5"
        and mechanism["anchor"]["operation"] == "sample_exact_passthrough",
        "v3 anchor changed",
    )
    require(
        mechanism["detector_and_anatomy"]["raw_sha256"]
        == "b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878",
        "v3 detector/anatomy pin changed",
    )
    require(
        mechanism["target_selection"]["required_roles"] == list(TARGET_ROLES)
        and mechanism["target_selection"]["required_target_count"] == 2,
        "v3 target roles changed",
    )
    donor = mechanism["donor_selection"]
    require(
        donor["accent_rank_distance_max"] == MAX_ACCENT_RANK_DISTANCE,
        "v3 accent-rank gate changed",
    )
    require(
        donor["eligibility"]
        == {
            "absolute_context_low_band_delta_db_max": CONTEXT_LOW_BAND_LIMIT_DB,
            "absolute_context_level_delta_db_max": CONTEXT_LEVEL_LIMIT_DB,
            "absolute_attack_peak_rms_delta_db_max": ATTACK_LEVEL_LIMIT_DB,
        },
        "v3 donor eligibility changed",
    )
    require(
        mechanism["render"]["crossfade_shape"] == "linear"
        and mechanism["render"]["crossfade_duration_ms"] == CROSSFADE_MS
        and mechanism["render"]["donor_reuse_count_max"] == 1,
        "v3 render contract changed",
    )
    require(
        mechanism["answer_gates"]
        == {
            "whole_answer_absolute_low_band_delta_db_max": ANSWER_LOW_BAND_LIMIT_DB,
            "whole_answer_absolute_level_delta_db_max": ANSWER_LEVEL_LIMIT_DB,
            "each_corresponding_half_beat_absolute_low_band_delta_db_max": LOCAL_LOW_BAND_LIMIT_DB,
            "each_corresponding_half_beat_absolute_level_delta_db_max": LOCAL_LEVEL_LIMIT_DB,
            "whole_phrase_normalized_rms_delta_min": MIN_NORMALIZED_DELTA,
            "whole_phrase_absolute_waveform_correlation_max": MAX_ABSOLUTE_CORRELATION,
        },
        "v3 answer gates changed",
    )
    development = contract["development_exploration"]
    require(
        development["variant_limit"] == 1
        and development["source"]["case_id"] == CASE_ID,
        "v3 Development source scope changed",
    )
    require(
        development["active_holdout_collision_registry"]["path"]
        == REGISTRY.as_posix()
        and development["active_holdout_collision_registry"]["raw_sha256"]
        == "af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812",
        "v3 collision-registry pin changed",
    )
    require(
        contract["attempt_limit"]["v4_same_role_or_selector_allowed_after_reject"]
        is False,
        "v3 final-attempt boundary changed",
    )
    if require_frozen:
        require(CONTRACT_SHA256 != "UNFROZEN", "v3 is not frozen")
        require(sha256_file(path) == CONTRACT_SHA256, "frozen v3 contract changed")
        require(contract["status"] == "frozen", "v3 status is not frozen")
    return contract


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    raise SystemExit(main())
