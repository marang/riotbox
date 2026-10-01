use riotbox_core::source_graph::SourceTimingProbeBpmCandidatePolicy;

pub(super) const SOURCE_TIMING_POLICY_PROFILE: SourceTimingPolicyProfile =
    SourceTimingPolicyProfile {
        name: SourceTimingProbeBpmCandidatePolicy::DANCE_LOOP_AUTO_READINESS_PROFILE,
        bpm_candidate_policy: SourceTimingProbeBpmCandidatePolicy::dance_loop_auto_readiness(),
    };

#[derive(Clone, Copy)]
pub(super) struct SourceTimingPolicyProfile {
    pub(super) name: &'static str,
    pub(super) bpm_candidate_policy: SourceTimingProbeBpmCandidatePolicy,
}
