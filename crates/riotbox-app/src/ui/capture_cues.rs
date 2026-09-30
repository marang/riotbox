use crate::ui::shell_state::JamShellState;

pub(in crate::ui) fn capture_or_recall_cue_label(shell: &JamShellState) -> String {
    shell
        .app
        .jam_view
        .pending_actions
        .iter()
        .find(|action| {
            matches!(
                action.command.as_str(),
                "w30.trigger_pad"
                    | "w30.step_focus"
                    | "w30.swap_bank"
                    | "w30.apply_damage_profile"
                    | "w30.hook_turnaround"
                    | "w30.pitch_dive"
                    | "w30.filter_slam"
                    | "w30.loop_freeze"
                    | "w30.live_recall"
                    | "w30.audition_raw_capture"
                    | "w30.audition_promoted"
                    | "promote.resample"
            )
        })
        .or_else(|| {
            shell
                .app
                .jam_view
                .pending_actions
                .iter()
                .find(|action| is_capture_command_view(action.command.as_str()))
        })
        .map(|action| format!("{} @ {}", action.command, action.quantization))
        .unwrap_or_else(|| "no capture cue queued".into())
}

pub(in crate::ui) fn is_capture_command_view(command: &str) -> bool {
    matches!(
        command,
        "capture.now"
            | "capture.loop"
            | "capture.bar_group"
            | "w30.capture_to_pad"
            | "promote.capture_to_pad"
            | "promote.capture_to_scene"
            | "w30.loop_freeze"
            | "promote.resample"
    )
}
