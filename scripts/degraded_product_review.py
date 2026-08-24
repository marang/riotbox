#!/usr/bin/env python3
"""Build and record exact-path reviews for degraded/unavailable source handling."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

from validate_user_session_observer_ndjson import load_events, validate_events


SCHEMA = "riotbox.degraded_product_review.v1"
SOURCE_FAMILIES = {"weak_source", "bad_timing", "pad_noise"}
OUTCOMES = {"degraded", "unavailable", "reject"}
OBSERVED_STATES = {"degraded", "unavailable"}
PRODUCT_VERDICTS = {"pass", "needs_fix", "inconclusive", "unverified"}
EVIDENCE_ROLES = {"live_product_review", "fixture_calibration"}
CONSUMER_REASONS = {
    "needs_user_confirmation",
    "fallback_grid",
    "unavailable",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)

    pack = commands.add_parser("pack")
    pack.add_argument("--ticket", required=True)
    pack.add_argument("--output", type=Path, required=True)
    pack.add_argument("--source-family", choices=sorted(SOURCE_FAMILIES), required=True)
    pack.add_argument("--outcome", choices=sorted(OUTCOMES), required=True)
    pack.add_argument("--source", type=Path, required=True)
    pack.add_argument("--source-graph", type=Path, required=True)
    pack.add_argument("--session", type=Path, required=True)
    pack.add_argument("--observer", type=Path, required=True)
    pack.add_argument("--reason", required=True)
    pack.add_argument(
        "--evidence-role",
        choices=sorted(EVIDENCE_ROLES),
        default="live_product_review",
    )

    record = commands.add_parser("record")
    record.add_argument("--review", type=Path, required=True)
    record.add_argument(
        "--product-verdict",
        choices=sorted(PRODUCT_VERDICTS - {"unverified"}),
        required=True,
    )
    record.add_argument("--risk-state-visible", choices=["yes", "no"], required=True)
    record.add_argument("--reason-useful", choices=["yes", "no"], required=True)
    record.add_argument(
        "--next-action-understandable",
        choices=["yes", "no"],
        required=True,
    )
    record.add_argument("--next-action", required=True)
    record.add_argument("--notes", default="")
    record.add_argument("--reviewer", required=True)

    validate = commands.add_parser("validate")
    validate.add_argument("review", type=Path)
    validate.add_argument("--require-human-pass", action="store_true")

    promote = commands.add_parser("promote")
    promote.add_argument("--review", type=Path, required=True)
    promote.add_argument("--bank", type=Path, required=True)
    promote.add_argument("--entry-id", required=True)
    promote.add_argument(
        "--fix-category",
        choices=["source_selection", "ui_cue"],
        default="source_selection",
    )
    promote.add_argument("--demo-worthiness-note", required=True)

    args = parser.parse_args()
    try:
        if args.command == "pack":
            create_pack(args)
        elif args.command == "record":
            record_review(args)
        elif args.command == "validate":
            review = read_json_object(args.review)
            validate_review(review, require_human_pass=args.require_human_pass)
            validate_artifact_identity(review)
            print(f"valid {SCHEMA}: {args.review}")
        else:
            promote_review(args)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid degraded product review: {error}", file=sys.stderr)
        return 1
    return 0


def create_pack(args: argparse.Namespace) -> None:
    source = args.source.resolve()
    source_graph_path = args.source_graph.resolve()
    session_path = args.session.resolve()
    observer_path = args.observer.resolve()
    events = load_events(observer_path)
    validate_events(events)
    source_graph = read_json_object(source_graph_path)
    session = read_json_object(session_path)
    proof = derive_product_path_proof(
        events,
        source,
        source_graph,
        source_graph_path,
        session,
    )
    if args.outcome != "reject" and proof["observed_state"] != args.outcome:
        raise ValueError(
            f"outcome {args.outcome!r} does not match observed state "
            f"{proof['observed_state']!r}"
        )

    args.output.mkdir(parents=True, exist_ok=True)
    review = {
        "schema": SCHEMA,
        "schema_version": 1,
        "ticket": args.ticket,
        "evidence_role": args.evidence_role,
        "quality_proof": False,
        "source_family": args.source_family,
        "outcome": args.outcome,
        "reason": args.reason.strip(),
        "demo_readiness": "not_demo_ready",
        "artifacts": {
            "source": file_ref(source),
            "source_graph": file_ref(source_graph_path),
            "session": file_ref(session_path),
            "observer": file_ref(observer_path),
        },
        "product_path_proof": proof,
        "review_questions": [
            "Is the degraded or unavailable performance state visible before the source score?",
            "Does the displayed reason make the bar/live risk understandable?",
            "Is source-only monitoring clearly distinct from generated Riotbox output?",
            "Does the surface avoid implying that generated Riotbox output exists?",
            "Is the next safe musician action understandable?",
        ],
        "human_review": {
            "product_handling_verdict": "unverified",
            "risk_state_visible": None,
            "reason_useful": None,
            "next_action_understandable": None,
            "next_action": "",
            "notes": "",
            "reviewer": None,
        },
        "review_result": "unverified",
    }
    review_path = args.output / "review.json"
    write_json(review_path, review)
    (args.output / "prompt.md").write_text(render_prompt(review))
    validate_review(review, require_human_pass=False)
    validate_artifact_identity(review)
    print(f"degraded product review pack written: {args.output}")


def record_review(args: argparse.Namespace) -> None:
    review = read_json_object(args.review)
    validate_review(review, require_human_pass=False)
    human = object_field(review, "human_review", "review")
    human.update(
        {
            "product_handling_verdict": args.product_verdict,
            "risk_state_visible": yes_no(args.risk_state_visible),
            "reason_useful": yes_no(args.reason_useful),
            "next_action_understandable": yes_no(args.next_action_understandable),
            "next_action": args.next_action.strip(),
            "notes": args.notes.strip(),
            "reviewer": args.reviewer.strip(),
        }
    )
    review["review_result"] = derived_review_result(human)
    validate_review(review, require_human_pass=False)
    validate_artifact_identity(review)
    write_json(args.review, review)
    prompt = args.review.with_name("prompt.md")
    if prompt.exists():
        prompt.write_text(render_prompt(review))
    print(f"degraded product review recorded: {args.review}")


def promote_review(args: argparse.Namespace) -> None:
    review_path = args.review.resolve()
    review = read_json_object(review_path)
    validate_review(review, require_human_pass=True)
    validate_artifact_identity(review)
    prompt_path = review_path.with_name("prompt.md")
    require(prompt_path.is_file(), f"review prompt missing: {prompt_path}")
    source_ref = object_field(object_field(review, "artifacts", "review"), "source", "artifacts")
    human = object_field(review, "human_review", "review")
    outcome = str(review["outcome"])
    entry = {
        "entry_id": args.entry_id,
        "source_family": review["source_family"],
        "source_path": source_ref["path"],
        "metrics": file_ref(review_path),
        "review_prompt": file_ref(prompt_path),
        "human_verdict": "weak" if outcome == "degraded" else "fail",
        "demo_readiness": "not_demo_ready",
        "quality_claim": False,
        "demo_worthiness_note": args.demo_worthiness_note.strip(),
        "fix_categories": [args.fix_category],
        "human_review_evidence": {
            "schema": "riotbox.demo_bank_human_review_evidence.v1",
            "reviewer": human["reviewer"],
            "reviewer_kind": "human",
            "review_path": str(review_path),
            "review_sha256": sha256_file(review_path),
        },
        "degraded_or_reject_evidence": {
            "schema": "riotbox.demo_bank_degraded_or_reject_review_evidence.v1",
            "outcome": outcome,
            "product_path_reviewed": True,
            "fallback_music_present": False,
            "reason": review["reason"],
        },
        "musical_summary": negative_musical_summary(review),
    }
    bank = (
        read_json_object(args.bank)
        if args.bank.is_file()
        else {
            "schema": "riotbox.release_grade_demo_bank.v1",
            "schema_version": 1,
            "readiness_rubric_schema": "riotbox.sound_product_readiness_rubric.v1",
            "evidence_role": "live_review",
            "hidden_taste_oracle_allowed": False,
            "entries": [],
        }
    )
    require(bank.get("evidence_role") == "live_review", "promotion bank must be live_review")
    entries = list_field(bank, "entries", "bank")
    bank["entries"] = [item for item in entries if not isinstance(item, dict) or item.get("entry_id") != args.entry_id]
    bank["entries"].append(entry)
    from validate_release_grade_demo_bank import validate_manifest as validate_demo_bank

    validate_demo_bank(bank, args.bank)
    args.bank.parent.mkdir(parents=True, exist_ok=True)
    write_json(args.bank, bank)
    print(f"degraded product review promoted: {args.entry_id} -> {args.bank}")


def negative_musical_summary(review: dict[str, Any]) -> dict[str, str]:
    state = review["product_path_proof"]["observed_state"]
    return {
        "hook_within_two_bars": f"Not applicable: {state} handling withheld generated performance output.",
        "hardest_audible_element": "Not applicable: the reviewed product decision did not generate a candidate mix.",
        "source_character": "Raw source preview remained available and was not mislabeled as generated Riotbox output.",
        "destructive_contrast": "Not applicable until source timing becomes trusted through the existing confirmation path.",
        "bass_drum_pressure": "Not assigned: no generated bass or drum lane was configured for this review.",
        "live_triggerability": f"Blocked safely while performance readiness remained {state}.",
        "eight_bar_replay_value": "Not demo-ready by design; correct degraded handling is the reviewed product success.",
    }


def derive_product_path_proof(
    events: list[dict[str, Any]],
    source_path: Path,
    source_graph: dict[str, Any],
    source_graph_path: Path,
    session: dict[str, Any],
) -> dict[str, Any]:
    snapshots = [
        event["snapshot"]
        for event in events
        if isinstance(event.get("snapshot"), dict)
    ]
    require(bool(snapshots), "observer has no assigned review snapshots")
    for item in snapshots:
        item_timing = object_field(item, "source_timing", "snapshot")
        object_field(item_timing, "performance_readiness", "source_timing")
    snapshot = max(
        snapshots,
        key=lambda item: int(object_field(item, "runtime", "snapshot").get("audio_callback_count") or 0),
    )
    timing = object_field(snapshot, "source_timing", "snapshot")
    readiness = object_field(timing, "performance_readiness", "source_timing")
    runtime = object_field(snapshot, "runtime", "snapshot")
    queue = object_field(snapshot, "queue", "snapshot")
    transport = object_field(snapshot, "transport", "snapshot")
    state = enum_value(readiness, "state", OBSERVED_STATES, "performance_readiness")
    reason = enum_value(readiness, "reason", CONSUMER_REASONS, "performance_readiness")
    for item in snapshots:
        item_timing = object_field(item, "source_timing", "snapshot")
        item_readiness = object_field(
            item_timing,
            "performance_readiness",
            "source_timing",
        )
        require(
            item_readiness.get("state") == state
            and item_readiness.get("reason") == reason
            and item_readiness.get("confident_bar_locked_output_allowed") is False
            and item_readiness.get("live_source_policy_active") is False
            and item_readiness.get("generated_output_configured") is False
            and item_readiness.get("fallback_music_present") is False,
            "observer performance readiness changed during assigned review",
        )
        item_lanes = object_field(
            item_readiness,
            "generated_lanes_configured",
            "performance_readiness",
        )
        require(
            all(item_lanes.get(lane) is False for lane in ("tr909", "mc202", "w30")),
            "a generated lane was configured during assigned review",
        )
        item_runtime = object_field(item, "runtime", "snapshot")
        require(
            item_runtime.get("audio_status") in {"unknown", "running"},
            "audio runtime failed during assigned review",
        )
        require(item_runtime.get("source_monitor_mode") == "source", "source monitor changed from source-only")
        require(
            item_runtime.get("source_monitor_audio_route") == "source_only",
            "source monitor route changed from source-only",
        )
        require(item_runtime.get("tr909_mode") == "idle", "TR-909 became active during assigned review")
        require(item_runtime.get("mc202_mode") == "idle", "MC-202 became active during assigned review")
        require(item_runtime.get("mc202_routing") == "silent", "MC-202 routing became audible during assigned review")
        require(
            item_runtime.get("w30_resample_tap_mode") == "idle",
            "W-30 resample tap became active during assigned review",
        )
        item_transport = object_field(item, "transport", "snapshot")
        require(item_transport.get("is_playing") is False, "transport started during assigned review")
        item_queue = object_field(item, "queue", "snapshot")
        require(item_queue.get("pending_count") == 0, "observer queue was not empty during assigned review")
        require(item_queue.get("session_log_count") == 0, "observer session log changed during assigned review")
    require(readiness.get("confident_bar_locked_output_allowed") is False, "bar-locked output was allowed")
    require(readiness.get("live_source_policy_active") is False, "live source policy was active")
    require(readiness.get("generated_output_configured") is False, "generated output was configured")
    require(readiness.get("fallback_music_present") is False, "fallback music was not proven absent")
    lanes = object_field(readiness, "generated_lanes_configured", "performance_readiness")
    require(all(lanes.get(lane) is False for lane in ("tr909", "mc202", "w30")), "a generated lane was configured")
    require(runtime.get("audio_status") == "running", "audio runtime was not running")
    callback_count = runtime.get("audio_callback_count")
    require(isinstance(callback_count, int) and callback_count > 0, "audio callback path was not exercised")
    require(runtime.get("source_monitor_mode") == "source", "source monitor was not source-only")
    require(runtime.get("source_monitor_audio_route") == "source_only", "source monitor route was not source-only")
    require(runtime.get("tr909_mode") == "idle", "TR-909 was not idle")
    require(runtime.get("mc202_mode") == "idle", "MC-202 was not idle")
    require(runtime.get("mc202_routing") == "silent", "MC-202 was not silent")
    require(runtime.get("w30_resample_tap_mode") == "idle", "W-30 resample tap was not idle")
    require(transport.get("is_playing") is False, "transport must remain stopped for review assignment")
    require(queue.get("pending_count") == 0, "observer queue was not empty")
    require(queue.get("session_log_count") == 0, "observer session log was not empty")

    source_sha = sha256_file(source_path)
    graph_source = object_field(source_graph, "source", "source_graph")
    source_id = non_empty_string(graph_source.get("source_id"), "source_graph.source_id")
    require(graph_source.get("content_hash") == f"sha256:{source_sha}", "source graph hash does not match source")
    require(Path(str(graph_source.get("path"))).resolve() == source_path, "source graph path does not match source")
    require(timing.get("source_id") == source_id, "observer source id does not match source graph")
    for item in snapshots:
        item_timing = object_field(item, "source_timing", "snapshot")
        require(
            item_timing.get("source_id") == source_id,
            "observer source id changed during assigned review",
        )
    launch = object_field(events[0], "launch", "observer_started")
    launch_source = launch.get("source_path", launch.get("source"))
    require(
        isinstance(launch_source, str)
        and Path(launch_source).resolve() == source_path,
        "observer launch source does not match reviewed source",
    )
    source_refs = list_field(session, "source_refs", "session")
    require(
        any(
            ref.get("source_id") == source_id
            and ref.get("content_hash") == f"sha256:{source_sha}"
            for ref in source_refs
            if isinstance(ref, dict)
        ),
        "session does not reference exact source hash",
    )
    graph_refs = list_field(session, "source_graph_refs", "session")
    require(
        any(
            isinstance(ref, dict)
            and ref.get("source_id") == source_id
            and Path(str(ref.get("external_path"))).resolve() == source_graph_path
            for ref in graph_refs
        ),
        "session does not reference exact source graph path",
    )
    runtime_state = object_field(session, "runtime_state", "session")
    session_timing = object_field(runtime_state, "source_timing", "runtime_state")
    require(session_timing.get("confirmed_grid") is None, "session unexpectedly contains a confirmed grid")
    action_log = object_field(session, "action_log", "session")
    require(action_log.get("actions") == [], "session contains unexpected actions")
    require(action_log.get("commit_records") == [], "session contains unexpected commits")

    return {
        "observer_schema": events[0].get("schema"),
        "observed_state": state,
        "observed_reason": reason,
        "timing_cue": timing.get("cue"),
        "timing_actionability": timing.get("actionability"),
        "grid_use": timing.get("grid_use"),
        "primary_warning_code": timing.get("primary_warning_code"),
        "confident_bar_locked_output_allowed": False,
        "live_source_policy_active": False,
        "generated_lanes_configured": {lane: False for lane in ("tr909", "mc202", "w30")},
        "generated_output_configured": False,
        "fallback_music_present": False,
        "source_monitor_role": "source_preview_only",
        "audio_status": runtime.get("audio_status"),
        "audio_callback_count": callback_count,
        "transport_playing": False,
        "pending_action_count": 0,
        "committed_action_count": 0,
        "session_grid_confirmed": False,
    }


def validate_review(review: dict[str, Any], require_human_pass: bool) -> None:
    require(review.get("schema") == SCHEMA, f"schema must be {SCHEMA}")
    require(review.get("schema_version") == 1, "schema_version must be 1")
    enum_value(review, "evidence_role", EVIDENCE_ROLES, "review")
    require(review.get("quality_proof") is False, "quality_proof must be false")
    enum_value(review, "source_family", SOURCE_FAMILIES, "review")
    outcome = enum_value(review, "outcome", OUTCOMES, "review")
    reason = non_empty_string(review.get("reason"), "review.reason")
    require(bool(reason), "review.reason must not be empty")
    require(review.get("demo_readiness") == "not_demo_ready", "demo_readiness must be not_demo_ready")
    artifacts = object_field(review, "artifacts", "review")
    for name in ("source", "source_graph", "session", "observer"):
        validate_file_ref(object_field(artifacts, name, "artifacts"), f"artifacts.{name}")
    require(
        str(object_field(artifacts, "source", "artifacts")["path"]).endswith(".wav"),
        "review source artifact must be a WAV",
    )
    proof = object_field(review, "product_path_proof", "review")
    state = enum_value(proof, "observed_state", OBSERVED_STATES, "product_path_proof")
    enum_value(proof, "observed_reason", CONSUMER_REASONS, "product_path_proof")
    if outcome != "reject":
        require(outcome == state, "review outcome must match observed state")
    require(proof.get("confident_bar_locked_output_allowed") is False, "review allows bar-locked output")
    require(proof.get("live_source_policy_active") is False, "review has active live source policy")
    require(proof.get("generated_output_configured") is False, "review has generated output")
    require(proof.get("fallback_music_present") is False, "review has fallback music")
    require(proof.get("source_monitor_role") == "source_preview_only", "source monitor role is misleading")
    require(isinstance(proof.get("audio_callback_count"), int) and proof["audio_callback_count"] > 0, "audio callbacks missing")
    require(proof.get("transport_playing") is False, "transport must be stopped")
    require(proof.get("pending_action_count") == 0, "pending actions must be zero")
    require(proof.get("committed_action_count") == 0, "committed actions must be zero")
    questions = review.get("review_questions")
    require(isinstance(questions, list) and len(questions) >= 5, "review questions are incomplete")
    human = object_field(review, "human_review", "review")
    verdict = enum_value(human, "product_handling_verdict", PRODUCT_VERDICTS, "human_review")
    result = enum_value(review, "review_result", {"pass", "needs_fix", "inconclusive", "unverified"}, "review")
    if verdict == "unverified":
        require(result == "unverified", "unverified review must keep unverified result")
        for field in (
            "risk_state_visible",
            "reason_useful",
            "next_action_understandable",
        ):
            require(human.get(field) is None, f"unverified human_review.{field} must be null")
        require(human.get("reviewer") is None, "unverified human_review.reviewer must be null")
        require(human.get("next_action") == "", "unverified human_review.next_action must be empty")
    else:
        for field in (
            "risk_state_visible",
            "reason_useful",
            "next_action_understandable",
        ):
            require(isinstance(human.get(field), bool), f"human_review.{field} must be boolean")
        non_empty_string(human.get("next_action"), "human_review.next_action")
        reviewer = non_empty_string(human.get("reviewer"), "human_review.reviewer")
        if review.get("evidence_role") == "live_product_review":
            require(
                not reviewer_is_fixture(reviewer),
                "live human reviewer must not be a fixture identity",
            )
        require(result == derived_review_result(human), "review_result does not match human review fields")
    if require_human_pass:
        require(review_is_human_pass(review), "review does not contain an accepted human product pass")


def review_is_human_pass(review: dict[str, Any]) -> bool:
    try:
        validate_review(review, require_human_pass=False)
    except (TypeError, ValueError):
        return False
    human = review["human_review"]
    return (
        review.get("evidence_role") == "live_product_review"
        and review.get("review_result") == "pass"
        and human.get("product_handling_verdict") == "pass"
        and human.get("risk_state_visible") is True
        and human.get("reason_useful") is True
        and human.get("next_action_understandable") is True
        and isinstance(human.get("reviewer"), str)
        and bool(human["reviewer"].strip())
    )


def derived_review_result(human: dict[str, Any]) -> str:
    verdict = human.get("product_handling_verdict")
    if verdict == "unverified":
        return "unverified"
    if (
        verdict == "pass"
        and human.get("risk_state_visible") is True
        and human.get("reason_useful") is True
        and human.get("next_action_understandable") is True
    ):
        return "pass"
    if (
        verdict == "needs_fix"
        or human.get("risk_state_visible") is False
        or human.get("reason_useful") is False
        or human.get("next_action_understandable") is False
    ):
        return "needs_fix"
    return "inconclusive"


def reviewer_is_fixture(reviewer: str) -> bool:
    normalized = reviewer.casefold().replace("_", "-")
    return any(
        marker in normalized
        for marker in ("fixture", "synthetic", "test-listener", "calibration")
    )


def validate_artifact_identity(review: dict[str, Any]) -> None:
    artifacts = object_field(review, "artifacts", "review")
    artifact_paths: dict[str, Path] = {}
    for name in ("source", "source_graph", "session", "observer"):
        ref = object_field(artifacts, name, "artifacts")
        path = Path(ref["path"])
        require(path.is_file(), f"artifacts.{name}.path does not exist")
        require(sha256_file(path) == ref["sha256"], f"artifacts.{name} hash is stale")
        artifact_paths[name] = path

    events = load_events(artifact_paths["observer"])
    validate_events(events)
    derived_proof = derive_product_path_proof(
        events,
        artifact_paths["source"].resolve(),
        read_json_object(artifact_paths["source_graph"]),
        artifact_paths["source_graph"].resolve(),
        read_json_object(artifact_paths["session"]),
    )
    require(
        review.get("product_path_proof") == derived_proof,
        "product_path_proof does not match the bound runtime artifacts",
    )


def render_prompt(review: dict[str, Any]) -> str:
    proof = review["product_path_proof"]
    human = review["human_review"]
    lines = [
        f"# {review['ticket']} Degraded Product Review",
        "",
        f"- Source family: `{review['source_family']}`",
        f"- Product outcome: `{review['outcome']}`",
        f"- Observed state / reason: `{proof['observed_state']}` / `{proof['observed_reason']}`",
        f"- UI cue / actionability: `{proof['timing_cue']}` / `{proof['timing_actionability']}`",
        f"- Generated output configured: `{str(proof['generated_output_configured']).lower()}`",
        f"- Fallback music present: `{str(proof['fallback_music_present']).lower()}`",
        f"- Source monitor role: `{proof['source_monitor_role']}`",
        f"- Review result: `{review['review_result']}`",
        f"- Reviewer: `{human['reviewer'] or 'unverified'}`",
        "",
        "## Intent",
        "",
        review["reason"],
        "",
        "## Questions",
        "",
    ]
    lines.extend(f"- {question}" for question in review["review_questions"])
    lines.append("")
    return "\n".join(lines)


def file_ref(path: Path) -> dict[str, str]:
    require(path.is_file(), f"missing artifact: {path}")
    return {"path": str(path), "sha256": sha256_file(path)}


def validate_file_ref(ref: dict[str, Any], prefix: str) -> None:
    non_empty_string(ref.get("path"), f"{prefix}.path")
    digest = non_empty_string(ref.get("sha256"), f"{prefix}.sha256")
    require(len(digest) == 64 and all(char in "0123456789abcdef" for char in digest), f"{prefix}.sha256 invalid")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    return object_value(value, str(path))


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def object_field(parent: dict[str, Any], field: str, prefix: str) -> dict[str, Any]:
    return object_value(parent.get(field), f"{prefix}.{field}")


def object_value(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def list_field(parent: dict[str, Any], field: str, prefix: str) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{prefix}.{field} must be an array")
    return value


def enum_value(parent: dict[str, Any], field: str, allowed: set[str], prefix: str) -> str:
    value = parent.get(field)
    require(value in allowed, f"{prefix}.{field} must be one of {sorted(allowed)}")
    return str(value)


def non_empty_string(value: Any, name: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise TypeError(f"{name} must be a non-empty string")
    return value


def yes_no(value: str) -> bool:
    return value == "yes"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
