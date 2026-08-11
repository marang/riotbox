#!/usr/bin/env python3
"""Run the versioned RIOTBOX-1430 Matrix-v6 advanced screen correction.

V2 preserves the already-qualified raw source/candidate event identity while
evaluating signal-domain metrics in raw and attenuation-matched views. It does
not change any threshold, renderer result, or confound basis.
"""

from __future__ import annotations

import argparse
import hashlib
import math
from pathlib import Path
from typing import Any

import numpy as np

import validate_percussive_force_stage_a_matrix_v6 as v1


OUTPUT_NAME = "advanced-mechanical-result-v2.json"
PRIOR_RESULT = Path(
    "artifacts/audio_qa/riotbox-1430/stage-a-v6-matrix/advanced-mechanical-result.json"
)
PRIOR_RESULT_SHA = "d78461412f98e145e9767a111c3c5654cdaf8f0b54f84e31d2e56dcf7c9cd406"


def screen_view_v2(
    label: str,
    source: np.ndarray,
    candidate: np.ndarray,
    source_input: Any,
    source_analysis: Any,
    source_event: Any,
    policy: dict[str, Any],
    source_gain: float,
    candidate_gain: float,
    protocol: Any,
) -> dict[str, Any]:
    """Apply signal screens without making event identity gain-dependent."""

    scaled_source = source * source_gain
    scaled_candidate = candidate * candidate_gain

    # Event identity was classified on raw signals. Uniform view gain must not
    # create, remove, or move it; only the comparisons below use matched gains.
    candidate_event_analysis = v1.analyze(source_input, candidate, protocol, 1.0)
    corresponding, event_metrics = v1.corresponding_event(
        source_event,
        candidate_event_analysis,
        source_input.sample_rate_hz,
        protocol,
    )
    event_count_pass = len(candidate_event_analysis.event_level_onset_frames) <= len(
        source_analysis.event_level_onset_frames
    )

    onset = source_event.physical_onset_frame
    attack_end = source_event.attack_end_frame
    body_end = source_event.body_end_frame
    x = scaled_source[onset:body_end]
    y = scaled_candidate[onset:body_end]
    source_energy = float(np.sum(x**2))
    delta = math.sqrt(float(np.sum((y - x) ** 2)) / source_energy)
    x_centered = x - np.mean(x, axis=0)
    y_centered = y - np.mean(y, axis=0)
    correlation = float(
        np.sum(x_centered * y_centered)
        / math.sqrt(float(np.sum(x_centered**2)) * float(np.sum(y_centered**2)))
    )
    cosine = v1.attack_spectral_cosine(
        scaled_source,
        scaled_candidate,
        onset,
        attack_end,
        source_input.sample_rate_hz,
        int(protocol.value("attack_spectral_band_count")),
    )
    body_ratio = float(np.sum(scaled_candidate[attack_end:body_end] ** 2)) / float(
        np.sum(scaled_source[attack_end:body_end] ** 2)
    )

    failures = []
    if corresponding is None or not event_count_pass:
        failures.append("event_integrity")
    if delta < float(protocol.value("near_identity_delta_rms_min")):
        failures.append("near_identity")
    if correlation < float(protocol.value("identity_correlation_min")) or cosine < float(
        protocol.value("attack_spectral_cosine_min")
    ):
        failures.append("identity")
    body_range = protocol.value("body_energy_ratio_range")
    if not float(body_range[0]) <= body_ratio <= float(body_range[1]):
        failures.append("body_energy")
    try:
        fits = v1.fit_metrics(
            scaled_source,
            scaled_candidate,
            source_event,
            source_input.sample_rate_hz,
            protocol,
        )
    except Exception as error:
        fits = {"error": str(error)}
        failures.append("confound_screen_undefined")
    else:
        if fits["gain_fit_residual"] < float(protocol.value("gain_only_fit_residual_min")):
            failures.append("gain_only_confound")
        if fits["static_eq_fit_residual"] < float(protocol.value("static_eq_fit_residual_min")):
            failures.append("static_eq_confound")
        if fits["static_distortion_fit_residual"] < float(
            protocol.value("static_distortion_fit_residual_min")
        ):
            failures.append("static_distortion_confound")
    boundaries = v1.boundary_pass(
        scaled_source,
        scaled_candidate,
        source_event,
        policy,
        source_input.sample_rate_hz,
        protocol,
    )
    if not boundaries:
        failures.append("boundary_discontinuity")

    return {
        "view": label,
        "source_gain": source_gain,
        "candidate_gain": candidate_gain,
        "event_identity_basis": "source_and_candidate_raw_frozen_before_view_gain",
        "candidate_event_level_count": len(candidate_event_analysis.event_level_onset_frames),
        "source_event_level_count": len(source_analysis.event_level_onset_frames),
        "event_integrity": event_metrics,
        "near_identity_delta": delta,
        "zero_lag_identity_correlation": correlation,
        "attack_spectral_cosine": cosine,
        "body_energy_ratio": body_ratio,
        "fit_metrics": fits,
        "boundary_discontinuity_pass": boundaries,
        "failures": sorted(set(failures)),
        "passed": not failures,
    }


def run(matrix_result_path: Path) -> dict[str, Any]:
    prior_raw = PRIOR_RESULT.read_bytes()
    v1.require(
        hashlib.sha256(prior_raw).hexdigest() == PRIOR_RESULT_SHA,
        "advanced-screen-v1 evidence changed",
    )
    original = v1.screen_view
    try:
        v1.screen_view = screen_view_v2
        result = v1.run(matrix_result_path)
    finally:
        v1.screen_view = original
    result["schema"] = "riotbox.percussive_force_development_matrix_advanced_result.v2"
    result["algorithm_version"] = "source_frozen_event_identity_v2"
    result["supersedes_result_path"] = PRIOR_RESULT.as_posix()
    result["supersedes_result_sha256"] = PRIOR_RESULT_SHA
    result["v1_rejection_preserved"] = True
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--matrix-result",
        type=Path,
        default=Path("artifacts/audio_qa/riotbox-1430/stage-a-v6-matrix") / v1.RESULT_NAME,
    )
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    try:
        v1.shared.read_pinned_json(v1.MATRIX, v1.MATRIX_SHA)
        v1.shared.read_pinned_json(v1.SOURCE_SET, v1.SOURCE_SET_SHA)
        protocol = v1.stage_a.load_frozen_protocol(v1.PROTOCOL)
        v1.require(protocol.sha256 == v1.PROTOCOL_SHA, "Protocol-v2 pin changed")
        v1.require(v1.screen_view is not screen_view_v2, "v1 screen was mutated before run")
        if args.validate_only:
            print("PASS: Matrix-v6 advanced-screen-v2 contracts; candidate_audio_accessed=false")
            return 0
        output = run(args.matrix_result)
        output_path = args.matrix_result.parent / OUTPUT_NAME
        v1.shared.create_exclusive_json(output_path, output)
    except Exception as error:
        print(f"FAIL: Matrix-v6 advanced screens v2 stopped fail-closed: {error}")
        return 1
    print("PASS: Matrix-v6 advanced screens v2 complete")
    print(f"advanced_survivor_count={output['advanced_survivor_count']}")
    print(f"result={output_path}")
    print("human_verdict=unverified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
