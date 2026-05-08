use super::*;

#[test]
fn source_timing_probe_bpm_candidates_preserve_bounded_groove_residuals() {
    let (onsets, strengths) = swung_eighth_onsets();
    let timing = timing_model_from_probe_bpm_candidates(
        &weighted_candidate_input("swung-eighths-120", 8.0, &onsets, &strengths),
        focused_120_bpm_policy(),
    );

    assert_bpm_close(timing.bpm_estimate, 120.0);
    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    let eighth = primary
        .groove
        .iter()
        .find(|residual| residual.subdivision == GrooveSubdivision::Eighth)
        .expect("eighth groove residual");

    assert!(
        (10.0..=30.0).contains(&eighth.offset_ms),
        "expected weighted late-eighth feel, got {eighth:?}"
    );
    assert!(eighth.confidence > 0.4, "{eighth:?}");
    assert!(primary
        .provenance
        .contains(&"source-timing-probe.groove-residual.v0".into()));
}

#[test]
fn source_timing_probe_bpm_candidates_skip_groove_when_onsets_are_too_sparse() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input("sparse-groove", 4.0, &[0.0, 0.54, 1.0]),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert!(timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.groove.is_empty())
        .unwrap_or(true));
}

fn swung_eighth_onsets() -> (Vec<f32>, Vec<f32>) {
    let mut onsets = Vec::new();
    let mut strengths = Vec::new();
    for beat in 0..16 {
        let beat_time = beat as f32 * 0.5;
        onsets.push(beat_time);
        strengths.push(if beat % 4 == 0 { 1.5 } else { 1.0 });

        onsets.push(beat_time + 0.29);
        strengths.push(0.75);
    }
    (onsets, strengths)
}
