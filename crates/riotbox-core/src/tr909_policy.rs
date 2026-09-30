// Compatibility facade for the existing Core-owned TR-909 policy.
mod model;
mod pattern_variation;
mod render_policy;
mod source_support;

pub use model::{
    Tr909PatternAdoptionPolicy, Tr909PhraseVariationPolicy, Tr909RenderModePolicy,
    Tr909RenderPolicyProjection, Tr909RenderRoutingPolicy, Tr909SourceSupportContextPolicy,
    Tr909SourceSupportProfilePolicy, Tr909SourceSupportReasonPolicy,
    Tr909TakeoverRenderProfilePolicy,
};
pub use render_policy::{
    derive_tr909_render_policy, derive_tr909_render_policy_with_scene_context,
};
pub use source_support::derive_tr909_source_support_reason;

#[cfg(test)]
mod tests;
