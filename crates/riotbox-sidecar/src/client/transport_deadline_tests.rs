use super::{ClientError, SidecarOperation, SidecarTimeoutPolicy, StdioSidecarClient};
use riotbox_core::{
    ids::SourceId,
    source_graph::{DecodeProfile, SourceDescriptor},
};
use std::{
    fs,
    time::{Duration, Instant},
};

fn peer(body: &str) -> (tempfile::TempDir, StdioSidecarClient) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic_peer.py");
    fs::write(&path, format!("import json, sys, time\n{body}")).unwrap();
    let client = StdioSidecarClient::spawn_python(&path).unwrap();
    (dir, client)
}

const PING: &str = "request = json.loads(sys.stdin.readline())\nprint(json.dumps({'type': 'pong', 'request_id': request['request_id'], 'protocol_version': '0.1', 'sidecar_version': 'synthetic'}), flush=True)\n";

fn oversized_metadata() -> SourceDescriptor {
    SourceDescriptor {
        source_id: SourceId::from("synthetic-deadline-probe"),
        // Public API metadata only: this is not an actual file path or source.
        path: "x".repeat(1024 * 1024),
        content_hash: "synthetic".into(),
        duration_seconds: 0.1,
        sample_rate: 48_000,
        channel_count: 1,
        decode_profile: DecodeProfile::Native,
    }
}

fn assert_analysis_timeout(error: ClientError, expected: Duration) {
    match error {
        ClientError::ResponseTimeout { operation, timeout } => {
            assert_eq!(operation, SidecarOperation::Analysis);
            assert_eq!(timeout, expected);
        }
        other => panic!("expected analysis transport deadline, got {other}"),
    }
}

#[test]
fn deadline_covers_pipe_backpressure() {
    // Even the old broken implementation releases after two seconds; no mock
    // remains indefinitely stuck and interpreter startup is outside the budget.
    let (_dir, mut client) = peer(&format!("{PING}time.sleep(2)\n"));
    client.ping().unwrap();
    let budget = Duration::from_millis(50);
    let mut client = client.with_timeout_policy(SidecarTimeoutPolicy {
        control: budget,
        analysis: budget,
    });
    let start = Instant::now();
    let error = client
        .build_source_graph_stub(oversized_metadata(), 0)
        .unwrap_err();
    assert_analysis_timeout(error, budget);
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "write was released only by mock exit"
    );
    assert!(
        client.child.try_wait().unwrap().is_some(),
        "timeout must reap the unusable peer"
    );
    assert!(matches!(
        client.ping(),
        Err(ClientError::TransportUnavailable)
    ));
}

#[test]
fn response_wait_does_not_restart_the_write_budget() {
    let (_dir, mut client) = peer(&format!(
        "{PING}time.sleep(0.2)\nrequest = json.loads(sys.stdin.readline())\ntime.sleep(0.2)\nprint(json.dumps({{'type': 'error', 'request_id': request['request_id'], 'code': 'fixture_complete', 'message': 'bounded late reply', 'retryable': False}}), flush=True)\n"
    ));
    client.ping().unwrap();
    let budget = Duration::from_millis(300);
    let mut client = client.with_timeout_policy(SidecarTimeoutPolicy {
        control: Duration::from_secs(1),
        analysis: budget,
    });
    assert_analysis_timeout(
        client
            .build_source_graph_stub(oversized_metadata(), 0)
            .unwrap_err(),
        budget,
    );
}

#[test]
fn partial_response_still_expires() {
    let (_dir, mut client) = peer(&format!(
        "{PING}request = json.loads(sys.stdin.readline())\nsys.stdout.write('{{\"type\":\"error\"')\nsys.stdout.flush()\ntime.sleep(2)\n"
    ));
    client.ping().unwrap();
    let budget = Duration::from_millis(50);
    let mut client = client.with_response_timeout(budget);
    let mut source = oversized_metadata();
    source.path = "synthetic-metadata".into();
    assert_analysis_timeout(
        client.build_source_graph_stub(source, 0).unwrap_err(),
        budget,
    );
}

#[test]
fn final_valid_response_without_newline_retains_legacy_behavior() {
    let (_dir, mut client) = peer(
        "request = json.loads(sys.stdin.readline())\nsys.stdout.write(json.dumps({'type': 'pong', 'request_id': request['request_id'], 'protocol_version': '0.1', 'sidecar_version': 'synthetic'}))\nsys.stdout.flush()\n",
    );
    assert_eq!(client.ping().unwrap().protocol_version, "0.1");
}

#[test]
fn mismatched_response_id_closes_the_desynchronized_peer() {
    let (_dir, mut client) = peer(
        "request = json.loads(sys.stdin.readline())\nprint(json.dumps({'type': 'pong', 'request_id': 'wrong-id', 'protocol_version': '0.1', 'sidecar_version': 'synthetic'}), flush=True)\ntime.sleep(2)\n",
    );
    assert!(matches!(
        client.ping(),
        Err(ClientError::RequestIdMismatch { .. })
    ));
    assert!(client.child.try_wait().unwrap().is_some());
    assert!(matches!(
        client.ping(),
        Err(ClientError::TransportUnavailable)
    ));
}

#[test]
fn complete_sidecar_error_keeps_a_synchronized_peer_usable() {
    let (_dir, mut client) = peer(&format!(
        "{PING}request = json.loads(sys.stdin.readline())\nprint(json.dumps({{'type': 'error', 'request_id': request['request_id'], 'code': 'source_unavailable', 'message': 'synthetic', 'retryable': False}}), flush=True)\n{PING}"
    ));
    client.ping().unwrap();
    let mut source = oversized_metadata();
    source.path = "synthetic-metadata".into();
    assert!(matches!(
        client.build_source_graph_stub(source, 0),
        Err(ClientError::Sidecar(_))
    ));
    assert_eq!(client.ping().unwrap().protocol_version, "0.1");
}

#[test]
fn zero_budget_fails_as_a_timeout_not_a_panic() {
    let (_dir, client) = peer(&format!("{PING}time.sleep(2)\n"));
    let mut client = client.with_response_timeout(Duration::ZERO);
    assert!(
        matches!(client.ping(), Err(ClientError::ResponseTimeout { operation: SidecarOperation::Control, timeout }) if timeout == Duration::ZERO)
    );
    assert!(client.child.try_wait().unwrap().is_some());
}

#[test]
fn coalesced_frames_preserve_the_next_complete_frame() {
    let (_dir, mut client) = peer(
        "request = json.loads(sys.stdin.readline())\nsys.stdout.write(json.dumps({'type': 'pong', 'request_id': request['request_id'], 'protocol_version': '0.1', 'sidecar_version': 'synthetic'}) + '\\n' + json.dumps({'type': 'error', 'request_id': 'req-2', 'code': 'coalesced_frame', 'message': 'synthetic', 'retryable': False}) + '\\n')\nsys.stdout.flush()\nsys.stdin.readline()\n",
    );
    client.ping().unwrap();
    let mut source = oversized_metadata();
    source.path = "synthetic-metadata".into();
    assert!(
        matches!(client.build_source_graph_stub(source, 0), Err(ClientError::Sidecar(payload)) if payload.code == "coalesced_frame")
    );
}

#[test]
fn unicode_response_crosses_read_chunks_without_corruption() {
    let (_dir, mut client) = peer(
        "sys.stdout.reconfigure(encoding='utf-8')\nrequest = json.loads(sys.stdin.readline())\nprint(json.dumps({'type': 'error', 'request_id': request['request_id'], 'code': 'synthetic_unicode', 'message': 'ä' * 10000, 'retryable': False}, ensure_ascii=False), flush=True)\n",
    );
    assert!(
        matches!(client.ping(), Err(ClientError::Sidecar(payload)) if payload.message == "ä".repeat(10000))
    );
}
