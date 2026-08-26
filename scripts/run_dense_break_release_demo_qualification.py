#!/usr/bin/env python3
"""Run RIOTBOX-1474's frozen current Dense release-demo qualification."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import os
import re
import subprocess
import sys
import wave
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from run_w30_gesture_vocabulary_qualification import validate_reports
from source_holdout_development_access import validate_contained_source_file


CONTRACT = Path("docs/benchmarks/dense_break_release_demo_qualification_v1.json")
EXPECTED_CONTRACT_SHA256 = "15840ecd0f548343a472e8b2242011717892e1fc56c013d80543e855c9b8aa08"
EXPECTED_ACTION_ORDER = [
    "w30.hook_turnaround",
    "w30.trigger_pad",
    "w30.pitch_dive",
    "w30.trigger_pad",
    "w30.filter_slam",
    "w30.trigger_pad",
]
TRUE_PEAK_RE = re.compile(r"Peak:\s+(-?(?:\d+(?:\.\d+)?|inf))\s+dBFS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--session")
    parser.add_argument("--source-blind-fixtures", action="store_true")
    args = parser.parse_args()

    repo = Path(__file__).resolve().parents[1]
    contract_path = repo / CONTRACT
    contract = load_frozen_contract(contract_path, repo)
    if args.source_blind_fixtures:
        run_source_blind_fixtures(contract, repo)
        print("dense-break release-demo source-blind fixtures: pass")
        return 0
    require(args.session is not None and safe_token(args.session), "safe --session is required")
    return run_qualification(repo, contract, args.session)


def load_frozen_contract(path: Path, repo: Path) -> dict[str, Any]:
    payload = path.read_bytes()
    require(
        hashlib.sha256(payload).hexdigest() == EXPECTED_CONTRACT_SHA256,
        "frozen Dense qualification contract changed",
    )
    contract = json.loads(payload)
    validate_contract(contract, repo, verify_pins=True)
    return contract


def validate_contract(
    contract: dict[str, Any], repo: Path, *, verify_pins: bool
) -> None:
    require(
        contract.get("schema") == "riotbox.dense_break_release_demo_qualification.v1",
        "unexpected Dense qualification schema",
    )
    require(contract.get("schema_version") == 1, "Dense qualification version changed")
    require(contract.get("status") == "frozen", "Dense qualification is not frozen")
    require(contract.get("owner_ticket") == "RIOTBOX-1474", "Dense owner changed")

    source = object_field(contract, "source")
    require(source.get("case_id") == "dense_beat03_130", "Dense source case changed")
    require(source.get("source_family") == "dense_break", "Dense source family changed")
    require(source.get("confirmed_bpm") == 130.0, "confirmed product tempo changed")
    require(source.get("downbeat_seconds") is None, "unexpected manual downbeat")
    require(is_sha256(source.get("sha256")), "Dense source hash is invalid")
    access = object_field(contract, "development_session")
    require(access.get("session_limit") == 1, "Development session limit changed")
    require(access.get("maximum_unique_source_files") == 1, "source-file limit changed")
    require(access.get("fresh_exclusive_access_log_required_before_first_open") is True, "fresh access log is required")
    require(access.get("exact_registered_path_and_hash_only") is True, "exact-path access is required")
    require(access.get("directory_discovery") is False, "directory discovery must stay forbidden")
    require(access.get("holdout_audio_access") is False, "Holdout audio must stay forbidden")
    require(access.get("commercial_reference_access") is False, "commercial audio must stay forbidden")
    require(access.get("source_substitution") is False, "source substitution must stay forbidden")

    journey = object_field(contract, "product_journey")
    require(journey.get("exact_action_order") == EXPECTED_ACTION_ORDER, "Dense action order changed")
    require(journey.get("segment_beats") == [5, 3, 13, 3, 9, 4], "Dense segment lengths changed")
    require(journey.get("duration_beats") == 37, "Dense journey duration changed")
    require(journey.get("ordinary_reentry_committed_beats") == [13, 29, 41], "Dense re-entry points changed")
    require(journey.get("automatic_gesture_selection_allowed") is False, "automatic gesture selection is forbidden")
    require(journey.get("gesture_stacking_allowed") is False, "gesture stacking is forbidden")
    gates = object_field(contract, "technical_gates")
    require(gates.get("product_tempo_bpm") == 130.0, "technical tempo changed")
    require(gates.get("product_tempo_absolute_tolerance") == 0.000001, "tempo tolerance changed")
    require(gates.get("callback_partitions_128_and_257_sample_exact") is True, "callback equality is required")
    require(gates.get("restart_recall_sample_exact_to_ordinary_w30") is True, "restart equality is required")
    require(gates.get("isolated_contributors") == ["w30_preview"], "Dense lane owner changed")
    require(gates.get("other_lanes_active_sample_count") == 0, "other lanes must stay silent")
    require(gates.get("missing_source_active_sample_count") == 0, "missing source must stay silent")
    require(gates.get("limited_sample_count") == 0, "limiter intervention is forbidden")

    identity = object_field(contract, "identity_gate")
    require(identity.get("prior_product_tempo_bpm") == 130.28494262695312, "prior tempo identity changed")
    require(identity.get("current_required_product_tempo_bpm") == 130.0, "current tempo identity changed")
    require(identity.get("additional_playback_if_all_identities_match") is False, "duplicate playback must stay forbidden")
    require(identity.get("changed_artifact_requires_fresh_review") is True, "changed artifact review gate changed")
    human = object_field(contract, "human_review")
    require(human.get("changed_artifact_review_limit") == 1, "human review limit changed")
    require(human.get("audio_judge_label_required") is True, "professional label is required")
    forbidden = object_field(contract, "forbidden")
    for field in (
        "gesture_retune",
        "gesture_reorder",
        "tempo_retune",
        "mix_retune",
        "source_substitution",
        "alternative_variant",
        "inferred_or_strengthened_verdict",
        "duplicate_playback",
        "fallback_music",
        "holdout_audio",
    ):
        require(forbidden.get(field) is True, f"forbidden rule changed: {field}")

    if verify_pins:
        pin_records = [
            object_field(source, "source_corpus"),
            object_field(source, "active_holdout_collision_registry"),
        ]
        for record in pin_records:
            assert_file_pin(repo, record)
        for mechanism in list_field(contract, "frozen_mechanisms"):
            assert_file_pin(
                repo,
                {
                    "path": string_field(mechanism, "contract_path"),
                    "raw_sha256": string_field(mechanism, "contract_raw_sha256"),
                },
            )
        validate_prior_evidence_pins(repo, object_field(contract, "predecessor_evidence"))
        reject_holdout_collision(repo, source)


def validate_prior_evidence_pins(repo: Path, evidence: dict[str, Any]) -> None:
    gesture = object_field(evidence, "gesture_product_qualification")
    assert_file_pin(
        repo,
        {"path": gesture["contract_path"], "raw_sha256": gesture["contract_raw_sha256"]},
    )
    assert_file_pin(
        repo,
        {"path": gesture["review_path"], "raw_sha256": gesture["review_raw_sha256"]},
    )
    musical = object_field(evidence, "gesture_musical_review")
    assert_file_pin(
        repo,
        {"path": musical["review_path"], "raw_sha256": musical["review_raw_sha256"]},
    )
    structured = repo / string_field(musical, "structured_review_path")
    if structured.is_file():
        require(
            sha256_file(structured) == musical.get("structured_review_sha256"),
            "prior structured review changed",
        )
    else:
        durable_review = (repo / string_field(musical, "review_path")).read_text(
            encoding="utf-8"
        )
        require(
            str(musical.get("structured_review_sha256")) in durable_review,
            "durable prior review does not bind the local structured-review identity",
        )
    foundation = object_field(evidence, "current_dense_foundation")
    assert_file_pin(
        repo,
        {"path": foundation["contract_path"], "raw_sha256": foundation["contract_raw_sha256"]},
    )
    assert_file_pin(
        repo,
        {"path": foundation["review_path"], "raw_sha256": foundation["review_raw_sha256"]},
    )


def reject_holdout_collision(repo: Path, source: dict[str, Any]) -> None:
    registry_ref = object_field(source, "active_holdout_collision_registry")
    registry = json.loads((repo / string_field(registry_ref, "path")).read_text())
    require(registry.get("schema") == registry_ref.get("schema"), "Holdout registry schema changed")
    for entry in list_field(registry, "entries"):
        if not str(entry.get("partition", "")).startswith("holdout_"):
            continue
        require(entry.get("case_id") != source.get("case_id"), "source case collides with Holdout")
        require(entry.get("source_path") != source.get("path"), "source path collides with Holdout")
        require(entry.get("sha256") != source.get("sha256"), "source hash collides with Holdout")


def run_source_blind_fixtures(contract: dict[str, Any], repo: Path) -> None:
    validate_contract(contract, repo, verify_pins=True)
    mutations = [
        ("development_session", "holdout_audio_access", True),
        ("development_session", "directory_discovery", True),
        ("development_session", "session_limit", 2),
        ("technical_gates", "product_tempo_bpm", 130.28494262695312),
        ("technical_gates", "restart_recall_sample_exact_to_ordinary_w30", False),
        ("human_review", "changed_artifact_review_limit", 2),
        ("identity_gate", "additional_playback_if_all_identities_match", True),
    ]
    for section, field, value in mutations:
        mutated = copy.deepcopy(contract)
        mutated[section][field] = value
        expect_rejection(lambda value=mutated: validate_contract(value, repo, verify_pins=False))
    mutated_order = copy.deepcopy(contract)
    mutated_order["product_journey"]["exact_action_order"].reverse()
    expect_rejection(lambda: validate_contract(mutated_order, repo, verify_pins=False))


def run_qualification(repo: Path, contract: dict[str, Any], session: str) -> int:
    source = object_field(contract, "source")
    root = repo / object_field(contract, "development_session")["output_root"]
    output = root / f"qualification-{session}"
    access_log_path = root / f"access-log-{session}.json"
    require(not output.exists(), f"qualification output already exists: {output}")
    require(not access_log_path.exists(), f"access log already exists: {access_log_path}")
    root.mkdir(parents=True, exist_ok=True)
    output.mkdir()
    log: dict[str, Any] = {
        "schema": "riotbox.dense_break_release_demo_access_log.v1",
        "ticket": "RIOTBOX-1474",
        "session": session,
        "started_at_utc": utc_now(),
        "contract_path": CONTRACT.as_posix(),
        "contract_sha256": EXPECTED_CONTRACT_SHA256,
        "mode": "one_exact_registered_development_path_no_directory_discovery",
        "requested_case_ids": [source["case_id"]],
        "source_path": source["path"],
        "expected_source_sha256": source["sha256"],
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "opened_unique_development_files": [],
        "status": "started_before_source_open",
    }
    create_exclusive_log(access_log_path, log)

    try:
        source_path = repo / source["path"]
        log["status"] = "verifying_exact_registered_source"
        write_log(access_log_path, log)
        source_metadata = validate_contained_source_file(
            repo,
            Path(source["path"]),
            source["sha256"],
            source["source_format"],
            "RIOTBOX-1474 dense_break release-demo qualification",
        )
        log["opened_unique_development_files"] = [
            {
                "case_id": source["case_id"],
                "source_path": source["path"],
                "sha256": source_metadata["actual_sha256"],
                "format": source_metadata["actual_source_format"],
            }
        ]
        log["status"] = "rendering_exact_current_product_path"
        write_log(access_log_path, log)

        command = [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "riotbox-app",
            "--bin",
            "w30_live_path_render",
            "--",
            "--source",
            str(source_path),
            "--output",
            str(output),
            "--bpm",
            str(source["confirmed_bpm"]),
            "--qualify-gesture-vocabulary-v1",
        ]
        completed = subprocess.run(
            command,
            cwd=repo,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        log["renderer_exit_code"] = completed.returncode
        log["renderer_output_tail"] = completed.stdout.splitlines()[-12:]
        require(completed.returncode == 0, "exact current product renderer failed")

        journey = validate_reports(output, float(source["confirmed_bpm"]))
        validate_current_journey(journey, contract)
        presentation = build_presentation(repo, output, source_path, contract, journey)
        identity = evaluate_prior_identity(contract, journey, presentation)
        review = build_review(output, contract, journey, presentation, identity)
        report = build_qualification_report(
            repo, output, contract, source_metadata, journey, presentation, identity, review
        )
        report_path = output / "qualification-report.json"
        write_json(report_path, report)
        log.update(
            {
                "journey_report_sha256": sha256_file(output / "gesture-vocabulary-qualification.json"),
                "continuous_journey_sha256": presentation["continuous_journey_sha256"],
                "presentation_sha256": presentation["presentation_sha256"],
                "identity_result": identity["result"],
                "review_required": identity["review_required"],
                "qualification_report_sha256": sha256_file(report_path),
                "status": "completed_technical_pass_waiting_human_review"
                if identity["review_required"]
                else "completed_hash_identical_prior_verdict_reuse_eligible",
                "completed_at_utc": utc_now(),
            }
        )
        write_log(access_log_path, log)
        print(report_path.relative_to(repo))
        return 0
    except Exception as error:
        log.update(
            {
                "status": "failed_closed",
                "failure": str(error),
                "completed_at_utc": utc_now(),
            }
        )
        write_log(access_log_path, log)
        raise


def validate_current_journey(journey: dict[str, Any], contract: dict[str, Any]) -> None:
    expected_bpm = float(object_field(contract, "technical_gates")["product_tempo_bpm"])
    tolerance = float(object_field(contract, "technical_gates")["product_tempo_absolute_tolerance"])
    require(abs(float(journey["exact_product_tempo_bpm"]) - expected_bpm) <= tolerance, "current product tempo changed")
    require(journey.get("action_order") == EXPECTED_ACTION_ORDER, "current action order changed")
    require(journey.get("isolated_contributors") == ["w30_preview"], "current lane ownership changed")
    require(journey.get("capture_lineage_unchanged") is True, "capture lineage changed")
    require(journey.get("source_monitor_unchanged") is True, "Source Monitor changed")
    restart = object_field(journey, "restart_recall")
    for field in (
        "preset_survived",
        "capture_identity_preserved",
        "articulation_cleared",
        "sample_exact_to_ordinary_reentry",
        "callback_partition_128_vs_257_sample_exact",
    ):
        require(restart.get(field) is True, f"restart gate failed: {field}")
    require(restart.get("recall_action") == "w30.live_recall", "restart recall action changed")
    require(restart.get("trigger_action") == "w30.trigger_pad", "restart trigger action changed")
    require(restart.get("active_samples", 0) > 0 and restart.get("rms", 0.0) > 0.001, "restart output is silent")
    require(restart.get("pre_limiter_clip_count") == 0, "restart pre-limiter clipping")
    require(restart.get("limited_sample_count") == 0, "restart limiter intervention")
    require(restart.get("post_limiter_clip_count") == 0, "restart post-limiter clipping")


def build_presentation(
    repo: Path,
    output: Path,
    source_path: Path,
    contract: dict[str, Any],
    journey: dict[str, Any],
) -> dict[str, Any]:
    review_dir = output / "review"
    review_dir.mkdir()
    journey_path = output / journey["continuous_journey"]["path"]
    raw_path = review_dir / "raw-dense-release-demo-presentation.wav"
    final_path = review_dir / "00_dense_release_demo_presentation.wav"
    bpm = float(journey["exact_product_tempo_bpm"])
    source_seconds = 8.0 * 60.0 / bpm
    filter_graph = (
        f"[0:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,"
        f"atrim=0:{source_seconds:.9f},asetpts=PTS-STARTPTS[src];"
        "anullsrc=r=48000:cl=stereo:d=1[pause];"
        "[1:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo[journey];"
        "anullsrc=r=48000:cl=stereo:d=1[terminal];"
        "[src][pause][journey][terminal]concat=n=4:v=0:a=1[out]"
    )
    run_checked(
        [
            "ffmpeg",
            "-v",
            "error",
            "-nostdin",
            "-i",
            str(source_path),
            "-i",
            str(journey_path),
            "-filter_complex",
            filter_graph,
            "-map",
            "[out]",
            "-c:a",
            "pcm_s16le",
            str(raw_path),
        ],
        repo,
        "Dense review presentation build failed",
    )
    pre_gain_true_peak = measure_true_peak(raw_path, repo)
    maximum = float(object_field(contract, "presentation")["maximum_true_peak_dbtp"])
    target = maximum - 0.2
    gain_db = min(0.0, target - pre_gain_true_peak) if math.isfinite(pre_gain_true_peak) else 0.0
    gain = 10.0 ** (gain_db / 20.0)
    if gain_db < 0.0:
        run_checked(
            [
                "ffmpeg",
                "-v",
                "error",
                "-nostdin",
                "-i",
                str(raw_path),
                "-af",
                f"volume={gain:.12f}",
                "-c:a",
                "pcm_s16le",
                str(final_path),
            ],
            repo,
            "Dense uniform presentation gain failed",
        )
        raw_path.unlink()
    else:
        raw_path.replace(final_path)
    post_gain_true_peak = measure_true_peak(final_path, repo)
    require(post_gain_true_peak <= maximum + 0.01, "Dense presentation true-peak gate failed")
    with wave.open(str(final_path), "rb") as handle:
        require(handle.getframerate() == 48_000 and handle.getnchannels() == 2, "Dense presentation format changed")
        frames = handle.getnframes()
    return {
        "schema": "riotbox.audio_presentation_true_peak_safety.v1",
        "result": "pass",
        "raw_exact_product_path": str(journey_path.resolve()),
        "presentation_path": str(final_path.resolve()),
        "presentation_sha256": sha256_file(final_path),
        "continuous_journey_sha256": sha256_file(journey_path),
        "duration_seconds": frames / 48_000.0,
        "audible_order": object_field(contract, "presentation")["audible_order"],
        "uniform_gain": gain,
        "uniform_gain_db": gain_db,
        "pre_gain_true_peak_dbtp": pre_gain_true_peak,
        "post_gain_true_peak_dbtp": post_gain_true_peak,
        "maximum_true_peak_dbtp": maximum,
        "quality_proof": False,
        "human_verdict": "unverified",
    }


def evaluate_prior_identity(
    contract: dict[str, Any], journey: dict[str, Any], presentation: dict[str, Any]
) -> dict[str, Any]:
    gate = object_field(contract, "identity_gate")
    audio_equal = presentation["presentation_sha256"] == gate["prior_review_artifact_sha256"]
    journey_equal = presentation["continuous_journey_sha256"] == gate["prior_continuous_journey_sha256"]
    tempo_equal = float(journey["exact_product_tempo_bpm"]) == float(gate["prior_product_tempo_bpm"])
    exact = audio_equal and journey_equal and tempo_equal
    return {
        "schema": gate["schema"],
        "result": "exact_prior_identity" if exact else "changed_current_product_identity",
        "presentation_bit_identical_to_prior": audio_equal,
        "continuous_journey_bit_identical_to_prior": journey_equal,
        "product_tempo_identical_to_prior": tempo_equal,
        "prior_product_tempo_bpm": gate["prior_product_tempo_bpm"],
        "current_product_tempo_bpm": journey["exact_product_tempo_bpm"],
        "prior_human_verdict_reuse_allowed": exact,
        "additional_playback_allowed_if_identical": False,
        "review_required": not exact,
        "changed_artifact_review_budget": 1 if not exact else 0,
    }


def build_review(
    output: Path,
    contract: dict[str, Any],
    journey: dict[str, Any],
    presentation: dict[str, Any],
    identity: dict[str, Any],
) -> dict[str, Any]:
    review_dir = output / "review"
    performance_report = output / "gesture-vocabulary-qualification.json"
    review = {
        "schema": "riotbox.listening_review.v1",
        "schema_version": 1,
        "ticket": "RIOTBOX-1474",
        "pr": None,
        "command": None,
        "source_file": object_field(contract, "source")["path"],
        "seed_or_config": CONTRACT.as_posix(),
        "technical_status": "pass",
        "automated_musical_fitness_status": "pass",
        "human_verdict": "unverified",
        "strongest_element": "none",
        "source_recognition": "unverified",
        "hook_after_two_bars": "unverified",
        "failure_reason": "",
        "preferred_direction": "",
        "avoid": [],
        "concrete_follow_up": "",
        "reviewer": None,
        "demo_readiness_consequence": "unverified_until_human_verdict",
        "expected_audible_behavior": object_field(contract, "presentation")["listening_question_if_changed"],
        "artifacts": {
            "candidate_audio": [presentation["presentation_path"]],
            "source_audio": object_field(contract, "source")["path"],
            "metrics_json": "metrics.json",
            "prompt_markdown": "prompt.md",
        },
        "demo_readiness": "unverified",
        "presentation_safety": presentation,
        "current_product_identity": identity,
        "audio_judge_label": {
            "created_at": "2026-08-26",
            "source_family": "dense_break",
            "source_id": "dense_beat03_130",
            "release_coverage_alias": "dense_break",
            "review_pack_schema": contract["schema"],
            "review_pack_id": "riotbox-1474:dense_beat03_130",
            "artifact_identity": {
                "performance_report_sha256": sha256_file(performance_report),
                "audio_sha256": {
                    "rebuild_only_performance": presentation["presentation_sha256"]
                },
            },
            "artifact_paths": {
                "performance_report": "../gesture-vocabulary-qualification.json",
                "audio": {
                    "rebuild_only_performance": "00_dense_release_demo_presentation.wav"
                },
            },
            "reason_tags": {
                "hook_clarity": "unverified",
                "hardest_hit": "unverified",
                "bass_pressure": "not_claimed_w30_only",
                "destructive_contrast": "unverified",
                "source_character": "unverified",
                "replay_value_after_eight_bars": "unverified",
            },
            "exact_product_path_review_gate": {
                "schema": "riotbox.exact_product_path_review_gate.v1",
                "result": "pass",
                "source_family": "dense_break",
                "product_path_kind": "exact_runtime_mix_live_journey",
                "source_backed": True,
                "source_timing_backed": True,
                "source_graph_capture_lineage_proven": journey["capture_lineage_unchanged"],
                "action_lexicon_queue_commit_proven": journey["action_order"] == EXPECTED_ACTION_ORDER,
                "session_replay_proven": journey["session_round_trip_exact"] and journey["suffix_replay_equivalent"],
                "callback_partitions_sample_exact": journey["continuous_journey"]["callback_partition_128_vs_257_sample_exact"],
                "restart_recall_sample_exact": journey["restart_recall"]["sample_exact_to_ordinary_reentry"],
                "source_role_decision_proven": journey["isolated_contributors"] == ["w30_preview"],
                "scripted_performer_driver": True,
                "hardcoded_musical_output": False,
                "primitive_or_template_only": False,
                "fallback_music_present": False,
                "quality_proof": False,
                "human_verdict": "unverified",
                "promotion_blocked_until_human_pass": True,
                "failure_codes": [],
            },
            "summary": "Exact current confirmed-tempo W-30 Dense journey with three performer-owned gestures and clean ordinary re-entry as the formal demo-worthiness target.",
        },
    }
    metrics = {
        "schema": "riotbox.dense_break_release_demo_review_metrics.v1",
        "technical_status": "pass",
        "product_tempo_bpm": journey["exact_product_tempo_bpm"],
        "action_order": journey["action_order"],
        "continuous_journey": journey["continuous_journey"],
        "restart_recall": journey["restart_recall"],
        "presentation_safety": presentation,
        "identity_gate": identity,
        "quality_proof": False,
        "human_verdict": "unverified",
    }
    prompt = render_prompt(review, identity)
    write_json(review_dir / "review.json", review)
    write_json(review_dir / "metrics.json", metrics)
    (review_dir / "prompt.md").write_text(prompt, encoding="utf-8")
    return review


def render_prompt(review: dict[str, Any], identity: dict[str, Any]) -> str:
    return (
        "# RIOTBOX-1474 Dense Release-Demo Review\n\n"
        "The exact current artifact has passed source identity, confirmed-tempo, queue/commit, "
        "Session/replay, callback-partition, restart/recall, lane-isolation, missing-source, clip, "
        "limiter, and presentation-safety gates. These gates prove execution, not musical quality.\n\n"
        "The presentation contains eight beats of the registered Development source, one second "
        "of silence, the complete current 37-beat W-30 journey, and one second of terminal silence. "
        "Only the source context and W-30 product lane are audible.\n\n"
        f"Prior identity result: `{identity['result']}`. No prior verdict is transferred.\n\n"
        "Question: Is the complete current Dense journey strong, coherent, source-recognizable, "
        "and replay-worthy enough to serve as one formal `dense_break` demo-family example?\n\n"
        f"Artifact: `{review['artifacts']['candidate_audio'][0]}`\n"
    )


def build_qualification_report(
    repo: Path,
    output: Path,
    contract: dict[str, Any],
    source_metadata: dict[str, Any],
    journey: dict[str, Any],
    presentation: dict[str, Any],
    identity: dict[str, Any],
    review: dict[str, Any],
) -> dict[str, Any]:
    review_path = output / "review/review.json"
    return {
        "schema": "riotbox.dense_break_release_demo_qualification_report.v1",
        "ticket": "RIOTBOX-1474",
        "result": "technically_eligible_for_one_professional_human_review"
        if identity["review_required"]
        else "technically_eligible_for_exact_prior_verdict_reuse",
        "contract_sha256": EXPECTED_CONTRACT_SHA256,
        "source": {
            "case_id": object_field(contract, "source")["case_id"],
            "family": "dense_break",
            "sha256": source_metadata["actual_sha256"],
            "format": source_metadata["actual_source_format"],
            "confirmed_bpm": journey["exact_product_tempo_bpm"],
        },
        "product_evidence": {
            "performance_report_path": str((output / "gesture-vocabulary-qualification.json").relative_to(repo)),
            "performance_report_sha256": sha256_file(output / "gesture-vocabulary-qualification.json"),
            "continuous_journey_path": str((output / journey["continuous_journey"]["path"]).relative_to(repo)),
            "continuous_journey_sha256": presentation["continuous_journey_sha256"],
            "action_order": journey["action_order"],
            "restart_recall": journey["restart_recall"],
        },
        "presentation": presentation,
        "prior_identity": identity,
        "review": {
            "path": str(review_path.relative_to(repo)),
            "sha256": sha256_file(review_path),
            "human_verdict": review["human_verdict"],
            "review_budget": identity["changed_artifact_review_budget"],
        },
        "claim_boundary": contract["claim_boundary"],
    }


def measure_true_peak(path: Path, repo: Path) -> float:
    completed = subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-nostats",
            "-i",
            str(path),
            "-filter_complex",
            "ebur128=peak=true",
            "-f",
            "null",
            "-",
        ],
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    require(completed.returncode == 0, "true-peak analysis failed")
    matches = TRUE_PEAK_RE.findall(completed.stdout)
    require(matches, "true-peak analysis produced no peak")
    return float(matches[-1])


def run_checked(command: list[str], cwd: Path, failure: str) -> None:
    completed = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    require(completed.returncode == 0, f"{failure}: {completed.stdout[-1200:]}")


def assert_file_pin(repo: Path, record: dict[str, Any]) -> None:
    path = repo / string_field(record, "path")
    require(path.is_file(), f"pinned file is missing: {path}")
    require(sha256_file(path) == string_field(record, "raw_sha256"), f"pinned file changed: {path}")


def create_exclusive_log(path: Path, payload: dict[str, Any]) -> None:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags, 0o600)
    try:
        os.write(descriptor, (json.dumps(payload, indent=2) + "\n").encode())
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_log(path: Path, payload: dict[str, Any]) -> None:
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    temporary.chmod(0o600)
    temporary.replace(path)


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def safe_token(value: str) -> bool:
    return bool(re.fullmatch(r"[a-zA-Z0-9][a-zA-Z0-9._-]{0,63}", value))


def is_sha256(value: Any) -> bool:
    return isinstance(value, str) and bool(re.fullmatch(r"[0-9a-f]{64}", value))


def object_field(data: dict[str, Any], field: str) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{field} must be an object")
    return value


def list_field(data: dict[str, Any], field: str) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list), f"{field} must be an array")
    return value


def string_field(data: dict[str, Any], field: str) -> str:
    value = data.get(field)
    require(isinstance(value, str) and bool(value), f"{field} must be a string")
    return value


def expect_rejection(operation: Any) -> None:
    try:
        operation()
    except (KeyError, TypeError, ValueError, RuntimeError):
        return
    raise RuntimeError("mutation fixture unexpectedly passed")


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, ValueError, json.JSONDecodeError) as error:
        print(f"dense-break release-demo qualification failed closed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
