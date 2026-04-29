#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct SmokeMetrics {
    active_samples: usize,
    peak_abs: f64,
    rms: f64,
    sum: f64,
    mean_abs: f64,
    zero_crossings: usize,
    crest_factor: f64,
    active_sample_ratio: f64,
    silence_ratio: f64,
    dc_offset: f64,
    onset_count: usize,
    event_density_per_bar: f64,
}

#[derive(Serialize)]
struct W30PreviewSmokeManifest {
    schema_version: u32,
    pack_id: &'static str,
    case_id: &'static str,
    artifacts: Vec<ManifestArtifact>,
    limits: DriftLimits,
    metrics: ManifestMetrics,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestMetrics {
    baseline: SmokeMetrics,
    candidate: SmokeMetrics,
    deltas: ManifestMetricDeltas,
}

#[derive(Serialize)]
struct ManifestMetricDeltas {
    active_samples: usize,
    peak_abs: f64,
    rms: f64,
    sum: f64,
    mean_abs: f64,
    zero_crossings: usize,
    crest_factor: f64,
    active_sample_ratio: f64,
    silence_ratio: f64,
    dc_offset: f64,
    onset_count: usize,
    event_density_per_bar: f64,
}

impl SmokeMetrics {
    fn read_from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        Self::parse_markdown(&contents).map_err(|error| {
            format!("failed to parse metrics from {}: {error}", path.display()).into()
        })
    }

    fn parse_markdown(contents: &str) -> Result<Self, String> {
        Ok(Self {
            active_samples: parse_metric_value(contents, "Active samples")?
                .parse::<usize>()
                .map_err(|_| "Active samples must be an integer".to_string())?,
            peak_abs: parse_finite_metric(contents, "Peak abs")?,
            rms: parse_finite_metric(contents, "RMS")?,
            sum: parse_finite_metric(contents, "Sum")?,
            mean_abs: parse_finite_metric(contents, "Mean abs")?,
            zero_crossings: parse_metric_value(contents, "Zero crossings")?
                .parse::<usize>()
                .map_err(|_| "Zero crossings must be an integer".to_string())?,
            crest_factor: parse_finite_metric(contents, "Crest factor")?,
            active_sample_ratio: parse_finite_metric(contents, "Active sample ratio")?,
            silence_ratio: parse_finite_metric(contents, "Silence ratio")?,
            dc_offset: parse_finite_metric(contents, "DC offset")?,
            onset_count: parse_metric_value(contents, "Onset count")?
                .parse::<usize>()
                .map_err(|_| "Onset count must be an integer".to_string())?,
            event_density_per_bar: parse_finite_metric(contents, "Event density per bar")?,
        })
    }
}

fn parse_finite_metric(contents: &str, label: &str) -> Result<f64, String> {
    let parsed = parse_metric_value(contents, label)?
        .parse::<f64>()
        .map_err(|_| format!("{label} must be a finite number"))?;
    if !parsed.is_finite() {
        return Err(format!("{label} must be a finite number"));
    }
    Ok(parsed)
}

fn parse_metric_value(contents: &str, label: &str) -> Result<String, String> {
    let prefix = format!("- {label}: `");
    contents
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix(&prefix)
                .and_then(|rest| rest.split('`').next())
                .map(ToOwned::to_owned)
        })
        .ok_or_else(|| format!("missing metric `{label}`"))
}

#[derive(Clone, Debug, PartialEq)]
struct ComparisonReport {
    active_samples: MetricComparison<usize>,
    peak_abs: MetricComparison<f64>,
    rms: MetricComparison<f64>,
    sum: MetricComparison<f64>,
    mean_abs: DiagnosticMetric<f64>,
    zero_crossings: DiagnosticMetric<usize>,
    crest_factor: DiagnosticMetric<f64>,
    active_sample_ratio: DiagnosticMetric<f64>,
    silence_ratio: DiagnosticMetric<f64>,
    dc_offset: DiagnosticMetric<f64>,
    onset_count: DiagnosticMetric<usize>,
    event_density_per_bar: DiagnosticMetric<f64>,
}

impl ComparisonReport {
    fn has_failures(&self) -> bool {
        !self.active_samples.passed || !self.peak_abs.passed || !self.rms.passed || !self.sum.passed
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MetricComparison<T> {
    baseline: T,
    candidate: T,
    delta: T,
    min_delta: T,
    max_delta: T,
    passed: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct DiagnosticMetric<T> {
    baseline: T,
    candidate: T,
    delta: T,
}

fn compare_metrics(
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
