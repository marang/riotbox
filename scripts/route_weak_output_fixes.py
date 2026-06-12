#!/usr/bin/env python3
"""Route weak Riotbox audio reports to concrete production fix categories."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from audio_qa_evidence_boundary import apply_evidence_boundary
import validate_automated_musical_fitness
import validate_destructive_variation_professional
import validate_sparse_bass_pressure_professional
import validate_tonal_hook_professional


SCHEMA = "riotbox.weak_output_fix_routing.v1"
DEFAULT_MANIFEST = Path("scripts/fixtures/weak_output_fix_routing/manifest.json")
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-weak-output-fix-routing")
CATEGORIES = {
    "source_selection",
    "chop_policy",
    "drum_pressure",
    "bass_movement",
    "mix_bus",
    "destructive_gesture",
    "fixture_threshold",
    "ui_cue",
}
CATEGORY_ORDER = [
    "chop_policy",
    "bass_movement",
    "destructive_gesture",
    "mix_bus",
    "source_selection",
    "drum_pressure",
    "ui_cue",
    "fixture_threshold",
]

CODE_RULES: tuple[tuple[str, str, int, str], ...] = (
    ("w30_hook", "chop_policy", 5, "W-30 hook is weak or not dominant."),
    ("hook_transient", "chop_policy", 5, "Hook transient does not cut through."),
    ("hookless", "chop_policy", 5, "The source family needs a stronger riff/chop policy."),
    ("w30_trigger", "chop_policy", 4, "W-30 trigger variation is too static."),
    ("w30_slice", "chop_policy", 4, "W-30 slice choices are not varied enough."),
    ("w30_unique", "chop_policy", 4, "W-30 source offsets are too narrow."),
    ("w30_velocity", "chop_policy", 3, "W-30 accent dynamics are too flat."),
    ("source_copy", "chop_policy", 5, "The source is copied instead of transformed into a hook."),
    ("identity_", "chop_policy", 4, "The response is too close to source or fallback identity."),
    ("response_signature_missing", "chop_policy", 3, "The generated response lacks a unique signature."),
    ("pressure_section_lacks_bass_lift", "bass_movement", 5, "Pressure section lacks low-end lift."),
    ("low_end", "bass_movement", 5, "Low-end pressure is weak."),
    ("low_band_pressure", "bass_movement", 5, "Bass pressure is weak in the low band."),
    ("low_band_support", "bass_movement", 4, "Low-band support is weak."),
    ("mc202_bass", "bass_movement", 5, "MC-202 bass pressure is too weak."),
    ("mc202_pressure", "bass_movement", 4, "MC-202 pressure behavior is missing or weak."),
    ("bass_pressure", "bass_movement", 5, "Bass pressure is not carrying the section."),
    ("rebuild_only_too_weak", "mix_bus", 5, "Rebuild-only output is too weak without raw source layer."),
    ("rebuild_only_too_quiet", "mix_bus", 5, "Rebuild-only output is too quiet against the source."),
    ("rebuild_only_too_source_masked", "source_selection", 5, "Rebuild-only output still follows the source too closely."),
    (
        "rebuild_only_source_character_not_surviving",
        "source_selection",
        7,
        "Rebuild-only output lost the transformed source identity.",
    ),
    (
        "rebuild_only_source_transient_character_lost",
        "source_selection",
        6,
        "Rebuild-only output lost the source transient signature.",
    ),
    (
        "rebuild_only_source_spectral_character_lost",
        "source_selection",
        6,
        "Rebuild-only output lost the source spectral shape.",
    ),
    ("source_layer_toggle", "fixture_threshold", 4, "Source-layer-off toggle did not produce a distinct diagnostic render."),
    ("rebuild_only_pressure", "bass_movement", 4, "Rebuild-only pressure does not carry enough low-end movement."),
    ("rebuild_only_restore", "destructive_gesture", 4, "Rebuild-only restore impact is too weak after pressure."),
    ("tr909", "drum_pressure", 4, "TR-909 drum pressure is missing or decorative."),
    ("kick_pressure", "drum_pressure", 4, "Kick pressure is not strong enough."),
    ("dropout", "destructive_gesture", 5, "Dropout/stutter contrast is too weak."),
    ("stutter", "destructive_gesture", 5, "Stutter gesture does not hit hard enough."),
    ("restore", "destructive_gesture", 4, "Restore gesture is not bigger than the cut."),
    ("bars_too_static", "destructive_gesture", 4, "Bars collapse into a static loop."),
    ("movement_bar_similarity", "destructive_gesture", 4, "Bar movement is too static."),
    ("destructive_contrast", "destructive_gesture", 5, "Destructive contrast is weak or missing."),
    ("arrangement_missing_hook", "chop_policy", 5, "Arrangement lost the hook role."),
    ("arrangement_missing_chop", "chop_policy", 5, "Arrangement lost the chop role."),
    ("arrangement_missing_pressure", "bass_movement", 5, "Arrangement lost bass-pressure role."),
    ("arrangement_pressure_lift", "bass_movement", 5, "Arrangement pressure lift is too short."),
    ("arrangement_missing_dropout", "destructive_gesture", 5, "Arrangement lost dropout contrast."),
    ("arrangement_missing_restore", "destructive_gesture", 5, "Arrangement lost restore impact."),
    ("arrangement_destructive", "destructive_gesture", 5, "Arrangement lost destructive tail logic."),
    ("arrangement_role_order", "destructive_gesture", 4, "Arrangement role order collapsed or is invalid."),
    ("arrangement_policy", "destructive_gesture", 4, "Arrangement policy did not make a useful source-aware decision."),
    ("generated_support_balance", "mix_bus", 5, "Generated support/source balance is out of range."),
    ("generated_support_too", "mix_bus", 4, "Generated support is not balanced usefully."),
    ("support_masks", "mix_bus", 5, "Generated support masks the source or hook."),
    ("source_first_generated", "mix_bus", 5, "Source-first balance masks the useful response."),
    ("full_mix_too_quiet", "mix_bus", 4, "Full mix is too quiet."),
    ("technical_near_silence", "mix_bus", 5, "Technical output is near silence."),
    ("full_mix_near_clipping", "mix_bus", 4, "Full mix is near clipping."),
    ("technical_clipping", "mix_bus", 4, "Technical output is clipping-prone."),
    ("source_relation_missing", "source_selection", 4, "Source relation lacks anchor evidence."),
    ("source_anchor", "source_selection", 4, "Source anchor evidence is too weak."),
    ("source_not_transformed", "source_selection", 4, "Source transformation sits outside the useful range."),
    ("source_lost", "source_selection", 5, "Source character is lost."),
    ("source_masked", "mix_bus", 5, "Source character is masked by generated support."),
    ("fallback_collapse", "source_selection", 5, "Output collapsed to fallback identity."),
    ("source_report_schema", "fixture_threshold", 4, "Fixture/report schema mismatch needs QA threshold work."),
    ("source_report_not_passed", "fixture_threshold", 3, "Input report already failed its own gate."),
    ("threshold", "fixture_threshold", 3, "Fixture threshold needs review."),
    ("grid_drift", "ui_cue", 4, "Timing drift needs a user-visible cue or cautious routing."),
    ("peak_offset", "ui_cue", 3, "Grid peak offset is too loose for a confident musical move."),
    ("low_timing_confidence", "ui_cue", 5, "Source timing confidence is too low for confident moves."),
    ("timing_unavailable", "ui_cue", 5, "Source timing is unavailable and needs a visible cautious path."),
    ("unavailable", "ui_cue", 4, "Source/timing evidence is unavailable."),
    ("manual_confirm_only", "ui_cue", 4, "Timing must be manually confirmed before confident moves."),
    ("candidate_ambiguous", "ui_cue", 5, "Timing candidate is ambiguous."),
    ("ambiguous_downbeat", "ui_cue", 5, "Downbeat ambiguity needs confirmation or cautious routing."),
    ("bar_locked_policy_on_bad_timing", "ui_cue", 5, "Bad timing source reached a bar-locked policy path."),
    ("pad_noise", "source_selection", 5, "Pad/noise material needs its own source policy, not dense-break promotion."),
)

TAG_RULES: tuple[tuple[str, str, str, int, str], ...] = (
    ("hook_clarity", "weak", "chop_policy", 5, "Human label says hook clarity is weak."),
    ("hook_clarity", "missing", "chop_policy", 5, "Human label says hook is missing."),
    ("bass_pressure", "weak", "bass_movement", 5, "Human label says bass pressure is weak."),
    ("bass_pressure", "missing", "bass_movement", 5, "Human label says bass pressure is missing."),
    (
        "destructive_contrast",
        "weak",
        "destructive_gesture",
        5,
        "Human label says destructive contrast is weak.",
    ),
    (
        "destructive_contrast",
        "missing",
        "destructive_gesture",
        5,
        "Human label says destructive contrast is missing.",
    ),
    ("source_character", "source_lost", "source_selection", 5, "Human label says source is lost."),
    (
        "source_character",
        "source_copy",
        "chop_policy",
        5,
        "Human label says source-copy collapse is present.",
    ),
    (
        "replay_value_after_eight_bars",
        "none",
        "destructive_gesture",
        4,
        "Human label says replay value is absent.",
    ),
    (
        "replay_value_after_eight_bars",
        "low",
        "destructive_gesture",
        4,
        "Human label says replay value is low.",
    ),
)

AVOID_RULES: tuple[tuple[str, str, int, str], ...] = (
    ("weak hook", "chop_policy", 4, "Avoid-list calls out weak hook."),
    ("source-copy", "chop_policy", 4, "Avoid-list calls out source-copy collapse."),
    ("buried bass", "bass_movement", 4, "Avoid-list calls out buried bass."),
    ("fallback", "source_selection", 4, "Avoid-list calls out fallback collapse."),
    ("source character lost", "source_selection", 4, "Avoid-list calls out lost source character."),
    ("no destructive", "destructive_gesture", 4, "Avoid-list calls out missing destructive contrast."),
    ("polite", "destructive_gesture", 3, "Avoid-list calls out polite output."),
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-weak-output-fix-routing")
    args = parser.parse_args()

    try:
        manifest = read_json_object(args.manifest)
        report = build_report(manifest, args.manifest, args.date)
        args.output.mkdir(parents=True, exist_ok=True)
        write_reports(args.output, report)
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid weak-output fix routing: {error}", file=sys.stderr)
        return 1

    if report["result"] != "pass":
        print(
            "weak-output fix routing failed: " + ", ".join(report["failure_codes"]),
            file=sys.stderr,
        )
        return 1
    print(f"weak-output fix routing written to {args.output}")
    return 0


def build_report(manifest: dict[str, Any], manifest_path: Path, date: str) -> dict[str, Any]:
    require(manifest.get("schema") == SCHEMA, f"{manifest_path}: schema must be {SCHEMA}")
    require(manifest.get("schema_version") == 1, f"{manifest_path}: schema_version must be 1")
    entries = manifest.get("entries")
    require(isinstance(entries, list) and entries, f"{manifest_path}: entries must be non-empty")

    cases = []
    failures = []
    for index, entry in enumerate(entries):
        require(isinstance(entry, dict), f"{manifest_path}: entry {index} must be object")
        case = build_case(entry, manifest_path.parent)
        cases.append(case)
        expected = entry.get("expected_next_fix_category")
        if isinstance(expected, str) and expected != case["proposed_next_fix_category"]:
            failures.append(
                f"{case['case_id']}_expected_{expected}_got_{case['proposed_next_fix_category']}"
            )
    if not any(case["source_verdict"] in {"agent_weak", "agent_fail", "weak", "fail"} for case in cases):
        failures.append("no_weak_or_fail_case_routed")
    for case in cases:
        if not case["proposed_fix_categories"]:
            failures.append(f"{case['case_id']}_missing_fix_category")
        if case["automated_musical_approval"] is not False:
            failures.append(f"{case['case_id']}_claims_automated_musical_approval")
        if case["quality_proof"] is not False:
            failures.append(f"{case['case_id']}_claims_quality_proof")
        if not case["matched_known_routing_signal"]:
            failures.append(f"{case['case_id']}_unknown_failure_route")
        reason = case.get("musician_fix_reason")
        if not isinstance(reason, str) or not reason.strip():
            failures.append(f"{case['case_id']}_missing_musician_fix_reason")
    candidates = build_production_fix_candidates(cases)
    if not candidates:
        failures.append("production_fix_candidates_missing")
    for candidate in candidates:
        candidate_id = candidate["candidate_id"]
        if candidate["quality_proof"] is not False:
            failures.append(f"{candidate_id}_claims_quality_proof")
        if candidate["automated_musical_approval"] is not False:
            failures.append(f"{candidate_id}_claims_automated_musical_approval")
        if not candidate["artifact_refs"]:
            failures.append(f"{candidate_id}_missing_artifact_refs")
        if not candidate["musician_payoff"]:
            failures.append(f"{candidate_id}_missing_musician_payoff")

    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "created_at": date,
        "result": "pass" if not failures else "fail",
        "agent_verdict": "agent_promising" if not failures else "agent_fail",
        "human_verdict": "unverified",
        "automated_musical_approval": False,
        "manifest": str(manifest_path),
        "case_count": len(cases),
        "routed_case_count": sum(1 for case in cases if case["proposed_fix_categories"]),
        "fix_categories": sorted({category for case in cases for category in case["proposed_fix_categories"]}),
        "production_fix_candidate_count": len(candidates),
        "production_fix_candidates": candidates,
        "cases": cases,
        "failure_codes": failures,
        "boundary": (
            "Weak-output routing turns known weak/fail signals into production "
            "work categories. It may reject or route weak output, but it must "
            "not claim automated musical approval or human musical pass."
        ),
    }
    return apply_evidence_boundary(
        report,
        evidence_role="diagnostic",
        source_backed=all(case["source_backed"] for case in cases),
        source_timing_backed=all(case["source_timing_backed"] for case in cases),
        scripted_generation=True,
        notes=(
            "This report is a routing and actionability diagnostic. It does "
            "not prove product sound quality."
        ),
    )


def build_production_fix_candidates(cases: list[dict[str, Any]]) -> list[dict[str, Any]]:
    candidates = []
    for category in CATEGORY_ORDER:
        category_cases = [
            case for case in cases if category in case["proposed_fix_categories"]
        ]
        if not category_cases:
            continue
        primary_cases = [
            case for case in category_cases if case["proposed_next_fix_category"] == category
        ]
        score = len(primary_cases) * 2 + len(category_cases)
        artifact_refs = sorted(
            {
                str(case["artifact_to_hear"])
                for case in category_cases
                if isinstance(case.get("artifact_to_hear"), str) and case["artifact_to_hear"]
            }
        )
        source_families = sorted(
            {
                str(case["source_family"])
                for case in category_cases
                if isinstance(case.get("source_family"), str) and case["source_family"]
            }
        )
        candidates.append(
            {
                "candidate_id": f"p023_fix_{category}",
                "category": category,
                "score": score,
                "primary_case_count": len(primary_cases),
                "case_count": len(category_cases),
                "case_ids": [str(case["case_id"]) for case in category_cases],
                "primary_case_ids": [str(case["case_id"]) for case in primary_cases],
                "source_families": source_families,
                "artifact_refs": artifact_refs,
                "software_next_step": production_software_next_step(category),
                "musician_payoff": production_musician_payoff(category),
                "routing_reasons": {
                    str(case["case_id"]): case["routing_reasons"].get(category, [])
                    for case in category_cases
                    if case["routing_reasons"].get(category)
                },
                "evidence_role": "production_fix_candidate",
                "quality_proof": False,
                "automated_musical_approval": False,
            }
        )
    return sorted(
        candidates,
        key=lambda candidate: (-candidate["score"], CATEGORY_ORDER.index(candidate["category"])),
    )


def build_case(entry: dict[str, Any], fixture_dir: Path) -> dict[str, Any]:
    case_id = required_string(entry, "case_id")
    kind = required_string(entry, "kind")
    source = load_source(entry, fixture_dir)
    signals = source["failure_codes"] + reason_tag_signals(source["reason_tags"]) + source["avoid"]
    route = route_signals(signals, source["reason_tags"], source["avoid"])
    return {
        "case_id": case_id,
        "input_kind": kind,
        "source_report": source["source_report"],
        "source_schema": source["source_schema"],
        "source_family": source["source_family"],
        "source_result": source["source_result"],
        "source_verdict": source["source_verdict"],
        "human_verdict": source["human_verdict"],
        "artifact_to_hear": source["artifact_to_hear"],
        "strongest_audible_element": source["strongest_audible_element"],
        "main_weakness": route["main_weakness"],
        "proposed_next_fix_category": route["proposed_next_fix_category"],
        "proposed_fix_categories": route["proposed_fix_categories"],
        "musician_fix_reason": route["musician_fix_reason"],
        "routing_reasons": route["routing_reasons"],
        "matched_known_routing_signal": route["matched_known_routing_signal"],
        "failure_codes": source["failure_codes"],
        "reason_tags": source["reason_tags"],
        "avoid": source["avoid"],
        "source_backed": source["source_backed"],
        "source_timing_backed": source["source_timing_backed"],
        "scripted_generation": source["scripted_generation"],
        "quality_proof": False,
        "automated_musical_approval": False,
    }


def load_source(entry: dict[str, Any], fixture_dir: Path) -> dict[str, Any]:
    kind = required_string(entry, "kind")
    path = resolve_path(fixture_dir, Path(required_string(entry, "path")))
    if kind == "agent_review":
        return source_from_agent_review(path)
    if kind == "human_label_corpus":
        label_id = required_string(entry, "label_id")
        return source_from_human_label(path, label_id)
    if kind == "destructive_report":
        return source_from_report(
            validate_destructive_variation_professional.build_report(path),
            path,
            source_family="dense_break",
            artifact_to_hear=str(path),
        )
    if kind == "tonal_manifest":
        report = validate_tonal_hook_professional.build_report(path)
        return source_from_report(report, path, artifact_to_hear=str(path))
    if kind == "sparse_bass_manifest":
        report = validate_sparse_bass_pressure_professional.build_report(path)
        return source_from_report(report, path, artifact_to_hear=str(path))
    if kind == "automated_musical_fitness_manifest":
        candidates = validate_automated_musical_fitness.collect_candidates([path])
        report = validate_automated_musical_fitness.build_report(candidates)
        selected = report["selected_candidate"]
        return source_from_report(
            report,
            path,
            source_family=entry.get("source_family", selected["case_id"]),
            artifact_to_hear=selected["manifest"],
        )
    if kind == "dense_performance_report":
        return source_from_dense_performance_report(path)
    raise ValueError(f"unsupported input kind: {kind}")


def source_from_agent_review(path: Path) -> dict[str, Any]:
    report = read_json_object(path)
    audio_files = object_or_empty(report.get("audio_files"))
    artifact = audio_files.get("full_performance") or audio_files.get("source_window") or str(path)
    if isinstance(artifact, str) and not Path(artifact).is_absolute():
        artifact = str(path.parent / artifact)
    return {
        "source_report": str(path),
        "source_schema": string_or(report.get("schema"), "unknown"),
        "source_family": infer_source_family(report, path),
        "source_result": string_or(report.get("result"), "unknown"),
        "source_verdict": string_or(report.get("agent_verdict"), "unknown"),
        "human_verdict": string_or(report.get("human_verdict"), "unverified"),
        "artifact_to_hear": artifact,
        "strongest_audible_element": string_or(report.get("strongest_element"), "unknown"),
        "failure_codes": string_list(report.get("failure_codes")),
        "reason_tags": object_or_empty(report.get("reason_tags")),
        "avoid": string_list(report.get("avoid")),
        "source_backed": bool(report.get("source_backed", True)),
        "source_timing_backed": bool(report.get("source_timing_backed", True)),
        "scripted_generation": bool(report.get("scripted_generation", True)),
    }


def source_from_human_label(path: Path, label_id: str) -> dict[str, Any]:
    corpus = read_json_object(path)
    labels = corpus.get("labels")
    require(isinstance(labels, list), f"{path}: labels must be array")
    label = next(
        (item for item in labels if isinstance(item, dict) and item.get("label_id") == label_id),
        None,
    )
    require(label is not None, f"{path}: missing label_id {label_id}")
    artifact_paths = object_or_empty(label.get("artifact_paths"))
    artifact_audio = object_or_empty(artifact_paths.get("audio"))
    artifact_identity = object_or_empty(label.get("artifact_identity"))
    identity_audio = object_or_empty(artifact_identity.get("audio_sha256"))
    artifact = (
        artifact_audio.get("full_performance")
        or artifact_paths.get("agent_review")
        or identity_audio.get("full_performance")
        or label.get("review_pack_id")
        or str(path)
    )
    return {
        "source_report": str(path),
        "source_schema": string_or(corpus.get("schema"), "unknown"),
        "source_family": string_or(label.get("source_family"), "unknown"),
        "source_result": string_or(label.get("human_verdict"), "unknown"),
        "source_verdict": string_or(label.get("human_verdict"), "unknown"),
        "human_verdict": string_or(label.get("human_verdict"), "unverified"),
        "artifact_to_hear": str(artifact),
        "strongest_audible_element": string_or(
            object_or_empty(label.get("reason_tags")).get("hardest_hit"),
            "unknown",
        ),
        "failure_codes": [],
        "reason_tags": object_or_empty(label.get("reason_tags")),
        "avoid": string_list(label.get("avoid")),
        "source_backed": True,
        "source_timing_backed": True,
        "scripted_generation": True,
    }


def source_from_report(
    report: dict[str, Any],
    path: Path,
    *,
    source_family: str | None = None,
    artifact_to_hear: str,
) -> dict[str, Any]:
    selected = object_or_empty(report.get("selected_candidate"))
    return {
        "source_report": str(path),
        "source_schema": string_or(report.get("schema"), "unknown"),
        "source_family": source_family or string_or(report.get("source_family"), selected.get("case_id", "unknown")),
        "source_result": string_or(report.get("result"), "unknown"),
        "source_verdict": string_or(report.get("agent_verdict"), report.get("result", "unknown")),
        "human_verdict": string_or(report.get("human_verdict"), "unverified"),
        "artifact_to_hear": artifact_to_hear,
        "strongest_audible_element": strongest_from_failures(string_list(report.get("failure_codes"))),
        "failure_codes": string_list(report.get("failure_codes")),
        "reason_tags": object_or_empty(report.get("reason_tags")),
        "avoid": string_list(report.get("avoid")),
        "source_backed": bool(report.get("source_backed", True)),
        "source_timing_backed": bool(report.get("source_timing_backed", True)),
        "scripted_generation": bool(report.get("scripted_generation", True)),
    }


def source_from_dense_performance_report(path: Path) -> dict[str, Any]:
    report = read_json_object(path)
    files = object_or_empty(report.get("files"))
    artifact = files.get("rebuild_only_performance") or files.get("full_performance") or str(path)
    if isinstance(artifact, str) and not Path(artifact).is_absolute():
        artifact = str(path.parent / artifact)
    source_policy = object_or_empty(report.get("source_policy"))
    pressure_policy = object_or_empty(source_policy.get("pressure_lift_policy"))
    proof = object_or_empty(report.get("proof"))
    return {
        "source_report": str(path),
        "source_schema": string_or(report.get("schema"), "unknown"),
        "source_family": string_or(pressure_policy.get("source_family"), infer_source_family(report, path)),
        "source_result": string_or(report.get("result"), "unknown"),
        "source_verdict": string_or(report.get("agent_verdict"), report.get("result", "unknown")),
        "human_verdict": string_or(report.get("human_verdict"), "unverified"),
        "artifact_to_hear": artifact,
        "strongest_audible_element": string_or(
            proof.get("strongest_audible_element"),
            strongest_from_failures(string_list(report.get("failure_codes"))),
        ),
        "failure_codes": string_list(report.get("failure_codes")),
        "reason_tags": object_or_empty(report.get("reason_tags")),
        "avoid": string_list(report.get("avoid")),
        "source_backed": bool(report.get("source_backed", True)),
        "source_timing_backed": bool(report.get("source_timing_backed", True)),
        "scripted_generation": bool(report.get("scripted_generation", True)),
    }


def route_signals(
    signals: list[str],
    reason_tags: dict[str, Any],
    avoid: list[str],
) -> dict[str, Any]:
    scores = {category: 0 for category in CATEGORIES}
    reasons: dict[str, list[str]] = {category: [] for category in CATEGORIES}
    for signal in signals:
        normalized = normalize(signal)
        for token, category, weight, reason in CODE_RULES:
            if token in normalized:
                scores[category] += weight
                reasons[category].append(f"{signal}: {reason}")
    for key, value in reason_tags.items():
        key_text = normalize(str(key))
        value_text = normalize(str(value))
        for tag_key, tag_value, category, weight, reason in TAG_RULES:
            if key_text == tag_key and value_text == tag_value:
                scores[category] += weight
                reasons[category].append(f"{key}={value}: {reason}")
    for item in avoid:
        item_text = normalize(item)
        for token, category, weight, reason in AVOID_RULES:
            if normalize(token) in item_text:
                scores[category] += weight
                reasons[category].append(f"avoid={item}: {reason}")

    ranked = sorted(
        (category for category, score in scores.items() if score > 0),
        key=lambda category: (-scores[category], CATEGORY_ORDER.index(category)),
    )
    matched_known_signal = bool(ranked)
    if not ranked:
        ranked = ["fixture_threshold"]
        reasons["fixture_threshold"].append("No known weak-output signal matched; add a routing rule.")
    primary = ranked[0]
    return {
        "proposed_next_fix_category": primary,
        "proposed_fix_categories": ranked,
        "main_weakness": weakness_label(primary),
        "musician_fix_reason": musician_fix_reason(primary),
        "routing_reasons": {category: reasons[category] for category in ranked},
        "matched_known_routing_signal": matched_known_signal,
    }


def reason_tag_signals(reason_tags: dict[str, Any]) -> list[str]:
    return [f"{key}_{value}" for key, value in reason_tags.items()]


def strongest_from_failures(failure_codes: list[str]) -> str:
    joined = " ".join(failure_codes)
    if "bass" in joined or "low_end" in joined or "low_band" in joined:
        return "bass_pressure"
    if "hook" in joined or "w30" in joined:
        return "chop"
    if "dropout" in joined or "stutter" in joined or "restore" in joined:
        return "destructive_gesture"
    if "source" in joined:
        return "source_character"
    return "partial_audio_evidence"


def weakness_label(category: str) -> str:
    labels = {
        "source_selection": "source choice or source recognition is not strong enough",
        "chop_policy": "hook/chop is not memorable or transformed enough",
        "drum_pressure": "drum pressure does not hit hard enough",
        "bass_movement": "bass pressure and low-end movement are too weak",
        "mix_bus": "mix balance hides impact or source character",
        "destructive_gesture": "dropout/stutter/restore contrast does not change the room",
        "fixture_threshold": "QA threshold or fixture classification needs tightening",
        "ui_cue": "timing/source confidence needs a clearer user cue",
    }
    return labels[category]


def musician_fix_reason(category: str) -> str:
    labels = {
        "source_selection": "Pick or expose a source window whose identity survives the rebuild-only path.",
        "chop_policy": "Change the chop policy so the hook becomes memorable instead of generic support.",
        "drum_pressure": "Push the drum treatment until the kick/snare pressure carries the gesture.",
        "bass_movement": "Rework bass movement so low-end pressure hits instead of sitting behind the source.",
        "mix_bus": "Rebalance or drive the mix so impact and source character are not hidden.",
        "destructive_gesture": "Strengthen the cut, stutter, or restore so the live gesture changes the room.",
        "fixture_threshold": "Tighten the fixture or threshold before trusting this weak-output class.",
        "ui_cue": "Show the timing/source risk so the player does not trigger a confident move blindly.",
    }
    return labels[category]


def production_software_next_step(category: str) -> str:
    labels = {
        "source_selection": "Review source-window and source-character policy before promoting more output.",
        "chop_policy": "Tune W-30 hook/chop selection and transformation thresholds for the routed cases.",
        "drum_pressure": "Adjust TR-909 pressure treatment and verify it survives the rendered output path.",
        "bass_movement": "Refine low-band movement policy and MC-202 pressure checks for the routed cases.",
        "mix_bus": "Change mix-bus balance/drive so generated support does not mask the source or impact.",
        "destructive_gesture": "Strengthen dropout, stutter, restore, or cut policy and rerun destructive fixtures.",
        "fixture_threshold": "Add or tighten the fixture threshold before treating this failure class as routed.",
        "ui_cue": "Expose timing/source confidence risk before confident bar-locked or live-trigger moves.",
    }
    return labels[category]


def production_musician_payoff(category: str) -> str:
    labels = {
        "source_selection": "The musician hears transformed source character instead of fallback or generic rebuild.",
        "chop_policy": "The first two bars gain a hook or riff worth triggering again.",
        "drum_pressure": "Kick, snare, or break pressure becomes physical instead of decorative.",
        "bass_movement": "Low-end pressure starts carrying the room instead of reading as a midrange phrase.",
        "mix_bus": "The strongest element becomes clear without burying the source.",
        "destructive_gesture": "Cuts, stutters, and restores become stage-meaningful gestures.",
        "fixture_threshold": "Weak audio stops slipping through as acceptable diagnostic evidence.",
        "ui_cue": "The player can see when timing or source risk makes a move unsafe.",
    }
    return labels[category]


def infer_source_family(report: dict[str, Any], path: Path) -> str:
    explicit = report.get("source_family")
    if isinstance(explicit, str) and explicit:
        return explicit
    text = str(path)
    if "sparse-bass" in text or "sparse_bass" in text:
        return "sparse_bass_pressure"
    if "tonal" in text:
        return "tonal_hook"
    return "dense_break"


def write_reports(output: Path, report: dict[str, Any]) -> None:
    (output / "weak-output-fix-routing.json").write_text(json.dumps(report, indent=2) + "\n")
    (output / "weak-output-fix-routing.md").write_text(render_markdown(report))


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# Weak-Output Fix Routing",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Evidence role: `{report['evidence_role']}`",
        f"- Quality proof: `{str(report['quality_proof']).lower()}`",
        f"- Automated musical approval: `{str(report['automated_musical_approval']).lower()}`",
        f"- Routed cases: `{report['routed_case_count']}/{report['case_count']}`",
        f"- Production fix candidates: `{report['production_fix_candidate_count']}`",
        "",
        "## Production Fix Candidates",
        "",
    ]
    for candidate in report["production_fix_candidates"]:
        lines.extend(
            [
                f"### `{candidate['candidate_id']}`",
                "",
                f"- Category: `{candidate['category']}`",
                f"- Score: `{candidate['score']}`",
                f"- Cases: `{', '.join(candidate['case_ids'])}`",
                f"- Source families: `{', '.join(candidate['source_families'])}`",
                f"- Software next step: {candidate['software_next_step']}",
                f"- Musician payoff: {candidate['musician_payoff']}",
                "",
            ]
        )
    lines.extend(
        [
            "## Cases",
            "",
        ]
    )
    for case in report["cases"]:
        lines.extend(
            [
                f"### `{case['case_id']}`",
                "",
                f"- Artifact to hear: `{case['artifact_to_hear']}`",
                f"- Strongest audible element: `{case['strongest_audible_element']}`",
                f"- Main weakness: {case['main_weakness']}",
                f"- Proposed next fix category: `{case['proposed_next_fix_category']}`",
                f"- Musician fix reason: {case['musician_fix_reason']}",
                f"- All fix categories: `{', '.join(case['proposed_fix_categories'])}`",
                "",
            ]
        )
    lines.extend(["## Boundary", "", report["boundary"], ""])
    return "\n".join(lines)


def resolve_path(fixture_dir: Path, path: Path) -> Path:
    if path.is_absolute():
        return path
    if path.exists():
        return path
    return fixture_dir / path


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value if isinstance(item, str) and item]


def string_or(value: Any, default: Any) -> str:
    if isinstance(value, str) and value:
        return value
    if isinstance(default, str) and default:
        return default
    return str(default)


def required_string(data: dict[str, Any], field: str) -> str:
    value = data.get(field)
    require(isinstance(value, str) and value, f"missing {field}")
    return value


def normalize(value: str) -> str:
    return value.strip().lower().replace("-", "_").replace(" ", "_")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
