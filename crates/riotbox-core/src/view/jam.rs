// Textual includes keep this large file split mechanical and behavior-preserving.
// Preserve the original view-facing import while Core owns the comparison.
pub use crate::session::source_timing_confirmation_matches_graph;
use crate::source_graph::sorted_sections;
include!("jam/view_model_types.rs");
include!("jam/source_timing_summary.rs");
include!("jam/source_map.rs");
include!("jam/arrangement_contract.rs");
include!("jam/build_view_model.rs");
include!("jam/capture_actions.rs");
include!("jam/scene_launch.rs");
include!("jam/tests.rs");
