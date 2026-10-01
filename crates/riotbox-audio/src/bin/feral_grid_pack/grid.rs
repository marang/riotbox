use super::config::{CHANNEL_COUNT, SAMPLE_RATE};

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
        // Vec<f32> needs a representable interleaved byte layout, not only frames.
        // Saturated float-to-usize counts are rejected here too, before rendering.
        let buffer_bytes = total_frames
            .checked_mul(usize::from(CHANNEL_COUNT))
            .and_then(|samples| samples.checked_mul(size_of::<f32>()));
        if buffer_bytes.is_none_or(|bytes| bytes > isize::MAX as usize) {
            return Err("grid render buffer capacity exceeded".to_string());
        }
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

#[cfg(test)]
mod tests {
    use super::{Grid, frames_for_beats};
    use crate::config::{CHANNEL_COUNT, SAMPLE_RATE};

    const CAPACITY_ERROR: &str = "grid render buffer capacity exceeded";

    // Derive boundary controls for this architecture; never allocate these grids.
    fn bpm_for_frames(frames: f64) -> f32 {
        (8.0 * f64::from(SAMPLE_RATE) * 60.0 / frames) as f32
    }

    fn sample_bytes(frames: usize) -> Option<usize> {
        frames
            .checked_mul(usize::from(CHANNEL_COUNT))
            .and_then(|samples| samples.checked_mul(size_of::<f32>()))
    }

    #[test]
    fn rejects_saturated_frame_count() {
        assert_eq!(frames_for_beats(1e-38, 8), usize::MAX);
        assert_eq!(Grid::new(1e-38, 4, 2).unwrap_err(), CAPACITY_ERROR);
    }

    #[test]
    fn rejects_interleaved_sample_count_overflow() {
        let bpm = bpm_for_frames(usize::MAX as f64 * 0.75);
        let frames = frames_for_beats(bpm, 8);
        assert!(frames < usize::MAX);
        assert!(frames.checked_mul(usize::from(CHANNEL_COUNT)).is_none());
        assert_eq!(Grid::new(bpm, 4, 2).unwrap_err(), CAPACITY_ERROR);
    }

    #[test]
    fn rejects_interleaved_byte_count_overflow() {
        let bpm = bpm_for_frames(usize::MAX as f64 * 0.25);
        let frames = frames_for_beats(bpm, 8);
        assert!(frames.checked_mul(usize::from(CHANNEL_COUNT)).is_some());
        assert!(sample_bytes(frames).is_none());
        assert_eq!(Grid::new(bpm, 4, 2).unwrap_err(), CAPACITY_ERROR);
    }

    #[test]
    fn rejects_address_space_overflow_with_representable_usize_bytes() {
        let frame_limit =
            isize::MAX as f64 / usize::from(CHANNEL_COUNT) as f64 / size_of::<f32>() as f64;
        let bpm = bpm_for_frames(frame_limit * 1.5);
        let bytes = sample_bytes(frames_for_beats(bpm, 8)).expect("usize bytes fit");
        assert!(bytes > isize::MAX as usize);
        assert_eq!(Grid::new(bpm, 4, 2).unwrap_err(), CAPACITY_ERROR);
    }

    #[test]
    fn accepts_representable_capacity_without_allocating() {
        let frame_limit =
            isize::MAX as f64 / usize::from(CHANNEL_COUNT) as f64 / size_of::<f32>() as f64;
        let bpm = bpm_for_frames(frame_limit * 0.5);
        let grid = Grid::new(bpm, 4, 2).expect("representable, not a memory-budget policy");
        assert_eq!(grid.total_frames, frames_for_beats(bpm, 8));
        assert!(sample_bytes(grid.total_frames).unwrap() <= isize::MAX as usize);
    }

    #[test]
    fn normal_cumulative_grid_and_boundary_errors_are_unchanged() {
        assert_eq!(Grid::new(128.0, 4, 2).unwrap().total_frames, 165_375);
        for bpm in [60.0, 128.0, 140.0, 173.25, 320.0] {
            for bars in [2, 4, 8] {
                let grid = Grid::new(bpm, 4, bars).unwrap();
                assert_eq!(grid.total_beats, 4 * bars);
                assert_eq!(grid.total_frames, frames_for_beats(bpm, 4 * bars));
                assert_eq!(grid.bar_end_frame(bars - 1), grid.total_frames);
                for bar in 0..bars {
                    assert_eq!(grid.bar_start_frame(bar), frames_for_beats(bpm, bar * 4));
                    assert!(grid.bar_end_frame(bar) <= grid.total_frames);
                }
            }
        }
        assert_eq!(
            Grid::new(128.0, 4, u32::MAX).unwrap_err(),
            "grid beat count overflowed"
        );
        for bpm in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                Grid::new(bpm, 4, 2).unwrap_err(),
                "bpm must be greater than zero"
            );
        }
        assert_eq!(
            Grid::new(128.0, 0, 2).unwrap_err(),
            "beats_per_bar and bars must be greater than zero"
        );
        assert_eq!(
            Grid::new(128.0, 4, 0).unwrap_err(),
            "beats_per_bar and bars must be greater than zero"
        );
    }
}
