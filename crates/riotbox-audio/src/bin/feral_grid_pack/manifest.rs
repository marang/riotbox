use std::path::{Path, PathBuf};

use serde::Serialize;

use riotbox_audio::listening_manifest::{
    LISTENING_MANIFEST_SCHEMA_VERSION, ListeningPackArtifact as ManifestArtifact,
    ListeningPackRenderMetrics as ManifestRenderMetrics, write_manifest_json,
};

use super::{
    AllLaneMixMovementProof, Args, BarVariationMetrics, CHANNEL_COUNT, Grid, GridBpmDecision,
    MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO, MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
    MIN_LOW_BAND_RMS, MIN_SIGNAL_RMS, ManifestMc202BassPressureProof,
    ManifestMc202SourceContourProof, ManifestSourceTimingReadiness, ManifestTr909KickPressureProof,
    ManifestTr909SourceAccentDynamicsProof, ManifestTr909SourceProfile,
    ManifestW30SourceAccentDynamicsProof, ManifestW30SourceChopProfile,
    ManifestW30SourceLoopClosureProof, ManifestW30SourceSliceChoiceProof,
    ManifestW30SourceTriggerVariationProof, PACK_ID, PackReport, RenderMetrics, SAMPLE_RATE,
    SourceCharacterWindowSelection, SourceGridOutputDriftMetrics, SourceTimingAnalysisForManifest,
    SpectralEnergyMetrics, Tr909GrooveTimingPolicy, Tr909RenderedDrumPressureProof,
    grid_bpm_decision_reason_label, grid_bpm_source_label, manifest_mc202_bass_pressure_proof,
    manifest_mc202_source_contour_proof, manifest_source_timing_readiness,
    manifest_tr909_kick_pressure_proof, manifest_tr909_source_accent_dynamics_proof,
    manifest_tr909_source_profile, manifest_w30_source_accent_dynamics_proof,
    manifest_w30_source_chop_profile, manifest_w30_source_loop_closure_proof,
    manifest_w30_source_slice_choice_proof, manifest_w30_source_trigger_variation_proof,
    metrics_path_for, verification_command,
};

#[derive(Serialize)]
struct ListeningPackManifest {
    schema_version: u32,
    pack_id: &'static str,
    source: String,
    sample_rate: u32,
    channel_count: u16,
    bpm: f32,
    grid_bpm_source: &'static str,
    grid_bpm_decision_reason: &'static str,
    source_timing_bpm_delta: Option<f32>,
    beats_per_bar: u32,
    bars: u32,
    total_beats: u32,
    total_frames: usize,
    duration_seconds: f32,
    source_start_seconds: f32,
    source_window_seconds: f32,
    artifacts: Vec<ManifestArtifact>,
    feral_scorecard: ManifestFeralScorecard,
    primitive_renderer_boundary: ManifestPrimitiveRendererBoundary,
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
    lane_gestures: [&'static str; 3],
    material_sources: [&'static str; 3],
    warnings: [&'static str; 1],
}

#[derive(Serialize)]
struct ManifestPrimitiveRendererBoundary {
    schema: &'static str,
    evidence_role: &'static str,
    product_output_allowed: bool,
    quality_proof: bool,
    demo_readiness: &'static str,
    promotion_blocked: bool,
    affected_paths: [&'static str; 1],
    musician_message: &'static str,
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
    source_character_window_selection: SourceCharacterWindowSelection,
    tr909_source_profile: ManifestTr909SourceProfile,
    tr909_groove_timing: Tr909GrooveTimingPolicy,
    tr909_kick_pressure: ManifestTr909KickPressureProof,
    tr909_source_accent_dynamics: ManifestTr909SourceAccentDynamicsProof,
    tr909_rendered_drum_pressure: Tr909RenderedDrumPressureProof,
    mc202_bass_pressure: ManifestMc202BassPressureProof,
    mc202_source_contour: ManifestMc202SourceContourProof,
    w30_source_chop_profile: ManifestW30SourceChopProfile,
    w30_source_loop_closure: ManifestW30SourceLoopClosureProof,
    w30_source_trigger_variation: ManifestW30SourceTriggerVariationProof,
    w30_source_slice_choice: ManifestW30SourceSliceChoiceProof,
    w30_source_accent_dynamics: ManifestW30SourceAccentDynamicsProof,
    all_lane_mix_movement: AllLaneMixMovementProof,
    tr909_beat_fill: ManifestRenderMetrics,
    mc202_bass_pressure_stem: ManifestRenderMetrics,
    w30_feral_source_chop: ManifestRenderMetrics,
    source_first_mix: ManifestRenderMetrics,
    full_grid_mix: ManifestRenderMetrics,
    mix_balance: ManifestMixBalanceMetrics,
    tr909_source_grid_alignment: SourceGridOutputDriftMetrics,
    mc202_source_grid_alignment: SourceGridOutputDriftMetrics,
    w30_source_grid_alignment: SourceGridOutputDriftMetrics,
    source_grid_output_drift: SourceGridOutputDriftMetrics,
    bar_variation: ManifestBarVariationMetrics,
    spectral_energy: ManifestSpectralEnergyMetrics,
}

#[derive(Serialize)]
struct ManifestBarVariationMetrics {
    tr909_beat_fill: BarVariationMetrics,
    mc202_bass_pressure_stem: BarVariationMetrics,
    w30_feral_source_chop: BarVariationMetrics,
    source_first_mix: BarVariationMetrics,
    full_grid_mix: BarVariationMetrics,
}

#[derive(Serialize)]
struct ManifestSpectralEnergyMetrics {
    tr909_beat_fill: SpectralEnergyMetrics,
    mc202_bass_pressure_stem: SpectralEnergyMetrics,
    w30_feral_source_chop: SpectralEnergyMetrics,
    source_first_mix: SpectralEnergyMetrics,
    full_grid_mix: SpectralEnergyMetrics,
}

#[derive(Serialize)]
struct ManifestMixBalanceMetrics {
    source_first_generated_to_source_rms_ratio: f32,
    support_generated_to_source_rms_ratio: f32,
}

pub(super) fn write_manifest(
    path: &Path,
    args: &Args,
    grid: &Grid,
    report: PackReport,
    source_timing_analysis: &SourceTimingAnalysisForManifest,
    grid_bpm: GridBpmDecision,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = args.output_dir();
    let source_window_seconds = args.source_window_seconds.min(grid.duration_seconds());
    let manifest = ListeningPackManifest {
        schema_version: LISTENING_MANIFEST_SCHEMA_VERSION,
        pack_id: PACK_ID,
        source: args.source_path.display().to_string(),
        sample_rate: SAMPLE_RATE,
        channel_count: CHANNEL_COUNT,
        bpm: grid.bpm,
        grid_bpm_source: grid_bpm_source_label(grid_bpm.source),
        grid_bpm_decision_reason: grid_bpm_decision_reason_label(grid_bpm.reason),
        source_timing_bpm_delta: grid_bpm.source_delta_bpm,
        beats_per_bar: grid.beats_per_bar,
        bars: grid.bars,
        total_beats: grid.total_beats,
        total_frames: grid.total_frames,
        duration_seconds: grid.duration_seconds(),
        source_start_seconds: args.source_start_seconds,
        source_window_seconds,
        artifacts: manifest_artifacts(&output_dir),
        feral_scorecard: manifest_feral_scorecard(),
        primitive_renderer_boundary: manifest_primitive_renderer_boundary(),
        source_timing: manifest_source_timing_readiness(
            &source_timing_analysis.readiness,
            grid_bpm,
            &source_timing_analysis.anchor_evidence,
            &source_timing_analysis.groove_evidence,
        ),
        thresholds: ManifestThresholds {
            min_signal_rms: MIN_SIGNAL_RMS,
            min_low_band_rms: MIN_LOW_BAND_RMS,
            max_source_first_generated_to_source_rms_ratio:
                MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
            max_support_generated_to_source_rms_ratio: MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
        },
        metrics: ManifestPackMetrics {
            source_character_window_selection: report.source_character_window_selection,
            tr909_source_profile: manifest_tr909_source_profile(report.tr909_source_profile),
            tr909_groove_timing: report.tr909_groove_timing,
            tr909_kick_pressure: manifest_tr909_kick_pressure_proof(report.tr909_kick_pressure),
            tr909_source_accent_dynamics: manifest_tr909_source_accent_dynamics_proof(
                report.tr909_source_accent_dynamics,
            ),
            tr909_rendered_drum_pressure: report.tr909_rendered_drum_pressure,
            mc202_bass_pressure: manifest_mc202_bass_pressure_proof(report.mc202_bass_pressure),
            mc202_source_contour: manifest_mc202_source_contour_proof(report.mc202_source_contour),
            w30_source_chop_profile: manifest_w30_source_chop_profile(
                report.w30_source_chop_profile,
            ),
            w30_source_loop_closure: manifest_w30_source_loop_closure_proof(
                report.w30_source_loop_closure,
            ),
            w30_source_trigger_variation: manifest_w30_source_trigger_variation_proof(
                report.w30_source_trigger_variation,
            ),
            w30_source_slice_choice: manifest_w30_source_slice_choice_proof(
                report.w30_source_slice_choice,
            ),
            w30_source_accent_dynamics: manifest_w30_source_accent_dynamics_proof(
                report.w30_source_accent_dynamics,
            ),
            all_lane_mix_movement: report.all_lane_mix_movement,
            tr909_beat_fill: manifest_render_metrics(report.tr909),
            mc202_bass_pressure_stem: manifest_render_metrics(report.mc202),
            w30_feral_source_chop: manifest_render_metrics(report.w30),
            source_first_mix: manifest_render_metrics(report.source_first_mix),
            full_grid_mix: manifest_render_metrics(report.full_mix),
            mix_balance: ManifestMixBalanceMetrics {
                source_first_generated_to_source_rms_ratio: report
                    .source_first_generated_to_source_rms_ratio,
                support_generated_to_source_rms_ratio: report.support_generated_to_source_rms_ratio,
            },
            tr909_source_grid_alignment: report.tr909_source_grid_alignment,
            mc202_source_grid_alignment: report.mc202_source_grid_alignment,
            w30_source_grid_alignment: report.w30_source_grid_alignment,
            source_grid_output_drift: report.source_grid_output_drift,
            bar_variation: ManifestBarVariationMetrics {
                tr909_beat_fill: report.tr909.bar_variation,
                mc202_bass_pressure_stem: report.mc202.bar_variation,
                w30_feral_source_chop: report.w30.bar_variation,
                source_first_mix: report.source_first_mix.bar_variation,
                full_grid_mix: report.full_mix.bar_variation,
            },
            spectral_energy: ManifestSpectralEnergyMetrics {
                tr909_beat_fill: report.tr909.spectral_energy,
                mc202_bass_pressure_stem: report.mc202.spectral_energy,
                w30_feral_source_chop: report.w30.spectral_energy,
                source_first_mix: report.source_first_mix.spectral_energy,
                full_grid_mix: report.full_mix.spectral_energy,
            },
        },
        verification_command: verification_command(args, grid, source_window_seconds),
        result: "pass",
    };

    write_manifest_json(path, &manifest)?;
    Ok(())
}

fn manifest_feral_scorecard() -> ManifestFeralScorecard {
    ManifestFeralScorecard {
        readiness: "ready",
        break_rebuild_potential: "high",
        hook_fragment_count: 1,
        break_support_count: 3,
        quote_risk_count: 0,
        capture_candidate_count: 1,
        top_reason: "grid-locked generated feral QA pack",
        source_backed: true,
        generated: true,
        fallback_like: false,
        lane_gestures: ["tr909 beat/fill", "mc202 bass pressure", "w30 source chop"],
        material_sources: [
            "generated tr909",
            "generated mc202",
            "source-backed w30 window",
        ],
        warnings: ["offline QA pack, not live arranger"],
    }
}

fn manifest_primitive_renderer_boundary() -> ManifestPrimitiveRendererBoundary {
    ManifestPrimitiveRendererBoundary {
        schema: "riotbox.primitive_renderer_boundary.v1",
        evidence_role: "non_product_diagnostic_control",
        product_output_allowed: false,
        quality_proof: false,
        demo_readiness: "unverified",
        promotion_blocked: true,
        affected_paths: ["metrics.mc202_bass_pressure.pattern_origin"],
        musician_message: "Primitive renderer lanes are diagnostic controls; source-derived product plans or unavailable/degraded state are required before demo or product promotion. TR-909 kick pressure is source-derived only when its source profile and accent-dynamics evidence pass.",
    }
}

fn manifest_artifacts(output_dir: &Path) -> Vec<ManifestArtifact> {
    vec![
        manifest_audio_artifact(
            "tr909_beat_fill",
            output_dir.join("stems/01_tr909_beat_fill.wav"),
        ),
        manifest_audio_artifact(
            "w30_feral_source_chop",
            output_dir.join("stems/02_w30_feral_source_chop.wav"),
        ),
        manifest_audio_artifact(
            "mc202_bass_pressure_stem",
            output_dir.join("stems/03_mc202_bass_pressure.wav"),
        ),
        manifest_audio_artifact(
            "source_first_mix",
            output_dir.join("04_riotbox_source_first_mix.wav"),
        ),
        manifest_audio_artifact(
            "full_grid_mix",
            output_dir.join("05_riotbox_generated_support_mix.wav"),
        ),
        ManifestArtifact::markdown_report("grid_report", &output_dir.join("grid-report.md")),
        ManifestArtifact::markdown_readme("readme", &output_dir.join("README.md")),
    ]
}

fn manifest_audio_artifact(role: &'static str, path: PathBuf) -> ManifestArtifact {
    let metrics_path = metrics_path_for(&path);
    ManifestArtifact::audio_wav(role, &path, Some(&metrics_path))
}

fn manifest_render_metrics(metrics: RenderMetrics) -> ManifestRenderMetrics {
    ManifestRenderMetrics {
        signal: metrics.signal.into(),
        low_band: metrics.low_band.into(),
    }
}
