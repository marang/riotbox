fn add_feral_ready_evidence(graph: &mut SourceGraph) {
    graph.assets.push(Asset {
        asset_id: AssetId::from("asset-hook"),
        asset_type: AssetType::HookFragment,
        start_seconds: 4.0,
        end_seconds: 5.0,
        start_bar: 3,
        end_bar: 3,
        confidence: 0.88,
        tags: vec!["feral".into()],
        source_refs: vec!["src-1".into()],
    });
    graph.candidates.push(Candidate {
        candidate_id: "candidate-capture".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: "asset-hook".into(),
        score: 0.86,
        confidence: 0.84,
        tags: vec!["capture_first".into()],
        constraints: vec!["lineage_safe".into()],
        provenance_refs: vec!["test".into()],
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::SupportsBreakRebuild,
        from_id: "asset-hook".into(),
        to_id: "section-a".into(),
        weight: 0.82,
        notes: Some("feral support".into()),
    });
}

fn add_quote_risk_evidence(graph: &mut SourceGraph) {
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::HighQuoteRiskWith,
        from_id: "asset-hook".into(),
        to_id: "src-1".into(),
        weight: 0.7,
        notes: Some("quote-risk guard".into()),
    });
}

fn lineage_ready_resample_state(graph: SourceGraph) -> JamAppState {
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    state.session.captures[0].is_pinned = true;
    state.session.captures[0].lineage_capture_refs = vec![CaptureId::from("cap-root")];
    state.session.captures[0].resample_generation_depth = 1;
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();
    state
}

fn commit_lineage_ready_resample(state: &mut JamAppState) {
    assert_eq!(
        state.queue_w30_internal_resample(650),
        Some(QueueControlResult::Enqueued)
    );
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        740,
    );
    assert_eq!(committed.len(), 1);
}

#[test]
fn committed_w30_internal_resample_holds_feral_rebake_approval_on_quote_risk() {
    let mut graph = sample_graph();
    add_feral_ready_evidence(&mut graph);
    add_quote_risk_evidence(&mut graph);
    let mut state = lineage_ready_resample_state(graph);

    commit_lineage_ready_resample(&mut state);

    let capture = state
        .session
        .captures
        .last()
        .expect("new resample capture should exist");
    let notes = capture.notes.as_deref().expect("resample notes");
    assert!(notes.contains("feral rebake held: quote risk 1"));
    assert!(!notes.contains("feral rebake approved"));
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("feral rebake held: quote risk 1, lineage-safe W-30 reuse, gen 2, lineage 2")
    );
    assert_eq!(
        state.jam_view.capture.last_capture_notes.as_deref(),
        Some(notes)
    );
}
