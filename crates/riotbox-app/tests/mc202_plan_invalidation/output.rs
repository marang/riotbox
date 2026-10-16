//! Exact App projection into existing renderer, generated PCM only, no host.
use super::{JamAppState, assert_no_hydration, invalidated_history};
use riotbox_audio::mc202::{Mc202RenderRouting, render_mc202_buffer};
use riotbox_core::{queue::ActionQueue, session::ReplayPolicy};

#[test]
fn restored_invalidated_plan_keeps_actual_mc202_projection_silent_with_nonzero_control() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-output-source.wav");
    let history = invalidated_history(&source, ReplayPolicy::DeterministicCommittedOrderV2);
    let mut input = history.session;
    input.runtime_state.transport.is_playing = true;
    input.runtime_state.mixer_state.music_level = 0.5;
    let mut app = JamAppState::from_parts(input, history.source_graph, ActionQueue::new());
    app.apply_restore_target_suffix(2).unwrap();
    assert_eq!(
        app.runtime.mc202_render.routing,
        Mc202RenderRouting::MusicBusBass
    );
    let mut active = vec![0.0; 48_000 * 2];
    render_mc202_buffer(&mut active, 48_000, 2, &app.runtime.mc202_render);
    assert!(active.iter().all(|sample| sample.is_finite()));
    assert!(
        active.iter().any(|sample| sample.abs() > 0.0001),
        "positive renderer control must not be silent: {:?}",
        app.runtime.mc202_render
    );
    app.apply_restore_target_suffix(4).unwrap();
    assert_eq!(app.runtime.mc202_render.routing, Mc202RenderRouting::Silent);
    assert!(app.runtime.mc202_render.source_phrase_plan.is_none());
    let mut invalidated = vec![0.0; active.len()];
    render_mc202_buffer(&mut invalidated, 48_000, 2, &app.runtime.mc202_render);
    assert!(invalidated.iter().all(|sample| *sample == 0.0));
    assert_no_hydration(&app);
    assert!(!source.exists());
}
