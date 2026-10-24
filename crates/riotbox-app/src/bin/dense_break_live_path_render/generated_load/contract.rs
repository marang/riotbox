use std::error::Error;

use serde::Deserialize;
use sha2::{Digest, Sha256};

const BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/benchmarks/generated_live_master_load_fixture_v1.json"
));

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Contract {
    schema: String,
    schema_version: u32,
    pub(super) source_case_id: String,
    pub(super) source_seconds: u32,
    pub(super) source_bpm: f32,
    pub(super) source_sample_rate: u32,
    pub(super) source_channels: u16,
    prefix_profile: String,
    window_bars: [u8; 3],
    pub(super) output_sample_rates: [u32; 2],
    pub(super) output_channels: u16,
    pub(super) offline_callback_frames: usize,
    classification: String,
    quality_proof: bool,
    source_intelligence_qualified: bool,
    host_execution_allowed: bool,
    holdout_audio_allowed: bool,
}

impl Contract {
    pub(super) fn load() -> Result<Self, Box<dyn Error>> {
        Self::parse(BYTES)
    }

    fn parse(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let value: Self = serde_json::from_slice(bytes)?;
        if value.schema != "riotbox.generated_live_master_load_fixture.v1"
            || value.schema_version != 1
            || value.source_case_id != "dense_break_132"
            || value.source_seconds != 32
            || value.source_bpm.to_bits() != 132.0_f32.to_bits()
            || value.source_sample_rate != 44_100
            || value.source_channels != 2
            || value.prefix_profile != "feral_break_alpha_v2_ready_v1"
            || value.window_bars != [2, 8, 16]
            || value.output_sample_rates != [44_100, 48_000]
            || value.output_channels != 2
            || value.offline_callback_frames != 128
            || value.classification != "generated_control_preparation_only"
            || value.quality_proof
            || value.source_intelligence_qualified
            || value.host_execution_allowed
            || value.holdout_audio_allowed
        {
            return Err(
                "unsupported generated fixture contract: a new version/decision is required".into(),
            );
        }
        Ok(value)
    }
}

pub(super) fn digest() -> String {
    hex::encode(Sha256::digest(BYTES))
}

#[cfg(test)]
mod tests {
    use super::{BYTES, Contract};

    #[test]
    fn frozen_contract_rejects_profile_geometry_claim_or_extra_key_drift() {
        Contract::load().expect("frozen contract");
        for (key, changed) in [
            ("source_case_id", serde_json::json!("arbitrary-input")),
            ("output_sample_rates", serde_json::json!([44_100, 96_000])),
            ("host_execution_allowed", serde_json::json!(true)),
            ("quality_proof", serde_json::json!(true)),
            ("schema_version", serde_json::json!(2)),
            ("unknown_key", serde_json::json!(false)),
        ] {
            let mut value: serde_json::Value = serde_json::from_slice(BYTES).unwrap();
            value[key] = changed;
            assert!(Contract::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        }
    }
}
