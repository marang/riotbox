mod capture;
mod ghost;
mod mc202;
mod preset;
mod scene;
mod source_monitor;
mod source_timing;
mod tr909;
mod transport;
mod w30;

use riotbox_core::{
    action::{Action, ActionCommand},
    session::SessionFile,
    source_graph::SourceGraph,
    transport::CommitBoundaryState,
};

use capture::apply_capture_side_effects;
use ghost::apply_ghost_side_effects;
pub(super) use mc202::apply_mc202_side_effects;
use preset::apply_preset_side_effects;
use scene::apply_scene_side_effects;
use source_monitor::apply_source_monitor_side_effects;
use source_timing::apply_source_timing_side_effects;
use tr909::apply_tr909_side_effects;
use transport::apply_transport_side_effects;
use w30::apply_w30_side_effects;

/// Own the post-materialization lane/control effects of each existing command.
/// No wildcard: adding an ActionCommand requires an explicit ownership choice.
pub(super) fn apply_committed_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: &CommitBoundaryState,
    source_graph: Option<&SourceGraph>,
) {
    match action.command {
        ActionCommand::W30LiveRecall
        | ActionCommand::W30SwapBank
        | ActionCommand::W30BrowseSlicePool
        | ActionCommand::W30ApplyDamageProfile
        | ActionCommand::W30HookTurnaround
        | ActionCommand::W30PitchDive
        | ActionCommand::W30FilterSlam
        | ActionCommand::W30LoopFreeze
        | ActionCommand::W30StepFocus
        | ActionCommand::W30AuditionRawCapture
        | ActionCommand::W30AuditionPromoted
        | ActionCommand::W30TriggerPad => apply_w30_side_effects(session, action, Some(boundary)),
        ActionCommand::Mc202SetRole
        | ActionCommand::Mc202GenerateFollower
        | ActionCommand::Mc202GenerateAnswer
        | ActionCommand::Mc202GeneratePressure
        | ActionCommand::Mc202GenerateInstigator
        | ActionCommand::Mc202MutatePhrase => {
            apply_mc202_side_effects(session, action, Some(boundary), source_graph);
        }
        ActionCommand::Tr909SetSlam
        | ActionCommand::Tr909FillNext
        | ActionCommand::Tr909ReinforceBreak
        | ActionCommand::Tr909Takeover
        | ActionCommand::Tr909SceneLock
        | ActionCommand::Tr909Release => apply_tr909_side_effects(session, action, Some(boundary)),
        ActionCommand::TransportPlay
        | ActionCommand::TransportPause
        | ActionCommand::TransportStop
        | ActionCommand::TransportSeek => {
            apply_transport_side_effects(session, action);
        }
        ActionCommand::CaptureSetLength => {
            apply_capture_side_effects(session, action);
        }
        ActionCommand::PresetActivate => {
            apply_preset_side_effects(session, action);
        }
        ActionCommand::SourceMonitorSetMode => {
            apply_source_monitor_side_effects(session, action);
        }
        ActionCommand::SourceTimingConfirmGrid | ActionCommand::SourceTimingRevertGrid => {
            apply_source_timing_side_effects(session, action);
        }
        ActionCommand::MutateScene | ActionCommand::SceneLaunch | ActionCommand::SceneRestore => {
            apply_scene_side_effects(session, action, Some(boundary), source_graph);
        }
        ActionCommand::GhostSetMode
        | ActionCommand::GhostAcceptSuggestion
        | ActionCommand::GhostRejectSuggestion => {
            apply_ghost_side_effects(session, action);
        }
        // Capture materialization/promotion runs earlier in the commit pipeline.
        // LoopFreeze also requires its W-30 effect and is deliberately above.
        ActionCommand::CaptureNow
        | ActionCommand::CaptureLoop
        | ActionCommand::CaptureBarGroup
        | ActionCommand::W30CaptureToPad
        | ActionCommand::PromoteCaptureToPad
        | ActionCommand::PromoteCaptureToScene
        | ActionCommand::PromoteResample => {}
        // Exports own their external side-effect transaction, not a lane effect.
        ActionCommand::ExportProductMix
        | ActionCommand::ExportStemPackage
        | ActionCommand::ExportLiveRecording
        | ActionCommand::ExportDawSession => {}
        // Structural/other commands have no effect in these lane/control owners.
        // This does not grant currently unsupported commands replay coverage.
        ActionCommand::MutateLane
        | ActionCommand::MutateLoop
        | ActionCommand::MutatePattern
        | ActionCommand::MutateHook
        | ActionCommand::SceneRegenerate
        | ActionCommand::SceneReinterpret
        | ActionCommand::LockObject
        | ActionCommand::UnlockObject
        | ActionCommand::SnapshotSave
        | ActionCommand::SnapshotLoad
        | ActionCommand::UndoLast
        | ActionCommand::RedoLast
        | ActionCommand::RestoreSource
        | ActionCommand::GhostExecuteTool => {}
    }
}

#[cfg(test)]
mod tests;
