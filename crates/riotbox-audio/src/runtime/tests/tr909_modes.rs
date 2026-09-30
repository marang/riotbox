use crate::runtime::render_tr909_w30_preview::render_tr909_buffer;
use crate::runtime::shared_transport_tr909::RealtimeTr909RenderState;
use crate::runtime::shared_w30_resample_callback::Tr909CallbackState;
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909SourceSupportContext, Tr909SourceSupportProfile, Tr909TakeoverRenderProfile,
};

#[test]
fn render_buffer_stays_silent_when_idle() {
    let mut state = Tr909CallbackState::default();
    let mut buffer = [0.0_f32; 128];

    render_tr909_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::Idle,
            routing: Tr909RenderRouting::SourceOnly,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: None,
            phrase_variation: None,
            takeover_profile: None,
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.2,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn render_buffer_produces_audible_samples_for_support_mode() {
    let mut state = Tr909CallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_tr909_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: None,
            phrase_variation: None,
            takeover_profile: None,
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.6,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut state,
    );

    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn render_buffer_respects_zero_drum_bus_level() {
    let mut state = Tr909CallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_tr909_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: None,
            phrase_variation: None,
            takeover_profile: None,
            drum_bus_level: 0.0,
            slam_enabled: false,
            slam_intensity: 0.6,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn source_support_profiles_produce_different_peak_levels() {
    let mut steady_state = Tr909CallbackState::default();
    let mut drive_state = Tr909CallbackState::default();
    let mut steady = [0.0_f32; 512];
    let mut drive = [0.0_f32; 512];

    render_tr909_buffer(
        &mut steady,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: Some(Tr909SourceSupportProfile::SteadyPulse),
            source_support_context: Some(Tr909SourceSupportContext::TransportBar),
            pattern_adoption: Some(Tr909PatternAdoption::SupportPulse),
            phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
            takeover_profile: None,
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut steady_state,
    );

    render_tr909_buffer(
        &mut drive,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: Some(Tr909SourceSupportProfile::DropDrive),
            source_support_context: Some(Tr909SourceSupportContext::SceneTarget),
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            takeover_profile: None,
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut drive_state,
    );

    let steady_peak = steady
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let drive_peak = drive
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

    assert!(drive_peak > steady_peak);
}

#[test]
fn scene_target_context_adds_bounded_support_accent() {
    let mut transport_state = Tr909CallbackState::default();
    let mut scene_state = Tr909CallbackState::default();
    let mut transport = [0.0_f32; 512];
    let mut scene_target = [0.0_f32; 512];
    let base = RealtimeTr909RenderState {
        mode: Tr909RenderMode::SourceSupport,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: Some(Tr909SourceSupportProfile::BreakLift),
        source_support_context: Some(Tr909SourceSupportContext::TransportBar),
        pattern_adoption: Some(Tr909PatternAdoption::SupportPulse),
        phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
        takeover_profile: None,
        drum_bus_level: 0.8,
        slam_enabled: false,
        slam_intensity: 0.35,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
        source_bar_grid_anchor_position_beats: None,
    };

    render_tr909_buffer(&mut transport, 44_100, 2, &base, &mut transport_state);

    let mut scene_render = base;
    scene_render.source_support_context = Some(Tr909SourceSupportContext::SceneTarget);
    render_tr909_buffer(
        &mut scene_target,
        44_100,
        2,
        &scene_render,
        &mut scene_state,
    );

    let transport_peak = transport
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let scene_peak = scene_target
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let transport_active = transport
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();
    let scene_active = scene_target
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();

    assert!(scene_peak > transport_peak);
    assert!(scene_peak < transport_peak * 1.3);
    assert!(
        scene_active.abs_diff(transport_active) <= 4,
        "scene_active={scene_active} transport_active={transport_active}"
    );
}

#[test]
fn controlled_phrase_takeover_profile_is_more_active_than_scene_lock() {
    let mut controlled_state = Tr909CallbackState::default();
    let mut lock_state = Tr909CallbackState::default();
    let mut controlled = [0.0_f32; 512];
    let mut scene_lock = [0.0_f32; 512];

    render_tr909_buffer(
        &mut controlled,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
            phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
            takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.45,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut controlled_state,
    );

    render_tr909_buffer(
        &mut scene_lock,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::SupportPulse),
            phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
            takeover_profile: Some(Tr909TakeoverRenderProfile::SceneLock),
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.45,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut lock_state,
    );

    let controlled_active = controlled
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();
    let scene_lock_active = scene_lock
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();

    assert!(controlled_active > scene_lock_active);
}

#[test]
fn pattern_adoption_variants_produce_distinct_activity() {
    let mut pulse_state = Tr909CallbackState::default();
    let mut drive_state = Tr909CallbackState::default();
    let mut grid_state = Tr909CallbackState::default();
    let mut pulse = [0.0_f32; 512];
    let mut drive = [0.0_f32; 512];
    let mut grid = [0.0_f32; 512];

    render_tr909_buffer(
        &mut pulse,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: Some(Tr909SourceSupportProfile::SteadyPulse),
            source_support_context: Some(Tr909SourceSupportContext::TransportBar),
            pattern_adoption: Some(Tr909PatternAdoption::SupportPulse),
            phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
            takeover_profile: None,
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut pulse_state,
    );

    render_tr909_buffer(
        &mut drive,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: Some(Tr909SourceSupportProfile::DropDrive),
            source_support_context: Some(Tr909SourceSupportContext::SceneTarget),
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            takeover_profile: None,
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut drive_state,
    );

    render_tr909_buffer(
        &mut grid,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
            phrase_variation: Some(Tr909PhraseVariation::PhraseRelease),
            takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut grid_state,
    );

    let pulse_peak = pulse
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let drive_peak = drive
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let grid_peak = grid
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

    assert_ne!(pulse_peak, drive_peak);
    assert_ne!(drive_peak, grid_peak);
    assert!(grid_peak > pulse_peak);
}

#[test]
fn phrase_variations_produce_distinct_activity() {
    let mut anchor_state = Tr909CallbackState::default();
    let mut drive_state = Tr909CallbackState::default();
    let mut release_state = Tr909CallbackState::default();
    let mut anchor = [0.0_f32; 512];
    let mut drive = [0.0_f32; 512];
    let mut release = [0.0_f32; 512];

    let base = RealtimeTr909RenderState {
        mode: Tr909RenderMode::Takeover,
        routing: Tr909RenderRouting::DrumBusTakeover,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
        phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
        takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
        drum_bus_level: 0.8,
        slam_enabled: false,
        slam_intensity: 0.45,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
        source_bar_grid_anchor_position_beats: None,
    };

    render_tr909_buffer(&mut anchor, 44_100, 2, &base, &mut anchor_state);

    let mut drive_render = base;
    drive_render.phrase_variation = Some(Tr909PhraseVariation::PhraseDrive);
    render_tr909_buffer(&mut drive, 44_100, 2, &drive_render, &mut drive_state);

    let mut release_render = base;
    release_render.phrase_variation = Some(Tr909PhraseVariation::PhraseRelease);
    render_tr909_buffer(&mut release, 44_100, 2, &release_render, &mut release_state);

    let anchor_peak = anchor
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let drive_peak = drive
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let release_peak = release
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let anchor_active = anchor.iter().filter(|sample| sample.abs() > 0.0001).count();
    let release_active = release
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();

    assert!(drive_peak > anchor_peak);
    assert!(release_peak < drive_peak);
    assert!(release_active < anchor_active);
}
