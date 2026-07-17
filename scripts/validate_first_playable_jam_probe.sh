#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

reuse_exact_mix_dir=""
if [[ "${1:-}" == "--exact-mix-dir" ]]; then
  if [[ $# -ne 2 ]]; then
    echo "usage: $0 [--exact-mix-dir PATH]" >&2
    exit 2
  fi
  reuse_exact_mix_dir="$2"
elif [[ $# -ne 0 ]]; then
  echo "usage: $0 [--exact-mix-dir PATH]" >&2
  exit 2
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

probe_dir="$tmpdir/first-playable-jam"
mkdir -p "$probe_dir"
observer_fixture="$probe_dir/events.ndjson"

cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe first-playable-jam \
  --observer "$observer_fixture"
python3 scripts/validate_user_session_observer_ndjson.py "$observer_fixture"
jq -s -e \
  'def committed_command($command; $boundary; $beat; $bar; $phrase):
      any(.[]; . as $event
        | $event.event == "transport_commit"
          and any($event.committed[]; . as $commit
            | $commit.boundary == $boundary
              and $commit.beat_index == $beat
              and $commit.bar_index == $bar
              and $commit.phrase_index == $phrase
              and $commit.scene_id == "scene-01-break"
              and any($event.snapshot.queue.recent_history[];
                .id == $commit.action_id
                  and .command == $command
                  and .status == "Committed")));
    length >= 20
    and .[0].event == "observer_started"
    and .[0].launch.probe == "first-playable-jam"
    and .[0].launch.source_path == "synthetic-first-playable-source.wav"
    and .[0].snapshot.source_timing.source_id == "src-first-playable-jam"
    and .[0].snapshot.source_timing.beat_count == 64
    and .[0].snapshot.source_timing.bar_count == 16
    and .[0].snapshot.source_timing.phrase_count == 4
    and .[0].snapshot.source_map.mode == "bar grid"
    and .[0].snapshot.source_map.capture_range_available == true
    and .[0].snapshot.runtime.source_monitor_mode == "source"
    and .[0].snapshot.runtime.source_monitor_audio_route == "source_only"
    and all(.[]; has("snapshot"))
    and all(.[]; .snapshot.transport | type == "object")
    and all(.[]; .snapshot.queue | type == "object")
    and all(.[]; .snapshot.runtime | type == "object")
    and all(.[]; .snapshot.recovery | type == "object")
    and any(.[]; .event == "key_outcome" and .key == "space" and .outcome == "toggle_transport" and .snapshot.transport.is_playing == true)
    and any(.[]; .event == "key_outcome" and .key == "c" and .outcome == "queue_capture_bar" and .snapshot.queue.pending_count >= 1)
    and any(.[]; .event == "key_outcome" and .key == "o" and .outcome == "queue_w30_audition")
    and any(.[]; .event == "key_outcome" and .key == "p" and .outcome == "promote_last_capture")
    and any(.[]; .event == "transport_commit" and .snapshot.queue.session_log_count >= 1)
    and any(.[]; .event == "key_outcome" and .key == "M" and .outcome == "queue_source_monitor_mode" and .snapshot.runtime.source_monitor_mode == "blend" and .snapshot.runtime.source_monitor_audio_route == "blend" and .snapshot.queue.pending_count == 0 and .snapshot.transport.beat_index == 20 and .snapshot.transport.bar_index == 6 and .snapshot.transport.phrase_index == 2 and .snapshot.transport.current_scene == "scene-01-break")
    and any(.[]; .event == "transport_commit" and .timestamp_ms == 650 and (.committed | length) == 1 and .committed[0].boundary == "Immediate")
    and any(.[]; .event == "key_outcome" and .key == "w" and .outcome == "queue_w30_trigger_pad")
    and any(.[]; .event == "key_outcome" and .key == "f" and .outcome == "queue_tr909_fill")
    and any(.[]; .event == "key_outcome" and .key == "s" and .outcome == "queue_tr909_slam")
    and any(.[]; .event == "key_outcome" and .key == "y" and .outcome == "queue_scene_select")
    and any(.[]; .event == "key_outcome" and .key == "Y" and .outcome == "queue_scene_restore")
    and committed_command("capture.bar_group"; "Phrase"; 16; 5; 2)
    and committed_command("w30.audition_raw_capture"; "Bar"; 20; 6; 2)
    and committed_command("promote.capture_to_pad"; "Bar"; 20; 6; 2)
    and committed_command("source_monitor.set_mode"; "Immediate"; 20; 6; 2)
    and committed_command("w30.trigger_pad"; "Beat"; 21; 6; 2)
    and committed_command("tr909.fill_next"; "Bar"; 24; 7; 2)
    and committed_command("tr909.set_slam"; "Beat"; 25; 7; 2)
    and committed_command("scene.launch"; "Bar"; 36; 10; 3)
    and any(.[]; . as $event
      | $event.event == "transport_commit"
        and any($event.committed[]; . as $commit
          | $commit.boundary == "Bar"
            and $commit.beat_index == 40
            and $commit.bar_index == 11
            and $commit.phrase_index == 3
            and $commit.scene_id == "scene-02-drop"
            and any($event.snapshot.queue.recent_history[];
              .id == $commit.action_id
                and .command == "scene.restore"
                and .status == "Committed")))
    and ([.[] | select(.event == "transport_commit") | .committed[].beat_index] as $beats
      | $beats == [16, 20, 20, 20, 21, 24, 25, 36, 40]
        and all(range(1; $beats | length); $beats[.] >= $beats[. - 1]))
    and ([.[] | select(.event == "transport_commit") | .committed[].scene_id] == ["scene-01-break", "scene-01-break", "scene-01-break", "scene-01-break", "scene-01-break", "scene-01-break", "scene-01-break", "scene-01-break", "scene-02-drop"])
    and any(.[]; .event == "transport_commit" and .timestamp_ms == 800 and .snapshot.runtime.w30_preview_target == "bank-a / pad-01 | cap-01")
    and any(.[]; .event == "transport_commit" and .timestamp_ms == 1000 and .snapshot.runtime.tr909_mode == "fill" and .snapshot.runtime.tr909_routing == "drum_bus_support")
    and any(.[]; .event == "transport_commit" and .timestamp_ms == 1400 and .snapshot.scene.active_scene == "scene-02-drop" and .snapshot.scene.last_movement.from_scene == "scene-01-break" and .snapshot.scene.last_movement.to_scene == "scene-02-drop" and .snapshot.scene.source_monitor.source_anchor_seconds == 16.0 and .snapshot.capture.source_window.source_id == "src-first-playable-jam")
    and any(.[]; .event == "transport_commit" and .timestamp_ms == 1600 and .snapshot.scene.active_scene == "scene-01-break" and .snapshot.scene.last_movement.kind == "restore" and .snapshot.scene.last_movement.from_scene == "scene-02-drop" and .snapshot.scene.last_movement.to_scene == "scene-01-break" and .snapshot.queue.pending_count == 0 and .snapshot.queue.session_log_count == 9 and .snapshot.transport.beat_index == 40 and .snapshot.transport.bar_index == 11 and .snapshot.transport.phrase_index == 3 and .snapshot.transport.current_scene == "scene-01-break")' \
  "$observer_fixture"

if [[ -n "$reuse_exact_mix_dir" ]]; then
  exact_mix_dir="$reuse_exact_mix_dir"
  scripts/validate_dense_break_live_path.sh --validate-existing "$exact_mix_dir" 132
else
  exact_mix_dir="$probe_dir/exact-runtime-mix"
  scripts/validate_dense_break_live_path.sh "$exact_mix_dir" 132
fi
exact_mix_manifest="$exact_mix_dir/gesture-manifest.json"

jq -n -e \
  --slurpfile observer "$observer_fixture" \
  --slurpfile manifest "$exact_mix_manifest" \
  'def exact_limiter_ok($max_limited):
      .limited_sample_count <= $max_limited
      and .pre.clip_count == 0
      and .post.clip_count == 0
      and (.applied == (.limited_sample_count > 0));
    $manifest[0] as $mix
    | [$observer[] | select(.event == "key_outcome") | .key] as $keys
    | [$observer[]
        | select(.event == "transport_commit")
        | .snapshot.queue.recent_history[]
        | select(.status == "Committed")
        | .command] as $committed_commands
    | $mix.pack_id == "dense-break-live-path"
      and $mix.result == "pass"
      and $mix.evidence_role == "diagnostic"
      and $mix.source_backed == true
      and $mix.source_timing_backed == true
      and $mix.scripted_generation == true
      and $mix.quality_proof == false
      and $mix.human_verdict == "unverified"
      and $mix.evidence_boundary.schema == "riotbox.audio_qa_evidence_boundary.v1"
      and $mix.evidence_boundary.evidence_role == "diagnostic"
      and $mix.evidence_boundary.scripted_generation == true
      and $mix.exact_mixer_proof.kind == "runtime_mix_callback_block_realtime_simulation"
      and $mix.exact_mixer_proof.stateful_sequence == true
      and $mix.exact_mixer_proof.pre_post_limiter_reported == true
      and $mix.exact_mixer_proof.limiter_activity_gated == true
      and $mix.thresholds.max_exact_mix_limited_sample_count == 0
      and $mix.monitor_cycle.review_duration_bars == 4
      and $mix.correlation_scope.kind == "action_contract_only"
      and $mix.correlation_scope.shared_source_fixture == false
      and $mix.correlation_scope.shared_transport_timeline == false
      and $mix.correlation_scope.sample_exact_observer_correlation == false
      and $mix.source.sample_rate == 44100
      and $mix.sample_rate == 48000
      and all(["M", "w", "f", "s", "y", "Y"][]; . as $key | $keys | index($key) != null)
      and all($mix.gesture_transitions[]; . as $gesture
        | $keys | index($gesture.key) != null
          and ($committed_commands | index($gesture.command)) != null
          and ($gesture.counterfactual_limiter | exact_limiter_ok($mix.thresholds.max_exact_mix_limited_sample_count))
          and ($gesture.candidate_limiter | exact_limiter_ok($mix.thresholds.max_exact_mix_limited_sample_count))
          and $gesture.candidate_metrics.rms > $mix.thresholds.min_mix_rms
          and $gesture.qa_candidate_metrics.rms > $mix.thresholds.min_mix_rms
          and $gesture.delta.rms > $gesture.qa_thresholds.min_delta_rms
          and $gesture.delta.peak_abs > $gesture.qa_thresholds.min_delta_peak
          and $gesture.relative_delta_rms > $gesture.qa_thresholds.min_relative_delta_rms)
      and $mix.scene_transition_proof.launch_changed_scene == true
      and $mix.scene_transition_proof.launch_anchor_matches_expected == true
      and $mix.scene_transition_proof.restore_returned_to_pre_jump_scene == true
      and $mix.scene_transition_proof.restore_anchor_matches_expected == true
      and $mix.scene_transition_proof.launch_action_id == ($mix.gesture_transitions[] | select(.key == "y") | .action_id)
      and $mix.scene_transition_proof.restore_action_id == ($mix.gesture_transitions[] | select(.key == "Y") | .action_id)
      and ($mix.monitor_cycle.modes | map(.mode)) == ["source", "blend", "riotbox"]
      and ($mix.monitor_cycle.modes | map(.route)) == ["source_only", "blend", "riotbox_only"]
      and all($mix.monitor_cycle.modes[]; .limiter | exact_limiter_ok($mix.thresholds.max_exact_mix_limited_sample_count))
      and all($mix.performance_stages[]; .limiter | exact_limiter_ok($mix.thresholds.max_exact_mix_limited_sample_count))
      and $mix.legacy_lane_regression.frozen_before_live_fill_slam_scene_gestures == true
      and $mix.legacy_lane_regression.plan.tr909_mode == "break_reinforce"
      and $mix.legacy_lane_regression.tr909.rms > $mix.thresholds.min_isolated_tr909_regression_rms
      and ($mix.legacy_lane_regression.mix_limiter | exact_limiter_ok($mix.thresholds.max_exact_mix_limited_sample_count))
      and ($mix.legacy_lane_regression.damage_limiter | exact_limiter_ok($mix.thresholds.max_exact_mix_limited_sample_count))
      and ($mix.legacy_lane_regression.w30_limiter | exact_limiter_ok($mix.thresholds.max_exact_mix_limited_sample_count))
      and ($mix.legacy_lane_regression.tr909_limiter | exact_limiter_ok($mix.thresholds.max_exact_mix_limited_sample_count))
      and ($mix.legacy_lane_regression.mc202_limiter | exact_limiter_ok($mix.thresholds.max_exact_mix_limited_sample_count))
      and ($mix.failures | length) == 0'
