#!/usr/bin/env bash
set -euo pipefail

validate_existing=false
if [[ "${1:-}" == "--validate-existing" ]]; then
  validate_existing=true
  shift
fi

output="${1:-artifacts/audio_qa/local-dense-break-live-path-smoke}"
bpm="${2:-132}"
source_dir="$output/generated-source"

if [[ "$validate_existing" == false ]]; then
  python3 scripts/write_diverse_test_source_wavs.py --output "$source_dir" --seconds 32.0
  cargo run -p riotbox-app --bin dense_break_live_path_render -- \
    --source "$source_dir/dense_break_132.wav" \
    --bpm "$bpm" \
    --output "$output"
fi

for artifact in \
  00_source.wav \
  01_all_lane_hook.wav \
  02_all_lane_destructive.wav \
  monitor/00_source.wav \
  monitor/01_blend.wav \
  monitor/02_riotbox.wav \
  gestures/00_ready_riotbox.wav \
  gestures/01_after_w_hit.wav \
  gestures/02_after_s_slam.wav \
  gestures/03_after_f_fill.wav \
  gestures/04_after_y_scene_jump.wav \
  gestures/05_after_Y_D_changed_return.wav \
  gestures/06_live_sequence.wav \
  gestures/proofs/01_w_before.wav \
  gestures/proofs/01_w_after.wav \
  gestures/proofs/02_s_before.wav \
  gestures/proofs/02_s_after.wav \
  gestures/proofs/03_f_before.wav \
  gestures/proofs/03_f_after.wav \
  gestures/proofs/04_y_before.wav \
  gestures/proofs/04_y_after.wav \
  gestures/proofs/05_Y_before.wav \
  gestures/proofs/05_Y_after.wav \
  alpha/00_hook_establish.wav \
  alpha/01_pressure_lift.wav \
  alpha/02_destructive_fill.wav \
  alpha/03_destructive_role_swap.wav \
  alpha/04_changed_return.wav \
  alpha/05_feral_break_alpha_eight_bar.wav \
  alpha/06_source_reference_raw.wav \
  alpha/07_candidate_loudness_matched.wav \
  alpha/08_source_reference_loudness_matched.wav \
  alpha/09_restart_recall_trigger.wav \
  stems/01_w30_hook.wav \
  stems/02_tr909_pressure.wav \
  stems/03_mc202_selected_role.wav \
  gesture-manifest.json \
  session.json \
  source-graph.json
do
  test -s "$output/$artifact"
done

python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$output/gesture-manifest.json"

jq -e \
  --argjson cli_bpm_hint "$bpm" \
  --slurpfile source_graph "$output/source-graph.json" \
  --slurpfile session "$output/session.json" \
  '
  def exact_limiter_ok($max_limited):
    .threshold > 0
    and .ceiling > .threshold
    and .limited_sample_count <= $max_limited
    and .pre.clip_count == 0
    and .post.clip_count == 0
    and (.applied == (.limited_sample_count > 0));
  . as $manifest
  | ($source_graph[0]) as $graph
  | ($session[0]) as $session_file
  | ($graph.timing.hypotheses[]
      | select(.hypothesis_id == $graph.timing.primary_hypothesis_id)) as $primary_timing
  | .schema_version == 1
  and .pack_id == "dense-break-live-path"
  and .result == "pass"
  and .evidence_role == "diagnostic"
  and .source_backed == true
  and .source_timing_backed == true
  and .scripted_generation == true
  and .quality_proof == false
  and .human_verdict == "unverified"
  and .evidence_boundary.schema == "riotbox.audio_qa_evidence_boundary.v1"
  and .evidence_boundary.schema_version == 1
  and .evidence_boundary.evidence_role == "diagnostic"
  and .evidence_boundary.source_backed == true
  and .evidence_boundary.source_timing_backed == true
  and .evidence_boundary.scripted_generation == true
  and .evidence_boundary.quality_proof == false
  and .evidence_boundary.human_verdict == "unverified"
  and .performance_preset.preset_id == "feral_break_alpha_v2"
  and .performance_preset.profile_id == "feral_rebuild"
  and .performance_preset.label == "Feral Break Alpha v2"
  and .performance_preset.w30_role == "source_hook_lead"
  and .performance_preset.tr909_role == "break_pressure"
  and .performance_preset.tr909_reinforcement_mode == "break_reinforce"
  and .performance_preset.mc202_role == "source_evidence_selected"
  and .performance_preset.source_monitor_mode == "riotbox"
  and .performance_preset.bass_ownership_policy == "live_performance_policy"
  and (["mc202", "unassigned"] | index($manifest.performance_preset.actual_bass_owner)) != null
  and (.performance_preset.activation_action_id | type == "number")
  and $session_file.runtime_state.style.active_profile == .performance_preset.profile_id
  and $session_file.runtime_state.style.active_preset == .performance_preset.preset_id
  and .feral_break_alpha_capture_journey.sequence == [
    "capture",
    "raw_audition",
    "promote_to_pad",
    "save",
    "restart",
    "live_recall",
    "trigger"
  ]
  and .feral_break_alpha_capture_journey.saved_before_restart == true
  and (.feral_break_alpha_capture_journey.capture_action_id | type == "number")
  and (.feral_break_alpha_capture_journey.raw_audition_action_id | type == "number")
  and (.feral_break_alpha_capture_journey.promotion_action_id | type == "number")
  and (.feral_break_alpha_capture_journey.restart_recall_action_id | type == "number")
  and (.feral_break_alpha_capture_journey.restart_trigger_action_id | type == "number")
  and .feral_break_alpha_arc.duration_bars == 8
  and .feral_break_alpha_arc.duration_beats == 32
  and .feral_break_alpha_arc.human_verdict == "unverified"
  and .feral_break_alpha_arc.typed_bass_owner == .performance_preset.actual_bass_owner
  and .feral_break_alpha_arc.artifact == "alpha/05_feral_break_alpha_eight_bar.wav"
  and (.feral_break_alpha_arc.stages | map(.case_id)) == [
    "alpha-hook-establish",
    "alpha-pressure-lift",
    "alpha-destructive-fill",
    "alpha-destructive-role-swap",
    "alpha-changed-return"
  ]
  and (.feral_break_alpha_arc.stages | map(.duration_beats)) == [8, 8, 4, 4, 8]
  and (.feral_break_alpha_arc.stages | map(.key)) == ["w", "s", "f", "y", "Y+D"]
  and (.feral_break_alpha_arc.stages | map(.command)) == [
    "w30.trigger_pad",
    "tr909.set_slam",
    "tr909.fill_next",
    "scene.launch",
    "scene.restore"
  ]
  and .feral_break_alpha_arc.stages[2].tr909_mode == "fill"
  and .feral_break_alpha_arc.stages[2].tr909_fill_recipe_id
    == "phrase_drive_break_cut_stomp_v2"
  and .feral_break_alpha_arc.destructive_negative_space.window == {
    "start_step": 26,
    "end_step_exclusive": 30,
    "steps_per_beat": 8
  }
  and .feral_break_alpha_arc.destructive_negative_space.metrics.rms
    <= .feral_break_alpha_arc.destructive_negative_space.thresholds.max_rms
  and .feral_break_alpha_arc.destructive_negative_space.metrics.silence_ratio
    >= .feral_break_alpha_arc.destructive_negative_space.thresholds.min_silence_ratio
  and .feral_break_alpha_arc.destructive_negative_space.hard_return.start_step == 30
  and .feral_break_alpha_arc.destructive_negative_space.hard_return.end_step_exclusive == 31
  and .feral_break_alpha_arc.destructive_negative_space.hard_return.metrics.rms
    >= .feral_break_alpha_arc.destructive_negative_space.hard_return.min_rms
  and all(.feral_break_alpha_arc.stages[];
    .metrics.rms > $manifest.thresholds.min_mix_rms
    and .metrics.clip_count == 0
    and (.limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count)))
  and .feral_break_alpha_arc.hook_to_pressure_delta.rms > .thresholds.min_monitor_delta_rms
  and .feral_break_alpha_arc.hook_to_changed_return_delta.rms > .thresholds.min_monitor_delta_rms
  and (.feral_break_alpha_arc.hook_to_changed_return_correlation | type == "number")
  and ((.feral_break_alpha_arc.hook_to_changed_return_correlation | abs) < 0.985)
  and .feral_break_alpha_arc.scenes.original != .feral_break_alpha_arc.scenes.contrast
  and .feral_break_alpha_arc.scenes.returned == .feral_break_alpha_arc.scenes.original
  and .feral_break_alpha_arc.raw_level_ab.candidate_artifact == "alpha/05_feral_break_alpha_eight_bar.wav"
  and .feral_break_alpha_arc.raw_level_ab.source_artifact == "alpha/06_source_reference_raw.wav"
  and .feral_break_alpha_arc.raw_level_ab.candidate_metrics.rms > .thresholds.min_mix_rms
  and .feral_break_alpha_arc.raw_level_ab.source_metrics.rms > .thresholds.min_mix_rms
  and .feral_break_alpha_arc.loudness_matched_ab.candidate_artifact == "alpha/07_candidate_loudness_matched.wav"
  and .feral_break_alpha_arc.loudness_matched_ab.source_artifact == "alpha/08_source_reference_loudness_matched.wav"
  and .feral_break_alpha_arc.loudness_matched_ab.target_rms > .thresholds.min_mix_rms
  and ((.feral_break_alpha_arc.loudness_matched_ab.candidate_metrics.rms
    - .feral_break_alpha_arc.loudness_matched_ab.source_metrics.rms) | abs) <= 0.00001
  and .feral_break_alpha_arc.loudness_matched_ab.candidate_metrics.clip_count == 0
  and .feral_break_alpha_arc.loudness_matched_ab.source_metrics.clip_count == 0
  and .feral_break_alpha_restart_recall.preset_survived_restart == true
  and .feral_break_alpha_restart_recall.artifact == "alpha/09_restart_recall_trigger.wav"
  and .feral_break_alpha_restart_recall.monitor_mode == "riotbox"
  and .feral_break_alpha_restart_recall.w30_routing == "music_bus_preview"
  and .feral_break_alpha_restart_recall.metrics.rms > .thresholds.min_mix_rms
  and .feral_break_alpha_restart_recall.metrics.clip_count == 0
  and (.feral_break_alpha_restart_recall.limiter
    | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
  and .pattern_provenance.tr909_fill.pattern_origin == "primitive_renderer"
  and .pattern_provenance.tr909_fill.source_evidence_role == "availability_timing_and_pressure_modulation"
  and .pattern_provenance.tr909_fill.source_evidence_selects_pattern == false
  and .pattern_provenance.tr909_fill.source_evidence_modulates_output == true
  and .pattern_provenance.tr909_fill.primitive_schema == "riotbox.tr909_fill_recipe.v1"
  and .pattern_provenance.tr909_fill.recipe_id == "phrase_drive_break_cut_stomp_v2"
  and .pattern_provenance.tr909_fill.selection_inputs.mode == "fill"
  and .pattern_provenance.tr909_fill.selection_inputs.routing == "drum_bus_support"
  and .pattern_provenance.tr909_fill.selection_inputs.pattern_adoption == "mainline_drive"
  and .pattern_provenance.tr909_fill.selection_inputs.phrase_variation == "phrase_drive_hard_cut"
  and .pattern_provenance.tr909_fill.source_modulation.schema == "riotbox.tr909_fill_source_modulation.v2"
  and .pattern_provenance.tr909_fill.source_modulation.source_feature_path == "session.runtime_state.lane_state.mc202.source_phrase_plan.source_expression.transient_backbeat"
  and .pattern_provenance.tr909_fill.source_modulation.source_timing_path == "source_graph.timing.primary_hypothesis.transport_bar_grid_anchor.beat_cursor"
  and (.pattern_provenance.tr909_fill.source_modulation.source_feature_value | type == "number")
  and .pattern_provenance.tr909_fill.source_modulation.source_feature_value >= 0
  and .pattern_provenance.tr909_fill.source_modulation.source_feature_value <= 1
  and ((.pattern_provenance.tr909_fill.source_modulation.derived_policy.tr909_drum_level - (0.68 + .pattern_provenance.tr909_fill.source_modulation.source_feature_value * 0.16)) > -0.00001)
  and ((.pattern_provenance.tr909_fill.source_modulation.derived_policy.tr909_drum_level - (0.68 + .pattern_provenance.tr909_fill.source_modulation.source_feature_value * 0.16)) < 0.00001)
  and ((.pattern_provenance.tr909_fill.source_modulation.derived_policy.tr909_slam_floor - (0.54 + .pattern_provenance.tr909_fill.source_modulation.source_feature_value * 0.16)) > -0.00001)
  and ((.pattern_provenance.tr909_fill.source_modulation.derived_policy.tr909_slam_floor - (0.54 + .pattern_provenance.tr909_fill.source_modulation.source_feature_value * 0.16)) < 0.00001)
  and .pattern_provenance.tr909_fill.source_modulation.derived_policy.source_bar_grid_anchor_beat_cursor == .timing_identity.primary_bar_anchor_beat_cursor
  and .pattern_provenance.tr909_fill.source_modulation.resolved_render_inputs.drum_bus_level == .gesture_transitions[2].control_values.tr909_drum_bus_level_after
  and .pattern_provenance.tr909_fill.source_modulation.resolved_render_inputs.slam_intensity == .gesture_transitions[2].control_values.tr909_slam_after
  and .pattern_provenance.tr909_fill.source_modulation.resolved_render_inputs.slam_enabled == .gesture_transitions[2].control_values.tr909_slam_enabled_after
  and .pattern_provenance.tr909_fill.source_modulation.resolved_render_inputs.source_bar_grid_anchor_position_beats == .timing_identity.primary_bar_anchor_beat_cursor
  and (.pattern_provenance.tr909_fill.source_modulation.affected_runtime_parameters == [
    "runtime_mix.tr909.drum_bus_level",
    "runtime_mix.tr909.slam_intensity",
    "runtime_mix.tr909.source_bar_grid_phase"
  ])
  and .pattern_provenance.tr909_fill.source_modulation.pattern_selection_changed == false
  and .pattern_provenance.tr909_fill.activation_ref == "/gesture_transitions/2"
  and (.pattern_provenance.tr909_fill.affected_artifacts == [
    "gestures/03_after_f_fill.wav",
    "gestures/06_live_sequence.wav",
    "gestures/proofs/03_f_after.wav"
  ])
  and .primitive_renderer_boundary.schema == "riotbox.primitive_renderer_boundary.v2"
  and .primitive_renderer_boundary.evidence_role == "product_primitive_vocabulary"
  and .primitive_renderer_boundary.product_output_allowed == true
  and .primitive_renderer_boundary.quality_proof == false
  and .primitive_renderer_boundary.demo_readiness == "unverified"
  and .primitive_renderer_boundary.promotion_blocked == true
  and .primitive_renderer_boundary.promotion_target == "source_derived_musical_intelligence"
  and .primitive_renderer_boundary.promotion_target_scope == "recipe_and_pattern_selection"
  and .primitive_renderer_boundary.recipe_derivation_claimed == false
  and .primitive_renderer_boundary.pattern_selection_claimed == false
  and .primitive_renderer_boundary.source_output_modulation_claimed == true
  and .primitive_renderer_boundary.activation.kind == "explicit_committed_performer_gesture"
  and (.primitive_renderer_boundary.activation.references == [
    "/gesture_transitions/2"
  ])
  and .primitive_renderer_boundary.source_failure_fallback == false
  and (.primitive_renderer_boundary.affected_paths == [
    "pattern_provenance.tr909_fill.pattern_origin"
  ])
  and (.primitive_renderer_boundary.affected_runtime_paths == [
    "runtime_mix.tr909.fill_recipe",
    "runtime_mix.tr909.drum_bus_level",
    "runtime_mix.tr909.slam_intensity",
    "runtime_mix.tr909.source_bar_grid_phase",
    "runtime_mix.non_tr909_bed.fill_focus",
    "runtime_mix.source_monitor.blend_fill_focus"
  ])
  and (.primitive_renderer_boundary.affected_artifacts == [
    "gestures/03_after_f_fill.wav",
    "gestures/06_live_sequence.wav",
    "gestures/proofs/03_f_after.wav"
  ])
  and (.primitive_renderer_boundary.musician_message | length > 0)
  and .fill_exit_boundary_proof.from_case_id == "after-f-fill"
  and .fill_exit_boundary_proof.to_case_id == "after-y-scene-jump"
  and .fill_exit_boundary_proof.expected_role == "fill_release_to_scene_contrast_downbeat"
  and .fill_exit_boundary_proof.exact_runtime_mix_sequence == true
  and .fill_exit_boundary_proof.window_ms == 10
  and .fill_exit_boundary_proof.window_frames == 480
  and .fill_exit_boundary_proof.thresholds.max_boundary_step >= 0.199
  and .fill_exit_boundary_proof.thresholds.max_boundary_step <= 0.201
  and .fill_exit_boundary_proof.thresholds.max_boundary_to_local_p99_ratio == 4.0
  and .fill_exit_boundary_proof.boundary_step <= .fill_exit_boundary_proof.thresholds.max_boundary_step
  and .fill_exit_boundary_proof.boundary_to_local_p99_ratio <= .fill_exit_boundary_proof.thresholds.max_boundary_to_local_p99_ratio
  and .fill_exit_boundary_proof.local_adjacent_step_p99 > 0
  and .exact_mixer_proof.kind == "runtime_mix_callback_block_realtime_simulation"
  and .exact_mixer_proof.stateful_sequence == true
  and .exact_mixer_proof.source_monitor_included == true
  and .exact_mixer_proof.master_limiter_included == true
  and .exact_mixer_proof.pre_post_limiter_reported == true
  and .exact_mixer_proof.limiter_activity_gated == true
  and .correlation_scope.kind == "action_contract_only"
  and .correlation_scope.shared_source_fixture == false
  and .correlation_scope.shared_transport_timeline == false
  and .correlation_scope.sample_exact_observer_correlation == false
  and .sample_rate == 48000
  and .source.sample_rate == 44100
  and .timing_identity.cli_bpm_hint == $cli_bpm_hint
  and .timing_identity.confirmed_source_id == $graph.source.source_id
  and .timing_identity.confirmed_source_id == $session_file.runtime_state.source_timing.confirmed_grid.source_id
  and .timing_identity.confirmed_hypothesis_id == $graph.timing.primary_hypothesis_id
  and .timing_identity.confirmed_hypothesis_id == $session_file.runtime_state.source_timing.confirmed_grid.hypothesis_id
  and .timing_identity.confirmed_hypothesis_kind == (
    if $primary_timing.kind == "Manual" then "musician_manual"
    elif $primary_timing.kind == "Primary" then "analyzer_primary"
    elif $primary_timing.kind == "HalfTime" then "half_time"
    elif $primary_timing.kind == "DoubleTime" then "double_time"
    elif $primary_timing.kind == "AlternateDownbeat" then "alternate_downbeat"
    elif $primary_timing.kind == "Ambiguous" then "ambiguous"
    else "unknown"
    end
  )
  and (if $primary_timing.kind == "Manual" then
    .timing_identity.manual_grid_input.declared_bpm == $primary_timing.bpm
    and .timing_identity.manual_grid_input.declared_downbeat_seconds
      == ($primary_timing.bar_grid | min_by(.bar_index) | .start_seconds)
  else
    .timing_identity.manual_grid_input == null
  end)
  and .timing_identity.confirmed_hypothesis_bpm == $primary_timing.bpm
  and .timing_identity.beats_per_bar == $primary_timing.meter.beats_per_bar
  and .timing_identity.primary_bar_anchor_beat_index == (.timing_identity.primary_bar_anchor_beat_cursor + 1)
  and .timing_identity.primary_bar_anchor_bar_index == ($primary_timing.bar_grid | min_by(.bar_index) | .bar_index)
  and .bpm == .timing_identity.confirmed_hypothesis_bpm
  and .timing_identity.render_plan_bpm == .timing_identity.confirmed_hypothesis_bpm
  and .timing_identity.frame_count_bpm == .timing_identity.confirmed_hypothesis_bpm
  and .timing_identity.metrics_grid_bpm == .timing_identity.confirmed_hypothesis_bpm
  and .timing_identity.all_render_plans_match_confirmed_bpm == true
  and .thresholds.max_exact_mix_limited_sample_count == 0
  and .thresholds.min_isolated_tr909_regression_rms >= 0.004999
  and .thresholds.max_source_monitor_silence_ratio <= 0.05
  and .monitor_cycle.review_duration_bars == 4
  and (.monitor_cycle.modes | map(.mode)) == ["source", "blend", "riotbox"]
  and (.monitor_cycle.modes | map(.route)) == ["source_only", "blend", "riotbox_only"]
  and all(.monitor_cycle.modes[]; .metrics.rms > $manifest.thresholds.min_mix_rms and .metrics.clip_count == 0)
  and (.monitor_cycle.modes[] | select(.mode == "source") | .metrics.silence_ratio) <= .thresholds.max_source_monitor_silence_ratio
  and all(.monitor_cycle.modes[]; .limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
  and .monitor_cycle.deltas.source_vs_blend.rms > .thresholds.min_monitor_delta_rms
  and .monitor_cycle.deltas.blend_vs_riotbox.rms > .thresholds.min_monitor_delta_rms
  and .monitor_cycle.deltas.source_vs_riotbox.rms > .thresholds.min_monitor_delta_rms
  and (.gesture_transitions | map(.key)) == ["w", "s", "f", "y", "Y"]
  and all(.gesture_transitions[] | select(.key != "Y"); .companion_actions == [])
  and (.gesture_transitions[] | select(.key == "Y") | .companion_actions) == [{
    "command": "w30.apply_damage_profile",
    "action_id": $manifest.scene_transition_proof.return_damage_action_id
  }]
  and (.gesture_transitions[] | select(.key == "w") | .commit_boundary.beat_cursor)
    == (.timing_identity.primary_bar_anchor_beat_cursor + (4 * .timing_identity.beats_per_bar) + 1)
  and (.gesture_transitions[] | select(.key == "s") | .commit_boundary.beat_cursor)
    == (.timing_identity.primary_bar_anchor_beat_cursor + (5 * .timing_identity.beats_per_bar))
  and (.gesture_transitions[] | select(.key == "f") | .commit_boundary.beat_cursor)
    == (.timing_identity.primary_bar_anchor_beat_cursor + (6 * .timing_identity.beats_per_bar))
  and (.gesture_transitions[] | select(.key == "y") | .commit_boundary.beat_cursor)
    == (.timing_identity.primary_bar_anchor_beat_cursor + (7 * .timing_identity.beats_per_bar))
  and (.gesture_transitions[] | select(.key == "Y") | .commit_boundary.beat_cursor)
    == (.timing_identity.primary_bar_anchor_beat_cursor + (8 * .timing_identity.beats_per_bar))
  and (.gesture_transitions | map(.commit_boundary.bar_index)) == [5, 6, 7, 8, 9]
  and (.gesture_transitions | map(.commit_boundary.phrase_index)) == [2, 2, 2, 2, 3]
  and .gesture_transitions[2].command == "tr909.fill_next"
  and .gesture_transitions[2].boundary == "Bar"
  and (.gesture_transitions[2].action_id | type == "number")
  and (.gesture_transitions[] | select(.key == "f") | .control_values.tr909_mode_before) == "break_reinforce"
  and (.gesture_transitions[] | select(.key == "f") | .control_values.tr909_mode_after) == "fill"
  and (.gesture_transitions[] | select(.key == "s") | .control_values.tr909_mode_before) == "break_reinforce"
  and (.gesture_transitions[] | select(.key == "s") | .control_values.tr909_mode_after) == "break_reinforce"
  and (.gesture_transitions[] | select(.key == "s") | .control_values.tr909_slam_enabled_before) == false
  and (.gesture_transitions[] | select(.key == "s") | .control_values.tr909_slam_enabled_after) == true
  and all(.gesture_transitions[];
    .candidate_metrics.rms > $manifest.thresholds.min_mix_rms
    and .candidate_metrics.clip_count == 0
    and (.counterfactual_limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
    and (.candidate_limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
    and .qa_candidate_metrics.rms > $manifest.thresholds.min_mix_rms
    and .delta.rms > .qa_thresholds.min_delta_rms
    and .delta.peak_abs > .qa_thresholds.min_delta_peak
    and .relative_delta_rms > .qa_thresholds.min_relative_delta_rms
    and .perceptual_delta.window_ms == 10
    and (((.perceptual_delta.candidate_relative_floor - 0.10) | abs) < 0.000001)
    and (((.perceptual_delta.absolute_floor - 0.00001) | abs) < 0.0000001)
    and (.perceptual_delta.relevant_window_activity_ratio | type == "number")
    and (.perceptual_delta.waveform_correlation | type == "number")
    and (.perceptual_delta.absolute_waveform_correlation | type == "number")
    and (.qa_thresholds.min_relevant_10ms_activity_ratio == null
      or .perceptual_delta.relevant_window_activity_ratio >= .qa_thresholds.min_relevant_10ms_activity_ratio)
    and (.qa_thresholds.max_waveform_correlation == null
      or .perceptual_delta.absolute_waveform_correlation <= .qa_thresholds.max_waveform_correlation)
    and .delta.active_samples > 0)
  and .scene_transition_proof.launch_changed_scene == true
  and .scene_transition_proof.expected_launch_anchor_seconds != null
  and .scene_transition_proof.launched_anchor_seconds != null
  and .scene_transition_proof.launch_anchor_matches_expected == true
  and .scene_transition_proof.restore_returned_to_pre_jump_scene == true
  and .scene_transition_proof.expected_restore_anchor_seconds != null
  and .scene_transition_proof.restored_anchor_seconds != null
  and .scene_transition_proof.restore_anchor_matches_expected == true
  and (.scene_transition_proof.return_damage_action_id | type == "number")
  and .scene_transition_proof.mc202_plan_source_section != null
  and .scene_transition_proof.launched_source_section != null
  and .scene_transition_proof.mc202_plan_source_section != .scene_transition_proof.launched_source_section
  and .scene_transition_proof.launch_mc202_stayed_out_for_section_mismatch == true
  and .scene_transition_proof.restore_audio_projection_matches_pre_jump == true
  and .scene_transition_proof.restore_only_lane_projection_matches_pre_jump == true
  and .scene_transition_proof.changed_return_w30_differs_from_restore_only == true
  and .scene_transition_proof.changed_return_non_w30_projection_matches_restore_only == true
  and .scene_transition_proof.pre_jump_scene != .scene_transition_proof.launched_scene
  and .scene_transition_proof.launched_anchor_seconds == .scene_transition_proof.expected_launch_anchor_seconds
  and .scene_transition_proof.restored_scene == .scene_transition_proof.pre_jump_scene
  and .scene_transition_proof.restored_anchor_seconds == .scene_transition_proof.expected_restore_anchor_seconds
  and .scene_transition_proof.launch_action_id == (.gesture_transitions[] | select(.key == "y") | .action_id)
  and .scene_transition_proof.restore_action_id == (.gesture_transitions[] | select(.key == "Y") | .action_id)
  and (.performance_stages | length) == 6
  and all(.performance_stages[]; .monitor_mode == "riotbox" and .monitor_route == "riotbox_only")
  and all(.performance_stages[]; .limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
  and .legacy_lane_regression.frozen_before_live_fill_slam_scene_gestures == true
  and .legacy_lane_regression.plan.tr909_mode == "break_reinforce"
  and .legacy_lane_regression.plan.tr909_routing == "drum_bus_support"
  and .legacy_lane_regression.plan.transport_position_beats == 0
  and .legacy_lane_regression.plan.monitor_mode == "riotbox"
  and .legacy_lane_regression.plan.monitor_route == "riotbox_only"
  and .legacy_lane_regression.tr909.rms > .thresholds.min_isolated_tr909_regression_rms
  and (.legacy_lane_regression.mix_limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
  and (.legacy_lane_regression.damage_limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
  and (.legacy_lane_regression.w30_limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
  and (.legacy_lane_regression.tr909_limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
  and (.legacy_lane_regression.mc202_limiter | exact_limiter_ok($manifest.thresholds.max_exact_mix_limited_sample_count))
  and (.failures | length) == 0
' "$output/gesture-manifest.json"
