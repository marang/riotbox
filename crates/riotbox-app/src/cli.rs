//! CLI ownership: configuration, launch, terminal, controls and offline modes.
//! Persisted product truth remains in Core/Session and the app facade.

mod args;
mod controls;
mod daw_export_report;
mod daw_session_export;
mod daw_session_json_package;
mod daw_session_writer_plan;
mod daw_session_writer_proof;
mod event_loop;
mod launch;
mod live_master_dawproject;
mod live_master_recording;
mod live_recording_report;
mod model;
mod observer;
mod stem_package_export;
mod stem_package_handoff;
mod stem_package_report;
mod terminal;
mod w30_hook_dawproject;

pub use launch::run;

// Preserve the existing crate-internal handoff summary path during this
// mechanical migration. Its external caller is currently a Jam regression.
#[allow(unused_imports)]
pub(crate) use stem_package_handoff::{
    W30HookMusicianHandoffSummary, w30_hook_musician_handoff_summary,
};

#[cfg(test)]
mod intentional_quit_persistence_tests;
#[cfg(test)]
mod tests;
