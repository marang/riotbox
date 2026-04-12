#!/usr/bin/env python3

import hashlib
import json
import math
import os
import sys
import wave


PROTOCOL_VERSION = "0.1"
SIDECAR_VERSION = "0.1.0"
SUPPORTED_WAVE_SAMPLE_WIDTHS = {1, 2, 3, 4}
TIMING_BPM_CANDIDATES = [80, 90, 100, 110, 120, 126, 128, 130, 135, 140, 145, 150, 160]


def write_message(message: dict) -> None:
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def build_stub_graph(source: dict, analysis_seed: int) -> dict:
    source_id = source["source_id"]
    source_hash = source["content_hash"]

    return {
        "graph_version": "V1",
        "source": source,
        "timing": {
            "bpm_estimate": 140.0,
            "bpm_confidence": 0.82,
            "meter_hint": {"beats_per_bar": 4, "beat_unit": 4},
            "beat_grid": [
                {"beat_index": 1, "time_seconds": 0.0, "confidence": 0.95},
                {"beat_index": 2, "time_seconds": 0.43, "confidence": 0.94},
            ],
            "bar_grid": [
                {
                    "bar_index": 1,
                    "start_seconds": 0.0,
                    "end_seconds": 1.72,
                    "downbeat_confidence": 0.94,
                    "phrase_index": 1,
                }
            ],
            "phrase_grid": [
                {
                    "phrase_index": 1,
                    "start_bar": 1,
                    "end_bar": 8,
                    "confidence": 0.9,
                }
            ],
        },
        "sections": [
            {
                "section_id": "section-1",
                "label_hint": "Drop",
                "start_seconds": 0.0,
                "end_seconds": 13.76,
                "bar_start": 1,
                "bar_end": 8,
                "energy_class": "High",
                "confidence": 0.88,
                "tags": ["stub", "drop"],
            }
        ],
        "assets": [
            {
                "asset_id": "asset-1",
                "asset_type": "LoopWindow",
                "start_seconds": 0.0,
                "end_seconds": 3.44,
                "start_bar": 1,
                "end_bar": 2,
                "confidence": 0.87,
                "tags": ["loop", "transport_stub"],
                "source_refs": [source_id],
            }
        ],
        "candidates": [
            {
                "candidate_id": "candidate-1",
                "candidate_type": "LoopCandidate",
                "asset_ref": "asset-1",
                "score": 0.91,
                "confidence": 0.86,
                "tags": ["transport_stub"],
                "constraints": ["bar_aligned"],
                "provenance_refs": ["provider:stub.transport"],
            }
        ],
        "relationships": [
            {
                "relation_type": "BelongsToSection",
                "from_id": "asset-1",
                "to_id": "section-1",
                "weight": 1.0,
                "notes": "stub loop belongs to first section",
            }
        ],
        "analysis_summary": {
            "overall_confidence": 0.84,
            "timing_quality": "High",
            "section_quality": "Medium",
            "loop_candidate_count": 1,
            "hook_candidate_count": 0,
            "break_rebuild_potential": "Medium",
            "warnings": [
                {
                    "code": "stub_transport_only",
                    "message": "transport spike returned a stub graph",
                }
            ],
        },
        "provenance": {
            "sidecar_version": SIDECAR_VERSION,
            "provider_set": ["stub.transport"],
            "generated_at": "2026-04-12T19:30:00Z",
            "source_hash": source_hash,
            "analysis_seed": analysis_seed,
            "run_notes": "stdio ndjson spike",
        },
    }


def decode_pcm_samples(frames: bytes, channel_count: int, sample_width: int) -> list[float]:
    if sample_width not in SUPPORTED_WAVE_SAMPLE_WIDTHS:
        raise ValueError(f"unsupported PCM sample width: {sample_width}")

    scale = float((1 << ((sample_width * 8) - 1)) - 1)
    frame_width = channel_count * sample_width
    sample_values = []

    for frame_start in range(0, len(frames), frame_width):
        frame = frames[frame_start : frame_start + frame_width]
        if len(frame) < frame_width:
            break

        channel_sum = 0.0
        for channel_index in range(channel_count):
            start = channel_index * sample_width
            sample_bytes = frame[start : start + sample_width]
            if sample_width == 1:
                value = sample_bytes[0] - 128
                scale_value = 127.0
            else:
                value = int.from_bytes(sample_bytes, byteorder="little", signed=True)
                scale_value = scale
            channel_sum += float(value) / scale_value

        sample_values.append(channel_sum / channel_count)

    return sample_values


def rms(values: list[float]) -> float:
    if not values:
        return 0.0

    return math.sqrt(sum(value * value for value in values) / len(values))


def classify_energy(value: float) -> str:
    if value >= 0.6:
        return "Peak"
    if value >= 0.3:
        return "High"
    if value >= 0.12:
        return "Medium"
    return "Low"


def estimate_timing_from_duration(duration_seconds: float) -> tuple[float, float, int]:
    best_choice = None

    for bpm in TIMING_BPM_CANDIDATES:
        bar_duration = (60.0 / bpm) * 4.0
        if bar_duration <= 0.0:
            continue

        raw_bars = duration_seconds / bar_duration if duration_seconds > 0.0 else 0.0
        rounded_bars = max(2, int(round(raw_bars)))
        fit_error = abs(raw_bars - rounded_bars)

        if rounded_bars < 4:
            range_penalty = (4 - rounded_bars) * 0.25
        elif rounded_bars > 64:
            range_penalty = (rounded_bars - 64) * 0.05
        else:
            range_penalty = 0.0

        score = fit_error + range_penalty
        choice = (score, fit_error, bpm, rounded_bars)
        if best_choice is None or choice < best_choice:
            best_choice = choice

    assert best_choice is not None
    _, fit_error, bpm, bar_count = best_choice
    confidence = max(0.45, min(0.94, 0.94 - (fit_error * 1.6)))
    return float(bpm), round(confidence, 3), bar_count


def build_graph_from_decoded_wave(source_path: str, analysis_seed: int) -> dict:
    canonical_path = os.path.realpath(source_path)

    if not os.path.exists(canonical_path):
        raise FileNotFoundError(canonical_path)

    with open(canonical_path, "rb") as handle:
        content = handle.read()

    content_hash = f"sha256:{hashlib.sha256(content).hexdigest()}"
    source_id = f"src-{content_hash.split(':', 1)[1][:12]}"

    with wave.open(canonical_path, "rb") as wav_file:
        if wav_file.getcomptype() != "NONE":
            raise ValueError(f"unsupported WAV compression: {wav_file.getcomptype()}")

        sample_rate = wav_file.getframerate()
        channel_count = wav_file.getnchannels()
        frame_count = wav_file.getnframes()
        sample_width = wav_file.getsampwidth()
        frames = wav_file.readframes(frame_count)

    if sample_width not in SUPPORTED_WAVE_SAMPLE_WIDTHS:
        raise ValueError(f"unsupported WAV sample width: {sample_width}")

    duration_seconds = max(frame_count / float(sample_rate), 0.001)
    sample_values = decode_pcm_samples(frames, channel_count, sample_width)
    total_energy = rms(sample_values)
    midpoint = max(1, len(sample_values) // 2)
    first_half_energy = rms(sample_values[:midpoint])
    second_half_energy = rms(sample_values[midpoint:])
    peak_abs = max((abs(value) for value in sample_values), default=0.0)

    bpm_estimate, bpm_confidence, bar_count = estimate_timing_from_duration(duration_seconds)
    bar_duration = (60.0 / bpm_estimate) * 4.0
    phrase_count = max(1, math.ceil(bar_count / 8))
    phrase_end_bar = min(bar_count, 8)
    intro_bars = max(1, min(bar_count - 1, bar_count // 2))
    outro_bars = max(intro_bars + 1, bar_count)
    intro_end_seconds = round(min(duration_seconds, intro_bars * bar_duration), 3)
    full_duration = round(duration_seconds, 3)
    loop_end_bar = min(2, bar_count)
    loop_end_seconds = round(min(duration_seconds, loop_end_bar * bar_duration), 3)
    second_section_start_seconds = intro_end_seconds
    second_section_bar_start = min(bar_count, intro_bars + 1)

    warnings = [
        {
            "code": "wav_baseline_only",
            "message": "decoded-source baseline used WAV metadata and simple energy heuristics",
        }
    ]

    source = {
        "source_id": source_id,
        "path": canonical_path,
        "content_hash": content_hash,
        "duration_seconds": full_duration,
        "sample_rate": sample_rate,
        "channel_count": channel_count,
        "decode_profile": "Native",
    }

    beat_grid = []
    for beat_index in range(1, min(bar_count * 4, 16) + 1):
        beat_time = round(((beat_index - 1) * (60.0 / bpm_estimate)), 3)
        if beat_time > duration_seconds:
            break
        beat_grid.append(
            {
                "beat_index": beat_index,
                "time_seconds": beat_time,
                "confidence": round(max(0.5, bpm_confidence - 0.03), 3),
            }
        )

    bar_grid = []
    for bar_index in range(1, min(bar_count, 8) + 1):
        bar_start = round((bar_index - 1) * bar_duration, 3)
        bar_end = round(min(duration_seconds, bar_index * bar_duration), 3)
        if bar_start >= duration_seconds:
            break
        phrase_index = min(phrase_count, ((bar_index - 1) // 8) + 1)
        bar_grid.append(
            {
                "bar_index": bar_index,
                "start_seconds": bar_start,
                "end_seconds": bar_end,
                "downbeat_confidence": bpm_confidence,
                "phrase_index": phrase_index,
            }
        )

    phrase_grid = [
        {
            "phrase_index": 1,
            "start_bar": 1,
            "end_bar": phrase_end_bar,
            "confidence": round(max(0.48, bpm_confidence - 0.04), 3),
        }
    ]
    if bar_count > phrase_end_bar:
        phrase_grid.append(
            {
                "phrase_index": 2,
                "start_bar": phrase_end_bar + 1,
                "end_bar": bar_count,
                "confidence": round(max(0.45, bpm_confidence - 0.08), 3),
            }
        )

    overall_confidence = round(
        min(
            0.9,
            max(
                0.52,
                (bpm_confidence * 0.5)
                + (min(peak_abs, 1.0) * 0.3)
                + (min(total_energy * 2.0, 1.0) * 0.2),
            ),
        ),
        3,
    )

    return {
        "graph_version": "V1",
        "source": source,
        "timing": {
            "bpm_estimate": bpm_estimate,
            "bpm_confidence": bpm_confidence,
            "meter_hint": {"beats_per_bar": 4, "beat_unit": 4},
            "beat_grid": beat_grid,
            "bar_grid": bar_grid,
            "phrase_grid": phrase_grid,
        },
        "sections": [
            {
                "section_id": "section-a",
                "label_hint": "Intro",
                "start_seconds": 0.0,
                "end_seconds": intro_end_seconds,
                "bar_start": 1,
                "bar_end": intro_bars,
                "energy_class": classify_energy(first_half_energy),
                "confidence": round(max(0.5, overall_confidence - 0.08), 3),
                "tags": ["decoded_wave", "entry"],
            },
            {
                "section_id": "section-b",
                "label_hint": "Drop",
                "start_seconds": second_section_start_seconds,
                "end_seconds": full_duration,
                "bar_start": second_section_bar_start,
                "bar_end": outro_bars,
                "energy_class": classify_energy(second_half_energy),
                "confidence": round(max(0.5, overall_confidence - 0.03), 3),
                "tags": ["decoded_wave", "main"],
            },
        ],
        "assets": [
            {
                "asset_id": "asset-loop-1",
                "asset_type": "LoopWindow",
                "start_seconds": 0.0,
                "end_seconds": loop_end_seconds,
                "start_bar": 1,
                "end_bar": loop_end_bar,
                "confidence": round(max(0.52, overall_confidence - 0.05), 3),
                "tags": ["loop", "decoded_wave"],
                "source_refs": [source_id],
            }
        ],
        "candidates": [
            {
                "candidate_id": "candidate-loop-1",
                "candidate_type": "LoopCandidate",
                "asset_ref": "asset-loop-1",
                "score": round(max(0.55, overall_confidence - 0.02), 3),
                "confidence": round(max(0.52, overall_confidence - 0.04), 3),
                "tags": ["decoded_wave"],
                "constraints": ["bar_aligned"],
                "provenance_refs": ["provider:decoded.wav_baseline"],
            }
        ],
        "relationships": [
            {
                "relation_type": "BelongsToSection",
                "from_id": "asset-loop-1",
                "to_id": "section-a",
                "weight": 1.0,
                "notes": "first detected loop belongs to the opening section",
            }
        ],
        "analysis_summary": {
            "overall_confidence": overall_confidence,
            "timing_quality": "Medium" if bpm_confidence < 0.78 else "High",
            "section_quality": "Medium",
            "loop_candidate_count": 1,
            "hook_candidate_count": 0,
            "break_rebuild_potential": "Medium" if peak_abs < 0.55 else "High",
            "warnings": warnings,
        },
        "provenance": {
            "sidecar_version": SIDECAR_VERSION,
            "provider_set": ["decoded.wav_baseline"],
            "generated_at": "2026-04-12T22:30:00Z",
            "source_hash": content_hash,
            "analysis_seed": analysis_seed,
            "run_notes": "decoded source-file ingest baseline",
        },
    }


for raw_line in sys.stdin:
    line = raw_line.strip()
    if not line:
        continue

    try:
        message = json.loads(line)
    except json.JSONDecodeError as error:
        write_message(
            {
                "type": "error",
                "request_id": None,
                "code": "invalid_json",
                "message": str(error),
                "retryable": False,
            }
        )
        continue

    request_type = message.get("type")

    if request_type == "ping":
        write_message(
            {
                "type": "pong",
                "request_id": message["request_id"],
                "protocol_version": PROTOCOL_VERSION,
                "sidecar_version": SIDECAR_VERSION,
            }
        )
    elif request_type == "build_source_graph_stub":
        write_message(
            {
                "type": "source_graph_built",
                "request_id": message["request_id"],
                "graph": build_stub_graph(message["source"], message["analysis_seed"]),
            }
        )
    elif request_type == "analyze_source_file":
        try:
            graph = build_graph_from_decoded_wave(message["source_path"], message["analysis_seed"])
        except FileNotFoundError as error:
            write_message(
                {
                    "type": "error",
                    "request_id": message.get("request_id"),
                    "code": "source_missing",
                    "message": f"source file not found: {error}",
                    "retryable": False,
                }
            )
            continue
        except (ValueError, wave.Error) as error:
            write_message(
                {
                    "type": "error",
                    "request_id": message.get("request_id"),
                    "code": "source_unsupported",
                    "message": str(error),
                    "retryable": False,
                }
            )
            continue
        except OSError as error:
            write_message(
                {
                    "type": "error",
                    "request_id": message.get("request_id"),
                    "code": "source_unreadable",
                    "message": str(error),
                    "retryable": False,
                }
            )
            continue

        write_message(
            {
                "type": "source_graph_built",
                "request_id": message["request_id"],
                "graph": graph,
            }
        )
    else:
        write_message(
            {
                "type": "error",
                "request_id": message.get("request_id"),
                "code": "unknown_request",
                "message": f"unknown request type: {request_type}",
                "retryable": False,
            }
        )
