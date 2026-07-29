use riotbox_audio::w30::{
    W30_RESAMPLE_HARD_SLICE_COUNT, W30ResampleHardSuitability, W30ResampleTapHardPolicy,
};
use riotbox_core::w30::{W30HardIntent, W30HardIntentOutcome};

pub(super) fn resolve_w30_hard_intent(
    hard_intent: Option<W30HardIntent>,
    suitability: W30ResampleHardSuitability,
    analyzed_policy: W30ResampleTapHardPolicy,
    analyzed_trigger_mask: u8,
    analyzed_slice_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
) -> (
    W30ResampleTapHardPolicy,
    u8,
    [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    W30HardIntentOutcome,
) {
    if suitability != W30ResampleHardSuitability::Suitable {
        return (
            W30ResampleTapHardPolicy::Unavailable,
            0,
            [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            if hard_intent.is_some() {
                W30HardIntentOutcome::SourceUnavailable
            } else {
                W30HardIntentOutcome::Inactive
            },
        );
    }

    match hard_intent {
        None => (
            analyzed_policy,
            analyzed_trigger_mask,
            analyzed_slice_cursors,
            W30HardIntentOutcome::Inactive,
        ),
        Some(W30HardIntent::LegacyAuto) => (
            analyzed_policy,
            analyzed_trigger_mask,
            analyzed_slice_cursors,
            W30HardIntentOutcome::LegacyAuto,
        ),
        Some(W30HardIntent::Impact)
            if analyzed_policy == W30ResampleTapHardPolicy::SourceTransientChop =>
        {
            (
                analyzed_policy,
                analyzed_trigger_mask,
                analyzed_slice_cursors,
                W30HardIntentOutcome::RealizedImpact,
            )
        }
        Some(W30HardIntent::Impact) => (
            W30ResampleTapHardPolicy::Unavailable,
            0,
            [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            W30HardIntentOutcome::SourceMismatch,
        ),
        Some(W30HardIntent::Texture) => (
            W30ResampleTapHardPolicy::SourceTextureBite,
            0,
            [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            W30HardIntentOutcome::RealizedTexture,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURSORS: [u16; W30_RESAMPLE_HARD_SLICE_COUNT] = [4, 8, 12, 16, 20, 24, 28, 32];

    #[test]
    fn impact_realizes_only_source_transient_policy() {
        assert_eq!(
            resolve_w30_hard_intent(
                Some(W30HardIntent::Impact),
                W30ResampleHardSuitability::Suitable,
                W30ResampleTapHardPolicy::SourceTransientChop,
                0b1011,
                CURSORS,
            ),
            (
                W30ResampleTapHardPolicy::SourceTransientChop,
                0b1011,
                CURSORS,
                W30HardIntentOutcome::RealizedImpact,
            )
        );
        assert_eq!(
            resolve_w30_hard_intent(
                Some(W30HardIntent::Impact),
                W30ResampleHardSuitability::Suitable,
                W30ResampleTapHardPolicy::SourceTextureBite,
                0,
                CURSORS,
            ),
            (
                W30ResampleTapHardPolicy::Unavailable,
                0,
                [0; W30_RESAMPLE_HARD_SLICE_COUNT],
                W30HardIntentOutcome::SourceMismatch,
            )
        );
    }

    #[test]
    fn texture_realization_never_exposes_an_invented_trigger_grid() {
        assert_eq!(
            resolve_w30_hard_intent(
                Some(W30HardIntent::Texture),
                W30ResampleHardSuitability::Suitable,
                W30ResampleTapHardPolicy::SourceTransientChop,
                0b1111,
                CURSORS,
            ),
            (
                W30ResampleTapHardPolicy::SourceTextureBite,
                0,
                [0; W30_RESAMPLE_HARD_SLICE_COUNT],
                W30HardIntentOutcome::RealizedTexture,
            )
        );
    }

    #[test]
    fn legacy_auto_preserves_analysis_and_unavailable_source_is_explicit() {
        assert_eq!(
            resolve_w30_hard_intent(
                Some(W30HardIntent::LegacyAuto),
                W30ResampleHardSuitability::Suitable,
                W30ResampleTapHardPolicy::SourceTextureBite,
                0,
                CURSORS,
            ),
            (
                W30ResampleTapHardPolicy::SourceTextureBite,
                0,
                CURSORS,
                W30HardIntentOutcome::LegacyAuto,
            )
        );
        assert_eq!(
            resolve_w30_hard_intent(
                Some(W30HardIntent::Impact),
                W30ResampleHardSuitability::InsufficientLevel,
                W30ResampleTapHardPolicy::Unavailable,
                0,
                CURSORS,
            ),
            (
                W30ResampleTapHardPolicy::Unavailable,
                0,
                [0; W30_RESAMPLE_HARD_SLICE_COUNT],
                W30HardIntentOutcome::SourceUnavailable,
            )
        );
    }
}
