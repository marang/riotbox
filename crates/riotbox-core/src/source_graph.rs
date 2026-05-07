use serde::{Deserialize, Serialize};

// Textual includes keep Source Graph sub-areas in one public module while
// letting each file keep a semantic review boundary.
include!("source_graph/timing.rs");
include!("source_graph/timing_analysis.rs");
include!("source_graph/model_and_helpers.rs");
include!("source_graph/tests.rs");
include!("source_graph/timing_tests.rs");
