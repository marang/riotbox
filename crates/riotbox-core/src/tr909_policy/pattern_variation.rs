use super::model::{
    Tr909PatternAdoptionPolicy, Tr909PhraseVariationPolicy, Tr909RenderModePolicy,
    Tr909SourceSupportProfilePolicy, Tr909TakeoverRenderProfilePolicy,
};
use crate::transport::TransportClockState;

pub(super) fn derive_tr909_pattern_adoption(
    mode: Tr909RenderModePolicy,
    pattern_ref: Option<&str>,
    source_support_profile: Option<Tr909SourceSupportProfilePolicy>,
    takeover_profile: Option<Tr909TakeoverRenderProfilePolicy>,
) -> Option<Tr909PatternAdoptionPolicy> {
    if matches!(mode, Tr909RenderModePolicy::Idle) {
        return None;
    }

    if matches!(mode, Tr909RenderModePolicy::Takeover)
        || matches!(
            takeover_profile,
            Some(Tr909TakeoverRenderProfilePolicy::ControlledPhrase)
        )
    {
        return Some(Tr909PatternAdoptionPolicy::TakeoverGrid);
    }

    let pattern_ref = pattern_ref.map(str::to_ascii_lowercase);
    if pattern_ref
        .as_deref()
        .is_some_and(|pattern| pattern.contains("takeover"))
    {
        return Some(Tr909PatternAdoptionPolicy::TakeoverGrid);
    }

    if pattern_ref
        .as_deref()
        .is_some_and(|pattern| pattern.contains("main") || pattern.contains("drop"))
        || matches!(
            source_support_profile,
            Some(Tr909SourceSupportProfilePolicy::DropDrive)
        )
        || matches!(
            mode,
            Tr909RenderModePolicy::Fill | Tr909RenderModePolicy::BreakReinforce
        )
    {
        return Some(Tr909PatternAdoptionPolicy::MainlineDrive);
    }

    Some(Tr909PatternAdoptionPolicy::SupportPulse)
}

pub(super) fn derive_tr909_phrase_variation(
    mode: Tr909RenderModePolicy,
    transport: &TransportClockState,
    pattern_ref: Option<&str>,
    source_support_profile: Option<Tr909SourceSupportProfilePolicy>,
    takeover_profile: Option<Tr909TakeoverRenderProfilePolicy>,
) -> Option<Tr909PhraseVariationPolicy> {
    if matches!(mode, Tr909RenderModePolicy::Idle) {
        return None;
    }

    let pattern_ref = pattern_ref.map(str::to_ascii_lowercase);
    if pattern_ref
        .as_deref()
        .is_some_and(|pattern| pattern.contains("release"))
    {
        return Some(Tr909PhraseVariationPolicy::PhraseRelease);
    }

    let phrase_cycle = transport.phrase_index % 4;
    let variation = match mode {
        Tr909RenderModePolicy::Takeover => match takeover_profile {
            Some(Tr909TakeoverRenderProfilePolicy::ControlledPhrase) | None => match phrase_cycle {
                0 => Tr909PhraseVariationPolicy::PhraseAnchor,
                1 => Tr909PhraseVariationPolicy::PhraseLift,
                2 => Tr909PhraseVariationPolicy::PhraseDrive,
                _ => Tr909PhraseVariationPolicy::PhraseRelease,
            },
            Some(Tr909TakeoverRenderProfilePolicy::SceneLock) => match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseDrive,
                _ => Tr909PhraseVariationPolicy::PhraseAnchor,
            },
        },
        Tr909RenderModePolicy::Fill | Tr909RenderModePolicy::BreakReinforce => {
            match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseDrive,
                _ => Tr909PhraseVariationPolicy::PhraseLift,
            }
        }
        Tr909RenderModePolicy::SourceSupport => match source_support_profile {
            Some(Tr909SourceSupportProfilePolicy::SteadyPulse) | None => match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseAnchor,
                _ => Tr909PhraseVariationPolicy::PhraseLift,
            },
            Some(Tr909SourceSupportProfilePolicy::BreakLift) => match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseLift,
                _ => Tr909PhraseVariationPolicy::PhraseDrive,
            },
            Some(Tr909SourceSupportProfilePolicy::DropDrive) => match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseDrive,
                _ => Tr909PhraseVariationPolicy::PhraseLift,
            },
        },
        Tr909RenderModePolicy::Idle => Tr909PhraseVariationPolicy::PhraseAnchor,
    };

    Some(variation)
}
