use std::f32::consts::PI;

#[test]
fn ingest_observer_source_map_uses_decoded_bucket_evidence() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_path = temp.path().join("bucketed-source.wav");
    let session_path = temp.path().join("session.json");
    write_rising_pcm16_wave(&source_path, 44_100, 2, 2.0);

    let state = JamAppState::analyze_source_file_to_json(
        &source_path,
        &session_path,
        None,
        sidecar_script_path(),
        41,
    )
    .expect("ingest source through sidecar");
    let shell = JamShellState::new(state, ShellLaunchMode::Ingest);
    let graph = shell.app.source_graph.as_ref().expect("source graph");

    assert_eq!(graph.source_map.buckets.len(), 32);
    assert!(
        graph
            .source_map
            .buckets
            .iter()
            .any(|bucket| bucket.peak_class != riotbox_core::source_graph::SourceMapPeakClass::None)
    );

    let source_map = &shell.app.jam_view.source.source_map;
    let unique_energy_blocks = source_map
        .energy_row
        .chars()
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        unique_energy_blocks.len() >= 3,
        "expected bucket-backed energy contour, got {}",
        source_map.energy_row
    );

    let snapshot = observer_snapshot(&shell);
    assert_eq!(
        snapshot["source_map"]["energy_row"],
        source_map.energy_row.as_str()
    );
    assert_eq!(snapshot["source_map"]["peak_row"], source_map.peak_row.as_str());
}

fn sidecar_script_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../python/sidecar/json_stdio_sidecar.py")
        .canonicalize()
        .expect("resolve sidecar script path")
}

fn write_rising_pcm16_wave(
    path: impl AsRef<std::path::Path>,
    sample_rate: u32,
    channel_count: u16,
    duration_seconds: f32,
) {
    let path = path.as_ref();
    let frame_count = (sample_rate as f32 * duration_seconds) as u32;
    let bits_per_sample = 16_u16;
    let bytes_per_sample = (bits_per_sample / 8) as u32;
    let byte_rate = sample_rate * channel_count as u32 * bytes_per_sample;
    let block_align = channel_count * (bits_per_sample / 8);
    let data_len = frame_count * channel_count as u32 * bytes_per_sample;

    let mut bytes = Vec::with_capacity((44 + data_len) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());

    for frame_index in 0..frame_count {
        let progress = frame_index as f32 / frame_count.max(1) as f32;
        let amplitude = 0.05 + (0.90 * progress);
        let phase = (frame_index as f32 / sample_rate as f32) * 220.0 * 2.0 * PI;
        let sample = (phase.sin() * i16::MAX as f32 * amplitude) as i16;
        for _ in 0..channel_count {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
    }

    std::fs::write(path, bytes).expect("write PCM wave fixture");
}
