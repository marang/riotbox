use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

use serde::Serialize;

use riotbox_audio::listening_manifest::{
    LISTENING_MANIFEST_SCHEMA_VERSION, ListeningPackArtifact as ManifestArtifact,
    write_manifest_json,
};

const DEFAULT_DATE: &str = "local";
const PACK_ID: &str = "w30-preview-smoke";
const CASE_ID: &str = "raw_capture_source_window_preview";
const DEFAULT_MAX_ACTIVE_SAMPLES_DELTA: usize = 0;
const DEFAULT_MAX_PEAK_DELTA: f64 = 0.000001;
const DEFAULT_MAX_RMS_DELTA: f64 = 0.000001;
const DEFAULT_MAX_SUM_DELTA: f64 = 0.000001;
const DEFAULT_MIN_ACTIVE_SAMPLES_DELTA: usize = 0;
const DEFAULT_MIN_PEAK_DELTA: f64 = 0.0;
const DEFAULT_MIN_RMS_DELTA: f64 = 0.0;
const DEFAULT_MIN_SUM_DELTA: f64 = 0.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    let baseline = SmokeMetrics::read_from_path(&args.baseline_metrics_path)?;
    let candidate = SmokeMetrics::read_from_path(&args.candidate_metrics_path)?;
    let report = compare_metrics(&baseline, &candidate, &args.limits);
    let rendered_report = render_report(
        &args.baseline_metrics_path,
        &args.candidate_metrics_path,
        &report,
    );

    println!("{rendered_report}");
    write_report_markdown(&args.report_path, &rendered_report)?;
    println!("wrote {}", args.report_path.display());
    write_manifest(&args, baseline, candidate, &report)?;
    println!(
        "wrote {}",
        manifest_path_for_report_path(&args.report_path).display()
    );

    if report.has_failures() {
        process::exit(2);
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
struct Args {
    baseline_metrics_path: PathBuf,
    candidate_metrics_path: PathBuf,
    report_path: PathBuf,
    limits: DriftLimits,
    show_help: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct DriftLimits {
    min_active_samples_delta: usize,
    max_active_samples_delta: usize,
    min_peak_delta: f64,
    max_peak_delta: f64,
    min_rms_delta: f64,
    max_rms_delta: f64,
    min_sum_delta: f64,
    max_sum_delta: f64,
}

impl Default for DriftLimits {
    fn default() -> Self {
        Self {
            min_active_samples_delta: DEFAULT_MIN_ACTIVE_SAMPLES_DELTA,
            max_active_samples_delta: DEFAULT_MAX_ACTIVE_SAMPLES_DELTA,
            min_peak_delta: DEFAULT_MIN_PEAK_DELTA,
            max_peak_delta: DEFAULT_MAX_PEAK_DELTA,
            min_rms_delta: DEFAULT_MIN_RMS_DELTA,
            max_rms_delta: DEFAULT_MAX_RMS_DELTA,
            min_sum_delta: DEFAULT_MIN_SUM_DELTA,
            max_sum_delta: DEFAULT_MAX_SUM_DELTA,
        }
    }
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut date = DEFAULT_DATE.to_string();
        let mut baseline_override = None;
        let mut candidate_override = None;
        let mut report_override = None;
        let mut limits = DriftLimits::default();
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--date" => {
                    let Some(value) = args.next() else {
                        return Err("--date requires a value".into());
                    };
                    date = value;
                }
                "--baseline" => {
                    let Some(value) = args.next() else {
                        return Err("--baseline requires a path".into());
                    };
                    baseline_override = Some(PathBuf::from(value));
                }
                "--candidate" => {
                    let Some(value) = args.next() else {
                        return Err("--candidate requires a path".into());
                    };
                    candidate_override = Some(PathBuf::from(value));
                }
                "--report" => {
                    let Some(value) = args.next() else {
                        return Err("--report requires a path".into());
                    };
                    report_override = Some(PathBuf::from(value));
                }
                "--max-active-samples-delta" => {
                    let Some(value) = args.next() else {
                        return Err("--max-active-samples-delta requires a value".into());
                    };
                    limits.max_active_samples_delta = value.parse::<usize>().map_err(|_| {
                        "--max-active-samples-delta must be a non-negative integer".to_string()
                    })?;
                }
                "--min-active-samples-delta" => {
                    let Some(value) = args.next() else {
                        return Err("--min-active-samples-delta requires a value".into());
                    };
                    limits.min_active_samples_delta = value.parse::<usize>().map_err(|_| {
                        "--min-active-samples-delta must be a non-negative integer".to_string()
                    })?;
                }
                "--max-peak-delta" => {
                    let Some(value) = args.next() else {
                        return Err("--max-peak-delta requires a value".into());
                    };
                    limits.max_peak_delta = parse_non_negative_float("--max-peak-delta", &value)?;
                }
                "--min-peak-delta" => {
                    let Some(value) = args.next() else {
                        return Err("--min-peak-delta requires a value".into());
                    };
                    limits.min_peak_delta = parse_non_negative_float("--min-peak-delta", &value)?;
                }
                "--max-rms-delta" => {
                    let Some(value) = args.next() else {
                        return Err("--max-rms-delta requires a value".into());
                    };
                    limits.max_rms_delta = parse_non_negative_float("--max-rms-delta", &value)?;
                }
                "--min-rms-delta" => {
                    let Some(value) = args.next() else {
                        return Err("--min-rms-delta requires a value".into());
                    };
                    limits.min_rms_delta = parse_non_negative_float("--min-rms-delta", &value)?;
                }
                "--max-sum-delta" => {
                    let Some(value) = args.next() else {
                        return Err("--max-sum-delta requires a value".into());
                    };
                    limits.max_sum_delta = parse_non_negative_float("--max-sum-delta", &value)?;
                }
                "--min-sum-delta" => {
                    let Some(value) = args.next() else {
                        return Err("--min-sum-delta requires a value".into());
                    };
                    limits.min_sum_delta = parse_non_negative_float("--min-sum-delta", &value)?;
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        let baseline_metrics_path =
            baseline_override.unwrap_or_else(|| convention_metrics_path(&date, "baseline"));
        let candidate_metrics_path =
            candidate_override.unwrap_or_else(|| convention_metrics_path(&date, "candidate"));
        let report_path = report_override.unwrap_or_else(|| convention_report_path(&date));

        Ok(Self {
            baseline_metrics_path,
            candidate_metrics_path,
            report_path,
            limits,
            show_help,
        })
    }
}

fn parse_non_negative_float(flag: &str, value: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| format!("{flag} must be a non-negative number"))?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(format!("{flag} must be a non-negative number"));
    }
    Ok(parsed)
}

fn print_help() {
    println!(
        "Usage: w30_preview_compare [--date YYYY-MM-DD|local] [--baseline PATH] [--candidate PATH] [--report PATH]\n\
         \n\
         Optional drift limits:\n\
           --min-active-samples-delta N\n\
           --max-active-samples-delta N\n\
           --min-peak-delta FLOAT\n\
           --max-peak-delta FLOAT\n\
           --min-rms-delta FLOAT\n\
           --max-rms-delta FLOAT\n\
           --min-sum-delta FLOAT\n\
           --max-sum-delta FLOAT\n\
         \n\
         Compares W-30 preview smoke baseline and candidate metrics Markdown files\n\
         from the local audio QA artifact convention, writes comparison.md and\n\
         manifest.json, and verifies that referenced local artifacts exist. This\n\
         is still a narrow metrics helper, not a waveform diff."
    );
}

fn convention_metrics_path(date: &str, role: &str) -> PathBuf {
    let mut path = PathBuf::from("artifacts");
    path.push("audio_qa");
    path.push(date);
    path.push(PACK_ID);
    path.push(CASE_ID);
    path.push(format!("{role}.metrics.md"));
    path
}

fn convention_report_path(date: &str) -> PathBuf {
    let mut path = PathBuf::from("artifacts");
    path.push("audio_qa");
    path.push(date);
    path.push(PACK_ID);
    path.push(CASE_ID);
    path.push("comparison.md");
    path
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct SmokeMetrics {
    active_samples: usize,
    peak_abs: f64,
    rms: f64,
    sum: f64,
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

fn compare_metrics(
    baseline: &SmokeMetrics,
    candidate: &SmokeMetrics,
    limits: &DriftLimits,
) -> ComparisonReport {
    let active_delta = baseline.active_samples.abs_diff(candidate.active_samples);
    let peak_delta = (baseline.peak_abs - candidate.peak_abs).abs();
    let rms_delta = (baseline.rms - candidate.rms).abs();
    let sum_delta = (baseline.sum - candidate.sum).abs();

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
        peak_abs: MetricComparison {
            baseline: baseline.peak_abs,
            candidate: candidate.peak_abs,
            delta: peak_delta,
            min_delta: limits.min_peak_delta,
            max_delta: limits.max_peak_delta,
            passed: float_delta_within_range(
                peak_delta,
                limits.min_peak_delta,
                limits.max_peak_delta,
            ),
        },
        rms: MetricComparison {
            baseline: baseline.rms,
            candidate: candidate.rms,
            delta: rms_delta,
            min_delta: limits.min_rms_delta,
            max_delta: limits.max_rms_delta,
            passed: float_delta_within_range(rms_delta, limits.min_rms_delta, limits.max_rms_delta),
        },
        sum: MetricComparison {
            baseline: baseline.sum,
            candidate: candidate.sum,
            delta: sum_delta,
            min_delta: limits.min_sum_delta,
            max_delta: limits.max_sum_delta,
            passed: float_delta_within_range(sum_delta, limits.min_sum_delta, limits.max_sum_delta),
        },
    }
}

fn float_delta_within_range(delta: f64, min_delta: f64, max_delta: f64) -> bool {
    let epsilon = f64::EPSILON * 16.0;
    (delta >= min_delta || (min_delta - delta).abs() <= epsilon)
        && (delta <= max_delta || (delta - max_delta).abs() <= epsilon)
}

