// Textual includes keep this large file split mechanical and behavior-preserving.
use super::transport_helpers::trusted_source_timing_bpm;

mod w30_hard_intent;
mod w30_low_impact;

use w30_hard_intent::resolve_w30_hard_intent;
use w30_low_impact::derive_w30_resample_low_impact;

include!("projection/tr909_projection.rs");
include!("projection/w30_projection.rs");
