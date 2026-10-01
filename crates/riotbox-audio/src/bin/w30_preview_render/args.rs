use super::config::{
    CASE_ID, CHANNEL_COUNT, DEFAULT_DATE, DEFAULT_DURATION_SECONDS,
    DEFAULT_SOURCE_DURATION_SECONDS, DEFAULT_SOURCE_START_SECONDS, PACK_ID, PCM16_BYTES_PER_SAMPLE,
    PCM16_RIFF_SIZE_OVERHEAD, SAMPLE_RATE,
};
use super::source_window::{source_preview_from_interleaved, synthetic_source_window_preview};
use riotbox_audio::{source_audio::SourceAudioCache, w30::W30PreviewSampleWindow};
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub(super) struct Args {
    pub(super) output_path: PathBuf,
    pub(super) duration_seconds: f32,
    pub(super) date: String,
    pub(super) role: RenderRole,
    pub(super) source_path: Option<PathBuf>,
    pub(super) source_start_seconds: f32,
    pub(super) source_duration_seconds: f32,
    pub(super) show_help: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RenderRole {
    Baseline,
    Candidate,
}

impl RenderRole {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "baseline" => Ok(Self::Baseline),
            "candidate" => Ok(Self::Candidate),
            other => Err(format!("unsupported role: {other}")),
        }
    }

    const fn file_stem(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Candidate => "candidate",
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Candidate => "candidate",
        }
    }
}

impl Args {
    pub(super) fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut output_override = None;
        let mut duration_seconds = DEFAULT_DURATION_SECONDS;
        let mut date = DEFAULT_DATE.to_string();
        let mut role = RenderRole::Candidate;
        let mut source_path = None;
        let mut source_start_seconds = DEFAULT_SOURCE_START_SECONDS;
        let mut source_duration_seconds = DEFAULT_SOURCE_DURATION_SECONDS;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--out" => {
                    let Some(value) = args.next() else {
                        return Err("--out requires a path".into());
                    };
                    output_override = Some(PathBuf::from(value));
                }
                "--date" => {
                    let Some(value) = args.next() else {
                        return Err("--date requires a value".into());
                    };
                    date = value;
                }
                "--role" => {
                    let Some(value) = args.next() else {
                        return Err("--role requires a value".into());
                    };
                    role = RenderRole::parse(&value)?;
                }
                "--source" => {
                    let Some(value) = args.next() else {
                        return Err("--source requires a path".into());
                    };
                    source_path = Some(PathBuf::from(value));
                }
                "--source-start-seconds" => {
                    let Some(value) = args.next() else {
                        return Err("--source-start-seconds requires a value".into());
                    };
                    source_start_seconds =
                        parse_non_negative_seconds("--source-start-seconds", &value)?;
                }
                "--source-duration-seconds" => {
                    let Some(value) = args.next() else {
                        return Err("--source-duration-seconds requires a value".into());
                    };
                    source_duration_seconds =
                        parse_positive_seconds("--source-duration-seconds", &value)?;
                }
                "--duration-seconds" => {
                    let Some(value) = args.next() else {
                        return Err("--duration-seconds requires a value".into());
                    };
                    duration_seconds = value
                        .parse::<f32>()
                        .map_err(|_| "--duration-seconds must be a number".to_string())?;
                    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
                        return Err("--duration-seconds must be greater than zero".into());
                    }
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        let output_path = output_override.unwrap_or_else(|| convention_output_path(&date, role));

        Ok(Self {
            output_path,
            duration_seconds,
            date,
            role,
            source_path,
            source_start_seconds,
            source_duration_seconds,
            show_help,
        })
    }

    pub(super) fn render_frame_count(&self) -> Result<usize, String> {
        let frames = (SAMPLE_RATE as f32 * self.duration_seconds).round() as usize;
        validate_render_frame_count(frames)?;
        Ok(frames)
    }

    pub(super) fn source_window_preview(
        &self,
    ) -> Result<W30PreviewSampleWindow, Box<dyn std::error::Error>> {
        let Some(source_path) = self.source_path.as_ref() else {
            return Ok(synthetic_source_window_preview());
        };

        let cache = SourceAudioCache::load_pcm_wav(source_path)?;
        let window =
            cache.window_by_seconds(self.source_start_seconds, self.source_duration_seconds);
        let samples = cache.window_samples(window);

        source_preview_from_interleaved(
            samples,
            usize::from(cache.channel_count),
            u64::try_from(window.start_frame).unwrap_or(u64::MAX),
            u64::try_from(window.start_frame.saturating_add(window.frame_count))
                .unwrap_or(u64::MAX),
        )
        .ok_or_else(|| {
            format!(
                "source window {} + {}s produced no samples",
                self.source_start_seconds, self.source_duration_seconds
            )
            .into()
        })
    }

    pub(super) fn source_input_label(&self) -> String {
        self.source_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "synthetic".to_string())
    }
}

pub(super) fn print_help() {
    println!(
        "Usage: w30_preview_render [--date YYYY-MM-DD|local] [--role baseline|candidate] [--out PATH] [--duration-seconds SECONDS]\n\
         \n\
         Optional source-backed preview input:\n\
           --source PATH\n\
           --source-start-seconds SECONDS\n\
           --source-duration-seconds SECONDS\n\
         \n\
         Renders the initial w30-preview-smoke source-window case to a PCM16 WAV\n\
         plus a sibling metrics Markdown file. This is a local review helper,\n\
         not a full listening-pack harness yet."
    );
}

fn convention_output_path(date: &str, role: RenderRole) -> PathBuf {
    let mut path = PathBuf::from("artifacts");
    path.push("audio_qa");
    path.push(date);
    path.push(PACK_ID);
    path.push(CASE_ID);
    path.push(format!("{}.wav", role.file_stem()));
    path
}

fn parse_non_negative_seconds(flag: &str, value: &str) -> Result<f32, String> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("{flag} must be a non-negative number"))?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(format!("{flag} must be a non-negative number"));
    }
    Ok(parsed)
}

fn parse_positive_seconds(flag: &str, value: &str) -> Result<f32, String> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("{flag} must be greater than zero"))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(format!("{flag} must be greater than zero"));
    }
    Ok(parsed)
}

fn validate_render_frame_count(frames: usize) -> Result<(), String> {
    let samples = frames
        .checked_mul(usize::from(CHANNEL_COUNT))
        .ok_or_else(|| "W-30 render buffer capacity exceeded".to_string())?;
    let buffer_bytes = samples.checked_mul(size_of::<f32>());
    if buffer_bytes.is_none_or(|bytes| bytes > isize::MAX as usize) {
        return Err("W-30 render buffer capacity exceeded".to_string());
    }
    let riff_size = samples
        .checked_mul(usize::from(PCM16_BYTES_PER_SAMPLE))
        .and_then(|bytes| u32::try_from(bytes).ok())
        .and_then(|bytes| bytes.checked_add(PCM16_RIFF_SIZE_OVERHEAD));
    if riff_size.is_none() {
        return Err("WAV output too large".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod capacity_tests {
    use super::{Args, validate_render_frame_count};
    use crate::config::{CHANNEL_COUNT, SAMPLE_RATE};

    const BUFFER_ERROR: &str = "W-30 render buffer capacity exceeded";
    #[cfg(target_pointer_width = "64")]
    const WAV_ERROR: &str = "WAV output too large";

    #[test]
    fn rejects_finite_duration_with_saturated_frames() {
        let args = Args::parse(["--duration-seconds".into(), "3.4028235e38".into()]).unwrap();
        assert!(args.duration_seconds.is_finite());
        assert_eq!(args.render_frame_count().unwrap_err(), BUFFER_ERROR);
    }

    #[test]
    fn rejects_unrepresentable_sample_and_f32_byte_counts() {
        for frames in [usize::MAX, usize::MAX / 2 + 1, usize::MAX / 4] {
            assert_eq!(
                validate_render_frame_count(frames).unwrap_err(),
                BUFFER_ERROR
            );
        }
    }

    #[test]
    fn rejects_f32_bytes_above_isize_max_even_when_usize_fits() {
        let frames = isize::MAX as usize / usize::from(CHANNEL_COUNT) / size_of::<f32>() + 1;
        let bytes = frames * usize::from(CHANNEL_COUNT) * size_of::<f32>();
        assert!(bytes > isize::MAX as usize);
        assert_eq!(
            validate_render_frame_count(frames).unwrap_err(),
            BUFFER_ERROR
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn enforces_exact_pcm16_riff_boundary_without_allocating() {
        // On 32-bit hosts the Vec layout binds earlier than this RIFF size.
        let frame_bytes = usize::from(CHANNEL_COUNT) * size_of::<i16>();
        let limit = (u32::MAX as usize - 36) / frame_bytes;
        assert!(validate_render_frame_count(limit).is_ok());
        assert!(u32::try_from((limit + 1) * frame_bytes).is_ok());
        assert_eq!(
            validate_render_frame_count(limit + 1).unwrap_err(),
            WAV_ERROR
        );
        assert_eq!(
            validate_render_frame_count(u32::MAX as usize / frame_bytes + 1).unwrap_err(),
            WAV_ERROR
        );
    }

    #[test]
    fn preserves_normal_f32_rounding_and_zero_frame_compatibility() {
        for duration in [0.000_001, 0.05, 0.5, 1.5, 2.0] {
            let args = Args::parse(["--duration-seconds".into(), duration.to_string()]).unwrap();
            assert_eq!(
                args.render_frame_count().unwrap(),
                (SAMPLE_RATE as f32 * duration).round() as usize
            );
        }
        assert!(validate_render_frame_count(0).is_ok());
        assert_eq!(
            Args::parse(["--duration-seconds".into(), "0.05".into()])
                .unwrap()
                .render_frame_count()
                .unwrap(),
            2_205
        );
    }
}
