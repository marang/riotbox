use super::artifact_io::write_pcm16_wav;
use super::config::BEATS_PER_BAR;
use super::config::CHANNEL_COUNT;
use super::config::DEFAULT_BPM;
use super::config::SAMPLE_RATE;
use super::mc202_phrase_grid::mc202_phrase_grid_metrics;
use super::mc202_source_phrase_slot::lane_recipe_source_timing_model;
use super::mc202_source_phrase_slot::mc202_source_phrase_slot_metrics;
use super::report_markdown::render_comparison_markdown;
use super::report_markdown::render_metrics_markdown;
use super::report_model::CaseReport;
use super::report_model::PackCase;
use super::report_model::RenderPair;
use super::signal_delta::rms_delta;
use super::signal_delta::signal_delta_metrics;
use riotbox_audio::runtime::render_mc202_offline;
use riotbox_audio::runtime::render_tr909_offline;
use riotbox_audio::runtime::signal_metrics_with_grid;
use std::fs;
use std::path::Path;

pub(super) fn render_case(
    output_dir: &Path,
    case: PackCase,
    frame_count: usize,
    duration_seconds: f32,
) -> Result<CaseReport, Box<dyn std::error::Error>> {
    let case_dir = output_dir.join(case.id);
    fs::create_dir_all(&case_dir)?;

    let (baseline, candidate) = render_pair(&case.render_pair, frame_count);
    let mc202_phrase_grid = mc202_phrase_grid_metrics(&case.render_pair, &candidate);
    let source_timing = lane_recipe_source_timing_model();
    let mc202_source_phrase_slot =
        mc202_source_phrase_slot_metrics(&case.render_pair, &source_timing);
    let baseline_metrics = signal_metrics_with_grid(
        &baseline,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        DEFAULT_BPM,
        BEATS_PER_BAR,
    );
    let candidate_metrics = signal_metrics_with_grid(
        &candidate,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        DEFAULT_BPM,
        BEATS_PER_BAR,
    );
    let signal_delta_metrics = signal_delta_metrics(&baseline, &candidate);
    let passed = rms_delta(baseline_metrics, candidate_metrics) >= case.min_rms_delta
        && signal_delta_metrics.rms >= case.min_signal_delta_rms
        && mc202_phrase_grid
            .map(|metrics| metrics.passed)
            .unwrap_or(true)
        && mc202_source_phrase_slot
            .as_ref()
            .map(|metrics| metrics.passed)
            .unwrap_or(true);
    let report = CaseReport {
        id: case.id,
        title: case.title,
        recipe_refs: case.recipe_refs,
        baseline_label: case.baseline_label,
        candidate_label: case.candidate_label,
        baseline_metrics,
        candidate_metrics,
        signal_delta_metrics,
        mc202_phrase_grid,
        mc202_source_phrase_slot,
        min_rms_delta: case.min_rms_delta,
        min_signal_delta_rms: case.min_signal_delta_rms,
        passed,
    };

    let baseline_path = case_dir.join("baseline.wav");
    let candidate_path = case_dir.join("candidate.wav");
    write_pcm16_wav(&baseline_path, SAMPLE_RATE, CHANNEL_COUNT, &baseline)?;
    write_pcm16_wav(&candidate_path, SAMPLE_RATE, CHANNEL_COUNT, &candidate)?;

    fs::write(
        case_dir.join("baseline.metrics.md"),
        render_metrics_markdown(&case, "baseline", duration_seconds, baseline_metrics),
    )?;
    fs::write(
        case_dir.join("candidate.metrics.md"),
        render_metrics_markdown(&case, "candidate", duration_seconds, candidate_metrics),
    )?;
    fs::write(
        case_dir.join("comparison.md"),
        render_comparison_markdown(&case, &report),
    )?;

    if !report.passed {
        return Err(format!(
            "{} output delta failed: RMS delta {:.6} / min {:.6}, signal delta RMS {:.6} / min {:.6}, MC-202 phrase grid {:?}, MC-202 source phrase slot {:?}",
            report.id,
            rms_delta(report.baseline_metrics, report.candidate_metrics),
            report.min_rms_delta,
            report.signal_delta_metrics.rms,
            report.min_signal_delta_rms,
            report.mc202_phrase_grid,
            report.mc202_source_phrase_slot
        )
        .into());
    }

    Ok(report)
}

pub(super) fn render_pair(render_pair: &RenderPair, frame_count: usize) -> (Vec<f32>, Vec<f32>) {
    match render_pair {
        RenderPair::Tr909 {
            baseline,
            candidate,
        } => (
            render_tr909_offline(baseline, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
            render_tr909_offline(candidate, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
        ),
        RenderPair::Mc202 {
            baseline,
            candidate,
        } => (
            render_mc202_offline(baseline, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
            render_mc202_offline(candidate, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
        ),
    }
}
