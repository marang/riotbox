"""Descriptive RIOTBOX-1501 PCM measurements; never a musical verdict."""

from __future__ import annotations

import hashlib
from enum import Enum
from typing import Any

import numpy as np

from source_holdout_development_access import require


SAMPLE_RATE = 48_000
CHANNELS = 2
POLICIES = (("A", 0.92, 0.985), ("B", 0.9525, 0.985), ("C", 0.92, 0.9525))
LOCAL_WINDOWS = (("attack", 0, 960), ("body", 960, 5760), ("recovery", 5760, 12000))


class Version(Enum):
    """The two closed preregistered experiments, never caller-selected gains."""

    V1 = "v1"
    V2 = "v2"

    @property
    def conditions(self) -> tuple[tuple[str, float], ...]:
        controls = (("clean", 1.0), ("stress_2x", 2.0))
        return controls if self is Version.V1 else (*controls, ("stress_4x", 4.0))


def pcm(values: Any, sample_count: int) -> np.ndarray:
    signal = np.asarray(values, dtype=np.float32)
    require(signal.ndim == 1 and signal.size == sample_count, "PCM alignment mismatch")
    require(sample_count > 0 and sample_count % CHANNELS == 0, "invalid frame count")
    require(bool(np.isfinite(signal).all()), "nonfinite PCM")
    return signal


def pcm_identity(signal: np.ndarray) -> str:
    return hashlib.sha256(signal.astype("<f4", copy=False).tobytes()).hexdigest()


def rms(signal: np.ndarray) -> float:
    return float(np.sqrt(np.mean(np.square(signal.astype(np.float64)))))


def level(signal: np.ndarray) -> dict[str, Any]:
    peak = float(np.max(np.abs(signal)))
    energy = rms(signal)
    return {
        "peak_abs": peak,
        "rms_f64": energy,
        "dc_f64": float(np.mean(signal.astype(np.float64))),
        "clip_count": int(np.count_nonzero(np.abs(signal) >= np.float32(1.0))),
        "near_clip_count": int(np.count_nonzero(np.abs(signal) >= np.float32(0.98))),
        "headroom_to_full_scale": 1.0 - peak,
        "crest_factor": peak / energy if energy != 0.0 else None,
    }


def spectral_fractions(signal: np.ndarray) -> list[float] | None:
    """Whole-window symmetric Hann, undoubled one-sided power, no epsilon floor."""
    power = np.abs(np.fft.rfft(signal.astype(np.float64) * np.hanning(signal.size))) ** 2
    total = float(np.sum(power))
    if total == 0.0:
        return None
    frequencies = np.fft.rfftfreq(signal.size, 1.0 / SAMPLE_RATE)
    return [float(np.sum(power[mask]) / total) for mask in (
        frequencies < 180.0,
        (frequencies >= 180.0) & (frequencies < 2500.0),
        frequencies >= 2500.0,
    )]


def delta(control: np.ndarray, candidate: np.ndarray) -> dict[str, Any]:
    difference = candidate.astype(np.float64) - control.astype(np.float64)
    delta_rms = rms(difference)
    control_rms = rms(control)
    x = control.astype(np.float64) - np.mean(control.astype(np.float64))
    y = candidate.astype(np.float64) - np.mean(candidate.astype(np.float64))
    denominator = float(np.linalg.norm(x) * np.linalg.norm(y))
    return {
        "bit_identical": bool(np.array_equal(control.view(np.uint32), candidate.view(np.uint32))),
        "signed_delta_min": float(np.min(difference)),
        "signed_delta_max": float(np.max(difference)),
        "absolute_delta_peak": float(np.max(np.abs(difference))),
        "delta_rms_f64": delta_rms,
        "relative_delta_rms": delta_rms / control_rms if control_rms != 0.0 else None,
        "pearson_correlation": float(np.dot(x, y) / denominator) if denominator != 0.0 else None,
    }


def measure_case(raw: dict[str, Any], frame_count: int, *,
                 version: Version = Version.V1,
                 historical_controls: dict[str, str] | None = None,
                 minimum_mix_rms: float | None = None,
                 listening_frame_window: tuple[int, int] | None = None) -> dict[str, Any]:
    """Validate unchanged controls, then describe only the version's fixed conditions."""
    require(isinstance(version, Version), "unknown calibration version")
    if version is Version.V2:
        require(raw.get("protocol_version") == "v2", "response protocol version mismatch")
        require(isinstance(historical_controls, dict)
                and set(historical_controls) == {"clean_pcm_sha256_f32le", "stress_2x_pcm_sha256_f32le"},
                "missing historical control identities")
    require(raw["controls"] == {"repeat_128_bit_exact": True, "partition_257_bit_exact": True,
                                "baseline_api_bit_exact": True}, "baseline parity failed")
    require(raw["sample_rate_hz"] == SAMPLE_RATE and raw["channels"] == CHANNELS,
            "render format mismatch")
    require(frame_count >= LOCAL_WINDOWS[-1][2], "local windows exceed render")
    if listening_frame_window is not None:
        start, end = listening_frame_window
        require(0 <= start < end <= frame_count, "preselected window exceeds render")
    pre = pcm(raw["pre_samples"], frame_count * CHANNELS)
    require(raw["frame_count"] == frame_count, "render duration mismatch")
    require([row["condition"] for row in raw["conditions"]]
            == [name for name, _ in version.conditions],
            "condition order/budget mismatch")
    report: dict[str, Any] = {"controls": raw["controls"], "frame_count": frame_count,
                              "sample_rate_hz": SAMPLE_RATE, "channels": CHANNELS,
                              "conditions": [], "human_verdict": "unverified"}
    for condition, (condition_name, factor) in zip(raw["conditions"], version.conditions, strict=True):
        # Every condition derives directly from the original f32 buffer, never another output.
        shared = pre * np.float32(factor)
        require(bool(np.isfinite(shared).all()), "nonfinite condition input")
        require([row["policy"] for row in condition["outputs"]] == ["A", "B", "C"],
                "policy order/budget mismatch")
        outputs = [pcm(row["samples"], pre.size) for row in condition["outputs"]]
        measured = {"condition": condition["condition"], "input_sha256_f32le": pcm_identity(shared),
                    "input": level(shared), "outputs": []}
        historical_hash = None
        if version is Version.V2 and condition_name in {"clean", "stress_2x"}:
            historical_hash = historical_controls[f"{condition_name}_pcm_sha256_f32le"]
            require(measured["input_sha256_f32le"] == historical_hash,
                    f"{condition_name}: historical input identity mismatch")
        for row, output, (name, threshold, ceiling) in zip(
                condition["outputs"], outputs, POLICIES, strict=True):
            output_hash = pcm_identity(output)
            if historical_hash is not None:
                require(output_hash == historical_hash,
                        f"{condition_name}/{name}: historical output identity mismatch")
            limiter = row["limiter"]
            require(limiter["threshold_bits"] == int(np.float32(threshold).view(np.uint32))
                    and limiter["ceiling_bits"] == int(np.float32(ceiling).view(np.uint32)),
                    "policy f32 identity mismatch")
            writes = int(np.count_nonzero(output.view(np.uint32) != shared.view(np.uint32)))
            require(writes == limiter["limited_sample_count"], "actual write-count mismatch")
            require(level(output)["clip_count"] == 0, "post-protection clips")
            require(float(np.max(np.abs(output))) <= float(np.float32(ceiling)),
                    "output exceeds policy ceiling")
            if factor == 1.0:
                require(writes == 0 and level(shared)["clip_count"] == 0,
                        "clean-product baseline/candidate regression")
                require(limiter["post"]["active_samples"] > 0, "inactive clean-product output")
                if minimum_mix_rms is not None:
                    require(limiter["post"]["rms"] >= float(np.float32(minimum_mix_rms)),
                            "clean-product mix below inherited RMS minimum")
            local = []
            for window, start, end in LOCAL_WINDOWS:
                for channel in range(CHANNELS):
                    source = shared.reshape(-1, CHANNELS)[start:end, channel]
                    control = outputs[0].reshape(-1, CHANNELS)[start:end, channel]
                    candidate = output.reshape(-1, CHANNELS)[start:end, channel]
                    local.append({"window": window, "frames": [start, end], "channel": channel,
                                  "input": level(source), "output": level(candidate),
                                  "modified_samples": int(np.count_nonzero(source != candidate)),
                                  "peak_change_vs_a": level(candidate)["peak_abs"] - level(control)["peak_abs"],
                                  "delta_vs_a": delta(control, candidate),
                                  "spectral_power_fractions": spectral_fractions(candidate)})
            output_report = {"policy": name, "limiter": limiter,
                             "output_sha256_f32le": output_hash,
                             "output": level(output), "delta_vs_a": delta(outputs[0], output),
                             "local_windows": local}
            if listening_frame_window is not None:
                start, end = listening_frame_window
                selected = slice(start * CHANNELS, end * CHANNELS)
                output_report["preselected_window"] = {
                    "frames": [start, end],
                    "modified_samples": int(np.count_nonzero(
                        output[selected].view(np.uint32) != shared[selected].view(np.uint32))),
                    "bit_identical_to_a": bool(np.array_equal(
                        output[selected].view(np.uint32), outputs[0][selected].view(np.uint32))),
                }
            measured["outputs"].append(output_report)
        report["conditions"].append(measured)
    if version is Version.V2:
        report["protocol_version"] = version.value
        report["historical_controls"] = dict(historical_controls)
    return report
