use super::comparison::ComparisonReport;
use std::{fs, path::Path};

pub(super) fn render_report(
    baseline_path: &Path,
    candidate_path: &Path,
    report: &ComparisonReport,
) -> String {
    format!(
        "W-30 preview smoke metrics comparison\n\
         baseline: {}\n\
         candidate: {}\n\
         active_samples: {} -> {} | delta {} | min {} | max {} | {}\n\
         peak_abs: {:.6} -> {:.6} | delta {:.6} | min {:.6} | max {:.6} | {}\n\
         rms: {:.6} -> {:.6} | delta {:.6} | min {:.6} | max {:.6} | {}\n\
         sum: {:.6} -> {:.6} | delta {:.6} | min {:.6} | max {:.6} | {}\n\
         mean_abs: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         zero_crossings: {} -> {} | delta {} | diagnostic\n\
         crest_factor: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         active_sample_ratio: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         silence_ratio: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         dc_offset: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         onset_count: {} -> {} | delta {} | diagnostic\n\
         event_density_per_bar: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         result: {}",
        baseline_path.display(),
        candidate_path.display(),
        report.active_samples.baseline,
        report.active_samples.candidate,
        report.active_samples.delta,
        report.active_samples.min_delta,
        report.active_samples.max_delta,
        status_label(report.active_samples.passed),
        report.peak_abs.baseline,
        report.peak_abs.candidate,
        report.peak_abs.delta,
        report.peak_abs.min_delta,
        report.peak_abs.max_delta,
        status_label(report.peak_abs.passed),
        report.rms.baseline,
        report.rms.candidate,
        report.rms.delta,
        report.rms.min_delta,
        report.rms.max_delta,
        status_label(report.rms.passed),
        report.sum.baseline,
        report.sum.candidate,
        report.sum.delta,
        report.sum.min_delta,
        report.sum.max_delta,
        status_label(report.sum.passed),
        report.mean_abs.baseline,
        report.mean_abs.candidate,
        report.mean_abs.delta,
        report.zero_crossings.baseline,
        report.zero_crossings.candidate,
        report.zero_crossings.delta,
        report.crest_factor.baseline,
        report.crest_factor.candidate,
        report.crest_factor.delta,
        report.active_sample_ratio.baseline,
        report.active_sample_ratio.candidate,
        report.active_sample_ratio.delta,
        report.silence_ratio.baseline,
        report.silence_ratio.candidate,
        report.silence_ratio.delta,
        report.dc_offset.baseline,
        report.dc_offset.candidate,
        report.dc_offset.delta,
        report.onset_count.baseline,
        report.onset_count.candidate,
        report.onset_count.delta,
        report.event_density_per_bar.baseline,
        report.event_density_per_bar.candidate,
        report.event_density_per_bar.delta,
        if report.has_failures() {
            "fail"
        } else {
            "pass"
        }
    )
}

pub(super) fn write_report_markdown(path: &Path, report: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{report}\n"))
}

const fn status_label(passed: bool) -> &'static str {
    if passed { "ok" } else { "drift" }
}
