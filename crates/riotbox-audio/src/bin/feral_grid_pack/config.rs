pub(super) const PACK_ID: &str = "feral-grid-demo";

pub(super) const SAMPLE_RATE: u32 = 44_100;

pub(super) const CHANNEL_COUNT: u16 = 2;

pub(super) const DEFAULT_DATE: &str = "local";

pub(super) const DEFAULT_BPM: f32 = 128.0;

pub(super) const DEFAULT_BARS: u32 = 8;

pub(super) const DEFAULT_BEATS_PER_BAR: u32 = 4;

pub(super) const MIN_BARS: u32 = 2;

pub(super) const DEFAULT_SOURCE_START_SECONDS: f32 = 0.0;

pub(super) const DEFAULT_SOURCE_WINDOW_SECONDS: f32 = 1.0;

pub(super) const MIN_SIGNAL_RMS: f32 = 0.001;

pub(super) const MIN_LOW_BAND_RMS: f32 = 0.004;

pub(super) const MIN_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO: f32 = 0.145;

pub(super) const MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO: f32 = 0.08;

pub(super) const MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO: f32 = 0.46;

pub(super) const SOURCE_TIMING_BPM_MATCH_TOLERANCE: f32 = 1.0;

pub(super) const PATTERN_ORIGIN_SOURCE_DERIVED: &str = "source_derived";

pub(super) const PATTERN_ORIGIN_PRIMITIVE_RENDERER: &str = "primitive_renderer";
