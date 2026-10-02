"""Exactly three fixed-window diagnostic WAVs; never source access or playback."""

from __future__ import annotations

import hashlib
import json
import math
import os
from pathlib import Path
import re
import selectors
import subprocess
import tempfile
import time
from typing import Any

import numpy as np

import generate_dense_break_performance_pack as dense
import master_bus_limiter_calibration_metrics as metrics
from source_holdout_development_access import require


POLICIES = ("A", "B", "C")
FRAMES = 96_000
FULL_FRAMES = 192_000
SAMPLE_RATE = 48_000
CHANNELS = 2
TOOL_TIMEOUT_SECONDS = 30.0
MAX_WAV_BYTES = 1024 * 1024
FORMAT = {
    "codec": "pcm_f32le", "sample_rate_hz": SAMPLE_RATE, "channels": CHANNELS,
    "frame_count": FRAMES, "duration_seconds": 2.0,
}
GAIN_RULE = ("min(1.0, 10**(-1.2/20) / max(estimated_raw_crop_peak_A, estimated_raw_crop_peak_B, "
             "estimated_raw_crop_peak_C)); exactly one shared attenuation-only gain applied once in f32 to all three crops")


def _validate_contract(contract: dict[str, Any]) -> None:
    require(contract["schema"] == "riotbox.master_bus_limiter_review_artifact.v1"
            and contract["case_id"] == "sparse_kicksnr_120", "closed artifact contract required")
    require(contract["condition"] == "stress_4x" and contract["frame_window"] == [0, FRAMES],
            "fixed artifact condition/window mismatch")
    require(contract["artifacts"] == [{"policy": name, "filename": f"{name}.wav"} for name in POLICIES],
            "fixed A/B/C artifact mapping mismatch")
    require(contract["format"] == FORMAT, "fixed artifact format mismatch")
    presentation = contract["presentation"]
    require(presentation["estimator"] == "conservative_four_x_bandlimited_fft_v1"
            and presentation["oversample_factor"] == 4
            and presentation["target_true_peak_dbtp"] == -1.2
            and presentation["maximum_true_peak_dbtp"] == -1.0
            and presentation["gain_rule"] == GAIN_RULE, "fixed presentation rule mismatch")
    require(presentation["forbidden"] == ["per_policy_normalization", "makeup_gain", "fades",
            "resampling", "additional_limiting", "dither", "PCM16_conversion", "concatenated_fourth_WAV"],
            "presentation exclusions changed")
    require(contract["analysis"]["local_frame_windows"] == [list(row) for row in metrics.LOCAL_WINDOWS],
            "fixed analysis windows changed")
    require(contract["review"]["playback_authorized"] is False, "this publisher never authorizes playback")


class ArtifactPublicationError(ValueError):
    """Available exact-file evidence survives first failure; no retry or cleanup."""

    def __init__(self, message: str, evidence: dict[str, Any]):
        super().__init__(message)
        self.evidence = evidence


def _tool(args: list[str], data: bytes = b"", *, stdout_limit: int = 4 * 1024 * 1024,
          stderr_limit: int = 256 * 1024) -> tuple[bytes, bytes]:
    """Linux pipe transport with live byte caps and one wall-clock deadline."""
    process = subprocess.Popen(args, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, bufsize=0)
    buffers = {"stdout": bytearray(), "stderr": bytearray()}
    limits = {"stdout": stdout_limit, "stderr": stderr_limit}
    deadline = time.monotonic() + TOOL_TIMEOUT_SECONDS
    offset = 0
    try:
        with selectors.DefaultSelector() as selector:
            for name in ("stdout", "stderr"):
                stream = getattr(process, name)
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_READ, name)
            if data:
                os.set_blocking(process.stdin.fileno(), False)
                selector.register(process.stdin, selectors.EVENT_WRITE, "stdin")
            else:
                process.stdin.close()
            while selector.get_map():
                remaining = deadline - time.monotonic()
                require(remaining > 0, f"{args[0]} exceeded tool timeout")
                for key, _ in selector.select(remaining):
                    if key.data == "stdin":
                        try:
                            offset += os.write(key.fd, data[offset:offset + 65_536])
                        except BrokenPipeError:
                            offset = len(data)
                        if offset == len(data):
                            selector.unregister(key.fileobj)
                            key.fileobj.close()
                    else:
                        chunk = os.read(key.fd, 65_536)
                        if not chunk:
                            selector.unregister(key.fileobj)
                            key.fileobj.close()
                        else:
                            buffers[key.data].extend(chunk)
                            require(len(buffers[key.data]) <= limits[key.data],
                                    f"{args[0]} exceeded {key.data} byte limit")
            remaining = deadline - time.monotonic()
            require(remaining > 0, f"{args[0]} exceeded tool timeout")
            process.wait(timeout=remaining)
        require(process.returncode == 0,
                f"{args[0]} failed ({process.returncode}): "
                + buffers["stderr"].decode("utf-8", errors="replace")[-2048:])
        return bytes(buffers["stdout"]), bytes(buffers["stderr"])
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        for stream in (process.stdin, process.stdout, process.stderr):
            stream.close()


def _db(amplitude: float) -> float:
    require(math.isfinite(amplitude) and amplitude > 0, "invalid/silent true-peak estimate")
    return 20.0 * math.log10(amplitude)


def _crop_outputs(raw: dict[str, Any]) -> tuple[list[np.ndarray], list[str]]:
    require(raw["case_id"] == "sparse_kicksnr_120" and raw["protocol_version"] == "v2",
            "only the Sparse V2 case is authorized")
    require(raw["sample_rate_hz"] == SAMPLE_RATE and raw["channels"] == CHANNELS
            and raw["frame_count"] == FULL_FRAMES, "fixed full-render format mismatch")
    selected = [row for row in raw["conditions"] if row["condition"] == "stress_4x"]
    require(len(selected) == 1, "one stress_4x condition is required")
    require([row["policy"] for row in selected[0]["outputs"]] == list(POLICIES),
            "closed A/B/C policy order required")
    crops, full_hashes = [], []
    for row in selected[0]["outputs"]:
        signal = metrics.pcm(row["samples"], FULL_FRAMES * CHANNELS)
        full_hashes.append(metrics.pcm_identity(signal))
        crops.append(signal.reshape(FULL_FRAMES, CHANNELS)[:FRAMES].copy())
    return crops, full_hashes


def _write_wav(path: Path, samples: np.ndarray) -> None:
    _tool(["ffmpeg", "-hide_banner", "-loglevel", "error", "-nostdin", "-n",
           "-f", "f32le", "-ar", str(SAMPLE_RATE), "-ac", str(CHANNELS),
           "-i", "pipe:0", "-map", "0:a:0", "-c:a", "pcm_f32le", "-f", "wav", str(path)],
          samples.astype("<f4", copy=False).tobytes(), stdout_limit=1024)


def _analyze_wav(path: Path, expected: np.ndarray, report: dict[str, Any]) -> np.ndarray:
    require(path.is_file() and not path.is_symlink(), "written artifact must be an ordinary file")
    with path.open("rb") as stream:
        payload = stream.read(MAX_WAV_BYTES + 1)
    require(len(payload) <= MAX_WAV_BYTES, "written WAV exceeds byte budget")
    report.update(sha256=hashlib.sha256(payload).hexdigest(), byte_count=len(payload))
    # All tools receive these exact hashed bytes, never a second path-based read.
    probe, _ = _tool([
        "ffprobe", "-v", "error", "-f", "wav", "-i", "pipe:0", "-show_entries",
        "stream=codec_name,codec_type,sample_rate,channels,bits_per_sample,duration_ts,time_base,duration:format=format_name,duration",
        "-of", "json"], payload)
    probe_data = json.loads(probe)
    report["ffprobe"] = probe_data
    streams = probe_data["streams"]
    require(len(streams) == 1, "WAV must contain exactly one audio stream")
    audio = streams[0]
    require(audio["codec_name"] == "pcm_f32le" and audio["codec_type"] == "audio"
            and int(audio["sample_rate"]) == SAMPLE_RATE and audio["channels"] == CHANNELS
            and audio["bits_per_sample"] == 32, "written WAV format mismatch")
    require(int(audio["duration_ts"]) == FRAMES and audio["time_base"] == "1/48000"
            and float(audio["duration"]) == 2.0, "ffprobe duration/frame identity mismatch")
    decoded, _ = _tool(["ffmpeg", "-hide_banner", "-loglevel", "error", "-nostdin",
                        "-f", "wav", "-i", "pipe:0", "-map", "0:a:0",
                        "-c:a", "pcm_f32le", "-f", "f32le", "pipe:1"],
                       payload, stdout_limit=FRAMES * CHANNELS * 4)
    require(len(decoded) == FRAMES * CHANNELS * 4, "decoded frame count mismatch")
    signal = metrics.pcm(np.frombuffer(decoded, dtype="<f4"), FRAMES * CHANNELS)
    report["presented_pcm_sha256_f32le"] = metrics.pcm_identity(signal)
    require(decoded == expected.astype("<f4", copy=False).tobytes(),
            "written PCM is not bit-identical to the shared-gain crop")
    report["decode_bit_exact"] = True
    report["level"] = metrics.level(signal)
    report["silence"] = {"all_zero": bool(np.all(signal == 0)),
                         "zero_sample_count": int(np.count_nonzero(signal == 0)),
                         "zero_sample_fraction": float(np.count_nonzero(signal == 0) / signal.size),
                         "sample_count": int(signal.size)}
    require(report["level"]["clip_count"] == 0 and report["level"]["rms_f64"] > 0,
            "written WAV is clipped or silent")
    true_peak = dense.conservative_true_peak_amplitude(signal.reshape(FRAMES, CHANNELS))
    report["conservative_true_peak_dbtp"] = _db(true_peak)
    require(report["conservative_true_peak_dbtp"] <= -1.0, "written WAV fails true-peak ceiling")
    _, loudness = _tool(["ffmpeg", "-hide_banner", "-nostdin", "-loglevel", "info",
                         "-f", "wav", "-i", "pipe:0", "-af", "ebur128=peak=true",
                         "-f", "null", "-"], payload, stdout_limit=1024)
    text = loudness.decode("utf-8", errors="strict")
    summary = text.rsplit("Summary:", 1)
    require(len(summary) == 2, "missing ebur128 summary")
    report["ebur128_summary"] = summary[1].strip()
    integrated = re.search(r"\bI:\s*([-+\w.]+)\s+LUFS", summary[1])
    peak = re.search(r"\bPeak:\s*([-+\w.]+)\s+dBFS", summary[1])
    require(integrated is not None and peak is not None, "incomplete ebur128 summary")
    report["ebur128_summary"] = summary[1][:peak.end()].strip()
    lufs, diagnostic_peak = float(integrated.group(1)), float(peak.group(1))
    # Keep the original summary but never put NaN/Infinity in a failure envelope.
    report["integrated_lufs"] = lufs if math.isfinite(lufs) else None
    report["ffmpeg_true_peak_dbfs"] = diagnostic_peak if math.isfinite(diagnostic_peak) else None
    require(math.isfinite(lufs) and math.isfinite(diagnostic_peak), "nonfinite ebur128 result")
    require(diagnostic_peak <= -1.0, "ffmpeg true-peak safety failed")
    report["status"] = "technical_preflight_pass"
    return signal.reshape(FRAMES, CHANNELS)


def _comparisons(control: np.ndarray, candidate: np.ndarray) -> list[dict[str, Any]]:
    measured = []
    for name, start, end in (*metrics.LOCAL_WINDOWS, ("full_2s", 0, FRAMES)):
        for channel in range(CHANNELS):
            left = control[start:end, channel]
            right = candidate[start:end, channel]
            control_spectrum = metrics.spectral_fractions(left)
            spectrum = metrics.spectral_fractions(right)
            measured.append({
                "window": name, "frames": [start, end], "channel": channel,
                "level": metrics.level(right), "delta_vs_a": metrics.delta(left, right),
                "spectral_power_fractions": spectrum,
                "spectral_power_fraction_delta_vs_a": (
                    [value - reference for value, reference in zip(spectrum, control_spectrum, strict=True)]
                    if spectrum is not None and control_spectrum is not None else None),
            })
    return measured


def preflight_tools(contract: dict[str, Any]) -> dict[str, Any]:
    """Generated-only proof before source access; no review artifact is created."""
    _validate_contract(contract)
    dense.require_numpy()
    tools = {}
    for name in ("ffmpeg", "ffprobe"):
        version, _ = _tool([name, "-version"])
        tools[name] = version.decode("utf-8").splitlines()[0]
    clock = np.arange(FRAMES, dtype=np.float64) / SAMPLE_RATE
    signal = np.column_stack((0.1 * np.sin(2 * np.pi * 440 * clock),
                              0.1 * np.cos(2 * np.pi * 660 * clock))).astype(np.float32)
    with tempfile.TemporaryDirectory(prefix="riotbox-limiter-generated-tool-proof-") as temp:
        path = Path(temp) / "generated.wav"
        _write_wav(path, signal)
        report: dict[str, Any] = {}
        _analyze_wav(path, signal, report)
    return {"tools": tools, "generated_roundtrip": report,
            "source_accesses": 0, "playbacks": 0}


def publish_artifacts(raw: dict[str, Any], output: Path, contract: dict[str, Any], *,
                      tool_preflight: dict[str, Any]) -> dict[str, Any]:
    """Caller must verify full historical identities before calling; never retries."""
    evidence: dict[str, Any] = {
        "status": "preflight", "quality_proof": False, "human_verdict": "unverified",
        "condition": "stress_4x", "frame_window": [0, FRAMES],
        "sample_rate_hz": SAMPLE_RATE, "channels": CHANNELS, "format": dict(FORMAT), "artifacts": [],
    }
    stage = "preflight"
    try:
        _validate_contract(contract)
        require(tool_preflight["generated_roundtrip"]["status"] == "technical_preflight_pass"
                and tool_preflight["generated_roundtrip"]["decode_bit_exact"] is True
                and set(tool_preflight["tools"]) == {"ffmpeg", "ffprobe"},
                "generated tool preflight required before publication")
        require(output.is_dir() and not output.is_symlink(), "caller-created ordinary output directory required")
        paths = [output / f"{policy}.wav" for policy in POLICIES]
        require(all(not os.path.lexists(path) for path in paths), "all three output filenames must be absent")
        raw_crops, full_hashes = _crop_outputs(raw)
        dense.require_numpy()
        raw_peaks = [dense.conservative_true_peak_amplitude(signal) for signal in raw_crops]
        require(all(math.isfinite(peak) and peak > 0 for peak in raw_peaks), "silent/nonfinite raw crop")
        requested_gain = min(1.0, 10 ** (-1.2 / 20.0) / max(raw_peaks))
        gain = np.float32(requested_gain)
        presented = [dense.apply_presentation_gain(signal, gain) for signal in raw_crops]
        evidence["presentation_safety"] = {
            "schema": "riotbox.audio_presentation_true_peak_safety.v1",
            "estimator": "conservative_four_x_bandlimited_fft_v1", "oversample_factor": 4,
            "normalization_target_true_peak_dbtp": -1.2, "max_allowed_true_peak_dbtp": -1.0,
            "requested_uniform_gain": requested_gain, "uniform_gain": float(gain),
            "uniform_gain_f32_bits": f"{int(gain.view(np.uint32)):08x}",
            "uniform_gain_db": _db(float(gain)),
            "maximum_pre_gain_true_peak_dbtp": _db(max(raw_peaks)),
            "pre_gain_true_peak_dbtp": {path.name: _db(peak) for path, peak in zip(paths, raw_peaks, strict=True)},
            "coverage": [path.name for path in paths], "presentation_only": True,
            "makeup_gain": False, "loudness_matching": False, "fades": False,
            "resampling": False, "quality_proof": False, "human_verdict": "unverified",
        }
        evidence["tool_preflight"] = tool_preflight
        evidence["tools"] = tool_preflight["tools"]
        decoded = []
        for policy, path, crop, signal, peak, full_hash in zip(
                POLICIES, paths, raw_crops, presented, raw_peaks, full_hashes, strict=True):
            report = {"policy": policy, "path": str(path), "status": "not_written",
                      "full_raw_pcm_sha256_f32le": full_hash,
                      "raw_crop_pcm_sha256_f32le": metrics.pcm_identity(crop),
                      "raw_crop_level": metrics.level(crop), "raw_crop_true_peak_dbtp": _db(peak),
                      "expected_presented_pcm_sha256_f32le": metrics.pcm_identity(signal)}
            evidence["artifacts"].append(report)
            stage = f"write_{policy}"
            report["status"] = "write_started"
            _write_wav(path, signal)
            report["status"] = "written_unverified"
            stage = f"analyze_{policy}"
            decoded.append(_analyze_wav(path, signal, report))
        stage = "comparisons"
        for report, signal in zip(evidence["artifacts"], decoded, strict=True):
            report["delta_vs_a"] = metrics.delta(decoded[0].reshape(-1), signal.reshape(-1))
            report["per_channel_windows"] = _comparisons(decoded[0], signal)
        evidence["status"] = "technical_preflight_complete_no_playback"
        evidence["presentation_safety"]["result"] = "pass"
        evidence["presentation_safety"]["post_gain_true_peak_dbtp"] = {
            Path(row["path"]).name: row["conservative_true_peak_dbtp"] for row in evidence["artifacts"]}
        evidence["presentation_safety"]["maximum_post_gain_true_peak_dbtp"] = max(
            row["conservative_true_peak_dbtp"] for row in evidence["artifacts"])
        return evidence
    except (ValueError, KeyError, TypeError, OSError, subprocess.SubprocessError) as error:
        evidence.update(status="failed", failed_stage=stage, reason=str(error))
        raise ArtifactPublicationError(str(error), evidence) from error
