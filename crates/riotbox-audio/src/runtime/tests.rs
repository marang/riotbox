// Textual includes preserve the existing runtime::tests::* names while keeping
// audio regression groups small enough to inspect without loading every test.
include!("tests/fixtures_lifecycle.rs");
include!("tests/shared_w30_preview.rs");
include!("tests/w30_resample_support.rs");
include!("tests/mix_offline_tr909.rs");
include!("tests/fixture_regressions_variations.rs");
include!("tests/signal_metrics.rs");
