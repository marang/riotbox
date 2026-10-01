use std::fs;
use std::path::Path;

pub(super) fn write_pcm16_wav(
    path: &Path,
    sample_rate: u32,
    channel_count: u16,
    samples: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let data_len = samples.len() * 2;
    let riff_len = 36 + data_len;
    let byte_rate = sample_rate * u32::from(channel_count) * 2;
    let block_align = channel_count * 2;

    let mut bytes = Vec::with_capacity(44 + data_len);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(riff_len as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data_len as u32).to_le_bytes());

    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }

    fs::write(path, bytes)?;
    Ok(())
}
