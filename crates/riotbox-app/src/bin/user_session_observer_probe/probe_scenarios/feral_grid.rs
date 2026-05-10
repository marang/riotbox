use std::{io, path::Path};

use crossterm::event::KeyCode;
use riotbox_app::ui::JamShellState;
use riotbox_core::{
    action::CommitBoundary,
    ids::SourceId,
    source_graph::{
        DecodeProfile, GraphProvenance, SourceDescriptor, SourceGraph, TimingDegradedPolicy,
        TimingQuality, TimingWarning, TimingWarningCode,
    },
};

use super::{
    NdjsonWriter, apply_probe_key, commit_boundary, locked_timing_grid::attach_locked_timing_grid,
    probe_shell, record_probe_start,
};

pub(crate) fn write_feral_grid_jam_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("feral-grid-jam-probe");
    attach_probe_source_timing(
        &mut shell,
        "src-feral-grid-probe",
        "synthetic-feral-grid-source.wav",
        128.0,
    );

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "feral-grid-jam",
        "synthetic-feral-grid-source.wav",
        "headless-feral-grid-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('f'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Bar, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 500, CommitBoundary::Phrase, 2, 1)?;
    Ok(())
}

pub(crate) fn write_feral_grid_fallback_jam_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("feral-grid-jam-fallback-probe");
    attach_fallback_probe_source_timing(
        &mut shell,
        "src-feral-grid-probe",
        "synthetic-feral-grid-fallback-source.wav",
    );

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "feral-grid-jam-fallback",
        "synthetic-feral-grid-fallback-source.wav",
        "headless-feral-grid-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('f'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Bar, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 500, CommitBoundary::Phrase, 2, 1)?;
    Ok(())
}

pub(crate) fn write_feral_grid_locked_jam_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("feral-grid-jam-locked-probe");
    attach_locked_probe_source_timing(
        &mut shell,
        "src-feral-grid-probe",
        "synthetic-feral-grid-source.wav",
        128.0,
    );

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "feral-grid-jam-locked",
        "synthetic-feral-grid-source.wav",
        "headless-feral-grid-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('f'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Bar, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 500, CommitBoundary::Phrase, 2, 1)?;
    Ok(())
}

fn attach_probe_source_timing(shell: &mut JamShellState, source_id: &str, path: &str, bpm: f32) {
    attach_source_timing(
        shell,
        source_id,
        path,
        Some(bpm),
        ProbeTimingProfile::Cautious,
    );
}

fn attach_fallback_probe_source_timing(shell: &mut JamShellState, source_id: &str, path: &str) {
    attach_source_timing(shell, source_id, path, None, ProbeTimingProfile::Fallback);
}

fn attach_locked_probe_source_timing(
    shell: &mut JamShellState,
    source_id: &str,
    path: &str,
    bpm: f32,
) {
    attach_source_timing(
        shell,
        source_id,
        path,
        Some(bpm),
        ProbeTimingProfile::Locked,
    );
}

enum ProbeTimingProfile {
    Cautious,
    Fallback,
    Locked,
}

fn attach_source_timing(
    shell: &mut JamShellState,
    source_id: &str,
    path: &str,
    bpm: Option<f32>,
    profile: ProbeTimingProfile,
) {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from(source_id),
            path: path.into(),
            content_hash: "headless-probe-source-hash".into(),
            duration_seconds: 8.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "headless-probe".into(),
            provider_set: vec!["user_session_observer_probe".into()],
            generated_at: "2026-05-08T00:00:00Z".into(),
            source_hash: "headless-probe-source-hash".into(),
            analysis_seed: 19,
            run_notes: Some("synthetic source timing readiness for observer QA".into()),
        },
    );
    graph.timing.bpm_estimate = bpm;
    match profile {
        ProbeTimingProfile::Cautious => {
            graph.timing.bpm_confidence = 0.86;
            graph.timing.quality = TimingQuality::Medium;
            graph.timing.degraded_policy = TimingDegradedPolicy::Cautious;
            graph.timing.warnings.push(TimingWarning {
                code: TimingWarningCode::PhraseUncertain,
                message: "headless probe uses synthetic source timing readiness".into(),
            });
        }
        ProbeTimingProfile::Fallback => {
            graph.timing.bpm_confidence = 0.0;
            graph.timing.quality = TimingQuality::Low;
            graph.timing.degraded_policy = TimingDegradedPolicy::FallbackGrid;
            graph.timing.warnings.push(TimingWarning {
                code: TimingWarningCode::LowTimingConfidence,
                message: "headless probe has no trusted timing estimate".into(),
            });
            graph.timing.warnings.push(TimingWarning {
                code: TimingWarningCode::WeakKickAnchor,
                message: "headless probe has no trusted kick anchor".into(),
            });
        }
        ProbeTimingProfile::Locked => {
            let bpm = bpm.expect("locked probe timing requires a BPM");
            graph.timing.bpm_confidence = 0.92;
            graph.timing.quality = TimingQuality::High;
            graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
            attach_locked_timing_grid(&mut graph, bpm);
        }
    }

    shell.app.source_graph = Some(graph);
    shell.app.refresh_view();
}
