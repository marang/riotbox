use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use riotbox_audio::{
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

const PACK_ID: &str = "feral-grid-demo";
const SAMPLE_RATE: u32 = 44_100;
const CHANNEL_COUNT: u16 = 2;
const DEFAULT_DATE: &str = "local";
const DEFAULT_BPM: f32 = 128.0;
const DEFAULT_BARS: u32 = 8;
const DEFAULT_BEATS_PER_BAR: u32 = 4;
const MIN_BARS: u32 = 2;
const DEFAULT_SOURCE_START_SECONDS: f32 = 0.0;
const DEFAULT_SOURCE_WINDOW_SECONDS: f32 = 1.0;
const MIN_SIGNAL_RMS: f32 = 0.001;
const MIN_MC202_BAR_DELTA_RMS: f32 = 0.003;
const MIN_LOW_BAND_RMS: f32 = 0.004;

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
    bpm: f32,
    bars: u32,
    source_start_seconds: f32,
    source_window_seconds: f32,
    show_help: bool,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut source_path = None;
        let mut output_dir = None;
        let mut date = DEFAULT_DATE.to_string();
        let mut bpm = DEFAULT_BPM;
        let mut bars = DEFAULT_BARS;
        let mut source_start_seconds = DEFAULT_SOURCE_START_SECONDS;
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
                "--bpm" => {
                    bpm = parse_positive_f32(
                        "--bpm",
                        &args
                            .next()
                            .ok_or_else(|| "--bpm requires a value".to_string())?,
                    )?;
                }
                "--bars" => {
                    bars = parse_bars(
                        &args
                            .next()
                            .ok_or_else(|| "--bars requires a value".to_string())?,
                    )?;
                }
                "--source-start-seconds" => {
                    source_start_seconds = parse_non_negative_f32(
                        "--source-start-seconds",
                        &args
                            .next()
                            .ok_or_else(|| "--source-start-seconds requires a value".to_string())?,
                    )?;
                }
                "--source-window-seconds" => {
                    source_window_seconds = parse_positive_f32(
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
            bpm,
            bars,
            source_start_seconds,
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

#[derive(Clone, Debug)]
struct Grid {
    bpm: f32,
    beats_per_bar: u32,
    bars: u32,
    total_beats: u32,
    total_frames: usize,
}

#[derive(Clone, Copy, Debug)]
struct RenderMetrics {
    signal: OfflineAudioMetrics,
    low_band: OfflineAudioMetrics,
}

#[derive(Clone, Copy, Debug)]
struct PackReport {
    mc202: RenderMetrics,
    tr909: RenderMetrics,
    w30: RenderMetrics,
    full_mix: RenderMetrics,
    mc202_question_answer_delta: OfflineAudioMetrics,
}

#[derive(Serialize)]
struct ListeningPackManifest {
    schema_version: u32,
    pack_id: &'static str,
    source: String,
    sample_rate: u32,
    channel_count: u16,
    bpm: f32,
    beats_per_bar: u32,
    bars: u32,
    total_beats: u32,
    total_frames: usize,
    duration_seconds: f32,
    source_start_seconds: f32,
    source_window_seconds: f32,
    artifacts: Vec<ManifestArtifact>,
    thresholds: ManifestThresholds,
    metrics: ManifestPackMetrics,
    verification_command: String,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestArtifact {
    role: &'static str,
    kind: &'static str,
    path: String,
    metrics_path: Option<String>,
}

#[derive(Serialize)]
struct ManifestThresholds {
    min_signal_rms: f32,
    min_mc202_bar_delta_rms: f32,
    min_low_band_rms: f32,
}

#[derive(Serialize)]
struct ManifestPackMetrics {
    mc202_question_answer: ManifestRenderMetrics,
    tr909_beat_fill: ManifestRenderMetrics,
    w30_feral_source_chop: ManifestRenderMetrics,
    full_grid_mix: ManifestRenderMetrics,
    mc202_question_answer_delta: ManifestSignalMetrics,
}

#[derive(Serialize)]
struct ManifestRenderMetrics {
    signal: ManifestSignalMetrics,
    low_band: ManifestSignalMetrics,
}

#[derive(Serialize)]
struct ManifestSignalMetrics {
    active_samples: usize,
    peak_abs: f32,
    rms: f32,
    sum: f32,
}

fn print_help() {
    println!(
        "Usage: feral_grid_pack --source PATH [--date NAME] [--output-dir PATH]\n\
         \n\
         Optional grid controls:\n\
           --bpm BPM\n\
           --bars BARS\n\
           --source-start-seconds SECONDS\n\
           --source-window-seconds SECONDS\n\
         \n\
         Renders a local grid-locked Feral demo pack. MC-202 question/answer,\n\
         TR-909 beat/fill, and W-30 source chop stems share one beat/bar grid\n\
         so the output can be checked for musical timing instead of only logs."
    );
}

fn parse_positive_f32(flag: &str, value: &str) -> Result<f32, String> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("{flag} must be greater than zero"))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(format!("{flag} must be greater than zero"));
    }
    Ok(parsed)
}

fn parse_non_negative_f32(flag: &str, value: &str) -> Result<f32, String> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("{flag} must be a non-negative number"))?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(format!("{flag} must be a non-negative number"));
    }
    Ok(parsed)
}

fn parse_positive_u32(flag: &str, value: &str) -> Result<u32, String> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| format!("{flag} must be greater than zero"))?;
    if parsed == 0 {
        return Err(format!("{flag} must be greater than zero"));
    }
    Ok(parsed)
}

fn parse_bars(value: &str) -> Result<u32, String> {
    let bars = parse_positive_u32("--bars", value)?;
    if bars < MIN_BARS {
        return Err(format!("--bars must be at least {MIN_BARS}"));
    }
    Ok(bars)
}

fn render_pack(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let grid = Grid::new(args.bpm, DEFAULT_BEATS_PER_BAR, args.bars)?;
    let output_dir = args.output_dir();
    let stems_dir = output_dir.join("stems");
    fs::create_dir_all(&stems_dir)?;

    let source = SourceAudioCache::load_pcm_wav(&args.source_path)?;
    validate_source_format(&source)?;

    let w30_source_window = source.window_by_seconds(
        args.source_start_seconds,
        args.source_window_seconds.min(grid.duration_seconds()),
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

    let mc202 = render_mc202_question_answer(&grid);
    let tr909 = render_tr909_beat_fill(&grid);
    let w30 = render_w30_source_chop(&grid, w30_preview);
    let full_mix = render_full_mix(&mc202, &tr909, &w30);

    assert_grid_len("mc202", &mc202, &grid);
    assert_grid_len("tr909", &tr909, &grid);
    assert_grid_len("w30", &w30, &grid);
    assert_grid_len("full_mix", &full_mix, &grid);

    write_audio_with_metrics(&stems_dir.join("01_mc202_question_answer.wav"), &mc202)?;
    write_audio_with_metrics(&stems_dir.join("02_tr909_beat_fill.wav"), &tr909)?;
    write_audio_with_metrics(&stems_dir.join("03_w30_feral_source_chop.wav"), &w30)?;
    write_audio_with_metrics(&output_dir.join("04_riotbox_grid_feral_mix.wav"), &full_mix)?;

    let report = PackReport {
        mc202: render_metrics(&mc202),
        tr909: render_metrics(&tr909),
        w30: render_metrics(&w30),
        full_mix: render_metrics(&full_mix),
        mc202_question_answer_delta: mc202_first_question_answer_delta(&mc202, &grid),
    };
    validate_report(&report)?;
    write_report(&output_dir.join("grid-report.md"), args, &grid, report)?;
    write_manifest(&output_dir.join("manifest.json"), args, &grid, report)?;
    write_readme(&output_dir, args, &grid)?;

    Ok(())
}

impl Grid {
    fn new(bpm: f32, beats_per_bar: u32, bars: u32) -> Result<Self, String> {
        if !bpm.is_finite() || bpm <= 0.0 {
            return Err("bpm must be greater than zero".to_string());
        }
        if beats_per_bar == 0 || bars == 0 {
            return Err("beats_per_bar and bars must be greater than zero".to_string());
        }
        let total_beats = beats_per_bar
            .checked_mul(bars)
            .ok_or_else(|| "grid beat count overflowed".to_string())?;
        let total_frames = frames_for_beats(bpm, total_beats);
        Ok(Self {
            bpm,
            beats_per_bar,
            bars,
            total_beats,
            total_frames,
        })
    }

    fn duration_seconds(&self) -> f32 {
        self.total_beats as f32 * 60.0 / self.bpm
    }

    fn bar_start_frame(&self, bar: u32) -> usize {
        frames_for_beats(self.bpm, bar.saturating_mul(self.beats_per_bar))
    }

    fn bar_end_frame(&self, bar: u32) -> usize {
        frames_for_beats(self.bpm, (bar + 1).saturating_mul(self.beats_per_bar))
    }

    fn bar_frame_count(&self, bar: u32) -> usize {
        self.bar_end_frame(bar)
            .saturating_sub(self.bar_start_frame(bar))
    }
}

fn frames_for_beats(bpm: f32, beats: u32) -> usize {
    (beats as f64 * f64::from(SAMPLE_RATE) * 60.0 / f64::from(bpm)).round() as usize
}

fn validate_source_format(source: &SourceAudioCache) -> Result<(), Box<dyn std::error::Error>> {
    if source.sample_rate != SAMPLE_RATE || source.channel_count != CHANNEL_COUNT {
        return Err(format!(
            "feral_grid_pack currently expects {SAMPLE_RATE} Hz / {CHANNEL_COUNT} channel PCM WAV, got {} Hz / {} channels",
            source.sample_rate, source.channel_count
        )
        .into());
    }
    Ok(())
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

fn render_mc202_question_answer(grid: &Grid) -> Vec<f32> {
    let mut output =
        Vec::with_capacity(grid.total_frames.saturating_mul(usize::from(CHANNEL_COUNT)));
    for bar in 0..grid.bars {
        let question = bar.is_multiple_of(2);
        let state =
            mc202_question_answer_state(question, grid.bpm, f64::from(bar * grid.beats_per_bar));
        let chunk = render_mc202_offline(
            &state,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            grid.bar_frame_count(bar),
        );
        output.extend(chunk);
    }
    output
}

fn mc202_question_answer_state(question: bool, bpm: f32, position_beats: f64) -> Mc202RenderState {
    if question {
        Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            note_budget: Mc202NoteBudget::Balanced,
            contour_hint: Mc202ContourHint::Lift,
            hook_response: Mc202HookResponse::Direct,
            touch: 0.76,
            music_bus_level: 0.86,
            tempo_bpm: bpm,
            position_beats,
            is_transport_running: true,
        }
    } else {
        Mc202RenderState {
            mode: Mc202RenderMode::Answer,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::AnswerHook,
            note_budget: Mc202NoteBudget::Sparse,
            contour_hint: Mc202ContourHint::Drop,
            hook_response: Mc202HookResponse::AnswerSpace,
            touch: 0.92,
            music_bus_level: 0.88,
            tempo_bpm: bpm,
            position_beats,
            is_transport_running: true,
        }
    }
}

fn render_tr909_beat_fill(grid: &Grid) -> Vec<f32> {
    render_tr909_offline(
        &Tr909RenderState {
            mode: Tr909RenderMode::Fill,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            drum_bus_level: 0.94,
            slam_intensity: 0.32,
            is_transport_running: true,
            tempo_bpm: grid.bpm,
            position_beats: 0.0,
            ..Tr909RenderState::default()
        },
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.total_frames,
    )
}

fn render_w30_source_chop(grid: &Grid, source_window_preview: W30PreviewSampleWindow) -> Vec<f32> {
    render_w30_preview_offline(
        &W30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            active_bank_id: Some("bank-a".into()),
            focused_pad_id: Some("pad-01".into()),
            capture_id: Some("cap-feral-grid".into()),
            trigger_revision: 1,
            trigger_velocity: 0.82,
            source_window_preview: Some(source_window_preview),
            pad_playback: None,
            music_bus_level: 0.72,
            grit_level: 0.46,
            is_transport_running: true,
            tempo_bpm: grid.bpm,
            position_beats: 0.0,
        },
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.total_frames,
    )
}

fn render_full_mix(mc202: &[f32], tr909: &[f32], w30: &[f32]) -> Vec<f32> {
    debug_assert_eq!(mc202.len(), tr909.len());
    debug_assert_eq!(mc202.len(), w30.len());

    let tr909_low = one_pole_lowpass(tr909, 165.0);
    mc202
        .iter()
        .zip(tr909.iter())
        .zip(tr909_low.iter())
        .zip(w30.iter())
        .map(|(((mc202, tr909), tr909_low), w30)| {
            let mixed = mc202 * 1.12 + tr909 * 10.0 + tr909_low * 18.0 + w30 * 0.94;
            (mixed * 1.7).tanh() * 0.92
        })
        .collect()
}

fn one_pole_lowpass(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
    let dt = 1.0 / SAMPLE_RATE as f32;
    let rc = 1.0 / (std::f32::consts::TAU * cutoff_hz.max(1.0));
    let alpha = dt / (rc + dt);
    let mut state = [0.0_f32; CHANNEL_COUNT as usize];
    let mut output = Vec::with_capacity(samples.len());

    for frame in samples.chunks_exact(usize::from(CHANNEL_COUNT)) {
        for (channel, sample) in frame.iter().enumerate() {
            state[channel] += alpha * (*sample - state[channel]);
            output.push(state[channel]);
        }
    }

    output
}

fn assert_grid_len(name: &str, samples: &[f32], grid: &Grid) {
    assert_eq!(
        samples.len(),
        grid.total_frames.saturating_mul(usize::from(CHANNEL_COUNT)),
        "{name} must match grid length"
    );
}

fn render_metrics(samples: &[f32]) -> RenderMetrics {
    RenderMetrics {
        signal: signal_metrics(samples),
        low_band: signal_metrics(&one_pole_lowpass(samples, 165.0)),
    }
}

fn mc202_first_question_answer_delta(mc202: &[f32], grid: &Grid) -> OfflineAudioMetrics {
    if grid.bars < 2 {
        return OfflineAudioMetrics {
            active_samples: 0,
            peak_abs: 0.0,
            rms: 0.0,
            sum: 0.0,
        };
    }

    let channels = usize::from(CHANNEL_COUNT);
    let question_start = grid.bar_start_frame(0) * channels;
    let question_end = grid.bar_end_frame(0) * channels;
    let answer_start = grid.bar_start_frame(1) * channels;
    let answer_end = grid.bar_end_frame(1) * channels;
    let question = &mc202[question_start..question_end];
    let answer = &mc202[answer_start..answer_end];
    let delta = question
        .iter()
        .zip(answer.iter())
        .map(|(question, answer)| question - answer)
        .collect::<Vec<_>>();
    signal_metrics(&delta)
}

fn validate_report(report: &PackReport) -> Result<(), Box<dyn std::error::Error>> {
    for (name, metrics) in [
        ("mc202", report.mc202),
        ("tr909", report.tr909),
        ("w30", report.w30),
        ("full_mix", report.full_mix),
    ] {
        if metrics.signal.rms <= MIN_SIGNAL_RMS {
            return Err(format!("{name} rendered near silence").into());
        }
    }

    if report.mc202_question_answer_delta.rms <= MIN_MC202_BAR_DELTA_RMS {
        return Err(format!(
            "MC-202 question/answer bars are too similar: delta RMS {:.6}",
            report.mc202_question_answer_delta.rms
        )
        .into());
    }

    if report.full_mix.low_band.rms <= MIN_LOW_BAND_RMS {
        return Err(format!(
            "full mix low-band support is too weak: low-band RMS {:.6}",
            report.full_mix.low_band.rms
        )
        .into());
    }

    Ok(())
}

fn write_audio_with_metrics(path: &Path, samples: &[f32]) -> Result<(), SourceAudioError> {
    write_interleaved_pcm16_wav(path, SAMPLE_RATE, CHANNEL_COUNT, samples)?;
    write_metrics_markdown(&metrics_path_for(path), render_metrics(samples))
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

fn write_metrics_markdown(path: &Path, metrics: RenderMetrics) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Grid Demo Metrics\n\n\
             - Pack: `{PACK_ID}`\n\
             - Peak abs: `{:.6}`\n\
             - RMS: `{:.6}`\n\
             - Active samples: `{}`\n\
             - Sum: `{:.6}`\n\
             - Low-band peak abs: `{:.6}`\n\
             - Low-band RMS: `{:.6}`\n",
            metrics.signal.peak_abs,
            metrics.signal.rms,
            metrics.signal.active_samples,
            metrics.signal.sum,
            metrics.low_band.peak_abs,
            metrics.low_band.rms
        ),
    )
}

fn write_report(path: &Path, args: &Args, grid: &Grid, report: PackReport) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Grid Demo Report\n\n\
             - Pack: `{PACK_ID}`\n\
             - Source: `{}`\n\
             - BPM: `{:.3}`\n\
             - Bars: `{}`\n\
             - Beats per bar: `{}`\n\
             - Total beats: `{}`\n\
             - Total frames: `{}`\n\
             - Duration seconds: `{:.6}`\n\
             - MC-202 question/answer delta RMS: `{:.6}`\n\
             - Minimum MC-202 delta RMS: `{MIN_MC202_BAR_DELTA_RMS:.6}`\n\
             - Full mix low-band RMS: `{:.6}`\n\
             - Minimum full mix low-band RMS: `{MIN_LOW_BAND_RMS:.6}`\n\
             - Result: `pass`\n\n\
             | Stem | RMS | Peak abs | Low-band RMS | Active samples |\n\
             | --- | ---: | ---: | ---: | ---: |\n\
             | MC-202 question/answer | {:.6} | {:.6} | {:.6} | {} |\n\
             | TR-909 beat/fill | {:.6} | {:.6} | {:.6} | {} |\n\
             | W-30 Feral source chop | {:.6} | {:.6} | {:.6} | {} |\n\
             | Full grid mix | {:.6} | {:.6} | {:.6} | {} |\n",
            args.source_path.display(),
            grid.bpm,
            grid.bars,
            grid.beats_per_bar,
            grid.total_beats,
            grid.total_frames,
            grid.duration_seconds(),
            report.mc202_question_answer_delta.rms,
            report.full_mix.low_band.rms,
            report.mc202.signal.rms,
            report.mc202.signal.peak_abs,
            report.mc202.low_band.rms,
            report.mc202.signal.active_samples,
            report.tr909.signal.rms,
            report.tr909.signal.peak_abs,
            report.tr909.low_band.rms,
            report.tr909.signal.active_samples,
            report.w30.signal.rms,
            report.w30.signal.peak_abs,
            report.w30.low_band.rms,
            report.w30.signal.active_samples,
            report.full_mix.signal.rms,
            report.full_mix.signal.peak_abs,
            report.full_mix.low_band.rms,
            report.full_mix.signal.active_samples
        ),
    )
}

fn write_manifest(
    path: &Path,
    args: &Args,
    grid: &Grid,
    report: PackReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = args.output_dir();
    let source_window_seconds = args.source_window_seconds.min(grid.duration_seconds());
    let manifest = ListeningPackManifest {
        schema_version: 1,
        pack_id: PACK_ID,
        source: args.source_path.display().to_string(),
        sample_rate: SAMPLE_RATE,
        channel_count: CHANNEL_COUNT,
        bpm: grid.bpm,
        beats_per_bar: grid.beats_per_bar,
        bars: grid.bars,
        total_beats: grid.total_beats,
        total_frames: grid.total_frames,
        duration_seconds: grid.duration_seconds(),
        source_start_seconds: args.source_start_seconds,
        source_window_seconds,
        artifacts: manifest_artifacts(&output_dir),
        thresholds: ManifestThresholds {
            min_signal_rms: MIN_SIGNAL_RMS,
            min_mc202_bar_delta_rms: MIN_MC202_BAR_DELTA_RMS,
            min_low_band_rms: MIN_LOW_BAND_RMS,
        },
        metrics: ManifestPackMetrics {
            mc202_question_answer: report.mc202.into(),
            tr909_beat_fill: report.tr909.into(),
            w30_feral_source_chop: report.w30.into(),
            full_grid_mix: report.full_mix.into(),
            mc202_question_answer_delta: report.mc202_question_answer_delta.into(),
        },
        verification_command: format!(
            "just feral-grid-pack \"{}\" {} {:.3} {} {:.3} {:.3}",
            args.source_path.display(),
            args.date,
            grid.bpm,
            grid.bars,
            source_window_seconds,
            args.source_start_seconds
        ),
        result: "pass",
    };

    fs::write(path, serde_json::to_string_pretty(&manifest)? + "\n")?;
    Ok(())
}

fn manifest_artifacts(output_dir: &Path) -> Vec<ManifestArtifact> {
    vec![
        manifest_audio_artifact(
            "mc202_question_answer",
            output_dir.join("stems/01_mc202_question_answer.wav"),
        ),
        manifest_audio_artifact(
            "tr909_beat_fill",
            output_dir.join("stems/02_tr909_beat_fill.wav"),
        ),
        manifest_audio_artifact(
            "w30_feral_source_chop",
            output_dir.join("stems/03_w30_feral_source_chop.wav"),
        ),
        manifest_audio_artifact(
            "full_grid_mix",
            output_dir.join("04_riotbox_grid_feral_mix.wav"),
        ),
        ManifestArtifact {
            role: "grid_report",
            kind: "markdown_report",
            path: output_dir.join("grid-report.md").display().to_string(),
            metrics_path: None,
        },
        ManifestArtifact {
            role: "readme",
            kind: "markdown_readme",
            path: output_dir.join("README.md").display().to_string(),
            metrics_path: None,
        },
    ]
}

fn manifest_audio_artifact(role: &'static str, path: PathBuf) -> ManifestArtifact {
    ManifestArtifact {
        role,
        kind: "audio_wav",
        metrics_path: Some(metrics_path_for(&path).display().to_string()),
        path: path.display().to_string(),
    }
}

impl From<RenderMetrics> for ManifestRenderMetrics {
    fn from(metrics: RenderMetrics) -> Self {
        Self {
            signal: metrics.signal.into(),
            low_band: metrics.low_band.into(),
        }
    }
}

impl From<OfflineAudioMetrics> for ManifestSignalMetrics {
    fn from(metrics: OfflineAudioMetrics) -> Self {
        Self {
            active_samples: metrics.active_samples,
            peak_abs: metrics.peak_abs,
            rms: metrics.rms,
            sum: metrics.sum,
        }
    }
}

fn write_readme(output_dir: &Path, args: &Args, grid: &Grid) -> std::io::Result<()> {
    fs::write(
        output_dir.join("README.md"),
        format!(
            "# Feral Grid Demo Pack\n\n\
             This pack is the current Riotbox offline QA path for checking a musical grid,\n\
             not only a log path. All stems use the same BPM, bar count, and frame count.\n\n\
             ## Grid\n\n\
             - Source: `{}`\n\
             - BPM: `{:.3}`\n\
             - Bars: `{}`\n\
             - Beats per bar: `{}`\n\
             - Duration: `{:.3}s`\n\
             - Source window start: `{:.3}s`\n\
             - W-30 source window length: `{:.3}s`\n\n\
             ## Files\n\n\
             - `stems/01_mc202_question_answer.wav`: one-bar question, one-bar answer, alternating across the grid.\n\
             - `stems/02_tr909_beat_fill.wav`: TR-909 beat/fill support rendered on the same grid.\n\
             - `stems/03_w30_feral_source_chop.wav`: W-30 source-backed Feral chop rendered on the same grid.\n\
             - `04_riotbox_grid_feral_mix.wav`: combined grid-locked listening mix with low-end support.\n\
             - `grid-report.md`: timing and output metrics.\n\
             - `manifest.json`: machine-readable pack metadata, artifact paths, thresholds, and key metrics.\n\
\n\
             ## Current Limit\n\n\
             This is an offline QA/listening pack. It proves the render seams can align musically,\n\
             but it does not yet mean the live TUI mixer exposes this whole arrangement path directly.\n",
            args.source_path.display(),
            grid.bpm,
            grid.bars,
            grid.beats_per_bar,
            grid.duration_seconds(),
            args.source_start_seconds,
            args.source_window_seconds.min(grid.duration_seconds())
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_grid_controls() {
        let parsed = Args::parse([
            "--source".to_string(),
            "input.wav".to_string(),
            "--output-dir".to_string(),
            "out".to_string(),
            "--bpm".to_string(),
            "130.0".to_string(),
            "--bars".to_string(),
            "4".to_string(),
            "--source-window-seconds".to_string(),
            "0.5".to_string(),
        ])
        .expect("parse args");

        assert_eq!(parsed.source_path, PathBuf::from("input.wav"));
        assert_eq!(parsed.output_dir, Some(PathBuf::from("out")));
        assert_eq!(parsed.bpm, 130.0);
        assert_eq!(parsed.bars, 4);
        assert_eq!(parsed.source_window_seconds, 0.5);
    }

    #[test]
    fn rejects_missing_source() {
        assert!(Args::parse(Vec::<String>::new()).is_err());
    }

    #[test]
    fn rejects_single_bar_pack() {
        assert!(
            Args::parse([
                "--source".to_string(),
                "input.wav".to_string(),
                "--bars".to_string(),
                "1".to_string(),
            ])
            .is_err()
        );
    }

    #[test]
    fn grid_uses_cumulative_frame_rounding() {
        let grid = Grid::new(128.0, 4, 8).expect("grid");

        assert_eq!(grid.total_beats, 32);
        assert_eq!(grid.total_frames, frames_for_beats(128.0, 32));
        assert_eq!(grid.bar_frame_count(0), grid.bar_end_frame(0));
        assert_eq!(grid.bar_end_frame(7), grid.total_frames);
    }

    #[test]
    fn renders_grid_pack_files_and_noncollapsed_audio() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source_path = temp.path().join("source.wav");
        let output_dir = temp.path().join("pack");
        write_interleaved_pcm16_wav(
            &source_path,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &synthetic_break_source(frames_for_beats(128.0, 8)),
        )
        .expect("write source");

        let args = Args {
            source_path,
            output_dir: Some(output_dir.clone()),
            date: "test".into(),
            bpm: 128.0,
            bars: 2,
            source_start_seconds: 0.0,
            source_window_seconds: 0.5,
            show_help: false,
        };

        render_pack(&args).expect("render pack");

        assert!(
            output_dir
                .join("stems/01_mc202_question_answer.wav")
                .is_file()
        );
        assert!(output_dir.join("stems/02_tr909_beat_fill.wav").is_file());
        assert!(
            output_dir
                .join("stems/03_w30_feral_source_chop.wav")
                .is_file()
        );
        assert!(output_dir.join("04_riotbox_grid_feral_mix.wav").is_file());
        assert!(output_dir.join("grid-report.md").is_file());
        assert!(output_dir.join("manifest.json").is_file());

        let mc202 =
            SourceAudioCache::load_pcm_wav(output_dir.join("stems/01_mc202_question_answer.wav"))
                .expect("load mc202");
        let full_mix =
            SourceAudioCache::load_pcm_wav(output_dir.join("04_riotbox_grid_feral_mix.wav"))
                .expect("load full mix");
        let grid = Grid::new(128.0, 4, 2).expect("grid");

        assert_eq!(mc202.frame_count(), grid.total_frames);
        assert_eq!(full_mix.frame_count(), grid.total_frames);
        assert!(signal_metrics(full_mix.interleaved_samples()).rms > MIN_SIGNAL_RMS);
        assert!(
            mc202_first_question_answer_delta(mc202.interleaved_samples(), &grid).rms
                > MIN_MC202_BAR_DELTA_RMS
        );
        assert!(
            signal_metrics(&one_pole_lowpass(full_mix.interleaved_samples(), 165.0)).rms
                > MIN_LOW_BAND_RMS
        );

        let manifest = fs::read_to_string(output_dir.join("manifest.json")).expect("manifest");
        let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("parse manifest");
        assert_manifest_smoke_gate(&manifest, &output_dir);
    }

    fn assert_manifest_smoke_gate(manifest: &serde_json::Value, output_dir: &Path) {
        assert_eq!(manifest["schema_version"], 1);
        assert_eq!(manifest["pack_id"], PACK_ID);
        assert_eq!(manifest["result"], "pass");
        assert_eq!(manifest["bars"], 2);
        assert_manifest_f32(
            &manifest["thresholds"]["min_signal_rms"],
            MIN_SIGNAL_RMS,
            "min_signal_rms",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["min_mc202_bar_delta_rms"],
            MIN_MC202_BAR_DELTA_RMS,
            "min_mc202_bar_delta_rms",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["min_low_band_rms"],
            MIN_LOW_BAND_RMS,
            "min_low_band_rms",
        );

        let artifacts = manifest["artifacts"].as_array().expect("artifacts");
        assert_eq!(artifacts.len(), 6);
        assert_manifest_artifact(
            artifacts,
            "mc202_question_answer",
            "audio_wav",
            output_dir.join("stems/01_mc202_question_answer.wav"),
            Some(output_dir.join("stems/01_mc202_question_answer.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "tr909_beat_fill",
            "audio_wav",
            output_dir.join("stems/02_tr909_beat_fill.wav"),
            Some(output_dir.join("stems/02_tr909_beat_fill.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "w30_feral_source_chop",
            "audio_wav",
            output_dir.join("stems/03_w30_feral_source_chop.wav"),
            Some(output_dir.join("stems/03_w30_feral_source_chop.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "full_grid_mix",
            "audio_wav",
            output_dir.join("04_riotbox_grid_feral_mix.wav"),
            Some(output_dir.join("04_riotbox_grid_feral_mix.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "grid_report",
            "markdown_report",
            output_dir.join("grid-report.md"),
            None,
        );
        assert_manifest_artifact(
            artifacts,
            "readme",
            "markdown_readme",
            output_dir.join("README.md"),
            None,
        );

        assert!(
            manifest["metrics"]["full_grid_mix"]["signal"]["rms"]
                .as_f64()
                .expect("full mix rms")
                > f64::from(MIN_SIGNAL_RMS)
        );
        assert!(
            manifest["metrics"]["full_grid_mix"]["low_band"]["rms"]
                .as_f64()
                .expect("low-band rms")
                > f64::from(MIN_LOW_BAND_RMS)
        );
        assert!(
            manifest["metrics"]["mc202_question_answer_delta"]["rms"]
                .as_f64()
                .expect("delta rms")
                > f64::from(MIN_MC202_BAR_DELTA_RMS)
        );
    }

    fn assert_manifest_artifact(
        artifacts: &[serde_json::Value],
        role: &str,
        kind: &str,
        path: PathBuf,
        metrics_path: Option<PathBuf>,
    ) {
        let artifact = artifacts
            .iter()
            .find(|artifact| artifact["role"] == role)
            .unwrap_or_else(|| panic!("missing artifact role {role}"));

        assert_eq!(artifact["kind"], kind);
        assert_eq!(artifact["path"], path.display().to_string());
        assert!(path.is_file(), "manifest artifact should exist: {path:?}");

        match metrics_path {
            Some(metrics_path) => {
                assert_eq!(artifact["metrics_path"], metrics_path.display().to_string());
                assert!(
                    metrics_path.is_file(),
                    "manifest metrics artifact should exist: {metrics_path:?}"
                );
            }
            None => assert!(artifact["metrics_path"].is_null()),
        }
    }

    fn assert_manifest_f32(value: &serde_json::Value, expected: f32, name: &str) {
        let actual = value.as_f64().unwrap_or_else(|| panic!("{name} missing"));
        assert!(
            (actual - f64::from(expected)).abs() < 0.000_001,
            "{name} expected {expected}, got {actual}"
        );
    }

    fn synthetic_break_source(frame_count: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
        for frame in 0..frame_count {
            let phase = frame as f32 / SAMPLE_RATE as f32;
            let bar_pulse = frame % frames_for_beats(128.0, 1);
            let kick = if bar_pulse < 1_200 {
                ((1.0 - bar_pulse as f32 / 1_200.0).max(0.0) * 0.9)
                    * (phase * 74.0 * std::f32::consts::TAU).sin()
            } else {
                0.0
            };
            let grit = (phase * 510.0 * std::f32::consts::TAU).sin() * 0.08;
            let sample = kick + grit;
            samples.push(sample);
            samples.push(sample * 0.97);
        }
        samples
    }
}
