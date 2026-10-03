use super::{ClientError, SIDECAR_MAX_RESPONSE_BYTES_V1, SidecarOperation, StdioSidecarClient};
use riotbox_core::{
    ids::SourceId,
    source_graph::{DecodeProfile, SourceDescriptor},
};
use std::fs;

fn peer(body: &str) -> (tempfile::TempDir, StdioSidecarClient) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("response_limit_peer.py");
    fs::write(&path, format!("import json, sys, time\n{body}")).unwrap();
    let client = StdioSidecarClient::spawn_python(&path).unwrap();
    (dir, client)
}

fn padded_pong(bytes: usize, ending: &str) -> String {
    format!(
        r#"
request = json.loads(sys.stdin.readline())
body = json.dumps({{'type': 'pong', 'request_id': request['request_id'], 'protocol_version': '0.1', 'sidecar_version': 'synthetic'}}).encode()
ending = {ending}.encode()
sys.stdout.buffer.write(body + b' ' * ({bytes} - len(body) - len(ending)) + ending)
sys.stdout.buffer.flush()
"#
    )
}

fn assert_rejected_and_closed(
    client: &mut StdioSidecarClient,
    error: ClientError,
    expected_operation: SidecarOperation,
) {
    assert!(
        matches!(error, ClientError::ResponseTooLarge { operation, limit_bytes }
            if operation == expected_operation && limit_bytes == SIDECAR_MAX_RESPONSE_BYTES_V1),
        "expected typed response limit, got {error}"
    );
    assert!(client.child.try_wait().unwrap().is_some());
    assert!(matches!(
        client.ping(),
        Err(ClientError::TransportUnavailable)
    ));
}

#[test]
fn over_limit_valid_response_is_rejected_with_or_without_newline() {
    for ending in ["'\\n'", "'\\r\\n'", "''"] {
        let (_dir, mut client) = peer(&padded_pong(SIDECAR_MAX_RESPONSE_BYTES_V1 + 1, ending));
        let error = client.ping().unwrap_err();
        assert_rejected_and_closed(&mut client, error, SidecarOperation::Control);
    }
}

#[test]
fn exact_limit_valid_frames_accept_newline_crlf_and_legacy_eof() {
    for ending in ["'\\n'", "'\\r\\n'", "''"] {
        let (_dir, mut client) = peer(&padded_pong(SIDECAR_MAX_RESPONSE_BYTES_V1, ending));
        assert_eq!(client.ping().unwrap().protocol_version, "0.1");
    }
}

#[test]
fn unterminated_oversize_is_rejected_without_waiting_for_peer_eof() {
    let (_dir, mut client) = peer(&format!(
        "{}time.sleep(30)\n",
        padded_pong(SIDECAR_MAX_RESPONSE_BYTES_V1 + 1, "''")
    ));
    // The normal ten-second deadline cannot produce the required size error if
    // framing waits for EOF. No tiny policy dependent on interpreter startup.
    let error = client.ping().unwrap_err();
    assert_rejected_and_closed(&mut client, error, SidecarOperation::Control);
}

#[test]
fn oversized_analysis_reply_keeps_operation_context_and_invalidates_peer() {
    let (_dir, mut client) = peer(&format!(
        "{}{}time.sleep(30)\n",
        padded_pong(256, "'\\n'"),
        padded_pong(SIDECAR_MAX_RESPONSE_BYTES_V1 + 1, "'\\n'")
    ));
    client.ping().unwrap();
    let source = SourceDescriptor {
        source_id: SourceId::from("synthetic-response-limit"),
        path: "metadata-only-not-a-file".into(),
        content_hash: "synthetic".into(),
        duration_seconds: 0.1,
        sample_rate: 48_000,
        channel_count: 1,
        decode_profile: DecodeProfile::Native,
    };
    let error = client.build_source_graph_stub(source, 0).unwrap_err();
    assert_rejected_and_closed(&mut client, error, SidecarOperation::Analysis);
}

#[test]
fn coalesced_reply_after_a_near_limit_frame_is_preserved() {
    for first_bytes in [
        SIDECAR_MAX_RESPONSE_BYTES_V1 - 1,
        SIDECAR_MAX_RESPONSE_BYTES_V1,
    ] {
        let (_dir, mut client) = peer(&format!(
            r#"
request = json.loads(sys.stdin.readline())
def frame(request_id, size):
    body = json.dumps({{'type': 'pong', 'request_id': request_id, 'protocol_version': '0.1', 'sidecar_version': 'synthetic'}}).encode()
    return body + b' ' * (size - len(body) - 1) + b'\n'
sys.stdout.buffer.write(frame(request['request_id'], {first_bytes}) + frame('req-2', 256))
sys.stdout.buffer.flush()
sys.stdin.readline()
"#
        ));
        client.ping().unwrap();
        client.ping().unwrap();
    }
}

#[test]
fn limit_counts_utf8_bytes_not_characters() {
    let (_dir, mut client) = peer(&format!(
        r#"
request = json.loads(sys.stdin.readline())
text = json.dumps({{'type': 'error', 'request_id': request['request_id'], 'code': 'synthetic_unicode', 'message': 'ä' * ({limit} // 2), 'retryable': False}}, ensure_ascii=False)
assert len(text) < {limit}
sys.stdout.buffer.write(text.encode('utf-8') + b'\n')
sys.stdout.buffer.flush()
"#,
        limit = SIDECAR_MAX_RESPONSE_BYTES_V1,
    ));
    let error = client.ping().unwrap_err();
    assert_rejected_and_closed(&mut client, error, SidecarOperation::Control);
}
