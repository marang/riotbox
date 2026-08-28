use std::{fs, path::Path};

use riotbox_app::{
    jam_app::JamAppState,
    observer::observer_snapshot,
    ui::{JamShellState, ShellLaunchMode},
};
use riotbox_audio::{
    runtime::{
        RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
    },
    source_audio::write_interleaved_pcm16_wav,
    w30::W30PreviewRenderState,
};
use riotbox_core::{
    action::{ActionCommand, ActionStatus},
    session::ExportArtifactRole,
};
use serde_json::json;

use super::super::{CHANNEL_COUNT, SAMPLE_RATE, isolated_w30_plan};

const START_BEAT: f64 = 8.0;
const DURATION_BEATS: f64 = 8.0;
const PRIMARY_CALLBACK_FRAMES: usize = 128;
const PARITY_CALLBACK_FRAMES: usize = 257;

const EXACT_OWNER_ACTIONS: [ActionCommand; 6] = [
    ActionCommand::SourceTimingConfirmGrid,
    ActionCommand::PresetActivate,
    ActionCommand::CaptureSetLength,
    ActionCommand::CaptureBarGroup,
    ActionCommand::PromoteCaptureToPad,
    ActionCommand::W30TriggerPad,
];

pub(crate) fn qualify_semantic_hook_product_v4(
    mut state: JamAppState,
    ordinary_render: &W30PreviewRenderState,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let committed_actions = state
        .session
        .action_log
        .actions
        .iter()
        .filter(|action| action.status == ActionStatus::Committed)
        .map(|action| (action.command, action.id))
        .collect::<Vec<_>>();
    let actual_commands = committed_actions
        .iter()
        .map(|(command, _)| *command)
        .collect::<Vec<_>>();
    if actual_commands != EXACT_OWNER_ACTIONS {
        return Err(format!(
            "semantic-hook V4 requires the exact six-action W-30 owner; got {actual_commands:?}"
        )
        .into());
    }

    let committed_bpm = state
        .session
        .runtime_state
        .source_timing
        .confirmed_bpm
        .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
        .ok_or("semantic-hook V4 requires a positive committed BPM")?;
    if (ordinary_render.tempo_bpm - committed_bpm).abs() > f32::EPSILON {
        return Err("semantic-hook V4 ordinary render tempo differs from Session truth".into());
    }
    let frame_count = (DURATION_BEATS * 60.0 * f64::from(SAMPLE_RATE) / f64::from(committed_bpm))
        .round() as usize;
    let owner_plan = isolated_w30_plan(ordinary_render.clone(), committed_bpm, START_BEAT);
    let owner_primary = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(&owner_plan, frame_count)],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        PRIMARY_CALLBACK_FRAMES,
    )
    .pop()
    .ok_or("semantic-hook V4 exact-owner render produced no output")?;
    let owner_parity = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(&owner_plan, frame_count)],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        PARITY_CALLBACK_FRAMES,
    )
    .pop()
    .ok_or("semantic-hook V4 exact-owner parity render produced no output")?;
    if owner_primary.samples != owner_parity.samples {
        return Err(
            "semantic-hook V4 exact-owner render changed across callback partitions".into(),
        );
    }
    for report in [&owner_primary.limiter, &owner_parity.limiter] {
        if report.applied
            || report.pre.clip_count != 0
            || report.limited_sample_count != 0
            || report.post.clip_count != 0
            || report.post.active_samples == 0
        {
            return Err(
                "semantic-hook V4 exact-owner render is silent, clipped, or limiter-dependent"
                    .into(),
            );
        }
    }

    let owner_control_path = output_dir.join("09_w30_semantic_hook_owner_control_v4.wav");
    write_interleaved_pcm16_wav(
        &owner_control_path,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &owner_primary.samples,
    )?;

    let destination = output_dir.join("w30-hook-product-export");
    let receipt = state.commit_stem_package_export_w30_hook_loop(&destination, 90_000)?;
    let stem = receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == ExportArtifactRole::W30HookLoop)
        .ok_or("semantic-hook V4 receipt omitted w30_hook_loop")?;
    let written_path = Path::new(stem.location_identity());
    if fs::read(&owner_control_path)? != fs::read(written_path)? {
        return Err(
            "semantic-hook V4 product WAV differs from the independent exact-owner render".into(),
        );
    }

    let semantic_stem = json!({
        "role": stem.role,
        "location": stem.location,
        "sha256": stem.sha256,
        "sample_rate_hz": stem.sample_rate_hz,
        "channel_count": stem.channel_count,
        "duration_ms": stem.duration_ms,
    });
    let source_sha256 = state
        .source_graph
        .as_ref()
        .map(|graph| graph.source.content_hash.clone());
    let owner_action_ids = committed_actions
        .iter()
        .map(|(_, action_id)| *action_id)
        .collect::<Vec<_>>();
    let owner_metrics = json!({
        "frame_count": frame_count,
        "start_beat": START_BEAT,
        "duration_beats": DURATION_BEATS,
        "active_samples": owner_primary.limiter.post.active_samples,
        "peak_abs": owner_primary.limiter.post.peak_abs,
        "rms": owner_primary.limiter.post.rms,
        "pre_limiter_clip_count": owner_primary.limiter.pre.clip_count,
        "limited_sample_count": owner_primary.limiter.limited_sample_count,
        "post_limiter_clip_count": owner_primary.limiter.post.clip_count,
        "callback_partition_128_vs_257_sample_exact": true,
        "written_product_wav_byte_exact": true,
    });

    state.save()?;
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    fs::write(
        output_dir.join("w30-hook-product-export-summary.json"),
        serde_json::to_vec_pretty(&json!({
            "schema": "riotbox.w30_hook_product_export_summary.v4",
            "status": "pass",
            "source_sha256": source_sha256,
            "product_bpm": committed_bpm,
            "committed_bpm": committed_bpm,
            "exact_owner": {
                "product_path": "ordinary_promoted_w30_control_v1",
                "binary": "w30_live_path_render",
                "committed_actions": EXACT_OWNER_ACTIONS,
                "committed_action_ids": owner_action_ids,
            },
            "owner_render": owner_metrics,
            "owner_control_path": owner_control_path,
            "receipt": receipt,
            "semantic_stem": semantic_stem,
            "observer_snapshot": observer_snapshot(&shell),
        }))?,
    )?;
    Ok(())
}
