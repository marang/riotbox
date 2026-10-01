use super::config::DriftLimits;
use super::metrics_input::SmokeMetrics;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ComparisonReport {
    pub(super) active_samples: MetricComparison<usize>,
    pub(super) peak_abs: MetricComparison<f64>,
    pub(super) rms: MetricComparison<f64>,
    pub(super) sum: MetricComparison<f64>,
    pub(super) mean_abs: DiagnosticMetric<f64>,
    pub(super) zero_crossings: DiagnosticMetric<usize>,
    pub(super) crest_factor: DiagnosticMetric<f64>,
    pub(super) active_sample_ratio: DiagnosticMetric<f64>,
    pub(super) silence_ratio: DiagnosticMetric<f64>,
    pub(super) dc_offset: DiagnosticMetric<f64>,
    pub(super) onset_count: DiagnosticMetric<usize>,
    pub(super) event_density_per_bar: DiagnosticMetric<f64>,
}

impl ComparisonReport {
    pub(super) fn has_failures(&self) -> bool {
        !self.active_samples.passed || !self.peak_abs.passed || !self.rms.passed || !self.sum.passed
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MetricComparison<T> {
    pub(super) baseline: T,
    pub(super) candidate: T,
    pub(super) delta: T,
    pub(super) min_delta: T,
    pub(super) max_delta: T,
    pub(super) passed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct DiagnosticMetric<T> {
    pub(super) baseline: T,
    pub(super) candidate: T,
    pub(super) delta: T,
}

pub(super) fn compare_metrics(
    baseline: &SmokeMetrics,
    candidate: &SmokeMetrics,
    limits: &DriftLimits,
) -> ComparisonReport {
    let active_delta = baseline.active_samples.abs_diff(candidate.active_samples);

    ComparisonReport {
        active_samples: MetricComparison {
            baseline: baseline.active_samples,
            candidate: candidate.active_samples,
            delta: active_delta,
            min_delta: limits.min_active_samples_delta,
            max_delta: limits.max_active_samples_delta,
            passed: active_delta >= limits.min_active_samples_delta
                && active_delta <= limits.max_active_samples_delta,
        },
        peak_abs: compared_float_metric(
            baseline.peak_abs,
            candidate.peak_abs,
            limits.min_peak_delta,
            limits.max_peak_delta,
        ),
        rms: compared_float_metric(
            baseline.rms,
            candidate.rms,
            limits.min_rms_delta,
            limits.max_rms_delta,
        ),
        sum: compared_float_metric(
            baseline.sum,
            candidate.sum,
            limits.min_sum_delta,
            limits.max_sum_delta,
        ),
        mean_abs: diagnostic_float_metric(baseline.mean_abs, candidate.mean_abs),
        zero_crossings: DiagnosticMetric {
            baseline: baseline.zero_crossings,
            candidate: candidate.zero_crossings,
            delta: baseline.zero_crossings.abs_diff(candidate.zero_crossings),
        },
        crest_factor: diagnostic_float_metric(baseline.crest_factor, candidate.crest_factor),
        active_sample_ratio: diagnostic_float_metric(
            baseline.active_sample_ratio,
            candidate.active_sample_ratio,
        ),
        silence_ratio: diagnostic_float_metric(baseline.silence_ratio, candidate.silence_ratio),
        dc_offset: diagnostic_float_metric(baseline.dc_offset, candidate.dc_offset),
        onset_count: DiagnosticMetric {
            baseline: baseline.onset_count,
            candidate: candidate.onset_count,
            delta: baseline.onset_count.abs_diff(candidate.onset_count),
        },
        event_density_per_bar: diagnostic_float_metric(
            baseline.event_density_per_bar,
            candidate.event_density_per_bar,
        ),
    }
}

fn compared_float_metric(
    baseline: f64,
    candidate: f64,
    min_delta: f64,
    max_delta: f64,
) -> MetricComparison<f64> {
    let delta = (baseline - candidate).abs();
    MetricComparison {
        baseline,
        candidate,
        delta,
        min_delta,
        max_delta,
        passed: float_delta_within_range(delta, min_delta, max_delta),
    }
}

fn diagnostic_float_metric(baseline: f64, candidate: f64) -> DiagnosticMetric<f64> {
    DiagnosticMetric {
        baseline,
        candidate,
        delta: (baseline - candidate).abs(),
    }
}

fn float_delta_within_range(delta: f64, min_delta: f64, max_delta: f64) -> bool {
    let epsilon = f64::EPSILON * 16.0;
    (delta >= min_delta || (min_delta - delta).abs() <= epsilon)
        && (delta <= max_delta || (delta - max_delta).abs() <= epsilon)
}
