use riotbox_audio::tr909::{Tr909FillRecipeId, Tr909PhraseVariation, Tr909RenderMode};
use riotbox_core::{
    session::SessionFile, style::PerformancePresetId, transport::TransportClockState,
};

use super::projection::build_tr909_render_state;

#[test]
fn feral_break_alpha_fill_keeps_its_destructive_phrase_drive_recipe() {
    let fill_transport = || TransportClockState {
        is_playing: true,
        position_beats: 34.0,
        beat_index: 34,
        bar_index: 9,
        phrase_index: 3,
        current_scene: None,
    };
    let fill_session = |preset: Option<PerformancePresetId>| {
        let mut session =
            SessionFile::new("feral-fill-projection", "0.1.0", "2026-07-18T00:00:00Z");
        if let Some(preset) = preset {
            preset.apply_to_session(&mut session);
        }
        session.runtime_state.lane_state.tr909.last_fill_bar = Some(9);
        session
    };

    let baseline = build_tr909_render_state(&fill_session(None), &fill_transport(), None);
    assert_eq!(baseline.mode, Tr909RenderMode::Fill);
    assert_eq!(
        baseline.phrase_variation,
        Some(Tr909PhraseVariation::PhraseLift),
        "the odd phrase-cycle control should remain the non-preset baseline"
    );
    assert_eq!(
        baseline.fill_recipe_id(),
        Some(Tr909FillRecipeId::GenericFillV1)
    );

    let v1_transport = TransportClockState {
        phrase_index: 2,
        ..fill_transport()
    };
    let v1 = build_tr909_render_state(
        &fill_session(Some(PerformancePresetId::FeralBreakAlphaV1)),
        &v1_transport,
        None,
    );
    assert_eq!(v1.phrase_variation, Some(Tr909PhraseVariation::PhraseDrive));
    assert_eq!(
        v1.fill_recipe_id(),
        Some(Tr909FillRecipeId::PhraseDriveBreakCutStompV1),
        "the historical v1 preset must not inherit the v2 hard-cut recipe"
    );

    let preset = build_tr909_render_state(
        &fill_session(Some(PerformancePresetId::FeralBreakAlphaV2)),
        &fill_transport(),
        None,
    );
    assert_eq!(preset.mode, Tr909RenderMode::Fill);
    assert_eq!(
        preset.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDriveHardCut)
    );
    assert_eq!(
        preset.fill_recipe_id(),
        Some(Tr909FillRecipeId::PhraseDriveBreakCutStompV2)
    );
}
