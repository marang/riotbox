//! Input correlation only; the real queue and Core records own action status.
use std::{fmt, io};

use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use riotbox_core::{
    TimestampMs,
    action::{ActionCommand, ActionStatus},
    ids::ActionId,
    queue::ActionIdExhausted,
};
use serde::Serialize;
use serde_json::Value;

use super::{AppLaunch, Observation, RuntimeControl, execute_policy, stop_live_master_runtime};
use crate::{
    cli::live_master_recording::terminal_input::RecordingInput, cli::model::LiveRecordingInputMode,
    jam_app::JamAppState,
};

pub(super) fn execute(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    observation: &mut Observation<'_>,
    input: Option<&mut dyn RecordingInput>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let enabled = input.is_some();
    if enabled != (launch.mode.live_recording_input_mode() == LiveRecordingInputMode::TerminalFill)
    {
        stop_live_master_runtime(state, runtime, false)?;
        return Err("live recording input adapter does not match the explicit launch mode".into());
    }
    let mut control = InputControl::default();
    let execution = execute_policy(launch, state, runtime, observation, input, &mut control);
    // Always retire pending ownership, including failure returns. Queue rejects
    // cannot mutate a committed action or the writer's persisted Session truth.
    let report = control.finish(state);
    execution.and_then(|mut summary| {
        if enabled {
            summary["terminal_input"] = serde_json::to_value(report)?;
        }
        Ok(summary)
    })
}

#[derive(Default)]
pub(super) struct InputControl {
    owned_ids: Vec<ActionId>,
    ignored_event_count: u64,
}

pub(super) enum InputFailure {
    Cancelled,
    Read(io::Error),
    ActionId(ActionIdExhausted),
}

impl fmt::Display for InputFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("cancelled while capture was pending"),
            Self::Read(error) => write!(formatter, "event read failed: {error}"),
            Self::ActionId(error) => write!(formatter, "Fill could not be queued: {error}"),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum RenderDelivery {
    NotQualified,
}

#[derive(Serialize)]
pub(super) struct InputReport {
    queued_action_ids: Vec<ActionId>,
    core_committed_action_ids: Vec<ActionId>,
    not_committed_action_ids: Vec<ActionId>,
    ignored_event_count: u64,
    render_delivery: RenderDelivery,
}

impl InputControl {
    pub(super) fn pending(
        &mut self,
        state: &mut JamAppState,
        input: &mut dyn RecordingInput,
        requested_at: TimestampMs,
    ) -> Result<(), InputFailure> {
        let Some(key) = input.poll_event().map_err(InputFailure::Read)? else {
            return Ok(());
        };
        if key.kind == KeyEventKind::Press {
            if (key.modifiers == KeyModifiers::NONE
                && matches!(key.code, KeyCode::Esc | KeyCode::Char('q')))
                || (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c'))
            {
                return Err(InputFailure::Cancelled);
            }
            if key.modifiers == KeyModifiers::NONE
                && key.code == KeyCode::Char('f')
                && !state
                    .queue
                    .has_pending_command(ActionCommand::Tr909FillNext)
                && !state
                    .session
                    .runtime_state
                    .lane_state
                    .tr909
                    .takeover_enabled
            {
                let id = state
                    .queue_tr909_fill(requested_at)
                    .map_err(InputFailure::ActionId)?;
                self.owned_ids.push(id);
                return Ok(());
            }
        }
        self.ignored_event_count = self.ignored_event_count.saturating_add(1);
        Ok(())
    }

    pub(super) fn finish(self, state: &mut JamAppState) -> InputReport {
        let mut committed = Vec::new();
        let mut not_committed = Vec::new();
        for &id in &self.owned_ids {
            if state
                .session
                .action_log
                .actions
                .iter()
                .any(|action| action.id == id && action.status == ActionStatus::Committed)
            {
                committed.push(id);
            } else {
                // reject changes pending only. Never clear foreign actions or
                // rewrite an action already committed by the final sync.
                state
                    .queue
                    .reject(id, "recording ended before this input-owned Fill committed");
                not_committed.push(id);
            }
        }
        if !not_committed.is_empty() {
            state.refresh_view();
        }
        InputReport {
            queued_action_ids: self.owned_ids,
            core_committed_action_ids: committed,
            not_committed_action_ids: not_committed,
            ignored_event_count: self.ignored_event_count,
            render_delivery: RenderDelivery::NotQualified,
        }
    }
}
