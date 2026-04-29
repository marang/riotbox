// Keep these includes textually in this module so the existing tests retain their
// names and shared helpers while avoiding one token-heavy 7k-line test file.
include!("tests/common_imports_fixtures.rs");
include!("tests/common_state_fixtures.rs");
include!("tests/restore_contracts.rs");
include!("tests/persistence_runtime_view.rs");
include!("tests/replay_hardening.rs");
include!("tests/recovery_surface.rs");
include!("tests/feral_support_runtime_controls.rs");
include!("tests/transport_scene_select.rs");
include!("tests/scene_restore_mc202_queue_start.rs");
include!("tests/ghost_assist_queue.rs");
include!("tests/mc202_queue_timing.rs");
include!("tests/capture_w30_audition.rs");
include!("tests/w30_source_window_previews.rs");
include!("tests/w30_queue_core.rs");
include!("tests/w30_queue_conflicts_live_recall.rs");
include!("tests/w30_committed_bank_damage.rs");
include!("tests/w30_committed_preview_resample.rs");
include!("tests/w30_feral_rebake_policy.rs");
include!("tests/w30_backfill_mc202_commits.rs");
include!("tests/mc202_recipe_helpers.rs");
include!("tests/fixture_regressions_tr909_slam.rs");
include!("tests/tr909_takeover_source_support.rs");
include!("tests/tr909_policy_undo_ingest.rs");
include!("tests/scene_fixture_regression.rs");
