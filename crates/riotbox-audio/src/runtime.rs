// Keep the runtime textually included for this mechanical split so realtime
// behavior and private-state boundaries stay unchanged.
include!("runtime/public_api_shell.rs");
include!("runtime/shared_transport_tr909.rs");
include!("runtime/shared_mc202_w30_preview.rs");
include!("runtime/shared_w30_resample_callback.rs");
include!("runtime/render_tr909_w30_preview.rs");
include!("runtime/w30_tr909_signal_helpers.rs");
include!("runtime/tr909_tail_telemetry.rs");

#[cfg(test)]
mod tests;
