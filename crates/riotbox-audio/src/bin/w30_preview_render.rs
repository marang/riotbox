use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use riotbox_audio::{
    runtime::{render_w30_preview_offline, signal_metrics},
    source_audio::SourceAudioCache,
    w30::{
        W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
        W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile,
    },
};

const DEFAULT_DATE: &str = "local";
const PACK_ID: &str = "w30-preview-smoke";
const CASE_ID: &str = "raw_capture_source_window_preview";
const SAMPLE_RATE: u32 = 44_100;
const CHANNEL_COUNT: u16 = 2;
const DEFAULT_DURATION_SECONDS: f32 = 2.0;
const DEFAULT_SOURCE_START_SECONDS: f32 = 0.0;
const DEFAULT_SOURCE_DURATION_SECONDS: f32 = 0.25;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    let frame_count = (SAMPLE_RATE as f32 * args.duration_seconds).round() as usize;
    let source_window_preview = args.source_window_preview()?;
    let samples = render_w30_preview_offline(
        &source_window_smoke_state(source_window_preview),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
    );
    let metrics = signal_metrics(&samples);

    write_pcm16_wav(&args.output_path, SAMPLE_RATE, CHANNEL_COUNT, &samples)?;

    let metrics_path = metrics_path_for(&args.output_path);
    write_metrics_markdown(&metrics_path, &args, samples.len(), metrics)?;

    println!("wrote {}", args.output_path.display());
    println!("wrote {}", metrics_path.display());

    Ok(())
}

#[derive(Debug, PartialEq)]
struct Args {
    output_path: PathBuf,
    duration_seconds: f32,
    date: String,
    role: RenderRole,
    source_path: Option<PathBuf>,
    source_start_seconds: f32,
    source_duration_seconds: f32,
    show_help: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderRole {
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

    const fn label(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Candidate => "candidate",
        }
    }
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
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
                    if duration_seconds <= 0.0 {
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

    fn source_window_preview(&self) -> Result<W30PreviewSampleWindow, Box<dyn std::error::Error>> {
        let Some(source_path) = self.source_path.as_ref() else {
            return Ok(synthetic_source_window_preview());
        };

        let cache = SourceAudioCache::load_pcm16_wav(source_path)?;
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

    fn source_input_label(&self) -> String {
        self.source_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "synthetic".to_string())
    }
}

fn print_help() {
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

fn source_window_smoke_state(
    source_window_preview: W30PreviewSampleWindow,
) -> W30PreviewRenderState {
    W30PreviewRenderState {
        mode: W30PreviewRenderMode::RawCaptureAudition,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
        active_bank_id: Some("bank-a".into()),
        focused_pad_id: Some("pad-01".into()),
        capture_id: Some("cap-01".into()),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: Some(source_window_preview),
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 32.0,
    }
}

fn synthetic_source_window_preview() -> W30PreviewSampleWindow {
    let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample = 0.18 + index as f32 * 0.002;
    }

    W30PreviewSampleWindow {
        source_start_frame: 0,
        source_end_frame: 64,
        sample_count: 64,
        samples,
    }
}

fn source_preview_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_start_frame: u64,
    source_end_frame: u64,
) -> Option<W30PreviewSampleWindow> {
    let channel_count = channel_count.max(1);
    let frame_count = samples.len() / channel_count;
    if frame_count == 0 {
        return None;
    }

    let sample_count = frame_count.min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    let stride = (frame_count / sample_count).max(1);
    let mut preview = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];

    for (index, slot) in preview.iter_mut().take(sample_count).enumerate() {
        let frame_index = (index * stride).min(frame_count - 1);
        let base = frame_index * channel_count;
        let sum: f32 = samples[base..base + channel_count].iter().sum();
        *slot = sum / channel_count as f32;
    }

    Some(W30PreviewSampleWindow {
        source_start_frame,
        source_end_frame,
        sample_count,
        samples: preview,
    })
}

fn write_pcm16_wav(
    path: &Path,
    sample_rate: u32,
    channel_count: u16,
    samples: &[f32],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let bytes_per_sample = 2_u16;
    let bits_per_sample = 16_u16;
    let byte_rate = sample_rate * u32::from(channel_count) * u32::from(bytes_per_sample);
    let block_align = channel_count * bytes_per_sample;
    let data_bytes = u32::try_from(samples.len().saturating_mul(usize::from(bytes_per_sample)))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "WAV output too large"))?;
    let riff_size = 36_u32
        .checked_add(data_bytes)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "WAV output too large"))?;

    let mut file = fs::File::create(path)?;
    file.write_all(b"RIFF")?;
    file.write_all(&riff_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;
    file.write_all(b"fmt ")?;
    file.write_all(&16_u32.to_le_bytes())?;
    file.write_all(&1_u16.to_le_bytes())?;
    file.write_all(&channel_count.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;

    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        file.write_all(&pcm.to_le_bytes())?;
    }

    Ok(())
}

fn metrics_path_for(output_path: &Path) -> PathBuf {
    let mut path = output_path.to_path_buf();
    path.set_file_name(
        match output_path.file_stem().and_then(|stem| stem.to_str()) {
            Some(stem) => format!("{stem}.metrics.md"),
            None => "candidate.metrics.md".to_string(),
        },
    );
    path
}

fn write_metrics_markdown(
    path: &Path,
    args: &Args,
    sample_count: usize,
    metrics: riotbox_audio::runtime::OfflineAudioMetrics,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(
        path,
        format!(
            "# W-30 Preview Smoke Metrics\n\n\
             - Pack: `{PACK_ID}`\n\
             - Case: `{CASE_ID}`\n\
             - Role: `{}`\n\
             - Source input: `{}`\n\
             - Output: `{}`\n\
             - Sample rate: `{SAMPLE_RATE}`\n\
             - Channels: `{CHANNEL_COUNT}`\n\
             - Duration seconds: `{:.3}`\n\
             - Samples: `{sample_count}`\n\
             - Active samples: `{}`\n\
             - Peak abs: `{:.6}`\n\
             - RMS: `{:.6}`\n\
             - Sum: `{:.6}`\n",
            args.role.label(),
            args.source_input_label(),
            args.output_path.display(),
            args.duration_seconds,
            metrics.active_samples,
            metrics.peak_abs,
            metrics.rms,
            metrics.sum
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_args() {
        assert_eq!(
            Args::parse(Vec::<String>::new()).expect("parse args"),
            Args {
                output_path: PathBuf::from(
                    "artifacts/audio_qa/local/w30-preview-smoke/raw_capture_source_window_preview/candidate.wav",
                ),
                duration_seconds: DEFAULT_DURATION_SECONDS,
                date: DEFAULT_DATE.to_string(),
                role: RenderRole::Candidate,
                source_path: None,
                source_start_seconds: DEFAULT_SOURCE_START_SECONDS,
                source_duration_seconds: DEFAULT_SOURCE_DURATION_SECONDS,
                show_help: false,
            }
        );
    }

    #[test]
    fn parses_custom_output_and_duration() {
        assert_eq!(
            Args::parse([
                "--out".to_string(),
                "tmp/render.wav".to_string(),
                "--date".to_string(),
                "2026-04-26".to_string(),
                "--role".to_string(),
                "baseline".to_string(),
                "--source".to_string(),
                "data/test_audio/examples/DH_BeatC_120-01.wav".to_string(),
                "--source-start-seconds".to_string(),
                "0.25".to_string(),
                "--source-duration-seconds".to_string(),
                "0.75".to_string(),
                "--duration-seconds".to_string(),
                "0.5".to_string(),
            ])
            .expect("parse args"),
            Args {
                output_path: PathBuf::from("tmp/render.wav"),
                duration_seconds: 0.5,
                date: "2026-04-26".to_string(),
                role: RenderRole::Baseline,
                source_path: Some(PathBuf::from(
                    "data/test_audio/examples/DH_BeatC_120-01.wav"
                )),
                source_start_seconds: 0.25,
                source_duration_seconds: 0.75,
                show_help: false,
            }
        );
    }

    #[test]
    fn derives_convention_path_from_date_and_role() {
        assert_eq!(
            Args::parse([
                "--date".to_string(),
                "2026-04-26".to_string(),
                "--role".to_string(),
                "baseline".to_string(),
            ])
            .expect("parse args")
            .output_path,
            PathBuf::from(
                "artifacts/audio_qa/2026-04-26/w30-preview-smoke/raw_capture_source_window_preview/baseline.wav",
            )
        );
    }

    #[test]
    fn rejects_unknown_roles() {
        assert!(Args::parse(["--role".to_string(), "review".to_string()]).is_err());
    }

    #[test]
    fn rejects_invalid_source_window_seconds() {
        assert!(Args::parse(["--source-start-seconds".to_string(), "-0.1".to_string()]).is_err());
        assert!(Args::parse(["--source-duration-seconds".to_string(), "0".to_string()]).is_err());
    }

    #[test]
    fn averages_interleaved_source_frames_into_preview() {
        let preview =
            source_preview_from_interleaved(&[1.0, 3.0, 5.0, 7.0], 2, 10, 12).expect("preview");

        assert_eq!(preview.source_start_frame, 10);
        assert_eq!(preview.source_end_frame, 12);
        assert_eq!(preview.sample_count, 2);
        assert_eq!(preview.samples[0], 2.0);
        assert_eq!(preview.samples[1], 6.0);
    }

    #[test]
    fn rejects_unknown_args() {
        assert!(Args::parse(["--unknown".to_string()]).is_err());
    }

    #[test]
    fn derives_sibling_metrics_path() {
        assert_eq!(
            metrics_path_for(Path::new("out/candidate.wav")),
            PathBuf::from("out/candidate.metrics.md")
        );
    }
}
