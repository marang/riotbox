use riotbox_audio::{
    mc202::{
        Mc202ContourHint, Mc202HookResponse, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode,
        Mc202RenderRouting, Mc202RenderState,
    },
    tr909::{
        Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
        Tr909RenderState,
    },
    w30::{
        W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState,
        W30PreviewSampleWindow, W30PreviewSourceProfile,
    },
};

pub(super) fn w30_source_chop_state(
    source_window_preview: W30PreviewSampleWindow,
) -> W30PreviewRenderState {
    W30PreviewRenderState {
        mode: W30PreviewRenderMode::RawCaptureAudition,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
        active_bank_id: Some("bank-a".into()),
        focused_pad_id: Some("pad-01".into()),
        capture_id: Some("cap-feral-preview".into()),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: Some(source_window_preview),
        pad_playback: None,
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
    }
}

pub(super) fn tr909_fill_state() -> Tr909RenderState {
    Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
        drum_bus_level: 0.82,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Tr909RenderState::default()
    }
}

pub(super) fn mc202_instigator_state() -> Mc202RenderState {
    Mc202RenderState {
        mode: Mc202RenderMode::Instigator,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::InstigatorSpike,
        note_budget: Mc202NoteBudget::Push,
        contour_hint: Mc202ContourHint::Lift,
        hook_response: Mc202HookResponse::Direct,
        source_phrase_plan: None,
        touch: 0.9,
        music_bus_level: 0.74,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
    }
}
