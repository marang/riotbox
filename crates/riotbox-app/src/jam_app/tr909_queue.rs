use super::state::{JamAppState, QueueControlResult};
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        SourceMonitorMode, TargetScope,
    },
    live_performance_policy::{LivePerformanceCharacter, derive_live_performance_policy},
    session::{Tr909ReinforcementModeState, Tr909TakeoverProfileState},
    style::PerformancePresetId,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909CutHitReturnUnavailableReason {
    TransportStopped,
    PresetInactive,
    SourceMonitorNotRiotbox,
    MissingTrustedLivePolicy,
    SourceCharacterNotDenseBreak,
    Tr909NotReinforcingBreak,
    SlamAlreadyEnabled,
}

impl Tr909CutHitReturnUnavailableReason {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TransportStopped => "transport stopped",
            Self::PresetInactive => "Feral Break Alpha v2 inactive",
            Self::SourceMonitorNotRiotbox => "Source Monitor is not Riotbox",
            Self::MissingTrustedLivePolicy => "trusted live source policy unavailable",
            Self::SourceCharacterNotDenseBreak => "source is not a dense break",
            Self::Tr909NotReinforcingBreak => "TR-909 break reinforcement inactive",
            Self::SlamAlreadyEnabled => "TR-909 slam already enabled",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909CutHitReturnQueueResult {
    Enqueued,
    AlreadyPending,
    Unavailable(Tr909CutHitReturnUnavailableReason),
}

impl JamAppState {
    pub fn queue_tr909_fill(&mut self, requested_at: TimestampMs) {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.explanation = Some("trigger TR-909 fill on next bar".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
    }

    pub fn queue_tr909_reinforce(&mut self, requested_at: TimestampMs) {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909ReinforceBreak,
            Quantization::NextPhrase,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.explanation = Some("reinforce next phrase with TR-909 drum layer".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
    }

    fn tr909_takeover_change_pending(&self) -> bool {
        self.queue.pending_actions().iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::Tr909Takeover
                    | ActionCommand::Tr909SceneLock
                    | ActionCommand::Tr909Release
            )
        })
    }

    pub fn queue_tr909_slam_toggle(&mut self, requested_at: TimestampMs) -> bool {
        if self
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.command == ActionCommand::Tr909SetSlam)
        {
            return false;
        }

        let enabling = !self.session.runtime_state.lane_state.tr909.slam_enabled;
        let intensity = if enabling {
            self.session.runtime_state.macro_state.tr909_slam.max(0.85)
        } else {
            0.0
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909SetSlam,
            Quantization::NextBeat,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity,
            target_id: Some(if enabling { "enabled" } else { "disabled" }.into()),
        };
        draft.explanation = Some(if enabling {
            format!("enable TR-909 slam at {:.2}", intensity)
        } else {
            "disable TR-909 slam".into()
        });
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        true
    }

    pub fn queue_tr909_cut_hit_return(
        &mut self,
        requested_at: TimestampMs,
    ) -> Tr909CutHitReturnQueueResult {
        if self.queue.pending_actions().iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::Tr909FillNext | ActionCommand::Tr909SetSlam
            )
        }) {
            return Tr909CutHitReturnQueueResult::AlreadyPending;
        }

        let unavailable = if !self.session.runtime_state.transport.is_playing {
            Some(Tr909CutHitReturnUnavailableReason::TransportStopped)
        } else if self.session.runtime_state.style.active_preset
            != Some(PerformancePresetId::FeralBreakAlphaV2)
        {
            Some(Tr909CutHitReturnUnavailableReason::PresetInactive)
        } else if self.session.runtime_state.source_monitor.mode != SourceMonitorMode::Riotbox {
            Some(Tr909CutHitReturnUnavailableReason::SourceMonitorNotRiotbox)
        } else if self
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode
            != Some(Tr909ReinforcementModeState::BreakReinforce)
        {
            Some(Tr909CutHitReturnUnavailableReason::Tr909NotReinforcingBreak)
        } else if self.session.runtime_state.lane_state.tr909.slam_enabled {
            Some(Tr909CutHitReturnUnavailableReason::SlamAlreadyEnabled)
        } else {
            match self
                .source_graph
                .as_ref()
                .and_then(|graph| derive_live_performance_policy(&self.session, graph))
            {
                None => Some(Tr909CutHitReturnUnavailableReason::MissingTrustedLivePolicy),
                Some(policy) if policy.character != LivePerformanceCharacter::DenseBreak => {
                    Some(Tr909CutHitReturnUnavailableReason::SourceCharacterNotDenseBreak)
                }
                Some(_) => None,
            }
        };
        if let Some(reason) = unavailable {
            return Tr909CutHitReturnQueueResult::Unavailable(reason);
        }

        let mut fill = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        fill.explanation = Some("cut the source-backed bed and land the late TR-909 hit".into());
        self.queue.enqueue(fill, requested_at);

        let intensity = self.session.runtime_state.macro_state.tr909_slam.max(0.85);
        let mut slam = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909SetSlam,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        slam.params = ActionParams::Mutation {
            intensity,
            target_id: Some("enabled".into()),
        };
        slam.explanation = Some(format!(
            "hold TR-909 slam at {intensity:.2} after the cut-hit return"
        ));
        self.queue.enqueue(slam, requested_at);

        self.refresh_view();
        Tr909CutHitReturnQueueResult::Enqueued
    }

    pub fn queue_tr909_takeover(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.tr909_takeover_change_pending() {
            return QueueControlResult::AlreadyPending;
        }
        if self.session.runtime_state.lane_state.tr909.takeover_enabled {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909Takeover,
            Quantization::NextPhrase,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some("takeover".into()),
        };
        draft.explanation = Some("engage controlled TR-909 takeover on next phrase".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_tr909_scene_lock(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.tr909_takeover_change_pending() {
            return QueueControlResult::AlreadyPending;
        }

        if self.session.runtime_state.lane_state.tr909.takeover_enabled
            && self.session.runtime_state.lane_state.tr909.takeover_profile
                == Some(Tr909TakeoverProfileState::SceneLockTakeover)
        {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909SceneLock,
            Quantization::NextPhrase,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some("scene_lock".into()),
        };
        draft.explanation = Some("engage scene-lock TR-909 variation on next phrase".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_tr909_release(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.tr909_takeover_change_pending() {
            return QueueControlResult::AlreadyPending;
        }
        if !self.session.runtime_state.lane_state.tr909.takeover_enabled {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909Release,
            Quantization::NextPhrase,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.0,
            target_id: Some("release".into()),
        };
        draft.explanation = Some("release controlled TR-909 takeover on next phrase".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }
}
