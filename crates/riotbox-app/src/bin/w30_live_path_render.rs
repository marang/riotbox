use std::{env, fs, path::PathBuf};

use riotbox_app::jam_app::JamAppState;
use riotbox_audio::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan,
        render_runtime_mix_realtime_simulation_offline, signal_delta_metrics, signal_metrics,
    },
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
};
use riotbox_core::{
    action::{CaptureLengthIntent, CommitBoundary, SourceMonitorMode},
    transport::CommitBoundaryState,
};

const SAMPLE_RATE: u32 = 48_000;
const CHANNEL_COUNT: u16 = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let source_path = required_path(&args, "--source")?;
    let output_dir = required_path(&args, "--output")?;
    let bpm = required_value(&args, "--bpm")?.parse::<f32>()?;
    fs::create_dir_all(&output_dir)?;

    let session_path = output_dir.join("session.json");
    let graph_path = output_dir.join("source-graph.json");
    let mut state = JamAppState::analyze_source_file_to_json_with_source_bpm_confirmation(
        &source_path,
        &session_path,
        Some(graph_path),
        "python/sidecar/json_stdio_sidecar.py",
        19,
        Some(bpm),
    )?;
    state.set_transport_playing(true);
    let scene_id = state.runtime.transport.current_scene.clone();
    state.queue_source_monitor_mode(SourceMonitorMode::Riotbox, 90);
    commit(
        &mut state,
        CommitBoundary::Immediate,
        0,
        1,
        1,
        scene_id.clone(),
        95,
    )?;
    state.queue_capture_length_intent(CaptureLengthIntent::OneBar, 96);
    commit(
        &mut state,
        CommitBoundary::Immediate,
        0,
        1,
        1,
        scene_id.clone(),
        97,
    )?;

    state.queue_capture_bar(100);
    commit(
        &mut state,
        CommitBoundary::Bar,
        5,
        2,
        1,
        scene_id.clone(),
        200,
    )?;
    if !state.queue_promote_last_capture(210) {
        return Err("capture promotion was unavailable".into());
    }
    commit(
        &mut state,
        CommitBoundary::Bar,
        5,
        2,
        1,
        scene_id.clone(),
        300,
    )?;
    if state.queue_w30_trigger_pad(310).is_none() {
        return Err("W-30 trigger was unavailable".into());
    }
    commit(
        &mut state,
        CommitBoundary::Beat,
        6,
        2,
        1,
        scene_id.clone(),
        400,
    )?;

    print_w30_render_summary("normal", &state.runtime.w30_preview);
    let normal = render_state(&state, bpm);
    if state.queue_w30_apply_damage_profile(410).is_none() {
        return Err("W-30 damage gesture was unavailable".into());
    }
    commit(&mut state, CommitBoundary::Bar, 9, 3, 1, scene_id, 500)?;
    print_w30_render_summary("damaged", &state.runtime.w30_preview);
    let damaged = render_state(&state, bpm);

    write_interleaved_pcm16_wav(
        output_dir.join("01_w30_live_hook.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &normal,
    )?;
    write_interleaved_pcm16_wav(
        output_dir.join("02_w30_live_hook_pitch_damage.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &damaged,
    )?;
    let source = SourceAudioCache::load_pcm_wav(&source_path)?;
    write_interleaved_pcm16_wav(
        output_dir.join("00_source.wav"),
        source.sample_rate,
        source.channel_count,
        source.interleaved_samples(),
    )?;
    state.save()?;

    let normal_metrics = signal_metrics(&normal);
    let damaged_metrics = signal_metrics(&damaged);
    let delta = signal_delta_metrics(&normal, &damaged);
    println!("normal: {normal_metrics:?}");
    println!("damaged: {damaged_metrics:?}");
    println!("gesture delta: {delta:?}");
    if normal_metrics.rms <= 0.001 || damaged_metrics.rms <= 0.001 || delta.rms <= 0.001 {
        return Err("live W-30 render was silent or gesture-collapsed".into());
    }
    Ok(())
}

fn render_state(state: &JamAppState, bpm: f32) -> Vec<f32> {
    let bars = 8.0_f32;
    let frame_count = (bars * 4.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats: 0.0,
        },
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        w30_preview_render: state.runtime.w30_preview.clone(),
        w30_resample_tap: Default::default(),
        source_monitor_render: state.source_monitor_render_state(),
    };
    render_runtime_mix_realtime_simulation_offline(
        &plan,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        128,
    )
}

fn print_w30_render_summary(label: &str, render: &riotbox_audio::w30::W30PreviewRenderState) {
    println!(
        "{label} render: mode={:?} routing={:?} bus={} running={} tempo={} capture={:?} pad={:?}",
        render.mode,
        render.routing,
        render.music_bus_level,
        render.is_transport_running,
        render.tempo_bpm,
        render.capture_id,
        render.pad_playback.as_ref().map(|pad| (
            pad.sample_count,
            pad.playback_frame_count,
            pad.playback_rate,
            pad.reverse,
        )),
    );
}

fn commit(
    state: &mut JamAppState,
    kind: CommitBoundary,
    beat_index: u64,
    bar_index: u64,
    phrase_index: u64,
    scene_id: Option<riotbox_core::ids::SceneId>,
    timestamp: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind,
            beat_index,
            bar_index,
            phrase_index,
            scene_id,
        },
        timestamp,
    );
    if committed.len() != 1 {
        return Err(format!("expected one {kind:?} commit, got {}", committed.len()).into());
    }
    Ok(())
}

fn required_path(args: &[String], flag: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(required_value(args, flag)?))
}

fn required_value<'a>(
    args: &'a [String],
    flag: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let index = args
        .iter()
        .position(|arg| arg == flag)
        .ok_or(flag.to_string())?;
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {flag}").into())
}
