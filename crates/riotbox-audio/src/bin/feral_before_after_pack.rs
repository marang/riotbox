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

fn write_metrics_markdown(path: &Path, metrics: OfflineAudioMetrics) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Before / After Metrics\n\n\
             - Peak abs: `{:.6}`\n\
             - RMS: `{:.6}`\n\
             - Active samples: `{}`\n\
             - Sum: `{:.6}`\n",
            metrics.peak_abs, metrics.rms, metrics.active_samples, metrics.sum
        ),
    )
}

fn signal_delta_metrics(baseline: &[f32], candidate: &[f32]) -> OfflineAudioMetrics {
    let delta = baseline
        .iter()
        .zip(candidate.iter())
        .map(|(baseline, candidate)| baseline - candidate)
        .collect::<Vec<_>>();
    signal_metrics(&delta)
}

fn write_comparison_markdown(
    path: &Path,
    source: OfflineAudioMetrics,
    after: OfflineAudioMetrics,
    delta: OfflineAudioMetrics,
) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Before / After Comparison\n\n\
             - Pack: `{PACK_ID}`\n\
             - Source RMS: `{:.6}`\n\
             - Riotbox after RMS: `{:.6}`\n\
             - Signal delta RMS: `{:.6}`\n\
             - Source peak abs: `{:.6}`\n\
             - Riotbox after peak abs: `{:.6}`\n\
             - Signal delta peak abs: `{:.6}`\n\
             - Result: `{}`\n",
            source.rms,
            after.rms,
            delta.rms,
            source.peak_abs,
            after.peak_abs,
            delta.peak_abs,
            if source.rms > MIN_SOURCE_RMS && after.rms > MIN_AFTER_RMS && delta.rms > MIN_DELTA_RMS
            {
                "pass"
            } else {
                "fail"
            }
        ),
    )
}

fn write_readme(output_dir: &Path, args: &Args, source_excerpt_path: &Path) -> std::io::Result<()> {
    fs::write(
        output_dir.join("README.md"),
        format!(
            "# Feral Before / After Pack\n\n\
             - Pack: `{PACK_ID}`\n\
             - Source: `{}`\n\
             - Source window: `{:.3}s` to `{:.3}s`\n\
             - Source preview window for W-30: `{:.3}s`\n\n\
             ## Files\n\n\
             - `01_source_excerpt.wav`: direct source excerpt.\n\
             - `02_riotbox_feral_changed.wav`: Riotbox-rendered Feral preview mix.\n\
             - `03_before_then_after.wav`: source excerpt, short silence, then Riotbox after render.\n\
             - `comparison.md`: source-vs-after metrics.\n\n\
             - `manifest.json`: machine-readable artifact paths, thresholds, and metrics.\n\n\
             ## Stems\n\n\
             - `stems/w30_source_chop.wav`: source-backed W-30 preview render.\n\
             - `stems/tr909_fill.wav`: TR-909 fill render.\n\
             - `stems/mc202_instigator.wav`: MC-202 instigator render.\n\n\
             ## Current Limit\n\n\
             This pack proves a current offline listening/QA path. It does not claim the live TUI mixer can perform this combined result directly yet.\n\n\
             ## Source Excerpt\n\n\
             `{}`\n",
            args.source_path.display(),
            args.source_start_seconds,
            args.source_start_seconds + args.duration_seconds,
            args.source_window_seconds.min(args.duration_seconds),
            source_excerpt_path.display()
        ),
    )
}

#[derive(Serialize)]
struct ListeningPackManifest {
    schema_version: u32,
    pack_id: &'static str,
    source: String,
    sample_rate: u32,
    channel_count: u16,
    duration_seconds: f32,
    source_start_seconds: f32,
    source_window_seconds: f32,
    silence_seconds: f32,
    artifacts: Vec<ManifestArtifact>,
    thresholds: ManifestThresholds,
    metrics: ManifestMetrics,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestThresholds {
    min_source_rms: f32,
    min_after_rms: f32,
    min_delta_rms: f32,
}

#[derive(Clone, Copy, Serialize)]
struct ManifestMetrics {
    source_excerpt: ManifestSignalMetrics,
    riotbox_after: ManifestSignalMetrics,
    source_after_delta: ManifestSignalMetrics,
    w30_source_chop: ManifestSignalMetrics,
    tr909_fill: ManifestSignalMetrics,
    mc202_instigator: ManifestSignalMetrics,
}

fn write_manifest(
    path: &Path,
    args: &Args,
    output_dir: &Path,
    metrics: ManifestMetrics,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = ListeningPackManifest {
        schema_version: LISTENING_MANIFEST_SCHEMA_VERSION,
        pack_id: PACK_ID,
        source: args.source_path.display().to_string(),
        sample_rate: SAMPLE_RATE,
        channel_count: CHANNEL_COUNT,
        duration_seconds: args.duration_seconds,
        source_start_seconds: args.source_start_seconds,
        source_window_seconds: args.source_window_seconds.min(args.duration_seconds),
        silence_seconds: SILENCE_SECONDS,
        artifacts: manifest_artifacts(output_dir),
        thresholds: ManifestThresholds {
            min_source_rms: MIN_SOURCE_RMS,
            min_after_rms: MIN_AFTER_RMS,
            min_delta_rms: MIN_DELTA_RMS,
        },
        metrics,
        result: "pass",
    };

    write_manifest_json(path, &manifest)?;
    Ok(())
}

fn manifest_artifacts(output_dir: &Path) -> Vec<ManifestArtifact> {
    let source_path = output_dir.join("01_source_excerpt.wav");
    let source_metrics_path = output_dir.join("01_source_excerpt.metrics.md");
    let after_path = output_dir.join("02_riotbox_feral_changed.wav");
    let after_metrics_path = metrics_path_for(&after_path);
    let before_after_path = output_dir.join("03_before_then_after.wav");
    let w30_path = output_dir.join("stems/w30_source_chop.wav");
    let w30_metrics_path = metrics_path_for(&w30_path);
    let tr909_path = output_dir.join("stems/tr909_fill.wav");
    let tr909_metrics_path = metrics_path_for(&tr909_path);
    let mc202_path = output_dir.join("stems/mc202_instigator.wav");
    let mc202_metrics_path = metrics_path_for(&mc202_path);

    vec![
        ManifestArtifact::audio_wav("source_excerpt", &source_path, Some(&source_metrics_path)),
        ManifestArtifact::audio_wav("riotbox_after", &after_path, Some(&after_metrics_path)),
        ManifestArtifact::audio_wav("before_then_after", &before_after_path, None),
        ManifestArtifact::audio_wav("w30_source_chop", &w30_path, Some(&w30_metrics_path)),
        ManifestArtifact::audio_wav("tr909_fill", &tr909_path, Some(&tr909_metrics_path)),
        ManifestArtifact::audio_wav("mc202_instigator", &mc202_path, Some(&mc202_metrics_path)),
        ManifestArtifact::markdown_report("comparison", &output_dir.join("comparison.md")),
        ManifestArtifact::markdown_readme("readme", &output_dir.join("README.md")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_source_and_custom_output() {
        let parsed = Args::parse([
            "--source".to_string(),
            "input.wav".to_string(),
            "--output-dir".to_string(),
            "out".to_string(),
            "--duration-seconds".to_string(),
            "0.5".to_string(),
            "--source-window-seconds".to_string(),
            "0.25".to_string(),
        ])
        .expect("parse args");

        assert_eq!(parsed.source_path, PathBuf::from("input.wav"));
        assert_eq!(parsed.output_dir, Some(PathBuf::from("out")));
        assert_eq!(parsed.duration_seconds, 0.5);
        assert_eq!(parsed.source_window_seconds, 0.25);
    }

    #[test]
    fn rejects_missing_source() {
        assert!(Args::parse(Vec::<String>::new()).is_err());
    }

    #[test]
    fn renders_pack_files_and_distinct_after_audio() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source_path = temp.path().join("source.wav");
        let output_dir = temp.path().join("pack");
        write_interleaved_pcm16_wav(
            &source_path,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &synthetic_break_source(seconds_to_frames(0.5)),
        )
        .expect("write source");

        let args = Args {
            source_path,
            output_dir: Some(output_dir.clone()),
            date: "test".into(),
            source_start_seconds: 0.0,
            duration_seconds: 0.5,
            source_window_seconds: 0.25,
            show_help: false,
        };

        render_pack(&args).expect("render pack");

        assert!(output_dir.join("01_source_excerpt.wav").is_file());
        assert!(output_dir.join("02_riotbox_feral_changed.wav").is_file());
        assert!(output_dir.join("03_before_then_after.wav").is_file());
        assert!(output_dir.join("stems/w30_source_chop.wav").is_file());
        assert!(output_dir.join("stems/tr909_fill.wav").is_file());
        assert!(output_dir.join("stems/mc202_instigator.wav").is_file());
        assert!(output_dir.join("comparison.md").is_file());
        assert!(output_dir.join("manifest.json").is_file());

        let source = SourceAudioCache::load_pcm_wav(output_dir.join("01_source_excerpt.wav"))
            .expect("load source");
        let after = SourceAudioCache::load_pcm_wav(output_dir.join("02_riotbox_feral_changed.wav"))
            .expect("load after");
        let source_metrics = signal_metrics(source.interleaved_samples());
        let after_metrics = signal_metrics(after.interleaved_samples());
        let delta = signal_delta_metrics(source.interleaved_samples(), after.interleaved_samples());

        assert!(source_metrics.rms > 0.001);
        assert!(after_metrics.rms > 0.001);
        assert!(delta.rms > 0.005);

        let manifest = fs::read_to_string(output_dir.join("manifest.json")).expect("manifest");
        let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("parse manifest");
        assert_eq!(
            manifest["schema_version"],
            LISTENING_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest["pack_id"], PACK_ID);
        assert_eq!(manifest["result"], "pass");
        assert_eq!(
            manifest["artifacts"].as_array().expect("artifacts").len(),
            8
        );
        assert!(
            manifest["metrics"]["riotbox_after"]["rms"]
                .as_f64()
                .expect("after rms")
                > f64::from(MIN_AFTER_RMS)
        );
        assert!(
            manifest["metrics"]["source_after_delta"]["rms"]
                .as_f64()
                .expect("delta rms")
                > f64::from(MIN_DELTA_RMS)
        );
        for artifact in manifest["artifacts"].as_array().expect("artifacts") {
            let path = PathBuf::from(artifact["path"].as_str().expect("artifact path"));
            assert!(path.is_file(), "{} missing", path.display());
            if let Some(metrics_path) = artifact["metrics_path"].as_str() {
                let metrics_path = PathBuf::from(metrics_path);
                assert!(metrics_path.is_file(), "{} missing", metrics_path.display());
            }
        }
    }

    fn synthetic_break_source(frame_count: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
        for frame in 0..frame_count {
            let phase = frame as f32 / SAMPLE_RATE as f32;
            let beat = frame % 11_025;
            let kick = if beat < 1_200 {
                ((1.0 - beat as f32 / 1_200.0).max(0.0) * 0.9)
                    * (phase * 80.0 * std::f32::consts::TAU).sin()
            } else {
                0.0
            };
            let grit = (phase * 730.0 * std::f32::consts::TAU).sin() * 0.08;
            let sample = kick + grit;
            samples.push(sample);
            samples.push(sample * 0.96);
        }
        samples
    }
}
