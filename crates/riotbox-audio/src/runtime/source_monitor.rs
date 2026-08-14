use super::*;
use riotbox_core::action::SourceMonitorMode;

const SOURCE_GAIN: f32 = 0.88;
const BLEND_SOURCE_GAIN: f32 = 0.62;
const BLEND_RIOTBOX_GAIN: f32 = 0.62;
const SOURCE_MONITOR_TRANSITION_SECONDS: f64 = 0.005;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceMonitorAudioSource {
    pub sample_rate: u32,
    pub channel_count: u16,
    pub frame_count: usize,
    samples: Arc<Vec<f32>>,
}

impl SourceMonitorAudioSource {
    #[must_use]
    pub fn from_cache(cache: &SourceAudioCache) -> Self {
        Self {
            sample_rate: cache.sample_rate,
            channel_count: cache.channel_count,
            frame_count: cache.frame_count(),
            samples: cache.shared_interleaved_samples(),
        }
    }

    #[must_use]
    pub fn interleaved_samples(&self) -> &[f32] {
        self.samples.as_slice()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SourceMonitorAudioRoute {
    RiotboxOnly,
    SourceOnly,
    Blend,
    SourceUnavailable,
}

impl SourceMonitorAudioRoute {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RiotboxOnly => "riotbox_only",
            Self::SourceOnly => "source_only",
            Self::Blend => "blend",
            Self::SourceUnavailable => "source_unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceMonitorRenderState {
    pub mode: SourceMonitorMode,
    pub source: Option<SourceMonitorAudioSource>,
    pub is_transport_running: bool,
    pub tempo_bpm: f32,
    pub position_beats: f64,
    pub source_anchor_seconds: Option<f64>,
    pub source_anchor_position_beats: f64,
}

impl Default for SourceMonitorRenderState {
    fn default() -> Self {
        Self {
            mode: SourceMonitorMode::Source,
            source: None,
            is_transport_running: false,
            tempo_bpm: 128.0,
            position_beats: 0.0,
            source_anchor_seconds: None,
            source_anchor_position_beats: 0.0,
        }
    }
}

impl SourceMonitorRenderState {
    #[must_use]
    pub fn from_source_cache(mode: SourceMonitorMode, cache: Option<&SourceAudioCache>) -> Self {
        Self {
            mode,
            source: cache.map(SourceMonitorAudioSource::from_cache),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn control_only(mode: SourceMonitorMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn route_for_output(
        &self,
        sample_rate: u32,
        channel_count: usize,
    ) -> SourceMonitorAudioRoute {
        source_monitor_route(self.mode, self.source.as_ref(), sample_rate, channel_count)
    }
}

pub(super) struct SharedSourceMonitorRenderState {
    revision: AtomicU64,
    mode: AtomicU32,
    source_gain_bits: AtomicU32,
    riotbox_gain_bits: AtomicU32,
    source_anchor_present: AtomicBool,
    source_anchor_seconds_bits: AtomicU64,
    source_anchor_position_beats_bits: AtomicU64,
    source: Option<SourceMonitorAudioSource>,
}

impl SharedSourceMonitorRenderState {
    pub(super) fn new(render_state: &SourceMonitorRenderState) -> Self {
        let shared = Self {
            revision: AtomicU64::new(0),
            mode: AtomicU32::new(source_monitor_mode_to_u32(render_state.mode)),
            source_gain_bits: AtomicU32::new(SOURCE_GAIN.to_bits()),
            riotbox_gain_bits: AtomicU32::new(1.0_f32.to_bits()),
            source_anchor_present: AtomicBool::new(false),
            source_anchor_seconds_bits: AtomicU64::new(0.0_f64.to_bits()),
            source_anchor_position_beats_bits: AtomicU64::new(0.0_f64.to_bits()),
            source: render_state.source.clone(),
        };
        shared.update(render_state);
        shared
    }

    pub(super) fn update(&self, render_state: &SourceMonitorRenderState) {
        begin_coherent_snapshot_update(&self.revision);
        self.mode.store(
            source_monitor_mode_to_u32(render_state.mode),
            Ordering::Relaxed,
        );
        match render_state.mode {
            SourceMonitorMode::Source => {
                self.source_gain_bits
                    .store(SOURCE_GAIN.to_bits(), Ordering::Relaxed);
                self.riotbox_gain_bits
                    .store(0.0_f32.to_bits(), Ordering::Relaxed);
            }
            SourceMonitorMode::Blend => {
                self.source_gain_bits
                    .store(BLEND_SOURCE_GAIN.to_bits(), Ordering::Relaxed);
                self.riotbox_gain_bits
                    .store(BLEND_RIOTBOX_GAIN.to_bits(), Ordering::Relaxed);
            }
            SourceMonitorMode::Riotbox => {
                self.source_gain_bits
                    .store(0.0_f32.to_bits(), Ordering::Relaxed);
                self.riotbox_gain_bits
                    .store(1.0_f32.to_bits(), Ordering::Relaxed);
            }
        }
        self.source_anchor_present.store(
            render_state.source_anchor_seconds.is_some(),
            Ordering::Relaxed,
        );
        self.source_anchor_seconds_bits.store(
            render_state.source_anchor_seconds.unwrap_or(0.0).to_bits(),
            Ordering::Relaxed,
        );
        self.source_anchor_position_beats_bits.store(
            render_state.source_anchor_position_beats.to_bits(),
            Ordering::Relaxed,
        );
        finish_coherent_snapshot_update(&self.revision);
    }

    #[cfg(test)]
    pub(super) fn snapshot(&self) -> RealtimeSourceMonitorRenderState<'_> {
        self.render_snapshot_from_control(self.control_snapshot())
    }

    pub(super) fn control_snapshot(&self) -> RealtimeSourceMonitorControlSnapshot {
        coherent_snapshot(&self.revision, || self.read_control_fields())
    }

    pub(super) fn control_snapshot_or_previous(
        &self,
        previous: &RealtimeSourceMonitorControlSnapshot,
    ) -> RealtimeSourceMonitorControlSnapshot {
        coherent_snapshot_or(&self.revision, previous, || self.read_control_fields())
    }

    pub(super) fn render_snapshot_from_control(
        &self,
        control: RealtimeSourceMonitorControlSnapshot,
    ) -> RealtimeSourceMonitorRenderState<'_> {
        RealtimeSourceMonitorRenderState {
            mode: control.mode,
            source_gain: control.source_gain,
            riotbox_gain: control.riotbox_gain,
            source: self.source.as_ref(),
            is_transport_running: false,
            tempo_bpm: 128.0,
            position_beats: 0.0,
            source_anchor_seconds: control.source_anchor_seconds,
            source_anchor_position_beats: control.source_anchor_position_beats,
        }
    }

    fn read_control_fields(&self) -> RealtimeSourceMonitorControlSnapshot {
        RealtimeSourceMonitorControlSnapshot {
            mode: source_monitor_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            source_gain: f32::from_bits(self.source_gain_bits.load(Ordering::Relaxed)),
            riotbox_gain: f32::from_bits(self.riotbox_gain_bits.load(Ordering::Relaxed)),
            source_anchor_seconds: self
                .source_anchor_present
                .load(Ordering::Relaxed)
                .then(|| f64::from_bits(self.source_anchor_seconds_bits.load(Ordering::Relaxed))),
            source_anchor_position_beats: f64::from_bits(
                self.source_anchor_position_beats_bits
                    .load(Ordering::Relaxed),
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct RealtimeSourceMonitorControlSnapshot {
    pub(super) mode: SourceMonitorMode,
    pub(super) source_gain: f32,
    pub(super) riotbox_gain: f32,
    pub(super) source_anchor_seconds: Option<f64>,
    pub(super) source_anchor_position_beats: f64,
}

impl Default for RealtimeSourceMonitorControlSnapshot {
    fn default() -> Self {
        Self {
            mode: SourceMonitorMode::Source,
            source_gain: SOURCE_GAIN,
            riotbox_gain: 0.0,
            source_anchor_seconds: None,
            source_anchor_position_beats: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RealtimeSourceMonitorRenderState<'a> {
    pub(super) mode: SourceMonitorMode,
    pub(super) source_gain: f32,
    pub(super) riotbox_gain: f32,
    pub(super) source: Option<&'a SourceMonitorAudioSource>,
    pub(super) is_transport_running: bool,
    pub(super) tempo_bpm: f32,
    pub(super) position_beats: f64,
    pub(super) source_anchor_seconds: Option<f64>,
    pub(super) source_anchor_position_beats: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SourceMonitorCallbackState {
    initialized: bool,
    current_source_gain: f32,
    current_riotbox_gain: f32,
    target_source_gain: f32,
    target_riotbox_gain: f32,
    gain_transition_frames_remaining: usize,
    expected_next_source_position: Option<f64>,
    anchor_transition_old_position: f64,
    anchor_transition_frames_remaining: usize,
    anchor_transition_total_frames: usize,
}

impl Default for SourceMonitorCallbackState {
    fn default() -> Self {
        Self {
            initialized: false,
            current_source_gain: 0.0,
            current_riotbox_gain: 0.0,
            target_source_gain: 0.0,
            target_riotbox_gain: 0.0,
            gain_transition_frames_remaining: 0,
            expected_next_source_position: None,
            anchor_transition_old_position: 0.0,
            anchor_transition_frames_remaining: 0,
            anchor_transition_total_frames: 0,
        }
    }
}

pub(super) fn source_monitor_mode_to_u32(mode: SourceMonitorMode) -> u32 {
    match mode {
        SourceMonitorMode::Source => 0,
        SourceMonitorMode::Blend => 1,
        SourceMonitorMode::Riotbox => 2,
    }
}

pub(super) fn source_monitor_mode_from_u32(value: u32) -> SourceMonitorMode {
    match value {
        1 => SourceMonitorMode::Blend,
        2 => SourceMonitorMode::Riotbox,
        _ => SourceMonitorMode::Source,
    }
}

#[must_use]
pub fn source_monitor_route_for_cache(
    mode: SourceMonitorMode,
    cache: Option<&SourceAudioCache>,
) -> SourceMonitorAudioRoute {
    let Some(cache) = cache else {
        return source_monitor_route_for_metadata(mode, None, 0, 0);
    };
    source_monitor_route_for_output(mode, Some(cache), cache.sample_rate, cache.channel_count)
}

#[must_use]
pub fn source_monitor_route_for_output(
    mode: SourceMonitorMode,
    cache: Option<&SourceAudioCache>,
    sample_rate: u32,
    channel_count: u16,
) -> SourceMonitorAudioRoute {
    let source = cache.map(|cache| (cache.sample_rate, cache.channel_count, cache.frame_count()));
    source_monitor_route_for_metadata(mode, source, sample_rate, usize::from(channel_count))
}

#[must_use]
fn source_monitor_route(
    mode: SourceMonitorMode,
    source: Option<&SourceMonitorAudioSource>,
    sample_rate: u32,
    channel_count: usize,
) -> SourceMonitorAudioRoute {
    let source_available = source.is_some_and(|source| {
        source.sample_rate > 0
            && sample_rate > 0
            && source.frame_count > 0
            && source.channel_count > 0
            && channel_count > 0
    });
    match (mode, source_available) {
        (SourceMonitorMode::Riotbox, _) => SourceMonitorAudioRoute::RiotboxOnly,
        (SourceMonitorMode::Source, true) => SourceMonitorAudioRoute::SourceOnly,
        (SourceMonitorMode::Blend, true) => SourceMonitorAudioRoute::Blend,
        (SourceMonitorMode::Source | SourceMonitorMode::Blend, false) => {
            SourceMonitorAudioRoute::SourceUnavailable
        }
    }
}

#[must_use]
fn source_monitor_route_for_metadata(
    mode: SourceMonitorMode,
    source: Option<(u32, u16, usize)>,
    sample_rate: u32,
    channel_count: usize,
) -> SourceMonitorAudioRoute {
    let source_available =
        source.is_some_and(|(source_sample_rate, source_channel_count, frame_count)| {
            source_sample_rate > 0
                && sample_rate > 0
                && frame_count > 0
                && source_channel_count > 0
                && channel_count > 0
        });
    match (mode, source_available) {
        (SourceMonitorMode::Riotbox, _) => SourceMonitorAudioRoute::RiotboxOnly,
        (SourceMonitorMode::Source, true) => SourceMonitorAudioRoute::SourceOnly,
        (SourceMonitorMode::Blend, true) => SourceMonitorAudioRoute::Blend,
        (SourceMonitorMode::Source | SourceMonitorMode::Blend, false) => {
            SourceMonitorAudioRoute::SourceUnavailable
        }
    }
}

pub fn apply_source_monitor_policy(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeSourceMonitorRenderState<'_>,
) -> SourceMonitorAudioRoute {
    apply_source_monitor_policy_with_state(
        data,
        sample_rate,
        channel_count,
        render,
        &mut SourceMonitorCallbackState::default(),
    )
}

pub(super) fn apply_source_monitor_policy_with_state(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeSourceMonitorRenderState<'_>,
    callback_state: &mut SourceMonitorCallbackState,
) -> SourceMonitorAudioRoute {
    apply_source_monitor_policy_with_state_and_fill_focus(
        data,
        sample_rate,
        channel_count,
        render,
        FillFocusRenderState::inactive(),
        callback_state,
    )
}

pub(super) fn apply_source_monitor_policy_with_state_and_fill_focus(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeSourceMonitorRenderState<'_>,
    fill_focus: FillFocusRenderState,
    callback_state: &mut SourceMonitorCallbackState,
) -> SourceMonitorAudioRoute {
    let route = source_monitor_route(render.mode, render.source, sample_rate, channel_count);
    if channel_count == 0 || sample_rate == 0 {
        if render.mode == SourceMonitorMode::Source {
            data.fill(0.0);
        }
        return route;
    }

    let source = render.source.as_ref();
    let source_is_audible = render.is_transport_running
        && matches!(
            route,
            SourceMonitorAudioRoute::SourceOnly | SourceMonitorAudioRoute::Blend
        );
    let target_source_gain = if source_is_audible {
        render.source_gain
    } else {
        0.0
    };
    let target_riotbox_gain = match render.mode {
        SourceMonitorMode::Source => 0.0,
        SourceMonitorMode::Blend => render.riotbox_gain,
        SourceMonitorMode::Riotbox => render.riotbox_gain,
    };
    prepare_gain_transition(
        callback_state,
        target_source_gain,
        target_riotbox_gain,
        sample_rate,
    );

    let frame_count = data.len() / channel_count;
    let (start_position, source_frames_per_output_frame, source_channels) =
        source.map_or((0.0, 0.0, 0), |source| {
            (
                source_start_position(render, source),
                f64::from(source.sample_rate) / f64::from(sample_rate),
                usize::from(source.channel_count),
            )
        });
    prepare_anchor_transition(
        callback_state,
        source_is_audible,
        start_position,
        source_frames_per_output_frame,
        sample_rate,
    );
    let playback_start_position = if source_is_audible {
        start_position
    } else {
        callback_state
            .expected_next_source_position
            .unwrap_or(start_position)
    };

    for frame_index in 0..frame_count {
        advance_gain_transition(callback_state);
        let anchor_crossfade = anchor_crossfade(callback_state);
        let source_position =
            playback_start_position + (frame_index as f64 * source_frames_per_output_frame);
        let source_focus_gain = if matches!(route, SourceMonitorAudioRoute::Blend) {
            fill_focus.gain_at_frame(sample_rate, frame_index)
        } else {
            1.0
        };
        for channel in 0..channel_count {
            let output_index = frame_index * channel_count + channel;
            let source_sample = source.map_or(0.0, |source| {
                let new_sample =
                    source_sample_with_end_fade(source, source_position, channel, source_channels);
                anchor_crossfade.map_or(new_sample, |crossfade| {
                    let old_sample = source_sample_with_end_fade(
                        source,
                        callback_state.anchor_transition_old_position,
                        channel,
                        source_channels,
                    );
                    old_sample + ((new_sample - old_sample) * crossfade)
                })
            });
            data[output_index] = (data[output_index] * callback_state.current_riotbox_gain)
                + (source_sample * callback_state.current_source_gain * source_focus_gain);
        }
        if callback_state.anchor_transition_frames_remaining > 0 {
            callback_state.anchor_transition_old_position += source_frames_per_output_frame;
            callback_state.anchor_transition_frames_remaining -= 1;
        }
    }

    callback_state.expected_next_source_position =
        if source_is_audible || callback_state.current_source_gain.abs() > f32::EPSILON {
            Some(playback_start_position + (frame_count as f64 * source_frames_per_output_frame))
        } else {
            None
        };

    route
}

fn transition_frame_count(sample_rate: u32) -> usize {
    ((f64::from(sample_rate) * SOURCE_MONITOR_TRANSITION_SECONDS).round() as usize).max(1)
}

fn prepare_gain_transition(
    state: &mut SourceMonitorCallbackState,
    source_gain: f32,
    riotbox_gain: f32,
    sample_rate: u32,
) {
    if !state.initialized {
        state.initialized = true;
        state.current_source_gain = source_gain;
        state.current_riotbox_gain = riotbox_gain;
        state.target_source_gain = source_gain;
        state.target_riotbox_gain = riotbox_gain;
        return;
    }
    if state.target_source_gain.to_bits() != source_gain.to_bits()
        || state.target_riotbox_gain.to_bits() != riotbox_gain.to_bits()
    {
        state.target_source_gain = source_gain;
        state.target_riotbox_gain = riotbox_gain;
        state.gain_transition_frames_remaining = transition_frame_count(sample_rate);
    }
}

fn advance_gain_transition(state: &mut SourceMonitorCallbackState) {
    let remaining = state.gain_transition_frames_remaining;
    if remaining == 0 {
        state.current_source_gain = state.target_source_gain;
        state.current_riotbox_gain = state.target_riotbox_gain;
        return;
    }
    let remaining = remaining as f32;
    state.current_source_gain += (state.target_source_gain - state.current_source_gain) / remaining;
    state.current_riotbox_gain +=
        (state.target_riotbox_gain - state.current_riotbox_gain) / remaining;
    state.gain_transition_frames_remaining -= 1;
}

fn prepare_anchor_transition(
    state: &mut SourceMonitorCallbackState,
    source_is_audible: bool,
    start_position: f64,
    source_frames_per_output_frame: f64,
    sample_rate: u32,
) {
    if !source_is_audible {
        state.anchor_transition_frames_remaining = 0;
        return;
    }
    let Some(expected_position) = state.expected_next_source_position else {
        return;
    };
    if state.anchor_transition_frames_remaining > 0 {
        return;
    }
    let discontinuity = (start_position - expected_position).abs();
    if discontinuity <= source_frames_per_output_frame.abs().max(1.0) * 1.5 {
        return;
    }
    let transition_frames = transition_frame_count(sample_rate);
    state.anchor_transition_old_position = expected_position;
    state.anchor_transition_frames_remaining = transition_frames;
    state.anchor_transition_total_frames = transition_frames;
}

fn anchor_crossfade(state: &SourceMonitorCallbackState) -> Option<f32> {
    (state.anchor_transition_frames_remaining > 0).then(|| {
        let total_intervals = state
            .anchor_transition_total_frames
            .saturating_sub(1)
            .max(1);
        let remaining_intervals = state.anchor_transition_frames_remaining.saturating_sub(1);
        1.0 - (remaining_intervals as f32 / total_intervals as f32)
    })
}

fn source_sample_with_end_fade(
    source: &SourceMonitorAudioSource,
    frame_position: f64,
    output_channel: usize,
    source_channels: usize,
) -> f32 {
    let sample =
        interpolated_source_sample(source, frame_position, output_channel, source_channels);
    if sample == 0.0 || !frame_position.is_finite() {
        return sample;
    }
    let remaining_frames = source.frame_count as f64 - frame_position;
    let fade_frames = (f64::from(source.sample_rate) * SOURCE_MONITOR_TRANSITION_SECONDS)
        .min(source.frame_count as f64 / 16.0)
        .max(1.0);
    let fade = (remaining_frames / fade_frames.max(1.0)).clamp(0.0, 1.0) as f32;
    sample * fade
}

#[must_use]
pub fn render_source_monitor_mix_offline(
    generated: &[f32],
    sample_rate: u32,
    channel_count: u16,
    render_state: &SourceMonitorRenderState,
) -> Vec<f32> {
    let mut output = generated.to_vec();
    let render = RealtimeSourceMonitorRenderState {
        mode: render_state.mode,
        source_gain: match render_state.mode {
            SourceMonitorMode::Source => SOURCE_GAIN,
            SourceMonitorMode::Blend => BLEND_SOURCE_GAIN,
            SourceMonitorMode::Riotbox => 0.0,
        },
        riotbox_gain: match render_state.mode {
            SourceMonitorMode::Source => 0.0,
            SourceMonitorMode::Blend => BLEND_RIOTBOX_GAIN,
            SourceMonitorMode::Riotbox => 1.0,
        },
        source: render_state.source.as_ref(),
        is_transport_running: render_state.is_transport_running,
        tempo_bpm: render_state.tempo_bpm,
        position_beats: render_state.position_beats,
        source_anchor_seconds: render_state.source_anchor_seconds,
        source_anchor_position_beats: render_state.source_anchor_position_beats,
    };
    apply_source_monitor_policy(
        &mut output,
        sample_rate,
        usize::from(channel_count),
        &render,
    );
    output
}

fn source_start_position(
    render: &RealtimeSourceMonitorRenderState<'_>,
    source: &SourceMonitorAudioSource,
) -> f64 {
    if !render.is_transport_running
        || render.tempo_bpm <= 0.0
        || !render.tempo_bpm.is_finite()
        || !render.position_beats.is_finite()
        || source.frame_count == 0
    {
        return 0.0;
    }

    let transport_seconds = match render.source_anchor_seconds {
        Some(anchor_seconds) => {
            let relative_beats =
                (render.position_beats - render.source_anchor_position_beats).max(0.0);
            anchor_seconds.max(0.0) + relative_beats * 60.0 / f64::from(render.tempo_bpm)
        }
        None => render.position_beats.max(0.0) * 60.0 / f64::from(render.tempo_bpm),
    };
    transport_seconds * f64::from(source.sample_rate)
}

fn interpolated_source_sample(
    source: &SourceMonitorAudioSource,
    frame_position: f64,
    output_channel: usize,
    source_channels: usize,
) -> f32 {
    if !frame_position.is_finite()
        || frame_position < 0.0
        || frame_position >= source.frame_count as f64
    {
        return 0.0;
    }
    let first_frame = frame_position.floor() as usize;
    let second_frame = (first_frame + 1).min(source.frame_count.saturating_sub(1));
    let fraction = (frame_position - frame_position.floor()) as f32;
    let first = source_sample(source, first_frame, output_channel, source_channels);
    let second = source_sample(source, second_frame, output_channel, source_channels);
    first + ((second - first) * fraction)
}

fn source_sample(
    source: &SourceMonitorAudioSource,
    frame: usize,
    output_channel: usize,
    source_channels: usize,
) -> f32 {
    let source_channel = if source_channels == 1 {
        0
    } else {
        output_channel.min(source_channels.saturating_sub(1))
    };
    source
        .interleaved_samples()
        .get(frame * source_channels + source_channel)
        .copied()
        .unwrap_or(0.0)
}
