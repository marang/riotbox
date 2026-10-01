use super::config::SAMPLE_RATE;

#[derive(Clone, Debug)]
pub(super) struct Grid {
    pub(super) bpm: f32,
    pub(super) beats_per_bar: u32,
    pub(super) bars: u32,
    pub(super) total_beats: u32,
    pub(super) total_frames: usize,
}

impl Grid {
    pub(super) fn new(bpm: f32, beats_per_bar: u32, bars: u32) -> Result<Self, String> {
        if !bpm.is_finite() || bpm <= 0.0 {
            return Err("bpm must be greater than zero".to_string());
        }
        if beats_per_bar == 0 || bars == 0 {
            return Err("beats_per_bar and bars must be greater than zero".to_string());
        }
        let total_beats = beats_per_bar
            .checked_mul(bars)
            .ok_or_else(|| "grid beat count overflowed".to_string())?;
        let total_frames = frames_for_beats(bpm, total_beats);
        Ok(Self {
            bpm,
            beats_per_bar,
            bars,
            total_beats,
            total_frames,
        })
    }

    pub(super) fn duration_seconds(&self) -> f32 {
        self.total_beats as f32 * 60.0 / self.bpm
    }

    pub(super) fn bar_start_frame(&self, bar: u32) -> usize {
        frames_for_beats(self.bpm, bar.saturating_mul(self.beats_per_bar))
    }

    pub(super) fn bar_end_frame(&self, bar: u32) -> usize {
        frames_for_beats(self.bpm, (bar + 1).saturating_mul(self.beats_per_bar))
    }

    #[cfg(test)]
    pub(super) fn bar_frame_count(&self, bar: u32) -> usize {
        self.bar_end_frame(bar)
            .saturating_sub(self.bar_start_frame(bar))
    }
}

pub(super) fn frames_for_beats(bpm: f32, beats: u32) -> usize {
    (beats as f64 * f64::from(SAMPLE_RATE) * 60.0 / f64::from(bpm)).round() as usize
}
