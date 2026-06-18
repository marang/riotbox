use riotbox_audio::w30::{W30PreviewRenderMode, W30PreviewRenderState, W30PreviewSourceProfile};

use super::JamShellState;

pub(super) fn w30_preview_mode_profile_compact(shell: &JamShellState) -> String {
    let render = &shell.app.runtime.w30_preview;
    let mode = match render.mode {
        W30PreviewRenderMode::Idle => "idle",
        W30PreviewRenderMode::LiveRecall => "recall",
        W30PreviewRenderMode::RawCaptureAudition => "audition raw",
        W30PreviewRenderMode::PromotedAudition => "audition",
    };
    let profile = match render.source_profile {
        None => "unset",
        Some(W30PreviewSourceProfile::PinnedRecall) => "pinned",
        Some(W30PreviewSourceProfile::PromotedRecall) => "promoted",
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => "browse",
        Some(W30PreviewSourceProfile::RawCaptureAudition) => "raw",
        Some(W30PreviewSourceProfile::PromotedAudition) => "audition",
    };

    if matches!(render.mode, W30PreviewRenderMode::RawCaptureAudition) {
        return format!(
            "{mode}/{}",
            w30_preview_source_suffix(render).unwrap_or("unavailable")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::PromotedAudition) {
        return format!(
            "{mode}/{}",
            w30_preview_source_suffix(render).unwrap_or("unavailable")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::LiveRecall) {
        return format!(
            "{mode}/{profile}/{}",
            w30_preview_source_suffix(render).unwrap_or("unavailable")
        );
    }

    format!("{mode}/{profile}")
}

pub(super) fn w30_preview_log_compact(shell: &JamShellState) -> String {
    let render = &shell.app.runtime.w30_preview;
    if matches!(render.mode, W30PreviewRenderMode::RawCaptureAudition) {
        return format!(
            "raw/{}",
            w30_preview_source_suffix(render).unwrap_or("unavailable")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::PromotedAudition) {
        return format!(
            "audition/{}",
            w30_preview_source_suffix(render).unwrap_or("unavailable")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::LiveRecall) {
        return format!(
            "recall/{}",
            w30_preview_source_suffix(render).unwrap_or("unavailable")
        );
    }

    w30_preview_mode_profile_compact(shell)
}

fn w30_preview_source_suffix(render: &W30PreviewRenderState) -> Option<&'static str> {
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return None;
    }

    if render.source_window_preview.is_some() {
        Some("src")
    } else if render.pad_playback.is_some() {
        Some("artifact")
    } else {
        Some("unavailable")
    }
}

pub(super) fn w30_preview_source_readiness(shell: &JamShellState) -> Option<&'static str> {
    let render = &shell.app.runtime.w30_preview;
    if matches!(render.mode, W30PreviewRenderMode::RawCaptureAudition) {
        return match w30_preview_source_suffix(render)? {
            "src" => Some("source-backed"),
            "artifact" => Some("artifact-backed"),
            "unavailable" => Some("unavailable"),
            _ => None,
        };
    }

    if render.source_window_preview.is_some() {
        Some("source-backed")
    } else if render.pad_playback.is_some() {
        Some("artifact-backed")
    } else if !matches!(render.mode, W30PreviewRenderMode::Idle) {
        Some("unavailable")
    } else {
        None
    }
}
