use std::{env, fs, path::PathBuf};

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_audio::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, SourceMonitorRenderState,
        render_mc202_offline, render_runtime_mix_realtime_simulation_offline, signal_delta_metrics,
        signal_metrics,
    },
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
};
use riotbox_core::{
    action::{CaptureLengthIntent, CommitBoundary, SourceMonitorMode},
    live_performance_policy::derive_live_performance_policy,
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
    fs::create_dir_all(output_dir.join("stems"))?;

    let mut state = JamAppState::analyze_source_file_to_json_with_source_bpm_confirmation(
        &source_path,
        output_dir.join("session.json"),
        Some(output_dir.join("source-graph.json")),
        "python/sidecar/json_stdio_sidecar.py",
        23,
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
        1,
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
        1,
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
        1,
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
        1,
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
        1,
    )?;

    state.queue_tr909_reinforce(410);
    if state.queue_mc202_generate_pressure(411) != QueueControlResult::Enqueued {
        return Err("MC-202 pressure generation was unavailable".into());
    }
    commit(
        &mut state,
        CommitBoundary::Phrase,
        8,
        2,
        1,
        scene_id.clone(),
        500,
        2,
    )?;
    println!("TR-909 render: {:?}", state.runtime.tr909_render);
    println!(
        "MC-202 committed decision: {:?}",
        state
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
    );
    println!("MC-202 render: {:?}", state.runtime.mc202_render);
    let live_policy = state
        .source_graph
        .as_ref()
        .and_then(|graph| derive_live_performance_policy(&state.session, graph))
        .ok_or("dense-break live performance policy was unavailable")?;
    println!(
        "live performance policy: lead={} bass_owner={} mc202_intent={}",
        live_policy.lead.label(),
        live_policy.bass_owner.label(),
        live_policy.mc202_intent.label()
    );
    let normal_plan = render_plan(&state, bpm);
    let normal = render(&normal_plan, bpm);
    let w30 = render(&only_w30(&normal_plan), bpm);
    let tr909 = render(&only_tr909(&normal_plan), bpm);
    let mc202_selected_role = render(&only_mc202(&normal_plan), bpm);
    let mut direct_mc202_render = normal_plan.mc202_render;
    direct_mc202_render.is_transport_running = normal_plan.transport.is_transport_running;
    direct_mc202_render.tempo_bpm = normal_plan.transport.tempo_bpm;
    direct_mc202_render.position_beats = normal_plan.transport.position_beats;
    let direct_mc202 = render_mc202_offline(
        &direct_mc202_render,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        normal.len() / usize::from(CHANNEL_COUNT),
    );

    if state.queue_w30_apply_damage_profile(510).is_none() {
        return Err("W-30 damage gesture was unavailable".into());
    }
    commit(&mut state, CommitBoundary::Bar, 9, 3, 1, scene_id, 600, 1)?;
    let damaged = render(&render_plan(&state, bpm), bpm);

    for (name, samples) in [
        ("01_all_lane_hook.wav", &normal),
        ("02_all_lane_destructive.wav", &damaged),
        ("stems/01_w30_hook.wav", &w30),
        ("stems/02_tr909_pressure.wav", &tr909),
        ("stems/03_mc202_selected_role.wav", &mc202_selected_role),
    ] {
        write_interleaved_pcm16_wav(output_dir.join(name), SAMPLE_RATE, CHANNEL_COUNT, samples)?;
    }
    let source = SourceAudioCache::load_pcm_wav(&source_path)?;
    write_interleaved_pcm16_wav(
        output_dir.join("00_source.wav"),
        source.sample_rate,
        source.channel_count,
        source.interleaved_samples(),
    )?;
    state.save()?;

    let mix_metrics = signal_metrics(&normal);
    let damage_delta = signal_delta_metrics(&normal, &damaged);
    let w30_metrics = signal_metrics(&w30);
    let tr909_metrics = signal_metrics(&tr909);
    let mc202_metrics = signal_metrics(&mc202_selected_role);
    let mc202_stem_delta = signal_delta_metrics(&mc202_selected_role, &direct_mc202);
    println!("mix: {mix_metrics:?}");
    println!("damage delta: {damage_delta:?}");
    println!("w30: {w30_metrics:?}");
    println!("tr909: {tr909_metrics:?}");
    println!("mc202: {mc202_metrics:?}");
    println!("direct mc202: {:?}", signal_metrics(&direct_mc202));
    println!("mc202 stem/direct delta: {mc202_stem_delta:?}");
    if mix_metrics.rms <= 0.01
        || damage_delta.rms <= 0.01
        || w30_metrics.rms <= 0.005
        || tr909_metrics.rms <= 0.005
        || mc202_metrics.rms <= 0.005
        || mc202_stem_delta.rms > 0.000_01
        || mix_metrics.clip_count > 0
    {
        return Err("dense-break live path was silent, lane-collapsed, or clipping".into());
    }
    Ok(())
}

fn render_plan(state: &JamAppState, bpm: f32) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats: 0.0,
        },
        tr909_render: state.runtime.tr909_render.clone(),
        mc202_render: state.runtime.mc202_render,
        w30_preview_render: state.runtime.w30_preview.clone(),
        w30_resample_tap: Default::default(),
        source_monitor_render: state.source_monitor_render_state(),
    }
}

fn only_w30(plan: &RuntimeMixRenderPlan) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
        ..plan.clone()
    }
}

fn only_tr909(plan: &RuntimeMixRenderPlan) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        mc202_render: Default::default(),
        w30_preview_render: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
        ..plan.clone()
    }
}

fn only_mc202(plan: &RuntimeMixRenderPlan) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        tr909_render: Default::default(),
        w30_preview_render: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
        ..plan.clone()
    }
}

fn render(plan: &RuntimeMixRenderPlan, bpm: f32) -> Vec<f32> {
    let frame_count = (8.0 * 4.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    render_runtime_mix_realtime_simulation_offline(
        plan,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        128,
    )
}

#[allow(clippy::too_many_arguments)]
fn commit(
    state: &mut JamAppState,
    kind: CommitBoundary,
    beat_index: u64,
    bar_index: u64,
    phrase_index: u64,
    scene_id: Option<riotbox_core::ids::SceneId>,
    timestamp: u64,
    expected_count: usize,
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
    if committed.len() != expected_count {
        return Err(format!(
            "expected {expected_count} {kind:?} commits, got {}",
            committed.len()
        )
        .into());
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
