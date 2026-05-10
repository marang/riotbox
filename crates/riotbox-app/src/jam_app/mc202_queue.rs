use super::{
    helpers::user_lane_mutation_draft,
    state::{JamAppState, QueueControlResult},
};
use riotbox_core::{
    TimestampMs,
    action::{ActionCommand, Quantization, TargetScope},
};

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

        let draft = user_lane_mutation_draft(
            ActionCommand::Mc202SetRole,
            Quantization::NextPhrase,
            TargetScope::LaneMc202,
            next_role,
            target_touch,
            format!("set MC-202 role to {next_role} on next phrase"),
        );
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

        let draft = user_lane_mutation_draft(
            ActionCommand::Mc202MutatePhrase,
            Quantization::NextPhrase,
            TargetScope::LaneMc202,
            "mutated_drive",
            0.88,
            "mutate MC-202 phrase on next phrase boundary",
        );
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

        let draft = user_lane_mutation_draft(
            ActionCommand::Mc202GenerateFollower,
            Quantization::NextPhrase,
            TargetScope::LaneMc202,
            "follower",
            0.78,
            "generate MC-202 follower phrase on next phrase boundary",
        );
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_generate_answer(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let draft = user_lane_mutation_draft(
            ActionCommand::Mc202GenerateAnswer,
            Quantization::NextPhrase,
            TargetScope::LaneMc202,
            "answer",
            0.82,
            "generate MC-202 answer phrase on next phrase boundary",
        );
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

        let draft = user_lane_mutation_draft(
            ActionCommand::Mc202GeneratePressure,
            Quantization::NextPhrase,
            TargetScope::LaneMc202,
            "pressure",
            0.84,
            "generate MC-202 pressure phrase on next phrase boundary",
        );
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

        let draft = user_lane_mutation_draft(
            ActionCommand::Mc202GenerateInstigator,
            Quantization::NextPhrase,
            TargetScope::LaneMc202,
            "instigator",
            0.90,
            "generate MC-202 instigator phrase on next phrase boundary",
        );
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }
}
