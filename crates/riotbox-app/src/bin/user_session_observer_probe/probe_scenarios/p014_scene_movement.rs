use std::{io, path::Path};

use crossterm::event::KeyCode;
use riotbox_app::ui::JamShellState;
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
    NdjsonWriter, apply_probe_key, commit_boundary_for_scene, probe_shell, record_probe_start,
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
    commit_boundary_for_scene(
        &mut shell,
        &mut writer,
        300,
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-01-break")),
        },
        1,
    )?;

    assert_p014_scene_movement_probe_state(&shell)
}

fn attach_p014_scene_source(shell: &mut JamShellState) {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-p014-scene-movement"),
            path: "synthetic-p014-scene-movement.wav".into(),
            content_hash: "headless-p014-scene-movement-hash".into(),
            duration_seconds: 32.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "headless-probe".into(),
            provider_set: vec!["user_session_observer_probe".into()],
            generated_at: "2026-05-30T00:00:00Z".into(),
            source_hash: "headless-p014-scene-movement-hash".into(),
            analysis_seed: 30,
            run_notes: Some("P014 scene movement observer probe".into()),
        },
    );
    graph.timing.bpm_estimate = Some(120.0);
    graph.timing.bpm_confidence = 0.9;
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
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
