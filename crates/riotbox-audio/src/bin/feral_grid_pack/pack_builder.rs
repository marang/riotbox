use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use riotbox_core::source_graph::{
    MeterHint, SourceTimingCandidateConfidenceResult, SourceTimingCandidateDriftStatus,
    SourceTimingCandidatePhraseStatus, SourceTimingProbeBeatEvidenceStatus,
    SourceTimingProbeBpmCandidatePolicy, SourceTimingProbeDownbeatEvidenceStatus,
    SourceTimingProbeReadinessReport, SourceTimingProbeReadinessStatus,
    source_timing_probe_readiness_report,
};

use riotbox_audio::{
    listening_manifest::{
        LISTENING_MANIFEST_SCHEMA_VERSION, ListeningPackArtifact as ManifestArtifact,
        ListeningPackRenderMetrics as ManifestRenderMetrics, write_manifest_json,
    },
    runtime::{
        OfflineAudioMetrics, render_tr909_offline, render_w30_preview_offline,
        signal_metrics_with_grid,
    },
    source_audio::{SourceAudioCache, SourceAudioError, write_interleaved_pcm16_wav},
    source_timing_probe::{SourceTimingProbeConfig, analyze_source_timing_probe},
    tr909::{
        Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
        Tr909RenderState, Tr909SourceSupportContext, Tr909SourceSupportProfile,
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
const MIN_LOW_BAND_RMS: f32 = 0.004;
const MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO: f32 = 0.45;
const MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO: f32 = 0.75;

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
    bar_variation: BarVariationMetrics,
    spectral_energy: SpectralEnergyMetrics,
}

#[derive(Clone, Copy, Debug)]
struct PackReport {
    tr909_source_profile: SourceAwareTr909Profile,
    w30_source_chop_profile: W30SourceChopProfile,
    tr909: RenderMetrics,
    w30: RenderMetrics,
    source_first_mix: RenderMetrics,
    full_mix: RenderMetrics,
    source_first_generated_to_source_rms_ratio: f32,
    support_generated_to_source_rms_ratio: f32,
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
    feral_scorecard: ManifestFeralScorecard,
    source_timing: ManifestSourceTimingReadiness,
    thresholds: ManifestThresholds,
    metrics: ManifestPackMetrics,
    verification_command: String,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestFeralScorecard {
    readiness: &'static str,
    break_rebuild_potential: &'static str,
    hook_fragment_count: u32,
    break_support_count: u32,
    quote_risk_count: u32,
    capture_candidate_count: u32,
    top_reason: &'static str,
    source_backed: bool,
    generated: bool,
    fallback_like: bool,
    lane_gestures: [&'static str; 2],
    material_sources: [&'static str; 2],
    warnings: [&'static str; 1],
}

#[derive(Serialize)]
struct ManifestThresholds {
    min_signal_rms: f32,
    min_low_band_rms: f32,
    max_source_first_generated_to_source_rms_ratio: f32,
    max_support_generated_to_source_rms_ratio: f32,
}

#[derive(Serialize)]
struct ManifestPackMetrics {
    tr909_source_profile: ManifestTr909SourceProfile,
    w30_source_chop_profile: ManifestW30SourceChopProfile,
    tr909_beat_fill: ManifestRenderMetrics,
    w30_feral_source_chop: ManifestRenderMetrics,
    source_first_mix: ManifestRenderMetrics,
    full_grid_mix: ManifestRenderMetrics,
    mix_balance: ManifestMixBalanceMetrics,
    bar_variation: ManifestBarVariationMetrics,
    spectral_energy: ManifestSpectralEnergyMetrics,
}

#[derive(Serialize)]
struct ManifestBarVariationMetrics {
    tr909_beat_fill: BarVariationMetrics,
    w30_feral_source_chop: BarVariationMetrics,
    source_first_mix: BarVariationMetrics,
    full_grid_mix: BarVariationMetrics,
}

#[derive(Serialize)]
struct ManifestSpectralEnergyMetrics {
    tr909_beat_fill: SpectralEnergyMetrics,
    w30_feral_source_chop: SpectralEnergyMetrics,
    source_first_mix: SpectralEnergyMetrics,
    full_grid_mix: SpectralEnergyMetrics,
}

#[derive(Serialize)]
struct ManifestMixBalanceMetrics {
    source_first_generated_to_source_rms_ratio: f32,
    support_generated_to_source_rms_ratio: f32,
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
         Renders a local grid-locked Feral demo pack. TR-909 beat/fill and\n\
         W-30 source chop stems share one beat/bar grid\n\
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
    let timing_readiness = source_timing_readiness_for_source(&source, &args.source_path);

    let w30_source_window = source.window_by_seconds(
        args.source_start_seconds,
        args.source_window_seconds.min(grid.duration_seconds()),
    );
    let source_window_samples = source.window_samples(w30_source_window);
    let (w30_preview, w30_source_chop_profile) = source_chop_preview_from_interleaved(
        source_window_samples,
        usize::from(CHANNEL_COUNT),
        w30_source_window.start_frame as u64,
        w30_source_window
            .start_frame
            .saturating_add(w30_source_window.frame_count) as u64,
    )
    .ok_or("source-backed W-30 chop window produced no samples")?;

    let tr909_source_profile = derive_source_aware_tr909_profile(source_window_samples, &grid);
    let tr909 = render_tr909_source_support(&grid, tr909_source_profile);
    let w30 = render_w30_source_chop(&grid, w30_preview);
    let source_first_mix = render_source_first_mix(&tr909, &w30);
    let full_mix = render_generated_support_mix(&tr909, &w30);

    assert_grid_len("tr909", &tr909, &grid);
    assert_grid_len("w30", &w30, &grid);
    assert_grid_len("source_first_mix", &source_first_mix, &grid);
    assert_grid_len("full_mix", &full_mix, &grid);

    write_audio_with_metrics(&stems_dir.join("01_tr909_beat_fill.wav"), &tr909, &grid)?;
    write_audio_with_metrics(&stems_dir.join("02_w30_feral_source_chop.wav"), &w30, &grid)?;
    write_audio_with_metrics(
        &output_dir.join("03_riotbox_source_first_mix.wav"),
        &source_first_mix,
        &grid,
    )?;
    write_audio_with_metrics(
        &output_dir.join("04_riotbox_generated_support_mix.wav"),
        &full_mix,
        &grid,
    )?;

    let report = PackReport {
        tr909_source_profile,
        w30_source_chop_profile,
        tr909: render_metrics(&tr909, &grid),
        w30: render_metrics(&w30, &grid),
        source_first_mix: render_metrics(&source_first_mix, &grid),
        full_mix: render_metrics(&full_mix, &grid),
        source_first_generated_to_source_rms_ratio:
            source_first_generated_to_source_rms_ratio(&tr909, &w30, &grid),
        support_generated_to_source_rms_ratio: support_generated_to_source_rms_ratio(
            &tr909, &w30, &grid,
        ),
    };
    validate_report(&report)?;
    write_report(&output_dir.join("grid-report.md"), args, &grid, report)?;
    write_manifest(
        &output_dir.join("manifest.json"),
        args,
        &grid,
        report,
        &timing_readiness,
    )?;
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

    #[cfg(test)]
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

fn source_timing_readiness_for_source(
    source: &SourceAudioCache,
    source_path: &Path,
) -> SourceTimingProbeReadinessReport {
    let probe = analyze_source_timing_probe(source, SourceTimingProbeConfig::default());
    let input = probe.bpm_candidate_input(
        source_path.display().to_string(),
        MeterHint {
            beats_per_bar: DEFAULT_BEATS_PER_BAR as u8,
            beat_unit: 4,
        },
    );
    source_timing_probe_readiness_report(&input, SourceTimingProbeBpmCandidatePolicy::default())
}
