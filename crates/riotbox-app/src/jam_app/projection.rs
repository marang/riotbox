// Compatibility facade over existing Core/Session-to-audio projections.
mod mc202;
mod scene_context;
mod source_phrase_render;
mod tr909;
mod w30_material;
mod w30_preview;
mod w30_resample;

pub(super) use mc202::build_mc202_render_state;
pub(super) use tr909::build_tr909_render_state;
pub(super) use w30_preview::{build_w30_preview_render_state, normalize_w30_preview_mode};
pub(super) use w30_resample::{build_w30_resample_tap_state, resample_source_from_interleaved};
