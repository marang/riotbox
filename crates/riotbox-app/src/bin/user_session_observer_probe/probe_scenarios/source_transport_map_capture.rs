use std::{f32::consts::PI, io, path::Path};

use crossterm::event::KeyCode;
use riotbox_app::ui::JamShellState;
use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_core::{
    action::CommitBoundary,
    ids::SourceId,
    source_graph::{
        DecodeProfile, GraphProvenance, SourceDescriptor, SourceGraph, TimingDegradedPolicy,
        TimingQuality,
    },
    transport::CommitBoundaryState,
};

use super::{
    NdjsonWriter, apply_probe_key, locked_timing_grid::attach_locked_timing_grid, probe_shell,
    record_probe_start,
};

const SOURCE_PATH: &str = "synthetic-source-transport-map-capture.wav";
const SOURCE_ID: &str = "src-source-transport-map-capture";
const SOURCE_BPM: f32 = 128.0;
const SOURCE_SAMPLE_RATE: u32 = 44_100;
const SOURCE_CHANNELS: u16 = 2;
const SOURCE_SECONDS: f32 = 8.0;

pub(crate) fn write_source_transport_map_capture_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("source-transport-map-capture-probe");
    attach_manual_confirm_source_transport(&mut shell)?;

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "source-transport-map-capture",
        SOURCE_PATH,
        "headless-source-transport-map-capture-session.json",
    )?;
    assert_unconfirmed_listen_first_state(&shell)?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('C'))?;
    assert_confirmed_map_state(&shell)?;
    apply_probe_key(&mut shell, &mut writer, 300, KeyCode::Right)?;
    assert_seeked_source_map_state(&shell)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('c'))?;
    commit_at_beat(&mut shell, &mut writer, 500, CommitBoundary::Bar, 8, 1)?;
    assert_source_window_capture_state(&shell)?;
    apply_probe_key(&mut shell, &mut writer, 600, KeyCode::Char('o'))?;
    commit_at_beat(&mut shell, &mut writer, 700, CommitBoundary::Bar, 12, 1)?;
    apply_probe_key(&mut shell, &mut writer, 800, KeyCode::Char('p'))?;
    commit_at_beat(&mut shell, &mut writer, 900, CommitBoundary::Bar, 16, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_000, KeyCode::Char('w'))?;
    commit_at_beat(&mut shell, &mut writer, 1_100, CommitBoundary::Beat, 17, 1)?;
    assert_promoted_w30_hit_state(&shell)
}

fn attach_manual_confirm_source_transport(shell: &mut JamShellState) -> io::Result<()> {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from(SOURCE_ID),
            path: SOURCE_PATH.into(),
            content_hash: "headless-source-transport-map-capture-hash".into(),
            duration_seconds: SOURCE_SECONDS,
            sample_rate: SOURCE_SAMPLE_RATE,
            channel_count: SOURCE_CHANNELS,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "headless-probe".into(),
            provider_set: vec!["user_session_observer_probe".into()],
            generated_at: "2026-05-23T00:00:00Z".into(),
            source_hash: "headless-source-transport-map-capture-hash".into(),
            analysis_seed: 31,
            run_notes: Some("source transport map capture workflow observer probe".into()),
        },
    );
    graph.timing.bpm_estimate = Some(SOURCE_BPM);
    graph.timing.bpm_confidence = 0.72;
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    attach_locked_timing_grid(&mut graph, SOURCE_BPM);
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;

    shell.app.source_graph = Some(graph);
    shell.app.source_audio_cache = Some(source_audio_cache()?);
    shell.app.refresh_view();
    Ok(())
}

fn source_audio_cache() -> io::Result<SourceAudioCache> {
    let frame_count = (SOURCE_SAMPLE_RATE as f32 * SOURCE_SECONDS) as usize;
    let mut samples = Vec::with_capacity(frame_count * usize::from(SOURCE_CHANNELS));
    for frame_index in 0..frame_count {
        let seconds = frame_index as f32 / SOURCE_SAMPLE_RATE as f32;
        let sample = (seconds * 220.0 * 2.0 * PI).sin() * 0.35;
        samples.push(sample);
        samples.push(-sample);
    }
    SourceAudioCache::from_interleaved_samples(
        SOURCE_PATH,
        SOURCE_SAMPLE_RATE,
        SOURCE_CHANNELS,
        samples,
    )
    .map_err(io::Error::other)
}

fn assert_unconfirmed_listen_first_state(shell: &JamShellState) -> io::Result<()> {
    let source_map = &shell.app.jam_view.source.source_map;
    if shell.app.runtime_view.source_monitor_audio_route != "source_only" {
        return Err(io::Error::other(
            "source transport probe should start with source monitor audio available",
        ));
    }
    if source_map.trust_label != "needs confirm"
        || source_map.mode.label() != "time fallback"
        || source_map.capture_range_row != ".".repeat(source_map.width)
    {
        return Err(io::Error::other(
            "source transport probe should start in listen-first map fallback",
        ));
    }
    Ok(())
}

fn assert_confirmed_map_state(shell: &JamShellState) -> io::Result<()> {
    let source_map = &shell.app.jam_view.source.source_map;
    if source_map.trust_label != "grid confirmed"
        || source_map.mode.label() != "bar grid"
        || !source_map.capture_range_row.contains('[')
    {
        return Err(io::Error::other(
            "confirmed source transport probe did not expose bar-grid capture projection",
        ));
    }
    let graph = shell
        .app
        .source_graph
        .as_ref()
        .ok_or_else(|| io::Error::other("source transport probe lost source graph"))?;
    if graph.timing.degraded_policy != TimingDegradedPolicy::ManualConfirm {
        return Err(io::Error::other(
            "source transport probe mutated analyzer timing policy",
        ));
    }
    Ok(())
}

fn assert_seeked_source_map_state(shell: &JamShellState) -> io::Result<()> {
    if shell.app.session.runtime_state.transport.position_beats != 4.0 {
        return Err(io::Error::other(
            "source map navigation did not commit expected transport seek",
        ));
    }
    if !shell.app.session.runtime_state.transport.is_playing {
        return Err(io::Error::other(
            "source map navigation should preserve transport playback state",
        ));
    }
    Ok(())
}

fn assert_source_window_capture_state(shell: &JamShellState) -> io::Result<()> {
    let capture = shell
        .app
        .session
        .captures
        .last()
        .ok_or_else(|| io::Error::other("source transport probe did not capture"))?;
    let source_window = capture
        .source_window
        .as_ref()
        .ok_or_else(|| io::Error::other("source transport capture missed source window"))?;
    if source_window.source_id != SourceId::from(SOURCE_ID) {
        return Err(io::Error::other(
            "source transport capture recorded mismatched source id",
        ));
    }
    if source_window.end_seconds <= source_window.start_seconds {
        return Err(io::Error::other(
            "source transport capture recorded an empty source window",
        ));
    }
    Ok(())
}

fn assert_promoted_w30_hit_state(shell: &JamShellState) -> io::Result<()> {
    if shell
        .app
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .is_none()
    {
        return Err(io::Error::other(
            "source transport probe lost promoted W-30 capture state",
        ));
    }
    if shell.app.runtime.w30_preview.capture_id.is_none() {
        return Err(io::Error::other(
            "source transport probe did not project W-30 hit preview state",
        ));
    }
    Ok(())
}

fn commit_at_beat(
    shell: &mut JamShellState,
    writer: &mut NdjsonWriter,
    timestamp_ms: u64,
    kind: CommitBoundary,
    beat_index: u64,
    expected_count: usize,
) -> io::Result<()> {
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind,
            beat_index,
            bar_index: beat_index / 4,
            phrase_index: beat_index / 16,
            scene_id: None,
        },
        timestamp_ms,
    );
    if committed.len() != expected_count {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "expected {expected_count} committed action(s) at {kind:?} beat {beat_index}, got {}",
                committed.len()
            ),
        ));
    }

    writer.record(serde_json::json!({
        "event": "transport_commit",
        "timestamp_ms": timestamp_ms,
        "committed": committed.iter().map(super::super::compact_commit).collect::<Vec<_>>(),
        "snapshot": riotbox_app::observer::observer_snapshot(shell),
    }))
}
