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
}

#[test]
fn writes_first_playable_jam_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_first_playable_jam_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"first-playable-jam""#));
    assert!(events.contains(r#""outcome":"queue_capture_bar""#));
    assert!(events.contains(r#""outcome":"queue_w30_audition""#));
    assert!(events.contains(r#""outcome":"promote_last_capture""#));
    assert!(events.contains(r#""outcome":"queue_w30_trigger_pad""#));
    assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 1);
    assert_eq!(events.matches(r#""boundary":"Bar""#).count(), 2);
    assert_eq!(events.matches(r#""boundary":"Beat""#).count(), 1);
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
    assert_eq!(source_timing["primary_warning_code"], "phrase_uncertain");
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
    assert_eq!(source_timing["bpm_estimate"], Value::Null);
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
    assert_eq!(source_timing["primary_downbeat_offset_beats"], 0);
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

fn parse_events(events: &str) -> Vec<Value> {
    events
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("observer event JSON"))
        .collect()
}
