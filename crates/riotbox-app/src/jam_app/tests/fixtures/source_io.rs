use riotbox_core::source_graph::SourceGraph;
use std::f32::consts::PI;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub(in crate::jam_app::tests) fn bind_synthetic_wav_identity(
    graph: &mut SourceGraph,
    source_path: &Path,
) {
    use sha2::{Digest, Sha256};

    let bytes = fs::read(source_path).expect("read synthetic source WAV");
    let content_hash = format!("sha256:{:x}", Sha256::digest(bytes));
    graph.source.content_hash = content_hash.clone();
    graph.provenance.source_hash = content_hash;
}

pub(in crate::jam_app::tests) fn sidecar_script_path() -> PathBuf {
    riotbox_sidecar::path::bundled_sidecar_script_path()
}

pub(in crate::jam_app::tests) fn write_pcm16_wave(
    path: impl AsRef<Path>,
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
        let phase = (frame_index as f32 / sample_rate as f32) * 220.0 * 2.0 * PI;
        let sample = (phase.sin() * i16::MAX as f32 * 0.25) as i16;
        for _ in 0..channel_count {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
    }

    fs::write(path, bytes).expect("write PCM wave fixture");
}

pub(in crate::jam_app::tests) fn write_pcm24_wave(
    path: impl AsRef<Path>,
    sample_rate: u32,
    channel_count: u16,
) {
    let path = path.as_ref();
    let samples = [-8_388_608_i32, 0, 8_388_607, 4_194_304];
    assert_eq!(samples.len() % usize::from(channel_count), 0);
    let bits_per_sample = 24_u16;
    let bytes_per_sample = u32::from(bits_per_sample / 8);
    let byte_rate = sample_rate * u32::from(channel_count) * bytes_per_sample;
    let block_align = channel_count * (bits_per_sample / 8);
    let data_len = samples.len() as u32 * bytes_per_sample;

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

    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes()[..3]);
    }

    fs::write(path, bytes).expect("write PCM24 wave fixture");
}
