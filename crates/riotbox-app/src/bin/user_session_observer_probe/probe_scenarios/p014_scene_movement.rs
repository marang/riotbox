use std::{io, path::Path};

use crossterm::event::KeyCode;
use riotbox_app::ui::JamShellState;
use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_core::{
    action::{ActionCommand, CommitBoundary, SourceMonitorMode},
    ids::{SceneId, SectionId, SourceId},
    source_graph::{
        DecodeProfile, EnergyClass, GraphProvenance, Section, SectionLabelHint, SourceDescriptor,
        SourceGraph, TimingDegradedPolicy,
    },
    transport::CommitBoundaryState,
};

use super::{
    NdjsonWriter, apply_probe_key, commit_boundary_for_scene,
    locked_timing_grid::attach_locked_timing_grid_for_bars, probe_shell, record_probe_start,
    source_aware_grid_position,
};

pub(crate) fn write_p014_scene_movement_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("p014-scene-movement-probe");
    attach_p014_scene_source(&mut shell);

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "p014-scene-movement",
        "synthetic-p014-scene-movement.wav",
        "headless-p014-scene-movement-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char('y'))?;
    let grid_position = source_aware_grid_position(&shell, 36);
    commit_boundary_for_scene(
        &mut shell,
        &mut writer,
        300,
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: grid_position.beat_cursor,
            bar_index: grid_position.bar_index,
            phrase_index: grid_position.phrase_index,
            scene_id: Some(SceneId::from("scene-01-break")),
        },
        1,
    )?;

    assert_p014_scene_movement_probe_state(&shell)
}

pub(super) fn attach_p014_scene_source(shell: &mut JamShellState) {
    attach_scene_probe_source(
        shell,
        SceneProbeSourceIdentity {
            source_id: "src-p014-scene-movement",
            source_path: "synthetic-p014-scene-movement.wav",
            source_hash: "headless-p014-scene-movement-hash",
            analysis_seed: 30,
            run_notes: "P014 scene movement observer probe",
        },
    );
}

pub(super) fn attach_first_playable_scene_source(shell: &mut JamShellState) {
    attach_scene_probe_source(
        shell,
        SceneProbeSourceIdentity {
            source_id: "src-first-playable-jam",
            source_path: "synthetic-first-playable-source.wav",
            source_hash: "headless-first-playable-source-hash",
            analysis_seed: 19,
            run_notes: "first-playable Jam observer probe",
        },
    );
}

#[derive(Copy, Clone)]
struct SceneProbeSourceIdentity<'a> {
    source_id: &'a str,
    source_path: &'a str,
    source_hash: &'a str,
    analysis_seed: u64,
    run_notes: &'a str,
}

fn attach_scene_probe_source(shell: &mut JamShellState, identity: SceneProbeSourceIdentity<'_>) {
    const SAMPLE_RATE: u32 = 44_100;
    const CHANNEL_COUNT: u16 = 2;
    const DURATION_SECONDS: usize = 32;
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from(identity.source_id),
            path: identity.source_path.into(),
            content_hash: identity.source_hash.into(),
            duration_seconds: 32.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "headless-probe".into(),
            provider_set: vec!["user_session_observer_probe".into()],
            generated_at: "2026-05-30T00:00:00Z".into(),
            source_hash: identity.source_hash.into(),
            analysis_seed: identity.analysis_seed,
            run_notes: Some(identity.run_notes.into()),
        },
    );
    graph.timing.bpm_estimate = Some(120.0);
    graph.timing.bpm_confidence = 0.9;
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
    attach_locked_timing_grid_for_bars(&mut graph, 120.0, 16);
    graph.sections = vec![
        Section {
            section_id: SectionId::from("section-break"),
            label_hint: SectionLabelHint::Break,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: EnergyClass::Medium,
            confidence: 0.9,
            tags: vec!["break".into()],
        },
        Section {
            section_id: SectionId::from("section-drop"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: EnergyClass::High,
            confidence: 0.9,
            tags: vec!["drop".into()],
        },
    ];

    shell.app.source_graph = Some(graph);
    shell.app.session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-break"));
    shell.app.runtime.transport.current_scene = Some(SceneId::from("scene-01-break"));
    shell.app.session.runtime_state.scene_state.active_scene =
        Some(SceneId::from("scene-01-break"));
    shell.app.session.runtime_state.scene_state.restore_scene = None;
    shell.app.session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-break"),
        SceneId::from("scene-02-drop"),
    ];
    shell.app.session.runtime_state.source_monitor.mode = SourceMonitorMode::Source;
    let sample_rate = usize::try_from(SAMPLE_RATE).expect("sample rate fits usize");
    let frame_count = sample_rate * DURATION_SECONDS;
    let samples = (0..frame_count)
        .flat_map(|frame| {
            let within_half_second = frame % (sample_rate / 2);
            let sample = if within_half_second < 180 { 0.25 } else { 0.0 };
            [sample, sample]
        })
        .collect();
    shell.app.source_audio_cache = Some(
        SourceAudioCache::from_interleaved_samples(
            identity.source_path,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            samples,
        )
        .expect("headless scene source audio"),
    );
    shell.app.refresh_view();
}

fn assert_p014_scene_movement_probe_state(shell: &JamShellState) -> io::Result<()> {
    let movement = shell
        .app
        .session
        .runtime_state
        .scene_state
        .last_movement
        .as_ref()
        .ok_or_else(|| io::Error::other("P014 scene movement probe did not land movement"))?;
    if movement.to_scene != SceneId::from("scene-02-drop") {
        return Err(io::Error::other(
            "P014 scene movement probe landed the wrong target scene",
        ));
    }
    if movement.committed_bar_index != 10 || movement.committed_phrase_index != 3 {
        return Err(io::Error::other(
            "P014 scene movement probe landed non-canonical transport-grid identity",
        ));
    }
    if shell.app.jam_view.scene.last_movement.is_none() {
        return Err(io::Error::other(
            "P014 scene movement probe did not project movement into Jam view",
        ));
    }
    if shell
        .app
        .source_monitor_control_state()
        .source_anchor_seconds
        != Some(16.0)
    {
        return Err(io::Error::other(
            "P014 scene movement probe did not expose source monitor anchor",
        ));
    }
    let action = shell
        .app
        .session
        .action_log
        .actions
        .last()
        .ok_or_else(|| io::Error::other("P014 scene movement probe did not commit"))?;
    if action.command != ActionCommand::SceneLaunch {
        return Err(io::Error::other(
            "P014 scene movement probe committed the wrong command",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_playable_fixture_keeps_source_graph_and_cache_identity_aligned() {
        let mut shell = probe_shell("first-playable-source-identity-test");
        attach_first_playable_scene_source(&mut shell);

        let graph = shell.app.source_graph.as_ref().expect("source graph");
        let cache = shell
            .app
            .source_audio_cache
            .as_ref()
            .expect("source audio cache");
        assert_eq!(
            graph.source.source_id,
            SourceId::from("src-first-playable-jam")
        );
        assert_eq!(graph.source.path, "synthetic-first-playable-source.wav");
        assert_eq!(cache.path.to_string_lossy(), graph.source.path);
        assert_eq!(graph.source.content_hash, graph.provenance.source_hash);
        assert_eq!(graph.source.duration_seconds, 32.0);
        assert_eq!(cache.duration_seconds(), 32.0);
        assert_eq!(graph.timing.beat_grid.len(), 64);
        assert_eq!(graph.timing.bar_grid.len(), 16);
        assert_eq!(graph.timing.phrase_grid.len(), 4);
        assert_eq!(graph.timing.beat_grid.first().unwrap().beat_index, 1);
        assert_eq!(graph.timing.bar_grid.first().unwrap().bar_index, 1);
        assert_eq!(graph.timing.bar_grid[8].bar_index, 9);
        assert_eq!(graph.timing.bar_grid[8].start_seconds, 16.0);
        assert_eq!(graph.timing.phrase_grid.first().unwrap().phrase_index, 1);
        assert_eq!(graph.timing.phrase_grid.first().unwrap().start_bar, 1);
    }
}
