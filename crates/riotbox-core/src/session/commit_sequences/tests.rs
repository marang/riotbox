use crate::{
    action::CommitBoundary,
    ids::{ActionId, SceneId},
    session::{ActionCommitRecord, ActionLog, CommitSequenceExhausted},
    transport::CommitBoundaryState,
};

fn boundary() -> CommitBoundaryState {
    CommitBoundaryState {
        kind: CommitBoundary::Immediate,
        beat_index: 12,
        bar_index: 4,
        phrase_index: 1,
        scene_id: None,
    }
}

fn log(sequences: &[u32]) -> ActionLog {
    ActionLog {
        commit_records: sequences
            .iter()
            .enumerate()
            .map(|(index, &sequence)| ActionCommitRecord {
                action_id: ActionId(index as u64 + 1),
                boundary: boundary(),
                commit_sequence: sequence,
                committed_at: 2,
                mc202_source_phrase_plan: None,
            })
            .collect(),
        ..ActionLog::default()
    }
}

#[test]
fn empty_history_starts_at_one_and_empty_range_stays_empty() {
    let log = ActionLog::default();
    assert_eq!(
        log.prepare_commit_sequences(&boundary(), 3)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    let mut empty = log.prepare_commit_sequences(&boundary(), 0).unwrap();
    assert_eq!(empty.next(), None);
    assert_eq!(empty.next(), None);
    assert!(log.actions.is_empty());
    assert!(log.commit_records.is_empty());
}

#[test]
fn maximum_boundary_accepts_zero_but_refuses_one_with_typed_error() {
    let log = log(&[u32::MAX]);
    let before = log.clone();
    assert_eq!(
        log.prepare_commit_sequences(&boundary(), 0).unwrap().next(),
        None
    );
    let error = log.prepare_commit_sequences(&boundary(), 1).unwrap_err();
    assert_eq!(
        error,
        CommitSequenceExhausted {
            boundary: boundary(),
            requested_count: 1,
            max_existing: u32::MAX,
        }
    );
    let standard_error: &dyn std::error::Error = &error;
    assert!(
        standard_error
            .to_string()
            .contains("commit sequence exhausted")
    );
    assert!(standard_error.source().is_none());
    assert_eq!(log, before);
}

#[test]
fn final_slot_is_issued_once_per_range_and_whole_two_slot_request_is_refused() {
    let log = log(&[u32::MAX - 1]);
    let mut final_slot = log.prepare_commit_sequences(&boundary(), 1).unwrap();
    assert_eq!(final_slot.next(), Some(u32::MAX));
    assert_eq!(final_slot.next(), None);
    assert_eq!(final_slot.next(), None);
    assert_eq!(
        log.prepare_commit_sequences(&boundary(), 2)
            .unwrap_err()
            .requested_count,
        2
    );
    // Preparation is arithmetic evidence, not a reservation or persisted cursor.
    assert_eq!(
        log.prepare_commit_sequences(&boundary(), 1).unwrap().next(),
        Some(u32::MAX)
    );
}

#[test]
fn maximum_is_derived_from_all_matching_records_not_order_or_record_count() {
    let log = log(&[3, 100, 5]);
    assert_eq!(
        log.prepare_commit_sequences(&boundary(), 2)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![101, 102]
    );
}

#[test]
fn every_complete_boundary_field_has_independent_capacity() {
    let log = log(&[u32::MAX]);
    let original = boundary();
    let mut variants = Vec::new();
    let mut changed = original.clone();
    changed.kind = CommitBoundary::Beat;
    variants.push(changed);
    let mut changed = original.clone();
    changed.beat_index += 1;
    variants.push(changed);
    let mut changed = original.clone();
    changed.bar_index += 1;
    variants.push(changed);
    let mut changed = original.clone();
    changed.phrase_index += 1;
    variants.push(changed);
    let mut changed = original;
    changed.scene_id = Some(SceneId::from("generated-scene"));
    variants.push(changed);
    for changed in variants {
        assert_eq!(
            log.prepare_commit_sequences(&changed, 1).unwrap().next(),
            Some(1)
        );
    }
}

#[test]
fn full_u32_count_is_checked_without_allocating_or_iterating_the_range() {
    let mut range = ActionLog::default()
        .prepare_commit_sequences(&boundary(), u32::MAX as usize)
        .unwrap();
    assert_eq!(range.next(), Some(1));
    assert!(
        log(&[1])
            .prepare_commit_sequences(&boundary(), u32::MAX as usize)
            .is_err()
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn counts_larger_than_u32_fail_without_truncation() {
    let count = u32::MAX as usize + 1;
    let error = ActionLog::default()
        .prepare_commit_sequences(&boundary(), count)
        .unwrap_err();
    assert_eq!(error.requested_count, count);
    assert_eq!(error.max_existing, 0);
}
