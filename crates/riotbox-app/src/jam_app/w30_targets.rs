use std::collections::BTreeSet;

use riotbox_core::{
    action::{ActionCommand, TargetScope},
    ids::{BankId, CaptureId, PadId},
    session::{CaptureRef, CaptureTarget},
};

use super::{JamAppState, capture_targets_specific_w30_pad, capture_targets_w30_pad};

impl JamAppState {
    pub(super) fn focused_w30_capture(&self) -> Option<&CaptureRef> {
        let w30 = &self.session.runtime_state.lane_state.w30;
        let (bank_id, pad_id) = w30.active_bank.as_ref().zip(w30.focused_pad.as_ref())?;

        self.session
            .captures
            .iter()
            .rev()
            .find(|capture| capture_targets_specific_w30_pad(capture, bank_id, pad_id))
    }

    pub(super) fn w30_focus_targets(&self) -> Vec<(BankId, PadId)> {
        self.session
            .captures
            .iter()
            .filter_map(|capture| match capture.assigned_target.as_ref() {
                Some(CaptureTarget::W30Pad { bank_id, pad_id }) => {
                    Some((bank_id.clone(), pad_id.clone()))
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(super) fn next_w30_focus_target(&self) -> Option<(BankId, PadId)> {
        let targets = self.w30_focus_targets();
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

        if targets.is_empty() {
            return None;
        }

        if let Some(current_focus) = current_focus
            && let Some(index) = targets.iter().position(|target| *target == current_focus)
        {
            return Some(targets[(index + 1) % targets.len()].clone());
        }

        targets.first().cloned()
    }

    pub(super) fn next_w30_bank_target(&self) -> Option<(BankId, PadId, CaptureId)> {
        let current_bank = self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .clone();
        let current_pad = self
            .session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .clone();

        let targets = self.w30_focus_targets();

        if targets.is_empty() {
            return None;
        }

        let banks = targets
            .iter()
            .map(|(bank_id, _)| bank_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let target_bank = if let Some(current_bank) = current_bank {
            if let Some(index) = banks.iter().position(|bank_id| *bank_id == current_bank) {
                banks[(index + 1) % banks.len()].clone()
            } else {
                banks.first().cloned()?
            }
        } else {
            banks.first().cloned()?
        };

        let target_pad = if let Some(current_pad) = current_pad.as_ref()
            && targets
                .iter()
                .any(|(bank_id, pad_id)| *bank_id == target_bank && pad_id == current_pad)
        {
            current_pad.clone()
        } else {
            targets
                .iter()
                .find(|(bank_id, _)| *bank_id == target_bank)
                .map(|(_, pad_id)| pad_id.clone())?
        };

        let capture_id = self
            .session
            .captures
            .iter()
            .rev()
            .find(|capture| capture_targets_specific_w30_pad(capture, &target_bank, &target_pad))
            .map(|capture| capture.capture_id.clone())?;

        Some((target_bank, target_pad, capture_id))
    }

    pub(super) fn recallable_w30_capture(&self) -> Option<&CaptureRef> {
        if self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .is_some()
            && self
                .session
                .runtime_state
                .lane_state
                .w30
                .focused_pad
                .is_some()
        {
            return self.focused_w30_capture();
        }

        self.session
            .captures
            .iter()
            .rev()
            .find(|capture| capture.is_pinned && capture_targets_w30_pad(capture))
            .or_else(|| {
                self.session
                    .captures
                    .iter()
                    .rev()
                    .find(|capture| capture_targets_w30_pad(capture))
            })
    }

    pub(super) fn auditionable_w30_capture(&self) -> Option<&CaptureRef> {
        if self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .is_some()
            && self
                .session
                .runtime_state
                .lane_state
                .w30
                .focused_pad
                .is_some()
        {
            return self.focused_w30_capture();
        }

        self.session
            .captures
            .iter()
            .rev()
            .find(|capture| capture_targets_w30_pad(capture))
    }

    pub(super) fn raw_auditionable_capture(&self) -> Option<&CaptureRef> {
        self.session
            .captures
            .iter()
            .rev()
            .find(|capture| capture.assigned_target.is_none())
    }

    pub(super) fn triggerable_w30_capture(&self) -> Option<&CaptureRef> {
        if self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .is_some()
            && self
                .session
                .runtime_state
                .lane_state
                .w30
                .focused_pad
                .is_some()
        {
            return self.focused_w30_capture();
        }

        self.session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .and_then(|capture_id| {
                self.session
                    .captures
                    .iter()
                    .find(|capture| capture.capture_id == *capture_id)
            })
            .or_else(|| self.recallable_w30_capture())
    }

    pub(super) fn damage_profile_ready_w30_capture(&self) -> Option<&CaptureRef> {
        self.triggerable_w30_capture()
    }

    pub(super) fn loop_freeze_ready_w30_capture(&self) -> Option<&CaptureRef> {
        self.triggerable_w30_capture()
    }

    pub(super) fn resample_ready_w30_capture(&self) -> Option<&CaptureRef> {
        if self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .is_some()
            && self
                .session
                .runtime_state
                .lane_state
                .w30
                .focused_pad
                .is_some()
        {
            return self.focused_w30_capture();
        }

        self.session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .and_then(|capture_id| {
                self.session
                    .captures
                    .iter()
                    .find(|capture| capture.capture_id == *capture_id)
            })
    }

    pub(super) fn current_w30_lane_target(&self) -> Option<(BankId, PadId)> {
        self.session
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
            )
    }

    pub(super) fn next_w30_slice_pool_capture(&self) -> Option<(BankId, PadId, CaptureId)> {
        let (bank_id, pad_id) = self.current_w30_lane_target()?;
        let pool: Vec<&CaptureRef> = self
            .session
            .captures
            .iter()
            .filter(|capture| capture_targets_specific_w30_pad(capture, &bank_id, &pad_id))
            .collect();
        if pool.is_empty() {
            return None;
        }

        let next_capture = self
            .session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .and_then(|last_capture_id| {
                pool.iter()
                    .position(|capture| capture.capture_id == *last_capture_id)
                    .map(|index| pool[(index + 1) % pool.len()])
            })
            .unwrap_or_else(|| {
                pool.last()
                    .copied()
                    .expect("non-empty slice pool should have a last capture")
            });

        Some((bank_id, pad_id, next_capture.capture_id.clone()))
    }

    pub(super) fn w30_pad_cue_pending(&self) -> bool {
        self.queue.pending_actions().into_iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::W30SwapBank
                    | ActionCommand::W30BrowseSlicePool
                    | ActionCommand::W30ApplyDamageProfile
                    | ActionCommand::W30LoopFreeze
                    | ActionCommand::W30LiveRecall
                    | ActionCommand::W30StepFocus
                    | ActionCommand::W30AuditionRawCapture
                    | ActionCommand::W30AuditionPromoted
                    | ActionCommand::W30TriggerPad
            )
        })
    }

    pub(super) fn w30_phrase_capture_cue_pending(&self) -> bool {
        self.queue.pending_actions().into_iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::W30LoopFreeze | ActionCommand::PromoteResample
            ) && action.target.scope == Some(TargetScope::LaneW30)
        })
    }
}
