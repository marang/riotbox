use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use riotbox_audio::{
    listening_manifest::{
        LISTENING_MANIFEST_SCHEMA_VERSION, ListeningPackArtifact as ManifestArtifact,
        ListeningPackSignalMetrics as ManifestSignalMetrics, write_manifest_json,
    },
    mc202::{
        Mc202ContourHint, Mc202HookResponse, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode,
        Mc202RenderRouting, Mc202RenderState,
    },
    runtime::{
        OfflineAudioMetrics, render_mc202_offline, render_tr909_offline,
        render_w30_preview_offline, signal_metrics,
    },
    source_audio::{SourceAudioCache, SourceAudioError, write_interleaved_pcm16_wav},
    tr909::{
        Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
        Tr909RenderState,
    },
    w30::{
        W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
        W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile,
    },
};

const PACK_ID: &str = "feral-before-after";
const SAMPLE_RATE: u32 = 44_100;
const CHANNEL_COUNT: u16 = 2;
const DEFAULT_DATE: &str = "local";
const DEFAULT_SOURCE_START_SECONDS: f32 = 0.0;
const DEFAULT_DURATION_SECONDS: f32 = 2.0;
const DEFAULT_SOURCE_WINDOW_SECONDS: f32 = 1.0;
const SILENCE_SECONDS: f32 = 0.75;
const MIN_SOURCE_RMS: f32 = 0.001;
const MIN_AFTER_RMS: f32 = 0.001;
const MIN_DELTA_RMS: f32 = 0.005;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    render_pack(&args)?;
    println!("wrote {}", args.output_dir().display());
    Ok(())
}

#[derive(Debug, PartialEq)]
struct Args {
    source_path: PathBuf,
    output_dir: Option<PathBuf>,
    date: String,
    source_start_seconds: f32,
    duration_seconds: f32,
    source_window_seconds: f32,
    show_help: bool,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut source_path = None;
        let mut output_dir = None;
        let mut date = DEFAULT_DATE.to_string();
        let mut source_start_seconds = DEFAULT_SOURCE_START_SECONDS;
        let mut duration_seconds = DEFAULT_DURATION_SECONDS;
        let mut source_window_seconds = DEFAULT_SOURCE_WINDOW_SECONDS;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--source" => {
                    source_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--source requires a path".to_string())?,
                    ));
                }
                "--output-dir" => {
                    output_dir =
                        Some(PathBuf::from(args.next().ok_or_else(|| {
                            "--output-dir requires a value".to_string()
                        })?));
                }
                "--date" => {
                    date = args
                        .next()
                        .ok_or_else(|| "--date requires a value".to_string())?;
                }
                "--source-start-seconds" => {
                    source_start_seconds = parse_non_negative_seconds(
                        "--source-start-seconds",
                        &args
                            .next()
                            .ok_or_else(|| "--source-start-seconds requires a value".to_string())?,
                    )?;
                }
                "--duration-seconds" => {
                    duration_seconds = parse_positive_seconds(
                        "--duration-seconds",
                        &args
                            .next()
                            .ok_or_else(|| "--duration-seconds requires a value".to_string())?,
                    )?;
                }
                "--source-window-seconds" => {
                    source_window_seconds = parse_positive_seconds(
                        "--source-window-seconds",
                        &args.next().ok_or_else(|| {
                            "--source-window-seconds requires a value".to_string()
                        })?,
                    )?;
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        let source_path = source_path.ok_or_else(|| "--source is required".to_string())?;

        Ok(Self {
            source_path,
            output_dir,
            date,
            source_start_seconds,
            duration_seconds,
            source_window_seconds,
            show_help,
        })
    }

    fn output_dir(&self) -> PathBuf {
        self.output_dir.clone().unwrap_or_else(|| {
            Path::new("artifacts")
                .join("audio_qa")
                .join(&self.date)
                .join(PACK_ID)
        })
    }
}

fn print_help() {
    println!(
        "Usage: feral_before_after_pack --source PATH [--date NAME] [--output-dir PATH]\n\
         \n\
         Optional window controls:\n\
           --source-start-seconds SECONDS\n\
           --duration-seconds SECONDS\n\
           --source-window-seconds SECONDS\n\
         \n\
         Renders a local Feral before/after listening pack with source excerpt,\n\
         Riotbox-transformed after render, before-then-after comparison, stems,\n\
         metrics, and README. This helper currently expects 44.1 kHz stereo PCM WAV input."
    );
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

fn render_pack(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = args.output_dir();
    let stems_dir = output_dir.join("stems");
    fs::create_dir_all(&stems_dir)?;

    let source = SourceAudioCache::load_pcm_wav(&args.source_path)?;
    validate_source_format(&source)?;

    let frame_count = seconds_to_frames(args.duration_seconds);
    let source_window = source.window_by_seconds(args.source_start_seconds, args.duration_seconds);
    if source_window.frame_count < frame_count {
        return Err(format!(
            "source window produced {} frames, but {} are required for {:.3}s",
            source_window.frame_count, frame_count, args.duration_seconds
        )
        .into());
    }

    let source_samples = source.window_samples(source_window).to_vec();
    let source_excerpt_path = output_dir.join("01_source_excerpt.wav");
    write_interleaved_pcm16_wav(
        &source_excerpt_path,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &source_samples,
    )?;

    let w30_source_window = source.window_by_seconds(
        args.source_start_seconds,
        args.source_window_seconds.min(args.duration_seconds),
    );
    let w30_preview = source_preview_from_interleaved(
        source.window_samples(w30_source_window),
        usize::from(CHANNEL_COUNT),
        w30_source_window.start_frame as u64,
        w30_source_window
            .start_frame
            .saturating_add(w30_source_window.frame_count) as u64,
    )
    .ok_or("source-backed W-30 preview window produced no samples")?;

    let w30 = render_w30_preview_offline(
        &w30_source_chop_state(w30_preview),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
    );
    let tr909 = render_tr909_offline(&tr909_fill_state(), SAMPLE_RATE, CHANNEL_COUNT, frame_count);
    let mc202 = render_mc202_offline(
        &mc202_instigator_state(),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
    );
    let after = feral_after_mix(&source_samples, &w30, &tr909, &mc202);
    let before_then_after = before_then_after(&source_samples, &after);

    write_named_audio_with_metrics(&stems_dir.join("w30_source_chop.wav"), &w30)?;
    write_named_audio_with_metrics(&stems_dir.join("tr909_fill.wav"), &tr909)?;
    write_named_audio_with_metrics(&stems_dir.join("mc202_instigator.wav"), &mc202)?;
    write_named_audio_with_metrics(&output_dir.join("02_riotbox_feral_changed.wav"), &after)?;
    write_interleaved_pcm16_wav(
        output_dir.join("03_before_then_after.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &before_then_after,
    )?;

    let source_metrics = signal_metrics(&source_samples);
    let after_metrics = signal_metrics(&after);
    let delta_metrics = signal_delta_metrics(&source_samples, &after);
    let w30_metrics = signal_metrics(&w30);
    let tr909_metrics = signal_metrics(&tr909);
    let mc202_metrics = signal_metrics(&mc202);
    write_metrics_markdown(
        &output_dir.join("01_source_excerpt.metrics.md"),
        source_metrics,
    )?;
    write_comparison_markdown(
        &output_dir.join("comparison.md"),
        source_metrics,
        after_metrics,
        delta_metrics,
    )?;
    write_readme(&output_dir, args, &source_excerpt_path)?;

    if source_metrics.rms <= MIN_SOURCE_RMS {
        return Err("source excerpt rendered near silence".into());
    }
    if after_metrics.rms <= MIN_AFTER_RMS {
        return Err("Riotbox Feral after render produced near silence".into());
    }
    if delta_metrics.rms <= MIN_DELTA_RMS {
        return Err(format!(
            "Riotbox Feral after render is too similar to source: delta RMS {:.6}",
            delta_metrics.rms
        )
        .into());
    }

    write_manifest(
        &output_dir.join("manifest.json"),
        args,
        &output_dir,
        ManifestMetrics {
            source_excerpt: source_metrics.into(),
            riotbox_after: after_metrics.into(),
            source_after_delta: delta_metrics.into(),
            w30_source_chop: w30_metrics.into(),
            tr909_fill: tr909_metrics.into(),
            mc202_instigator: mc202_metrics.into(),
        },
    )?;

    Ok(())
}

fn validate_source_format(source: &SourceAudioCache) -> Result<(), Box<dyn std::error::Error>> {
    if source.sample_rate != SAMPLE_RATE || source.channel_count != CHANNEL_COUNT {
        return Err(format!(
            "feral_before_after_pack currently expects {SAMPLE_RATE} Hz / {CHANNEL_COUNT} channel PCM WAV, got {} Hz / {} channels",
            source.sample_rate, source.channel_count
        )
        .into());
    }
    Ok(())
}

fn seconds_to_frames(seconds: f32) -> usize {
    (seconds * SAMPLE_RATE as f32).round() as usize
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

fn w30_source_chop_state(source_window_preview: W30PreviewSampleWindow) -> W30PreviewRenderState {
    W30PreviewRenderState {
        mode: W30PreviewRenderMode::RawCaptureAudition,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
        active_bank_id: Some("bank-a".into()),
        focused_pad_id: Some("pad-01".into()),
        capture_id: Some("cap-feral-preview".into()),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: Some(source_window_preview),
        pad_playback: None,
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
    }
}

fn tr909_fill_state() -> Tr909RenderState {
    Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
        drum_bus_level: 0.82,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Tr909RenderState::default()
    }
}

fn mc202_instigator_state() -> Mc202RenderState {
    Mc202RenderState {
        mode: Mc202RenderMode::Instigator,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::InstigatorSpike,
        note_budget: Mc202NoteBudget::Push,
        contour_hint: Mc202ContourHint::Lift,
        hook_response: Mc202HookResponse::AnswerSpace,
        touch: 0.9,
        music_bus_level: 0.74,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
    }
}

fn feral_after_mix(source: &[f32], w30: &[f32], tr909: &[f32], mc202: &[f32]) -> Vec<f32> {
    debug_assert_eq!(source.len(), w30.len());
    debug_assert_eq!(source.len(), tr909.len());
    debug_assert_eq!(source.len(), mc202.len());

    source
        .iter()
        .zip(w30.iter())
        .zip(tr909.iter())
        .zip(mc202.iter())
        .map(|(((source, w30), tr909), mc202)| {
            let mixed = source * 0.28 + w30 * 1.10 + tr909 * 1.20 + mc202 * 0.95;
            (mixed * 2.5).tanh() * 0.94
        })
        .collect()
}

fn before_then_after(source: &[f32], after: &[f32]) -> Vec<f32> {
    let silence_len = seconds_to_frames(SILENCE_SECONDS) * usize::from(CHANNEL_COUNT);
    let mut output = Vec::with_capacity(source.len() + silence_len + after.len());
    output.extend_from_slice(source);
    output.extend(std::iter::repeat_n(0.0, silence_len));
    output.extend_from_slice(after);
    output
}

fn write_named_audio_with_metrics(path: &Path, samples: &[f32]) -> Result<(), SourceAudioError> {
    write_interleaved_pcm16_wav(path, SAMPLE_RATE, CHANNEL_COUNT, samples)?;
    write_metrics_markdown(&metrics_path_for(path), signal_metrics(samples))
        .map_err(|error| SourceAudioError::Io(error.to_string()))
}

fn metrics_path_for(path: &Path) -> PathBuf {
    let mut metrics_path = path.to_path_buf();
    metrics_path.set_file_name(match path.file_stem().and_then(|stem| stem.to_str()) {
        Some(stem) => format!("{stem}.metrics.md"),
        None => "metrics.md".to_string(),
    });
    metrics_path
}

