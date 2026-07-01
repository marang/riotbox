#!/usr/bin/env python3
"""Validate the generated professional-output suite contract used by Just gates."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.professional_output_suite.v1"
EXPECTED_CHILDREN = [
    "dense_break",
    "destructive_variation",
    "edge_source_professional_diagnostics",
    "non_dense_professional_proof_pack",
    "pro_pressure_source_matrix",
    "professional_output_listening_pack",
    "professional_source_wav_pack",
    "rendered_weak_professional_outputs",
]
EXPECTED_LISTENING_FAMILIES = [
    "dense_break",
    "sparse_bass_pressure",
    "tonal_hook",
]
EXPECTED_EDGE_FAMILIES = ["bad_timing", "pad_noise"]
AUDIBLE_ELEMENTS = {"kick", "snare", "bass", "stab", "silence", "restore"}
MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO = 0.22
MIN_HOOK_FORWARD_W30_TO_SOURCE_MARGIN = 0.10
MIN_TONAL_HOOK_RESTRAINT_PRESSURE_LIFT_RATIO = 2.20
MIN_TONAL_MIX_BUS_MC202_TO_W30_RMS_RATIO = 0.20
MIN_HOOK_CHOP_RIFF_HIT_COUNT = 6.0
MIN_HOOK_CHOP_RIFF_VELOCITY_SPAN = 0.20
MIN_HOOK_CHOP_RIFF_REVERSE_COUNT = 1.0
MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ = 1.75
MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ = 15.00
MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO = 1.70
MIN_SPARSE_PRESSURE_LOW_BAND_SHARE = 0.32
MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO = 2.10
MIN_SPARSE_BASS_DOMINANCE_MARGIN = 0.12
MIN_DENSE_ANSWER_SCRIPTED_ROLE_DISTANCE = 3.0
MIN_DENSE_ANSWER_STAB_SCORE = 1.65
MIN_DENSE_ANSWER_STAB_MARGIN = 0.15
MIN_DENSE_ANSWER_PRESSURE_SNAP_RATIO = 1.06
MIN_DENSE_ANSWER_BITE_SCORE = 1.0
MIN_DENSE_DESTRUCTIVE_PRESSURE_LIFT_RATIO = 1.10
MAX_DESTRUCTIVE_DROPOUT_TO_STUTTER_RMS_RATIO = 0.10
MAX_DESTRUCTIVE_DROPOUT_SILENCE_TO_STUTTER_RMS_RATIO = 0.08
MIN_DESTRUCTIVE_STUTTER_TO_HOOK_TRANSIENT_RATIO = 1.20
MIN_DESTRUCTIVE_RESTORE_TO_PRESSURE_RMS_RATIO = 1.22
MIN_DESTRUCTIVE_RESTORE_TO_DROPOUT_SILENCE_RMS_RATIO = 6.00
MIN_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO = 0.145
MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO = 0.08
MIN_FERAL_SOURCE_FIRST_MASKING_HEADROOM = 0.04
MAX_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO = 0.46
MIN_FERAL_TR909_RENDERED_SUPPORT_CONTRIBUTION_RATIO = 0.050
MIN_FERAL_TR909_RENDERED_LOW_BAND_RMS = 0.0030
MIN_SOURCE_CHARACTER_WINDOW_RMS_RETENTION_RATIO = 0.98
MIN_SOURCE_CHARACTER_WINDOW_SEARCHED_CASE_COUNT = 3
MIN_SOURCE_CHARACTER_WINDOW_PROMOTED_CASE_COUNT = 1
MIN_REBUILD_ONLY_SOURCE_CHARACTER_SURVIVAL_MARGIN = 0.10


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("report", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--mutation-fixtures",
        action="store_true",
        help="also run built-in negative mutation fixtures against the contract",
    )
    args = parser.parse_args()

    try:
        report = read_json_object(args.report)
        output = args.output or args.report.parent
        failures = validate_report(report, output)
        if args.mutation_fixtures:
            failures.extend(validate_mutation_fixtures(report, output))
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid professional output suite contract: {error}", file=sys.stderr)
        return 1

    if failures:
        print(
            "invalid professional output suite contract: " + ", ".join(failures),
            file=sys.stderr,
        )
        return 1
    print(f"valid professional output suite contract: {args.report}")
    return 0


def validate_report(report: dict[str, Any], output: Path) -> list[str]:
    failures: list[str] = []
    require(report.get("schema") == SCHEMA, "schema_mismatch", failures)
    require(report.get("result") == "pass", "suite_result_not_pass", failures)
    require(
        report.get("agent_verdict") == "agent_promising",
        "suite_agent_verdict_not_promising",
        failures,
    )
    require(report.get("human_verdict") == "unverified", "suite_human_verdict_changed", failures)
    require(report.get("evidence_role") == "suite_diagnostic", "suite_evidence_role_changed", failures)
    require(report.get("scripted_generation") is True, "suite_not_scripted", failures)
    require(report.get("quality_proof") is False, "suite_claims_quality_proof", failures)
    require(int(number(report.get("child_report_count"))) == 8, "child_report_count_mismatch", failures)
    require(
        int(number(report.get("passed_child_report_count"))) == 8,
        "passed_child_report_count_mismatch",
        failures,
    )
    require(
        int(number(report.get("failed_child_report_count"))) == 0,
        "failed_child_report_count_mismatch",
        failures,
    )

    children = list_or_empty(report.get("children"))
    require(len(children) == 8, "child_list_length_mismatch", failures)
    child_ids = sorted(str(child.get("id", "")) for child in children)
    require(child_ids == EXPECTED_CHILDREN, "child_id_coverage_mismatch", failures)
    for child in children:
        validate_child_boundary(child, failures)

    dense = child_metrics(children, "dense_break")
    matrix = child_metrics(children, "pro_pressure_source_matrix")
    source_wav = child_metrics(children, "professional_source_wav_pack")
    destructive = child_metrics(children, "destructive_variation")
    listening = child_metrics(children, "professional_output_listening_pack")
    edge = child_metrics(children, "edge_source_professional_diagnostics")

    validate_listening_metrics(listening, report, failures)
    validate_edge_metrics(edge, failures)
    validate_hook_chop_metrics(dense, matrix, source_wav, failures)
    validate_destructive_metrics(dense, matrix, source_wav, destructive, failures)
    validate_sparse_bass_metrics(matrix, source_wav, failures)
    validate_arrangement_metrics(dense, matrix, source_wav, failures)
    validate_mix_metrics(dense, matrix, source_wav, failures)
    validate_tail_metrics(dense, matrix, source_wav, failures)
    validate_strongest_element_metrics(dense, matrix, source_wav, edge, failures)
    validate_source_character_metrics(dense, matrix, source_wav, edge, failures)
    validate_rebuild_balance_metrics(matrix, source_wav, failures)
    validate_feral_mix_balance_metrics(report, failures)
    validate_source_character_window_selection_metrics(report, failures)
    validate_tr909_rendered_drum_pressure_metrics(report, failures)
    validate_artifacts(output, failures)
    return failures


def validate_mutation_fixtures(report: dict[str, Any], output: Path) -> list[str]:
    fixtures = [
        (
            "non_source_derived_hook_chop",
            lambda data: set_child_metric(
                data,
                "dense_break",
                "hook_chop_selection_source_derived",
                0.0,
            ),
            "dense_hook_chop_selection_not_source_derived",
        ),
        (
            "weak_hook_chop_w30",
            lambda data: set_child_metric(
                data,
                "dense_break",
                "w30_to_source_rms_ratio",
                0.0,
            ),
            "dense_hook_chop_w30_too_weak",
        ),
        (
            "weak_hook_chop_w30_margin",
            lambda data: set_child_metric(
                data,
                "dense_break",
                "hook_chop_w30_to_source_margin",
                0.0,
            ),
            "dense_hook_chop_w30_margin_too_low",
        ),
        (
            "fixed_hook_chop_riff_pattern",
            lambda data: set_child_metric(
                data,
                "dense_break",
                "hook_chop_riff_hit_pattern_source_derived",
                0.0,
            ),
            "dense_hook_chop_riff_pattern_not_source_derived",
        ),
        (
            "weak_sparse_bass_dominance",
            lambda data: set_child_metric(
                data,
                "professional_source_wav_pack",
                "sparse_bass_dominance_margin",
                0.0,
            ),
            "source_wav_sparse_bass_dominance_margin_too_low",
        ),
        (
            "weak_tonal_mc202_mix_bus",
            lambda data: set_child_metric(
                data,
                "professional_source_wav_pack",
                "tonal_mix_bus_mc202_to_w30_rms_ratio",
                0.0,
            ),
            "source_wav_tonal_mix_bus_mc202_too_buried",
        ),
        (
            "weak_sparse_low_band_share",
            lambda data: set_child_metric(
                data,
                "professional_source_wav_pack",
                "sparse_pressure_low_band_share",
                0.0,
            ),
            "source_wav_sparse_pressure_low_band_share_too_low",
        ),
        (
            "sparse_midrange_phrase",
            lambda data: set_child_metric(
                data,
                "professional_source_wav_pack",
                "sparse_pressure_low_to_mid_ratio",
                0.0,
            ),
            "source_wav_sparse_pressure_reads_as_midrange_phrase",
        ),
        (
            "weak_destructive_stutter",
            lambda data: set_child_metric(
                data,
                "destructive_variation",
                "stutter_to_hook_transient_ratio",
                0.0,
            ),
            "destructive_stutter_lacks_transient_impact",
        ),
        (
            "weak_destructive_cut_depth",
            lambda data: set_child_metric(
                data,
                "destructive_variation",
                "dropout_silence_to_stutter_rms_ratio",
                1.0,
            ),
            "destructive_dropout_silence_not_deep_enough_before_stutter",
        ),
        (
            "weak_destructive_restore_from_cut",
            lambda data: set_child_metric(
                data,
                "destructive_variation",
                "restore_to_dropout_silence_rms_ratio",
                1.0,
            ),
            "destructive_restore_does_not_slam_out_of_cut",
        ),
        (
            "generated_support_masks_source",
            lambda data: set_nested_value(
                data,
                ["feral_mix_balance", "max_support_generated_to_source_rms_ratio"],
                0.99,
            ),
            "feral_generated_support_masks_source",
        ),
        (
            "source_first_masking_headroom_low",
            lambda data: set_nested_value(
                data,
                ["feral_mix_balance", "min_source_first_masking_headroom"],
                0.0,
            ),
            "feral_source_first_masking_headroom_too_low",
        ),
        (
            "source_character_survival_margin_low",
            lambda data: set_child_metric(
                data,
                "dense_break",
                "rebuild_only_source_character_survival_margin",
                0.0,
            ),
            "dense_source_character_margin_too_low",
        ),
        (
            "soft_dense_drum_transient",
            lambda data: set_child_metric(
                data,
                "dense_break",
                "dense_break_pressure_transient_to_hook_ratio",
                0.0,
            ),
            "dense_pressure_transient_too_soft",
        ),
        (
            "scripted_demo_ready",
            lambda data: set_first_listening_case_field(
                data,
                "demo_readiness",
                "demo_ready",
            ),
            "demo_readiness_not_unverified",
        ),
        (
            "missing_non_dense_mc202_gate",
            lambda data: set_child_metric(
                data,
                "professional_output_listening_pack",
                "mc202_non_dense_break_case_count",
                0,
            ),
            "listening_mc202_non_dense_candidate_missing",
        ),
        (
            "template_only_mc202_case",
            mark_first_listening_case_template_only_unblocked,
            "mc202_template_only_not_blocked",
        ),
        (
            "fallback_selection_strategy_child",
            lambda data: set_first_child_field(
                data,
                "fallback_selection_strategy_count",
                1,
            ),
            "dense_break_fallback_selection_strategy_present",
        ),
    ]

    failures: list[str] = []
    for name, mutate, expected_code in fixtures:
        mutated = copy.deepcopy(report)
        if not mutate(mutated):
            failures.append(f"{name}_mutation_unavailable")
            continue
        mutation_failures = validate_report(mutated, output)
        if not any(expected_code in failure for failure in mutation_failures):
            failures.append(f"{name}_mutation_missing_{expected_code}")
    return failures


def validate_child_boundary(child: dict[str, Any], failures: list[str]) -> None:
    child_id = str(child.get("id", "unknown"))
    require(child.get("result") == "pass", f"{child_id}_result_not_pass", failures)
    require(
        child.get("agent_verdict") == "agent_promising",
        f"{child_id}_agent_verdict_not_promising",
        failures,
    )
    require(child.get("human_verdict") == "unverified", f"{child_id}_human_verdict_changed", failures)
    require(child.get("scripted_generation") is True, f"{child_id}_not_scripted", failures)
    require(child.get("quality_proof") is False, f"{child_id}_claims_quality_proof", failures)
    require(
        int(number(child.get("fallback_selection_strategy_count"))) == 0,
        f"{child_id}_fallback_selection_strategy_present",
        failures,
    )
    require(len(str(child.get("report_sha256", ""))) == 64, f"{child_id}_report_hash_missing", failures)


def validate_listening_metrics(
    metrics: dict[str, Any], report: dict[str, Any], failures: list[str]
) -> None:
    require(
        int(number(metrics.get("demo_reason_count"))) == 3,
        "listening_demo_reason_count_mismatch",
        failures,
    )
    require(
        int(number(metrics.get("not_demo_ready_reason_count"))) == 3,
        "listening_not_demo_ready_reason_count_mismatch",
        failures,
    )
    require(
        metrics.get("mc202_source_composed_gate_result") == "pass",
        "listening_mc202_gate_not_pass",
        failures,
    )
    require(
        int(number(metrics.get("mc202_dense_break_case_count"))) >= 1,
        "listening_mc202_dense_candidate_missing",
        failures,
    )
    require(
        int(number(metrics.get("mc202_non_dense_break_case_count"))) >= 1,
        "listening_mc202_non_dense_candidate_missing",
        failures,
    )
    require(
        metrics.get("mc202_quality_proof") is False,
        "listening_mc202_gate_claims_quality_proof",
        failures,
    )
    identity = object_or_empty(report.get("listening_identity"))
    require(identity.get("result") == "pass", "listening_identity_result_not_pass", failures)
    require(int(number(identity.get("case_count"))) == 3, "listening_identity_case_count_mismatch", failures)
    require(
        sorted(str(item) for item in list_or_empty(identity.get("source_families")))
        == EXPECTED_LISTENING_FAMILIES,
        "listening_identity_family_coverage_mismatch",
        failures,
    )
    for case in list_or_empty(identity.get("cases")):
        case_id = str(case.get("case_id", "unknown"))
        require(
            case.get("demo_readiness") == "unverified",
            f"{case_id}_demo_readiness_not_unverified",
            failures,
        )
        require(
            str(case.get("demo_worthy_reason", "")).startswith("Worth review:"),
            f"{case_id}_demo_worthy_reason_missing",
            failures,
        )
        require(
            str(case.get("not_demo_worthy_reason", "")).startswith("Not demo-ready yet:"),
            f"{case_id}_not_demo_worthy_reason_missing",
            failures,
        )
        gate = object_or_empty(case.get("mc202_source_composed_review_gate"))
        source_composed = gate.get("source_composed_evidence") is True
        template_only = gate.get("primitive_or_template_only") is True
        template_blocked = (
            gate.get("promotion_blocked_until_human_pass") is True
            and gate.get("template_only_blocks_promotion") is True
        )
        require(
            source_composed or (template_only and template_blocked),
            f"{case_id}_mc202_source_composed_evidence_missing",
            failures,
        )
        require(
            not template_only or template_blocked,
            f"{case_id}_mc202_template_only_not_blocked",
            failures,
        )


def validate_edge_metrics(metrics: dict[str, Any], failures: list[str]) -> None:
    require(int(number(metrics.get("case_count"))) == 2, "edge_case_count_mismatch", failures)
    require(
        int(number(metrics.get("weak_routed_case_count"))) == 2,
        "edge_weak_routed_case_count_mismatch",
        failures,
    )
    require(
        sorted(str(item) for item in list_or_empty(metrics.get("source_families")))
        == EXPECTED_EDGE_FAMILIES,
        "edge_source_family_coverage_mismatch",
        failures,
    )
    pressure_families = list_or_empty(metrics.get("pressure_policy_families"))
    require("pad_noise" in pressure_families, "edge_pad_noise_policy_family_missing", failures)
    require("bad_timing" in pressure_families, "edge_bad_timing_policy_family_missing", failures)
    require(
        int(number(metrics.get("pad_noise_texture_source_derived_count"))) == 1,
        "edge_pad_noise_texture_not_source_derived",
        failures,
    )
    require(
        number(metrics.get("min_pad_noise_texture_candidate_count")) >= 3.0,
        "edge_pad_noise_texture_candidate_count_too_low",
        failures,
    )
    require(
        number(metrics.get("min_pad_noise_texture_gate_static_distance_frames")) >= 256.0,
        "edge_pad_noise_gate_collapsed_to_static",
        failures,
    )
    require(
        number(metrics.get("min_pad_noise_texture_stab_static_distance_frames")) >= 256.0,
        "edge_pad_noise_stab_collapsed_to_static",
        failures,
    )
    require(
        number(metrics.get("min_pad_noise_texture_gate_stab_distance_frames")) >= 512.0,
        "edge_pad_noise_gate_stab_distance_too_low",
        failures,
    )
    require(
        number(metrics.get("min_pad_noise_texture_transient_ratio")) >= 0.72,
        "edge_pad_noise_transient_ratio_too_low",
        failures,
    )
    signals = list_or_empty(metrics.get("weak_output_signals"))
    require("pad_noise_policy_path" in signals, "edge_pad_noise_signal_missing", failures)
    require("bad_timing_cautious_arrangement_path" in signals, "edge_bad_timing_signal_missing", failures)
    categories = list_or_empty(metrics.get("proposed_fix_categories"))
    require("source_selection" in categories, "edge_source_selection_fix_missing", failures)
    require("ui_cue" in categories, "edge_ui_cue_fix_missing", failures)
    require(
        int(number(metrics.get("source_selection_promotion_blocked_case_count"))) == 2,
        "edge_source_selection_blocked_count_mismatch",
        failures,
    )
    require(
        metrics.get("source_selection_promotion_allowed") is False,
        "edge_source_selection_promotion_allowed",
        failures,
    )
    require(
        sorted(str(item) for item in list_or_empty(metrics.get("source_selection_blocked_source_families")))
        == EXPECTED_EDGE_FAMILIES,
        "edge_source_selection_blocked_family_mismatch",
        failures,
    )
    promotion_blockers = list_or_empty(metrics.get("source_selection_promotion_blockers"))
    for blocker in (
        "human_verdict_unverified",
        "diagnostic_only_quality_proof_false",
        "source_selection_fix_required",
    ):
        require(
            blocker in promotion_blockers,
            f"edge_source_selection_{blocker}_missing",
            failures,
        )


def validate_hook_chop_metrics(
    dense: dict[str, Any],
    matrix: dict[str, Any],
    source_wav: dict[str, Any],
    failures: list[str],
) -> None:
    require(
        number(dense.get("w30_to_source_rms_ratio"))
        >= MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO,
        "dense_hook_chop_w30_too_weak",
        failures,
    )
    require(
        number(dense.get("hook_chop_w30_to_source_margin"))
        >= MIN_HOOK_FORWARD_W30_TO_SOURCE_MARGIN,
        "dense_hook_chop_w30_margin_too_low",
        failures,
    )
    require(
        number(dense.get("hook_chop_selection_source_derived")) == 1.0,
        "dense_hook_chop_selection_not_source_derived",
        failures,
    )
    require(
        number(dense.get("hook_chop_static_distance_frames")) >= 256.0,
        "dense_hook_chop_collapsed_to_static",
        failures,
    )
    require(
        number(dense.get("hook_chop_offset_distance_frames")) >= 512.0,
        "dense_hook_chop_offset_distance_too_low",
        failures,
    )
    require(
        number(dense.get("hook_chop_riff_unique_source_offset_count")) >= 3.0,
        "dense_hook_chop_riff_offsets_too_narrow",
        failures,
    )
    require(
        number(dense.get("hook_chop_riff_hit_pattern_source_derived")) == 1.0,
        "dense_hook_chop_riff_pattern_not_source_derived",
        failures,
    )
    require(
        number(dense.get("hook_chop_riff_hit_count"))
        >= MIN_HOOK_CHOP_RIFF_HIT_COUNT,
        "dense_hook_chop_riff_pattern_too_sparse",
        failures,
    )
    require(
        number(dense.get("hook_chop_riff_velocity_span"))
        >= MIN_HOOK_CHOP_RIFF_VELOCITY_SPAN,
        "dense_hook_chop_riff_velocity_too_flat",
        failures,
    )
    require(
        number(dense.get("hook_chop_riff_reverse_count"))
        >= MIN_HOOK_CHOP_RIFF_REVERSE_COUNT,
        "dense_hook_chop_riff_reverse_missing",
        failures,
    )
    require(
        number(dense.get("hook_chop_source_character_score_floor")) >= 0.60,
        "dense_hook_chop_source_character_too_weak",
        failures,
    )
    require(
        number(dense.get("hook_chop_source_character_score_span")) >= 0.10,
        "dense_hook_chop_source_character_too_narrow",
        failures,
    )
    require(
        number(matrix.get("min_dense_w30_to_source_rms_ratio"))
        >= MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO,
        "matrix_dense_hook_chop_w30_too_weak",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_w30_to_source_margin"))
        >= MIN_HOOK_FORWARD_W30_TO_SOURCE_MARGIN,
        "matrix_dense_hook_chop_w30_margin_too_low",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_static_distance_frames")) >= 256.0,
        "matrix_dense_hook_chop_collapsed_to_static",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_offset_distance_frames")) >= 512.0,
        "matrix_dense_hook_chop_offset_distance_too_low",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_riff_unique_source_offset_count")) >= 3.0,
        "matrix_dense_hook_chop_riff_offsets_too_narrow",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_riff_hit_pattern_source_derived"))
        == 1.0,
        "matrix_dense_hook_chop_riff_pattern_not_source_derived",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_riff_hit_count"))
        >= MIN_HOOK_CHOP_RIFF_HIT_COUNT,
        "matrix_dense_hook_chop_riff_pattern_too_sparse",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_riff_velocity_span"))
        >= MIN_HOOK_CHOP_RIFF_VELOCITY_SPAN,
        "matrix_dense_hook_chop_riff_velocity_too_flat",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_riff_reverse_count"))
        >= MIN_HOOK_CHOP_RIFF_REVERSE_COUNT,
        "matrix_dense_hook_chop_riff_reverse_missing",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_source_character_score_floor")) >= 0.60,
        "matrix_dense_hook_chop_source_character_too_weak",
        failures,
    )
    require(
        number(matrix.get("min_dense_hook_chop_source_character_score_span")) >= 0.10,
        "matrix_dense_hook_chop_source_character_too_narrow",
        failures,
    )
    require(
        number(source_wav.get("tonal_w30_to_source_rms_ratio"))
        >= MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO,
        "source_wav_tonal_hook_chop_w30_too_weak",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_w30_to_source_margin"))
        >= MIN_HOOK_FORWARD_W30_TO_SOURCE_MARGIN,
        "source_wav_tonal_hook_chop_w30_margin_too_low",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_restraint_pressure_lift_ratio"))
        >= MIN_TONAL_HOOK_RESTRAINT_PRESSURE_LIFT_RATIO,
        "source_wav_tonal_hook_restraint_pressure_lift_too_soft",
        failures,
    )
    require(
        number(source_wav.get("tonal_mix_bus_mc202_to_w30_rms_ratio"))
        >= MIN_TONAL_MIX_BUS_MC202_TO_W30_RMS_RATIO,
        "source_wav_tonal_mix_bus_mc202_too_buried",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_static_distance_frames")) >= 256.0,
        "source_wav_tonal_hook_chop_collapsed_to_static",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_offset_distance_frames")) >= 512.0,
        "source_wav_tonal_hook_chop_offset_distance_too_low",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_riff_unique_source_offset_count")) >= 3.0,
        "source_wav_tonal_hook_chop_riff_offsets_too_narrow",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_riff_hit_pattern_source_derived"))
        == 1.0,
        "source_wav_tonal_hook_chop_riff_pattern_not_source_derived",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_riff_hit_count"))
        >= MIN_HOOK_CHOP_RIFF_HIT_COUNT,
        "source_wav_tonal_hook_chop_riff_pattern_too_sparse",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_riff_velocity_span"))
        >= MIN_HOOK_CHOP_RIFF_VELOCITY_SPAN,
        "source_wav_tonal_hook_chop_riff_velocity_too_flat",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_riff_reverse_count"))
        >= MIN_HOOK_CHOP_RIFF_REVERSE_COUNT,
        "source_wav_tonal_hook_chop_riff_reverse_missing",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_source_character_score_floor")) >= 0.60,
        "source_wav_tonal_hook_chop_source_character_too_weak",
        failures,
    )
    require(
        number(source_wav.get("tonal_hook_chop_source_character_score_span")) >= 0.10,
        "source_wav_tonal_hook_chop_source_character_too_narrow",
        failures,
    )


def validate_destructive_metrics(
    dense: dict[str, Any],
    matrix: dict[str, Any],
    source_wav: dict[str, Any],
    destructive: dict[str, Any],
    failures: list[str],
) -> None:
    require(
        number(destructive.get("dropout_to_stutter_rms_ratio"))
        <= MAX_DESTRUCTIVE_DROPOUT_TO_STUTTER_RMS_RATIO,
        "destructive_dropout_not_contrasting_with_stutter",
        failures,
    )
    require(
        "dropout_silence_to_stutter_rms_ratio" in destructive
        and
        number(destructive.get("dropout_silence_to_stutter_rms_ratio"))
        <= MAX_DESTRUCTIVE_DROPOUT_SILENCE_TO_STUTTER_RMS_RATIO,
        "destructive_dropout_silence_not_deep_enough_before_stutter",
        failures,
    )
    require(
        number(destructive.get("stutter_to_hook_transient_ratio"))
        >= MIN_DESTRUCTIVE_STUTTER_TO_HOOK_TRANSIENT_RATIO,
        "destructive_stutter_lacks_transient_impact",
        failures,
    )
    require(
        number(destructive.get("restore_to_pressure_rms_ratio"))
        >= MIN_DESTRUCTIVE_RESTORE_TO_PRESSURE_RMS_RATIO,
        "destructive_restore_not_bigger_than_pressure",
        failures,
    )
    require(
        "restore_to_dropout_silence_rms_ratio" in destructive
        and
        number(destructive.get("restore_to_dropout_silence_rms_ratio"))
        >= MIN_DESTRUCTIVE_RESTORE_TO_DROPOUT_SILENCE_RMS_RATIO,
        "destructive_restore_does_not_slam_out_of_cut",
        failures,
    )
    require(number(dense.get("destructive_gesture_source_derived")) == 1.0, "dense_destructive_not_source_derived", failures)
    require(number(dense.get("destructive_static_distance_frames")) >= 256.0, "dense_destructive_collapsed_to_static", failures)
    require(number(dense.get("destructive_offset_distance_frames")) >= 512.0, "dense_destructive_offset_distance_too_low", failures)
    require(number(dense.get("dense_destructive_pressure_lift_ratio")) >= MIN_DENSE_DESTRUCTIVE_PRESSURE_LIFT_RATIO, "dense_destructive_pressure_lift_too_soft", failures)
    require(number(matrix.get("min_dense_destructive_static_distance_frames")) >= 256.0, "matrix_dense_destructive_collapsed_to_static", failures)
    require(number(matrix.get("min_dense_destructive_offset_distance_frames")) >= 512.0, "matrix_dense_destructive_offset_distance_too_low", failures)
    require(number(source_wav.get("tonal_destructive_static_distance_frames")) >= 256.0, "source_wav_tonal_destructive_collapsed_to_static", failures)
    require(number(source_wav.get("tonal_destructive_offset_distance_frames")) >= 512.0, "source_wav_tonal_destructive_offset_distance_too_low", failures)


def validate_sparse_bass_metrics(
    matrix: dict[str, Any], source_wav: dict[str, Any], failures: list[str]
) -> None:
    require(
        number(matrix.get("min_sparse_bass_movement_static_distance_hz"))
        >= MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ,
        "matrix_sparse_bass_movement_collapsed_to_static",
        failures,
    )
    require(
        number(matrix.get("min_sparse_bass_movement_frequency_span_hz"))
        >= MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ,
        "matrix_sparse_bass_movement_span_too_low",
        failures,
    )
    require(
        number(matrix.get("min_sparse_pressure_low_band_lift_ratio"))
        >= MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO,
        "matrix_sparse_pressure_low_band_too_weak",
        failures,
    )
    require(
        number(matrix.get("min_sparse_pressure_low_band_share"))
        >= MIN_SPARSE_PRESSURE_LOW_BAND_SHARE,
        "matrix_sparse_pressure_low_band_share_too_low",
        failures,
    )
    require(
        number(matrix.get("min_sparse_pressure_low_to_mid_ratio"))
        >= MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO,
        "matrix_sparse_pressure_reads_as_midrange_phrase",
        failures,
    )
    require(
        number(matrix.get("min_sparse_bass_dominance_margin"))
        >= MIN_SPARSE_BASS_DOMINANCE_MARGIN,
        "matrix_sparse_bass_dominance_margin_too_low",
        failures,
    )
    require(
        number(source_wav.get("sparse_bass_movement_static_distance_hz"))
        >= MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ,
        "source_wav_sparse_bass_movement_collapsed_to_static",
        failures,
    )
    require(
        number(source_wav.get("sparse_bass_movement_frequency_span_hz"))
        >= MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ,
        "source_wav_sparse_bass_movement_span_too_low",
        failures,
    )
    require(
        number(source_wav.get("sparse_pressure_low_band_lift_ratio"))
        >= MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO,
        "source_wav_sparse_pressure_low_band_too_weak",
        failures,
    )
    require(
        number(source_wav.get("sparse_pressure_low_band_share"))
        >= MIN_SPARSE_PRESSURE_LOW_BAND_SHARE,
        "source_wav_sparse_pressure_low_band_share_too_low",
        failures,
    )
    require(
        number(source_wav.get("sparse_pressure_low_to_mid_ratio"))
        >= MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO,
        "source_wav_sparse_pressure_reads_as_midrange_phrase",
        failures,
    )
    require(
        number(source_wav.get("sparse_bass_dominance_margin"))
        >= MIN_SPARSE_BASS_DOMINANCE_MARGIN,
        "source_wav_sparse_bass_dominance_margin_too_low",
        failures,
    )


def validate_arrangement_metrics(
    dense: dict[str, Any],
    matrix: dict[str, Any],
    source_wav: dict[str, Any],
    failures: list[str],
) -> None:
    require(number(matrix.get("arrangement_unique_role_order_signature_count")) >= 2.0, "matrix_arrangement_role_order_not_diverse", failures)
    require(len(list_or_empty(matrix.get("arrangement_role_order_signatures"))) >= 2, "matrix_arrangement_signature_list_too_short", failures)
    require(number(dense.get("arrangement_role_order_source_derived")) == 1.0, "dense_arrangement_not_source_derived", failures)
    require(number(dense.get("arrangement_role_candidate_count")) >= 6.0, "dense_arrangement_candidate_count_too_low", failures)
    require(number(dense.get("arrangement_scripted_role_distance")) >= 1.0, "dense_arrangement_scripted_distance_too_low", failures)
    require(number(dense.get("dense_answer_bite_source_derived")) == 1.0, "dense_answer_bite_not_source_derived", failures)
    require(number(dense.get("dense_answer_bite_scripted_role_distance")) >= MIN_DENSE_ANSWER_SCRIPTED_ROLE_DISTANCE, "dense_answer_bite_scripted_distance_too_low", failures)
    require(number(dense.get("dense_answer_bite_stab_score")) >= MIN_DENSE_ANSWER_STAB_SCORE, "dense_answer_bite_stab_score_too_low", failures)
    require(number(dense.get("dense_answer_bite_stab_margin")) >= MIN_DENSE_ANSWER_STAB_MARGIN, "dense_answer_bite_stab_margin_too_low", failures)
    require(number(dense.get("dense_answer_bite_pressure_snap_ratio")) >= MIN_DENSE_ANSWER_PRESSURE_SNAP_RATIO, "dense_answer_bite_snap_too_soft", failures)
    require(number(dense.get("dense_answer_bite_score")) >= MIN_DENSE_ANSWER_BITE_SCORE, "dense_answer_bite_score_too_low", failures)
    require(number(matrix.get("min_source_derived_arrangement_role_candidate_count")) >= 6.0, "matrix_arrangement_candidate_count_too_low", failures)
    require(number(matrix.get("min_source_derived_arrangement_scripted_role_distance")) >= 1.0, "matrix_arrangement_scripted_distance_too_low", failures)
    require(number(source_wav.get("min_source_derived_arrangement_role_candidate_count")) >= 6.0, "source_wav_arrangement_candidate_count_too_low", failures)
    require(number(source_wav.get("min_source_derived_arrangement_scripted_role_distance")) >= 1.0, "source_wav_arrangement_scripted_distance_too_low", failures)


def validate_mix_metrics(
    dense: dict[str, Any],
    matrix: dict[str, Any],
    source_wav: dict[str, Any],
    failures: list[str],
) -> None:
    require(number(dense.get("mix_treatment_source_derived")) == 1.0, "dense_mix_treatment_not_source_derived", failures)
    require(number(dense.get("mix_treatment_fixed_distance")) >= 0.08, "dense_mix_treatment_fixed_distance_too_low", failures)
    require(number(dense.get("mix_treatment_output_contrast_ratio")) >= 2.10, "dense_mix_treatment_contrast_too_low", failures)
    require(number(matrix.get("min_source_derived_mix_treatment_fixed_distance")) >= 0.08, "matrix_mix_treatment_fixed_distance_too_low", failures)
    require(number(matrix.get("min_source_derived_mix_treatment_output_contrast_ratio")) >= 2.10, "matrix_mix_treatment_contrast_too_low", failures)
    require(number(source_wav.get("min_source_derived_mix_treatment_fixed_distance")) >= 0.08, "source_wav_mix_treatment_fixed_distance_too_low", failures)
    require(number(source_wav.get("min_source_derived_mix_treatment_output_contrast_ratio")) >= 2.10, "source_wav_mix_treatment_contrast_too_low", failures)


def validate_tail_metrics(
    dense: dict[str, Any],
    matrix: dict[str, Any],
    source_wav: dict[str, Any],
    failures: list[str],
) -> None:
    require(number(dense.get("tail_shape_source_derived")) == 1.0, "dense_tail_shape_not_source_derived", failures)
    require(number(dense.get("tail_shape_fixed_distance")) >= 0.20, "dense_tail_shape_fixed_distance_too_low", failures)
    require(number(dense.get("tail_shape_output_contrast_ratio")) >= 3.00, "dense_tail_shape_contrast_too_low", failures)
    require(number(matrix.get("min_source_derived_tail_shape_fixed_distance")) >= 0.20, "matrix_tail_shape_fixed_distance_too_low", failures)
    require(number(matrix.get("min_source_derived_tail_shape_output_contrast_ratio")) >= 3.00, "matrix_tail_shape_contrast_too_low", failures)
    require(number(source_wav.get("min_source_derived_tail_shape_fixed_distance")) >= 0.20, "source_wav_tail_shape_fixed_distance_too_low", failures)
    require(number(source_wav.get("min_source_derived_tail_shape_output_contrast_ratio")) >= 3.00, "source_wav_tail_shape_contrast_too_low", failures)


def validate_strongest_element_metrics(
    dense: dict[str, Any],
    matrix: dict[str, Any],
    source_wav: dict[str, Any],
    edge: dict[str, Any],
    failures: list[str],
) -> None:
    require(str(dense.get("strongest_audible_element")) in AUDIBLE_ELEMENTS, "dense_strongest_audible_element_missing", failures)
    require(number(dense.get("strongest_audible_element_score")) >= 1.00, "dense_strongest_audible_element_score_too_low", failures)
    require(number(dense.get("strongest_audible_element_margin")) >= 0.05, "dense_strongest_audible_element_margin_too_low", failures)
    require(str(dense.get("strongest_audible_element")) == "snare", "dense_snare_not_strongest_element", failures)
    require(number(dense.get("dense_break_physical_drum_pressure_score")) >= 1.58, "dense_drum_pressure_too_weak", failures)
    require(number(dense.get("dense_break_snare_pressure_margin")) >= 0.22, "dense_snare_pressure_margin_too_low", failures)
    require(number(dense.get("dense_break_pressure_transient_to_hook_ratio")) >= 0.70, "dense_pressure_transient_too_soft", failures)
    require("snare" in list_or_empty(matrix.get("strongest_audible_elements")), "matrix_snare_strongest_element_missing", failures)
    require("bass" in list_or_empty(matrix.get("strongest_audible_elements")), "matrix_bass_strongest_element_missing", failures)
    require(number(matrix.get("min_strongest_audible_element_score")) >= 1.00, "matrix_strongest_audible_element_score_too_low", failures)
    require(number(matrix.get("min_strongest_audible_element_margin")) >= 0.05, "matrix_strongest_audible_element_margin_too_low", failures)
    require("stab" in list_or_empty(source_wav.get("strongest_audible_elements")), "source_wav_stab_strongest_element_missing", failures)
    require("bass" in list_or_empty(source_wav.get("strongest_audible_elements")), "source_wav_bass_strongest_element_missing", failures)
    require(number(source_wav.get("min_strongest_audible_element_score")) >= 1.00, "source_wav_strongest_audible_element_score_too_low", failures)
    require(number(source_wav.get("min_strongest_audible_element_margin")) >= 0.05, "source_wav_strongest_audible_element_margin_too_low", failures)
    require("stab" in list_or_empty(edge.get("strongest_audible_elements")), "edge_stab_strongest_element_missing", failures)
    require("snare" in list_or_empty(edge.get("strongest_audible_elements")), "edge_snare_strongest_element_missing", failures)
    require(number(edge.get("min_strongest_audible_element_score")) >= 1.00, "edge_strongest_audible_element_score_too_low", failures)
    require(number(edge.get("min_strongest_audible_element_margin")) >= 0.05, "edge_strongest_audible_element_margin_too_low", failures)


def validate_source_character_metrics(
    dense: dict[str, Any],
    matrix: dict[str, Any],
    source_wav: dict[str, Any],
    edge: dict[str, Any],
    failures: list[str],
) -> None:
    require(number(dense.get("rebuild_only_source_spectral_similarity")) >= 0.60, "dense_source_spectral_similarity_too_low", failures)
    require(number(dense.get("rebuild_only_source_transient_retention")) >= 0.45, "dense_source_transient_retention_too_low", failures)
    require(number(dense.get("rebuild_only_source_character_survival_score")) >= 0.70, "dense_source_character_not_surviving", failures)
    require(number(dense.get("rebuild_only_source_character_survival_margin")) >= MIN_REBUILD_ONLY_SOURCE_CHARACTER_SURVIVAL_MARGIN, "dense_source_character_margin_too_low", failures)
    require(number(matrix.get("min_rebuild_only_source_spectral_similarity")) >= 0.60, "matrix_source_spectral_similarity_too_low", failures)
    require(number(matrix.get("min_rebuild_only_source_transient_retention")) >= 0.45, "matrix_source_transient_retention_too_low", failures)
    require(number(matrix.get("min_rebuild_only_source_character_survival_score")) >= 0.70, "matrix_source_character_not_surviving", failures)
    require(number(matrix.get("min_rebuild_only_source_character_survival_margin")) >= MIN_REBUILD_ONLY_SOURCE_CHARACTER_SURVIVAL_MARGIN, "matrix_source_character_margin_too_low", failures)
    require(number(source_wav.get("min_rebuild_only_source_spectral_similarity")) >= 0.60, "source_wav_source_spectral_similarity_too_low", failures)
    require(number(source_wav.get("min_rebuild_only_source_transient_retention")) >= 0.45, "source_wav_source_transient_retention_too_low", failures)
    require(number(source_wav.get("min_rebuild_only_source_character_survival_score")) >= 0.70, "source_wav_source_character_not_surviving", failures)
    require(number(source_wav.get("min_rebuild_only_source_character_survival_margin")) >= MIN_REBUILD_ONLY_SOURCE_CHARACTER_SURVIVAL_MARGIN, "source_wav_source_character_margin_too_low", failures)
    require(number(edge.get("min_rebuild_only_source_spectral_similarity")) >= 0.60, "edge_source_spectral_similarity_too_low", failures)
    require(number(edge.get("min_rebuild_only_source_transient_retention")) >= 0.45, "edge_source_transient_retention_too_low", failures)
    require(number(edge.get("min_rebuild_only_source_character_survival_score")) >= 0.70, "edge_source_character_not_surviving", failures)
    require(number(edge.get("min_rebuild_only_source_character_survival_margin")) >= MIN_REBUILD_ONLY_SOURCE_CHARACTER_SURVIVAL_MARGIN, "edge_source_character_margin_too_low", failures)


def validate_rebuild_balance_metrics(
    matrix: dict[str, Any], source_wav: dict[str, Any], failures: list[str]
) -> None:
    require(number(matrix.get("min_rebuild_only_to_full_rms_ratio")) >= 0.42, "matrix_rebuild_only_to_full_ratio_too_low", failures)
    require(number(matrix.get("max_rebuild_only_to_source_correlation")) <= 0.92, "matrix_rebuild_only_source_correlation_too_high", failures)
    require(number(source_wav.get("min_rebuild_only_to_full_rms_ratio")) >= 0.42, "source_wav_rebuild_only_to_full_ratio_too_low", failures)


def validate_artifacts(output: Path, failures: list[str]) -> None:
    for relative in (
        "README.md",
        "professional-output-listening-pack/reviews/dense_beat03_130/review.json",
        "destructive-variation/destructive-variation.json",
        "rendered-weak-professional-outputs/rendered-weak-professional-outputs.json",
        "non-dense-professional-proof-pack/non-dense-professional-proof-pack.json",
        "edge-source-professional-diagnostics/edge-source-professional-diagnostics.json",
    ):
        path = output / relative
        require(path.is_file() and path.stat().st_size > 0, f"missing_artifact:{relative}", failures)


def validate_feral_mix_balance_metrics(report: dict[str, Any], failures: list[str]) -> None:
    balance = object_or_empty(report.get("feral_mix_balance"))
    cases = list_or_empty(balance.get("cases"))
    require(balance.get("result") == "pass", "feral_mix_balance_not_pass", failures)
    require(number(balance.get("case_count")) >= 8, "feral_mix_balance_case_count_too_low", failures)
    require(
        all(object_or_empty(case).get("has_required_mix_balance_fields") is True for case in cases),
        "feral_mix_balance_fields_missing",
        failures,
    )
    require(
        number(balance.get("max_source_first_generated_to_source_rms_ratio"))
        <= MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
        "feral_source_first_generated_support_masks_source",
        failures,
    )
    require(
        number(balance.get("min_source_first_masking_headroom"))
        >= MIN_FERAL_SOURCE_FIRST_MASKING_HEADROOM,
        "feral_source_first_masking_headroom_too_low",
        failures,
    )
    require(
        number(balance.get("min_support_generated_to_source_rms_ratio"))
        >= MIN_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
        "feral_generated_support_too_buried",
        failures,
    )
    require(
        number(balance.get("max_support_generated_to_source_rms_ratio"))
        <= MAX_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
        "feral_generated_support_masks_source",
        failures,
    )


def validate_source_character_window_selection_metrics(
    report: dict[str, Any], failures: list[str]
) -> None:
    selection = object_or_empty(report.get("source_character_window_selection"))
    cases = list_or_empty(selection.get("cases"))
    require(
        selection.get("result") == "pass",
        "source_character_window_selection_not_pass",
        failures,
    )
    require(
        number(selection.get("case_count")) >= 8,
        "source_character_window_selection_case_count_too_low",
        failures,
    )
    require(
        number(selection.get("searched_case_count"))
        >= MIN_SOURCE_CHARACTER_WINDOW_SEARCHED_CASE_COUNT,
        "source_character_window_selection_search_coverage_too_low",
        failures,
    )
    require(
        number(selection.get("promoted_case_count"))
        >= MIN_SOURCE_CHARACTER_WINDOW_PROMOTED_CASE_COUNT,
        "source_character_window_selection_promoted_count_too_low",
        failures,
    )
    require(
        number(selection.get("min_observed_rms_retention_ratio"))
        + 1e-6
        >= MIN_SOURCE_CHARACTER_WINDOW_RMS_RETENTION_RATIO,
        "source_character_window_selection_rms_retention_too_low",
        failures,
    )
    require(
        all(
            object_or_empty(case).get("has_required_source_character_window_fields") is True
            for case in cases
        ),
        "source_character_window_selection_fields_missing",
        failures,
    )
    require(
        all(
            number(object_or_empty(case).get("selected_score"))
            >= number(object_or_empty(case).get("requested_head_score"))
            for case in cases
        ),
        "source_character_window_selection_score_regressed",
        failures,
    )
    require(
        all(
            number(object_or_empty(case).get("scanned_candidate_count")) >= 1
            for case in cases
        ),
        "source_character_window_selection_not_scanned",
        failures,
    )
    require(
        all(
            number(object_or_empty(case).get("rms_retention_ratio"))
            + 1e-6
            >= number(object_or_empty(case).get("min_rms_retention_ratio"))
            for case in cases
            if str(object_or_empty(case).get("reason")) == "source_character_window_promoted"
        ),
        "source_character_window_selection_promoted_rms_retention_too_low",
        failures,
    )
    require(
        all(
            str(object_or_empty(case).get("reason"))
            in {"requested_source_window_kept", "source_character_window_promoted"}
            for case in cases
        ),
        "source_character_window_selection_unknown_reason",
        failures,
    )


def validate_tr909_rendered_drum_pressure_metrics(
    report: dict[str, Any], failures: list[str]
) -> None:
    pressure = object_or_empty(report.get("tr909_rendered_drum_pressure"))
    cases = list_or_empty(pressure.get("cases"))
    require(
        pressure.get("result") == "pass",
        "tr909_rendered_drum_pressure_not_pass",
        failures,
    )
    require(
        number(pressure.get("case_count")) >= 8,
        "tr909_rendered_drum_pressure_case_count_too_low",
        failures,
    )
    require(
        all(
            object_or_empty(case).get(
                "has_required_tr909_rendered_drum_pressure_fields"
            )
            is True
            for case in cases
        ),
        "tr909_rendered_drum_pressure_fields_missing",
        failures,
    )
    require(
        all(
            object_or_empty(case).get("pattern_origin") == "source_derived"
            for case in cases
        ),
        "tr909_rendered_drum_pressure_not_source_derived",
        failures,
    )
    require(
        all(
            number(object_or_empty(case).get("support_mix_tr909_contribution_ratio"))
            >= number(
                object_or_empty(case).get(
                    "min_required_support_mix_tr909_contribution_ratio"
                )
            )
            for case in cases
        ),
        "tr909_rendered_drum_pressure_too_buried",
        failures,
    )
    require(
        all(
            number(object_or_empty(case).get("tr909_low_band_rms"))
            >= number(object_or_empty(case).get("min_required_tr909_low_band_rms"))
            for case in cases
        ),
        "tr909_rendered_drum_pressure_low_band_too_weak",
        failures,
    )
    require(
        number(pressure.get("max_source_first_generated_to_source_rms_ratio"))
        <= MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
        "tr909_rendered_drum_pressure_masks_source_first",
        failures,
    )
    require(
        number(pressure.get("max_support_generated_to_source_rms_ratio"))
        <= MAX_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
        "tr909_rendered_drum_pressure_support_masks_source",
        failures,
    )


def child_metrics(children: list[Any], child_id: str) -> dict[str, Any]:
    for child in children:
        if isinstance(child, dict) and child.get("id") == child_id:
            return object_or_empty(child.get("key_metrics"))
    return {}


def set_child_metric(
    report: dict[str, Any],
    child_id: str,
    metric: str,
    value: Any,
) -> bool:
    for child in list_or_empty(report.get("children")):
        if not isinstance(child, dict) or child.get("id") != child_id:
            continue
        metrics = child.get("key_metrics")
        if not isinstance(metrics, dict):
            return False
        metrics[metric] = value
        return True
    return False


def set_first_child_field(report: dict[str, Any], field: str, value: Any) -> bool:
    children = report.get("children")
    if not isinstance(children, list) or not children or not isinstance(children[0], dict):
        return False
    children[0][field] = value
    return True


def set_nested_value(report: dict[str, Any], path: list[str], value: Any) -> bool:
    current: Any = report
    for key in path[:-1]:
        if not isinstance(current, dict):
            return False
        current = current.get(key)
    if not isinstance(current, dict):
        return False
    current[path[-1]] = value
    return True


def set_first_listening_case_field(report: dict[str, Any], field: str, value: Any) -> bool:
    identity = report.get("listening_identity")
    if not isinstance(identity, dict):
        return False
    cases = identity.get("cases")
    if not isinstance(cases, list) or not cases or not isinstance(cases[0], dict):
        return False
    cases[0][field] = value
    return True


def set_first_listening_case_gate_field(report: dict[str, Any], field: str, value: Any) -> bool:
    identity = report.get("listening_identity")
    if not isinstance(identity, dict):
        return False
    cases = identity.get("cases")
    if not isinstance(cases, list) or not cases or not isinstance(cases[0], dict):
        return False
    gate = cases[0].get("mc202_source_composed_review_gate")
    if not isinstance(gate, dict):
        return False
    gate[field] = value
    return True


def mark_first_listening_case_template_only_unblocked(report: dict[str, Any]) -> bool:
    identity = report.get("listening_identity")
    if not isinstance(identity, dict):
        return False
    cases = identity.get("cases")
    if not isinstance(cases, list) or not cases or not isinstance(cases[0], dict):
        return False
    gate = cases[0].get("mc202_source_composed_review_gate")
    if not isinstance(gate, dict):
        return False
    gate["source_composed_evidence"] = False
    gate["primitive_or_template_only"] = True
    gate["template_only_blocks_promotion"] = False
    return True


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"{path}: JSON root must be object")
    return value


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def list_or_empty(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def number(value: Any) -> float:
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def require(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


if __name__ == "__main__":
    sys.exit(main())
