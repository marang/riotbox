fn write_report(
    path: &Path,
    args: &Args,
    grid: &Grid,
    report: PackReport,
    timing_readiness: &SourceTimingProbeReadinessReport,
    grid_bpm: GridBpmDecision,
) -> std::io::Result<()> {
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
             {}\
             - TR-909 source reason: `{}`\n\
             - TR-909 support profile: `{}` / pattern `{}` / phrase `{}`\n\
             - TR-909 groove timing: `{}` applied `{}` offset `{:.3}` ms subdivision `{}`\n\
             - TR-909 source low/high energy: `{:.6}` / `{:.6}`\n\
             - TR-909 kick pressure: `{}` anchors `{}` gain `{:.6}` low-band ratio `{:.6}` delta `{:.6}` peak `{:.6}`\n\
             - MC-202 bass pressure: `{}` mode `{}` shape `{}` budget `{}` RMS `{:.6}` low-band `{:.6}` touch `{:.3}` level `{:.3}` peak `{:.6}`\n\
             - W-30 source-chop reason: `{}`\n\
             - W-30 source-chop preview RMS: `{:.6}` from source RMS `{:.6}` with gain `{:.6}`\n\
             - W-30 source-loop closure: `{}` edge delta `{:.6}` (max `{:.6}`), edge abs `{:.6}` (max `{:.6}`)\n\
             - W-30 trigger variation: `{}` triggers `{}` beat anchors `{}` offbeats `{}` skipped beat anchors `{}` distinct bar patterns `{}` max quantized offset `{:.6}` ms\n\
             - W-30 slice choice: `{}` unique offsets `{}` span `{}` samples min `{}` max `{}`\n\
             - Source-first generated/source RMS ratio: `{:.6}` (max `{MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO:.6}`)\n\
             - Support generated/source RMS ratio: `{:.6}` (max `{MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO:.6}`)\n\
             - Generated-support mix low-band RMS: `{:.6}`\n\
             - Minimum full mix low-band RMS: `{MIN_LOW_BAND_RMS:.6}`\n\
             - TR-909 source-grid alignment hit ratio: `{:.6}` (min `{SOURCE_GRID_OUTPUT_MIN_HIT_RATIO:.6}`), max peak offset `{:.3}` ms\n\
             - W-30 source-grid alignment hit ratio: `{:.6}` (min `{SOURCE_GRID_OUTPUT_MIN_HIT_RATIO:.6}`), max peak offset `{:.3}` ms\n\
             - Source-grid output hit ratio: `{:.6}` (min `{SOURCE_GRID_OUTPUT_MIN_HIT_RATIO:.6}`), max peak offset `{:.3}` ms\n\
             - Result: `pass`\n\n\
             | Stem | RMS | Peak abs | Low-band RMS | Active samples | Bar similarity | Identical bar run | Low energy | Mid energy | High energy |\n\
             | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n\
             | TR-909 source support | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | MC-202 bass pressure | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | W-30 Feral source chop | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | Source-first mix | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | Generated-support mix | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n",
            args.source_path.display(),
            grid.bpm,
            grid.bars,
            grid.beats_per_bar,
            grid.total_beats,
            grid.total_frames,
            grid.duration_seconds(),
            source_timing_report_lines(timing_readiness, grid_bpm),
            report.tr909_source_profile.reason,
            report.tr909_source_profile.support_profile.label(),
            report.tr909_source_profile.pattern_adoption.label(),
            report.tr909_source_profile.phrase_variation.label(),
            report.tr909_groove_timing.reason,
            report.tr909_groove_timing.applied,
            report.tr909_groove_timing.offset_ms,
            report.tr909_groove_timing.source_subdivision.unwrap_or("none"),
            report.tr909_source_profile.low_band_energy_ratio,
            report.tr909_source_profile.high_band_energy_ratio,
            report.tr909_kick_pressure.reason,
            report.tr909_kick_pressure.anchor_count,
            report.tr909_kick_pressure.pressure_gain,
            report.tr909_kick_pressure.low_band_rms_ratio,
            report.tr909_kick_pressure.low_band_rms_delta,
            report.tr909_kick_pressure.post_peak_abs,
            report.mc202_bass_pressure.reason,
            report.mc202_bass_pressure.mode.label(),
            report.mc202_bass_pressure.phrase_shape.label(),
            report.mc202_bass_pressure.note_budget.label(),
            report.mc202_bass_pressure.signal_rms,
            report.mc202_bass_pressure.low_band_rms,
            report.mc202_bass_pressure.touch,
            report.mc202_bass_pressure.music_bus_level,
            report.mc202_bass_pressure.peak_abs,
            report.w30_source_chop_profile.reason,
            report.w30_source_chop_profile.preview_rms,
            report.w30_source_chop_profile.source_window_rms,
            report.w30_source_chop_profile.gain,
            report.w30_source_loop_closure.reason,
            report.w30_source_loop_closure.edge_delta_abs,
            report.w30_source_loop_closure.max_allowed_edge_delta_abs,
            report.w30_source_loop_closure.edge_abs_max,
            report.w30_source_loop_closure.max_allowed_edge_abs,
            report.w30_source_trigger_variation.reason,
            report.w30_source_trigger_variation.trigger_count,
            report.w30_source_trigger_variation.beat_anchor_trigger_count,
            report.w30_source_trigger_variation.offbeat_trigger_count,
            report.w30_source_trigger_variation.skipped_beat_anchor_count,
            report.w30_source_trigger_variation.distinct_bar_pattern_count,
            report.w30_source_trigger_variation.max_quantized_offset_ms,
            report.w30_source_slice_choice.reason,
            report.w30_source_slice_choice.unique_source_offset_count,
            report.w30_source_slice_choice.selected_offset_span_samples,
            report.w30_source_slice_choice.min_selected_offset_samples,
            report.w30_source_slice_choice.max_selected_offset_samples,
            report.source_first_generated_to_source_rms_ratio,
            report.support_generated_to_source_rms_ratio,
            report.full_mix.low_band.rms,
            report.tr909_source_grid_alignment.hit_ratio,
            report.tr909_source_grid_alignment.max_peak_offset_ms,
            report.w30_source_grid_alignment.hit_ratio,
            report.w30_source_grid_alignment.max_peak_offset_ms,
            report.source_grid_output_drift.hit_ratio,
            report.source_grid_output_drift.max_peak_offset_ms,
            report.tr909.signal.rms,
            report.tr909.signal.peak_abs,
            report.tr909.low_band.rms,
            report.tr909.signal.active_samples,
            report.tr909.bar_variation.bar_similarity,
            report.tr909.bar_variation.identical_bar_run_length,
            report.tr909.spectral_energy.low_band_energy_ratio,
            report.tr909.spectral_energy.mid_band_energy_ratio,
            report.tr909.spectral_energy.high_band_energy_ratio,
            report.mc202.signal.rms,
            report.mc202.signal.peak_abs,
            report.mc202.low_band.rms,
            report.mc202.signal.active_samples,
            report.mc202.bar_variation.bar_similarity,
            report.mc202.bar_variation.identical_bar_run_length,
            report.mc202.spectral_energy.low_band_energy_ratio,
            report.mc202.spectral_energy.mid_band_energy_ratio,
            report.mc202.spectral_energy.high_band_energy_ratio,
            report.w30.signal.rms,
            report.w30.signal.peak_abs,
            report.w30.low_band.rms,
            report.w30.signal.active_samples,
            report.w30.bar_variation.bar_similarity,
            report.w30.bar_variation.identical_bar_run_length,
            report.w30.spectral_energy.low_band_energy_ratio,
            report.w30.spectral_energy.mid_band_energy_ratio,
            report.w30.spectral_energy.high_band_energy_ratio,
            report.source_first_mix.signal.rms,
            report.source_first_mix.signal.peak_abs,
            report.source_first_mix.low_band.rms,
            report.source_first_mix.signal.active_samples,
            report.source_first_mix.bar_variation.bar_similarity,
            report.source_first_mix.bar_variation.identical_bar_run_length,
            report.source_first_mix.spectral_energy.low_band_energy_ratio,
            report.source_first_mix.spectral_energy.mid_band_energy_ratio,
            report.source_first_mix.spectral_energy.high_band_energy_ratio,
            report.full_mix.signal.rms,
            report.full_mix.signal.peak_abs,
            report.full_mix.low_band.rms,
            report.full_mix.signal.active_samples,
            report.full_mix.bar_variation.bar_similarity,
            report.full_mix.bar_variation.identical_bar_run_length,
            report.full_mix.spectral_energy.low_band_energy_ratio,
            report.full_mix.spectral_energy.mid_band_energy_ratio,
            report.full_mix.spectral_energy.high_band_energy_ratio
        ),
    )
}

fn write_readme(
    output_dir: &Path,
    args: &Args,
    grid: &Grid,
    grid_bpm: GridBpmDecision,
    timing_readiness: &SourceTimingProbeReadinessReport,
) -> std::io::Result<()> {
    fs::write(
        output_dir.join("README.md"),
        format!(
            "# Feral Grid Demo Pack\n\n\
             This pack is the current Riotbox offline QA path for checking a musical grid,\n\
             not only a log path. All stems use the same BPM, bar count, and frame count.\n\n\
             ## Grid\n\n\
             - Source: `{}`\n\
             - BPM: `{:.3}`\n\
             - BPM source: `{}`\n\
             - BPM decision reason: `{}`\n\
             - Source timing BPM delta: `{}`\n\
             - Source timing readiness: `{}`\n\
             - Source timing downbeat: `{}`\n\
             - Source timing phrase: `{}`\n\
             - Source timing warnings: `{}`\n\
             - Bars: `{}`\n\
             - Beats per bar: `{}`\n\
             - Duration: `{:.3}s`\n\
             - Source window start: `{:.3}s`\n\
             - W-30 source window length: `{:.3}s`\n\n\
             ## Files\n\n\
             - `stems/01_tr909_beat_fill.wav`: source-aware TR-909 support rendered on the same grid.\n\
             - `stems/02_w30_feral_source_chop.wav`: W-30 source-backed Feral chop with articulate source-window selection and bounded loudness normalization.\n\
             - `stems/03_mc202_bass_pressure.wav`: generated MC-202 bass-pressure support rendered on the same grid.\n\
             - `04_riotbox_source_first_mix.wav`: listen here first; source-backed W-30 leads and generated support stays secondary.\n\
             - `05_riotbox_generated_support_mix.wav`: generated-support mix; TR-909 and MC-202 add low-end and movement without proving source extraction by themselves.\n\
             - `grid-report.md`: timing, source-timing readiness, and output metrics.\n\
             - `manifest.json`: machine-readable pack metadata, artifact paths, thresholds, and key metrics.\n\
\n\
             ## Current Limit\n\n\
             This is an offline QA/listening pack. It proves the render seams can align musically,\n\
             but it does not yet mean the live TUI mixer exposes this whole arrangement path directly.\n",
            args.source_path.display(),
            grid.bpm,
            grid_bpm_source_label(grid_bpm.source),
            grid_bpm_decision_reason_label(grid_bpm.reason),
            grid_bpm
                .source_delta_bpm
                .map(|delta| format!("{delta:.3}"))
                .unwrap_or_else(|| "unknown".to_string()),
            source_timing_readiness_line(timing_readiness),
            source_timing_downbeat_line(timing_readiness),
            source_timing_phrase_line(timing_readiness),
            source_timing_warnings_line(timing_readiness),
            grid.bars,
            grid.beats_per_bar,
            grid.duration_seconds(),
            args.source_start_seconds,
            args.source_window_seconds.min(grid.duration_seconds())
        ),
    )
}

fn source_timing_report_lines(
    report: &SourceTimingProbeReadinessReport,
    grid_bpm: GridBpmDecision,
) -> String {
    format!(
        "- Source timing readiness: `{}`\n\
         - Grid BPM decision reason: `{}`\n\
         - Source timing BPM: `primary={} delta={} agrees={}`\n\
         - Source timing downbeat: `{}`\n\
         - Source timing phrase: `{}`\n\
         - Source timing warnings: `{}`\n",
        source_timing_readiness_line(report),
        grid_bpm_decision_reason_label(grid_bpm.reason),
        optional_f32(report.primary_bpm),
        optional_f32(grid_bpm.source_delta_bpm),
        optional_bool(source_timing_bpm_agrees(grid_bpm.source_delta_bpm)),
        source_timing_downbeat_line(report),
        source_timing_phrase_line(report),
        source_timing_warnings_line(report)
    )
}

fn source_timing_readiness_line(report: &SourceTimingProbeReadinessReport) -> String {
    format!(
        "{} manual_confirm={}",
        readiness_status_label(report.readiness),
        bool_label(report.requires_manual_confirm)
    )
}

fn source_timing_downbeat_line(report: &SourceTimingProbeReadinessReport) -> String {
    format!(
        "{} offset={}",
        downbeat_evidence_status_label(report.downbeat_status),
        report
            .primary_downbeat_offset_beats
            .map_or_else(|| "unknown".to_string(), |offset| offset.to_string())
    )
}

fn source_timing_phrase_line(report: &SourceTimingProbeReadinessReport) -> String {
    format!(
        "{} confidence={} drift={} alternates={}",
        phrase_status_label(report.phrase_status),
        confidence_result_label(report.confidence_result),
        drift_status_label(report.drift_status),
        report.alternate_evidence_count
    )
}

fn source_timing_warnings_line(report: &SourceTimingProbeReadinessReport) -> String {
    if report.warning_codes.is_empty() {
        return "none".to_string();
    }

    report
        .warning_codes
        .iter()
        .map(|code| format!("{code:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn optional_f32(value: Option<f32>) -> String {
    value.map_or_else(|| "unknown".to_string(), |value| format!("{value:.3}"))
}

fn optional_bool(value: Option<bool>) -> &'static str {
    value.map_or("unknown", bool_label)
}

fn bool_label(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
