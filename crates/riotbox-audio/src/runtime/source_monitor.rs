use super::*;
use riotbox_core::action::SourceMonitorMode;

const SOURCE_GAIN: f32 = 0.88;
const BLEND_SOURCE_GAIN: f32 = 0.62;
const BLEND_RIOTBOX_GAIN: f32 = 0.62;

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
            samples: Arc::new(cache.interleaved_samples().to_vec()),
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

    pub(super) fn snapshot(&self) -> RealtimeSourceMonitorRenderState {
        coherent_snapshot(&self.revision, || self.read_snapshot_fields())
    }

    pub(super) fn snapshot_or_previous(
        &self,
        previous: &RealtimeSourceMonitorRenderState,
    ) -> RealtimeSourceMonitorRenderState {
        coherent_snapshot_or(&self.revision, previous, || self.read_snapshot_fields())
    }

    fn read_snapshot_fields(&self) -> RealtimeSourceMonitorRenderState {
        RealtimeSourceMonitorRenderState {
            mode: source_monitor_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            source_gain: f32::from_bits(self.source_gain_bits.load(Ordering::Relaxed)),
            riotbox_gain: f32::from_bits(self.riotbox_gain_bits.load(Ordering::Relaxed)),
            source: self.source.clone(),
            is_transport_running: false,
            tempo_bpm: 128.0,
            position_beats: 0.0,
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

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RealtimeSourceMonitorRenderState {
    pub(super) mode: SourceMonitorMode,
    pub(super) source_gain: f32,
    pub(super) riotbox_gain: f32,
    pub(super) source: Option<SourceMonitorAudioSource>,
    pub(super) is_transport_running: bool,
    pub(super) tempo_bpm: f32,
    pub(super) position_beats: f64,
    pub(super) source_anchor_seconds: Option<f64>,
    pub(super) source_anchor_position_beats: f64,
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
        source.sample_rate == sample_rate
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
            source_sample_rate == sample_rate
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
    render: &RealtimeSourceMonitorRenderState,
) -> SourceMonitorAudioRoute {
    let route = source_monitor_route(
        render.mode,
        render.source.as_ref(),
        sample_rate,
        channel_count,
    );
    if matches!(route, SourceMonitorAudioRoute::SourceUnavailable) {
        data.fill(0.0);
        return route;
    }
    let Some(source) = render.source.as_ref() else {
        return route;
    };
    if !matches!(
        route,
        SourceMonitorAudioRoute::SourceOnly | SourceMonitorAudioRoute::Blend
    ) {
        return route;
    }

    let frame_count = data.len() / channel_count.max(1);
    let source_channels = usize::from(source.channel_count);
    let start_frame = source_start_frame(render, source);

    for frame_index in 0..frame_count {
        let source_frame = (start_frame + frame_index) % source.frame_count;
        for channel in 0..channel_count {
            let output_index = frame_index * channel_count + channel;
            let source_sample = if render.is_transport_running {
                source_sample(source, source_frame, channel, source_channels)
            } else {
                0.0
            };
            data[output_index] = match route {
                SourceMonitorAudioRoute::SourceOnly => source_sample * render.source_gain,
                SourceMonitorAudioRoute::Blend => ((data[output_index] * render.riotbox_gain)
                    + (source_sample * render.source_gain))
                    .clamp(-1.0, 1.0),
                SourceMonitorAudioRoute::RiotboxOnly
                | SourceMonitorAudioRoute::SourceUnavailable => data[output_index],
            };
        }
    }

    route
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
        source: render_state.source.clone(),
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

fn source_start_frame(
    render: &RealtimeSourceMonitorRenderState,
    source: &SourceMonitorAudioSource,
) -> usize {
    if !render.is_transport_running
        || render.tempo_bpm <= 0.0
        || !render.tempo_bpm.is_finite()
        || !render.position_beats.is_finite()
        || source.frame_count == 0
    {
        return 0;
    }

    let transport_seconds = match render.source_anchor_seconds {
        Some(anchor_seconds) => {
            let relative_beats =
                (render.position_beats - render.source_anchor_position_beats).max(0.0);
            anchor_seconds.max(0.0) + relative_beats * 60.0 / f64::from(render.tempo_bpm)
        }
        None => render.position_beats.max(0.0) * 60.0 / f64::from(render.tempo_bpm),
    };
    ((transport_seconds * f64::from(source.sample_rate)).floor() as usize) % source.frame_count
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
