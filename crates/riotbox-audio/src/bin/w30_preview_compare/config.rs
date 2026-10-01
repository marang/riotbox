use serde::Serialize;

pub(super) const DEFAULT_DATE: &str = "local";

pub(super) const PACK_ID: &str = "w30-preview-smoke";

pub(super) const CASE_ID: &str = "raw_capture_source_window_preview";

const DEFAULT_MAX_ACTIVE_SAMPLES_DELTA: usize = 0;

const DEFAULT_MAX_PEAK_DELTA: f64 = 0.000001;

const DEFAULT_MAX_RMS_DELTA: f64 = 0.000001;

const DEFAULT_MAX_SUM_DELTA: f64 = 0.000001;

const DEFAULT_MIN_ACTIVE_SAMPLES_DELTA: usize = 0;

const DEFAULT_MIN_PEAK_DELTA: f64 = 0.0;

const DEFAULT_MIN_RMS_DELTA: f64 = 0.0;

const DEFAULT_MIN_SUM_DELTA: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(super) struct DriftLimits {
    pub(super) min_active_samples_delta: usize,
    pub(super) max_active_samples_delta: usize,
    pub(super) min_peak_delta: f64,
    pub(super) max_peak_delta: f64,
    pub(super) min_rms_delta: f64,
    pub(super) max_rms_delta: f64,
    pub(super) min_sum_delta: f64,
    pub(super) max_sum_delta: f64,
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
