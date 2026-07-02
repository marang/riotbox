use serde::Serialize;

pub const PERFORM_RISK_CUE_SCHEMA: &str = "riotbox.jam_perform_risk_cue_contract.v1";
pub const PERFORM_RISK_CUE_SURFACE: &str = "timing_source_risk_before_confident_moves";
pub const PERFORM_RISK_DEGRADED_LABEL: &str = "degraded";
pub const PERFORM_RISK_UNAVAILABLE_LABEL: &str = "unavailable";
pub const PERFORM_RISK_BAR_LIVE_CUE: &str = "bar/live?";
pub const PERFORM_RISK_EVIDENCE_ROLE: &str = "current_tui_contract";

pub const PERFORM_RISK_REQUIRED_PLAYER_CUES: [&str; 3] = [
    "show unavailable/degraded state before confident bar-locked moves",
    "show unavailable/degraded state before live-trigger promotion",
    "show timing/source-risk reason instead of generic failure text",
];

#[derive(Debug, Serialize)]
pub struct PerformRiskCueContract<'a> {
    pub schema: &'a str,
    pub schema_version: u8,
    pub result: &'a str,
    pub cue_surface: &'a str,
    pub evidence_role: &'a str,
    pub quality_proof: bool,
    pub automated_musical_approval: bool,
    pub degraded_state_label: &'a str,
    pub degraded_action: &'a str,
    pub unavailable_state_label: &'a str,
    pub unavailable_action: &'a str,
    pub required_player_cues: Vec<&'a str>,
}

pub fn perform_risk_cue_contract() -> PerformRiskCueContract<'static> {
    PerformRiskCueContract {
        schema: PERFORM_RISK_CUE_SCHEMA,
        schema_version: 1,
        result: "pass",
        cue_surface: PERFORM_RISK_CUE_SURFACE,
        evidence_role: PERFORM_RISK_EVIDENCE_ROLE,
        quality_proof: false,
        automated_musical_approval: false,
        degraded_state_label: PERFORM_RISK_DEGRADED_LABEL,
        degraded_action: PERFORM_RISK_BAR_LIVE_CUE,
        unavailable_state_label: PERFORM_RISK_UNAVAILABLE_LABEL,
        unavailable_action: PERFORM_RISK_BAR_LIVE_CUE,
        required_player_cues: PERFORM_RISK_REQUIRED_PLAYER_CUES.to_vec(),
    }
}
