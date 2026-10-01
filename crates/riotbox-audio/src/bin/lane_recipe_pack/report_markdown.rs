use super::args::Args;
use super::config::CHANNEL_COUNT;
use super::config::PACK_ID;
use super::config::SAMPLE_RATE;
use super::report_model::CaseReport;
use super::report_model::PackCase;
use super::signal_delta::rms_delta;
use riotbox_audio::runtime::OfflineAudioMetrics;
use std::path::Path;

pub(super) fn render_metrics_markdown(
    case: &PackCase,
    role: &str,
    duration_seconds: f32,
    metrics: OfflineAudioMetrics,
) -> String {
    let label = if role == "baseline" {
        case.baseline_label
    } else {
        case.candidate_label
    };
    format!(
        "# Lane Recipe Listening Metrics\n\n\
         - Pack: `{PACK_ID}`\n\
         - Case: `{}`\n\
         - Title: `{}`\n\
         - Recipes: `{}`\n\
         - Role: `{role}`\n\
         - Label: `{label}`\n\
         - Sample rate: `{SAMPLE_RATE}`\n\
         - Channels: `{CHANNEL_COUNT}`\n\
         - Duration seconds: `{duration_seconds:.3}`\n\
         - Active samples: `{}`\n\
         - Peak abs: `{:.6}`\n\
         - RMS: `{:.6}`\n\
         - Sum: `{:.6}`\n\
         - Mean abs: `{:.6}`\n\
         - Zero crossings: `{}`\n\
         - Crest factor: `{:.6}`\n\
             - Active sample ratio: `{:.6}`\n\
             - Silence ratio: `{:.6}`\n\
             - DC offset: `{:.6}`\n\
             - Onset count: `{}`\n\
             - Event density per bar: `{:.6}`\n",
        case.id,
        case.title,
        case.recipe_refs,
        metrics.active_samples,
        metrics.peak_abs,
        metrics.rms,
        metrics.sum,
        metrics.mean_abs,
        metrics.zero_crossings,
        metrics.crest_factor,
        metrics.active_sample_ratio,
        metrics.silence_ratio,
        metrics.dc_offset,
        metrics.onset_count,
        metrics.event_density_per_bar
    )
}

pub(super) fn render_comparison_markdown(case: &PackCase, report: &CaseReport) -> String {
    let baseline = report.baseline_metrics;
    let candidate = report.candidate_metrics;
    let active_delta = baseline.active_samples.abs_diff(candidate.active_samples);
    let peak_delta = (baseline.peak_abs - candidate.peak_abs).abs();
    let rms_delta = rms_delta(baseline, candidate);
    let sum_delta = (baseline.sum - candidate.sum).abs();
    let mean_abs_delta = (baseline.mean_abs - candidate.mean_abs).abs();
    let zero_crossings_delta = baseline.zero_crossings.abs_diff(candidate.zero_crossings);
    let crest_factor_delta = (baseline.crest_factor - candidate.crest_factor).abs();
    let active_ratio_delta = (baseline.active_sample_ratio - candidate.active_sample_ratio).abs();
    let silence_ratio_delta = (baseline.silence_ratio - candidate.silence_ratio).abs();
    let dc_offset_delta = (baseline.dc_offset - candidate.dc_offset).abs();
    let onset_count_delta = baseline.onset_count.abs_diff(candidate.onset_count);
    let event_density_delta =
        (baseline.event_density_per_bar - candidate.event_density_per_bar).abs();
    let signal_delta = report.signal_delta_metrics;
    let mc202_phrase_grid = report
        .mc202_phrase_grid
        .map(|metrics| {
            format!(
                "\n\
                 ## MC-202 Phrase/Grid Timing\n\n\
                 - Result: `{}`\n\
                 - Position beats: `{:.3}`\n\
                 - Starts on phrase boundary: `{}`\n\
                 - Candidate onset count: `{}`\n\
                 - Grid-aligned onset count: `{}`\n\
                 - Hit ratio: `{:.6}`\n\
                 - Max onset offset ms: `{:.3}`\n\
                 - Max allowed onset offset ms: `{:.3}`\n",
                if metrics.passed { "pass" } else { "fail" },
                metrics.position_beats,
                metrics.starts_on_phrase_boundary,
                metrics.candidate_onset_count,
                metrics.grid_aligned_onset_count,
                metrics.hit_ratio,
                metrics.max_onset_offset_ms,
                metrics.max_allowed_onset_offset_ms
            )
        })
        .unwrap_or_default();
    let mc202_source_phrase_slot = report
        .mc202_source_phrase_slot
        .as_ref()
        .map(|metrics| {
            format!(
                "\n\
                 ## MC-202 Source Phrase Slot\n\n\
                 - Result: `{}`\n\
                 - Contract: `{}`\n\
                 - Source hypothesis: `{}`\n\
                 - Phrase grid available: `{}`\n\
                 - Candidate position beats: `{:.3}`\n\
                 - Candidate bar index: `{}`\n\
                 - Phrase index: `{}`\n\
                 - Phrase bars: `{}`\n\
                 - Starts on source phrase boundary: `{}`\n",
                if metrics.passed { "pass" } else { "fail" },
                metrics.contract,
                metrics.source_hypothesis_id.as_deref().unwrap_or("unknown"),
                metrics.phrase_grid_available,
                metrics.candidate_position_beats,
                metrics.candidate_bar_index,
                metrics
                    .phrase_index
                    .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
                match (metrics.phrase_start_bar, metrics.phrase_end_bar) {
                    (Some(start), Some(end)) => format!("{start}-{end}"),
                    _ => "unknown".to_string(),
                },
                metrics.starts_on_source_phrase_boundary
            )
        })
        .unwrap_or_default();

    format!(
        "# Lane Recipe Listening Comparison\n\n\
         - Pack: `{PACK_ID}`\n\
         - Case: `{}`\n\
         - Title: `{}`\n\
         - Recipes: `{}`\n\
         - Baseline: `{}`\n\
         - Candidate: `{}`\n\
         - Minimum RMS delta: `{:.6}`\n\
         - Signal delta RMS: `{:.6}`\n\
         - Minimum signal delta RMS: `{:.6}`\n\
         - Signal delta peak abs: `{:.6}`\n\
         - Result: `{}`\n\
         - Note: {}\n\n\
         | Metric | Baseline | Candidate | Delta |\n\
         | --- | ---: | ---: | ---: |\n\
         | active_samples | {} | {} | {} |\n\
         | peak_abs | {:.6} | {:.6} | {:.6} |\n\
         | rms | {:.6} | {:.6} | {:.6} |\n\
         | sum | {:.6} | {:.6} | {:.6} |\n\
         | mean_abs | {:.6} | {:.6} | {:.6} |\n\
         | zero_crossings | {} | {} | {} |\n\
         | crest_factor | {:.6} | {:.6} | {:.6} |\n\
         | active_sample_ratio | {:.6} | {:.6} | {:.6} |\n\
         | silence_ratio | {:.6} | {:.6} | {:.6} |\n\
         | dc_offset | {:.6} | {:.6} | {:.6} |\n\
         | onset_count | {} | {} | {} |\n\
         | event_density_per_bar | {:.6} | {:.6} | {:.6} |\n{}{}",
        case.id,
        case.title,
        case.recipe_refs,
        case.baseline_label,
        case.candidate_label,
        report.min_rms_delta,
        signal_delta.rms,
        report.min_signal_delta_rms,
        signal_delta.peak_abs,
        if report.passed { "pass" } else { "fail" },
        case.note,
        baseline.active_samples,
        candidate.active_samples,
        active_delta,
        baseline.peak_abs,
        candidate.peak_abs,
        peak_delta,
        baseline.rms,
        candidate.rms,
        rms_delta,
        baseline.sum,
        candidate.sum,
        sum_delta,
        baseline.mean_abs,
        candidate.mean_abs,
        mean_abs_delta,
        baseline.zero_crossings,
        candidate.zero_crossings,
        zero_crossings_delta,
        baseline.crest_factor,
        candidate.crest_factor,
        crest_factor_delta,
        baseline.active_sample_ratio,
        candidate.active_sample_ratio,
        active_ratio_delta,
        baseline.silence_ratio,
        candidate.silence_ratio,
        silence_ratio_delta,
        baseline.dc_offset,
        candidate.dc_offset,
        dc_offset_delta,
        baseline.onset_count,
        candidate.onset_count,
        onset_count_delta,
        baseline.event_density_per_bar,
        candidate.event_density_per_bar,
        event_density_delta,
        mc202_phrase_grid,
        mc202_source_phrase_slot
    )
}

pub(super) fn render_pack_summary(
    args: &Args,
    output_dir: &Path,
    reports: &[CaseReport],
) -> String {
    let mut summary = format!(
        "# Lane Recipe Listening Pack\n\n\
         - Pack: `{PACK_ID}`\n\
         - Date: `{}`\n\
         - Output dir: `{}`\n\
         - Duration seconds: `{:.3}`\n\n\
         This pack is the first local recipe-level audio proof outside the W-30 source-preview path.\n\
         It renders bounded TR-909, MC-202, and Scene-coupled support comparisons as WAV files plus sibling metrics and `manifest.json`. All cases are labeled `primitive_renderer`; this pack proves renderer contrasts, not source-derived musical intelligence.\n\n\
         ## Cases\n\n\
         | Case | Active delta | Peak delta | RMS delta | Min RMS delta | Signal delta RMS | Min signal delta RMS | Sum delta |\n\
         | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n",
        args.date,
        output_dir.display(),
        args.duration_seconds
    );

    for report in reports {
        let active_delta = report
            .baseline_metrics
            .active_samples
            .abs_diff(report.candidate_metrics.active_samples);
        let peak_delta =
            (report.baseline_metrics.peak_abs - report.candidate_metrics.peak_abs).abs();
        let rms_delta = rms_delta(report.baseline_metrics, report.candidate_metrics);
        let sum_delta = (report.baseline_metrics.sum - report.candidate_metrics.sum).abs();
        summary.push_str(&format!(
            "| `{}` | {} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} |\n",
            report.id,
            active_delta,
            peak_delta,
            rms_delta,
            report.min_rms_delta,
            report.signal_delta_metrics.rms,
            report.min_signal_delta_rms,
            sum_delta
        ));
    }

    summary.push_str(
        "\n## Current MC-202 Status\n\n\
         MC-202 now has explicit offline audio cases for touch energy, pressure, instigator, phrase mutation, note budget, and source-section contour hints. These cases prove bounded renderable contrasts for the current `g`, `P`, `I`, `G`, `<`, and `>` gestures without injecting a hardcoded answer hook; `a` remains control-path only until source-derived phrase planning exists.\n\n\
         ## Current Scene Status\n\n\
         Scene Brain is represented here only through the current TR-909 `scene_target` support-accent seam. This does not claim a finished Scene transition engine.\n",
    );

    summary
}
