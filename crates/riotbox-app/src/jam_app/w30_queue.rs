use super::*;

impl JamAppState {
    pub fn queue_capture_bar(&mut self, requested_at: TimestampMs) {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::CaptureBarGroup,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Capture { bars: Some(4) };
        draft.explanation = Some("capture next phrase into W-30 path".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
    }

    pub fn queue_promote_last_capture(&mut self, requested_at: TimestampMs) -> bool {
        let Some(capture) = self
            .session
            .captures
            .iter()
            .rev()
            .find(|capture| capture.assigned_target.is_none())
            .or_else(|| self.session.captures.last())
        else {
            return false;
        };

        let Some(bank_id) = self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .clone()
        else {
            return false;
        };
        let Some(pad_id) = self
            .session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .clone()
        else {
            return false;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::PromoteCaptureToPad,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Promotion {
            capture_id: Some(capture.capture_id.clone()),
            destination: Some(format!("w30:{bank_id}/{pad_id}")),
        };
        draft.explanation = Some(format!(
            "promote {} into W-30 pad {bank_id}/{pad_id}",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        true
    }

    pub fn queue_w30_live_recall(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.recallable_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30LiveRecall,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "recall {} on W-30 pad {bank_id}/{pad_id}",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_step_focus(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let (bank_id, pad_id) = self.next_w30_focus_target()?;
        let current_focus = self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .clone()
            .zip(
                self.session
                    .runtime_state
                    .lane_state
                    .w30
                    .focused_pad
                    .clone(),
            );
        if current_focus == Some((bank_id.clone(), pad_id.clone())) {
            return Some(QueueControlResult::AlreadyInState);
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30StepFocus,
            Quantization::NextBeat,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.explanation = Some(format!(
            "step W-30 focus to {bank_id}/{pad_id} on next beat"
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_swap_bank(&mut self, requested_at: TimestampMs) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let (bank_id, pad_id, capture_id) = self.next_w30_bank_target()?;
        let current_focus = self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .clone()
            .zip(
                self.session
                    .runtime_state
                    .lane_state
                    .w30
                    .focused_pad
                    .clone(),
            );
        if current_focus == Some((bank_id.clone(), pad_id.clone())) {
            return Some(QueueControlResult::AlreadyInState);
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30SwapBank,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some(capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "swap W-30 bank to {bank_id}/{pad_id} with {capture_id} on next bar"
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_browse_slice_pool(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let target = self.next_w30_slice_pool_capture()?;
        if self
            .session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            == Some(&target.capture_id)
        {
            return Some(QueueControlResult::AlreadyInState);
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30BrowseSlicePool,
            Quantization::NextBeat,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(target.bank_id.clone()),
                pad_id: Some(target.pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some(target.capture_id.to_string()),
        };
        let reason = match target.selection_reason {
            w30_targets::W30SlicePoolSelectionReason::Cycle => "slice pool",
            w30_targets::W30SlicePoolSelectionReason::FeralScorecard => "feral slice pool",
        };
        draft.explanation = Some(format!(
            "browse W-30 {reason} to {} on {}/{} on next beat",
            target.capture_id, target.bank_id, target.pad_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_apply_damage_profile(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.damage_profile_ready_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30ApplyDamageProfile,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: Self::W30_DAMAGE_PROFILE_GRIT,
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "apply {} damage profile to {} on W-30 pad {bank_id}/{pad_id}",
            Self::W30_DAMAGE_PROFILE_LABEL,
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_loop_freeze(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_phrase_capture_cue_pending() || self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.loop_freeze_ready_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30LoopFreeze,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Promotion {
            capture_id: Some(capture.capture_id.clone()),
            destination: Some("w30:loop_freeze".into()),
        };
        draft.explanation = Some(format!(
            "{} {} for W-30 reuse on {bank_id}/{pad_id} on next phrase",
            Self::W30_LOOP_FREEZE_LABEL,
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_promoted_audition(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.auditionable_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30AuditionPromoted,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.68,
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "audition promoted {} on W-30 pad {bank_id}/{pad_id}",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_audition(&mut self, requested_at: TimestampMs) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        if self.auditionable_w30_capture().is_some() {
            return self.queue_w30_promoted_audition(requested_at);
        }

        self.queue_w30_raw_capture_audition(requested_at)
    }

    pub fn queue_w30_raw_capture_audition(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.raw_auditionable_capture()?.clone();
        let w30 = &self.session.runtime_state.lane_state.w30;
        let bank_id = w30.active_bank.clone()?;
        let pad_id = w30.focused_pad.clone()?;

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30AuditionRawCapture,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.58,
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "audition raw capture {} on W-30 preview {bank_id}/{pad_id}",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_trigger_pad(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.triggerable_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30TriggerPad,
            Quantization::NextBeat,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: if capture.is_pinned { 0.72 } else { 0.84 },
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "trigger W-30 pad {bank_id}/{pad_id} from {} on next beat",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_internal_resample(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_phrase_capture_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.resample_ready_w30_capture()?.clone();
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::PromoteResample,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Promotion {
            capture_id: Some(capture.capture_id.clone()),
            destination: Some("w30:resample".into()),
        };
        draft.explanation = Some(format!(
            "resample {} through W-30 on next phrase",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }
}
