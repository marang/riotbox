use super::state::JamAppState;

impl JamAppState {
    pub fn toggle_pin_latest_capture(&mut self) -> Option<bool> {
        let new_state = {
            let capture = self.session.captures.last_mut()?;
            capture.is_pinned = !capture.is_pinned;
            capture.is_pinned
        };
        self.refresh_view();
        Some(new_state)
    }

    pub fn adjust_drum_bus_level(&mut self, delta: f32) -> f32 {
        let next_level =
            (self.session.runtime_state.mixer_state.drum_level + delta).clamp(0.0, 1.0);
        self.session.runtime_state.mixer_state.drum_level = next_level;
        self.refresh_view();
        next_level
    }

    pub fn adjust_mc202_touch(&mut self, delta: f32) -> f32 {
        let next_touch =
            (self.session.runtime_state.macro_state.mc202_touch + delta).clamp(0.0, 1.0);
        self.session.runtime_state.macro_state.mc202_touch = next_touch;
        self.refresh_view();
        next_touch
    }
}
