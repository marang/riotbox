use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq)]
pub struct SourceAudioCache {
    pub path: PathBuf,
    pub sample_rate: u32,
    pub channel_count: u16,
    samples: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceAudioWindow {
    pub start_frame: usize,
    pub frame_count: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SourceAudioError {
    Io(String),
    InvalidWave(String),
    UnsupportedWave(String),
}

impl SourceAudioCache {
    pub fn load_pcm_wav(path: impl AsRef<Path>) -> Result<Self, SourceAudioError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|error| SourceAudioError::Io(error.to_string()))?;
        let decoded = decode_pcm_wav(&bytes)?;

        Ok(Self {
            path: path.to_path_buf(),
            sample_rate: decoded.sample_rate,
            channel_count: decoded.channel_count,
            samples: decoded.samples,
        })
    }

    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channel_count as usize
    }

    pub fn duration_seconds(&self) -> f32 {
        self.frame_count() as f32 / self.sample_rate as f32
    }

    pub fn interleaved_samples(&self) -> &[f32] {
        &self.samples
    }

    pub fn window_by_seconds(
        &self,
        start_seconds: f32,
        duration_seconds: f32,
    ) -> SourceAudioWindow {
        let start_frame = seconds_to_frame(start_seconds, self.sample_rate).min(self.frame_count());
        let end_frame = seconds_to_frame(start_seconds + duration_seconds, self.sample_rate)
            .min(self.frame_count())
            .max(start_frame);

        SourceAudioWindow {
            start_frame,
            frame_count: end_frame - start_frame,
        }
    }

    pub fn window_samples(&self, window: SourceAudioWindow) -> &[f32] {
        let channels = self.channel_count as usize;
        let start = window
            .start_frame
            .saturating_mul(channels)
            .min(self.samples.len());
        let end = window
            .start_frame
            .saturating_add(window.frame_count)
            .saturating_mul(channels)
            .min(self.samples.len());

        &self.samples[start..end.max(start)]
    }

    pub fn write_window_pcm16_wav(
        &self,
        path: impl AsRef<Path>,
        window: SourceAudioWindow,
    ) -> Result<(), SourceAudioError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).map_err(|error| SourceAudioError::Io(error.to_string()))?;
        }

        let samples = self.window_samples(window);
        let bytes = pcm16_wave_bytes(self.sample_rate, self.channel_count, samples)?;
        fs::write(path, bytes).map_err(|error| SourceAudioError::Io(error.to_string()))
    }
}

impl fmt::Display for SourceAudioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "source audio I/O failed: {message}"),
            Self::InvalidWave(message) => write!(formatter, "invalid WAV source audio: {message}"),
            Self::UnsupportedWave(message) => {
                write!(formatter, "unsupported WAV source audio: {message}")
            }
        }
    }
}

impl Error for SourceAudioError {}

#[derive(Debug)]
struct DecodedPcmWave {
    sample_rate: u32,
    channel_count: u16,
    samples: Vec<f32>,
}

fn decode_pcm_wav(bytes: &[u8]) -> Result<DecodedPcmWave, SourceAudioError> {
    if bytes.len() < 12 {
        return Err(SourceAudioError::InvalidWave(
            "header shorter than RIFF/WAVE".into(),
        ));
    }
    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(SourceAudioError::InvalidWave(
            "missing RIFF/WAVE header".into(),
        ));
    }

    let mut cursor = 12;
    let mut format: Option<PcmFormatChunk> = None;
    let mut data: Option<&[u8]> = None;

    while cursor + 8 <= bytes.len() {
        let chunk_id = &bytes[cursor..cursor + 4];
        let chunk_len = read_u32_le(bytes, cursor + 4)? as usize;
        let chunk_start = cursor + 8;
        let chunk_end = chunk_start
            .checked_add(chunk_len)
            .ok_or_else(|| SourceAudioError::InvalidWave("chunk length overflow".into()))?;
        if chunk_end > bytes.len() {
            return Err(SourceAudioError::InvalidWave(
                "chunk extends past file end".into(),
            ));
        }

        match chunk_id {
            b"fmt " => format = Some(parse_format_chunk(&bytes[chunk_start..chunk_end])?),
            b"data" => data = Some(&bytes[chunk_start..chunk_end]),
            _ => {}
        }

        cursor = chunk_end + (chunk_len % 2);
    }

    let format = format.ok_or_else(|| SourceAudioError::InvalidWave("missing fmt chunk".into()))?;
    validate_format(format)?;
    let data = data.ok_or_else(|| SourceAudioError::InvalidWave("missing data chunk".into()))?;
    if data.len() % format.block_align as usize != 0 {
        return Err(SourceAudioError::InvalidWave(
            "data chunk does not align to whole frames".into(),
        ));
    }

    let bytes_per_sample = usize::from(format.bits_per_sample / 8);
    let samples = data
        .chunks_exact(bytes_per_sample)
        .map(|bytes| decode_pcm_sample(bytes, format.bits_per_sample))
        .collect();

    Ok(DecodedPcmWave {
        sample_rate: format.sample_rate,
        channel_count: format.channel_count,
        samples,
    })
}

#[derive(Clone, Copy, Debug)]
struct PcmFormatChunk {
    audio_format: u16,
    channel_count: u16,
    sample_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

fn parse_format_chunk(bytes: &[u8]) -> Result<PcmFormatChunk, SourceAudioError> {
    if bytes.len() < 16 {
        return Err(SourceAudioError::InvalidWave(
            "fmt chunk shorter than 16 bytes".into(),
        ));
    }

    Ok(PcmFormatChunk {
        audio_format: read_u16_le(bytes, 0)?,
        channel_count: read_u16_le(bytes, 2)?,
        sample_rate: read_u32_le(bytes, 4)?,
        block_align: read_u16_le(bytes, 12)?,
        bits_per_sample: read_u16_le(bytes, 14)?,
    })
}

fn validate_format(format: PcmFormatChunk) -> Result<(), SourceAudioError> {
    if format.audio_format != 1 {
        return Err(SourceAudioError::UnsupportedWave(format!(
            "audio format {} is not PCM",
            format.audio_format
        )));
    }
    if format.channel_count == 0 {
        return Err(SourceAudioError::InvalidWave(
            "channel count is zero".into(),
        ));
    }
    if format.sample_rate == 0 {
        return Err(SourceAudioError::InvalidWave("sample rate is zero".into()));
    }
    if !matches!(format.bits_per_sample, 16 | 24) {
        return Err(SourceAudioError::UnsupportedWave(format!(
            "{} bits per sample is not supported",
            format.bits_per_sample
        )));
    }
    let expected_block_align = format.channel_count * (format.bits_per_sample / 8);
    if format.block_align != expected_block_align {
        return Err(SourceAudioError::InvalidWave(format!(
            "block align {} does not match expected {}",
            format.block_align, expected_block_align
        )));
    }

    Ok(())
}

fn decode_pcm_sample(bytes: &[u8], bits_per_sample: u16) -> f32 {
    match bits_per_sample {
        16 => i16::from_le_bytes([bytes[0], bytes[1]]) as f32 / i16::MAX as f32,
        24 => {
            let unsigned =
                i32::from(bytes[0]) | (i32::from(bytes[1]) << 8) | (i32::from(bytes[2]) << 16);
            let signed = if unsigned & 0x80_0000 != 0 {
                unsigned | !0xFF_FFFF
            } else {
                unsigned
            };
            (signed as f32 / 8_388_607.0).clamp(-1.0, 1.0)
        }
        _ => 0.0,
    }
}

fn pcm16_wave_bytes(
    sample_rate: u32,
    channel_count: u16,
    samples: &[f32],
) -> Result<Vec<u8>, SourceAudioError> {
    if channel_count == 0 {
        return Err(SourceAudioError::InvalidWave(
            "channel count is zero".into(),
        ));
    }
    if !samples.len().is_multiple_of(usize::from(channel_count)) {
        return Err(SourceAudioError::InvalidWave(
            "sample count does not align to whole frames".into(),
        ));
    }

    let bits_per_sample = 16_u16;
    let bytes_per_sample = u32::from(bits_per_sample / 8);
    let data_len = u32::try_from(samples.len())
        .ok()
        .and_then(|sample_count| sample_count.checked_mul(bytes_per_sample))
        .ok_or_else(|| SourceAudioError::InvalidWave("PCM data too large".into()))?;
    let riff_len = 36_u32
        .checked_add(data_len)
        .ok_or_else(|| SourceAudioError::InvalidWave("RIFF data too large".into()))?;
    let byte_rate = sample_rate
        .checked_mul(u32::from(channel_count))
        .and_then(|value| value.checked_mul(bytes_per_sample))
        .ok_or_else(|| SourceAudioError::InvalidWave("byte rate overflow".into()))?;
    let block_align = channel_count * (bits_per_sample / 8);

    let data_len_usize = usize::try_from(data_len)
        .map_err(|_| SourceAudioError::InvalidWave("PCM data too large".into()))?;
    let mut bytes = Vec::with_capacity(44 + data_len_usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_len.to_le_bytes());
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
        let pcm = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }

    Ok(bytes)
}

fn seconds_to_frame(seconds: f32, sample_rate: u32) -> usize {
    if !seconds.is_finite() || seconds <= 0.0 {
        return 0;
    }

    (seconds * sample_rate as f32).floor() as usize
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16, SourceAudioError> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| SourceAudioError::InvalidWave("unexpected end of chunk".into()))?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, SourceAudioError> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| SourceAudioError::InvalidWave("unexpected end of chunk".into()))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

#[cfg(test)]
mod tests {
    use std::{f32::consts::PI, fs, path::Path};

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn loads_pcm16_wav_into_interleaved_float_cache() {
        let tempdir = tempdir().expect("create tempdir");
        let path = tempdir.path().join("source.wav");
        write_pcm16_wave(&path, 44_100, 2, 0.25);

        let cache = SourceAudioCache::load_pcm_wav(&path).expect("load PCM WAV");

        assert_eq!(cache.sample_rate, 44_100);
        assert_eq!(cache.channel_count, 2);
        assert_eq!(cache.frame_count(), 11_025);
        assert!((cache.duration_seconds() - 0.25).abs() < 0.001);
        assert_eq!(cache.interleaved_samples().len(), 22_050);
        assert!(
            cache
                .interleaved_samples()
                .iter()
                .any(|sample| sample.abs() > 0.01)
        );
    }

    #[test]
    fn loads_pcm24_wav_into_interleaved_float_cache() {
        let tempdir = tempdir().expect("create tempdir");
        let path = tempdir.path().join("source24.wav");
        fs::write(
            &path,
            pcm24_wave_bytes(44_100, 1, &[-8_388_608, 0, 8_388_607]),
        )
        .expect("write PCM24 wave fixture");

        let cache = SourceAudioCache::load_pcm_wav(&path).expect("load PCM24 WAV");

        assert_eq!(cache.sample_rate, 44_100);
        assert_eq!(cache.channel_count, 1);
        assert_eq!(cache.frame_count(), 3);
        assert_eq!(cache.interleaved_samples().len(), 3);
        assert_eq!(cache.interleaved_samples()[0], -1.0);
        assert_eq!(cache.interleaved_samples()[1], 0.0);
        assert!((cache.interleaved_samples()[2] - 1.0).abs() < 0.000001);
    }

    #[test]
    fn returns_bounded_sample_window_by_seconds() {
        let tempdir = tempdir().expect("create tempdir");
        let path = tempdir.path().join("source.wav");
        write_pcm16_wave(&path, 1_000, 2, 1.0);
        let cache = SourceAudioCache::load_pcm_wav(&path).expect("load PCM WAV");

        let window = cache.window_by_seconds(0.25, 0.50);
        let samples = cache.window_samples(window);

        assert_eq!(
            window,
            SourceAudioWindow {
                start_frame: 250,
                frame_count: 500,
            }
        );
        assert_eq!(samples.len(), 1_000);
    }

    #[test]
    fn clamps_windows_to_available_audio() {
        let tempdir = tempdir().expect("create tempdir");
        let path = tempdir.path().join("source.wav");
        write_pcm16_wave(&path, 1_000, 1, 1.0);
        let cache = SourceAudioCache::load_pcm_wav(&path).expect("load PCM WAV");

        let window = cache.window_by_seconds(0.90, 0.50);
        let samples = cache.window_samples(window);

        assert_eq!(
            window,
            SourceAudioWindow {
                start_frame: 900,
                frame_count: 100,
            }
        );
        assert_eq!(samples.len(), 100);
    }

    #[test]
    fn writes_source_window_as_pcm16_wav_artifact() {
        let tempdir = tempdir().expect("create tempdir");
        let source_path = tempdir.path().join("source.wav");
        let capture_path = tempdir.path().join("captures/cap-01.wav");
        write_pcm16_wave(&source_path, 1_000, 2, 1.0);
        let cache = SourceAudioCache::load_pcm_wav(&source_path).expect("load PCM WAV");

        cache
            .write_window_pcm16_wav(
                &capture_path,
                SourceAudioWindow {
                    start_frame: 250,
                    frame_count: 500,
                },
            )
            .expect("write capture artifact");

        let capture = SourceAudioCache::load_pcm_wav(&capture_path).expect("load capture artifact");
        assert_eq!(capture.sample_rate, 1_000);
        assert_eq!(capture.channel_count, 2);
        assert_eq!(capture.frame_count(), 500);
        assert!((capture.duration_seconds() - 0.5).abs() < 0.001);
        assert!(
            capture
                .interleaved_samples()
                .iter()
                .any(|sample| sample.abs() > 0.01)
        );
    }

    #[test]
    fn rejects_unsupported_pcm_bit_depth() {
        let mut bytes = pcm16_wave_bytes(44_100, 1, 1);
        bytes[34..36].copy_from_slice(&32_u16.to_le_bytes());

        let error = decode_pcm_wav(&bytes).expect_err("32-bit WAV should be rejected");

        assert_eq!(
            error,
            SourceAudioError::UnsupportedWave("32 bits per sample is not supported".into())
        );
    }

    #[test]
    fn rejects_non_wave_bytes() {
        let error = decode_pcm_wav(b"not a wav").expect_err("invalid data should fail");

        assert_eq!(
            error,
            SourceAudioError::InvalidWave("header shorter than RIFF/WAVE".into())
        );
    }

    fn write_pcm16_wave(path: &Path, sample_rate: u32, channel_count: u16, duration_seconds: f32) {
        let frame_count = (sample_rate as f32 * duration_seconds) as u32;
        let bytes = pcm16_wave_bytes(sample_rate, channel_count, frame_count);

        fs::write(path, bytes).expect("write PCM wave fixture");
    }

    fn pcm16_wave_bytes(sample_rate: u32, channel_count: u16, frame_count: u32) -> Vec<u8> {
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

        bytes
    }

    fn pcm24_wave_bytes(sample_rate: u32, channel_count: u16, samples: &[i32]) -> Vec<u8> {
        assert_eq!(samples.len() % usize::from(channel_count), 0);

        let bits_per_sample = 24_u16;
        let bytes_per_sample = (bits_per_sample / 8) as u32;
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

        for &sample in samples {
            let sample = sample.clamp(-8_388_608, 8_388_607);
            let encoded = sample.to_le_bytes();
            bytes.extend_from_slice(&encoded[..3]);
        }

        bytes
    }
}
