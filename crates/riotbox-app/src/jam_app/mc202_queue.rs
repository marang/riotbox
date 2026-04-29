use super::*;

impl JamAppState {
    fn mc202_phrase_control_pending(&self) -> bool {
        self.queue.pending_actions().iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::Mc202SetRole
                    | ActionCommand::Mc202GenerateFollower
                    | ActionCommand::Mc202GenerateAnswer
                    | ActionCommand::Mc202GeneratePressure
                    | ActionCommand::Mc202GenerateInstigator
                    | ActionCommand::Mc202MutatePhrase
            )
        })
    }

    pub fn queue_mc202_role_toggle(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let next_role = match self.session.runtime_state.lane_state.mc202.role.as_deref() {
            Some("follower") => "leader",
            Some("leader") => "follower",
            Some(_) | None => "follower",
        };
        let target_touch = if next_role == "leader" { 0.85 } else { 0.65 };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202SetRole,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some(next_role.into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: target_touch,
            target_id: Some(next_role.into()),
        };
        draft.explanation = Some(format!("set MC-202 role to {next_role} on next phrase"));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_mutate_phrase(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }
        if self.session.runtime_state.lane_state.mc202.role.is_none() {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202MutatePhrase,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("mutated_drive".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.88,
            target_id: Some("mutated_drive".into()),
        };
        draft.explanation = Some("mutate MC-202 phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_generate_follower(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202GenerateFollower,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("follower".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.78,
            target_id: Some("follower".into()),
        };
        draft.explanation = Some("generate MC-202 follower phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_generate_answer(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202GenerateAnswer,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("answer".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.82,
            target_id: Some("answer".into()),
        };
        draft.explanation = Some("generate MC-202 answer phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_generate_pressure(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202GeneratePressure,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("pressure".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.84,
            target_id: Some("pressure".into()),
        };
        draft.explanation = Some("generate MC-202 pressure phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_generate_instigator(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202GenerateInstigator,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("instigator".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.90,
            target_id: Some("instigator".into()),
        };
        draft.explanation =
            Some("generate MC-202 instigator phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }
}
