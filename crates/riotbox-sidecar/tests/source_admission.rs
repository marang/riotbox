use riotbox_sidecar::{
    client::{ClientError, StdioSidecarClient},
    path::bundled_sidecar_script_path,
};
use std::{fs::File, time::Duration};

#[test]
fn python_rejects_oversized_encoded_source_and_keeps_protocol_synchronized() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("generated-sparse-oversize.wav");
    File::create(&source).unwrap().set_len(268_435_457).unwrap();
    let mut client = StdioSidecarClient::spawn_python(bundled_sidecar_script_path()).unwrap();
    client.ping().unwrap();
    let mut client = client.with_response_timeout(Duration::from_secs(5));
    let error = client.analyze_source_file(&source, 0).unwrap_err();
    match error {
        ClientError::Sidecar(payload) => {
            assert_eq!(payload.code, "source_resource_limit");
            assert_eq!(payload.request_id.as_deref(), Some("req-2"));
            assert!(!payload.retryable);
            assert!(payload.message.contains("268435456"));
        }
        other => panic!("expected encoded admission failure, got {other}"),
    }
    assert_eq!(client.ping().unwrap().protocol_version, "0.1");
}
