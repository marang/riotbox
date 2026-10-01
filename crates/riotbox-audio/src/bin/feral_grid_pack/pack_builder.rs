use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use riotbox_core::source_graph::SourceTimingProbeReadinessReport;
#[cfg(test)]
use riotbox_core::source_graph::SourceTimingProbeBpmCandidatePolicy;

use riotbox_audio::{
    mc202::Mc202ContourHint,
    runtime::{
        MasterBusLimiterReport,
        apply_master_bus_soft_limiter_with_report,
        render_w30_preview_offline, signal_metrics_with_grid,
    },
    source_audio::{
        SourceAudioCache, SourceAudioError, write_interleaved_pcm16_wav,
    },
    tr909::Tr909SourceSupportProfile,
    w30::{
        W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
        W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile,
    },
};

#[cfg(test)]
use riotbox_audio::tr909::{Tr909PatternAdoption, Tr909PhraseVariation, Tr909SourceSupportContext};

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



#[derive(Clone, Copy, Debug)]
struct PackReport {
    source_character_window_selection: SourceCharacterWindowSelection,
    tr909_source_profile: SourceAwareTr909Profile,
    tr909_groove_timing: Tr909GrooveTimingPolicy,
    tr909_kick_pressure: Tr909KickPressureProof,
    tr909_source_accent_dynamics: Tr909SourceAccentDynamicsProof,
    tr909_rendered_drum_pressure: Tr909RenderedDrumPressureProof,
    mc202_bass_pressure: Mc202BassPressureProof,
    mc202_source_contour: Mc202SourceContourProof,
    w30_source_chop_profile: W30SourceChopProfile,
    w30_source_loop_closure: W30SourceLoopClosureProof,
    w30_source_trigger_variation: W30SourceTriggerVariationProof,
    w30_source_slice_choice: W30SourceSliceChoiceProof,
    w30_source_accent_dynamics: W30SourceAccentDynamicsProof,
    all_lane_mix_movement: AllLaneMixMovementProof,
    tr909: RenderMetrics,
    mc202: RenderMetrics,
    w30: RenderMetrics,
    source_first_mix: RenderMetrics,
    full_mix: RenderMetrics,
    product_stem_drums: RenderMetrics,
    product_stem_music: RenderMetrics,
    product_stem_bass: RenderMetrics,
    product_stem_reconstruction: ProductStemReconstructionReport,
    source_first_master_bus_limiter: MasterBusLimiterReport,
    full_mix_master_bus_limiter: MasterBusLimiterReport,
    tr909_source_grid_alignment: SourceGridOutputDriftMetrics,
    mc202_source_grid_alignment: SourceGridOutputDriftMetrics,
    w30_source_grid_alignment: SourceGridOutputDriftMetrics,
    source_grid_output_drift: SourceGridOutputDriftMetrics,
    source_first_generated_to_source_rms_ratio: f32,
    support_generated_to_source_rms_ratio: f32,
}

fn render_pack(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = args.output_dir();
    let stems_dir = output_dir.join("stems");
    let product_stems_dir = stems_dir.join("product");
    fs::create_dir_all(&stems_dir)?;
    fs::create_dir_all(&product_stems_dir)?;

    let source = SourceAudioCache::load_pcm_wav(&args.source_path)?;
    validate_source_format(&source)?;
    let source_timing_analysis = source_timing_analysis_for_source(&source, &args.source_path);
    let timing_readiness = &source_timing_analysis.readiness;
    let grid_bpm = choose_grid_bpm(args, timing_readiness);
    let grid = Grid::new(grid_bpm.bpm, DEFAULT_BEATS_PER_BAR, args.bars)?;

    let requested_source_window = source.window_by_seconds(
        args.source_start_seconds,
        args.source_window_seconds.min(grid.duration_seconds()),
    );
    let source_character_search_window = source_character_search_window(&source, args, &grid);
    let (w30_source_window, source_character_window_selection) =
        select_source_character_window(
            &source,
            requested_source_window,
            source_character_search_window,
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
    let w30_source_loop_closure =
        w30_source_loop_closure_proof(&w30_preview, w30_source_chop_profile);

    let tr909_source_profile = derive_source_aware_tr909_profile(source_window_samples, &grid);
    let tr909_groove_timing =
        tr909_groove_timing_policy(grid_bpm, &source_timing_analysis.groove_evidence);
    let (tr909_source_support, tr909_kick_pressure, tr909_source_accent_dynamics) =
        render_tr909_source_support_with_pressure_and_accents(&grid, tr909_source_profile);
    let tr909 = apply_tr909_groove_timing(&tr909_source_support, tr909_groove_timing);
    let mc202_source_contour_profile =
        Mc202SourceContourProfile::from_source_window(source_window_samples, &grid);
    let (mc202, mc202_bass_pressure, mc202_source_contour) =
        render_mc202_bass_pressure_with_source_contour(
            &grid,
            tr909_source_profile,
            mc202_source_contour_profile,
        );
    let (w30, w30_source_trigger_variation, w30_source_slice_choice, w30_source_accent_dynamics) =
        render_w30_source_chop_with_variation(&grid, &w30_preview);
    let (source_first_mix, source_first_master_bus_limiter) =
        render_source_first_mix_with_master_bus_report(&tr909, &mc202, &w30, &grid);
    let full_mix_policy = generated_support_mix_policy_for_source_contour_and_stems(
        &tr909,
        &mc202,
        &w30,
        &grid,
        mc202_source_contour_profile,
    );
    let (full_mix, full_mix_master_bus_limiter) =
        render_mix_with_master_bus_report(&tr909, &mc202, &w30, full_mix_policy);
    let ProductStemContributionRender {
        drums: product_stem_drums,
        music: product_stem_music,
        bass: product_stem_bass,
    } = render_product_stem_contributions(
        &tr909,
        &mc202,
        &w30,
        &full_mix,
        full_mix_policy,
    )?;
    let all_lane_mix_movement = all_lane_mix_movement_proof_for_source_contour(
        &tr909,
        &mc202,
        &w30,
        &source_first_mix,
        &full_mix,
        &grid,
        mc202_source_contour_profile,
    );
    let source_first_generated_to_source_rms_ratio =
        source_first_generated_to_source_rms_ratio(&tr909, &mc202, &w30, &grid);
    let support_generated_to_source_rms_ratio = support_generated_to_source_rms_ratio_for_source_contour(
        &tr909,
        &mc202,
        &w30,
        &grid,
        mc202_source_contour_profile,
    );

    assert_grid_len("tr909", &tr909, &grid);
    assert_grid_len("mc202", &mc202, &grid);
    assert_grid_len("w30", &w30, &grid);
    assert_grid_len("source_first_mix", &source_first_mix, &grid);
    assert_grid_len("full_mix", &full_mix, &grid);
    assert_grid_len("product_stem_drums", &product_stem_drums, &grid);
    assert_grid_len("product_stem_music", &product_stem_music, &grid);
    assert_grid_len("product_stem_bass", &product_stem_bass, &grid);

    write_audio_with_metrics(&stems_dir.join("01_tr909_beat_fill.wav"), &tr909, &grid)?;
    write_audio_with_metrics(&stems_dir.join("02_w30_feral_source_chop.wav"), &w30, &grid)?;
    write_audio_with_metrics(&stems_dir.join("03_mc202_bass_pressure.wav"), &mc202, &grid)?;
    let product_stem_drums_path = product_stems_dir.join("01_stem_drums.wav");
    let product_stem_music_path = product_stems_dir.join("02_stem_music.wav");
    let product_stem_bass_path = product_stems_dir.join("03_stem_bass.wav");
    write_audio_with_metrics(&product_stem_drums_path, &product_stem_drums, &grid)?;
    write_audio_with_metrics(&product_stem_music_path, &product_stem_music, &grid)?;
    write_audio_with_metrics(&product_stem_bass_path, &product_stem_bass, &grid)?;
    write_audio_with_metrics(
        &output_dir.join("04_riotbox_source_first_mix.wav"),
        &source_first_mix,
        &grid,
    )?;
    write_audio_with_metrics(
        &output_dir.join("05_riotbox_generated_support_mix.wav"),
        &full_mix,
        &grid,
    )?;
    let product_stem_reconstruction = validate_written_product_stem_reconstruction(
        &product_stem_drums_path,
        &product_stem_music_path,
        &product_stem_bass_path,
        &output_dir.join("05_riotbox_generated_support_mix.wav"),
    )?;
    if !product_stem_reconstruction.passed {
        return Err(format!(
            "written product stems do not reconstruct full_grid_mix: max_abs_error {:.8} > {:.8} or rms_error {:.8} > {:.8}",
            product_stem_reconstruction.max_abs_error,
            product_stem_reconstruction.max_allowed_abs_error,
            product_stem_reconstruction.rms_error,
            product_stem_reconstruction.max_allowed_rms_error,
        )
        .into());
    }

    let source_grid_alignment = source_grid_alignment_report(&tr909, &mc202, &w30, &full_mix, &grid);
    let tr909_rendered_drum_pressure =
        tr909_rendered_drum_pressure_proof(Tr909RenderedDrumPressureInput {
            source_profile: tr909_source_profile,
            tr909_metrics: render_metrics(&tr909, &grid),
            full_mix_metrics: render_metrics(&full_mix, &grid),
            kick_pressure: tr909_kick_pressure,
            accent_dynamics: tr909_source_accent_dynamics,
            all_lane_mix_movement,
            tr909_source_grid_alignment: source_grid_alignment.tr909_source_grid_alignment,
            source_first_generated_to_source_rms_ratio,
            support_generated_to_source_rms_ratio,
        });
    let report = PackReport {
        source_character_window_selection,
        tr909_source_profile,
        tr909_groove_timing,
        tr909_kick_pressure,
        tr909_source_accent_dynamics,
        tr909_rendered_drum_pressure,
        mc202_bass_pressure,
        mc202_source_contour,
        w30_source_chop_profile,
        w30_source_loop_closure,
        w30_source_trigger_variation,
        w30_source_slice_choice,
        w30_source_accent_dynamics,
        all_lane_mix_movement,
        tr909: render_metrics(&tr909, &grid),
        mc202: render_metrics(&mc202, &grid),
        w30: render_metrics(&w30, &grid),
        source_first_mix: render_metrics(&source_first_mix, &grid),
        full_mix: render_metrics(&full_mix, &grid),
        product_stem_drums: render_metrics(&product_stem_drums, &grid),
        product_stem_music: render_metrics(&product_stem_music, &grid),
        product_stem_bass: render_metrics(&product_stem_bass, &grid),
        product_stem_reconstruction,
        source_first_master_bus_limiter,
        full_mix_master_bus_limiter,
        tr909_source_grid_alignment: source_grid_alignment.tr909_source_grid_alignment,
        mc202_source_grid_alignment: source_grid_alignment.mc202_source_grid_alignment,
        w30_source_grid_alignment: source_grid_alignment.w30_source_grid_alignment,
        source_grid_output_drift: source_grid_alignment.source_grid_output_drift,
        source_first_generated_to_source_rms_ratio,
        support_generated_to_source_rms_ratio,
    };
    validate_report(&report)?;
    let report_path = output_dir.join("grid-report.md");
    write_report(&report_path, args, &grid, report, timing_readiness, grid_bpm)?;
    write_manifest(
        &output_dir.join("manifest.json"),
        args,
        &grid,
        report,
        &source_timing_analysis,
        grid_bpm,
    )?;
    write_readme(
        &output_dir,
        args,
        &grid,
        grid_bpm,
        timing_readiness,
        source_character_window_selection,
    )?;

    Ok(())
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
