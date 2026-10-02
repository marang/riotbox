//! Closed version selection and frozen V1 PCM controls; no runtime tuning knobs.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::Case;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CalibrationVersion {
    V1,
    V2,
}

const V1_REPORT_SHA256: &str = "7f8cecb384357b2ea725ef916707320c51299f1d26adf577df2e644b4de206d8";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct HistoricalControls {
    pub(super) pre: String,
    pub(super) stress_2x: [String; 3],
}

impl HistoricalControls {
    pub(super) fn registered(case: Case) -> Self {
        let (pre, stress_2x) = match case {
            Case::Dense => (
                "4a0f95a42f482e5f655661499c7e46cf4696f6c772c80faa594aca8c0f082036",
                "ca217edf442f8a43cb69dbf9e9315dae41b89789bce1296340484c0b3bffbeb5",
            ),
            Case::Tonal => (
                "40e6af452477f4a29a2338dccc73face9b3d4aba743c49e940ffad726455deae",
                "7da1d98a6a274eda80d847ceeef16dac978536863c2f2c55920811b3db0cfe02",
            ),
            Case::Sparse => (
                "cbe1d1d7fa9c1457717ab5e0d439c15b31f480a08977c7c67055f7ce9e77f6ef",
                "f50cb7c9fcb2da0c65f255eb298a38a7a87183811c605198e89d8d07ca9953d7",
            ),
        };
        Self {
            pre: pre.into(),
            stress_2x: std::array::from_fn(|_| stress_2x.into()),
        }
    }

    pub(super) fn diagnostics(&self, actual: &Self) -> Value {
        json!({
            "reference_report_sha256": V1_REPORT_SHA256,
            "expected": {"pre_sha256_f32le": self.pre, "stress_2x_sha256_f32le": self.stress_2x},
            "actual": {"pre_sha256_f32le": actual.pre, "stress_2x_sha256_f32le": actual.stress_2x},
        })
    }
}

pub(super) fn pcm_hash(samples: &[f32]) -> String {
    let mut digest = Sha256::new();
    for sample in samples {
        digest.update(sample.to_le_bytes());
    }
    format!("{:x}", digest.finalize())
}
