#!/usr/bin/env python3

import json
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
