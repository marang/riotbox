use std::{fs, path::PathBuf};

use super::*;
use serde_json::Value;

#[test]
fn parses_required_probe_args() {
    let args = Args::parse([
        "--probe".into(),
        "recipe2-mc202".into(),
        "--observer".into(),
        "events.ndjson".into(),
    ])
    .expect("parse args");

    assert_eq!(args.probe, "recipe2-mc202");
    assert_eq!(args.observer_path, PathBuf::from("events.ndjson"));
    assert!(!args.show_help);
}

#[test]
fn observer_boundary_helper_uses_selected_nonzero_source_downbeat_phase() {
    use riotbox_core::{
        ids::SourceId,
        source_graph::{
            BarSpan, BeatPoint, DecodeProfile, GraphProvenance, MeterHint, SourceDescriptor,
            SourceGraph, TimingHypothesis, TimingHypothesisKind, TimingQuality,
        },
    };

    let mut shell = probe_shell("observer-phase-test");
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("observer-phase-source"),
            path: "observer-phase.wav".into(),
            content_hash: "observer-phase-hash".into(),
            duration_seconds: 20.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::NormalizedStereo,
        },
        GraphProvenance {
            sidecar_version: "test".into(),
            provider_set: vec!["test".into()],
            generated_at: "2026-07-16T00:00:00Z".into(),
            source_hash: "observer-phase-hash".into(),
            analysis_seed: 23,
            run_notes: None,
        },
    );
    graph.timing.primary_hypothesis_id = Some("phase-three".into());
    graph.timing.hypotheses = vec![TimingHypothesis {
        hypothesis_id: "phase-three".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: 120.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.95,
        score: 0.95,
        beat_grid: (4..=43)
            .map(|beat_index| BeatPoint {
                beat_index,
                time_seconds: (beat_index - 4) as f32 * 0.5,
                confidence: 0.95,
            })
            .collect(),
        bar_grid: (1..=10)
            .map(|bar_index| BarSpan {
                bar_index,
                start_seconds: (bar_index - 1) as f32 * 2.0,
                end_seconds: bar_index as f32 * 2.0,
                downbeat_confidence: 0.95,
                phrase_index: Some((bar_index - 1) / 4 + 1),
            })
            .collect(),
        phrase_grid: Vec::new(),
        anchors: Vec::new(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::High,
        warnings: Vec::new(),
        provenance: vec!["test:phase-three".into()],
    }];
    shell.app.source_graph = Some(graph);

    let position = source_aware_grid_position(&shell, 20);

    assert_eq!(position.beat_cursor, 20);
    assert_eq!(position.bar_index, 5);
    assert_eq!(position.phrase_index, 2);
}

#[test]
fn writes_recipe2_mc202_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_recipe2_mc202_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""schema":"riotbox.user_session_observer.v1""#));
    assert!(events.contains(r#""capture_context":"headless_probe""#));
    assert!(events.contains(r#""snapshot":{"#));
    assert!(events.contains(r#""transport":{"#));
    assert!(events.contains(r#""queue":{"#));
    assert!(events.contains(r#""runtime":{"#));
    assert!(events.contains(r#""recovery":{"#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_follower""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_answer""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_pressure""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_instigator""#));
    assert!(events.contains(r#""outcome":"queue_mc202_mutate_phrase""#));
    assert!(events.contains(r#""outcome":"raise_mc202_touch""#));
    assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 5);

    let parsed = parse_events(&events);
    let first_commit = parsed
        .iter()
        .find(|event| event["event"] == "transport_commit")
        .expect("first phrase commit");
    let committed = &first_commit["committed"][0];
    assert_eq!(committed["beat_index"], 16);
    assert_eq!(committed["bar_index"], 5);
    assert_eq!(committed["phrase_index"], 2);
    assert_eq!(first_commit["snapshot"]["transport"]["beat_index"], 16);
    assert_eq!(first_commit["snapshot"]["transport"]["bar_index"], 5);
    assert_eq!(first_commit["snapshot"]["transport"]["phrase_index"], 2);
}

#[test]
fn writes_first_playable_jam_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_first_playable_jam_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    let parsed = parse_events(&events);
    let start = parsed
        .iter()
        .find(|event| event["event"] == "observer_started")
        .expect("observer start");
    assert_eq!(start["launch"]["probe"], "first-playable-jam");
    assert_eq!(
        start["launch"]["source_path"],
        "synthetic-first-playable-source.wav"
    );
    assert_eq!(
        start["snapshot"]["source_timing"]["source_id"],
        "src-first-playable-jam"
    );
    assert_eq!(start["snapshot"]["source_timing"]["beat_count"], 64);
    assert_eq!(start["snapshot"]["source_timing"]["bar_count"], 16);
    assert_eq!(start["snapshot"]["source_timing"]["phrase_count"], 4);
    assert_eq!(start["snapshot"]["source_map"]["mode"], "bar grid");
    assert_eq!(
        start["snapshot"]["source_map"]["capture_range_available"],
        true
    );

    for (key, outcome) in [
        ("c", "queue_capture_bar"),
        ("o", "queue_w30_audition"),
        ("p", "promote_last_capture"),
        ("M", "queue_source_monitor_mode"),
        ("w", "queue_w30_trigger_pad"),
        ("f", "queue_tr909_fill"),
        ("s", "queue_tr909_slam"),
        ("y", "queue_scene_select"),
        ("Y", "queue_scene_restore"),
    ] {
        assert_eq!(key_outcome(&parsed, key)["outcome"], outcome);
    }

    let monitor_key = key_outcome(&parsed, "M");
    assert_eq!(
        monitor_key["snapshot"]["runtime"]["source_monitor_mode"],
        "blend"
    );
    assert_eq!(
        monitor_key["snapshot"]["runtime"]["source_monitor_audio_route"],
        "blend"
    );
    assert_eq!(monitor_key["snapshot"]["queue"]["pending_count"], 0);
    assert_eq!(monitor_key["snapshot"]["transport"]["beat_index"], 20);
    assert_eq!(monitor_key["snapshot"]["transport"]["bar_index"], 6);
    assert_eq!(monitor_key["snapshot"]["transport"]["phrase_index"], 2);
    assert_eq!(
        monitor_key["snapshot"]["transport"]["current_scene"],
        "scene-01-break"
    );

    let (monitor_commit, monitor_ref) =
        committed_command(&parsed, "source_monitor.set_mode", "Immediate");
    assert_eq!(monitor_commit["timestamp_ms"], 650);
    assert_eq!(
        monitor_commit["committed"]
            .as_array()
            .expect("monitor refs")
            .len(),
        1
    );
    assert_commit_position(monitor_ref, 20, 6, 2);

    let (w30_commit, w30_ref) = committed_command(&parsed, "w30.trigger_pad", "Beat");
    assert_commit_position(w30_ref, 21, 6, 2);
    assert_eq!(
        w30_commit["snapshot"]["runtime"]["w30_preview_target"],
        "bank-a / pad-01 | cap-01"
    );

    let (fill_commit, fill_ref) = committed_command(&parsed, "tr909.fill_next", "Bar");
    assert_commit_position(fill_ref, 24, 7, 2);
    assert_eq!(fill_commit["snapshot"]["runtime"]["tr909_mode"], "fill");
    assert_eq!(
        fill_commit["snapshot"]["runtime"]["tr909_routing"],
        "drum_bus_support"
    );

    let (_, slam_ref) = committed_command(&parsed, "tr909.set_slam", "Beat");
    assert_commit_position(slam_ref, 25, 7, 2);

    let (scene_commit, scene_ref) = committed_command(&parsed, "scene.launch", "Bar");
    assert_commit_position(scene_ref, 36, 10, 3);
    assert_eq!(
        scene_commit["snapshot"]["scene"]["active_scene"],
        "scene-02-drop"
    );
    assert_eq!(
        scene_commit["snapshot"]["scene"]["last_movement"]["from_scene"],
        "scene-01-break"
    );
    assert_eq!(
        scene_commit["snapshot"]["scene"]["last_movement"]["to_scene"],
        "scene-02-drop"
    );
    assert_eq!(
        scene_commit["snapshot"]["scene"]["source_monitor"]["source_anchor_seconds"],
        16.0
    );

    let (restore_commit, restore_ref) = committed_command(&parsed, "scene.restore", "Bar");
    assert_commit_position_for_scene(restore_ref, 40, 11, 3, "scene-02-drop");
    assert_eq!(restore_commit["timestamp_ms"], 1_600);
    assert_eq!(
        restore_commit["snapshot"]["scene"]["active_scene"],
        "scene-01-break"
    );
    assert_eq!(
        restore_commit["snapshot"]["scene"]["last_movement"]["kind"],
        "restore"
    );
    assert_eq!(
        restore_commit["snapshot"]["scene"]["last_movement"]["from_scene"],
        "scene-02-drop"
    );
    assert_eq!(
        restore_commit["snapshot"]["scene"]["last_movement"]["to_scene"],
        "scene-01-break"
    );

    let committed_beats = parsed
        .iter()
        .filter(|event| event["event"] == "transport_commit")
        .flat_map(|event| {
            event["committed"]
                .as_array()
                .expect("committed refs")
                .iter()
                .map(|committed| committed["beat_index"].as_u64().expect("commit beat"))
        })
        .collect::<Vec<_>>();
    assert_eq!(committed_beats, [16, 20, 20, 20, 21, 24, 25, 36, 40]);
    assert!(committed_beats.windows(2).all(|pair| pair[0] <= pair[1]));
    let committed_scenes = parsed
        .iter()
        .filter(|event| event["event"] == "transport_commit")
        .flat_map(|event| event["committed"].as_array().expect("committed refs"))
        .map(|committed| committed["scene_id"].as_str().expect("commit scene"))
        .collect::<Vec<_>>();
    assert_eq!(
        committed_scenes,
        [
            "scene-01-break",
            "scene-01-break",
            "scene-01-break",
            "scene-01-break",
            "scene-01-break",
            "scene-01-break",
            "scene-01-break",
            "scene-01-break",
            "scene-02-drop",
        ]
    );

    let final_snapshot = &parsed.last().expect("final event")["snapshot"];
    assert_eq!(final_snapshot["queue"]["pending_count"], 0);
    assert_eq!(final_snapshot["queue"]["session_log_count"], 9);
    assert_eq!(
        final_snapshot["capture"]["source_window"]["source_id"],
        "src-first-playable-jam"
    );
    assert_eq!(final_snapshot["transport"]["beat_index"], 40);
    assert_eq!(final_snapshot["transport"]["bar_index"], 11);
    assert_eq!(final_snapshot["transport"]["phrase_index"], 3);
    assert_eq!(
        final_snapshot["transport"]["current_scene"],
        "scene-01-break"
    );

    assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 1);
    assert_eq!(events.matches(r#""boundary":"Bar""#).count(), 5);
    assert_eq!(events.matches(r#""boundary":"Beat""#).count(), 2);
    assert_eq!(events.matches(r#""boundary":"Immediate""#).count(), 1);
}

#[test]
fn writes_source_timing_confirmation_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_source_timing_confirmation_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"source-timing-confirmation""#));
    assert!(events.contains(r#""outcome":"confirm_source_timing_grid""#));
    assert!(events.contains(r#""boundary":"Immediate""#));

    let parsed = parse_events(&events);
    let start = parsed
        .iter()
        .find(|event| event["event"] == "observer_started")
        .expect("observer start");
    let start_timing = &start["snapshot"]["source_timing"];
    assert_eq!(start_timing["source_id"], "src-source-timing-confirmation");
    assert_eq!(start_timing["degraded_policy"], "manual_confirm");
    assert_eq!(start_timing["cue"], "needs confirm");
    assert_eq!(start_timing["grid_use"], "manual_confirm_only");
    assert_eq!(start_timing["primary_warning_code"], "ambiguous_downbeat");
    assert_eq!(start_timing["grid_confirmed"], false);

    let key = parsed
        .iter()
        .find(|event| event["event"] == "key_outcome" && event["key"] == "C")
        .expect("confirm key outcome");
    assert_eq!(key["outcome"], "confirm_source_timing_grid");
    assert_eq!(key["status"], "confirmed source timing grid");
    assert_eq!(key["snapshot"]["queue"]["pending_count"], 0);
    assert_eq!(key["snapshot"]["queue"]["queue_history_count"], 1);
    assert_eq!(
        key["snapshot"]["queue"]["recent_history"][0]["command"],
        "source_timing.confirm_grid"
    );
    assert_eq!(
        key["snapshot"]["queue"]["recent_history"][0]["status"],
        "Committed"
    );
    assert_eq!(
        key["snapshot"]["queue"]["recent_history"][0]["committed_at"],
        100
    );

    let confirmed_timing = &key["snapshot"]["source_timing"];
    assert_eq!(confirmed_timing["cue"], "needs confirm");
    assert_eq!(confirmed_timing["degraded_policy"], "manual_confirm");
    assert_eq!(confirmed_timing["grid_confirmed"], true);
    assert_eq!(
        confirmed_timing["confirmed_grid_source_id"],
        "src-source-timing-confirmation"
    );
    assert_eq!(
        confirmed_timing["confirmed_grid_hypothesis_id"],
        "probe-primary"
    );
    assert_eq!(confirmed_timing["confirmed_grid_at"], 100);

    let commit = parsed
        .iter()
        .find(|event| event["event"] == "transport_commit")
        .expect("immediate commit event");
    assert_eq!(commit["committed"][0]["boundary"], "Immediate");
    assert_eq!(commit["snapshot"]["queue"]["session_log_count"], 1);
    assert_eq!(commit["snapshot"]["source_timing"]["grid_confirmed"], true);
}

#[test]
fn writes_p014_scene_movement_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_p014_scene_movement_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"p014-scene-movement""#));
    assert!(events.contains(r#""outcome":"queue_scene_select""#));
    assert!(events.contains(r#""command":"scene.launch""#));
    assert!(events.contains(r#""boundary":"Bar""#));

    let parsed = parse_events(&events);
    let commit = parsed
        .iter()
        .find(|event| event["event"] == "transport_commit")
        .expect("scene commit event");
    let scene = &commit["snapshot"]["scene"];
    assert_eq!(scene["active_scene"], "scene-02-drop");
    assert_eq!(scene["last_movement"]["kind"], "launch");
    assert_eq!(scene["last_movement"]["direction"], "rise");
    assert_eq!(scene["last_movement"]["tr909_intent"], "drive");
    assert_eq!(scene["last_movement"]["mc202_intent"], "lift");
    assert_eq!(scene["last_movement"]["from_scene"], "scene-01-break");
    assert_eq!(scene["last_movement"]["to_scene"], "scene-02-drop");
    assert_eq!(scene["last_movement"]["committed_bar_index"], 10);
    assert_eq!(scene["last_movement"]["committed_phrase_index"], 3);
    assert_eq!(commit["committed"][0]["beat_index"], 36);
    assert_eq!(commit["committed"][0]["bar_index"], 10);
    assert_eq!(commit["committed"][0]["phrase_index"], 3);
    assert_eq!(
        scene["arrangement_contract"]["can_use_source_locked_scene_movement"],
        true
    );
    assert_eq!(
        scene["arrangement_contract"]["bounded_extension"],
        "manual_scene_chain_ready"
    );
    assert_eq!(
        scene["arrangement_contract"]["allows_manual_scene_chain_extension"],
        true
    );
    assert_eq!(
        scene["arrangement_contract"]["allows_automatic_scene_chain_scheduler"],
        false
    );
    assert_eq!(scene["source_monitor"]["source_anchor_seconds"], 16.0);
    assert_eq!(
        scene["source_monitor"]["source_anchor_position_beats"],
        36.0
    );
}

#[test]
fn writes_stage_style_jam_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_stage_style_jam_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"stage-style-jam""#));
    assert!(events.contains(r#""outcome":"queue_capture_bar""#));
    assert!(events.contains(r#""outcome":"queue_w30_trigger_pad""#));
    assert!(events.contains(r#""outcome":"queue_tr909_fill""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_follower""#));
    assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 2);
    assert_eq!(events.matches(r#""boundary":"Bar""#).count(), 3);
    assert_eq!(events.matches(r#""boundary":"Beat""#).count(), 1);
}

#[test]
fn writes_stage_style_restore_diversity_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_stage_style_restore_diversity_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"stage-style-restore-diversity""#));
    assert!(events.contains(r#""outcome":"queue_capture_bar""#));
    assert!(events.contains(r#""outcome":"queue_w30_audition""#));
    assert!(events.contains(r#""outcome":"promote_last_capture""#));
    assert!(events.contains(r#""outcome":"queue_w30_trigger_pad""#));
    assert!(events.contains(r#""outcome":"queue_tr909_fill""#));
    assert!(events.contains(r#""outcome":"queue_tr909_reinforce""#));
    assert!(events.contains(r#""outcome":"queue_tr909_scene_lock""#));
    assert!(events.contains(r#""outcome":"queue_tr909_release""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_follower""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_answer""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_pressure""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_instigator""#));
    assert!(events.contains(r#""outcome":"queue_mc202_mutate_phrase""#));
    assert!(events.contains(r#""outcome":"raise_mc202_touch""#));
    assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 9);
    assert_eq!(events.matches(r#""boundary":"Bar""#).count(), 3);
    assert_eq!(events.matches(r#""boundary":"Beat""#).count(), 1);
}

#[test]
fn writes_interrupted_session_recovery_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_interrupted_session_recovery_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"interrupted-session-recovery""#));
    assert!(events.contains(r#""mode":"load""#));
    assert!(events.contains(r#""kind":"orphan temp file""#));
    assert!(events.contains(r#""status":"invalid session JSON""#));
    assert!(events.contains(r#""kind":"autosave file""#));
    assert!(events.contains(r#""trust":"RecoverableClue""#));
    assert!(events.contains(r#""manual_choice_dry_run":{"#));
    assert!(events.contains(r#""replay_family":"families"#));
    assert!(events.contains(r#""selected_for_restore":false"#));
    assert!(
        temp.path()
            .join("interrupted-session-recovery/session.json")
            .is_file()
    );
    assert!(
        temp.path()
            .join("interrupted-session-recovery/.session.json.tmp-1776359400")
            .is_file()
    );
    assert!(
        temp.path()
            .join("interrupted-session-recovery/session.autosave.2026-04-30T171500Z.json")
            .is_file()
    );
}

#[test]
fn writes_missing_target_recovery_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_missing_target_recovery_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"missing-target-recovery""#));
    assert!(events.contains(r#""mode":"load""#));
    assert!(events.contains(r#""kind":"normal session path""#));
    assert!(events.contains(r#""status":"missing""#));
    assert!(events.contains(r#""trust":"MissingTarget""#));
    assert!(events.contains(r#""kind":"autosave file""#));
    assert!(events.contains(r#""trust":"RecoverableClue""#));
    assert!(events.contains(r#""manual_choice_dry_run":{"#));
    assert!(events.contains(r#""replay_family":"families"#));
    assert!(events.contains(r#""selected_for_restore":false"#));
    assert!(
        !temp
            .path()
            .join("missing-target-recovery/session.json")
            .exists()
    );
    assert!(
        temp.path()
            .join("missing-target-recovery/session.autosave.2026-04-30T172000Z.json")
            .is_file()
    );
}

#[test]
fn writes_feral_grid_jam_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_feral_grid_jam_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"feral-grid-jam""#));
    assert!(events.contains(r#""source_timing":{"#));
    assert!(events.contains(r#""source_id":"src-feral-grid-probe""#));
    assert!(events.contains(r#""quality":"medium""#));
    assert!(events.contains(r#""degraded_policy":"cautious""#));
    assert!(events.contains(r#""cue":"listen first""#));
    let source_timing = first_source_timing_snapshot(&events);
    let source_map = first_source_map_snapshot(&events);
    assert_eq!(source_timing["primary_warning_code"], "phrase_uncertain");
    assert_eq!(source_map["mode"], "time fallback");
    assert_eq!(source_map["capture_range_available"], false);
    assert_eq!(source_map["capture_range_row"], ".".repeat(32));
    assert_eq!(source_timing["anchor_evidence"]["primary_anchor_count"], 0);
    assert_eq!(
        source_timing["anchor_evidence"]["primary_kick_anchor_count"],
        0
    );
    assert_eq!(
        source_timing["anchor_evidence"]["primary_backbeat_anchor_count"],
        0
    );
    assert_eq!(
        source_timing["anchor_evidence"]["primary_transient_anchor_count"],
        0
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_residual_count"],
        0
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_preview"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert!(events.contains(r#""outcome":"toggle_transport""#));
    assert!(events.contains(r#""outcome":"queue_tr909_fill""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_follower""#));
    assert_eq!(events.matches(r#""boundary":"Bar""#).count(), 1);
    assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 1);
}

#[test]
fn writes_feral_grid_fallback_jam_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_feral_grid_fallback_jam_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"feral-grid-jam-fallback""#));
    assert!(events.contains(r#""source_timing":{"#));
    assert!(events.contains(r#""source_id":"src-feral-grid-probe""#));
    assert!(events.contains(r#""quality":"low""#));
    assert!(events.contains(r#""degraded_policy":"fallback_grid""#));
    assert!(events.contains(r#""cue":"fallback grid""#));
    assert!(events.contains(r#""beat_status":"unknown""#));
    assert!(events.contains(r#""downbeat_status":"unknown""#));
    assert!(events.contains(r#""phrase_status":"unknown""#));
    let source_timing = first_source_timing_snapshot(&events);
    let source_map = first_source_map_snapshot(&events);
    assert_eq!(source_timing["bpm_estimate"], Value::Null);
    assert_eq!(source_map["mode"], "time fallback");
    assert_eq!(source_map["capture_range_available"], false);
    assert_eq!(source_map["capture_range_row"], ".".repeat(32));
    assert_eq!(source_timing["primary_downbeat_offset_beats"], Value::Null);
    assert_eq!(
        source_timing["primary_warning_code"],
        "low_timing_confidence"
    );
    assert_eq!(source_timing["anchor_evidence"]["primary_anchor_count"], 0);
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_residual_count"],
        0
    );
    assert!(events.contains(r#""low_timing_confidence""#));
    assert!(events.contains(r#""weak_kick_anchor""#));
    assert!(events.contains(r#""outcome":"toggle_transport""#));
    assert!(events.contains(r#""outcome":"queue_tr909_fill""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_follower""#));
    assert_eq!(events.matches(r#""boundary":"Bar""#).count(), 1);
    assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 1);
}

#[test]
fn writes_feral_grid_locked_jam_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_feral_grid_locked_jam_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"feral-grid-jam-locked""#));
    assert!(events.contains(r#""source_timing":{"#));
    assert!(events.contains(r#""source_id":"src-feral-grid-probe""#));
    assert!(events.contains(r#""quality":"high""#));
    assert!(events.contains(r#""degraded_policy":"locked""#));
    assert!(events.contains(r#""cue":"grid locked""#));
    assert!(events.contains(r#""beat_status":"grid""#));
    assert!(events.contains(r#""beat_count":16"#));
    assert!(events.contains(r#""downbeat_status":"bar_locked""#));
    assert!(events.contains(r#""bar_count":4"#));
    assert!(events.contains(r#""phrase_status":"phrase_locked""#));
    assert!(events.contains(r#""phrase_count":1"#));
    let source_timing = first_source_timing_snapshot(&events);
    let source_map = first_source_map_snapshot(&events);
    assert_eq!(source_timing["primary_downbeat_offset_beats"], 0);
    assert_eq!(source_map["mode"], "bar grid");
    assert_eq!(source_map["trust_label"], "grid locked");
    assert_eq!(source_map["capture_range_available"], true);
    assert!(
        source_map["capture_range_row"]
            .as_str()
            .expect("capture range row")
            .contains('[')
    );
    assert_eq!(source_timing["primary_warning_code"], Value::Null);
    assert_eq!(source_timing["anchor_evidence"]["primary_anchor_count"], 16);
    assert_eq!(
        source_timing["anchor_evidence"]["primary_kick_anchor_count"],
        4
    );
    assert_eq!(
        source_timing["anchor_evidence"]["primary_backbeat_anchor_count"],
        8
    );
    assert_eq!(
        source_timing["anchor_evidence"]["primary_transient_anchor_count"],
        4
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_residual_count"],
        2
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_max_abs_offset_ms"],
        6.0
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_preview"][0]["subdivision"],
        "eighth"
    );
    assert!(events.contains(r#""warning_codes":[]"#));
    assert!(events.contains(r#""outcome":"toggle_transport""#));
    assert!(events.contains(r#""outcome":"queue_tr909_fill""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_follower""#));
    assert_eq!(events.matches(r#""boundary":"Bar""#).count(), 1);
    assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 1);
}

fn first_source_timing_snapshot(events: &str) -> Value {
    events
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|event| event["snapshot"]["source_timing"].as_object().cloned())
        .map(Value::Object)
        .next()
        .expect("source timing snapshot")
}

fn first_source_map_snapshot(events: &str) -> Value {
    events
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|event| event["snapshot"]["source_map"].as_object().cloned())
        .map(Value::Object)
        .next()
        .expect("source map snapshot")
}

fn parse_events(events: &str) -> Vec<Value> {
    events
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("observer event JSON"))
        .collect()
}

fn key_outcome<'a>(events: &'a [Value], key: &str) -> &'a Value {
    events
        .iter()
        .find(|event| event["event"] == "key_outcome" && event["key"] == key)
        .unwrap_or_else(|| panic!("missing key outcome for {key}"))
}

fn committed_command<'a>(
    events: &'a [Value],
    command: &str,
    boundary: &str,
) -> (&'a Value, &'a Value) {
    for event in events
        .iter()
        .filter(|event| event["event"] == "transport_commit")
    {
        let history = event["snapshot"]["queue"]["recent_history"]
            .as_array()
            .expect("recent queue history");
        for committed in event["committed"].as_array().expect("committed refs") {
            let action_id = &committed["action_id"];
            if committed["boundary"] == boundary
                && history.iter().any(|action| {
                    action["id"] == *action_id
                        && action["command"] == command
                        && action["status"] == "Committed"
                })
            {
                return (event, committed);
            }
        }
    }

    panic!("missing committed {command} action at {boundary} boundary")
}

fn assert_commit_position(committed: &Value, beat: u64, bar: u64, phrase: u64) {
    assert_commit_position_for_scene(committed, beat, bar, phrase, "scene-01-break");
}

fn assert_commit_position_for_scene(
    committed: &Value,
    beat: u64,
    bar: u64,
    phrase: u64,
    scene: &str,
) {
    assert_eq!(committed["beat_index"], beat);
    assert_eq!(committed["bar_index"], bar);
    assert_eq!(committed["phrase_index"], phrase);
    assert_eq!(committed["scene_id"], scene);
}
