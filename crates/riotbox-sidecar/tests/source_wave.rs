use riotbox_sidecar::{
    client::{ClientError, StdioSidecarClient},
    path::bundled_sidecar_script_path,
};
use std::{fs, time::Duration};

#[test]
fn malformed_wave_rejection_keeps_real_python_peer_usable_for_valid_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let malformed_path = dir.path().join("malformed.wav");
    let valid_path = dir.path().join("generated.wav");
    let valid = generated_wave();
    fs::write(&valid_path, &valid).unwrap();
    let mut hidden_pcm = valid.clone();
    hidden_pcm[4..8].copy_from_slice(&36_u32.to_le_bytes());
    let cases = [vec![], b"RIFF".to_vec(), valid[..22].to_vec(), hidden_pcm];
    let mut client = StdioSidecarClient::spawn_python(bundled_sidecar_script_path()).unwrap();
    client.ping().unwrap();
    let mut client = client.with_response_timeout(Duration::from_secs(5));
    for (index, bytes) in cases.iter().enumerate() {
        fs::write(&malformed_path, bytes).unwrap();
        match client.analyze_source_file(&malformed_path, 0).unwrap_err() {
            ClientError::Sidecar(payload) => {
                assert_eq!(payload.code, "source_unsupported");
                assert_eq!(payload.request_id, Some(format!("req-{}", index * 3 + 2)));
                assert!(!payload.retryable);
                assert!(!payload.message.is_empty());
            }
            other => panic!("expected request-scoped rejection, got {other}"),
        }
        assert_eq!(client.ping().unwrap().protocol_version, "0.1");
        let graph = client.analyze_source_file(&valid_path, 0).unwrap();
        assert_eq!(graph.source.sample_rate, 8000);
        assert_eq!(graph.source.channel_count, 1);
        assert_eq!(graph.source.duration_seconds, 1.0);
    }
}

fn generated_wave() -> Vec<u8> {
    let mut bytes = b"RIFF".to_vec();
    bytes.extend(16036_u32.to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(8000_u32.to_le_bytes());
    bytes.extend(16000_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(16000_u32.to_le_bytes());
    for _ in 0..8000 {
        bytes.extend(4096_i16.to_le_bytes());
    }
    bytes
}
