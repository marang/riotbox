use super::config::{CASE_ID, DEFAULT_DATE, DriftLimits, PACK_ID};
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub(super) struct Args {
    pub(super) baseline_metrics_path: PathBuf,
    pub(super) candidate_metrics_path: PathBuf,
    pub(super) report_path: PathBuf,
    pub(super) limits: DriftLimits,
    pub(super) show_help: bool,
}

impl Args {
    pub(super) fn parse<I>(args: I) -> Result<Self, String>
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

pub(super) fn print_help() {
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
