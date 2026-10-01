//! Existing pack orchestration with explicit policy and artifact dependencies.
use std::fs;

use riotbox_audio::source_audio::SourceAudioCache;

use super::{
    args::Args,
    config::{CHANNEL_COUNT, DEFAULT_BEATS_PER_BAR, SAMPLE_RATE},
    grid::Grid,
    grid_bpm_decision::choose_grid_bpm,
    manifest::write_manifest,
    mc202_bass_pressure::render_mc202_bass_pressure_with_source_contour,
    mc202_source_contour::Mc202SourceContourProfile,
    mix_components::render_mix_with_master_bus_report,
    mix_movement_evidence::all_lane_mix_movement_proof_for_source_contour,
    mix_policy::{
        generated_support_mix_policy_for_source_contour_and_stems,
        render_source_first_mix_with_master_bus_report, source_first_generated_to_source_rms_ratio,
        support_generated_to_source_rms_ratio_for_source_contour,
    },
    output_paths::PackOutputPaths,
    pack_report::PackReport,
    product_stem_contributions::{
        ProductStemContributionRender, render_product_stem_contributions,
        validate_written_product_stem_reconstruction,
    },
    render_measurements::render_metrics,
    source_aware_tr909::derive_source_aware_tr909_profile,
    source_character_window_selection::{
        select_source_character_window, source_character_search_window,
    },
    source_grid_output_drift::source_grid_alignment_report,
    source_timing_analysis::source_timing_analysis_for_source,
    source_timing_groove_policy::{apply_tr909_groove_timing, tr909_groove_timing_policy},
    tr909_kick_pressure::render_tr909_source_support_with_pressure_and_accents,
    tr909_rendered_drum_pressure::{
        Tr909RenderedDrumPressureInput, tr909_rendered_drum_pressure_proof,
    },
    w30_source_chop::{source_chop_preview_from_interleaved, w30_source_loop_closure_proof},
};
// Narrow bridges to still-counted legacy writer/stem owners in the parent.
use super::{
    assert_grid_len, render_w30_source_chop_with_variation, validate_report,
    write_audio_with_metrics, write_readme, write_report,
};

pub(super) fn render_pack(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = args.output_dir();
    let paths = PackOutputPaths::new(&output_dir);
    let stems_dir = output_dir.join("stems");
    let product_stems_dir = stems_dir.join("product");
    fs::create_dir_all(&stems_dir)?;
    fs::create_dir_all(&product_stems_dir)?;

    let source = SourceAudioCache::load_pcm_wav(&args.source_path)?;
    validate_source_format(&source)?;
    paths.reject_source_aliases(&args.source_path)?;
    paths.reject_output_aliases()?;
    let source_timing_analysis = source_timing_analysis_for_source(&source, &args.source_path);
    let timing_readiness = &source_timing_analysis.readiness;
    let grid_bpm = choose_grid_bpm(args, timing_readiness);
    let grid = Grid::new(grid_bpm.bpm, DEFAULT_BEATS_PER_BAR, args.bars)?;

    let requested_source_window = source.window_by_seconds(
        args.source_start_seconds,
        args.source_window_seconds.min(grid.duration_seconds()),
    );
    let source_character_search_window = source_character_search_window(&source, args, &grid);
    let (w30_source_window, source_character_window_selection) = select_source_character_window(
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
    } = render_product_stem_contributions(&tr909, &mc202, &w30, &full_mix, full_mix_policy)?;
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
    let support_generated_to_source_rms_ratio =
        support_generated_to_source_rms_ratio_for_source_contour(
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

    write_audio_with_metrics(&paths.tr909, &tr909, &grid)?;
    write_audio_with_metrics(&paths.w30, &w30, &grid)?;
    write_audio_with_metrics(&paths.mc202, &mc202, &grid)?;
    let product_stem_drums_path = paths.product_drums;
    let product_stem_music_path = paths.product_music;
    let product_stem_bass_path = paths.product_bass;
    write_audio_with_metrics(&product_stem_drums_path, &product_stem_drums, &grid)?;
    write_audio_with_metrics(&product_stem_music_path, &product_stem_music, &grid)?;
    write_audio_with_metrics(&product_stem_bass_path, &product_stem_bass, &grid)?;
    write_audio_with_metrics(&paths.source_first_mix, &source_first_mix, &grid)?;
    write_audio_with_metrics(&paths.full_mix, &full_mix, &grid)?;
    let product_stem_reconstruction = validate_written_product_stem_reconstruction(
        &product_stem_drums_path,
        &product_stem_music_path,
        &product_stem_bass_path,
        &paths.full_mix,
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

    let source_grid_alignment =
        source_grid_alignment_report(&tr909, &mc202, &w30, &full_mix, &grid);
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
    let report_path = paths.report;
    write_report(
        &report_path,
        args,
        &grid,
        report,
        timing_readiness,
        grid_bpm,
    )?;
    write_manifest(
        &paths.manifest,
        args,
        &grid,
        report,
        &source_timing_analysis,
        grid_bpm,
    )?;
    write_readme(
        &paths.readme,
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
