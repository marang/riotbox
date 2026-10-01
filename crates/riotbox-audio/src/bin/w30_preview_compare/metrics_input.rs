use serde::Serialize;
use std::{fs, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(super) struct SmokeMetrics {
    pub(super) active_samples: usize,
    pub(super) peak_abs: f64,
    pub(super) rms: f64,
    pub(super) sum: f64,
    pub(super) mean_abs: f64,
    pub(super) zero_crossings: usize,
    pub(super) crest_factor: f64,
    pub(super) active_sample_ratio: f64,
    pub(super) silence_ratio: f64,
    pub(super) dc_offset: f64,
    pub(super) onset_count: usize,
    pub(super) event_density_per_bar: f64,
}

impl SmokeMetrics {
    pub(super) fn read_from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        Self::parse_markdown(&contents).map_err(|error| {
            format!("failed to parse metrics from {}: {error}", path.display()).into()
        })
    }

    pub(super) fn parse_markdown(contents: &str) -> Result<Self, String> {
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
