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
    fn builds_cache_from_interleaved_samples_without_disk_roundtrip() {
        let cache = SourceAudioCache::from_interleaved_samples(
            "generated.wav",
            1_000,
            2,
            vec![0.25, -0.25, 0.50, -0.50],
        )
        .expect("cache");

        assert_eq!(cache.path, Path::new("generated.wav"));
        assert_eq!(cache.sample_rate, 1_000);
        assert_eq!(cache.channel_count, 2);
        assert_eq!(cache.frame_count(), 2);
        assert_eq!(cache.duration_seconds(), 0.002);
        assert_eq!(cache.interleaved_samples(), &[0.25, -0.25, 0.50, -0.50]);
    }

    #[test]
    fn rejects_malformed_interleaved_cache_samples() {
        let error = SourceAudioCache::from_interleaved_samples(
            "bad.wav",
            1_000,
            2,
            vec![0.0, 1.0, 0.5],
        )
        .expect_err("malformed interleaved samples should fail");

        assert_eq!(
            error,
            SourceAudioError::InvalidWave(
                "interleaved sample count must be divisible by channel count".into()
            )
        );
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
