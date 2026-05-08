const SOURCE_TIMING_POLICY_PROFILE: SourceTimingPolicyProfile = SourceTimingPolicyProfile {
    name: SourceTimingProbeBpmCandidatePolicy::DANCE_LOOP_AUTO_READINESS_PROFILE,
    bpm_candidate_policy: SourceTimingProbeBpmCandidatePolicy::dance_loop_auto_readiness(),
};

#[derive(Clone, Copy)]
struct SourceTimingPolicyProfile {
    name: &'static str,
    bpm_candidate_policy: SourceTimingProbeBpmCandidatePolicy,
}
