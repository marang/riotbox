use crate::session::SessionFile;
use crate::session::Tr909ReinforcementModeState;
use crate::session::Tr909TakeoverProfileState;

pub(super) fn w30_pending_audition_view(
    action: &crate::action::Action,
    kind: W30PendingAuditionKind,
) -> Option<W30PendingAuditionView> {
    action
        .target
        .bank_id
        .as_ref()
        .zip(action.target.pad_id.as_ref())
        .map(|(bank_id, pad_id)| W30PendingAuditionView {
            kind,
            target: format!("{bank_id}/{pad_id}"),
            quantization: action.quantization.to_string(),
        })
}

#[derive(Clone, Debug, PartialEq)]
pub struct MacroStripView {
    pub source_retain: f32,
    pub chaos: f32,
    pub mc202_touch: f32,
    pub w30_grit: f32,
    pub tr909_slam: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneSummaryView {
    pub mc202_role: Option<String>,
    pub mc202_pending_role: Option<String>,
    pub mc202_pending_follower_generation: bool,
    pub mc202_pending_answer_generation: bool,
    pub mc202_pending_pressure_generation: bool,
    pub mc202_pending_instigator_generation: bool,
    pub mc202_pending_phrase_mutation: bool,
    pub mc202_phrase_ref: Option<String>,
    pub mc202_phrase_variant: Option<String>,
    pub w30_active_bank: Option<String>,
    pub w30_focused_pad: Option<String>,
    pub w30_pending_trigger_target: Option<String>,
    pub w30_pending_recall_target: Option<String>,
    pub w30_pending_audition: Option<W30PendingAuditionView>,
    pub w30_pending_audition_target: Option<String>,
    pub w30_pending_bank_swap_target: Option<String>,
    pub w30_pending_slice_pool_target: Option<String>,
    pub w30_pending_slice_pool_capture_id: Option<String>,
    pub w30_pending_slice_pool_reason: Option<String>,
    pub w30_pending_damage_profile_target: Option<String>,
    pub w30_pending_loop_freeze_target: Option<String>,
    pub w30_pending_focus_step_target: Option<String>,
    pub w30_pending_resample_capture_id: Option<String>,
    pub tr909_slam_enabled: bool,
    pub tr909_takeover_enabled: bool,
    pub tr909_takeover_pending_target: Option<bool>,
    pub tr909_takeover_pending_profile: Option<Tr909TakeoverProfileState>,
    pub tr909_takeover_profile: Option<Tr909TakeoverProfileState>,
    pub tr909_fill_armed_next_bar: bool,
    pub tr909_last_fill_bar: Option<u64>,
    pub tr909_reinforcement_mode: Option<Tr909ReinforcementModeState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct W30PendingAuditionView {
    pub kind: W30PendingAuditionKind,
    pub target: String,
    pub quantization: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum W30PendingAuditionKind {
    RawCapture,
    Promoted,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingActionView {
    pub id: String,
    pub actor: String,
    pub command: String,
    pub quantization: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentActionView {
    pub id: String,
    pub actor: String,
    pub command: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GhostStatusView {
    pub mode: String,
    pub suggestion_count: usize,
    pub is_blocked: bool,
    pub is_read_only: bool,
    pub latest_proposal_id: Option<String>,
    pub latest_summary: Option<String>,
    pub latest_status: Option<String>,
    pub decision_hint: Option<String>,
    pub safety: String,
    pub active_blocker: Option<String>,
}

pub(super) fn ghost_status_view(session: &SessionFile) -> GhostStatusView {
    let latest = session.ghost_state.suggestion_history.last();
    let active_blocker = session
        .runtime_state
        .lock_state
        .locked_object_ids
        .iter()
        .find(|lock| lock.contains("ghost"))
        .cloned();
    let is_blocked = active_blocker.is_some();

    GhostStatusView {
        mode: session.ghost_state.mode.to_string(),
        suggestion_count: session.ghost_state.suggestion_history.len(),
        is_blocked,
        is_read_only: matches!(session.ghost_state.mode, crate::action::GhostMode::Watch),
        latest_proposal_id: latest.map(|suggestion| suggestion.proposal_id.clone()),
        latest_summary: latest.map(|suggestion| suggestion.summary.clone()),
        latest_status: latest.map(|suggestion| suggestion.status().label().into()),
        decision_hint: latest.map(|suggestion| {
            if is_blocked {
                "blocked".into()
            } else if suggestion.rejected {
                "rejected".into()
            } else if suggestion.accepted {
                "queued intent".into()
            } else if matches!(session.ghost_state.mode, crate::action::GhostMode::Assist) {
                "accept/reject".into()
            } else {
                "assist required".into()
            }
        }),
        safety: if is_blocked {
            "blocked".into()
        } else {
            "clear".into()
        },
        active_blocker,
    }
}
