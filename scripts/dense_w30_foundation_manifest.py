#!/usr/bin/env python3
"""Build the RIOTBOX-1470 exact ordinary W-30 product manifest."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.dense_w30_foundation_product_manifest.v1"
PRIOR_AUDIO_SHA256 = "7140c8f24e383dc6a7cb75bc6183e03727ef8b5f068b28e9d08ead8371a5ebab"
PRIOR_FILES = {
    "review_document": (
        "docs/reviews/riotbox_1444_w30_pitch_dive_product_qualification_2026-08-19.md",
        "42a66e70c4688f78e91311cbbf68240a000498e430875d57751220f0f49e0c85",
    ),
    "structured_review": (
        "artifacts/audio_qa/local/listening-reviews/RIOTBOX-1444/review.json",
        "99b757860d90c2bceb055c586a8005b64ee867afaea8cccfc7799469fab71a8e",
    ),
    "review_access_log": (
        "artifacts/development/riotbox-1444/review-access-log-v1.json",
        "6989f5eb1f08fb7c258cda4428fe6377cbde96810dae3357c20a632daf07dbac",
    ),
    "session": (
        "artifacts/development/riotbox-1444/product-qualification-v1/dense_beat03_130/session.json",
        "d9decdd58c4d820c7be5bceaa75cdfe34b9f197fb9d633a2ff46ca3e78d40820",
    ),
    "source_graph": (
        "artifacts/development/riotbox-1444/product-qualification-v1/dense_beat03_130/source-graph.json",
        "bfaa59a3c3de2b43df1b410ef8d4719cd3771ac27148f308b8189d25e1c415ab",
    ),
    "qualification": (
        "artifacts/development/riotbox-1444/product-qualification-v1/dense_beat03_130/pitch-dive-qualification.json",
        "92957b1b9ee3f9bf6812466ff898b9bd1fd79e0b8f3827b8c7d6dddb99993688",
    ),
}
EXPECTED_ACTIONS = [
    "SourceTimingConfirmGrid",
    "PresetActivate",
    "CaptureSetLength",
    "CaptureBarGroup",
    "PromoteCaptureToPad",
    "W30TriggerPad",
]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"{path}: JSON root must be an object")
    return value


def serialize_manifest(manifest: dict[str, Any]) -> bytes:
    return (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode()


def manifest_sha256(manifest: dict[str, Any]) -> str:
    return hashlib.sha256(serialize_manifest(manifest)).hexdigest()


def normalize_action(action: dict[str, Any]) -> dict[str, Any]:
    return {
        "command": action["command"],
        "params": action["params"],
        "quantization": action["quantization"],
        "status": action["status"],
        "target": action["target"],
    }


def normalize_capture(capture: dict[str, Any]) -> dict[str, Any]:
    return {
        "assigned_target": capture["assigned_target"],
        "capture_id": capture["capture_id"],
        "capture_type": capture["capture_type"],
        "created_from_action": capture["created_from_action"],
        "lineage_capture_refs": capture["lineage_capture_refs"],
        "resample_generation_depth": capture["resample_generation_depth"],
        "source_origin_refs": capture["source_origin_refs"],
        "source_window": capture["source_window"],
        "storage_path": capture["storage_path"],
    }


def build_manifest(
    session: dict[str, Any],
    source_graph: dict[str, Any],
    runtime: dict[str, Any],
    audio_sha256: str,
    *,
    historical_post_control_action_allowed: bool,
) -> dict[str, Any]:
    actions = session["action_log"]["actions"]
    commands = [action["command"] for action in actions]
    require(commands[: len(EXPECTED_ACTIONS)] == EXPECTED_ACTIONS, "action prefix changed")
    if historical_post_control_action_allowed:
        require(
            commands == EXPECTED_ACTIONS + ["W30PitchDive"],
            "historical Session no longer contains the exact post-control Pitch Dive",
        )
    else:
        require(commands == EXPECTED_ACTIONS, "current Session added an action")
    require(
        all(action["status"] == "Committed" for action in actions[: len(EXPECTED_ACTIONS)]),
        "ordinary W-30 action prefix is not fully committed",
    )
    captures = session["captures"]
    require(len(captures) == 1, "ordinary W-30 path must own exactly one capture")
    source = source_graph["source"]
    timing = source_graph["timing"]
    state = session["runtime_state"]
    lanes = state["lane_state"]
    require(lanes["mc202"]["role"] is None, "MC-202 role did not stay out")
    require(lanes["mc202"]["phrase_ref"] is None, "MC-202 phrase did not stay out")
    require(lanes["tr909"]["pattern_ref"] is None, "TR-909 pattern did not stay out")
    require(not lanes["tr909"]["takeover_enabled"], "TR-909 takeover did not stay out")
    require(not lanes["tr909"]["slam_enabled"], "TR-909 slam did not stay out")
    require(runtime["render"]["isolated_contributors"] == ["w30_preview"], "render is not W-30-only")
    require(runtime["render"]["pre_limiter_clip_count"] == 0, "pre-limiter clipping")
    require(runtime["render"]["limited_sample_count"] == 0, "limiter intervention")
    require(runtime["render"]["post_limiter_clip_count"] == 0, "post-limiter clipping")
    return {
        "schema": SCHEMA,
        "product_path": "ordinary_promoted_w30_control_v1",
        "source": {
            "case_id": "dense_beat03_130",
            "channel_count": source["channel_count"],
            "content_hash": source["content_hash"],
            "decode_profile": source["decode_profile"],
            "sample_rate_hz": source["sample_rate"],
            "source_id": source["source_id"],
        },
        "timing": {
            "confirmed_hypothesis_id": state["source_timing"]["confirmed_grid"]["hypothesis_id"],
            "meter": timing["meter_hint"],
            "product_bpm": runtime["render"]["exact_product_tempo_bpm"],
        },
        "performer_path": {
            "actions": [normalize_action(action) for action in actions[: len(EXPECTED_ACTIONS)]],
            "control_captured_before_any_articulation": True,
        },
        "capture": normalize_capture(captures[0]),
        "session_state": {
            "hook_selection_policy": state["style"]["w30_hook_selection_policy"],
            "macro_state": state["macro_state"],
            "mixer_state": state["mixer_state"],
            "preset": state["style"]["active_preset"],
            "profile": state["style"]["active_profile"],
            "source_monitor_mode": state["source_monitor"]["mode"],
            "w30": {
                "active_bank": lanes["w30"]["active_bank"],
                "focused_pad": lanes["w30"]["focused_pad"],
                "hook_articulation": None,
                "last_capture": lanes["w30"]["last_capture"],
                "preview_mode": lanes["w30"]["preview_mode"],
            },
        },
        "lane_roles": {
            "mc202": "stay_out",
            "source_monitor": "stay_out",
            "tr909": "stay_out",
            "w30": "source_transform_foundation",
        },
        "render": {
            "audio_sha256": audio_sha256,
            "beat_count": 13.0,
            "channel_count": 2,
            "isolated_contributors": ["w30_preview"],
            "limited_sample_count": runtime["render"]["limited_sample_count"],
            "peak_abs": runtime["render"]["peak_abs"],
            "post_limiter_clip_count": runtime["render"]["post_limiter_clip_count"],
            "pre_limiter_clip_count": runtime["render"]["pre_limiter_clip_count"],
            "rms": runtime["render"]["rms"],
            "sample_rate_hz": 48_000,
            "start_beat": 8.0,
        },
    }


def reconstruct_prior_manifest(repo: Path) -> dict[str, Any]:
    resolved: dict[str, Path] = {}
    for label, (relative, expected_sha256) in PRIOR_FILES.items():
        path = repo / relative
        require(path.is_file(), f"missing prior {label}: {path}")
        require(sha256_file(path) == expected_sha256, f"prior {label} hash changed")
        resolved[label] = path
    review = read_json(resolved["structured_review"])
    access = read_json(resolved["review_access_log"])
    qualification = read_json(resolved["qualification"])
    require(review["human_verdict"] == "keep", "prior verdict is not keep")
    require(review["strongest_element"] == "chop", "prior strongest element changed")
    require(review["source_recognition"] == "source_transformed_but_present", "prior source verdict changed")
    require(review["hook_after_two_bars"] == "clear", "prior hook verdict changed")
    require(access["control_sha256"] == PRIOR_AUDIO_SHA256, "prior control identity changed")
    control = qualification["control"]
    runtime = {
        "render": {
            "exact_product_tempo_bpm": qualification["exact_product_tempo_bpm"],
            "isolated_contributors": qualification["isolated_contributors"],
            "peak_abs": control["peak_abs"],
            "rms": control["rms"],
            "pre_limiter_clip_count": control["pre_limiter_clip_count"],
            "limited_sample_count": control["limited_sample_count"],
            "post_limiter_clip_count": control["post_limiter_clip_count"],
        }
    }
    return build_manifest(
        read_json(resolved["session"]),
        read_json(resolved["source_graph"]),
        runtime,
        PRIOR_AUDIO_SHA256,
        historical_post_control_action_allowed=True,
    )


def build_current_manifest(
    session_path: Path,
    source_graph_path: Path,
    runtime_path: Path,
    audio_sha256: str,
) -> dict[str, Any]:
    runtime = read_json(runtime_path)
    runtime["render"]["exact_product_tempo_bpm"] = runtime["w30"]["tempo_bpm"]
    return build_manifest(
        read_json(session_path),
        read_json(source_graph_path),
        runtime,
        audio_sha256,
        historical_post_control_action_allowed=False,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--prior", action="store_true")
    args = parser.parse_args()
    require(args.prior, "only the source-blind --prior reconstruction is available directly")
    repo = Path(__file__).resolve().parent.parent
    print(serialize_manifest(reconstruct_prior_manifest(repo)).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
