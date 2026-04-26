use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use riotbox_audio::{
    runtime::{render_w30_preview_offline, signal_metrics},
    w30::{
        W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
        W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile,
    },
};

const DEFAULT_OUTPUT_PATH: &str =
    "artifacts/audio_qa/local/w30-preview-smoke/raw_capture_source_window_preview/candidate.wav";
const SAMPLE_RATE: u32 = 44_100;
const CHANNEL_COUNT: u16 = 2;
const DEFAULT_DURATION_SECONDS: f32 = 2.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    let frame_count = (SAMPLE_RATE as f32 * args.duration_seconds).round() as usize;
    let samples = render_w30_preview_offline(
        &source_window_smoke_state(),
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
    show_help: bool,
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut output_path = PathBuf::from(DEFAULT_OUTPUT_PATH);
        let mut duration_seconds = DEFAULT_DURATION_SECONDS;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--out" => {
                    let Some(value) = args.next() else {
                        return Err("--out requires a path".into());
                    };
                    output_path = PathBuf::from(value);
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

        Ok(Self {
            output_path,
            duration_seconds,
            show_help,
        })
    }
}

fn print_help() {
    println!(
        "Usage: w30_preview_render [--out PATH] [--duration-seconds SECONDS]\n\
         \n\
         Renders the initial w30-preview-smoke source-window case to a PCM16 WAV\n\
         plus a sibling candidate.metrics.md file. This is a local review helper,\n\
         not a full listening-pack harness yet."
    );
}

fn source_window_smoke_state() -> W30PreviewRenderState {
    let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample = 0.18 + index as f32 * 0.002;
    }

    W30PreviewRenderState {
        mode: W30PreviewRenderMode::RawCaptureAudition,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
        active_bank_id: Some("bank-a".into()),
        focused_pad_id: Some("pad-01".into()),
        capture_id: Some("cap-01".into()),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: Some(W30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: 64,
            sample_count: 64,
            samples,
        }),
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 32.0,
    }
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
             - Case: `raw_capture_source_window_preview`\n\
             - Output: `{}`\n\
             - Sample rate: `{SAMPLE_RATE}`\n\
             - Channels: `{CHANNEL_COUNT}`\n\
             - Duration seconds: `{:.3}`\n\
             - Samples: `{sample_count}`\n\
             - Active samples: `{}`\n\
             - Peak abs: `{:.6}`\n\
             - RMS: `{:.6}`\n\
             - Sum: `{:.6}`\n",
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
                output_path: PathBuf::from(DEFAULT_OUTPUT_PATH),
                duration_seconds: DEFAULT_DURATION_SECONDS,
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
                "--duration-seconds".to_string(),
                "0.5".to_string(),
            ])
            .expect("parse args"),
            Args {
                output_path: PathBuf::from("tmp/render.wav"),
                duration_seconds: 0.5,
                show_help: false,
            }
        );
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
