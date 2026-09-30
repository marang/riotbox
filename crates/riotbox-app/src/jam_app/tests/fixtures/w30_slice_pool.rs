use crate::jam_app::state::JamAppState;
use crate::jam_app::tests::fixtures::session_source::sample_session;
use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_core::ids::BankId;
use riotbox_core::ids::CaptureId;
use riotbox_core::ids::PadId;
use riotbox_core::queue::ActionQueue;
use riotbox_core::session::CaptureRef;
use riotbox_core::session::CaptureSourceWindow;
use riotbox_core::session::CaptureTarget;
use riotbox_core::session::CaptureType;
use riotbox_core::source_graph::SourceGraph;

pub(in crate::jam_app::tests) fn w30_slice_pool_state_with_source_windows(
    graph: SourceGraph,
    source_audio_cache: SourceAudioCache,
) -> JamAppState {
    let mut session = sample_session(&graph);
    session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    session.captures[0].source_window = Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds: 0.0,
        end_seconds: 0.5,
        start_frame: 0,
        end_frame: 24_000,
        hook_selection: None,
    });
    session.captures.push(CaptureRef {
        audio_identity: None,
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-c".into()],
        source_window: Some(CaptureSourceWindow {
            source_id: graph.source.source_id.clone(),
            start_seconds: 0.05,
            end_seconds: 0.55,
            start_frame: 2_400,
            end_frame: 26_400,
            hook_selection: None,
        }),
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: Some("cyclic slice".into()),
    });
    session.captures.push(CaptureRef {
        audio_identity: None,
        capture_id: CaptureId::from("cap-03"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-b".into()],
        source_window: Some(CaptureSourceWindow {
            source_id: graph.source.source_id.clone(),
            start_seconds: 0.123,
            end_seconds: 0.623,
            start_frame: 5_904,
            end_frame: 29_904,
            hook_selection: None,
        }),
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-03.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: Some("feral hook slice".into()),
    });
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.source_audio_cache = Some(source_audio_cache);
    state.refresh_view();
    state
}
