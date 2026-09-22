use crate::jam_app::{JamAppError, JamAppState};
use riotbox_core::{
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        CommitBoundary, Quantization, UndoPolicy,
    },
    ids::ActionId,
    persistence::save_session_json,
    session::{ActionCommitRecord, SessionFile},
    transport::CommitBoundaryState,
};
use std::{hint::black_box, time::Instant};

fn history(count: u64, undo_pairs: bool) -> SessionFile {
    let mut session = SessionFile::new("synthetic-history", "test", "2026-09-22");
    for n in 0..count {
        let id = ActionId(n + 1);
        let mut action = Action {
            id,
            actor: ActorType::User,
            command: ActionCommand::TransportPlay,
            params: ActionParams::Empty,
            target: ActionTarget::default(),
            requested_at: n,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(n + 10),
            result: Some(ActionResult {
                accepted: true,
                summary: "synthetic".into(),
            }),
            undo_policy: UndoPolicy::NotUndoable {
                reason: "synthetic".into(),
            },
            explanation: None,
        };
        if undo_pairs {
            if n % 2 == 0 {
                action.command = ActionCommand::Tr909FillNext;
                action.status = ActionStatus::Undone;
                action.undo_policy = UndoPolicy::Undoable;
            } else {
                action.command = ActionCommand::UndoLast;
                action.params = ActionParams::Undo {
                    target_action_id: ActionId(n),
                };
            }
        }
        session.action_log.actions.push(action);
        session.action_log.commit_records.push(ActionCommitRecord {
            action_id: id,
            boundary: CommitBoundaryState {
                kind: CommitBoundary::Immediate,
                beat_index: n,
                bar_index: n / 4 + 1,
                phrase_index: n / 16 + 1,
                scene_id: None,
            },
            commit_sequence: 1,
            committed_at: n + 10,
            mc202_source_phrase_plan: None,
        });
    }
    session
}

fn restore(session: &SessionFile) -> Result<JamAppState, JamAppError> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.json");
    save_session_json(&path, session).unwrap();
    JamAppState::from_json_files_for_export_metadata(path)
}

#[test]
fn long_histories_restore_without_changing_actions_or_commits() {
    for undo_pairs in [false, true] {
        let session = history(2048, undo_pairs);
        let restored = restore(&session).unwrap();
        assert_eq!(restored.session.action_log, session.action_log);
    }
}

#[test]
fn malformed_history_keeps_specific_rejection_diagnostics() {
    type InvalidHistoryCase = (&'static str, fn(&mut SessionFile));
    let cases: &[InvalidHistoryCase] = &[
        ("duplicate action id", |s| {
            s.action_log.actions[1].id = ActionId(1)
        }),
        ("references missing action", |s| {
            s.action_log.commit_records[1].action_id = ActionId(99)
        }),
        ("invalid committed-history status", |s| {
            s.action_log.actions[1].status = ActionStatus::Queued
        }),
        ("without committed_at", |s| {
            s.action_log.actions[1].committed_at = None
        }),
        ("but action has committed_at", |s| {
            s.action_log.commit_records[1].committed_at = 0
        }),
        ("invalid sequence 0", |s| {
            s.action_log.commit_records[1].commit_sequence = 0
        }),
        ("commit record is duplicated", |s| {
            s.action_log
                .commit_records
                .push(s.action_log.commit_records[0].clone())
        }),
        ("duplicated within boundary", |s| {
            s.action_log.commit_records[1].boundary =
                s.action_log.commit_records[0].boundary.clone()
        }),
    ];
    for (expected, corrupt) in cases {
        let mut session = history(4, false);
        corrupt(&mut session);
        assert!(
            restore(&session)
                .unwrap_err()
                .to_string()
                .contains(expected),
            "{expected}"
        );
    }
}

#[test]
fn earliest_duplicate_record_controls_error_precedence() {
    let mut session = history(3, false);
    // The third record collides with record 0's boundary and record 1's ID.
    // Preserve the legacy nested loop's earliest-previous-record diagnostic.
    session.action_log.commit_records[2] = session.action_log.commit_records[1].clone();
    session.action_log.commit_records[2].boundary =
        session.action_log.commit_records[0].boundary.clone();
    assert!(
        restore(&session)
            .unwrap_err()
            .to_string()
            .contains("duplicated within boundary")
    );
    session.action_log.commit_records.swap(0, 1);
    assert!(
        restore(&session)
            .unwrap_err()
            .to_string()
            .contains("commit record is duplicated")
    );
}

#[test]
fn legacy_undone_fill_requires_a_committed_typed_marker() {
    let mut session = history(2, true);
    assert!(restore(&session).is_ok());
    session.action_log.actions.pop();
    session.action_log.commit_records.pop();
    assert!(
        restore(&session)
            .unwrap_err()
            .to_string()
            .contains("no trusted typed undo marker")
    );
}

#[test]
fn sequence_uniqueness_uses_every_boundary_field() {
    for dimension in 0..5 {
        let mut session = history(2, false);
        let mut boundary = session.action_log.commit_records[0].boundary.clone();
        match dimension {
            0 => boundary.kind = CommitBoundary::Beat,
            1 => boundary.beat_index += 1,
            2 => boundary.bar_index += 1,
            3 => boundary.phrase_index += 1,
            4 => boundary.scene_id = Some("different-scene".into()),
            _ => unreachable!(),
        }
        session.action_log.commit_records[1].boundary = boundary;
        assert!(restore(&session).is_ok(), "boundary field {dimension}");
    }
}

/// Source-free end-to-end metadata restore, including JSON parse, undo-policy
/// normalization, validation, replay-plan construction and view projection.
/// Informational medians only: never use machine-dependent times as a CI gate.
#[test]
#[ignore = "manual release-mode scaling measurement"]
fn benchmark_restore_history_scaling() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.json");
    for undo_pairs in [false, true] {
        for count in [1000, 4000, 16000] {
            save_session_json(&path, &history(count, undo_pairs)).unwrap();
            let mut times = Vec::new();
            for _ in 0..5 {
                let start = Instant::now();
                let restored = JamAppState::from_json_files_for_export_metadata(&path).unwrap();
                black_box(&restored);
                times.push(start.elapsed().as_micros());
            }
            times.sort_unstable();
            println!(
                "restore_history undo_pairs={undo_pairs} actions={count} commits={count} median_us={}",
                times[2]
            );
        }
    }
}
