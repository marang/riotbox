use std::{fs, io, path::Path};

use crossterm::event::KeyCode;
use riotbox_app::{
    jam_app::JamAppState,
    observer::observer_snapshot,
    ui::{JamShellState, ShellLaunchMode},
};
use riotbox_core::{
    action::CommitBoundary,
    ids::SourceId,
    persistence::save_session_json,
    queue::ActionQueue,
    session::SessionFile,
    source_graph::{
        DecodeProfile, GraphProvenance, SourceDescriptor, SourceGraph, TimingDegradedPolicy,
        TimingQuality, TimingWarning, TimingWarningCode,
    },
};
use serde_json::{Value, json};

#[path = "locked_timing_grid.rs"]
mod locked_timing_grid;

use super::{
    NdjsonWriter, apply_probe_key, commit_boundary, headless_audio_health, probe_shell,
    record_probe_start,
};

use locked_timing_grid::attach_locked_timing_grid;

pub(super) fn write_recipe2_mc202_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("recipe2-mc202-probe");

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "recipe2-mc202",
        "synthetic-recipe2-mc202-probe.wav",
        "headless-recipe2-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 300, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 400, CommitBoundary::Phrase, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 500, KeyCode::Char('a'))?;
    commit_boundary(&mut shell, &mut writer, 600, CommitBoundary::Phrase, 2, 1)?;
    apply_probe_key(&mut shell, &mut writer, 700, KeyCode::Char('P'))?;
    commit_boundary(&mut shell, &mut writer, 800, CommitBoundary::Phrase, 3, 1)?;
    apply_probe_key(&mut shell, &mut writer, 900, KeyCode::Char('I'))?;
    commit_boundary(&mut shell, &mut writer, 1_000, CommitBoundary::Phrase, 4, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_100, KeyCode::Char('G'))?;
    commit_boundary(&mut shell, &mut writer, 1_200, CommitBoundary::Phrase, 5, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_300, KeyCode::Char('>'))?;

    Ok(())
}

pub(super) fn write_first_playable_jam_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("first-playable-jam-probe");

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "first-playable-jam",
        "synthetic-first-playable-source.wav",
        "headless-first-playable-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('c'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Phrase, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('o'))?;
    apply_probe_key(&mut shell, &mut writer, 500, KeyCode::Char('p'))?;
    commit_boundary(&mut shell, &mut writer, 600, CommitBoundary::Bar, 2, 2)?;
    apply_probe_key(&mut shell, &mut writer, 700, KeyCode::Char('w'))?;
    commit_boundary(&mut shell, &mut writer, 800, CommitBoundary::Beat, 3, 1)?;

    Ok(())
}

pub(super) fn write_stage_style_jam_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("stage-style-jam-probe");

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "stage-style-jam",
        "synthetic-stage-style-source.wav",
        "headless-stage-style-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('c'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Phrase, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('o'))?;
    apply_probe_key(&mut shell, &mut writer, 500, KeyCode::Char('p'))?;
    commit_boundary(&mut shell, &mut writer, 600, CommitBoundary::Bar, 2, 2)?;
    apply_probe_key(&mut shell, &mut writer, 700, KeyCode::Char('w'))?;
    commit_boundary(&mut shell, &mut writer, 800, CommitBoundary::Beat, 3, 1)?;
    apply_probe_key(&mut shell, &mut writer, 900, KeyCode::Char('f'))?;
    commit_boundary(&mut shell, &mut writer, 1_000, CommitBoundary::Bar, 4, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_100, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 1_200, CommitBoundary::Phrase, 5, 1)?;

    Ok(())
}

pub(super) fn write_stage_style_restore_diversity_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("stage-style-restore-diversity-probe");

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "stage-style-restore-diversity",
        "synthetic-stage-style-restore-diversity-source.wav",
        "headless-stage-style-restore-diversity-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('c'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Phrase, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('o'))?;
    apply_probe_key(&mut shell, &mut writer, 500, KeyCode::Char('p'))?;
    commit_boundary(&mut shell, &mut writer, 600, CommitBoundary::Bar, 2, 2)?;
    apply_probe_key(&mut shell, &mut writer, 700, KeyCode::Char('w'))?;
    commit_boundary(&mut shell, &mut writer, 800, CommitBoundary::Beat, 3, 1)?;

    apply_probe_key(&mut shell, &mut writer, 900, KeyCode::Char('f'))?;
    commit_boundary(&mut shell, &mut writer, 1_000, CommitBoundary::Bar, 4, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_100, KeyCode::Char('d'))?;
    commit_boundary(&mut shell, &mut writer, 1_200, CommitBoundary::Phrase, 5, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_300, KeyCode::Char('k'))?;
    commit_boundary(&mut shell, &mut writer, 1_400, CommitBoundary::Phrase, 6, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_500, KeyCode::Char('x'))?;
    commit_boundary(&mut shell, &mut writer, 1_600, CommitBoundary::Phrase, 7, 1)?;

    apply_probe_key(&mut shell, &mut writer, 1_700, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 1_800, CommitBoundary::Phrase, 8, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_900, KeyCode::Char('a'))?;
    commit_boundary(&mut shell, &mut writer, 2_000, CommitBoundary::Phrase, 9, 1)?;
    apply_probe_key(&mut shell, &mut writer, 2_100, KeyCode::Char('P'))?;
    commit_boundary(
        &mut shell,
        &mut writer,
        2_200,
        CommitBoundary::Phrase,
        10,
        1,
    )?;
    apply_probe_key(&mut shell, &mut writer, 2_300, KeyCode::Char('I'))?;
    commit_boundary(
        &mut shell,
        &mut writer,
        2_400,
        CommitBoundary::Phrase,
        11,
        1,
    )?;
    apply_probe_key(&mut shell, &mut writer, 2_500, KeyCode::Char('G'))?;
    commit_boundary(
        &mut shell,
        &mut writer,
        2_600,
        CommitBoundary::Phrase,
        12,
        1,
    )?;
    apply_probe_key(&mut shell, &mut writer, 2_700, KeyCode::Char('>'))?;

    Ok(())
}

pub(super) fn write_interrupted_session_recovery_observer(path: &Path) -> io::Result<()> {
    let probe_dir = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join("interrupted-session-recovery");
    fs::create_dir_all(&probe_dir)?;

    let session_path = probe_dir.join("session.json");
    let temp_path = probe_dir.join(".session.json.tmp-1776359400");
    let autosave_path = probe_dir.join("session.autosave.2026-04-30T171500Z.json");

    save_session_json(
        &session_path,
        &SessionFile::new("canonical", "0.1.0", "2026-04-30T17:15:00Z"),
    )
    .map_err(io::Error::other)?;
    fs::write(&temp_path, "{ truncated interrupted session")?;
    save_session_json(
        &autosave_path,
        &SessionFile::new("autosave", "0.1.0", "2026-04-30T17:15:01Z"),
    )
    .map_err(io::Error::other)?;

    let session_before = fs::read(&session_path)?;
    let temp_before = fs::read(&temp_path)?;
    let autosave_before = fs::read(&autosave_path)?;

    let mut shell = recovery_probe_shell(
        &session_path,
        SessionFile::new("loaded", "0.1.0", "2026-04-30T17:15:02Z"),
    )?;

    let mut writer = NdjsonWriter::open(path)?;
    record_recovery_probe_start(
        &mut writer,
        path,
        "interrupted-session-recovery",
        &session_path,
        json!({
            "temp_path": temp_path,
            "autosave_path": autosave_path,
        }),
        &shell,
    )?;
    record_recovery_probe_audio_runtime(&mut writer, &mut shell)?;

    if fs::read(&session_path)? != session_before {
        return Err(io::Error::other(
            "canonical session changed during recovery drill",
        ));
    }
    if fs::read(&temp_path)? != temp_before {
        return Err(io::Error::other(
            "temp recovery candidate changed during recovery drill",
        ));
    }
    if fs::read(&autosave_path)? != autosave_before {
        return Err(io::Error::other(
            "autosave recovery candidate changed during recovery drill",
        ));
    }

    Ok(())
}

pub(super) fn write_missing_target_recovery_observer(path: &Path) -> io::Result<()> {
    let probe_dir = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join("missing-target-recovery");
    fs::create_dir_all(&probe_dir)?;

    let session_path = probe_dir.join("session.json");
    let autosave_path = probe_dir.join("session.autosave.2026-04-30T172000Z.json");
    let _ = fs::remove_file(&session_path);
    save_session_json(
        &autosave_path,
        &SessionFile::new("autosave", "0.1.0", "2026-04-30T17:20:01Z"),
    )
    .map_err(io::Error::other)?;

    let autosave_before = fs::read(&autosave_path)?;

    let mut shell = recovery_probe_shell(
        &session_path,
        SessionFile::new("loaded-empty", "0.1.0", "2026-04-30T17:20:02Z"),
    )?;

    let mut writer = NdjsonWriter::open(path)?;
    record_recovery_probe_start(
        &mut writer,
        path,
        "missing-target-recovery",
        &session_path,
        json!({
            "autosave_path": autosave_path,
        }),
        &shell,
    )?;
    record_recovery_probe_audio_runtime(&mut writer, &mut shell)?;

    if session_path.exists() {
        return Err(io::Error::other(
            "missing target session was created during recovery drill",
        ));
    }
    if fs::read(&autosave_path)? != autosave_before {
        return Err(io::Error::other(
            "autosave recovery candidate changed during recovery drill",
        ));
    }

    Ok(())
}

fn recovery_probe_shell(
    session_path: &Path,
    loaded_session: SessionFile,
) -> io::Result<JamShellState> {
    let mut shell = JamShellState::new(
        JamAppState::from_parts(loaded_session, None, ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    let recovery_surface =
        JamAppState::scan_session_recovery_surface(session_path).map_err(io::Error::other)?;
    shell.set_recovery_surface(recovery_surface);
    Ok(shell)
}

fn record_recovery_probe_start(
    writer: &mut NdjsonWriter,
    path: &Path,
    probe: &str,
    session_path: &Path,
    drill: Value,
    shell: &JamShellState,
) -> io::Result<()> {
    writer.record(json!({
        "event": "observer_started",
        "schema": "riotbox.user_session_observer.v1",
        "timestamp_ms": 0,
        "opt_in": true,
        "capture_context": "headless_probe",
        "raw_audio_recording": false,
        "realtime_callback_io": false,
        "argv": [
            "user_session_observer_probe",
            "--probe",
            probe,
            "--observer",
            path.display().to_string(),
        ],
        "launch": {
            "mode": "load",
            "session_path": session_path,
            "source_graph_path": null,
            "observer_path": path.display().to_string(),
            "probe": probe,
            "drill": drill,
        },
        "snapshot": observer_snapshot(shell),
    }))
}

fn record_recovery_probe_audio_runtime(
    writer: &mut NdjsonWriter,
    shell: &mut JamShellState,
) -> io::Result<()> {
    shell.app.set_audio_health(headless_audio_health());
    writer.record(json!({
        "event": "audio_runtime",
        "timestamp_ms": 10,
        "status": "started",
        "error": null,
        "host": "headless-probe",
        "snapshot": observer_snapshot(shell),
    }))
}

pub(super) fn write_feral_grid_jam_observer(path: &Path) -> io::Result<()> {
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

pub(super) fn write_feral_grid_locked_jam_observer(path: &Path) -> io::Result<()> {
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
    attach_source_timing(shell, source_id, path, bpm, ProbeTimingProfile::Cautious);
}

fn attach_locked_probe_source_timing(
    shell: &mut JamShellState,
    source_id: &str,
    path: &str,
    bpm: f32,
) {
    attach_source_timing(shell, source_id, path, bpm, ProbeTimingProfile::Locked);
}

enum ProbeTimingProfile {
    Cautious,
    Locked,
}

fn attach_source_timing(
    shell: &mut JamShellState,
    source_id: &str,
    path: &str,
    bpm: f32,
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
    graph.timing.bpm_estimate = Some(bpm);
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
        ProbeTimingProfile::Locked => {
            graph.timing.bpm_confidence = 0.92;
            graph.timing.quality = TimingQuality::High;
            graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
            attach_locked_timing_grid(&mut graph, bpm);
        }
    }

    shell.app.source_graph = Some(graph);
    shell.app.refresh_view();
}
