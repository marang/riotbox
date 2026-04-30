use std::{fs, path::PathBuf};

use super::*;

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
fn writes_feral_grid_jam_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_feral_grid_jam_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"feral-grid-jam""#));
    assert!(events.contains(r#""outcome":"toggle_transport""#));
    assert!(events.contains(r#""outcome":"queue_tr909_fill""#));
    assert!(events.contains(r#""outcome":"queue_mc202_generate_follower""#));
    assert_eq!(events.matches(r#""boundary":"Bar""#).count(), 1);
    assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 1);
}
