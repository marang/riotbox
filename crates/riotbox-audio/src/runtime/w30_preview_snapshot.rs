//! W-30 control-thread publication and callback-local coherent snapshots.
#[cfg(test)]
use super::coherent_snapshot_or;
use super::{
    COHERENT_SNAPSHOT_READ_ATTEMPTS, begin_coherent_snapshot_update, coherent_snapshot,
    finish_coherent_snapshot_update, w30_mode_from_u32, w30_mode_to_u32, w30_routing_from_u32,
    w30_routing_to_u32, w30_source_profile_from_u32, w30_source_profile_to_u32,
};
use crate::w30::{
    W30_PAD_CHOP_SLICE_COUNT, W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30_PREVIEW_SAMPLE_WINDOW_LEN,
    W30HookArticulationProfile, W30PadPlaybackSampleWindow, W30PreviewRenderMode,
    W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSampleWindow,
    W30PreviewSourceProfile,
};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

#[cfg(test)]
mod tests;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct RealtimeW30PreviewRenderState {
    pub(super) mode: W30PreviewRenderMode,
    pub(super) routing: W30PreviewRenderRouting,
    pub(super) source_profile: Option<W30PreviewSourceProfile>,
    pub(super) trigger_revision: u64,
    pub(super) trigger_velocity: f32,
    pub(super) source_window_preview: RealtimeW30PreviewSampleWindow,
    pub(super) pad_playback: RealtimeW30PadPlaybackSampleWindow,
    pub(super) music_bus_level: f32,
    pub(super) grit_level: f32,
    pub(super) is_transport_running: bool,
    pub(super) tempo_bpm: f32,
    pub(super) position_beats: f64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct RealtimeW30PreviewSampleWindow {
    pub(super) source_start_frame: u64,
    pub(super) source_end_frame: u64,
    pub(super) sample_count: usize,
    pub(super) samples: [f32; W30_PREVIEW_SAMPLE_WINDOW_LEN],
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct RealtimeW30PadPlaybackSampleWindow {
    pub(super) source_start_frame: u64,
    pub(super) source_end_frame: u64,
    pub(super) source_sample_rate: u32,
    pub(super) playback_frame_count: u64,
    pub(super) sample_count: usize,
    pub(super) loop_enabled: bool,
    pub(super) playback_rate: f32,
    pub(super) reverse: bool,
    pub(super) gate_step_fraction: f32,
    pub(super) loop_crossfade_sample_count: usize,
    pub(super) chop_slice_count: usize,
    pub(super) chop_slice_starts: [u32; W30_PAD_CHOP_SLICE_COUNT],
    pub(super) hook_articulation_profile: Option<W30HookArticulationProfile>,
    pub(super) hook_articulation_started_at_beat: u64,
    pub(super) samples: [f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
}

impl Default for RealtimeW30PreviewSampleWindow {
    fn default() -> Self {
        Self {
            source_start_frame: 0,
            source_end_frame: 0,
            sample_count: 0,
            samples: [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN],
        }
    }
}

impl Default for RealtimeW30PadPlaybackSampleWindow {
    fn default() -> Self {
        Self {
            source_start_frame: 0,
            source_end_frame: 0,
            source_sample_rate: 0,
            playback_frame_count: 0,
            sample_count: 0,
            loop_enabled: false,
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
            loop_crossfade_sample_count: 0,
            chop_slice_count: 0,
            chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
            hook_articulation_profile: None,
            hook_articulation_started_at_beat: 0,
            samples: [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
        }
    }
}

pub(super) struct SharedW30PreviewRenderState {
    pub(super) revision: AtomicU64,
    pub(super) mode: AtomicU32,
    pub(super) routing: AtomicU32,
    pub(super) source_profile: AtomicU32,
    pub(super) trigger_revision: AtomicU64,
    pub(super) trigger_velocity_bits: AtomicU32,
    pub(super) source_start_frame: AtomicU64,
    pub(super) source_end_frame: AtomicU64,
    pub(super) source_sample_count: AtomicU32,
    pub(super) source_samples: [AtomicU32; W30_PREVIEW_SAMPLE_WINDOW_LEN],
    pub(super) pad_start_frame: AtomicU64,
    pub(super) pad_end_frame: AtomicU64,
    pub(super) pad_source_sample_rate: AtomicU32,
    pub(super) pad_playback_frame_count: AtomicU64,
    pub(super) pad_sample_count: AtomicU32,
    pub(super) pad_loop_enabled: AtomicBool,
    pub(super) pad_playback_rate_bits: AtomicU32,
    pub(super) pad_reverse: AtomicBool,
    pub(super) pad_gate_step_fraction_bits: AtomicU32,
    pub(super) pad_loop_crossfade_sample_count: AtomicU32,
    pub(super) pad_chop_slice_count: AtomicU32,
    pub(super) pad_chop_slice_starts: [AtomicU32; W30_PAD_CHOP_SLICE_COUNT],
    pub(super) pad_samples: [AtomicU32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
    pub(super) hook_articulation_profile: AtomicU32,
    pub(super) hook_articulation_started_at_beat: AtomicU64,
    pub(super) music_bus_level_bits: AtomicU32,
    pub(super) grit_level_bits: AtomicU32,
    pub(super) is_transport_running: AtomicBool,
    pub(super) tempo_bpm_bits: AtomicU32,
    pub(super) position_beats_bits: AtomicU64,
}

/// One callback's last complete snapshot of one fixed shared W-30 group.
/// The revision is the publication revision, not the musician's trigger revision.
pub(super) struct W30PreviewSnapshotCache {
    revision: Option<u64>,
    state: RealtimeW30PreviewRenderState,
}

impl W30PreviewSnapshotCache {
    pub(super) fn new(shared: &SharedW30PreviewRenderState) -> Self {
        // Startup also needs a complete fallback: never tag an unchecked read
        // with a separately sampled revision while the writer is active.
        let mut cache = Self {
            revision: None,
            state: RealtimeW30PreviewRenderState {
                mode: W30PreviewRenderMode::Idle,
                routing: W30PreviewRenderRouting::Silent,
                source_profile: None,
                trigger_revision: 0,
                trigger_velocity: 0.0,
                source_window_preview: RealtimeW30PreviewSampleWindow::default(),
                pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
                music_bus_level: 0.0,
                grit_level: 0.0,
                is_transport_running: false,
                tempo_bpm: 0.0,
                position_beats: 0.0,
            },
        };
        cache.refresh(shared);
        cache
    }

    pub(super) fn refresh(
        &mut self,
        shared: &SharedW30PreviewRenderState,
    ) -> &mut RealtimeW30PreviewRenderState {
        self.refresh_from(&shared.revision, || shared.read_snapshot_fields())
    }

    fn refresh_from(
        &mut self,
        revision: &AtomicU64,
        mut read: impl FnMut() -> RealtimeW30PreviewRenderState,
    ) -> &mut RealtimeW30PreviewRenderState {
        for _ in 0..COHERENT_SNAPSHOT_READ_ATTEMPTS {
            let before = revision.load(Ordering::Acquire);
            if !before.is_multiple_of(2) {
                continue;
            }
            if self.revision == Some(before) {
                // Borrow in place: no sample-array loads or large snapshot copy.
                return &mut self.state;
            }
            let candidate = read();
            let after = revision.load(Ordering::Acquire);
            if before == after && after.is_multiple_of(2) {
                self.state = candidate;
                self.revision = Some(after);
                return &mut self.state;
            }
        }
        // Keep the revision too, so the next callback retries a rejected update.
        &mut self.state
    }
}

impl SharedW30PreviewRenderState {
    pub(super) fn new(render_state: &W30PreviewRenderState) -> Self {
        let shared = Self {
            revision: AtomicU64::new(0),
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            source_profile: AtomicU32::new(0),
            trigger_revision: AtomicU64::new(0),
            trigger_velocity_bits: AtomicU32::new(0),
            source_start_frame: AtomicU64::new(0),
            source_end_frame: AtomicU64::new(0),
            source_sample_count: AtomicU32::new(0),
            source_samples: std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits())),
            pad_start_frame: AtomicU64::new(0),
            pad_end_frame: AtomicU64::new(0),
            pad_source_sample_rate: AtomicU32::new(0),
            pad_playback_frame_count: AtomicU64::new(0),
            pad_sample_count: AtomicU32::new(0),
            pad_loop_enabled: AtomicBool::new(false),
            pad_playback_rate_bits: AtomicU32::new(1.0_f32.to_bits()),
            pad_reverse: AtomicBool::new(false),
            pad_gate_step_fraction_bits: AtomicU32::new(0.0_f32.to_bits()),
            pad_loop_crossfade_sample_count: AtomicU32::new(0),
            pad_chop_slice_count: AtomicU32::new(0),
            pad_chop_slice_starts: std::array::from_fn(|_| AtomicU32::new(0)),
            pad_samples: std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits())),
            hook_articulation_profile: AtomicU32::new(0),
            hook_articulation_started_at_beat: AtomicU64::new(0),
            music_bus_level_bits: AtomicU32::new(0),
            grit_level_bits: AtomicU32::new(0),
            is_transport_running: AtomicBool::new(false),
            tempo_bpm_bits: AtomicU32::new(0),
            position_beats_bits: AtomicU64::new(0),
        };
        shared.update(render_state);
        shared
    }

    pub(super) fn update(&self, render_state: &W30PreviewRenderState) {
        begin_coherent_snapshot_update(&self.revision);
        self.mode
            .store(w30_mode_to_u32(render_state.mode), Ordering::Relaxed);
        self.routing
            .store(w30_routing_to_u32(render_state.routing), Ordering::Relaxed);
        self.source_profile.store(
            w30_source_profile_to_u32(render_state.source_profile),
            Ordering::Relaxed,
        );
        self.trigger_revision
            .store(render_state.trigger_revision, Ordering::Relaxed);
        self.trigger_velocity_bits
            .store(render_state.trigger_velocity.to_bits(), Ordering::Relaxed);
        self.update_source_window_preview(render_state.source_window_preview.as_ref());
        self.update_pad_playback(render_state.pad_playback.as_ref());
        self.music_bus_level_bits
            .store(render_state.music_bus_level.to_bits(), Ordering::Relaxed);
        self.grit_level_bits
            .store(render_state.grit_level.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(render_state.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(render_state.position_beats.to_bits(), Ordering::Relaxed);
        finish_coherent_snapshot_update(&self.revision);
    }

    pub(super) fn snapshot(&self) -> RealtimeW30PreviewRenderState {
        coherent_snapshot(&self.revision, || self.read_snapshot_fields())
    }

    #[cfg(test)]
    pub(super) fn snapshot_or_previous(
        &self,
        previous: &RealtimeW30PreviewRenderState,
    ) -> RealtimeW30PreviewRenderState {
        coherent_snapshot_or(&self.revision, previous, || self.read_snapshot_fields())
    }

    fn read_snapshot_fields(&self) -> RealtimeW30PreviewRenderState {
        RealtimeW30PreviewRenderState {
            mode: w30_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: w30_routing_from_u32(self.routing.load(Ordering::Relaxed)),
            source_profile: w30_source_profile_from_u32(
                self.source_profile.load(Ordering::Relaxed),
            ),
            trigger_revision: self.trigger_revision.load(Ordering::Relaxed),
            trigger_velocity: f32::from_bits(self.trigger_velocity_bits.load(Ordering::Relaxed)),
            source_window_preview: self.source_window_preview_snapshot(),
            pad_playback: self.pad_playback_snapshot(),
            music_bus_level: f32::from_bits(self.music_bus_level_bits.load(Ordering::Relaxed)),
            grit_level: f32::from_bits(self.grit_level_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
        }
    }

    fn update_source_window_preview(&self, source_window: Option<&W30PreviewSampleWindow>) {
        if let Some(source_window) = source_window {
            let sample_count = source_window
                .sample_count
                .min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
            self.source_start_frame
                .store(source_window.source_start_frame, Ordering::Relaxed);
            self.source_end_frame
                .store(source_window.source_end_frame, Ordering::Relaxed);
            for (index, sample) in source_window.samples.iter().copied().enumerate() {
                self.source_samples[index].store(sample.to_bits(), Ordering::Relaxed);
            }
            self.source_sample_count
                .store(sample_count as u32, Ordering::Relaxed);
        } else {
            self.source_sample_count.store(0, Ordering::Relaxed);
            self.source_start_frame.store(0, Ordering::Relaxed);
            self.source_end_frame.store(0, Ordering::Relaxed);
        }
    }

    fn source_window_preview_snapshot(&self) -> RealtimeW30PreviewSampleWindow {
        let sample_count = (self.source_sample_count.load(Ordering::Relaxed) as usize)
            .min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
        let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = f32::from_bits(self.source_samples[index].load(Ordering::Relaxed));
        }

        RealtimeW30PreviewSampleWindow {
            source_start_frame: self.source_start_frame.load(Ordering::Relaxed),
            source_end_frame: self.source_end_frame.load(Ordering::Relaxed),
            sample_count,
            samples,
        }
    }

    fn update_pad_playback(&self, pad_playback: Option<&W30PadPlaybackSampleWindow>) {
        if let Some(pad_playback) = pad_playback {
            let sample_count = pad_playback
                .sample_count
                .min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
            self.pad_start_frame
                .store(pad_playback.source_start_frame, Ordering::Relaxed);
            self.pad_end_frame
                .store(pad_playback.source_end_frame, Ordering::Relaxed);
            self.pad_source_sample_rate
                .store(pad_playback.source_sample_rate, Ordering::Relaxed);
            self.pad_playback_frame_count
                .store(pad_playback.playback_frame_count, Ordering::Relaxed);
            self.pad_loop_enabled
                .store(pad_playback.loop_enabled, Ordering::Relaxed);
            self.pad_playback_rate_bits
                .store(pad_playback.playback_rate.to_bits(), Ordering::Relaxed);
            self.pad_reverse
                .store(pad_playback.reverse, Ordering::Relaxed);
            self.pad_gate_step_fraction_bits
                .store(pad_playback.gate_step_fraction.to_bits(), Ordering::Relaxed);
            self.pad_loop_crossfade_sample_count.store(
                pad_playback.loop_crossfade_sample_count.min(sample_count) as u32,
                Ordering::Relaxed,
            );
            let chop_slice_count = pad_playback.chop_slice_count.min(W30_PAD_CHOP_SLICE_COUNT);
            for (index, start) in pad_playback.chop_slice_starts.iter().copied().enumerate() {
                self.pad_chop_slice_starts[index].store(start, Ordering::Relaxed);
            }
            self.pad_chop_slice_count
                .store(chop_slice_count as u32, Ordering::Relaxed);
            let (articulation_profile, articulation_started_at_beat) = pad_playback
                .hook_articulation
                .map_or((0, 0), |articulation| {
                    (
                        w30_hook_articulation_profile_to_u32(Some(articulation.profile)),
                        articulation.started_at_beat,
                    )
                });
            self.hook_articulation_profile
                .store(articulation_profile, Ordering::Relaxed);
            self.hook_articulation_started_at_beat
                .store(articulation_started_at_beat, Ordering::Relaxed);
            for (index, sample) in pad_playback.samples.iter().copied().enumerate() {
                self.pad_samples[index].store(sample.to_bits(), Ordering::Relaxed);
            }
            self.pad_sample_count
                .store(sample_count as u32, Ordering::Relaxed);
        } else {
            self.pad_sample_count.store(0, Ordering::Relaxed);
            self.pad_start_frame.store(0, Ordering::Relaxed);
            self.pad_end_frame.store(0, Ordering::Relaxed);
            self.pad_source_sample_rate.store(0, Ordering::Relaxed);
            self.pad_playback_frame_count.store(0, Ordering::Relaxed);
            self.pad_loop_enabled.store(false, Ordering::Relaxed);
            self.pad_playback_rate_bits
                .store(1.0_f32.to_bits(), Ordering::Relaxed);
            self.pad_reverse.store(false, Ordering::Relaxed);
            self.pad_gate_step_fraction_bits
                .store(0.0_f32.to_bits(), Ordering::Relaxed);
            self.pad_loop_crossfade_sample_count
                .store(0, Ordering::Relaxed);
            self.pad_chop_slice_count.store(0, Ordering::Relaxed);
            self.hook_articulation_profile.store(0, Ordering::Relaxed);
            self.hook_articulation_started_at_beat
                .store(0, Ordering::Relaxed);
        }
    }

    fn pad_playback_snapshot(&self) -> RealtimeW30PadPlaybackSampleWindow {
        let sample_count = (self.pad_sample_count.load(Ordering::Relaxed) as usize)
            .min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
        let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = f32::from_bits(self.pad_samples[index].load(Ordering::Relaxed));
        }
        let chop_slice_count = (self.pad_chop_slice_count.load(Ordering::Relaxed) as usize)
            .min(W30_PAD_CHOP_SLICE_COUNT);
        let chop_slice_starts = std::array::from_fn(|index| {
            self.pad_chop_slice_starts[index]
                .load(Ordering::Relaxed)
                .min(sample_count.saturating_sub(1) as u32)
        });

        RealtimeW30PadPlaybackSampleWindow {
            source_start_frame: self.pad_start_frame.load(Ordering::Relaxed),
            source_end_frame: self.pad_end_frame.load(Ordering::Relaxed),
            source_sample_rate: self.pad_source_sample_rate.load(Ordering::Relaxed),
            playback_frame_count: self.pad_playback_frame_count.load(Ordering::Relaxed),
            sample_count,
            loop_enabled: self.pad_loop_enabled.load(Ordering::Relaxed),
            playback_rate: f32::from_bits(self.pad_playback_rate_bits.load(Ordering::Relaxed)),
            reverse: self.pad_reverse.load(Ordering::Relaxed),
            gate_step_fraction: f32::from_bits(
                self.pad_gate_step_fraction_bits.load(Ordering::Relaxed),
            ),
            loop_crossfade_sample_count: (self
                .pad_loop_crossfade_sample_count
                .load(Ordering::Relaxed) as usize)
                .min(sample_count),
            chop_slice_count,
            chop_slice_starts,
            hook_articulation_profile: w30_hook_articulation_profile_from_u32(
                self.hook_articulation_profile.load(Ordering::Relaxed),
            ),
            hook_articulation_started_at_beat: self
                .hook_articulation_started_at_beat
                .load(Ordering::Relaxed),
            samples,
        }
    }
}

fn w30_hook_articulation_profile_to_u32(profile: Option<W30HookArticulationProfile>) -> u32 {
    match profile {
        Some(W30HookArticulationProfile::TurnaroundV1) => 1,
        Some(W30HookArticulationProfile::PitchDiveV1) => 2,
        Some(W30HookArticulationProfile::FilterSlamV1) => 3,
        None => 0,
    }
}

fn w30_hook_articulation_profile_from_u32(value: u32) -> Option<W30HookArticulationProfile> {
    match value {
        1 => Some(W30HookArticulationProfile::TurnaroundV1),
        2 => Some(W30HookArticulationProfile::PitchDiveV1),
        3 => Some(W30HookArticulationProfile::FilterSlamV1),
        _ => None,
    }
}
