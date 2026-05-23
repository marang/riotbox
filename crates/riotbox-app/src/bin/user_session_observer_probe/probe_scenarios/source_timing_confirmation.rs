use std::{io, path::Path};

use crossterm::event::KeyCode;
use riotbox_app::ui::JamShellState;
use riotbox_core::{
    action::ActionCommand,
    ids::SourceId,
    source_graph::{
        DecodeProfile, GraphProvenance, MeterHint, SourceDescriptor, SourceGraph,
        TimingDegradedPolicy, TimingHypothesis, TimingHypothesisKind, TimingQuality, TimingWarning,
        TimingWarningCode,
    },
};

use super::{NdjsonWriter, apply_probe_key, probe_shell, record_probe_start};

pub(crate) fn write_source_timing_confirmation_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("source-timing-confirmation-probe");
    attach_manual_confirm_source_timing(&mut shell);

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "source-timing-confirmation",
        "synthetic-source-timing-confirmation.wav",
        "headless-source-timing-confirmation-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char('C'))?;
    assert_source_timing_confirmation_probe_state(&shell)
}

fn attach_manual_confirm_source_timing(shell: &mut JamShellState) {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-source-timing-confirmation"),
            path: "synthetic-source-timing-confirmation.wav".into(),
            content_hash: "headless-source-timing-confirmation-hash".into(),
            duration_seconds: 8.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "headless-probe".into(),
            provider_set: vec!["user_session_observer_probe".into()],
            generated_at: "2026-05-23T00:00:00Z".into(),
            source_hash: "headless-source-timing-confirmation-hash".into(),
            analysis_seed: 23,
            run_notes: Some("manual-confirm source timing confirmation observer probe".into()),
        },
    );
    graph.timing.bpm_estimate = Some(128.0);
    graph.timing.bpm_confidence = 0.72;
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    graph.timing.primary_hypothesis_id = Some("probe-primary".into());
    graph.timing.hypotheses.push(TimingHypothesis {
        hypothesis_id: "probe-primary".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: 128.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.72,
        score: 0.68,
        beat_grid: Vec::new(),
        bar_grid: Vec::new(),
        phrase_grid: Vec::new(),
        anchors: Vec::new(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::Low,
        warnings: Vec::new(),
        provenance: vec!["user_session_observer_probe.source_timing_confirmation".into()],
    });
    graph.timing.warnings.push(TimingWarning {
        code: TimingWarningCode::AmbiguousDownbeat,
        message: "headless confirmation probe requires musician trust".into(),
    });

    shell.app.source_graph = Some(graph);
    shell.app.refresh_view();
}

fn assert_source_timing_confirmation_probe_state(shell: &JamShellState) -> io::Result<()> {
    let graph =
        shell.app.source_graph.as_ref().ok_or_else(|| {
            io::Error::other("source timing confirmation probe lost source graph")
        })?;
    if graph.timing.degraded_policy != TimingDegradedPolicy::ManualConfirm {
        return Err(io::Error::other(
            "source timing confirmation probe mutated Source Graph timing policy",
        ));
    }
    if !graph
        .timing
        .warnings
        .iter()
        .any(|warning| warning.code == TimingWarningCode::AmbiguousDownbeat)
    {
        return Err(io::Error::other(
            "source timing confirmation probe lost Source Graph warning evidence",
        ));
    }
    if shell.app.jam_view.source.timing.cue != "needs confirm" {
        return Err(io::Error::other(
            "source timing confirmation probe mutated analyzer cue",
        ));
    }

    let action = shell
        .app
        .session
        .action_log
        .actions
        .last()
        .ok_or_else(|| io::Error::other("source timing confirmation did not commit"))?;
    if action.command != ActionCommand::SourceTimingConfirmGrid {
        return Err(io::Error::other(
            "source timing confirmation committed the wrong command",
        ));
    }
    if action.committed_at != Some(100) {
        return Err(io::Error::other(
            "source timing confirmation committed at the wrong probe timestamp",
        ));
    }

    let confirmed = shell
        .app
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .ok_or_else(|| io::Error::other("source timing confirmation state was not persisted"))?;
    if confirmed.source_id != graph.source.source_id {
        return Err(io::Error::other(
            "source timing confirmation persisted a mismatched source id",
        ));
    }
    if confirmed.hypothesis_id.as_deref() != Some("probe-primary") {
        return Err(io::Error::other(
            "source timing confirmation persisted a mismatched hypothesis id",
        ));
    }

    Ok(())
}
