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

#[test]
fn decoded_sample_rejection_keeps_real_python_peer_usable_at_exact_limit() {
    let dir = tempfile::tempdir().unwrap();
    let over_path = dir.path().join("over.wav");
    let exact_path = dir.path().join("exact.wav");
    fs::write(&over_path, generated_pcm16_wave(2, 3)).unwrap();
    fs::write(&exact_path, generated_pcm16_wave(1, 4)).unwrap();
    let wrapper = dir.path().join("small_budget_sidecar.py");
    let sidecar_dir =
        serde_json::to_string(bundled_sidecar_script_path().parent().unwrap()).unwrap();
    // Test-only injection exercises the real request path without allocating a
    // default-limit payload. There is no product budget flag or environment knob.
    fs::write(
        &wrapper,
        format!(
            "import sys\nsys.path.insert(0, {sidecar_dir})\nimport source_wave\n\
             source_wave.SOURCE_WAV_MAX_DECODED_SAMPLES_V1 = 4\n\
             import json_stdio_sidecar\njson_stdio_sidecar.main()\n"
        ),
    )
    .unwrap();
    let mut client = StdioSidecarClient::spawn_python(&wrapper).unwrap();
    client.ping().unwrap();
    let mut client = client.with_response_timeout(Duration::from_secs(5));
    match client.analyze_source_file(&over_path, 0).unwrap_err() {
        ClientError::Sidecar(payload) => {
            assert_eq!(payload.code, "source_resource_limit");
            assert_eq!(payload.request_id.as_deref(), Some("req-2"));
            assert!(!payload.retryable);
            assert!(payload.message.contains("decoded interleaved samples"));
            assert!(payload.message.contains("required 6, limit 4"));
        }
        other => panic!("expected request-scoped sample rejection, got {other}"),
    }
    assert_eq!(client.ping().unwrap().protocol_version, "0.1");
    let graph = client.analyze_source_file(&exact_path, 0).unwrap();
    assert_eq!(graph.source.sample_rate, 8000);
    assert_eq!(graph.source.channel_count, 1);
    assert_eq!(graph.source.duration_seconds, 0.001);
}

fn generated_wave() -> Vec<u8> {
    generated_pcm16_wave(1, 8000)
}

fn generated_pcm16_wave(channels: u16, frames: u32) -> Vec<u8> {
    let block_align = channels * 2;
    let data_len = frames * u32::from(block_align);
    let mut bytes = b"RIFF".to_vec();
    bytes.extend((36 + data_len).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(channels.to_le_bytes());
    bytes.extend(8000_u32.to_le_bytes());
    bytes.extend((8000 * u32::from(block_align)).to_le_bytes());
    bytes.extend(block_align.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(data_len.to_le_bytes());
    for _ in 0..frames * u32::from(channels) {
        bytes.extend(4096_i16.to_le_bytes());
    }
    bytes
}
