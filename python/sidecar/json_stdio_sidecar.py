#!/usr/bin/env python3

import hashlib
import json
import os
import sys


PROTOCOL_VERSION = "0.1"
SIDECAR_VERSION = "0.1.0"


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


def build_graph_from_file(source_path: str, analysis_seed: int) -> dict:
    canonical_path = os.path.realpath(source_path)

    if not os.path.exists(canonical_path):
        raise FileNotFoundError(canonical_path)

    with open(canonical_path, "rb") as handle:
        content = handle.read()

    content_hash = f"sha256:{hashlib.sha256(content).hexdigest()}"
    source_id = f"src-{content_hash.split(':', 1)[1][:12]}"
    file_size = len(content)
    sample_rate = 44_100
    channel_count = 2
    bytes_per_second = sample_rate * channel_count * 2
    duration_seconds = max(file_size / bytes_per_second, 1.0)
    bpm_estimate = 120.0 + float(analysis_seed % 24)
    bar_duration = (60.0 / bpm_estimate) * 4.0
    bars = max(2, min(8, int(round(duration_seconds / bar_duration))))
    section_end_seconds = round(bar_duration * bars, 3)
    half_section_seconds = round(section_end_seconds / 2.0, 3)
    loop_end_seconds = round(min(bar_duration * 2.0, section_end_seconds), 3)

    source = {
        "source_id": source_id,
        "path": canonical_path,
        "content_hash": content_hash,
        "duration_seconds": round(duration_seconds, 3),
        "sample_rate": sample_rate,
        "channel_count": channel_count,
        "decode_profile": "Native",
    }

    return {
        "graph_version": "V1",
        "source": source,
        "timing": {
            "bpm_estimate": bpm_estimate,
            "bpm_confidence": 0.74,
            "meter_hint": {"beats_per_bar": 4, "beat_unit": 4},
            "beat_grid": [
                {"beat_index": 1, "time_seconds": 0.0, "confidence": 0.82},
                {
                    "beat_index": 2,
                    "time_seconds": round(bar_duration / 4.0, 3),
                    "confidence": 0.8,
                },
            ],
            "bar_grid": [
                {
                    "bar_index": 1,
                    "start_seconds": 0.0,
                    "end_seconds": round(bar_duration, 3),
                    "downbeat_confidence": 0.83,
                    "phrase_index": 1,
                }
            ],
            "phrase_grid": [
                {
                    "phrase_index": 1,
                    "start_bar": 1,
                    "end_bar": bars,
                    "confidence": 0.77,
                }
            ],
        },
        "sections": [
            {
                "section_id": "section-a",
                "label_hint": "Intro",
                "start_seconds": 0.0,
                "end_seconds": half_section_seconds,
                "bar_start": 1,
                "bar_end": max(1, bars // 2),
                "energy_class": "Medium",
                "confidence": 0.72,
                "tags": ["file_ingest", "entry"],
            },
            {
                "section_id": "section-b",
                "label_hint": "Drop",
                "start_seconds": half_section_seconds,
                "end_seconds": section_end_seconds,
                "bar_start": max(2, (bars // 2) + 1),
                "bar_end": bars,
                "energy_class": "High",
                "confidence": 0.76,
                "tags": ["file_ingest", "main"],
            },
        ],
        "assets": [
            {
                "asset_id": "asset-loop-1",
                "asset_type": "LoopWindow",
                "start_seconds": 0.0,
                "end_seconds": loop_end_seconds,
                "start_bar": 1,
                "end_bar": min(2, bars),
                "confidence": 0.79,
                "tags": ["loop", "file_ingest"],
                "source_refs": [source_id],
            }
        ],
        "candidates": [
            {
                "candidate_id": "candidate-loop-1",
                "candidate_type": "LoopCandidate",
                "asset_ref": "asset-loop-1",
                "score": 0.84,
                "confidence": 0.78,
                "tags": ["file_ingest"],
                "constraints": ["bar_aligned"],
                "provenance_refs": ["provider:stub.file_ingest"],
            }
        ],
        "relationships": [
            {
                "relation_type": "BelongsToSection",
                "from_id": "asset-loop-1",
                "to_id": "section-a",
                "weight": 1.0,
                "notes": "first loop belongs to first section",
            }
        ],
        "analysis_summary": {
            "overall_confidence": 0.75,
            "timing_quality": "Medium",
            "section_quality": "Medium",
            "loop_candidate_count": 1,
            "hook_candidate_count": 0,
            "break_rebuild_potential": "Medium",
            "warnings": [
                {
                    "code": "decode_not_run",
                    "message": "file ingest stub estimated graph structure without full decode",
                }
            ],
        },
        "provenance": {
            "sidecar_version": SIDECAR_VERSION,
            "provider_set": ["stub.file_ingest"],
            "generated_at": "2026-04-12T21:00:00Z",
            "source_hash": content_hash,
            "analysis_seed": analysis_seed,
            "run_notes": "stdio source-file ingest slice",
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
            graph = build_graph_from_file(message["source_path"], message["analysis_seed"])
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
