use std::fs;

use super::*;
use serde_json::Value;

#[test]
fn writes_source_transport_map_capture_observer_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");

    write_source_transport_map_capture_observer(&path).expect("write observer");

    let events = fs::read_to_string(path).expect("read observer");
    assert!(events.contains(r#""probe":"source-transport-map-capture""#));
    assert!(events.contains(r#""outcome":"confirm_source_timing_grid""#));
    assert!(events.contains(r#""outcome":"navigate_source_map_next_bar""#));
    assert!(events.contains(r#""outcome":"queue_capture_bar""#));
    assert!(events.contains(r#""outcome":"queue_w30_audition""#));
    assert!(events.contains(r#""outcome":"promote_last_capture""#));
    assert!(events.contains(r#""outcome":"queue_w30_trigger_pad""#));

    let parsed = parse_events(&events);
    let start = parsed
        .iter()
        .find(|event| event["event"] == "observer_started")
        .expect("observer start");
    assert_eq!(
        start["snapshot"]["runtime"]["source_monitor_audio_route"],
        "source_only"
    );
    assert_eq!(
        start["snapshot"]["source_map"]["trust_label"],
        "needs confirm"
    );
    assert_eq!(
        start["snapshot"]["source_map"]["capture_range_available"],
        false
    );

    let confirm = parsed
        .iter()
        .find(|event| event["event"] == "key_outcome" && event["key"] == "C")
        .expect("confirm key");
    assert_eq!(confirm["snapshot"]["source_timing"]["grid_confirmed"], true);
    assert_eq!(confirm["snapshot"]["source_map"]["mode"], "bar grid");
    assert_eq!(
        confirm["snapshot"]["source_map"]["capture_range_available"],
        true
    );

    let seek = parsed
        .iter()
        .find(|event| event["event"] == "key_outcome" && event["key"] == "Right")
        .expect("source map seek key");
    assert_eq!(seek["snapshot"]["transport"]["position_beats"], 4.0);
    assert_eq!(
        seek["snapshot"]["queue"]["recent_history"][0]["command"],
        "transport.seek"
    );

    let capture_commit = parsed
        .iter()
        .find(|event| {
            event["event"] == "transport_commit"
                && event["snapshot"]["capture"]["source_window_available"] == true
        })
        .expect("source-window capture commit");
    assert_eq!(
        capture_commit["snapshot"]["capture"]["source_window"]["source_id"],
        "src-source-transport-map-capture"
    );

    let final_commit = parsed
        .iter()
        .rev()
        .find(|event| event["event"] == "transport_commit")
        .expect("final commit");
    assert_eq!(
        final_commit["snapshot"]["runtime"]["w30_preview_mode"],
        "live_recall"
    );
    assert!(
        final_commit["snapshot"]["runtime"]["w30_preview_target"]
            .as_str()
            .expect("w30 target")
            .contains("cap-01")
    );
}

fn parse_events(events: &str) -> Vec<Value> {
    events
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid observer event"))
        .collect()
}
