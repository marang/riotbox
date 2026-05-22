use riotbox_core::session::{CaptureRef, CaptureTarget};

use super::{JamShellState, current_w30_lane_target};

fn current_w30_slice_pool(shell: &JamShellState) -> Vec<&CaptureRef> {
    let Some((active_bank, focused_pad)) = current_w30_lane_target(shell) else {
        return Vec::new();
    };

    shell
        .app
        .session
        .captures
        .iter()
        .filter(|capture| {
            matches!(
                capture.assigned_target.as_ref(),
                Some(CaptureTarget::W30Pad { bank_id, pad_id })
                    if bank_id.as_str() == active_bank && pad_id.as_str() == focused_pad
            )
        })
        .collect()
}

fn current_w30_slice_pool_position(shell: &JamShellState, pool: &[&CaptureRef]) -> Option<usize> {
    let last_capture = shell
        .app
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()?;
    pool.iter()
        .position(|capture| &capture.capture_id == last_capture)
}

pub(super) fn w30_slice_pool_relevant(shell: &JamShellState) -> bool {
    shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_capture_id
        .is_some()
        || current_w30_slice_pool(shell).len() > 1
}

pub(super) fn w30_slice_pool_compact(shell: &JamShellState) -> String {
    let pool = current_w30_slice_pool(shell);
    if pool.is_empty() {
        return "none".into();
    }

    let current_index =
        current_w30_slice_pool_position(shell, &pool).unwrap_or_else(|| pool.len() - 1);
    let current_capture = pool[current_index].capture_id.to_string();
    let next_capture = shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_capture_id
        .clone()
        .or_else(|| {
            (pool.len() > 1).then(|| {
                pool[(current_index + 1) % pool.len()]
                    .capture_id
                    .to_string()
            })
        })
        .unwrap_or_else(|| "hold".into());
    let next_capture = w30_slice_pool_next_label(shell, &next_capture);

    format!(
        "{current_capture} {}/{} -> {next_capture}",
        current_index + 1,
        pool.len()
    )
}

pub(super) fn w30_slice_pool_log_compact(shell: &JamShellState) -> String {
    let pool = current_w30_slice_pool(shell);
    if pool.is_empty() {
        return "none".into();
    }

    let current_index =
        current_w30_slice_pool_position(shell, &pool).unwrap_or_else(|| pool.len() - 1);
    let next_capture = shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_capture_id
        .clone()
        .or_else(|| {
            (pool.len() > 1).then(|| {
                pool[(current_index + 1) % pool.len()]
                    .capture_id
                    .to_string()
            })
        })
        .unwrap_or_else(|| "hold".into());
    let next_capture = w30_slice_pool_next_label(shell, &next_capture);

    format!("{}/{} -> {next_capture}", current_index + 1, pool.len())
}

fn w30_slice_pool_next_label(shell: &JamShellState, capture_id: &str) -> String {
    if shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_reason
        .as_deref()
        == Some("feral")
        && capture_id != "hold"
    {
        format!("feral {capture_id}")
    } else {
        capture_id.into()
    }
}
