// Textual includes are kept mechanical for the initial binary-to-library move.
// RIOTBOX-1325 makes the bin entrypoint thin without changing CLI behavior;
// semantic CLI child modules should be split in follow-up slices.
include!("bin/riotbox-app/launch.rs");
include!("bin/riotbox-app/stem_package_export_cli.rs");
include!("bin/riotbox-app/stem_package_handoff_cli.rs");
include!("bin/riotbox-app/stem_package_report_cli.rs");
include!("bin/riotbox-app/live_recording_report_cli.rs");
mod live_master_recording;
use live_master_recording::run_live_master_recording_execute;
include!("bin/riotbox-app/daw_export_report_cli.rs");
include!("bin/riotbox-app/daw_session_json_package_cli.rs");
include!("bin/riotbox-app/daw_session_writer_proof_cli.rs");
include!("bin/riotbox-app/daw_session_export_cli.rs");
include!("bin/riotbox-app/w30_hook_dawproject_cli.rs");
include!("bin/riotbox-app/daw_session_writer_plan_cli.rs");
include!("bin/riotbox-app/launch_summary.rs");
include!("bin/riotbox-app/event_loop.rs");
include!("bin/riotbox-app/event_loop_helpers.rs");
include!("bin/riotbox-app/capture_length_control.rs");
include!("bin/riotbox-app/product_export_control.rs");
include!("bin/riotbox-app/source_map_navigation_control.rs");
include!("bin/riotbox-app/source_timing_confirm_control.rs");
include!("bin/riotbox-app/args_support.rs");
include!("bin/riotbox-app/args_daw_session.rs");
include!("bin/riotbox-app/args.rs");
include!("bin/riotbox-app/tests.rs");

#[cfg(test)]
#[path = "bin/riotbox-app/tests/intentional_quit_persistence.rs"]
mod intentional_quit_persistence_tests;
